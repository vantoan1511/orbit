use kube::{
    api::{Api, ListParams},
    Client,
};
use k8s_openapi::api::core::v1::Secret;
use base64::engine::general_purpose::STANDARD as BASE64_STANDARD;
use base64::Engine as _;
use crate::kubernetes::models::SecretInfo;
use crate::kubernetes::{format_age, format_size};

/// Formats a Kubernetes `Secret` into `SecretInfo`.
pub fn format_secret(s: &Secret) -> SecretInfo {
    let name = s.metadata.name.clone().unwrap_or_default();
    let namespace = s.metadata.namespace.clone().unwrap_or_default();
    
    let age = format_age(&s.metadata.creation_timestamp);
    let created = s.metadata.creation_timestamp.as_ref()
        .map(|t| t.0.format("%b %d, %Y, %I:%M %p").to_string())
        .unwrap_or_default();
        
    let resource_version = s.metadata.resource_version.clone().unwrap_or_default();
    let immutable = s.immutable.unwrap_or(false);
    let secret_type = s.type_.clone().unwrap_or_else(|| "Opaque".to_string());
    
    let raw_data = s.data.clone().unwrap_or_default();
    let keys_count = raw_data.len() as i32;
    
    let mut total_bytes = 0;
    let mut data = std::collections::BTreeMap::new();
    for (k, v) in &raw_data {
        total_bytes += k.len() + v.0.len();
        let base64_str = BASE64_STANDARD.encode(&v.0);
        data.insert(k.clone(), base64_str);
    }
    let size = format_size(total_bytes);
    
    let annotations = s.metadata.annotations.as_ref().map(|a| a.len() as i32).unwrap_or(0);
    let labels = s.metadata.labels.clone().unwrap_or_default();
    
    // TODO: Implement pod mounting/usage check or resource dependency resolution.
    // Currently hardcoded as an optimization to prevent blocking Kubernetes API queries.
    let mounted_pods = 0;
    let used_by = Vec::new();
    
    SecretInfo {
        name,
        namespace,
        labels,
        annotations,
        r#type: secret_type,
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

/// Lists all Secrets in the specified namespace or all namespaces.
pub async fn list_secrets(client: &Client, namespace: Option<String>) -> Result<Vec<SecretInfo>, kube::Error> {
    let api: Api<Secret> = if let Some(ns) = namespace {
        Api::namespaced(client.clone(), &ns)
    } else {
        Api::all(client.clone())
    };

    let mut list = Vec::new();
    for s in api.list(&ListParams::default()).await? {
        list.push(format_secret(&s));
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
    fn test_format_secret_basic() {
        let mut raw_data = BTreeMap::new();
        raw_data.insert("username".to_string(), ByteString(b"admin".to_vec()));
        raw_data.insert("password".to_string(), ByteString(b"secret123".to_vec()));

        let mut labels = BTreeMap::new();
        labels.insert("type".to_string(), "credential".to_string());

        let creation_time = Utc.with_ymd_and_hms(2026, 7, 12, 12, 0, 0).unwrap();

        let secret = Secret {
            metadata: ObjectMeta {
                name: Some("test-secret".to_string()),
                namespace: Some("kube-system".to_string()),
                creation_timestamp: Some(Time(creation_time)),
                resource_version: Some("123".to_string()),
                labels: Some(labels),
                ..Default::default()
            },
            data: Some(raw_data),
            immutable: Some(false),
            type_: Some("Opaque".to_string()),
            ..Default::default()
        };

        let info = format_secret(&secret);

        assert_eq!(info.name, "test-secret");
        assert_eq!(info.namespace, "kube-system");
        assert_eq!(info.r#type, "Opaque");
        assert_eq!(info.keys_count, 2);
        assert!(!info.immutable);

        // Size:
        // "username" (8) + "admin" (5) = 13
        // "password" (8) + "secret123" (9) = 17
        // total = 30 -> "30 B"
        assert_eq!(info.size, "30 B");

        // Base64 encoded values:
        // "admin" -> "YWRtaW4="
        // "secret123" -> "c2VjcmV0MTIz"
        assert_eq!(info.data.get("username").map(|s| s.as_str()), Some("YWRtaW4="));
        assert_eq!(info.data.get("password").map(|s| s.as_str()), Some("c2VjcmV0MTIz"));
    }
}
