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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum EnvSourceType {
    #[default]
    Literal,
    ConfigMapKey,
    SecretKey,
    FieldRef,
    ResourceFieldRef,
    ConfigMapEnvFrom,
    SecretEnvFrom,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ContainerEnvEntry {
    pub source_type: EnvSourceType,
    pub name: String,
    pub value: String,
    pub ref_name: String,
    pub ref_key: String,
    pub field_path: String,
    pub prefix: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContainerPortEntry {
    pub name: String,
    pub container_port: String,
    pub protocol: String,
}

impl Default for ContainerPortEntry {
    fn default() -> Self {
        Self {
            name: String::new(),
            container_port: String::new(),
            protocol: "TCP".to_string(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ContainerResourcesValues {
    pub cpu_request: String,
    pub memory_request: String,
    pub cpu_limit: String,
    pub memory_limit: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContainerFormValues {
    pub name: String,
    pub image: String,
    pub image_pull_policy: String,
    pub working_dir: String,
    pub command: Vec<String>,
    pub args: Vec<String>,
    pub resources: ContainerResourcesValues,
    pub environments: Vec<ContainerEnvEntry>,
    pub ports: Vec<ContainerPortEntry>,
}

impl Default for ContainerFormValues {
    fn default() -> Self {
        Self {
            name: String::new(),
            image: String::new(),
            image_pull_policy: "IfNotPresent".to_string(),
            working_dir: String::new(),
            command: Vec::new(),
            args: Vec::new(),
            resources: ContainerResourcesValues::default(),
            environments: Vec::new(),
            ports: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PodSpecFormValues {
    pub service_account_name: String,
    pub restart_policy: String,
    pub termination_grace_period_seconds: String,
    pub dns_policy: String,
    pub node_selector: Vec<KeyValueEntry>,
    pub host_network: bool,
    pub host_pid: bool,
    pub host_ipc: bool,
    pub containers: Vec<ContainerFormValues>,
}

impl Default for PodSpecFormValues {
    fn default() -> Self {
        Self {
            service_account_name: String::new(),
            restart_policy: "Always".to_string(),
            termination_grace_period_seconds: "30".to_string(),
            dns_policy: "ClusterFirst".to_string(),
            node_selector: Vec::new(),
            host_network: false,
            host_pid: false,
            host_ipc: false,
            containers: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeploymentFormValues {
    pub replicas: String,
    pub paused: bool,
    pub strategy_type: String,
    pub max_surge: String,
    pub max_unavailable: String,
    pub min_ready_seconds: String,
    pub revision_history_limit: String,
    pub progress_deadline_seconds: String,
    pub selector_labels: Vec<KeyValueEntry>,
    pub labels: Vec<KeyValueEntry>,
    pub annotations: Vec<KeyValueEntry>,
    pub pod_labels: Vec<KeyValueEntry>,
    pub pod_annotations: Vec<KeyValueEntry>,
    pub pod_spec: PodSpecFormValues,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StatefulSetFormValues {
    pub replicas: String,
    pub service_name: String,
    pub pod_management_policy: String,
    pub strategy_type: String,
    pub partition: String,
    pub max_unavailable: String,
    pub min_ready_seconds: String,
    pub revision_history_limit: String,
    pub selector_labels: Vec<KeyValueEntry>,
    pub labels: Vec<KeyValueEntry>,
    pub annotations: Vec<KeyValueEntry>,
    pub pod_labels: Vec<KeyValueEntry>,
    pub pod_annotations: Vec<KeyValueEntry>,
    pub pod_spec: PodSpecFormValues,
    pub volume_claim_summaries: Vec<String>,
    pub pvc_retention_when_deleted: String,
    pub pvc_retention_when_scaled: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DaemonSetFormValues {
    pub strategy_type: String,
    pub max_unavailable: String,
    pub max_surge: String,
    pub min_ready_seconds: String,
    pub revision_history_limit: String,
    pub selector_labels: Vec<KeyValueEntry>,
    pub labels: Vec<KeyValueEntry>,
    pub annotations: Vec<KeyValueEntry>,
    pub pod_labels: Vec<KeyValueEntry>,
    pub pod_annotations: Vec<KeyValueEntry>,
    pub pod_spec: PodSpecFormValues,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReplicaSetFormValues {
    pub replicas: String,
    pub min_ready_seconds: String,
    pub selector_labels: Vec<KeyValueEntry>,
    pub labels: Vec<KeyValueEntry>,
    pub annotations: Vec<KeyValueEntry>,
    pub pod_labels: Vec<KeyValueEntry>,
    pub pod_annotations: Vec<KeyValueEntry>,
    pub pod_spec: PodSpecFormValues,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JobFormValues {
    pub completions: String,
    pub parallelism: String,
    pub backoff_limit: String,
    pub active_deadline_seconds: String,
    pub ttl_seconds_after_finished: String,
    pub suspend: bool,
    pub completion_mode: String,
    pub backoff_limit_per_index: String,
    pub max_failed_indexes: String,
    pub pod_replacement_policy: String,
    pub selector_labels: Vec<KeyValueEntry>,
    pub labels: Vec<KeyValueEntry>,
    pub annotations: Vec<KeyValueEntry>,
    pub pod_labels: Vec<KeyValueEntry>,
    pub pod_annotations: Vec<KeyValueEntry>,
    pub pod_spec: PodSpecFormValues,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CronJobFormValues {
    pub schedule: String,
    pub concurrency_policy: String,
    pub suspend: bool,
    pub starting_deadline_seconds: String,
    pub successful_jobs_history_limit: String,
    pub failed_jobs_history_limit: String,
    pub completions: String,
    pub parallelism: String,
    pub backoff_limit: String,
    pub active_deadline_seconds: String,
    pub ttl_seconds_after_finished: String,
    pub labels: Vec<KeyValueEntry>,
    pub annotations: Vec<KeyValueEntry>,
    pub pod_labels: Vec<KeyValueEntry>,
    pub pod_annotations: Vec<KeyValueEntry>,
    pub pod_spec: PodSpecFormValues,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PodFormValues {
    pub labels: Vec<KeyValueEntry>,
    pub annotations: Vec<KeyValueEntry>,
    pub pod_spec: PodSpecFormValues,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EditFormModel {
    ConfigMap(ConfigMapFormValues),
    Secret(SecretFormValues),
    Namespace(NamespaceFormValues),
    PersistentVolume(PvFormValues),
    PersistentVolumeClaim(PvcFormValues),
    StorageClass(StorageClassFormValues),
    Deployment(DeploymentFormValues),
    StatefulSet(StatefulSetFormValues),
    DaemonSet(DaemonSetFormValues),
    ReplicaSet(ReplicaSetFormValues),
    Job(JobFormValues),
    CronJob(CronJobFormValues),
    Pod(PodFormValues),
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

fn is_valid_rfc1123_segment(segment: &str) -> bool {
    if segment.is_empty() {
        return false;
    }
    let bytes = segment.as_bytes();
    let is_alnum = |b: u8| b.is_ascii_lowercase() || b.is_ascii_digit();
    if !is_alnum(bytes[0]) || !is_alnum(bytes[bytes.len() - 1]) {
        return false;
    }
    bytes.iter().all(|&b| is_alnum(b) || b == b'-')
}

pub fn is_valid_k8s_name(name: &str) -> bool {
    let trimmed = name.trim();
    if trimmed.is_empty() || trimmed.len() > 253 {
        return false;
    }
    trimmed.split('.').all(is_valid_rfc1123_segment)
}

pub fn is_valid_k8s_label(name: &str) -> bool {
    let trimmed = name.trim();
    if trimmed.is_empty() || trimmed.len() > 63 {
        return false;
    }
    is_valid_rfc1123_segment(trimmed)
}

pub fn is_valid_path(path: &str) -> bool {
    let trimmed = path.trim();
    trimmed.is_empty() || trimmed.starts_with('/')
}

pub fn is_valid_port(port: &str) -> bool {
    let trimmed = port.trim();
    trimmed
        .parse::<u16>()
        .is_ok_and(|p| p >= 1)
}

pub fn is_valid_cron(schedule: &str) -> bool {
    let trimmed = schedule.trim();
    if trimmed.is_empty() {
        return false;
    }
    const MACROS: &[&str] = &[
        "@yearly",
        "@annually",
        "@monthly",
        "@weekly",
        "@daily",
        "@midnight",
        "@hourly",
    ];
    if MACROS.contains(&trimmed) {
        return true;
    }
    let parts: Vec<&str> = trimmed.split_whitespace().collect();
    if parts.len() != 5 {
        return false;
    }
    parts.iter().all(|part| {
        !part.is_empty()
            && part
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'*' | b',' | b'-' | b'/'))
    })
}

fn is_valid_int_or_percent(value: &str) -> bool {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return true;
    }
    if let Some(pct_str) = trimmed.strip_suffix('%') {
        return !pct_str.is_empty()
            && pct_str.bytes().all(|b| b.is_ascii_digit())
            && pct_str.parse::<u32>().is_ok_and(|pct| pct <= 100);
    }
    trimmed.bytes().all(|b| b.is_ascii_digit()) && trimmed.parse::<u32>().is_ok()
}

pub fn is_valid_max_unavailable(value: &str) -> bool {
    is_valid_int_or_percent(value)
}

pub fn is_valid_max_surge(value: &str) -> bool {
    is_valid_int_or_percent(value)
}

fn non_empty_str(map: Option<&Map<String, Value>>, key: &str) -> String {
    map.and_then(|m| m.get(key))
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_string()
}

fn extract_container_env(c: &Map<String, Value>) -> Vec<ContainerEnvEntry> {
    let mut entries = Vec::new();
    if let Some(env_arr) = c.get("env").and_then(Value::as_array) {
        for item in env_arr {
            let Some(obj) = item.as_object() else {
                continue;
            };
            let name = non_empty_str(Some(obj), "name");
            if let Some(vf) = obj.get("valueFrom").and_then(Value::as_object) {
                if let Some(cm) = vf.get("configMapKeyRef").and_then(Value::as_object) {
                    entries.push(ContainerEnvEntry {
                        source_type: EnvSourceType::ConfigMapKey,
                        name,
                        ref_name: non_empty_str(Some(cm), "name"),
                        ref_key: non_empty_str(Some(cm), "key"),
                        ..Default::default()
                    });
                } else if let Some(sec) = vf.get("secretKeyRef").and_then(Value::as_object) {
                    entries.push(ContainerEnvEntry {
                        source_type: EnvSourceType::SecretKey,
                        name,
                        ref_name: non_empty_str(Some(sec), "name"),
                        ref_key: non_empty_str(Some(sec), "key"),
                        ..Default::default()
                    });
                } else if let Some(fr) = vf.get("fieldRef").and_then(Value::as_object) {
                    entries.push(ContainerEnvEntry {
                        source_type: EnvSourceType::FieldRef,
                        name,
                        field_path: non_empty_str(Some(fr), "fieldPath"),
                        ..Default::default()
                    });
                } else if let Some(rfr) = vf.get("resourceFieldRef").and_then(Value::as_object) {
                    entries.push(ContainerEnvEntry {
                        source_type: EnvSourceType::ResourceFieldRef,
                        name,
                        ref_name: non_empty_str(Some(rfr), "containerName"),
                        ref_key: non_empty_str(Some(rfr), "resource"),
                        ..Default::default()
                    });
                } else {
                    entries.push(ContainerEnvEntry {
                        source_type: EnvSourceType::Literal,
                        name,
                        value: non_empty_str(Some(obj), "value"),
                        ..Default::default()
                    });
                }
            } else {
                entries.push(ContainerEnvEntry {
                    source_type: EnvSourceType::Literal,
                    name,
                    value: non_empty_str(Some(obj), "value"),
                    ..Default::default()
                });
            }
        }
    }
    if let Some(env_from_arr) = c.get("envFrom").and_then(Value::as_array) {
        for item in env_from_arr {
            let Some(obj) = item.as_object() else {
                continue;
            };
            let prefix = non_empty_str(Some(obj), "prefix");
            if let Some(cm) = obj.get("configMapRef").and_then(Value::as_object) {
                entries.push(ContainerEnvEntry {
                    source_type: EnvSourceType::ConfigMapEnvFrom,
                    ref_name: non_empty_str(Some(cm), "name"),
                    prefix,
                    ..Default::default()
                });
            } else if let Some(sec) = obj.get("secretRef").and_then(Value::as_object) {
                entries.push(ContainerEnvEntry {
                    source_type: EnvSourceType::SecretEnvFrom,
                    ref_name: non_empty_str(Some(sec), "name"),
                    prefix,
                    ..Default::default()
                });
            }
        }
    }
    entries
}

fn extract_container_form(c: &Map<String, Value>) -> ContainerFormValues {
    let res = c.get("resources").and_then(Value::as_object);
    let req = res.and_then(|r| r.get("requests")).and_then(Value::as_object);
    let lim = res.and_then(|r| r.get("limits")).and_then(Value::as_object);

    let ports = c
        .get("ports")
        .and_then(Value::as_array)
        .map(|arr| {
            arr.iter()
                .filter_map(Value::as_object)
                .map(|p| ContainerPortEntry {
                    name: non_empty_str(Some(p), "name"),
                    container_port: p
                        .get("containerPort")
                        .map(value_as_string)
                        .unwrap_or_default(),
                    protocol: p
                        .get("protocol")
                        .and_then(Value::as_str)
                        .filter(|s| !s.is_empty())
                        .unwrap_or("TCP")
                        .to_string(),
                })
                .collect()
        })
        .unwrap_or_default();

    let image_pull_policy = c
        .get("imagePullPolicy")
        .and_then(Value::as_str)
        .filter(|s| !s.is_empty())
        .unwrap_or("IfNotPresent")
        .to_string();

    ContainerFormValues {
        name: non_empty_str(Some(c), "name"),
        image: non_empty_str(Some(c), "image"),
        image_pull_policy,
        working_dir: non_empty_str(Some(c), "workingDir"),
        command: value_to_string_vec(c.get("command")),
        args: value_to_string_vec(c.get("args")),
        resources: ContainerResourcesValues {
            cpu_request: non_empty_str(req, "cpu"),
            memory_request: non_empty_str(req, "memory"),
            cpu_limit: non_empty_str(lim, "cpu"),
            memory_limit: non_empty_str(lim, "memory"),
        },
        environments: extract_container_env(c),
        ports,
    }
}

pub fn extract_pod_spec_form(
    pod_spec: Option<&Map<String, Value>>,
    default_restart_policy: &str,
) -> PodSpecFormValues {
    let restart_policy = pod_spec
        .and_then(|s| s.get("restartPolicy"))
        .and_then(Value::as_str)
        .filter(|s| !s.is_empty())
        .unwrap_or(default_restart_policy)
        .to_string();
    let termination_grace_period_seconds = pod_spec
        .and_then(|s| s.get("terminationGracePeriodSeconds"))
        .and_then(|v| v.as_u64().or_else(|| v.as_i64().and_then(|n| u64::try_from(n).ok())))
        .map(|n| n.to_string())
        .unwrap_or_else(|| "30".to_string());
    let dns_policy = pod_spec
        .and_then(|s| s.get("dnsPolicy"))
        .and_then(Value::as_str)
        .filter(|s| !s.is_empty())
        .unwrap_or("ClusterFirst")
        .to_string();
    let containers = pod_spec
        .and_then(|s| s.get("containers"))
        .and_then(Value::as_array)
        .map(|arr| {
            arr.iter()
                .filter_map(Value::as_object)
                .map(extract_container_form)
                .collect()
        })
        .unwrap_or_default();

    PodSpecFormValues {
        service_account_name: non_empty_str(pod_spec, "serviceAccountName"),
        restart_policy,
        termination_grace_period_seconds,
        dns_policy,
        node_selector: map_to_kv_entries(pod_spec.and_then(|s| s.get("nodeSelector"))),
        host_network: pod_spec
            .and_then(|s| s.get("hostNetwork"))
            .and_then(Value::as_bool)
            .unwrap_or(false),
        host_pid: pod_spec
            .and_then(|s| s.get("hostPID"))
            .and_then(Value::as_bool)
            .unwrap_or(false),
        host_ipc: pod_spec
            .and_then(|s| s.get("hostIPC"))
            .and_then(Value::as_bool)
            .unwrap_or(false),
        containers,
    }
}

fn set_or_remove_trimmed_str(map: &mut Map<String, Value>, key: &str, value: &str) {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        map.remove(key);
    } else {
        map.insert(key.to_string(), Value::String(trimmed.to_string()));
    }
}

fn set_or_remove_u64_str(map: &mut Map<String, Value>, key: &str, value: &str) {
    let trimmed = value.trim();
    if let Ok(num) = trimmed.parse::<u64>() {
        map.insert(key.to_string(), Value::Number(num.into()));
    } else if trimmed.is_empty() {
        map.remove(key);
    }
}

fn set_or_remove_bool(map: &mut Map<String, Value>, key: &str, value: bool) {
    if value {
        map.insert(key.to_string(), Value::Bool(true));
    } else {
        map.remove(key);
    }
}

fn apply_container_env(c_map: &mut Map<String, Value>, environments: &[ContainerEnvEntry]) {
    let mut env_list = Vec::new();
    let mut env_from_list = Vec::new();

    for entry in environments {
        match entry.source_type {
            EnvSourceType::ConfigMapEnvFrom => {
                let ref_name = entry.ref_name.trim();
                if ref_name.is_empty() {
                    continue;
                }
                let mut item = Map::new();
                let mut cm_ref = Map::new();
                cm_ref.insert("name".to_string(), Value::String(ref_name.to_string()));
                item.insert("configMapRef".to_string(), Value::Object(cm_ref));
                set_or_remove_trimmed_str(&mut item, "prefix", &entry.prefix);
                env_from_list.push(Value::Object(item));
            }
            EnvSourceType::SecretEnvFrom => {
                let ref_name = entry.ref_name.trim();
                if ref_name.is_empty() {
                    continue;
                }
                let mut item = Map::new();
                let mut sec_ref = Map::new();
                sec_ref.insert("name".to_string(), Value::String(ref_name.to_string()));
                item.insert("secretRef".to_string(), Value::Object(sec_ref));
                set_or_remove_trimmed_str(&mut item, "prefix", &entry.prefix);
                env_from_list.push(Value::Object(item));
            }
            _ => {
                let name = entry.name.trim();
                if name.is_empty() {
                    continue;
                }
                let mut item = Map::new();
                item.insert("name".to_string(), Value::String(name.to_string()));
                match entry.source_type {
                    EnvSourceType::Literal => {
                        item.insert("value".to_string(), Value::String(entry.value.clone()));
                    }
                    EnvSourceType::ConfigMapKey => {
                        let mut key_ref = Map::new();
                        key_ref.insert("name".to_string(), Value::String(entry.ref_name.trim().to_string()));
                        key_ref.insert("key".to_string(), Value::String(entry.ref_key.trim().to_string()));
                        let mut vf = Map::new();
                        vf.insert("configMapKeyRef".to_string(), Value::Object(key_ref));
                        item.insert("valueFrom".to_string(), Value::Object(vf));
                    }
                    EnvSourceType::SecretKey => {
                        let mut key_ref = Map::new();
                        key_ref.insert("name".to_string(), Value::String(entry.ref_name.trim().to_string()));
                        key_ref.insert("key".to_string(), Value::String(entry.ref_key.trim().to_string()));
                        let mut vf = Map::new();
                        vf.insert("secretKeyRef".to_string(), Value::Object(key_ref));
                        item.insert("valueFrom".to_string(), Value::Object(vf));
                    }
                    EnvSourceType::FieldRef => {
                        let mut fr = Map::new();
                        fr.insert("fieldPath".to_string(), Value::String(entry.field_path.trim().to_string()));
                        let mut vf = Map::new();
                        vf.insert("fieldRef".to_string(), Value::Object(fr));
                        item.insert("valueFrom".to_string(), Value::Object(vf));
                    }
                    EnvSourceType::ResourceFieldRef => {
                        let mut rfr = Map::new();
                        set_or_remove_trimmed_str(&mut rfr, "containerName", &entry.ref_name);
                        rfr.insert("resource".to_string(), Value::String(entry.ref_key.trim().to_string()));
                        let mut vf = Map::new();
                        vf.insert("resourceFieldRef".to_string(), Value::Object(rfr));
                        item.insert("valueFrom".to_string(), Value::Object(vf));
                    }
                    EnvSourceType::ConfigMapEnvFrom | EnvSourceType::SecretEnvFrom => {}
                }
                env_list.push(Value::Object(item));
            }
        }
    }

    if env_list.is_empty() {
        c_map.remove("env");
    } else {
        c_map.insert("env".to_string(), Value::Array(env_list));
    }
    if env_from_list.is_empty() {
        c_map.remove("envFrom");
    } else {
        c_map.insert("envFrom".to_string(), Value::Array(env_from_list));
    }
}

fn apply_container_resources(c_map: &mut Map<String, Value>, res: &ContainerResourcesValues) {
    let mut requests = Map::new();
    set_or_remove_trimmed_str(&mut requests, "cpu", &res.cpu_request);
    set_or_remove_trimmed_str(&mut requests, "memory", &res.memory_request);

    let mut limits = Map::new();
    set_or_remove_trimmed_str(&mut limits, "cpu", &res.cpu_limit);
    set_or_remove_trimmed_str(&mut limits, "memory", &res.memory_limit);

    let mut resources = c_map
        .get("resources")
        .and_then(Value::as_object)
        .cloned()
        .unwrap_or_default();
    set_or_remove_map_field(&mut resources, "requests", requests);
    set_or_remove_map_field(&mut resources, "limits", limits);

    if resources.is_empty() {
        c_map.remove("resources");
    } else {
        c_map.insert("resources".to_string(), Value::Object(resources));
    }
}

fn apply_container_form(
    existing: Option<&Map<String, Value>>,
    c: &ContainerFormValues,
) -> Value {
    let mut c_map = existing.cloned().unwrap_or_default();
    set_or_remove_trimmed_str(&mut c_map, "name", &c.name);
    set_or_remove_trimmed_str(&mut c_map, "image", &c.image);
    set_or_remove_trimmed_str(&mut c_map, "imagePullPolicy", &c.image_pull_policy);
    set_or_remove_trimmed_str(&mut c_map, "workingDir", &c.working_dir);

    let clean_cmd = clean_string_vec(&c.command);
    if clean_cmd.is_empty() {
        c_map.remove("command");
    } else {
        c_map.insert("command".to_string(), Value::Array(clean_cmd));
    }

    let clean_args = clean_string_vec(&c.args);
    if clean_args.is_empty() {
        c_map.remove("args");
    } else {
        c_map.insert("args".to_string(), Value::Array(clean_args));
    }

    apply_container_resources(&mut c_map, &c.resources);
    apply_container_env(&mut c_map, &c.environments);

    let ports_arr: Vec<Value> = c
        .ports
        .iter()
        .filter_map(|p| {
            let port_num = p.container_port.trim().parse::<u16>().ok()?;
            let mut p_map = Map::new();
            set_or_remove_trimmed_str(&mut p_map, "name", &p.name);
            p_map.insert("containerPort".to_string(), Value::Number(port_num.into()));
            let proto = if p.protocol.trim().is_empty() {
                "TCP"
            } else {
                p.protocol.trim()
            };
            p_map.insert("protocol".to_string(), Value::String(proto.to_string()));
            Some(Value::Object(p_map))
        })
        .collect();
    if ports_arr.is_empty() {
        c_map.remove("ports");
    } else {
        c_map.insert("ports".to_string(), Value::Array(ports_arr));
    }

    Value::Object(c_map)
}

pub fn apply_pod_spec_form(pod_spec: &mut Map<String, Value>, form: &PodSpecFormValues) {
    set_or_remove_trimmed_str(pod_spec, "serviceAccountName", &form.service_account_name);
    set_or_remove_trimmed_str(pod_spec, "restartPolicy", &form.restart_policy);
    set_or_remove_u64_str(
        pod_spec,
        "terminationGracePeriodSeconds",
        &form.termination_grace_period_seconds,
    );
    set_or_remove_trimmed_str(pod_spec, "dnsPolicy", &form.dns_policy);
    set_or_remove_map_field(pod_spec, "nodeSelector", kv_entries_to_map(&form.node_selector));
    set_or_remove_bool(pod_spec, "hostNetwork", form.host_network);
    set_or_remove_bool(pod_spec, "hostPID", form.host_pid);
    set_or_remove_bool(pod_spec, "hostIPC", form.host_ipc);

    let existing_containers = pod_spec
        .get("containers")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    let updated_containers: Vec<Value> = form
        .containers
        .iter()
        .enumerate()
        .map(|(idx, c)| {
            let existing = existing_containers.get(idx).and_then(Value::as_object);
            apply_container_form(existing, c)
        })
        .collect();
    pod_spec.insert("containers".to_string(), Value::Array(updated_containers));
}

fn validate_non_negative_int_str(value: &str, label: &str) -> Option<String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return None;
    }
    if trimmed.parse::<u64>().is_err() {
        Some(format!("{label} must be a non-negative integer."))
    } else {
        None
    }
}

pub fn validate_pod_spec_form(pod_spec: &PodSpecFormValues) -> Option<String> {
    if !pod_spec.service_account_name.trim().is_empty()
        && !is_valid_k8s_name(&pod_spec.service_account_name)
    {
        return Some("Service account name must be a valid Kubernetes RFC 1123 name.".to_string());
    }
    if let Some(err) = validate_non_negative_int_str(
        &pod_spec.termination_grace_period_seconds,
        "Termination grace period",
    ) {
        return Some(err);
    }
    for (idx, c) in pod_spec.containers.iter().enumerate() {
        let cname = c.name.trim();
        if cname.is_empty() || !is_valid_k8s_label(cname) {
            return Some(format!(
                "Container #{} name must be a valid Kubernetes RFC 1123 label.",
                idx + 1
            ));
        }
        if c.image.trim().is_empty() {
            return Some(format!("Container '{cname}' image is required."));
        }
        if !is_valid_path(&c.working_dir) {
            return Some(format!(
                "Container '{cname}' working directory must be empty or start with '/'."
            ));
        }
        for (env_idx, env) in c.environments.iter().enumerate() {
            let env_num = env_idx + 1;
            match env.source_type {
                EnvSourceType::Literal => {
                    if env.name.trim().is_empty() {
                        return Some(format!(
                            "Container '{cname}' env #{env_num} variable name is required."
                        ));
                    }
                }
                EnvSourceType::ConfigMapKey | EnvSourceType::SecretKey => {
                    if env.name.trim().is_empty()
                        || env.ref_name.trim().is_empty()
                        || env.ref_key.trim().is_empty()
                    {
                        return Some(format!(
                            "Container '{cname}' env #{env_num} requires variable name, reference name, and key."
                        ));
                    }
                }
                EnvSourceType::FieldRef => {
                    if env.name.trim().is_empty() || env.field_path.trim().is_empty() {
                        return Some(format!(
                            "Container '{cname}' env #{env_num} requires variable name and fieldPath."
                        ));
                    }
                }
                EnvSourceType::ResourceFieldRef => {
                    if env.name.trim().is_empty() || env.ref_key.trim().is_empty() {
                        return Some(format!(
                            "Container '{cname}' env #{env_num} requires variable name and resource."
                        ));
                    }
                }
                EnvSourceType::ConfigMapEnvFrom | EnvSourceType::SecretEnvFrom => {
                    if env.ref_name.trim().is_empty() {
                        return Some(format!(
                            "Container '{cname}' envFrom #{env_num} requires a reference name."
                        ));
                    }
                }
            }
        }
        for port in &c.ports {
            if !port.name.trim().is_empty() && !is_valid_k8s_label(&port.name) {
                return Some(format!(
                    "Container '{cname}' port name '{}' must be a valid RFC 1123 label.",
                    port.name
                ));
            }
            if !is_valid_port(&port.container_port) {
                return Some(format!(
                    "Container '{cname}' port '{}' must be between 1 and 65535.",
                    port.container_port
                ));
            }
        }
    }
    None
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

fn int_or_default_str(map: Option<&Map<String, Value>>, key: &str, default: &str) -> String {
    map.and_then(|m| m.get(key))
        .and_then(|v| {
            v.as_i64()
                .map(|n| n.to_string())
                .or_else(|| v.as_u64().map(|n| n.to_string()))
                .or_else(|| v.as_str().map(ToOwned::to_owned))
        })
        .unwrap_or_else(|| default.to_string())
}

fn int_or_empty_str(map: Option<&Map<String, Value>>, key: &str) -> String {
    int_or_default_str(map, key, "")
}

fn str_or_default(map: Option<&Map<String, Value>>, key: &str, default: &str) -> String {
    map.and_then(|m| m.get(key))
        .and_then(Value::as_str)
        .filter(|s| !s.is_empty())
        .unwrap_or(default)
        .to_string()
}

fn int_or_str_quantity(map: Option<&Map<String, Value>>, key: &str, default: &str) -> String {
    map.and_then(|m| m.get(key))
        .map(value_as_string)
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| default.to_string())
}

fn extract_template_fields(
    template_parent: Option<&Map<String, Value>>,
    default_restart_policy: &str,
) -> (Vec<KeyValueEntry>, Vec<KeyValueEntry>, PodSpecFormValues) {
    let tpl = template_parent
        .and_then(|s| s.get("template"))
        .and_then(Value::as_object);
    let tpl_meta = tpl.and_then(|t| t.get("metadata")).and_then(Value::as_object);
    let tpl_spec = tpl.and_then(|t| t.get("spec")).and_then(Value::as_object);
    (
        map_to_kv_entries(tpl_meta.and_then(|m| m.get("labels"))),
        map_to_kv_entries(tpl_meta.and_then(|m| m.get("annotations"))),
        extract_pod_spec_form(tpl_spec, default_restart_policy),
    )
}

fn apply_template_fields(
    template_parent: &mut Map<String, Value>,
    pod_labels: &[KeyValueEntry],
    pod_annotations: &[KeyValueEntry],
    pod_spec: &PodSpecFormValues,
) {
    let tpl = ensure_object_mut(template_parent, "template");
    let tpl_meta = ensure_object_mut(tpl, "metadata");
    set_or_remove_map_field(tpl_meta, "labels", kv_entries_to_map(pod_labels));
    set_or_remove_map_field(tpl_meta, "annotations", kv_entries_to_map(pod_annotations));
    let tpl_spec = ensure_object_mut(tpl, "spec");
    apply_pod_spec_form(tpl_spec, pod_spec);
}

fn apply_meta_labels_annotations(
    root: &mut Map<String, Value>,
    labels: &[KeyValueEntry],
    annotations: &[KeyValueEntry],
) {
    let meta = ensure_object_mut(root, "metadata");
    set_or_remove_map_field(meta, "labels", kv_entries_to_map(labels));
    set_or_remove_map_field(meta, "annotations", kv_entries_to_map(annotations));
}

fn set_int_or_str_quantity(map: &mut Map<String, Value>, key: &str, value: &str) {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        map.remove(key);
    } else if let Ok(num) = trimmed.parse::<u64>() {
        map.insert(key.to_string(), Value::Number(num.into()));
    } else {
        map.insert(key.to_string(), Value::String(trimmed.to_string()));
    }
}

fn extract_volume_claim_summaries(spec: Option<&Map<String, Value>>) -> Vec<String> {
    spec.and_then(|s| s.get("volumeClaimTemplates"))
        .and_then(Value::as_array)
        .map(|arr| {
            arr.iter()
                .filter_map(Value::as_object)
                .map(|pvc| {
                    let name = pvc
                        .get("metadata")
                        .and_then(Value::as_object)
                        .and_then(|m| m.get("name"))
                        .and_then(Value::as_str)
                        .unwrap_or("unnamed");
                    let pvc_spec = pvc.get("spec").and_then(Value::as_object);
                    let storage = pvc_spec
                        .and_then(|s| s.get("resources"))
                        .and_then(|r| r.get("requests"))
                        .and_then(|r| r.get("storage"))
                        .and_then(Value::as_str)
                        .unwrap_or("-");
                    let sc = pvc_spec
                        .and_then(|s| s.get("storageClassName"))
                        .and_then(Value::as_str)
                        .unwrap_or("default");
                    let modes = value_to_string_vec(pvc_spec.and_then(|s| s.get("accessModes")));
                    let modes_str = if modes.is_empty() {
                        "-".to_string()
                    } else {
                        modes.join(", ")
                    };
                    format!("{name}: {storage} ({modes_str}, class: {sc})")
                })
                .collect()
        })
        .unwrap_or_default()
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
        "Deployment" => {
            let strategy = spec.and_then(|s| s.get("strategy")).and_then(Value::as_object);
            let rolling = strategy
                .and_then(|st| st.get("rollingUpdate"))
                .and_then(Value::as_object);
            let (pod_labels, pod_annotations, pod_spec) = extract_template_fields(spec, "Always");
            EditFormModel::Deployment(DeploymentFormValues {
                replicas: int_or_default_str(spec, "replicas", "1"),
                paused: spec
                    .and_then(|s| s.get("paused"))
                    .and_then(Value::as_bool)
                    .unwrap_or(false),
                strategy_type: str_or_default(strategy, "type", "RollingUpdate"),
                max_surge: int_or_str_quantity(rolling, "maxSurge", "25%"),
                max_unavailable: int_or_str_quantity(rolling, "maxUnavailable", "25%"),
                min_ready_seconds: int_or_default_str(spec, "minReadySeconds", "0"),
                revision_history_limit: int_or_default_str(spec, "revisionHistoryLimit", "10"),
                progress_deadline_seconds: int_or_default_str(spec, "progressDeadlineSeconds", "600"),
                selector_labels: map_to_kv_entries(
                    spec.and_then(|s| s.get("selector"))
                        .and_then(|sel| sel.get("matchLabels")),
                ),
                labels: map_to_kv_entries(meta.and_then(|m| m.get("labels"))),
                annotations: map_to_kv_entries(meta.and_then(|m| m.get("annotations"))),
                pod_labels,
                pod_annotations,
                pod_spec,
            })
        }
        "StatefulSet" => {
            let strategy = spec
                .and_then(|s| s.get("updateStrategy"))
                .and_then(Value::as_object);
            let rolling = strategy
                .and_then(|st| st.get("rollingUpdate"))
                .and_then(Value::as_object);
            let retention = spec
                .and_then(|s| s.get("persistentVolumeClaimRetentionPolicy"))
                .and_then(Value::as_object);
            let (pod_labels, pod_annotations, pod_spec) = extract_template_fields(spec, "Always");
            EditFormModel::StatefulSet(StatefulSetFormValues {
                replicas: int_or_default_str(spec, "replicas", "1"),
                service_name: non_empty_str(spec, "serviceName"),
                pod_management_policy: str_or_default(spec, "podManagementPolicy", "OrderedReady"),
                strategy_type: str_or_default(strategy, "type", "RollingUpdate"),
                partition: int_or_default_str(rolling, "partition", "0"),
                max_unavailable: int_or_str_quantity(rolling, "maxUnavailable", "1"),
                min_ready_seconds: int_or_default_str(spec, "minReadySeconds", "0"),
                revision_history_limit: int_or_default_str(spec, "revisionHistoryLimit", "10"),
                selector_labels: map_to_kv_entries(
                    spec.and_then(|s| s.get("selector"))
                        .and_then(|sel| sel.get("matchLabels")),
                ),
                labels: map_to_kv_entries(meta.and_then(|m| m.get("labels"))),
                annotations: map_to_kv_entries(meta.and_then(|m| m.get("annotations"))),
                pod_labels,
                pod_annotations,
                pod_spec,
                volume_claim_summaries: extract_volume_claim_summaries(spec),
                pvc_retention_when_deleted: non_empty_str(retention, "whenDeleted"),
                pvc_retention_when_scaled: non_empty_str(retention, "whenScaled"),
            })
        }
        "DaemonSet" => {
            let strategy = spec
                .and_then(|s| s.get("updateStrategy"))
                .and_then(Value::as_object);
            let rolling = strategy
                .and_then(|st| st.get("rollingUpdate"))
                .and_then(Value::as_object);
            let (pod_labels, pod_annotations, pod_spec) = extract_template_fields(spec, "Always");
            EditFormModel::DaemonSet(DaemonSetFormValues {
                strategy_type: str_or_default(strategy, "type", "RollingUpdate"),
                max_unavailable: int_or_str_quantity(rolling, "maxUnavailable", "1"),
                max_surge: int_or_str_quantity(rolling, "maxSurge", "0"),
                min_ready_seconds: int_or_default_str(spec, "minReadySeconds", "0"),
                revision_history_limit: int_or_default_str(spec, "revisionHistoryLimit", "10"),
                selector_labels: map_to_kv_entries(
                    spec.and_then(|s| s.get("selector"))
                        .and_then(|sel| sel.get("matchLabels")),
                ),
                labels: map_to_kv_entries(meta.and_then(|m| m.get("labels"))),
                annotations: map_to_kv_entries(meta.and_then(|m| m.get("annotations"))),
                pod_labels,
                pod_annotations,
                pod_spec,
            })
        }
        "ReplicaSet" => {
            let (pod_labels, pod_annotations, pod_spec) = extract_template_fields(spec, "Always");
            EditFormModel::ReplicaSet(ReplicaSetFormValues {
                replicas: int_or_default_str(spec, "replicas", "1"),
                min_ready_seconds: int_or_default_str(spec, "minReadySeconds", "0"),
                selector_labels: map_to_kv_entries(
                    spec.and_then(|s| s.get("selector"))
                        .and_then(|sel| sel.get("matchLabels")),
                ),
                labels: map_to_kv_entries(meta.and_then(|m| m.get("labels"))),
                annotations: map_to_kv_entries(meta.and_then(|m| m.get("annotations"))),
                pod_labels,
                pod_annotations,
                pod_spec,
            })
        }
        "Job" => {
            let (pod_labels, pod_annotations, pod_spec) = extract_template_fields(spec, "Never");
            EditFormModel::Job(JobFormValues {
                completions: int_or_empty_str(spec, "completions"),
                parallelism: int_or_default_str(spec, "parallelism", "1"),
                backoff_limit: int_or_default_str(spec, "backoffLimit", "6"),
                active_deadline_seconds: int_or_empty_str(spec, "activeDeadlineSeconds"),
                ttl_seconds_after_finished: int_or_empty_str(spec, "ttlSecondsAfterFinished"),
                suspend: spec
                    .and_then(|s| s.get("suspend"))
                    .and_then(Value::as_bool)
                    .unwrap_or(false),
                completion_mode: str_or_default(spec, "completionMode", "NonIndexed"),
                backoff_limit_per_index: int_or_empty_str(spec, "backoffLimitPerIndex"),
                max_failed_indexes: int_or_empty_str(spec, "maxFailedIndexes"),
                pod_replacement_policy: non_empty_str(spec, "podReplacementPolicy"),
                selector_labels: map_to_kv_entries(
                    spec.and_then(|s| s.get("selector"))
                        .and_then(|sel| sel.get("matchLabels")),
                ),
                labels: map_to_kv_entries(meta.and_then(|m| m.get("labels"))),
                annotations: map_to_kv_entries(meta.and_then(|m| m.get("annotations"))),
                pod_labels,
                pod_annotations,
                pod_spec,
            })
        }
        "CronJob" => {
            let job_spec = spec
                .and_then(|s| s.get("jobTemplate"))
                .and_then(|jt| jt.get("spec"))
                .and_then(Value::as_object);
            let (pod_labels, pod_annotations, pod_spec) =
                extract_template_fields(job_spec, "OnFailure");
            EditFormModel::CronJob(CronJobFormValues {
                schedule: str_or_default(spec, "schedule", "*/5 * * * *"),
                concurrency_policy: str_or_default(spec, "concurrencyPolicy", "Allow"),
                suspend: spec
                    .and_then(|s| s.get("suspend"))
                    .and_then(Value::as_bool)
                    .unwrap_or(false),
                starting_deadline_seconds: int_or_empty_str(spec, "startingDeadlineSeconds"),
                successful_jobs_history_limit: int_or_default_str(
                    spec,
                    "successfulJobsHistoryLimit",
                    "3",
                ),
                failed_jobs_history_limit: int_or_default_str(spec, "failedJobsHistoryLimit", "1"),
                completions: int_or_empty_str(job_spec, "completions"),
                parallelism: int_or_default_str(job_spec, "parallelism", "1"),
                backoff_limit: int_or_default_str(job_spec, "backoffLimit", "6"),
                active_deadline_seconds: int_or_empty_str(job_spec, "activeDeadlineSeconds"),
                ttl_seconds_after_finished: int_or_empty_str(job_spec, "ttlSecondsAfterFinished"),
                labels: map_to_kv_entries(meta.and_then(|m| m.get("labels"))),
                annotations: map_to_kv_entries(meta.and_then(|m| m.get("annotations"))),
                pod_labels,
                pod_annotations,
                pod_spec,
            })
        }
        "Pod" => EditFormModel::Pod(PodFormValues {
            labels: map_to_kv_entries(meta.and_then(|m| m.get("labels"))),
            annotations: map_to_kv_entries(meta.and_then(|m| m.get("annotations"))),
            pod_spec: extract_pod_spec_form(spec, "Always"),
        }),
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
        EditFormModel::Deployment(dep) => {
            apply_meta_labels_annotations(&mut root, &dep.labels, &dep.annotations);
            let spec = ensure_object_mut(&mut root, "spec");
            set_or_remove_u64_str(spec, "replicas", &dep.replicas);
            set_or_remove_bool(spec, "paused", dep.paused);
            set_or_remove_u64_str(spec, "minReadySeconds", &dep.min_ready_seconds);
            set_or_remove_u64_str(spec, "revisionHistoryLimit", &dep.revision_history_limit);
            set_or_remove_u64_str(
                spec,
                "progressDeadlineSeconds",
                &dep.progress_deadline_seconds,
            );
            let strategy = ensure_object_mut(spec, "strategy");
            let st_type = if dep.strategy_type.trim().is_empty() {
                "RollingUpdate"
            } else {
                dep.strategy_type.trim()
            };
            strategy.insert("type".to_string(), Value::String(st_type.to_string()));
            if st_type == "RollingUpdate" {
                let rolling = ensure_object_mut(strategy, "rollingUpdate");
                set_int_or_str_quantity(rolling, "maxSurge", &dep.max_surge);
                set_int_or_str_quantity(rolling, "maxUnavailable", &dep.max_unavailable);
            } else {
                strategy.remove("rollingUpdate");
            }
            apply_template_fields(spec, &dep.pod_labels, &dep.pod_annotations, &dep.pod_spec);
        }
        EditFormModel::StatefulSet(sts) => {
            apply_meta_labels_annotations(&mut root, &sts.labels, &sts.annotations);
            let spec = ensure_object_mut(&mut root, "spec");
            set_or_remove_u64_str(spec, "replicas", &sts.replicas);
            set_or_remove_trimmed_str(spec, "serviceName", &sts.service_name);
            set_or_remove_trimmed_str(spec, "podManagementPolicy", &sts.pod_management_policy);
            set_or_remove_u64_str(spec, "minReadySeconds", &sts.min_ready_seconds);
            set_or_remove_u64_str(spec, "revisionHistoryLimit", &sts.revision_history_limit);
            let strategy = ensure_object_mut(spec, "updateStrategy");
            let st_type = if sts.strategy_type.trim().is_empty() {
                "RollingUpdate"
            } else {
                sts.strategy_type.trim()
            };
            strategy.insert("type".to_string(), Value::String(st_type.to_string()));
            if st_type == "RollingUpdate" {
                let rolling = ensure_object_mut(strategy, "rollingUpdate");
                set_or_remove_u64_str(rolling, "partition", &sts.partition);
                set_int_or_str_quantity(rolling, "maxUnavailable", &sts.max_unavailable);
            } else {
                strategy.remove("rollingUpdate");
            }
            apply_template_fields(spec, &sts.pod_labels, &sts.pod_annotations, &sts.pod_spec);
        }
        EditFormModel::DaemonSet(ds) => {
            apply_meta_labels_annotations(&mut root, &ds.labels, &ds.annotations);
            let spec = ensure_object_mut(&mut root, "spec");
            set_or_remove_u64_str(spec, "minReadySeconds", &ds.min_ready_seconds);
            set_or_remove_u64_str(spec, "revisionHistoryLimit", &ds.revision_history_limit);
            let strategy = ensure_object_mut(spec, "updateStrategy");
            let st_type = if ds.strategy_type.trim().is_empty() {
                "RollingUpdate"
            } else {
                ds.strategy_type.trim()
            };
            strategy.insert("type".to_string(), Value::String(st_type.to_string()));
            if st_type == "RollingUpdate" {
                let rolling = ensure_object_mut(strategy, "rollingUpdate");
                set_int_or_str_quantity(rolling, "maxUnavailable", &ds.max_unavailable);
                set_int_or_str_quantity(rolling, "maxSurge", &ds.max_surge);
            } else {
                strategy.remove("rollingUpdate");
            }
            apply_template_fields(spec, &ds.pod_labels, &ds.pod_annotations, &ds.pod_spec);
        }
        EditFormModel::ReplicaSet(rs) => {
            apply_meta_labels_annotations(&mut root, &rs.labels, &rs.annotations);
            let spec = ensure_object_mut(&mut root, "spec");
            set_or_remove_u64_str(spec, "replicas", &rs.replicas);
            set_or_remove_u64_str(spec, "minReadySeconds", &rs.min_ready_seconds);
            apply_template_fields(spec, &rs.pod_labels, &rs.pod_annotations, &rs.pod_spec);
        }
        EditFormModel::Job(job) => {
            apply_meta_labels_annotations(&mut root, &job.labels, &job.annotations);
            let spec = ensure_object_mut(&mut root, "spec");
            set_or_remove_u64_str(spec, "completions", &job.completions);
            set_or_remove_u64_str(spec, "parallelism", &job.parallelism);
            set_or_remove_u64_str(spec, "backoffLimit", &job.backoff_limit);
            set_or_remove_u64_str(spec, "activeDeadlineSeconds", &job.active_deadline_seconds);
            set_or_remove_u64_str(
                spec,
                "ttlSecondsAfterFinished",
                &job.ttl_seconds_after_finished,
            );
            set_or_remove_bool(spec, "suspend", job.suspend);
            set_or_remove_trimmed_str(spec, "completionMode", &job.completion_mode);
            set_or_remove_u64_str(spec, "backoffLimitPerIndex", &job.backoff_limit_per_index);
            set_or_remove_u64_str(spec, "maxFailedIndexes", &job.max_failed_indexes);
            set_or_remove_trimmed_str(spec, "podReplacementPolicy", &job.pod_replacement_policy);
            apply_template_fields(spec, &job.pod_labels, &job.pod_annotations, &job.pod_spec);
        }
        EditFormModel::CronJob(cj) => {
            apply_meta_labels_annotations(&mut root, &cj.labels, &cj.annotations);
            let spec = ensure_object_mut(&mut root, "spec");
            set_or_remove_trimmed_str(spec, "schedule", &cj.schedule);
            set_or_remove_trimmed_str(spec, "concurrencyPolicy", &cj.concurrency_policy);
            set_or_remove_bool(spec, "suspend", cj.suspend);
            set_or_remove_u64_str(
                spec,
                "startingDeadlineSeconds",
                &cj.starting_deadline_seconds,
            );
            set_or_remove_u64_str(
                spec,
                "successfulJobsHistoryLimit",
                &cj.successful_jobs_history_limit,
            );
            set_or_remove_u64_str(
                spec,
                "failedJobsHistoryLimit",
                &cj.failed_jobs_history_limit,
            );
            let job_tpl = ensure_object_mut(spec, "jobTemplate");
            let job_spec = ensure_object_mut(job_tpl, "spec");
            set_or_remove_u64_str(job_spec, "completions", &cj.completions);
            set_or_remove_u64_str(job_spec, "parallelism", &cj.parallelism);
            set_or_remove_u64_str(job_spec, "backoffLimit", &cj.backoff_limit);
            set_or_remove_u64_str(
                job_spec,
                "activeDeadlineSeconds",
                &cj.active_deadline_seconds,
            );
            set_or_remove_u64_str(
                job_spec,
                "ttlSecondsAfterFinished",
                &cj.ttl_seconds_after_finished,
            );
            apply_template_fields(job_spec, &cj.pod_labels, &cj.pod_annotations, &cj.pod_spec);
        }
        EditFormModel::Pod(pod) => {
            apply_meta_labels_annotations(&mut root, &pod.labels, &pod.annotations);
            let spec = ensure_object_mut(&mut root, "spec");
            apply_pod_spec_form(spec, &pod.pod_spec);
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
        EditFormModel::Deployment(dep) => {
            validate_non_negative_int_str(&dep.replicas, "Replicas")
                .or_else(|| {
                    validate_non_negative_int_str(&dep.min_ready_seconds, "Min ready seconds")
                })
                .or_else(|| {
                    validate_non_negative_int_str(
                        &dep.revision_history_limit,
                        "Revision history limit",
                    )
                })
                .or_else(|| {
                    validate_non_negative_int_str(
                        &dep.progress_deadline_seconds,
                        "Progress deadline seconds",
                    )
                })
                .or_else(|| {
                    if dep.strategy_type == "RollingUpdate" && !is_valid_max_surge(&dep.max_surge) {
                        Some("Max surge must be a non-negative integer or percentage (0-100%).".to_string())
                    } else if dep.strategy_type == "RollingUpdate"
                        && !is_valid_max_unavailable(&dep.max_unavailable)
                    {
                        Some("Max unavailable must be a non-negative integer or percentage (0-100%).".to_string())
                    } else {
                        validate_pod_spec_form(&dep.pod_spec)
                    }
                })
        }
        EditFormModel::StatefulSet(sts) => {
            if sts.service_name.trim().is_empty() || !is_valid_k8s_name(&sts.service_name) {
                return Some("Service name must be a valid Kubernetes RFC 1123 name.".to_string());
            }
            validate_non_negative_int_str(&sts.replicas, "Replicas")
                .or_else(|| validate_non_negative_int_str(&sts.partition, "Partition"))
                .or_else(|| {
                    validate_non_negative_int_str(&sts.min_ready_seconds, "Min ready seconds")
                })
                .or_else(|| {
                    validate_non_negative_int_str(
                        &sts.revision_history_limit,
                        "Revision history limit",
                    )
                })
                .or_else(|| {
                    if sts.strategy_type == "RollingUpdate"
                        && !is_valid_max_unavailable(&sts.max_unavailable)
                    {
                        Some("Max unavailable must be a non-negative integer or percentage (0-100%).".to_string())
                    } else {
                        validate_pod_spec_form(&sts.pod_spec)
                    }
                })
        }
        EditFormModel::DaemonSet(ds) => {
            validate_non_negative_int_str(&ds.min_ready_seconds, "Min ready seconds")
                .or_else(|| {
                    validate_non_negative_int_str(
                        &ds.revision_history_limit,
                        "Revision history limit",
                    )
                })
                .or_else(|| {
                    if ds.strategy_type == "RollingUpdate"
                        && !is_valid_max_unavailable(&ds.max_unavailable)
                    {
                        Some("Max unavailable must be a non-negative integer or percentage (0-100%).".to_string())
                    } else if ds.strategy_type == "RollingUpdate"
                        && !is_valid_max_surge(&ds.max_surge)
                    {
                        Some("Max surge must be a non-negative integer or percentage (0-100%).".to_string())
                    } else {
                        validate_pod_spec_form(&ds.pod_spec)
                    }
                })
        }
        EditFormModel::ReplicaSet(rs) => {
            validate_non_negative_int_str(&rs.replicas, "Replicas")
                .or_else(|| {
                    validate_non_negative_int_str(&rs.min_ready_seconds, "Min ready seconds")
                })
                .or_else(|| validate_pod_spec_form(&rs.pod_spec))
        }
        EditFormModel::Job(job) => {
            if job.completion_mode == "Indexed" {
                let valid = job
                    .completions
                    .trim()
                    .parse::<u64>()
                    .is_ok_and(|n| n >= 1);
                if !valid {
                    return Some("Indexed Jobs require completions >= 1.".to_string());
                }
            }
            validate_non_negative_int_str(&job.completions, "Completions")
                .or_else(|| validate_non_negative_int_str(&job.parallelism, "Parallelism"))
                .or_else(|| validate_non_negative_int_str(&job.backoff_limit, "Backoff limit"))
                .or_else(|| {
                    validate_non_negative_int_str(
                        &job.active_deadline_seconds,
                        "Active deadline seconds",
                    )
                })
                .or_else(|| {
                    validate_non_negative_int_str(
                        &job.ttl_seconds_after_finished,
                        "TTL seconds after finished",
                    )
                })
                .or_else(|| {
                    validate_non_negative_int_str(
                        &job.backoff_limit_per_index,
                        "Backoff limit per index",
                    )
                })
                .or_else(|| {
                    validate_non_negative_int_str(&job.max_failed_indexes, "Max failed indexes")
                })
                .or_else(|| validate_pod_spec_form(&job.pod_spec))
        }
        EditFormModel::CronJob(cj) => {
            if !is_valid_cron(&cj.schedule) {
                return Some("Schedule must be a valid 5-field cron expression or macro (e.g. '*/5 * * * *', '@daily').".to_string());
            }
            validate_non_negative_int_str(
                &cj.starting_deadline_seconds,
                "Starting deadline seconds",
            )
            .or_else(|| {
                validate_non_negative_int_str(
                    &cj.successful_jobs_history_limit,
                    "Successful jobs history limit",
                )
            })
            .or_else(|| {
                validate_non_negative_int_str(
                    &cj.failed_jobs_history_limit,
                    "Failed jobs history limit",
                )
            })
            .or_else(|| validate_non_negative_int_str(&cj.completions, "Completions"))
            .or_else(|| validate_non_negative_int_str(&cj.parallelism, "Parallelism"))
            .or_else(|| validate_non_negative_int_str(&cj.backoff_limit, "Backoff limit"))
            .or_else(|| {
                validate_non_negative_int_str(
                    &cj.active_deadline_seconds,
                    "Active deadline seconds",
                )
            })
            .or_else(|| {
                validate_non_negative_int_str(
                    &cj.ttl_seconds_after_finished,
                    "TTL seconds after finished",
                )
            })
            .or_else(|| validate_pod_spec_form(&cj.pod_spec))
        }
        EditFormModel::Pod(pod) => validate_pod_spec_form(&pod.pod_spec),
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

    #[test]
    fn test_workload_validators_k8s_names_labels_paths_ports_cron_and_rollout_quantities() {
        // RFC 1123 names (max 253 chars, lowercase alphanumeric, '-' or '.')
        assert!(is_valid_k8s_name("my-service.default"));
        assert!(is_valid_k8s_name("a"));
        assert!(!is_valid_k8s_name(""));
        assert!(!is_valid_k8s_name("My-Service"));
        assert!(!is_valid_k8s_name("-bad-start"));
        assert!(!is_valid_k8s_name("bad-end-"));
        assert!(!is_valid_k8s_name(&"a".repeat(254)));

        // RFC 1123 labels (max 63 chars, lowercase alphanumeric or '-')
        assert!(is_valid_k8s_label("web-app"));
        assert!(is_valid_k8s_label(&"a".repeat(63)));
        assert!(!is_valid_k8s_label(""));
        assert!(!is_valid_k8s_label(&"a".repeat(64)));
        assert!(!is_valid_k8s_label("web.app"));
        assert!(!is_valid_k8s_label("WebApp"));

        // Paths (empty or starts with '/')
        assert!(is_valid_path(""));
        assert!(is_valid_path("/app"));
        assert!(!is_valid_path("app"));

        // Ports (1..=65535)
        assert!(is_valid_port("80"));
        assert!(is_valid_port("65535"));
        assert!(!is_valid_port("0"));
        assert!(!is_valid_port("65536"));
        assert!(!is_valid_port("abc"));

        // Cron expressions
        assert!(is_valid_cron("*/5 * * * *"));
        assert!(is_valid_cron("0 9 * * MON-FRI"));
        assert!(is_valid_cron("@daily"));
        assert!(is_valid_cron("@hourly"));
        assert!(!is_valid_cron(""));
        assert!(!is_valid_cron("invalid"));
        assert!(!is_valid_cron("* * * *"));

        // Rollout quantities (maxUnavailable / maxSurge)
        assert!(is_valid_max_unavailable("25%"));
        assert!(is_valid_max_unavailable("1"));
        assert!(is_valid_max_unavailable("0%"));
        assert!(is_valid_max_unavailable(""));
        assert!(!is_valid_max_unavailable("101%"));
        assert!(!is_valid_max_unavailable("-1"));
        assert!(!is_valid_max_unavailable("abc"));

        assert!(is_valid_max_surge("25%"));
        assert!(is_valid_max_surge("1"));
        assert!(is_valid_max_surge("0%"));
        assert!(!is_valid_max_surge("150%"));
        assert!(!is_valid_max_surge("-2"));
    }

    #[test]
    fn test_deployment_extract_apply_preserves_unmanaged_fields_and_roundtrips_containers() {
        let deploy = json!({
            "apiVersion": "apps/v1",
            "kind": "Deployment",
            "metadata": {
                "name": "web",
                "namespace": "prod",
                "uid": "dep-uid",
                "resourceVersion": "101",
                "labels": { "app": "web" },
                "annotations": { "deploy.io/note": "blue" }
            },
            "spec": {
                "replicas": 3,
                "paused": false,
                "minReadySeconds": 5,
                "revisionHistoryLimit": 10,
                "progressDeadlineSeconds": 600,
                "strategy": {
                    "type": "RollingUpdate",
                    "rollingUpdate": { "maxSurge": "25%", "maxUnavailable": 1 }
                },
                "selector": { "matchLabels": { "app": "web" } },
                "template": {
                    "metadata": {
                        "labels": { "app": "web" },
                        "annotations": { "prometheus.io/scrape": "true" }
                    },
                    "spec": {
                        "serviceAccountName": "web-sa",
                        "restartPolicy": "Always",
                        "terminationGracePeriodSeconds": 30,
                        "volumes": [{ "name": "cfg", "configMap": { "name": "web-cfg" } }],
                        "containers": [{
                            "name": "nginx",
                            "image": "nginx:1.25",
                            "imagePullPolicy": "IfNotPresent",
                            "workingDir": "/app",
                            "command": ["/bin/sh"],
                            "args": ["-c", "nginx -g 'daemon off;'"],
                            "env": [
                                { "name": "LOG_LEVEL", "value": "info" },
                                { "name": "API_KEY", "valueFrom": { "secretKeyRef": { "name": "api-sec", "key": "key" } } }
                            ],
                            "envFrom": [
                                { "configMapRef": { "name": "shared-cfg" }, "prefix": "CFG_" }
                            ],
                            "ports": [{ "name": "http", "containerPort": 80, "protocol": "TCP" }],
                            "resources": {
                                "requests": { "cpu": "100m", "memory": "128Mi" },
                                "limits": { "cpu": "500m", "memory": "256Mi" }
                            },
                            "volumeMounts": [{ "name": "cfg", "mountPath": "/etc/nginx" }]
                        }]
                    }
                }
            },
            "status": { "availableReplicas": 3 }
        });

        let mut form = extract_edit_form("Deployment", &deploy);
        match &mut form {
            EditFormModel::Deployment(v) => {
                assert_eq!(v.replicas, "3");
                assert_eq!(v.strategy_type, "RollingUpdate");
                assert_eq!(v.max_surge, "25%");
                assert_eq!(v.max_unavailable, "1");
                assert_eq!(v.pod_spec.containers.len(), 1);
                let c = &mut v.pod_spec.containers[0];
                assert_eq!(c.name, "nginx");
                assert_eq!(c.environments.len(), 3);
                assert_eq!(c.ports.len(), 1);
                v.replicas = "5".to_string();
                c.image = "nginx:1.27".to_string();
                c.resources.cpu_limit = "1000m".to_string();
                c.ports.push(ContainerPortEntry {
                    name: "metrics".to_string(),
                    container_port: "9090".to_string(),
                    protocol: "TCP".to_string(),
                });
            }
            other => panic!("expected Deployment, got {other:?}"),
        }

        assert!(validate_edit_form(&form).is_none());
        let updated = apply_edit_form(&deploy, &form);
        assert_eq!(updated["metadata"]["resourceVersion"], "101");
        assert_eq!(updated["metadata"]["uid"], "dep-uid");
        assert_eq!(updated["status"]["availableReplicas"], 3);
        assert_eq!(updated["spec"]["replicas"], 5);
        assert_eq!(updated["spec"]["template"]["spec"]["volumes"][0]["name"], "cfg");
        assert_eq!(updated["spec"]["template"]["spec"]["containers"][0]["image"], "nginx:1.27");
        assert_eq!(updated["spec"]["template"]["spec"]["containers"][0]["resources"]["limits"]["cpu"], "1000m");
        assert_eq!(updated["spec"]["template"]["spec"]["containers"][0]["ports"][1]["containerPort"], 9090);
        assert_eq!(updated["spec"]["template"]["spec"]["containers"][0]["volumeMounts"][0]["mountPath"], "/etc/nginx");
    }

    #[test]
    fn test_statefulset_daemonset_replicaset_extract_apply_roundtrip() {
        // StatefulSet
        let sts = json!({
            "apiVersion": "apps/v1",
            "kind": "StatefulSet",
            "metadata": { "name": "db", "namespace": "data" },
            "spec": {
                "replicas": 3,
                "serviceName": "db-headless",
                "podManagementPolicy": "Parallel",
                "updateStrategy": { "type": "RollingUpdate", "rollingUpdate": { "partition": 1, "maxUnavailable": "1" } },
                "volumeClaimTemplates": [{
                    "metadata": { "name": "data" },
                    "spec": { "accessModes": ["ReadWriteOnce"], "storageClassName": "gp3", "resources": { "requests": { "storage": "20Gi" } } }
                }],
                "template": { "spec": { "containers": [{ "name": "postgres", "image": "postgres:16" }] } }
            }
        });
        let mut sts_form = extract_edit_form("StatefulSet", &sts);
        match &mut sts_form {
            EditFormModel::StatefulSet(v) => {
                assert_eq!(v.service_name, "db-headless");
                assert_eq!(v.pod_management_policy, "Parallel");
                assert_eq!(v.partition, "1");
                assert_eq!(v.volume_claim_summaries.len(), 1);
                assert!(v.volume_claim_summaries[0].contains("data"));
                assert!(v.volume_claim_summaries[0].contains("20Gi"));
                v.replicas = "4".to_string();
            }
            other => panic!("expected StatefulSet, got {other:?}"),
        }
        let sts_updated = apply_edit_form(&sts, &sts_form);
        assert_eq!(sts_updated["spec"]["replicas"], 4);
        assert_eq!(sts_updated["spec"]["volumeClaimTemplates"][0]["metadata"]["name"], "data");

        // DaemonSet
        let ds = json!({
            "apiVersion": "apps/v1",
            "kind": "DaemonSet",
            "metadata": { "name": "agent", "namespace": "kube-system" },
            "spec": {
                "updateStrategy": { "type": "RollingUpdate", "rollingUpdate": { "maxUnavailable": "1", "maxSurge": "0" } },
                "template": { "spec": { "hostNetwork": true, "containers": [{ "name": "agent", "image": "agent:v1" }] } }
            }
        });
        let mut ds_form = extract_edit_form("DaemonSet", &ds);
        match &mut ds_form {
            EditFormModel::DaemonSet(v) => {
                assert!(v.pod_spec.host_network);
                v.max_surge = "25%".to_string();
            }
            other => panic!("expected DaemonSet, got {other:?}"),
        }
        let ds_updated = apply_edit_form(&ds, &ds_form);
        assert_eq!(ds_updated["spec"]["updateStrategy"]["rollingUpdate"]["maxSurge"], "25%");
        assert_eq!(ds_updated["spec"]["template"]["spec"]["hostNetwork"], true);

        // ReplicaSet
        let rs = json!({
            "apiVersion": "apps/v1",
            "kind": "ReplicaSet",
            "metadata": { "name": "rs-web", "namespace": "default" },
            "spec": {
                "replicas": 2,
                "minReadySeconds": 10,
                "template": { "spec": { "containers": [{ "name": "app", "image": "app:1.0" }] } }
            }
        });
        let mut rs_form = extract_edit_form("ReplicaSet", &rs);
        match &mut rs_form {
            EditFormModel::ReplicaSet(v) => {
                assert_eq!(v.replicas, "2");
                v.replicas = "6".to_string();
            }
            other => panic!("expected ReplicaSet, got {other:?}"),
        }
        let rs_updated = apply_edit_form(&rs, &rs_form);
        assert_eq!(rs_updated["spec"]["replicas"], 6);
    }

    #[test]
    fn test_job_cronjob_pod_extract_apply_and_validation_errors() {
        // Job
        let job = json!({
            "apiVersion": "batch/v1",
            "kind": "Job",
            "metadata": { "name": "migrate", "namespace": "default" },
            "spec": {
                "completions": 1,
                "parallelism": 1,
                "backoffLimit": 4,
                "completionMode": "NonIndexed",
                "template": { "spec": { "restartPolicy": "Never", "containers": [{ "name": "job", "image": "busybox" }] } }
            }
        });
        let mut job_form = extract_edit_form("Job", &job);
        assert!(validate_edit_form(&job_form).is_none());
        if let EditFormModel::Job(ref mut v) = job_form {
            v.pod_spec.containers[0].working_dir = "relative/path".to_string();
        }
        assert!(validate_edit_form(&job_form).is_some());

        // CronJob
        let cj = json!({
            "apiVersion": "batch/v1",
            "kind": "CronJob",
            "metadata": { "name": "nightly", "namespace": "default" },
            "spec": {
                "schedule": "0 2 * * *",
                "concurrencyPolicy": "Forbid",
                "successfulJobsHistoryLimit": 3,
                "failedJobsHistoryLimit": 1,
                "jobTemplate": {
                    "spec": {
                        "template": {
                            "spec": { "restartPolicy": "OnFailure", "containers": [{ "name": "cron", "image": "alpine" }] }
                        }
                    }
                }
            }
        });
        let mut cj_form = extract_edit_form("CronJob", &cj);
        assert!(validate_edit_form(&cj_form).is_none());
        if let EditFormModel::CronJob(ref mut v) = cj_form {
            v.schedule = "bad-cron".to_string();
        }
        assert!(validate_edit_form(&cj_form).is_some());
        if let EditFormModel::CronJob(ref mut v) = cj_form {
            v.schedule = "@daily".to_string();
            v.suspend = true;
        }
        assert!(validate_edit_form(&cj_form).is_none());
        let cj_updated = apply_edit_form(&cj, &cj_form);
        assert_eq!(cj_updated["spec"]["schedule"], "@daily");
        assert_eq!(cj_updated["spec"]["suspend"], true);

        // Pod
        let pod = json!({
            "apiVersion": "v1",
            "kind": "Pod",
            "metadata": { "name": "debug-pod", "namespace": "default", "labels": { "env": "dev" } },
            "spec": {
                "restartPolicy": "Always",
                "containers": [{ "name": "debug", "image": "alpine:latest", "ports": [{ "containerPort": 8080, "protocol": "TCP" }] }]
            }
        });
        let mut pod_form = extract_edit_form("Pod", &pod);
        assert!(validate_edit_form(&pod_form).is_none());
        if let EditFormModel::Pod(ref mut v) = pod_form {
            v.pod_spec.containers[0].ports[0].container_port = "99999".to_string();
        }
        assert!(validate_edit_form(&pod_form).is_some());
        if let EditFormModel::Pod(ref mut v) = pod_form {
            v.pod_spec.containers[0].name = "INVALID_NAME".to_string();
        }
        assert!(validate_edit_form(&pod_form).is_some());

        // Empty image and incomplete env entries must fail validation
        let mut env_pod = extract_edit_form("Pod", &pod);
        if let EditFormModel::Pod(ref mut v) = env_pod {
            v.pod_spec.containers[0].image = "   ".to_string();
        }
        assert!(validate_edit_form(&env_pod).is_some());

        let mut env_pod2 = extract_edit_form("Pod", &pod);
        if let EditFormModel::Pod(ref mut v) = env_pod2 {
            v.pod_spec.containers[0].environments.push(ContainerEnvEntry {
                source_type: EnvSourceType::ConfigMapKey,
                name: "DB_HOST".to_string(),
                ref_name: "cfg".to_string(),
                ref_key: "".to_string(),
                ..Default::default()
            });
        }
        assert!(validate_edit_form(&env_pod2).is_some());
    }
}
