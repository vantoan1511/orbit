use kube::{
    api::{Api, PostParams},
    Client,
};
use k8s_openapi::apimachinery::pkg::apis::meta::v1::ObjectMeta;
use k8s_openapi::Metadata;
use k8s_openapi::api::core::v1::{Pod, Service, ConfigMap, Secret, PersistentVolume, PersistentVolumeClaim, Node, Namespace, ResourceQuota, LimitRange};
use k8s_openapi::api::apps::v1::{Deployment, StatefulSet, DaemonSet, ReplicaSet};
use k8s_openapi::api::batch::v1::{Job, CronJob};
use k8s_openapi::api::networking::v1::{NetworkPolicy, Ingress};
use k8s_openapi::api::admissionregistration::v1::{ValidatingWebhookConfiguration, MutatingWebhookConfiguration};
use k8s_openapi::api::storage::v1::StorageClass;
use serde_json::Value;
use crate::kubernetes::models::ResourceApiScope;

pub(crate) fn resolve_resource_scope(kind: &str) -> Option<ResourceApiScope> {
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

fn to_json_val<T: serde::Serialize>(val: &T, kind: &str) -> Result<Value, kube::Error> {
    serde_json::to_value(val).map_err(|e| {
        kube::Error::Api(kube::error::ErrorResponse {
            status: "Failure".to_string(),
            message: format!("Failed to serialize {}: {}", kind, e),
            reason: "InternalError".to_string(),
            code: 500,
        })
    })
}

pub async fn get_resource_raw(
    client: &Client,
    namespace: &str,
    kind: &str,
    name: &str,
) -> Result<Value, kube::Error> {
    if kind != "Policy" && resolve_resource_scope(kind).is_none() {
        return Err(kube::Error::Api(kube::error::ErrorResponse {
            status: "Failure".to_string(),
            message: format!("Unsupported get resource kind: {}", kind),
            reason: "BadRequest".to_string(),
            code: 400,
        }));
    }

    match kind {
        "Pod" => {
            let api: Api<Pod> = Api::namespaced(client.clone(), namespace);
            let resource = api.get(name).await?;
            to_json_val(&resource, kind)
        }
        "Deployment" => {
            let api: Api<Deployment> = Api::namespaced(client.clone(), namespace);
            let resource = api.get(name).await?;
            to_json_val(&resource, kind)
        }
        "StatefulSet" => {
            let api: Api<StatefulSet> = Api::namespaced(client.clone(), namespace);
            let resource = api.get(name).await?;
            to_json_val(&resource, kind)
        }
        "DaemonSet" => {
            let api: Api<DaemonSet> = Api::namespaced(client.clone(), namespace);
            let resource = api.get(name).await?;
            to_json_val(&resource, kind)
        }
        "ReplicaSet" => {
            let api: Api<ReplicaSet> = Api::namespaced(client.clone(), namespace);
            let resource = api.get(name).await?;
            to_json_val(&resource, kind)
        }
        "Job" => {
            let api: Api<Job> = Api::namespaced(client.clone(), namespace);
            let resource = api.get(name).await?;
            to_json_val(&resource, kind)
        }
        "CronJob" => {
            let api: Api<CronJob> = Api::namespaced(client.clone(), namespace);
            let resource = api.get(name).await?;
            to_json_val(&resource, kind)
        }
        "Service" => {
            let api: Api<Service> = Api::namespaced(client.clone(), namespace);
            let resource = api.get(name).await?;
            to_json_val(&resource, kind)
        }
        "ConfigMap" => {
            let api: Api<ConfigMap> = Api::namespaced(client.clone(), namespace);
            let resource = api.get(name).await?;
            to_json_val(&resource, kind)
        }
        "Secret" => {
            let api: Api<Secret> = Api::namespaced(client.clone(), namespace);
            let resource = api.get(name).await?;
            to_json_val(&resource, kind)
        }
        "PersistentVolumeClaim" => {
            let api: Api<PersistentVolumeClaim> = Api::namespaced(client.clone(), namespace);
            let resource = api.get(name).await?;
            to_json_val(&resource, kind)
        }
        "NetworkPolicy" => {
            let api: Api<NetworkPolicy> = Api::namespaced(client.clone(), namespace);
            let resource = api.get(name).await?;
            to_json_val(&resource, kind)
        }
        "Ingress" => {
            let api: Api<Ingress> = Api::namespaced(client.clone(), namespace);
            let resource = api.get(name).await?;
            to_json_val(&resource, kind)
        }
        "Node" => {
            let api: Api<Node> = Api::all(client.clone());
            let resource = api.get(name).await?;
            to_json_val(&resource, kind)
        }
        "Namespace" => {
            let api: Api<Namespace> = Api::all(client.clone());
            let resource = api.get(name).await?;
            to_json_val(&resource, kind)
        }
        "ResourceQuota" => {
            let api: Api<ResourceQuota> = Api::namespaced(client.clone(), namespace);
            let resource = api.get(name).await?;
            to_json_val(&resource, kind)
        }
        "LimitRange" => {
            let api: Api<LimitRange> = Api::namespaced(client.clone(), namespace);
            let resource = api.get(name).await?;
            to_json_val(&resource, kind)
        }
        "ValidatingWebhookConfiguration" => {
            let api: Api<ValidatingWebhookConfiguration> = Api::all(client.clone());
            let resource = api.get(name).await?;
            to_json_val(&resource, kind)
        }
        "MutatingWebhookConfiguration" => {
            let api: Api<MutatingWebhookConfiguration> = Api::all(client.clone());
            let resource = api.get(name).await?;
            to_json_val(&resource, kind)
        }
        "PersistentVolume" => {
            let api: Api<PersistentVolume> = Api::all(client.clone());
            let resource = api.get(name).await?;
            to_json_val(&resource, kind)
        }
        "StorageClass" => {
            let api: Api<StorageClass> = Api::all(client.clone());
            let resource = api.get(name).await?;
            to_json_val(&resource, kind)
        }
        "Policy" => {
            // Attempt resolution across the 5 policy types supported by Orbit
            let net_api: Api<NetworkPolicy> = Api::namespaced(client.clone(), namespace);
            if let Ok(res) = net_api.get(name).await {
                return to_json_val(&res, "NetworkPolicy");
            }
            let quota_api: Api<ResourceQuota> = Api::namespaced(client.clone(), namespace);
            if let Ok(res) = quota_api.get(name).await {
                return to_json_val(&res, "ResourceQuota");
            }
            let limit_api: Api<LimitRange> = Api::namespaced(client.clone(), namespace);
            if let Ok(res) = limit_api.get(name).await {
                return to_json_val(&res, "LimitRange");
            }
            let val_api: Api<ValidatingWebhookConfiguration> = Api::all(client.clone());
            if let Ok(res) = val_api.get(name).await {
                return to_json_val(&res, "ValidatingWebhookConfiguration");
            }
            let mut_api: Api<MutatingWebhookConfiguration> = Api::all(client.clone());
            if let Ok(res) = mut_api.get(name).await {
                return to_json_val(&res, "MutatingWebhookConfiguration");
            }
            Err(kube::Error::Api(kube::error::ErrorResponse {
                status: "Failure".to_string(),
                message: format!("Policy resource not found: {}", name),
                reason: "NotFound".to_string(),
                code: 404,
            }))
        }
        _ => Err(kube::Error::Api(kube::error::ErrorResponse {
            status: "Failure".to_string(),
            message: format!("Unsupported get resource kind: {}", kind),
            reason: "BadRequest".to_string(),
            code: 400,
        })),
    }
}

pub fn is_conflict_error(error: &kube::Error) -> bool {
    matches!(error, kube::Error::Api(err) if err.code == 409)
}

pub fn sync_resource_version(target: &mut ObjectMeta, source: &ObjectMeta) {
    if let Some(ref version) = source.resource_version {
        target.resource_version = Some(version.clone());
    }
}

pub async fn replace_with_conflict_retry<K>(
    api: &Api<K>,
    name: &str,
    post_params: &PostParams,
    parsed: &mut K,
    max_attempts: usize,
) -> Result<K, kube::Error>
where
    K: kube::Resource<DynamicType = ()>
        + Clone
        + serde::de::DeserializeOwned
        + serde::Serialize
        + std::fmt::Debug
        + Metadata<Ty = ObjectMeta>,
{
    if parsed
        .metadata()
        .resource_version
        .as_ref()
        .map_or(true, |v| v.trim().is_empty())
    {
        if let Ok(latest) = api.get(name).await {
            sync_resource_version(parsed.metadata_mut(), latest.metadata());
        }
    }

    let mut attempts = 0;
    loop {
        attempts += 1;
        match api.replace(name, post_params, parsed).await {
            Ok(updated) => return Ok(updated),
            Err(e) if is_conflict_error(&e) && attempts < max_attempts => {
                tracing::warn!(
                    "Resource version conflict (HTTP 409) updating {} '{}', retrying ({}/{})",
                    K::kind(&()),
                    name,
                    attempts,
                    max_attempts
                );
                let latest = api.get(name).await?;
                sync_resource_version(parsed.metadata_mut(), latest.metadata());
                tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
            }
            Err(e) => return Err(e),
        }
    }
}

pub async fn apply_resource(
    client: &Client,
    namespace: &str,
    kind: &str,
    name: &str,
    raw_json: Value,
) -> Result<Value, kube::Error> {
    let post_params = PostParams::default();

    macro_rules! replace_resource {
        ($api_type:ty, $resolved_kind:expr) => {{
            let api: Api<$api_type> = Api::namespaced(client.clone(), namespace);
            let mut parsed: $api_type = serde_json::from_value(raw_json).map_err(|e| {
                kube::Error::Api(kube::error::ErrorResponse {
                    status: "Failure".to_string(),
                    message: format!("Failed to parse {}: {}", kind, e),
                    reason: "BadRequest".to_string(),
                    code: 400,
                })
            })?;
            let updated = replace_with_conflict_retry(&api, name, &post_params, &mut parsed, 3).await?;
            to_json_val(&updated, $resolved_kind)
        }};
    }

    macro_rules! replace_cluster_resource {
        ($api_type:ty, $resolved_kind:expr) => {{
            let api: Api<$api_type> = Api::all(client.clone());
            let mut parsed: $api_type = serde_json::from_value(raw_json).map_err(|e| {
                kube::Error::Api(kube::error::ErrorResponse {
                    status: "Failure".to_string(),
                    message: format!("Failed to parse {}: {}", kind, e),
                    reason: "BadRequest".to_string(),
                    code: 400,
                })
            })?;
            let updated = replace_with_conflict_retry(&api, name, &post_params, &mut parsed, 3).await?;
            to_json_val(&updated, $resolved_kind)
        }};
    }

    let resolved_kind = if kind == "Policy" {
        raw_json
            .get("kind")
            .and_then(|k| k.as_str())
            .unwrap_or(kind)
            .to_string()
    } else {
        kind.to_string()
    };

    if resolve_resource_scope(&resolved_kind).is_none() {
        return Err(kube::Error::Api(kube::error::ErrorResponse {
            status: "Failure".to_string(),
            message: format!("Unsupported apply resource kind: {}", kind),
            reason: "BadRequest".to_string(),
            code: 400,
        }));
    }

    match resolved_kind.as_str() {
        "Pod" => replace_resource!(Pod, &resolved_kind),
        "Deployment" => replace_resource!(Deployment, &resolved_kind),
        "StatefulSet" => replace_resource!(StatefulSet, &resolved_kind),
        "DaemonSet" => replace_resource!(DaemonSet, &resolved_kind),
        "ReplicaSet" => replace_resource!(ReplicaSet, &resolved_kind),
        "Job" => replace_resource!(Job, &resolved_kind),
        "CronJob" => replace_resource!(CronJob, &resolved_kind),
        "Service" => replace_resource!(Service, &resolved_kind),
        "ConfigMap" => replace_resource!(ConfigMap, &resolved_kind),
        "Secret" => replace_resource!(Secret, &resolved_kind),
        "PersistentVolumeClaim" => replace_resource!(PersistentVolumeClaim, &resolved_kind),
        "NetworkPolicy" => replace_resource!(NetworkPolicy, &resolved_kind),
        "Ingress" => replace_resource!(Ingress, &resolved_kind),
        "Node" => replace_cluster_resource!(Node, &resolved_kind),
        "Namespace" => replace_cluster_resource!(Namespace, &resolved_kind),
        "ResourceQuota" => replace_resource!(ResourceQuota, &resolved_kind),
        "LimitRange" => replace_resource!(LimitRange, &resolved_kind),
        "ValidatingWebhookConfiguration" => replace_cluster_resource!(ValidatingWebhookConfiguration, &resolved_kind),
        "MutatingWebhookConfiguration" => replace_cluster_resource!(MutatingWebhookConfiguration, &resolved_kind),
        "PersistentVolume" => replace_cluster_resource!(PersistentVolume, &resolved_kind),
        "StorageClass" => replace_cluster_resource!(StorageClass, &resolved_kind),
        _ => Err(kube::Error::Api(kube::error::ErrorResponse {
            status: "Failure".to_string(),
            message: format!("Unsupported apply resource kind: {}", kind),
            reason: "BadRequest".to_string(),
            code: 400,
        })),
    }
}

pub async fn create_resource(
    client: &Client,
    namespace: &str,
    kind: &str,
    raw_json: Value,
) -> Result<(), kube::Error> {
    let post_params = PostParams::default();

    macro_rules! create_resource {
        ($api_type:ty) => {{
            let api: Api<$api_type> = Api::namespaced(client.clone(), namespace);
            let parsed: $api_type = serde_json::from_value(raw_json).map_err(|e| {
                kube::Error::Api(kube::error::ErrorResponse {
                    status: "Failure".to_string(),
                    message: format!("Failed to parse {}: {}", kind, e),
                    reason: "BadRequest".to_string(),
                    code: 400,
                })
            })?;
            api.create(&post_params, &parsed).await?;
        }};
    }

    macro_rules! create_cluster_resource {
        ($api_type:ty) => {{
            let api: Api<$api_type> = Api::all(client.clone());
            let parsed: $api_type = serde_json::from_value(raw_json).map_err(|e| {
                kube::Error::Api(kube::error::ErrorResponse {
                    status: "Failure".to_string(),
                    message: format!("Failed to parse {}: {}", kind, e),
                    reason: "BadRequest".to_string(),
                    code: 400,
                })
            })?;
            api.create(&post_params, &parsed).await?;
        }};
    }

    let resolved_kind = if kind == "Policy" {
        raw_json
            .get("kind")
            .and_then(|k| k.as_str())
            .unwrap_or(kind)
    } else {
        kind
    };

    if resolve_resource_scope(resolved_kind).is_none() {
        return Err(kube::Error::Api(kube::error::ErrorResponse {
            status: "Failure".to_string(),
            message: format!("Unsupported create resource kind: {}", kind),
            reason: "BadRequest".to_string(),
            code: 400,
        }));
    }

    match resolved_kind {
        "Pod" => create_resource!(Pod),
        "Deployment" => create_resource!(Deployment),
        "StatefulSet" => create_resource!(StatefulSet),
        "DaemonSet" => create_resource!(DaemonSet),
        "ReplicaSet" => create_resource!(ReplicaSet),
        "Job" => create_resource!(Job),
        "CronJob" => create_resource!(CronJob),
        "Service" => create_resource!(Service),
        "ConfigMap" => create_resource!(ConfigMap),
        "Secret" => create_resource!(Secret),
        "PersistentVolumeClaim" => create_resource!(PersistentVolumeClaim),
        "NetworkPolicy" => create_resource!(NetworkPolicy),
        "Ingress" => create_resource!(Ingress),
        "Node" => create_cluster_resource!(Node),
        "Namespace" => create_cluster_resource!(Namespace),
        "ResourceQuota" => create_resource!(ResourceQuota),
        "LimitRange" => create_resource!(LimitRange),
        "ValidatingWebhookConfiguration" => create_cluster_resource!(ValidatingWebhookConfiguration),
        "MutatingWebhookConfiguration" => create_cluster_resource!(MutatingWebhookConfiguration),
        "PersistentVolume" => create_cluster_resource!(PersistentVolume),
        "StorageClass" => create_cluster_resource!(StorageClass),
        _ => return Err(kube::Error::Api(kube::error::ErrorResponse {
            status: "Failure".to_string(),
            message: format!("Unsupported create resource kind: {}", kind),
            reason: "BadRequest".to_string(),
            code: 400,
        })),
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::Serialize;

    #[derive(Serialize)]
    struct DummyResource {
        name: String,
        replicas: u32,
    }

    #[test]
    fn test_to_json_val_success() {
        let dummy = DummyResource {
            name: "test-dep".to_string(),
            replicas: 3,
        };
        let val = to_json_val(&dummy, "Deployment").expect("Serialization should succeed");
        assert_eq!(val["name"], "test-dep");
        assert_eq!(val["replicas"], 3);
    }

    #[test]
    fn test_is_conflict_error_matches_code_409() {
        let conflict_err = kube::Error::Api(kube::error::ErrorResponse {
            status: "Failure".to_string(),
            message: "Operation cannot be fulfilled: the object has been modified; please apply your changes to the latest version and try again".to_string(),
            reason: "Conflict".to_string(),
            code: 409,
        });
        assert!(is_conflict_error(&conflict_err));

        let bad_request_err = kube::Error::Api(kube::error::ErrorResponse {
            status: "Failure".to_string(),
            message: "Bad request".to_string(),
            reason: "BadRequest".to_string(),
            code: 400,
        });
        assert!(!is_conflict_error(&bad_request_err));

        let not_found_err = kube::Error::Api(kube::error::ErrorResponse {
            status: "Failure".to_string(),
            message: "Not found".to_string(),
            reason: "NotFound".to_string(),
            code: 404,
        });
        assert!(!is_conflict_error(&not_found_err));

        let internal_err = kube::Error::Api(kube::error::ErrorResponse {
            status: "Failure".to_string(),
            message: "Internal server error".to_string(),
            reason: "InternalError".to_string(),
            code: 500,
        });
        assert!(!is_conflict_error(&internal_err));
    }

    #[test]
    fn test_sync_resource_version_populates_target() {
        let mut target = k8s_openapi::apimachinery::pkg::apis::meta::v1::ObjectMeta {
            resource_version: Some("100".to_string()),
            ..Default::default()
        };
        let source = k8s_openapi::apimachinery::pkg::apis::meta::v1::ObjectMeta {
            resource_version: Some("102".to_string()),
            ..Default::default()
        };

        sync_resource_version(&mut target, &source);
        assert_eq!(target.resource_version.as_deref(), Some("102"));
    }

    #[test]
    fn test_namespace_manifest_parse() {
        let manifest_json = serde_json::json!({
            "apiVersion": "v1",
            "kind": "Namespace",
            "metadata": {
                "name": "dev-team",
                "labels": {
                    "environment": "development"
                },
                "annotations": {
                    "description": "Dev team namespace"
                }
            }
        });
        let parsed: Result<Namespace, _> = serde_json::from_value(manifest_json);
        assert!(parsed.is_ok());
        let ns = parsed.unwrap();
        assert_eq!(ns.metadata.name.as_deref(), Some("dev-team"));
        let labels = ns.metadata.labels.expect("labels should be present");
        assert_eq!(labels.get("environment").map(|s| s.as_str()), Some("development"));
        let annotations = ns.metadata.annotations.expect("annotations should be present");
        assert_eq!(annotations.get("description").map(|s| s.as_str()), Some("Dev team namespace"));
    }

    #[test]
    fn test_pod_manifest_parse() {
        let manifest_json = serde_json::json!({
            "apiVersion": "v1",
            "kind": "Pod",
            "metadata": {
                "name": "test-pod",
                "namespace": "default",
                "labels": {
                    "app": "test-pod"
                }
            },
            "spec": {
                "restartPolicy": "Always",
                "containers": [{
                    "name": "test-pod",
                    "image": "nginx:alpine",
                    "ports": [{
                        "containerPort": 80,
                        "protocol": "TCP"
                    }],
                    "env": [{
                        "name": "ENV_VAR",
                        "value": "prod"
                    }]
                }]
            }
        });
        let parsed: Result<Pod, _> = serde_json::from_value(manifest_json);
        assert!(parsed.is_ok());
        let pod = parsed.unwrap();
        assert_eq!(pod.metadata.name.as_deref(), Some("test-pod"));
        assert_eq!(pod.metadata.namespace.as_deref(), Some("default"));
        let labels = pod.metadata.labels.expect("labels should be present");
        assert_eq!(labels.get("app").map(|s| s.as_str()), Some("test-pod"));

        let spec = pod.spec.expect("spec should be present");
        assert_eq!(spec.restart_policy.as_deref(), Some("Always"));
        assert_eq!(spec.containers.len(), 1);
        assert_eq!(spec.containers[0].name, "test-pod");
        assert_eq!(spec.containers[0].image.as_deref(), Some("nginx:alpine"));
        let ports = spec.containers[0].ports.as_ref().expect("ports should be present");
        assert_eq!(ports.len(), 1);
        assert_eq!(ports[0].container_port, 80);
    }

    #[test]
    fn test_resource_quota_manifest_parse() {
        let manifest_json = serde_json::json!({
            "apiVersion": "v1",
            "kind": "ResourceQuota",
            "metadata": {
                "name": "compute-quota",
                "namespace": "default"
            },
            "spec": {
                "hard": {
                    "pods": "10",
                    "requests.cpu": "4"
                }
            }
        });
        let parsed: Result<ResourceQuota, _> = serde_json::from_value(manifest_json);
        assert!(parsed.is_ok());
        let quota = parsed.unwrap();
        assert_eq!(quota.metadata.name.as_deref(), Some("compute-quota"));
    }

    #[test]
    fn test_policy_fallback_kind_resolution_from_json() {
        let manifest_json = serde_json::json!({
            "apiVersion": "networking.k8s.io/v1",
            "kind": "NetworkPolicy",
            "metadata": {
                "name": "isolate-net",
                "namespace": "default"
            }
        });
        let kind = "Policy";
        let resolved_kind = if kind == "Policy" {
            manifest_json
                .get("kind")
                .and_then(|k| k.as_str())
                .unwrap_or(kind)
        } else {
            kind
        };
        assert_eq!(resolved_kind, "NetworkPolicy");
    }

    #[test]
    fn test_storage_resource_api_scope() {
        assert_eq!(resolve_resource_scope("PersistentVolume"), Some(ResourceApiScope::ClusterScoped));
        assert_eq!(resolve_resource_scope("StorageClass"), Some(ResourceApiScope::ClusterScoped));
        assert_eq!(resolve_resource_scope("PersistentVolumeClaim"), Some(ResourceApiScope::Namespaced));
        assert_eq!(resolve_resource_scope("UnknownKind"), None);
    }

    #[test]
    fn test_persistent_volume_manifest_parse() {
        let manifest_json = serde_json::json!({
            "apiVersion": "v1",
            "kind": "PersistentVolume",
            "metadata": {
                "name": "pv-demo",
                "labels": {
                    "type": "local"
                }
            },
            "spec": {
                "capacity": {
                    "storage": "20Gi"
                },
                "accessModes": ["ReadWriteOnce"],
                "persistentVolumeReclaimPolicy": "Retain",
                "storageClassName": "manual",
                "hostPath": {
                    "path": "/mnt/data"
                }
            }
        });
        let parsed: Result<PersistentVolume, _> = serde_json::from_value(manifest_json);
        assert!(parsed.is_ok());
        let pv = parsed.unwrap();
        assert_eq!(pv.metadata.name.as_deref(), Some("pv-demo"));
        assert!(pv.metadata.namespace.is_none());
        let spec = pv.spec.expect("spec should be present");
        let capacity = spec.capacity.expect("capacity should be present");
        assert_eq!(capacity.get("storage").map(|q| q.0.as_str()), Some("20Gi"));
    }

    #[test]
    fn test_persistent_volume_claim_manifest_parse() {
        let manifest_json = serde_json::json!({
            "apiVersion": "v1",
            "kind": "PersistentVolumeClaim",
            "metadata": {
                "name": "pvc-demo",
                "namespace": "default"
            },
            "spec": {
                "accessModes": ["ReadWriteOnce"],
                "resources": {
                    "requests": {
                        "storage": "10Gi"
                    }
                }
            }
        });
        let parsed: Result<PersistentVolumeClaim, _> = serde_json::from_value(manifest_json);
        assert!(parsed.is_ok());
        let pvc = parsed.unwrap();
        assert_eq!(pvc.metadata.name.as_deref(), Some("pvc-demo"));
        assert_eq!(pvc.metadata.namespace.as_deref(), Some("default"));
    }

    #[test]
    fn test_storage_class_manifest_parse() {
        let manifest_json = serde_json::json!({
            "apiVersion": "storage.k8s.io/v1",
            "kind": "StorageClass",
            "metadata": {
                "name": "fast-storage",
                "annotations": {
                    "storageclass.kubernetes.io/is-default-class": "true"
                }
            },
            "provisioner": "kubernetes.io/no-provisioner",
            "reclaimPolicy": "Delete",
            "volumeBindingMode": "WaitForFirstConsumer",
            "allowVolumeExpansion": true,
            "parameters": {
                "type": "gp3"
            },
            "mountOptions": ["debug"]
        });
        let parsed: Result<StorageClass, _> = serde_json::from_value(manifest_json);
        assert!(parsed.is_ok());
        let sc = parsed.unwrap();
        assert_eq!(sc.metadata.name.as_deref(), Some("fast-storage"));
        assert_eq!(sc.provisioner, "kubernetes.io/no-provisioner");
        assert_eq!(sc.reclaim_policy.as_deref(), Some("Delete"));
        assert_eq!(sc.volume_binding_mode.as_deref(), Some("WaitForFirstConsumer"));
        assert_eq!(sc.allow_volume_expansion, Some(true));
        assert_eq!(sc.mount_options.as_deref(), Some(&vec!["debug".to_string()][..]));
    }
}
