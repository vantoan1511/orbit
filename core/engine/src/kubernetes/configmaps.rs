use kube::{
    api::{Api, ListParams},
    Client,
};
use k8s_openapi::api::core::v1::ConfigMap;
use crate::kubernetes::models::ConfigMapInfo;
use crate::kubernetes::{format_age, format_size};

/// Formats a Kubernetes `ConfigMap` into `ConfigMapInfo`.
pub fn format_configmap(c: &ConfigMap) -> ConfigMapInfo {
    let name = c.metadata.name.clone().unwrap_or_default();
    let namespace = c.metadata.namespace.clone().unwrap_or_default();
    
    let age = format_age(&c.metadata.creation_timestamp);
    let created = c.metadata.creation_timestamp.as_ref()
        .map(|t| t.0.format("%b %d, %Y, %I:%M %p").to_string())
        .unwrap_or_default();
        
    let resource_version = c.metadata.resource_version.clone().unwrap_or_default();
    let immutable = c.immutable.unwrap_or(false);
    
    let data = c.data.clone().unwrap_or_default();
    let keys_count = data.len() as i32;
    
    let mut total_bytes = 0;
    for (k, v) in &data {
        total_bytes += k.len() + v.len();
    }
    if let Some(binary) = &c.binary_data {
        for (k, v) in binary {
            total_bytes += k.len() + v.0.len();
        }
    }
    let size = format_size(total_bytes);
    
    let annotations = c.metadata.annotations.as_ref().map(|a| a.len() as i32).unwrap_or(0);
    let labels = c.metadata.labels.clone().unwrap_or_default();
    
    // TODO: Implement pod mounting/usage check or resource dependency resolution.
    // Currently hardcoded as an optimization to prevent blocking Kubernetes API queries.
    let mounted_pods = 0;
    let used_by = Vec::new();
    
    ConfigMapInfo {
        name,
        namespace,
        labels,
        annotations,
        created,
        age,
        resource_version,
        immutable,
        keys_count,
        size,
        mounted_pods,
        used_by,
        data,
    }
}

/// Lists all ConfigMaps in the specified namespace or all namespaces.
pub async fn list_configmaps(client: &Client, namespace: Option<String>) -> Result<Vec<ConfigMapInfo>, kube::Error> {
    let api: Api<ConfigMap> = if let Some(ns) = namespace {
        Api::namespaced(client.clone(), &ns)
    } else {
        Api::all(client.clone())
    };

    let mut list = Vec::new();
    for c in api.list(&ListParams::default()).await? {
        list.push(format_configmap(&c));
    }
    
    Ok(list)
}

#[cfg(test)]
mod tests {
    use super::*;
    use k8s_openapi::apimachinery::pkg::apis::meta::v1::{ObjectMeta, Time};
    use k8s_openapi::ByteString;
    use std::collections::BTreeMap;
    use chrono::{TimeZone, Utc};

    #[test]
    fn test_format_configmap_basic() {
        let mut data = BTreeMap::new();
        data.insert("key1".to_string(), "val1".to_string());
        data.insert("key2".to_string(), "longer_value".to_string());

        let mut binary_data = BTreeMap::new();
        binary_data.insert("bin1".to_string(), ByteString(vec![0, 1, 2, 3]));

        let mut labels = BTreeMap::new();
        labels.insert("app".to_string(), "orbit".to_string());

        let creation_time = Utc.with_ymd_and_hms(2026, 7, 12, 12, 0, 0).unwrap();

        let cm = ConfigMap {
            metadata: ObjectMeta {
                name: Some("test-cm".to_string()),
                namespace: Some("default".to_string()),
                creation_timestamp: Some(Time(creation_time)),
                resource_version: Some("42".to_string()),
                labels: Some(labels),
                ..Default::default()
            },
            data: Some(data),
            binary_data: Some(binary_data),
            immutable: Some(true),
        };

        let info = format_configmap(&cm);

        assert_eq!(info.name, "test-cm");
        assert_eq!(info.namespace, "default");
        assert_eq!(info.resource_version, "42");
        assert!(info.immutable);
        assert_eq!(info.keys_count, 2);
        
        // Size should account for keys and values lengths:
        // data keys: "key1" (4) + "val1" (4) + "key2" (4) + "longer_value" (12) = 24 bytes
        // binary keys: "bin1" (4) + vec length (4) = 8 bytes
        // total = 32 bytes -> "32 B"
        assert_eq!(info.size, "32 B");
        assert_eq!(info.labels.get("app").map(|s| s.as_str()), Some("orbit"));
        assert_eq!(info.data.get("key1").map(|s| s.as_str()), Some("val1"));
    }
}
