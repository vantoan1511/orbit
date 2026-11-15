use std::time::Duration;

use k8s_openapi::api::core::v1::Namespace;
use k8s_openapi::apimachinery::pkg::apis::meta::v1::Time;
use kube::{
    api::{Api, ListParams},
    Client,
};

use crate::kubernetes::models::NamespaceInfo;

pub const NAMESPACE_FETCH_TIMEOUT: Duration = Duration::from_secs(10);

pub fn format_age(creation_timestamp: &Option<Time>) -> String {
    if let Some(creation) = creation_timestamp {
        let now = chrono::Utc::now();
        let duration = now.signed_duration_since(creation.0);
        let seconds = duration.num_seconds();
        if seconds < 0 {
            "0s".to_string()
        } else if seconds < 60 {
            format!("{}s", seconds)
        } else if seconds < 3600 {
            format!("{}m", duration.num_minutes())
        } else if seconds < 86400 {
            format!("{}h", duration.num_hours())
        } else {
            format!("{}d", duration.num_days())
        }
    } else {
        "Unknown".to_string()
    }
}

pub fn map_namespace(ns: &Namespace) -> NamespaceInfo {
    let name = ns.metadata.name.clone().unwrap_or_default();
    let is_system = name.starts_with("kube-")
        || name == "default"
        || name.ends_with("-system")
        || name == "cert-manager"
        || name == "monitoring"
        || name == "ingress-nginx";

    let status = ns
        .status
        .as_ref()
        .and_then(|s| s.phase.clone())
        .unwrap_or_else(|| "Unknown".to_string());

    let age = format_age(&ns.metadata.creation_timestamp);
    let labels = ns.metadata.labels.clone().unwrap_or_default();
    let annotations = ns.metadata.annotations.clone().unwrap_or_default();
    let uid = ns.metadata.uid.clone().unwrap_or_default();
    let created = ns
        .metadata
        .creation_timestamp
        .as_ref()
        .map(|t| t.0.to_rfc3339())
        .unwrap_or_default();

    NamespaceInfo {
        name,
        status,
        is_system,
        age,
        labels,
        annotations,
        uid,
        created,
    }
}

pub async fn list_namespaces(client: &Client) -> Result<Vec<NamespaceInfo>, kube::Error> {
    let namespaces_api: Api<Namespace> = Api::all(client.clone());
    let mut list = Vec::new();

    for ns in namespaces_api.list(&ListParams::default()).await? {
        list.push(map_namespace(&ns));
    }

    list.sort_by(|a, b| a.name.cmp(&b.name));

    Ok(list)
}

pub async fn fetch_namespaces(client: &Client) -> Result<Vec<NamespaceInfo>, String> {
    match tokio::time::timeout(NAMESPACE_FETCH_TIMEOUT, list_namespaces(client)).await {
        Ok(Ok(list)) => Ok(list),
        Ok(Err(kube::Error::Api(api_err))) => Err(api_err.message),
        Ok(Err(err)) => Err(err.to_string()),
        Err(_) => Err("Timed out fetching namespaces after 10s".to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;
    use k8s_openapi::api::core::v1::NamespaceStatus;
    use k8s_openapi::apimachinery::pkg::apis::meta::v1::ObjectMeta;

    fn make_namespace(name: &str, phase: Option<&str>, age_secs: i64) -> Namespace {
        let mut labels = BTreeMap::new();
        labels.insert("kubernetes.io/metadata.name".to_string(), name.to_string());

        let mut annotations = BTreeMap::new();
        annotations.insert("owner".to_string(), "platform".to_string());

        let ts = Time(chrono::Utc::now() - chrono::Duration::seconds(age_secs));

        Namespace {
            metadata: ObjectMeta {
                name: Some(name.to_string()),
                uid: Some(format!("uid-{name}")),
                labels: Some(labels),
                annotations: Some(annotations),
                creation_timestamp: Some(ts),
                ..Default::default()
            },
            status: phase.map(|p| NamespaceStatus {
                phase: Some(p.to_string()),
                ..Default::default()
            }),
            ..Default::default()
        }
    }

    #[test]
    fn test_map_namespace_extracts_status_system_classification_and_metadata() {
        let default_ns = map_namespace(&make_namespace("default", Some("Active"), 120));
        let kube_sys = map_namespace(&make_namespace("kube-system", Some("Active"), 7200));
        let ingress = map_namespace(&make_namespace("ingress-nginx", Some("Terminating"), 90000));
        let payments = map_namespace(&make_namespace("payments-prod", None, 30));

        assert!(default_ns.is_system);
        assert_eq!(default_ns.status, "Active");
        assert_eq!(default_ns.uid, "uid-default");
        assert_eq!(
            default_ns
                .labels
                .get("kubernetes.io/metadata.name")
                .map(String::as_str),
            Some("default")
        );
        assert_eq!(
            default_ns.annotations.get("owner").map(String::as_str),
            Some("platform")
        );
        assert!(!default_ns.created.is_empty());
        assert!(!default_ns.age.is_empty());

        assert!(kube_sys.is_system);
        assert_eq!(kube_sys.status, "Active");

        assert!(ingress.is_system);
        assert_eq!(ingress.status, "Terminating");

        assert!(!payments.is_system);
        assert_eq!(payments.status, "Unknown");
    }
}
