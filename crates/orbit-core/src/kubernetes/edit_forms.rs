use base64::{engine::general_purpose::STANDARD as BASE64_STANDARD, Engine as _};
use serde_json::{Map, Value};

use crate::kubernetes::raw::{format_raw_resource_yaml, parse_resource_yaml};

const STORAGE_CLASS_DEFAULT_ANNOTATION: &str = "storageclass.kubernetes.io/is-default-class";
const STORAGE_CLASS_BETA_DEFAULT_ANNOTATION: &str =
    "storageclass.beta.kubernetes.io/is-default-class";

const GENERATED_METADATA_FIELDS: &[&str] = &[
    "uid",
    "resourceVersion",
    "generation",
    "creationTimestamp",
    "deletionTimestamp",
    "deletionGracePeriodSeconds",
    "selfLink",
    "managedFields",
    "ownerReferences",
];

const PRESERVED_SERVER_METADATA_FIELDS: &[&str] = &[
    "uid",
    "resourceVersion",
    "generation",
    "creationTimestamp",
    "ownerReferences",
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EditMode {
    Form,
    Yaml,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KeyValueEntry {
    pub key: String,
    pub value: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConfigMapFormValues {
    pub data: Vec<KeyValueEntry>,
    pub labels: Vec<KeyValueEntry>,
    pub annotations: Vec<KeyValueEntry>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SecretFormValues {
    pub secret_type: String,
    pub data: Vec<KeyValueEntry>,
    pub preserved_binary_data: std::collections::BTreeMap<String, String>,
    pub labels: Vec<KeyValueEntry>,
    pub annotations: Vec<KeyValueEntry>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NamespaceFormValues {
    pub name: String,
    pub status_phase: String,
    pub finalizers: Vec<String>,
    pub labels: Vec<KeyValueEntry>,
    pub annotations: Vec<KeyValueEntry>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PvFormValues {
    pub name: String,
    pub capacity: String,
    pub access_modes: Vec<String>,
    pub reclaim_policy: String,
    pub storage_class_name: String,
    pub volume_mode: String,
    pub mount_options: Vec<String>,
    pub finalizers: Vec<String>,
    pub labels: Vec<KeyValueEntry>,
    pub annotations: Vec<KeyValueEntry>,
    pub source_summary: String,
    pub claim_ref_summary: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PvcFormValues {
    pub name: String,
    pub namespace: String,
    pub initial_storage_request: String,
    pub storage_request: String,
    pub access_modes: Vec<String>,
    pub storage_class_name: String,
    pub volume_name: String,
    pub volume_mode: String,
    pub labels: Vec<KeyValueEntry>,
    pub annotations: Vec<KeyValueEntry>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StorageClassFormValues {
    pub name: String,
    pub provisioner: String,
    pub reclaim_policy: String,
    pub volume_binding_mode: String,
    pub is_default_class: bool,
    pub allow_volume_expansion: bool,
    pub mount_options: Vec<String>,
    pub parameters: Vec<KeyValueEntry>,
    pub labels: Vec<KeyValueEntry>,
    pub annotations: Vec<KeyValueEntry>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MetadataFallbackFormValues {
    pub name: String,
    pub namespace: Option<String>,
    pub labels: Vec<KeyValueEntry>,
    pub annotations: Vec<KeyValueEntry>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EditFormModel {
    ConfigMap(ConfigMapFormValues),
    Secret(SecretFormValues),
    Namespace(NamespaceFormValues),
    PersistentVolume(PvFormValues),
    PersistentVolumeClaim(PvcFormValues),
    StorageClass(StorageClassFormValues),
    MetadataFallback(MetadataFallbackFormValues),
}

#[derive(Debug, Clone, PartialEq)]
pub struct EditModeTransition {
    pub mode: EditMode,
    pub raw_data: Value,
    pub yaml_content: String,
    pub form_model: Option<EditFormModel>,
}

fn resolve_storage_multiplier(raw_suffix: &str) -> Option<f64> {
    if raw_suffix.is_empty() {
        return Some(1.0);
    }
    if raw_suffix == "m" {
        return Some(1e-3);
    }

    match raw_suffix.to_ascii_lowercase().as_str() {
        "ki" | "kib" => return Some(1024_f64),
        "mi" | "mib" => return Some(1024_f64.powi(2)),
        "gi" | "gib" => return Some(1024_f64.powi(3)),
        "ti" | "tib" => return Some(1024_f64.powi(4)),
        "pi" | "pib" => return Some(1024_f64.powi(5)),
        "ei" | "eib" => return Some(1024_f64.powi(6)),
        _ => {}
    }

    match raw_suffix {
        "k" | "K" | "kB" | "KB" | "kb" => Some(1e3),
        "M" | "MB" | "mb" => Some(1e6),
        "G" | "GB" | "gb" => Some(1e9),
        "T" | "TB" | "tb" => Some(1e12),
        "P" | "PB" | "pb" => Some(1e15),
        "E" | "EB" | "eb" => Some(1e18),
        _ => None,
    }
}

fn split_storage_quantity(trimmed: &str) -> Option<(&str, &str)> {
    let bytes = trimmed.as_bytes();
    if bytes.is_empty() {
        return None;
    }

    let mut idx = 0;
    if bytes[idx] == b'+' || bytes[idx] == b'-' {
        idx += 1;
    }

    let num_start = idx;
    let mut has_digits = false;
    while idx < bytes.len() && bytes[idx].is_ascii_digit() {
        has_digits = true;
        idx += 1;
    }
    if idx < bytes.len() && bytes[idx] == b'.' {
        idx += 1;
        while idx < bytes.len() && bytes[idx].is_ascii_digit() {
            has_digits = true;
            idx += 1;
        }
    }
    if !has_digits || idx == num_start {
        return None;
    }

    // Optional scientific notation exponent (e.g. 1e9, 2.5E+3)
    if idx < bytes.len() && (bytes[idx] == b'e' || bytes[idx] == b'E') {
        let exp_marker = idx;
        idx += 1;
        if idx < bytes.len() && (bytes[idx] == b'+' || bytes[idx] == b'-') {
            idx += 1;
        }
        let exp_digits_start = idx;
        while idx < bytes.len() && bytes[idx].is_ascii_digit() {
            idx += 1;
        }
        if idx == exp_digits_start {
            // Not an exponent; rewind so 'E'/'Ei' is treated as a unit suffix.
            idx = exp_marker;
        }
    }

    let num_part = &trimmed[..idx];
    let rest = trimmed[idx..].trim_start();
    if !rest.bytes().all(|b| b.is_ascii_alphabetic()) {
        return None;
    }

    Some((num_part, rest))
}

pub fn parse_storage_bytes(value: &str) -> f64 {
    let trimmed = value.trim();
    if trimmed.is_empty() || trimmed == "-" {
        return 0.0;
    }
    let Some((num_str, suffix)) = split_storage_quantity(trimmed) else {
        return 0.0;
    };
    let Ok(num) = num_str.parse::<f64>() else {
        return 0.0;
    };
    if !num.is_finite() || num <= 0.0 {
        return 0.0;
    }
    let Some(multiplier) = resolve_storage_multiplier(suffix) else {
        return 0.0;
    };
    let result = num * multiplier;
    if result.is_finite() && result > 0.0 {
        result
    } else {
        0.0
    }
}

pub fn is_valid_storage_quantity(value: &str) -> bool {
    parse_storage_bytes(value) > 0.0
}

pub fn validate_pvc_storage_expansion(initial_quantity: &str, new_quantity: &str) -> bool {
    let initial_bytes = parse_storage_bytes(initial_quantity);
    let new_bytes = parse_storage_bytes(new_quantity);
    if new_bytes <= 0.0 {
        return false;
    }
    new_bytes >= initial_bytes
}

pub fn decode_secret_utf8(encoded: &str) -> Option<String> {
    let bytes = BASE64_STANDARD.decode(encoded.trim()).ok()?;
    String::from_utf8(bytes).ok()
}

pub fn decode_secret_value(encoded: &str) -> String {
    decode_secret_utf8(encoded).unwrap_or_else(|| encoded.to_string())
}

pub fn encode_secret_value(plain: &str) -> String {
    BASE64_STANDARD.encode(plain.as_bytes())
}

fn value_as_string(v: &Value) -> String {
    match v {
        Value::String(s) => s.clone(),
        Value::Null => String::new(),
        other => other.to_string(),
    }
}

fn map_to_kv_entries(value: Option<&Value>) -> Vec<KeyValueEntry> {
    let Some(map) = value.and_then(Value::as_object) else {
        return Vec::new();
    };
    map.iter()
        .map(|(k, v)| KeyValueEntry {
            key: k.clone(),
            value: value_as_string(v),
        })
        .collect()
}

fn kv_entries_to_map(entries: &[KeyValueEntry]) -> Map<String, Value> {
    let mut map = Map::new();
    for entry in entries {
        let key = entry.key.trim();
        if !key.is_empty() {
            map.insert(key.to_string(), Value::String(entry.value.clone()));
        }
    }
    map
}

fn set_or_remove_map_field(target: &mut Map<String, Value>, field: &str, map: Map<String, Value>) {
    if map.is_empty() {
        target.remove(field);
    } else {
        target.insert(field.to_string(), Value::Object(map));
    }
}

fn value_to_string_vec(value: Option<&Value>) -> Vec<String> {
    value
        .and_then(Value::as_array)
        .map(|arr| {
            arr.iter()
                .filter_map(Value::as_str)
                .map(ToOwned::to_owned)
                .collect()
        })
        .unwrap_or_default()
}

fn clean_string_vec(items: &[String]) -> Vec<Value> {
    items
        .iter()
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .map(|s| Value::String(s.to_string()))
        .collect()
}

fn ensure_object_mut<'a>(parent: &'a mut Map<String, Value>, key: &str) -> &'a mut Map<String, Value> {
    let needs_init = !matches!(parent.get(key), Some(Value::Object(_)));
    if needs_init {
        parent.insert(key.to_string(), Value::Object(Map::new()));
    }
    match parent.get_mut(key) {
        Some(Value::Object(map)) => map,
        _ => unreachable!("key was just initialized as Value::Object"),
    }
}

fn extract_pv_source_summary(spec: Option<&Map<String, Value>>) -> String {
    let Some(spec) = spec else {
        return "Configured volume source".to_string();
    };
    if let Some(hp) = spec.get("hostPath").and_then(Value::as_object) {
        let path = hp.get("path").and_then(Value::as_str).unwrap_or("-");
        let kind = hp.get("type").and_then(Value::as_str).unwrap_or("Any");
        return format!("hostPath: {path} ({kind})");
    }
    if let Some(csi) = spec.get("csi").and_then(Value::as_object) {
        let driver = csi.get("driver").and_then(Value::as_str).unwrap_or("-");
        let handle = csi.get("volumeHandle").and_then(Value::as_str).unwrap_or("-");
        return format!("CSI: {driver} / {handle}");
    }
    if let Some(nfs) = spec.get("nfs").and_then(Value::as_object) {
        let server = nfs.get("server").and_then(Value::as_str).unwrap_or("-");
        let path = nfs.get("path").and_then(Value::as_str).unwrap_or("-");
        return format!("NFS: {server}:{path}");
    }
    if let Some(local) = spec.get("local").and_then(Value::as_object) {
        let path = local.get("path").and_then(Value::as_str).unwrap_or("-");
        return format!("local: {path}");
    }
    "Configured volume source".to_string()
}

fn extract_pv_claim_ref_summary(spec: Option<&Map<String, Value>>) -> String {
    let Some(claim_ref) = spec
        .and_then(|s| s.get("claimRef"))
        .and_then(Value::as_object)
    else {
        return "-".to_string();
    };
    let Some(name) = claim_ref.get("name").and_then(Value::as_str).filter(|s| !s.is_empty()) else {
        return "-".to_string();
    };
    match claim_ref
        .get("namespace")
        .and_then(Value::as_str)
        .filter(|s| !s.is_empty())
    {
        Some(ns) => format!("{ns}/{name}"),
        None => name.to_string(),
    }
}

pub fn extract_edit_form(kind: &str, raw_data: &Value) -> EditFormModel {
    let meta = raw_data.get("metadata").and_then(Value::as_object);
    let spec = raw_data.get("spec").and_then(Value::as_object);

    match kind {
        "ConfigMap" => EditFormModel::ConfigMap(ConfigMapFormValues {
            data: map_to_kv_entries(raw_data.get("data")),
            labels: map_to_kv_entries(meta.and_then(|m| m.get("labels"))),
            annotations: map_to_kv_entries(meta.and_then(|m| m.get("annotations"))),
        }),
        "Secret" => {
            let secret_type = raw_data
                .get("type")
                .and_then(Value::as_str)
                .filter(|s| !s.is_empty())
                .unwrap_or("Opaque")
                .to_string();

            let mut data = Vec::new();
            let mut preserved_binary_data = std::collections::BTreeMap::new();
            if let Some(encoded_map) = raw_data.get("data").and_then(Value::as_object) {
                for (k, v) in encoded_map {
                    let raw_encoded = value_as_string(v);
                    if let Some(decoded) = decode_secret_utf8(&raw_encoded) {
                        data.push(KeyValueEntry {
                            key: k.clone(),
                            value: decoded,
                        });
                    } else {
                        preserved_binary_data.insert(k.clone(), raw_encoded);
                    }
                }
            }
            if let Some(string_data) = raw_data.get("stringData").and_then(Value::as_object) {
                for (k, v) in string_data {
                    let plain = value_as_string(v);
                    preserved_binary_data.remove(k);
                    if let Some(existing) = data.iter_mut().find(|entry| entry.key == *k) {
                        existing.value = plain;
                    } else {
                        data.push(KeyValueEntry {
                            key: k.clone(),
                            value: plain,
                        });
                    }
                }
            }

            EditFormModel::Secret(SecretFormValues {
                secret_type,
                data,
                preserved_binary_data,
                labels: map_to_kv_entries(meta.and_then(|m| m.get("labels"))),
                annotations: map_to_kv_entries(meta.and_then(|m| m.get("annotations"))),
            })
        }
        "Namespace" => {
            let name = meta
                .and_then(|m| m.get("name"))
                .and_then(Value::as_str)
                .unwrap_or_default()
                .to_string();
            let status_phase = raw_data
                .get("status")
                .and_then(|s| s.get("phase"))
                .and_then(Value::as_str)
                .filter(|s| !s.is_empty())
                .unwrap_or("Active")
                .to_string();
            EditFormModel::Namespace(NamespaceFormValues {
                name,
                status_phase,
                finalizers: value_to_string_vec(spec.and_then(|s| s.get("finalizers"))),
                labels: map_to_kv_entries(meta.and_then(|m| m.get("labels"))),
                annotations: map_to_kv_entries(meta.and_then(|m| m.get("annotations"))),
            })
        }
        "PersistentVolume" => {
            let mut access_modes = value_to_string_vec(spec.and_then(|s| s.get("accessModes")));
            if access_modes.is_empty() {
                access_modes.push("ReadWriteOnce".to_string());
            }
            let capacity = spec
                .and_then(|s| s.get("capacity"))
                .and_then(|c| c.get("storage"))
                .and_then(Value::as_str)
                .filter(|s| !s.is_empty())
                .unwrap_or("10Gi")
                .to_string();
            EditFormModel::PersistentVolume(PvFormValues {
                name: meta
                    .and_then(|m| m.get("name"))
                    .and_then(Value::as_str)
                    .unwrap_or_default()
                    .to_string(),
                capacity,
                access_modes,
                reclaim_policy: spec
                    .and_then(|s| s.get("persistentVolumeReclaimPolicy"))
                    .and_then(Value::as_str)
                    .filter(|s| !s.is_empty())
                    .unwrap_or("Retain")
                    .to_string(),
                storage_class_name: spec
                    .and_then(|s| s.get("storageClassName"))
                    .and_then(Value::as_str)
                    .unwrap_or_default()
                    .to_string(),
                volume_mode: spec
                    .and_then(|s| s.get("volumeMode"))
                    .and_then(Value::as_str)
                    .filter(|s| !s.is_empty())
                    .unwrap_or("Filesystem")
                    .to_string(),
                mount_options: value_to_string_vec(spec.and_then(|s| s.get("mountOptions"))),
                finalizers: value_to_string_vec(meta.and_then(|m| m.get("finalizers"))),
                labels: map_to_kv_entries(meta.and_then(|m| m.get("labels"))),
                annotations: map_to_kv_entries(meta.and_then(|m| m.get("annotations"))),
                source_summary: extract_pv_source_summary(spec),
                claim_ref_summary: extract_pv_claim_ref_summary(spec),
            })
        }
        "PersistentVolumeClaim" => {
            let mut access_modes = value_to_string_vec(spec.and_then(|s| s.get("accessModes")));
            if access_modes.is_empty() {
                access_modes.push("ReadWriteOnce".to_string());
            }
            let storage_request = spec
                .and_then(|s| s.get("resources"))
                .and_then(|r| r.get("requests"))
                .and_then(|req| req.get("storage"))
                .and_then(Value::as_str)
                .filter(|s| !s.is_empty())
                .unwrap_or("10Gi")
                .to_string();
            EditFormModel::PersistentVolumeClaim(PvcFormValues {
                name: meta
                    .and_then(|m| m.get("name"))
                    .and_then(Value::as_str)
                    .unwrap_or_default()
                    .to_string(),
                namespace: meta
                    .and_then(|m| m.get("namespace"))
                    .and_then(Value::as_str)
                    .filter(|s| !s.is_empty())
                    .unwrap_or("default")
                    .to_string(),
                initial_storage_request: storage_request.clone(),
                storage_request,
                access_modes,
                storage_class_name: spec
                    .and_then(|s| s.get("storageClassName"))
                    .and_then(Value::as_str)
                    .unwrap_or_default()
                    .to_string(),
                volume_name: spec
                    .and_then(|s| s.get("volumeName"))
                    .and_then(Value::as_str)
                    .unwrap_or_default()
                    .to_string(),
                volume_mode: spec
                    .and_then(|s| s.get("volumeMode"))
                    .and_then(Value::as_str)
                    .filter(|s| !s.is_empty())
                    .unwrap_or("Filesystem")
                    .to_string(),
                labels: map_to_kv_entries(meta.and_then(|m| m.get("labels"))),
                annotations: map_to_kv_entries(meta.and_then(|m| m.get("annotations"))),
            })
        }
        "StorageClass" => {
            let annotations_map = meta
                .and_then(|m| m.get("annotations"))
                .and_then(Value::as_object);
            let is_default_class = annotations_map
                .and_then(|a| a.get(STORAGE_CLASS_DEFAULT_ANNOTATION))
                .and_then(Value::as_str)
                == Some("true")
                || annotations_map
                    .and_then(|a| a.get(STORAGE_CLASS_BETA_DEFAULT_ANNOTATION))
                    .and_then(Value::as_str)
                    == Some("true");
            let annotations = map_to_kv_entries(meta.and_then(|m| m.get("annotations")))
                .into_iter()
                .filter(|entry| {
                    entry.key != STORAGE_CLASS_DEFAULT_ANNOTATION
                        && entry.key != STORAGE_CLASS_BETA_DEFAULT_ANNOTATION
                })
                .collect();

            EditFormModel::StorageClass(StorageClassFormValues {
                name: meta
                    .and_then(|m| m.get("name"))
                    .and_then(Value::as_str)
                    .unwrap_or_default()
                    .to_string(),
                provisioner: raw_data
                    .get("provisioner")
                    .and_then(Value::as_str)
                    .unwrap_or_default()
                    .to_string(),
                reclaim_policy: raw_data
                    .get("reclaimPolicy")
                    .and_then(Value::as_str)
                    .filter(|s| !s.is_empty())
                    .unwrap_or("Delete")
                    .to_string(),
                volume_binding_mode: raw_data
                    .get("volumeBindingMode")
                    .and_then(Value::as_str)
                    .filter(|s| !s.is_empty())
                    .unwrap_or("Immediate")
                    .to_string(),
                is_default_class,
                allow_volume_expansion: raw_data
                    .get("allowVolumeExpansion")
                    .and_then(Value::as_bool)
                    .unwrap_or(false),
                mount_options: value_to_string_vec(raw_data.get("mountOptions")),
                parameters: map_to_kv_entries(raw_data.get("parameters")),
                labels: map_to_kv_entries(meta.and_then(|m| m.get("labels"))),
                annotations,
            })
        }
        _ => EditFormModel::MetadataFallback(MetadataFallbackFormValues {
            name: meta
                .and_then(|m| m.get("name"))
                .and_then(Value::as_str)
                .unwrap_or_default()
                .to_string(),
            namespace: meta
                .and_then(|m| m.get("namespace"))
                .and_then(Value::as_str)
                .filter(|s| !s.is_empty())
                .map(ToOwned::to_owned),
            labels: map_to_kv_entries(meta.and_then(|m| m.get("labels"))),
            annotations: map_to_kv_entries(meta.and_then(|m| m.get("annotations"))),
        }),
    }
}

pub fn apply_edit_form(raw_data: &Value, form: &EditFormModel) -> Value {
    let mut root = match raw_data.as_object() {
        Some(map) => map.clone(),
        None => Map::new(),
    };

    match form {
        EditFormModel::ConfigMap(cm) => {
            let meta = ensure_object_mut(&mut root, "metadata");
            set_or_remove_map_field(meta, "labels", kv_entries_to_map(&cm.labels));
            set_or_remove_map_field(meta, "annotations", kv_entries_to_map(&cm.annotations));
            set_or_remove_map_field(&mut root, "data", kv_entries_to_map(&cm.data));
        }
        EditFormModel::Secret(secret) => {
            let meta = ensure_object_mut(&mut root, "metadata");
            set_or_remove_map_field(meta, "labels", kv_entries_to_map(&secret.labels));
            set_or_remove_map_field(meta, "annotations", kv_entries_to_map(&secret.annotations));
            root.insert("type".to_string(), Value::String(secret.secret_type.clone()));

            let mut encoded_map = Map::new();
            for (k, raw_encoded) in &secret.preserved_binary_data {
                encoded_map.insert(k.clone(), Value::String(raw_encoded.clone()));
            }
            for entry in &secret.data {
                let key = entry.key.trim();
                if !key.is_empty() {
                    encoded_map.insert(
                        key.to_string(),
                        Value::String(encode_secret_value(&entry.value)),
                    );
                }
            }
            set_or_remove_map_field(&mut root, "data", encoded_map);
            root.remove("stringData");
        }
        EditFormModel::Namespace(ns) => {
            let meta = ensure_object_mut(&mut root, "metadata");
            set_or_remove_map_field(meta, "labels", kv_entries_to_map(&ns.labels));
            set_or_remove_map_field(meta, "annotations", kv_entries_to_map(&ns.annotations));

            let clean_finalizers = clean_string_vec(&ns.finalizers);
            if !clean_finalizers.is_empty() {
                let spec = ensure_object_mut(&mut root, "spec");
                spec.insert("finalizers".to_string(), Value::Array(clean_finalizers));
            } else if let Some(Value::Object(spec)) = root.get_mut("spec") {
                spec.remove("finalizers");
                if spec.is_empty() {
                    root.remove("spec");
                }
            }
        }
        EditFormModel::PersistentVolume(pv) => {
            let meta = ensure_object_mut(&mut root, "metadata");
            if !pv.name.trim().is_empty() {
                meta.insert("name".to_string(), Value::String(pv.name.trim().to_string()));
            }
            meta.remove("namespace");
            set_or_remove_map_field(meta, "labels", kv_entries_to_map(&pv.labels));
            set_or_remove_map_field(meta, "annotations", kv_entries_to_map(&pv.annotations));
            let clean_finalizers = clean_string_vec(&pv.finalizers);
            if clean_finalizers.is_empty() {
                meta.remove("finalizers");
            } else {
                meta.insert("finalizers".to_string(), Value::Array(clean_finalizers));
            }

            let spec = ensure_object_mut(&mut root, "spec");
            let cap = ensure_object_mut(spec, "capacity");
            cap.insert(
                "storage".to_string(),
                Value::String(pv.capacity.trim().to_string()),
            );
            spec.insert(
                "accessModes".to_string(),
                Value::Array(
                    pv.access_modes
                        .iter()
                        .map(|m| Value::String(m.clone()))
                        .collect(),
                ),
            );
            spec.insert(
                "persistentVolumeReclaimPolicy".to_string(),
                Value::String(pv.reclaim_policy.clone()),
            );
            if pv.storage_class_name.trim().is_empty() {
                spec.remove("storageClassName");
            } else {
                spec.insert(
                    "storageClassName".to_string(),
                    Value::String(pv.storage_class_name.trim().to_string()),
                );
            }
            let clean_mount_options = clean_string_vec(&pv.mount_options);
            if clean_mount_options.is_empty() {
                spec.remove("mountOptions");
            } else {
                spec.insert("mountOptions".to_string(), Value::Array(clean_mount_options));
            }
        }
        EditFormModel::PersistentVolumeClaim(pvc) => {
            let meta = ensure_object_mut(&mut root, "metadata");
            if !pvc.name.trim().is_empty() {
                meta.insert("name".to_string(), Value::String(pvc.name.trim().to_string()));
            }
            if !pvc.namespace.trim().is_empty() {
                meta.insert(
                    "namespace".to_string(),
                    Value::String(pvc.namespace.trim().to_string()),
                );
            }
            set_or_remove_map_field(meta, "labels", kv_entries_to_map(&pvc.labels));
            set_or_remove_map_field(meta, "annotations", kv_entries_to_map(&pvc.annotations));

            let spec = ensure_object_mut(&mut root, "spec");
            let resources = ensure_object_mut(spec, "resources");
            let requests = ensure_object_mut(resources, "requests");
            requests.insert(
                "storage".to_string(),
                Value::String(pvc.storage_request.trim().to_string()),
            );
        }
        EditFormModel::StorageClass(sc) => {
            let meta = ensure_object_mut(&mut root, "metadata");
            if !sc.name.trim().is_empty() {
                meta.insert("name".to_string(), Value::String(sc.name.trim().to_string()));
            }
            meta.remove("namespace");
            set_or_remove_map_field(meta, "labels", kv_entries_to_map(&sc.labels));

            let mut annotations = kv_entries_to_map(&sc.annotations);
            if sc.is_default_class {
                annotations.insert(
                    STORAGE_CLASS_DEFAULT_ANNOTATION.to_string(),
                    Value::String("true".to_string()),
                );
            } else {
                annotations.remove(STORAGE_CLASS_DEFAULT_ANNOTATION);
                annotations.remove(STORAGE_CLASS_BETA_DEFAULT_ANNOTATION);
            }
            set_or_remove_map_field(meta, "annotations", annotations);

            root.insert(
                "allowVolumeExpansion".to_string(),
                Value::Bool(sc.allow_volume_expansion),
            );
            let clean_mount_options = clean_string_vec(&sc.mount_options);
            if clean_mount_options.is_empty() {
                root.remove("mountOptions");
            } else {
                root.insert("mountOptions".to_string(), Value::Array(clean_mount_options));
            }
        }
        EditFormModel::MetadataFallback(fallback) => {
            let meta = ensure_object_mut(&mut root, "metadata");
            if !fallback.name.trim().is_empty() {
                meta.insert(
                    "name".to_string(),
                    Value::String(fallback.name.trim().to_string()),
                );
            }
            match fallback.namespace.as_deref().map(str::trim).filter(|s| !s.is_empty()) {
                Some(ns) => {
                    meta.insert("namespace".to_string(), Value::String(ns.to_string()));
                }
                None => {
                    meta.remove("namespace");
                }
            }
            set_or_remove_map_field(meta, "labels", kv_entries_to_map(&fallback.labels));
            set_or_remove_map_field(
                meta,
                "annotations",
                kv_entries_to_map(&fallback.annotations),
            );
        }
    }

    Value::Object(root)
}

pub fn validate_edit_form(form: &EditFormModel) -> Option<String> {
    match form {
        EditFormModel::Secret(secret) => {
            if secret.secret_type.trim().is_empty() {
                Some("Secret type is required.".to_string())
            } else {
                None
            }
        }
        EditFormModel::PersistentVolume(pv) => {
            if !is_valid_storage_quantity(&pv.capacity) {
                Some("Capacity must be a valid positive Kubernetes storage quantity (e.g. 10Gi, 500Mi).".to_string())
            } else if pv.access_modes.is_empty() {
                Some("At least one access mode must be selected.".to_string())
            } else {
                None
            }
        }
        EditFormModel::PersistentVolumeClaim(pvc) => {
            if !is_valid_storage_quantity(&pvc.storage_request) {
                Some("Storage request must be a valid positive Kubernetes storage quantity (e.g. 10Gi, 500Mi).".to_string())
            } else if !validate_pvc_storage_expansion(
                &pvc.initial_storage_request,
                &pvc.storage_request,
            ) {
                Some(format!(
                    "PVC storage capacity cannot be shrunk below {}.",
                    pvc.initial_storage_request
                ))
            } else {
                None
            }
        }
        EditFormModel::ConfigMap(_)
        | EditFormModel::Namespace(_)
        | EditFormModel::StorageClass(_)
        | EditFormModel::MetadataFallback(_) => None,
    }
}

pub fn strip_generated_resource_fields(raw_data: &Value) -> Value {
    let Some(map) = raw_data.as_object() else {
        return raw_data.clone();
    };
    let mut cleaned = map.clone();
    cleaned.remove("status");

    if let Some(Value::Object(meta)) = cleaned.get_mut("metadata") {
        for field in GENERATED_METADATA_FIELDS {
            meta.remove(*field);
        }
    }

    Value::Object(cleaned)
}

pub fn merge_preserved_server_fields(edited: &Value, server_snapshot: &Value) -> Value {
    let Some(edited_map) = edited.as_object() else {
        return edited.clone();
    };
    let Some(server_map) = server_snapshot.as_object() else {
        return edited.clone();
    };

    let mut merged = edited_map.clone();
    if !merged.contains_key("status")
        && let Some(status) = server_map.get("status")
    {
        merged.insert("status".to_string(), status.clone());
    }

    if let Some(server_meta) = server_map.get("metadata").and_then(Value::as_object) {
        let target_meta = ensure_object_mut(&mut merged, "metadata");
        for field in PRESERVED_SERVER_METADATA_FIELDS {
            if !target_meta.contains_key(*field)
                && let Some(val) = server_meta.get(*field)
            {
                target_meta.insert((*field).to_string(), val.clone());
            }
        }
    }

    Value::Object(merged)
}

pub fn format_editable_resource_yaml(raw_data: &Value) -> Result<String, String> {
    format_raw_resource_yaml(&strip_generated_resource_fields(raw_data))
}

pub fn transition_edit_mode(
    target_mode: EditMode,
    kind: &str,
    raw_data: &Value,
    yaml_content: &str,
) -> Result<EditModeTransition, String> {
    match target_mode {
        EditMode::Yaml => {
            let next_yaml = format_editable_resource_yaml(raw_data)?;
            Ok(EditModeTransition {
                mode: EditMode::Yaml,
                raw_data: raw_data.clone(),
                yaml_content: next_yaml,
                form_model: None,
            })
        }
        EditMode::Form => {
            let parsed = parse_resource_yaml(yaml_content)?;
            let merged = merge_preserved_server_fields(&parsed, raw_data);
            let form_model = extract_edit_form(kind, &merged);
            Ok(EditModeTransition {
                mode: EditMode::Form,
                raw_data: merged,
                yaml_content: yaml_content.to_string(),
                form_model: Some(form_model),
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_storage_quantity_parsing_and_pvc_expansion_only_validation() {
        assert_eq!(parse_storage_bytes("1Ki"), 1024.0);
        assert_eq!(parse_storage_bytes("2Mi"), 2.0 * 1024.0 * 1024.0);
        assert_eq!(parse_storage_bytes("10Gi"), 10.0 * 1024.0_f64.powi(3));
        assert_eq!(parse_storage_bytes("1Ti"), 1024.0_f64.powi(4));
        assert_eq!(parse_storage_bytes("5k"), 5000.0);
        assert_eq!(parse_storage_bytes("2M"), 2_000_000.0);
        assert_eq!(parse_storage_bytes("3G"), 3_000_000_000.0);

        assert!(is_valid_storage_quantity("10Gi"));
        assert!(is_valid_storage_quantity("512Mi"));
        assert!(!is_valid_storage_quantity(""));
        assert!(!is_valid_storage_quantity("0Gi"));
        assert!(!is_valid_storage_quantity("-5Gi"));
        assert!(!is_valid_storage_quantity("10LightYears"));

        assert!(validate_pvc_storage_expansion("10Gi", "10Gi"));
        assert!(validate_pvc_storage_expansion("10Gi", "20Gi"));
        assert!(!validate_pvc_storage_expansion("10Gi", "5Gi"));
        assert!(!validate_pvc_storage_expansion("10Gi", "invalid"));
    }

    #[test]
    fn test_metadata_fallback_extract_apply_preserves_unmanaged_fields_and_cluster_scope() {
        let cluster_node = json!({
            "apiVersion": "v1",
            "kind": "Node",
            "metadata": {
                "name": "worker-1",
                "resourceVersion": "999",
                "labels": { "role": "worker" }
            },
            "spec": { "unschedulable": true },
            "status": { "phase": "Running" }
        });

        let mut form = extract_edit_form("Node", &cluster_node);
        match &mut form {
            EditFormModel::MetadataFallback(meta) => {
                assert_eq!(meta.name, "worker-1");
                assert_eq!(meta.namespace, None);
                meta.labels.push(KeyValueEntry {
                    key: "zone".to_string(),
                    value: "us-east-1a".to_string(),
                });
            }
            other => panic!("expected MetadataFallback, got {other:?}"),
        }

        let updated = apply_edit_form(&cluster_node, &form);
        assert_eq!(updated["metadata"]["resourceVersion"], "999");
        assert!(updated["metadata"].get("namespace").is_none());
        assert_eq!(updated["metadata"]["labels"]["zone"], "us-east-1a");
        assert_eq!(updated["spec"]["unschedulable"], true);
        assert_eq!(updated["status"]["phase"], "Running");
    }

    #[test]
    fn test_configmap_form_roundtrip_updates_data_labels_annotations() {
        let cm = json!({
            "apiVersion": "v1",
            "kind": "ConfigMap",
            "metadata": {
                "name": "app-cfg",
                "namespace": "default",
                "resourceVersion": "12",
                "labels": { "app": "web" }
            },
            "data": { "LOG_LEVEL": "info" }
        });

        let mut form = extract_edit_form("ConfigMap", &cm);
        match &mut form {
            EditFormModel::ConfigMap(values) => {
                assert_eq!(values.data.len(), 1);
                values.data[0].value = "debug".to_string();
                values.annotations.push(KeyValueEntry {
                    key: "owner".to_string(),
                    value: "platform".to_string(),
                });
            }
            other => panic!("expected ConfigMap, got {other:?}"),
        }

        let updated = apply_edit_form(&cm, &form);
        assert_eq!(updated["metadata"]["resourceVersion"], "12");
        assert_eq!(updated["data"]["LOG_LEVEL"], "debug");
        assert_eq!(updated["metadata"]["annotations"]["owner"], "platform");
    }

    #[test]
    fn test_secret_form_decodes_base64_merges_string_data_and_reencodes_on_apply() {
        let secret = json!({
            "apiVersion": "v1",
            "kind": "Secret",
            "type": "Opaque",
            "metadata": {
                "name": "db-creds",
                "namespace": "default"
            },
            "data": {
                "username": BASE64_STANDARD.encode("admin"),
                "password": BASE64_STANDARD.encode("old-pass")
            },
            "stringData": {
                "password": "new-plain-password",
                "host": "db.internal"
            }
        });

        let mut form = extract_edit_form("Secret", &secret);
        match &mut form {
            EditFormModel::Secret(values) => {
                assert_eq!(values.secret_type, "Opaque");
                let username = values.data.iter().find(|e| e.key == "username").expect("username");
                assert_eq!(username.value, "admin");
                let password = values.data.iter().find(|e| e.key == "password").expect("password");
                assert_eq!(password.value, "new-plain-password");
                let host = values.data.iter().find(|e| e.key == "host").expect("host");
                assert_eq!(host.value, "db.internal");
                values.secret_type = "kubernetes.io/basic-auth".to_string();
            }
            other => panic!("expected Secret, got {other:?}"),
        }

        let updated = apply_edit_form(&secret, &form);
        assert_eq!(updated["type"], "kubernetes.io/basic-auth");
        assert!(updated.get("stringData").is_none());
        assert_eq!(
            updated["data"]["password"],
            BASE64_STANDARD.encode("new-plain-password")
        );
        assert_eq!(updated["data"]["host"], BASE64_STANDARD.encode("db.internal"));
    }

    #[test]
    fn test_secret_form_preserves_binary_non_utf8_entries_without_double_encoding() {
        let binary_b64 = BASE64_STANDARD.encode([0xff_u8, 0xfe, 0x00, 0x01, 0x80]);
        let secret = json!({
            "apiVersion": "v1",
            "kind": "Secret",
            "type": "kubernetes.io/tls",
            "metadata": {
                "name": "tls-secret",
                "namespace": "default"
            },
            "data": {
                "cert.der": binary_b64,
                "note": BASE64_STANDARD.encode("hello")
            }
        });

        let mut form = extract_edit_form("Secret", &secret);
        match &mut form {
            EditFormModel::Secret(values) => {
                assert_eq!(values.data.len(), 1);
                assert_eq!(values.data[0].key, "note");
                assert_eq!(values.preserved_binary_data.get("cert.der"), Some(&binary_b64));
                values.labels.push(KeyValueEntry {
                    key: "env".to_string(),
                    value: "prod".to_string(),
                });
            }
            other => panic!("expected Secret, got {other:?}"),
        }

        let updated = apply_edit_form(&secret, &form);
        assert_eq!(updated["data"]["cert.der"], binary_b64);
        assert_eq!(updated["data"]["note"], BASE64_STANDARD.encode("hello"));
        assert_eq!(updated["metadata"]["labels"]["env"], "prod");
    }

    #[test]
    fn test_namespace_form_roundtrip_updates_finalizers_and_removes_empty_spec() {
        let ns = json!({
            "apiVersion": "v1",
            "kind": "Namespace",
            "metadata": { "name": "dev" },
            "spec": { "finalizers": ["kubernetes"] },
            "status": { "phase": "Active" }
        });

        let mut form = extract_edit_form("Namespace", &ns);
        match &mut form {
            EditFormModel::Namespace(values) => {
                assert_eq!(values.name, "dev");
                assert_eq!(values.status_phase, "Active");
                assert_eq!(values.finalizers, vec!["kubernetes"]);
                values.finalizers.clear();
            }
            other => panic!("expected Namespace, got {other:?}"),
        }

        let updated = apply_edit_form(&ns, &form);
        assert!(updated.get("spec").is_none(), "empty spec should be removed");
        assert_eq!(updated["status"]["phase"], "Active");
    }

    #[test]
    fn test_pv_pvc_and_storage_class_forms_enforce_mutability_and_default_class_annotation() {
        // PersistentVolume
        let pv = json!({
            "apiVersion": "v1",
            "kind": "PersistentVolume",
            "metadata": { "name": "pv-1", "namespace": "should-be-stripped" },
            "spec": {
                "capacity": { "storage": "10Gi" },
                "accessModes": ["ReadWriteOnce"],
                "persistentVolumeReclaimPolicy": "Retain",
                "hostPath": { "path": "/data" }
            }
        });
        let mut pv_form = extract_edit_form("PersistentVolume", &pv);
        match &mut pv_form {
            EditFormModel::PersistentVolume(v) => {
                assert!(v.source_summary.contains("/data"));
                v.capacity = "20Gi".to_string();
                v.reclaim_policy = "Delete".to_string();
            }
            other => panic!("expected PV, got {other:?}"),
        }
        let pv_updated = apply_edit_form(&pv, &pv_form);
        assert!(pv_updated["metadata"].get("namespace").is_none());
        assert_eq!(pv_updated["spec"]["capacity"]["storage"], "20Gi");
        assert_eq!(pv_updated["spec"]["persistentVolumeReclaimPolicy"], "Delete");
        assert_eq!(pv_updated["spec"]["hostPath"]["path"], "/data");

        // PersistentVolumeClaim
        let pvc = json!({
            "apiVersion": "v1",
            "kind": "PersistentVolumeClaim",
            "metadata": { "name": "pvc-1", "namespace": "default" },
            "spec": {
                "accessModes": ["ReadWriteOnce"],
                "storageClassName": "gp3",
                "resources": { "requests": { "storage": "10Gi" } }
            }
        });
        let mut pvc_form = extract_edit_form("PersistentVolumeClaim", &pvc);
        match &mut pvc_form {
            EditFormModel::PersistentVolumeClaim(v) => {
                v.storage_request = "5Gi".to_string();
            }
            other => panic!("expected PVC, got {other:?}"),
        }
        assert!(validate_edit_form(&pvc_form).is_some(), "shrinking PVC must fail validation");
        if let EditFormModel::PersistentVolumeClaim(v) = &mut pvc_form {
            v.storage_request = "15Gi".to_string();
        }
        assert!(validate_edit_form(&pvc_form).is_none());
        let pvc_updated = apply_edit_form(&pvc, &pvc_form);
        assert_eq!(pvc_updated["spec"]["resources"]["requests"]["storage"], "15Gi");
        assert_eq!(pvc_updated["spec"]["storageClassName"], "gp3");

        // StorageClass
        let sc = json!({
            "apiVersion": "storage.k8s.io/v1",
            "kind": "StorageClass",
            "metadata": {
                "name": "fast",
                "annotations": {
                    "storageclass.kubernetes.io/is-default-class": "true",
                    "custom": "keep"
                }
            },
            "provisioner": "ebs.csi.aws.com",
            "allowVolumeExpansion": false
        });
        let mut sc_form = extract_edit_form("StorageClass", &sc);
        match &mut sc_form {
            EditFormModel::StorageClass(v) => {
                assert!(v.is_default_class);
                assert_eq!(v.annotations.len(), 1);
                assert_eq!(v.annotations[0].key, "custom");
                v.is_default_class = false;
                v.allow_volume_expansion = true;
                v.mount_options = vec!["noatime".to_string()];
            }
            other => panic!("expected StorageClass, got {other:?}"),
        }
        let sc_updated = apply_edit_form(&sc, &sc_form);
        assert!(sc_updated.get("spec").is_none());
        assert_eq!(sc_updated["allowVolumeExpansion"], true);
        assert_eq!(sc_updated["mountOptions"][0], "noatime");
        assert!(sc_updated["metadata"]["annotations"]
            .get(STORAGE_CLASS_DEFAULT_ANNOTATION)
            .is_none());
        assert_eq!(sc_updated["metadata"]["annotations"]["custom"], "keep");
    }

    #[test]
    fn test_transition_edit_mode_blocks_invalid_yaml_and_syncs_valid_yaml_to_form() {
        let cm = json!({
            "apiVersion": "v1",
            "kind": "ConfigMap",
            "metadata": { "name": "cfg", "namespace": "default" },
            "data": { "k": "v1" }
        });

        let to_yaml = transition_edit_mode(EditMode::Yaml, "ConfigMap", &cm, "")
            .expect("form to yaml should succeed");
        assert_eq!(to_yaml.mode, EditMode::Yaml);
        assert!(to_yaml.yaml_content.contains("k: v1"));

        let invalid = transition_edit_mode(EditMode::Form, "ConfigMap", &cm, "data: [broken");
        assert!(invalid.is_err());

        let edited_yaml = "apiVersion: v1\nkind: ConfigMap\nmetadata:\n  name: cfg\n  namespace: default\n  resourceVersion: '77'\ndata:\n  k: v2\n";
        let to_form = transition_edit_mode(EditMode::Form, "ConfigMap", &cm, edited_yaml)
            .expect("valid yaml to form should succeed");
        assert_eq!(to_form.mode, EditMode::Form);
        assert_eq!(to_form.raw_data["metadata"]["resourceVersion"], "77");
        match to_form.form_model {
            Some(EditFormModel::ConfigMap(v)) => {
                assert_eq!(v.data[0].value, "v2");
            }
            other => panic!("expected ConfigMap form model, got {other:?}"),
        }
    }

    #[test]
    fn test_strip_generated_fields_hides_server_metadata_and_status_while_merge_restores_them() {
        let full = json!({
            "apiVersion": "v1",
            "kind": "ConfigMap",
            "metadata": {
                "name": "cfg",
                "namespace": "default",
                "uid": "abc-123",
                "resourceVersion": "42",
                "generation": 3,
                "creationTimestamp": "2026-01-01T00:00:00Z",
                "deletionTimestamp": "2026-01-02T00:00:00Z",
                "deletionGracePeriodSeconds": 30,
                "selfLink": "/api/v1/namespaces/default/configmaps/cfg",
                "managedFields": [{ "manager": "kubectl" }],
                "ownerReferences": [{ "kind": "Deployment", "name": "web" }],
                "labels": { "app": "web" }
            },
            "data": { "k": "v1" },
            "status": { "phase": "Active" }
        });

        let editable_yaml = format_editable_resource_yaml(&full).expect("editable yaml");
        assert!(editable_yaml.contains("name: cfg"));
        assert!(editable_yaml.contains("app: web"));
        assert!(editable_yaml.contains("k: v1"));
        for hidden in [
            "uid:",
            "resourceVersion:",
            "generation:",
            "creationTimestamp:",
            "deletionTimestamp:",
            "deletionGracePeriodSeconds:",
            "selfLink:",
            "managedFields:",
            "ownerReferences:",
            "status:",
        ] {
            assert!(
                !editable_yaml.contains(hidden),
                "generated field `{hidden}` must be hidden from editable YAML:\n{editable_yaml}"
            );
        }

        let edited = parse_resource_yaml(&editable_yaml.replace("k: v1", "k: v2"))
            .expect("parse edited yaml");
        let merged = merge_preserved_server_fields(&edited, &full);
        assert_eq!(merged["data"]["k"], "v2");
        assert_eq!(merged["metadata"]["resourceVersion"], "42");
        assert_eq!(merged["metadata"]["uid"], "abc-123");
        assert_eq!(merged["metadata"]["ownerReferences"][0]["name"], "web");
        assert_eq!(merged["status"]["phase"], "Active");
    }
}
