use std::time::Duration;

use k8s_openapi::api::admissionregistration::v1::{
    MutatingWebhookConfiguration, ValidatingWebhookConfiguration,
};
use k8s_openapi::api::apps::v1::{DaemonSet, Deployment, ReplicaSet, StatefulSet};
use k8s_openapi::api::autoscaling::v2::HorizontalPodAutoscaler;
use k8s_openapi::api::batch::v1::{CronJob, Job};
use k8s_openapi::api::core::v1::{
    ConfigMap, LimitRange, Namespace, Node, PersistentVolume, PersistentVolumeClaim, Pod,
    ResourceQuota, Secret, Service,
};
use k8s_openapi::api::networking::v1::{Ingress, NetworkPolicy};
use k8s_openapi::api::storage::v1::StorageClass;
use kube::{api::Api, Client};
use serde_json::{json, Value};

use crate::kubernetes::models::{EventInfo, ResourceApiScope};

pub const RESOURCE_YAML_FETCH_TIMEOUT: Duration = Duration::from_secs(10);

pub fn resolve_resource_scope(kind: &str) -> Option<ResourceApiScope> {
    match kind {
        "Pod"
        | "Deployment"
        | "StatefulSet"
        | "DaemonSet"
        | "ReplicaSet"
        | "Job"
        | "CronJob"
        | "Service"
        | "ConfigMap"
        | "Secret"
        | "PersistentVolumeClaim"
        | "NetworkPolicy"
        | "Ingress"
        | "HorizontalPodAutoscaler"
        | "ResourceQuota"
        | "LimitRange" => Some(ResourceApiScope::Namespaced),
        "Node"
        | "Namespace"
        | "PersistentVolume"
        | "StorageClass"
        | "ValidatingWebhookConfiguration"
        | "MutatingWebhookConfiguration" => Some(ResourceApiScope::ClusterScoped),
        _ => None,
    }
}

fn api_error(status: &str, message: String, reason: &str, code: u16) -> kube::Error {
    kube::Error::Api(kube::error::ErrorResponse {
        status: status.to_string(),
        message,
        reason: reason.to_string(),
        code,
    })
}

fn default_api_version(kind: &str) -> &'static str {
    match kind {
        "Pod"
        | "Service"
        | "ConfigMap"
        | "Secret"
        | "PersistentVolume"
        | "PersistentVolumeClaim"
        | "Namespace"
        | "Node"
        | "ResourceQuota"
        | "LimitRange" => "v1",
        "Deployment" | "StatefulSet" | "DaemonSet" | "ReplicaSet" => "apps/v1",
        "Job" | "CronJob" => "batch/v1",
        "HorizontalPodAutoscaler" => "autoscaling/v2",
        "NetworkPolicy" | "Ingress" => "networking.k8s.io/v1",
        "ValidatingWebhookConfiguration" | "MutatingWebhookConfiguration" => {
            "admissionregistration.k8s.io/v1"
        }
        "StorageClass" => "storage.k8s.io/v1",
        _ => "",
    }
}

fn to_json_val<T: serde::Serialize>(val: &T, kind: &str) -> Result<Value, kube::Error> {
    let mut json = serde_json::to_value(val).map_err(|e| {
        api_error(
            "Failure",
            format!("Failed to serialize {kind}: {e}"),
            "InternalError",
            500,
        )
    })?;

    if let Value::Object(ref mut map) = json {
        if !map.contains_key("kind") || map.get("kind").map(|v| v.is_null()).unwrap_or(false) {
            map.insert("kind".to_string(), Value::String(kind.to_string()));
        }
        if !map.contains_key("apiVersion")
            || map
                .get("apiVersion")
                .map(|v| v.is_null())
                .unwrap_or(false)
        {
            let api_version = default_api_version(kind);
            if !api_version.is_empty() {
                map.insert(
                    "apiVersion".to_string(),
                    Value::String(api_version.to_string()),
                );
            }
        }
    }

    Ok(json)
}

async fn get_namespaced_raw<K>(
    client: &Client,
    namespace: &str,
    kind: &str,
    name: &str,
) -> Result<Value, kube::Error>
where
    K: kube::Resource<Scope = k8s_openapi::NamespaceResourceScope>
        + serde::Serialize
        + serde::de::DeserializeOwned
        + Clone
        + std::fmt::Debug,
    <K as kube::Resource>::DynamicType: Default,
{
    let api: Api<K> = Api::namespaced(client.clone(), namespace);
    let resource = api.get(name).await?;
    to_json_val(&resource, kind)
}

async fn get_cluster_raw<K>(client: &Client, kind: &str, name: &str) -> Result<Value, kube::Error>
where
    K: kube::Resource<Scope = k8s_openapi::ClusterResourceScope>
        + serde::Serialize
        + serde::de::DeserializeOwned
        + Clone
        + std::fmt::Debug,
    <K as kube::Resource>::DynamicType: Default,
{
    let api: Api<K> = Api::all(client.clone());
    let resource = api.get(name).await?;
    to_json_val(&resource, kind)
}

pub async fn get_resource_raw(
    client: &Client,
    namespace: Option<&str>,
    kind: &str,
    name: &str,
) -> Result<Value, kube::Error> {
    let name = name.trim();
    if name.is_empty() {
        return Err(api_error(
            "Failure",
            format!("Resource name is required for {kind}"),
            "BadRequest",
            400,
        ));
    }

    let scope = resolve_resource_scope(kind);
    if kind != "Policy" && scope.is_none() {
        return Err(api_error(
            "Failure",
            format!("Unsupported get resource kind: {kind}"),
            "BadRequest",
            400,
        ));
    }

    let require_namespace = || -> Result<&str, kube::Error> {
        namespace
            .map(str::trim)
            .filter(|ns| !ns.is_empty())
            .ok_or_else(|| {
                api_error(
                    "Failure",
                    format!("Namespace is required for {kind}"),
                    "BadRequest",
                    400,
                )
            })
    };

    match kind {
        "Pod" => get_namespaced_raw::<Pod>(client, require_namespace()?, kind, name).await,
        "Deployment" => {
            get_namespaced_raw::<Deployment>(client, require_namespace()?, kind, name).await
        }
        "StatefulSet" => {
            get_namespaced_raw::<StatefulSet>(client, require_namespace()?, kind, name).await
        }
        "DaemonSet" => {
            get_namespaced_raw::<DaemonSet>(client, require_namespace()?, kind, name).await
        }
        "ReplicaSet" => {
            get_namespaced_raw::<ReplicaSet>(client, require_namespace()?, kind, name).await
        }
        "Job" => get_namespaced_raw::<Job>(client, require_namespace()?, kind, name).await,
        "CronJob" => get_namespaced_raw::<CronJob>(client, require_namespace()?, kind, name).await,
        "Service" => get_namespaced_raw::<Service>(client, require_namespace()?, kind, name).await,
        "ConfigMap" => {
            get_namespaced_raw::<ConfigMap>(client, require_namespace()?, kind, name).await
        }
        "Secret" => get_namespaced_raw::<Secret>(client, require_namespace()?, kind, name).await,
        "PersistentVolumeClaim" => {
            get_namespaced_raw::<PersistentVolumeClaim>(client, require_namespace()?, kind, name)
                .await
        }
        "NetworkPolicy" => {
            get_namespaced_raw::<NetworkPolicy>(client, require_namespace()?, kind, name).await
        }
        "Ingress" => get_namespaced_raw::<Ingress>(client, require_namespace()?, kind, name).await,
        "HorizontalPodAutoscaler" => {
            get_namespaced_raw::<HorizontalPodAutoscaler>(client, require_namespace()?, kind, name)
                .await
        }
        "ResourceQuota" => {
            get_namespaced_raw::<ResourceQuota>(client, require_namespace()?, kind, name).await
        }
        "LimitRange" => {
            get_namespaced_raw::<LimitRange>(client, require_namespace()?, kind, name).await
        }
        "Node" => get_cluster_raw::<Node>(client, kind, name).await,
        "Namespace" => get_cluster_raw::<Namespace>(client, kind, name).await,
        "PersistentVolume" => get_cluster_raw::<PersistentVolume>(client, kind, name).await,
        "StorageClass" => get_cluster_raw::<StorageClass>(client, kind, name).await,
        "ValidatingWebhookConfiguration" => {
            get_cluster_raw::<ValidatingWebhookConfiguration>(client, kind, name).await
        }
        "MutatingWebhookConfiguration" => {
            get_cluster_raw::<MutatingWebhookConfiguration>(client, kind, name).await
        }
        "Policy" => {
            let ns = namespace.map(str::trim).filter(|s| !s.is_empty());
            if let Some(ns) = ns {
                if let Ok(res) =
                    get_namespaced_raw::<NetworkPolicy>(client, ns, "NetworkPolicy", name).await
                {
                    return Ok(res);
                }
                if let Ok(res) =
                    get_namespaced_raw::<ResourceQuota>(client, ns, "ResourceQuota", name).await
                {
                    return Ok(res);
                }
                if let Ok(res) =
                    get_namespaced_raw::<LimitRange>(client, ns, "LimitRange", name).await
                {
                    return Ok(res);
                }
            }
            if let Ok(res) = get_cluster_raw::<ValidatingWebhookConfiguration>(
                client,
                "ValidatingWebhookConfiguration",
                name,
            )
            .await
            {
                return Ok(res);
            }
            if let Ok(res) = get_cluster_raw::<MutatingWebhookConfiguration>(
                client,
                "MutatingWebhookConfiguration",
                name,
            )
            .await
            {
                return Ok(res);
            }
            Err(api_error(
                "Failure",
                format!("Policy resource not found: {name}"),
                "NotFound",
                404,
            ))
        }
        _ => Err(api_error(
            "Failure",
            format!("Unsupported get resource kind: {kind}"),
            "BadRequest",
            400,
        )),
    }
}

pub fn format_raw_resource_yaml(value: &Value) -> Result<String, String> {
    serde_yaml::to_string(value).map_err(|e| format!("Failed to format YAML: {e}"))
}

pub async fn fetch_resource_yaml(
    client: &Client,
    namespace: Option<&str>,
    kind: &str,
    name: &str,
) -> Result<String, String> {
    let raw = tokio::time::timeout(
        RESOURCE_YAML_FETCH_TIMEOUT,
        get_resource_raw(client, namespace, kind, name),
    )
    .await
    .map_err(|_| {
        format!(
            "Timed out after {}s fetching {kind} '{name}'",
            RESOURCE_YAML_FETCH_TIMEOUT.as_secs()
        )
    })?
    .map_err(|e| format!("Failed to fetch {kind} '{name}': {e}"))?;

    format_raw_resource_yaml(&raw)
}

pub fn format_event_yaml(event: &EventInfo) -> String {
    let payload = json!({
        "apiVersion": "v1",
        "kind": "Event",
        "metadata": {
            "name": event.uid,
            "namespace": event.namespace,
            "uid": event.uid,
            "labels": event.labels,
            "annotations": event.annotations,
        },
        "type": event.r#type,
        "reason": event.reason,
        "message": event.message,
        "count": event.count,
        "firstTimestamp": event.first_seen,
        "lastTimestamp": event.last_seen,
        "source": {
            "component": event.source,
        },
        "involvedObject": {
            "kind": event.object_kind,
            "name": event.object_name,
            "namespace": event.namespace,
        }
    });

    format_raw_resource_yaml(&payload).unwrap_or_else(|e| format!("# Failed to format event: {e}\n"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;

    #[test]
    fn test_resolve_resource_scope_handles_namespaced_cluster_and_unknown_kinds() {
        assert_eq!(
            resolve_resource_scope("Deployment"),
            Some(ResourceApiScope::Namespaced)
        );
        assert_eq!(
            resolve_resource_scope("Pod"),
            Some(ResourceApiScope::Namespaced)
        );
        assert_eq!(
            resolve_resource_scope("Service"),
            Some(ResourceApiScope::Namespaced)
        );
        assert_eq!(
            resolve_resource_scope("Node"),
            Some(ResourceApiScope::ClusterScoped)
        );
        assert_eq!(
            resolve_resource_scope("Namespace"),
            Some(ResourceApiScope::ClusterScoped)
        );
        assert_eq!(
            resolve_resource_scope("PersistentVolume"),
            Some(ResourceApiScope::ClusterScoped)
        );
        assert_eq!(
            resolve_resource_scope("StorageClass"),
            Some(ResourceApiScope::ClusterScoped)
        );
        assert_eq!(resolve_resource_scope("UnknownKind"), None);
    }

    #[test]
    fn test_format_raw_resource_yaml_injects_expected_yaml_structure() {
        let value = json!({
            "apiVersion": "apps/v1",
            "kind": "Deployment",
            "metadata": {
                "name": "web",
                "namespace": "default"
            },
            "spec": {
                "replicas": 3
            }
        });

        let yaml = format_raw_resource_yaml(&value).expect("should format valid json to yaml");
        assert!(yaml.contains("apiVersion: apps/v1"));
        assert!(yaml.contains("kind: Deployment"));
        assert!(yaml.contains("metadata:"));
        assert!(yaml.contains("spec:"));
    }

    #[test]
    fn test_format_event_yaml_formats_event_metadata_and_involved_object() {
        let mut labels = BTreeMap::new();
        labels.insert("app".to_string(), "web".to_string());
        let mut annotations = BTreeMap::new();
        annotations.insert("note".to_string(), "test".to_string());

        let event = EventInfo {
            uid: "evt-123".to_string(),
            time: "2m".to_string(),
            r#type: "Normal".to_string(),
            reason: "Scheduled".to_string(),
            object_name: "web-abc".to_string(),
            object_kind: "Pod".to_string(),
            message: "Successfully assigned default/web-abc to node-1".to_string(),
            namespace: "default".to_string(),
            source: "default-scheduler".to_string(),
            first_seen: "2026-01-01T00:00:00Z".to_string(),
            last_seen: "2026-01-01T00:01:00Z".to_string(),
            count: 1,
            labels,
            annotations,
        };

        let yaml = format_event_yaml(&event);
        assert!(yaml.contains("kind: Event"));
        assert!(yaml.contains("involvedObject:"));
        assert!(yaml.contains("reason: Scheduled"));
        assert!(yaml.contains("message: Successfully assigned default/web-abc to node-1"));
        assert!(yaml.contains("count: 1"));
        assert!(yaml.contains("labels:"));
        assert!(yaml.contains("app: web"));
    }

    #[tokio::test]
    async fn test_get_resource_raw_rejects_unsupported_kind_and_missing_namespace() {
        let uri: http::Uri = "http://127.0.0.1:1".parse().expect("valid uri");
        let config = kube::Config::new(uri);
        let client = kube::Client::try_from(config).expect("valid dummy client");

        let unsupported = get_resource_raw(&client, Some("default"), "CustomThing", "foo").await;
        assert!(unsupported.is_err());
        let err_msg = unsupported.unwrap_err().to_string();
        assert!(
            err_msg.contains("Unsupported"),
            "unexpected error: {err_msg}"
        );

        let missing_ns = get_resource_raw(&client, None, "Deployment", "web").await;
        assert!(missing_ns.is_err());
        let err_msg_ns = missing_ns.unwrap_err().to_string();
        assert!(
            err_msg_ns.contains("Namespace") || err_msg_ns.contains("namespace"),
            "unexpected error: {err_msg_ns}"
        );

        let empty_ns = get_resource_raw(&client, Some("   "), "Deployment", "web").await;
        assert!(empty_ns.is_err());

        let empty_name = get_resource_raw(&client, Some("default"), "Deployment", "   ").await;
        assert!(empty_name.is_err());
    }
}
