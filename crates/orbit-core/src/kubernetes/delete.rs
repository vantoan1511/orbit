use std::time::Duration;

use k8s_openapi::api::admissionregistration::v1::{
    MutatingWebhookConfiguration, ValidatingWebhookConfiguration,
};
use k8s_openapi::api::apps::v1::{DaemonSet, Deployment, ReplicaSet, StatefulSet};
use k8s_openapi::api::autoscaling::v2::HorizontalPodAutoscaler;
use k8s_openapi::api::batch::v1::{CronJob, Job};
use k8s_openapi::api::core::v1::{
    ConfigMap, Event, LimitRange, Namespace, Node, PersistentVolume, PersistentVolumeClaim, Pod,
    ResourceQuota, Secret, Service,
};
use k8s_openapi::api::networking::v1::{Ingress, NetworkPolicy};
use k8s_openapi::api::storage::v1::StorageClass;
use kube::{
    api::{Api, DeleteParams},
    Client,
};

use crate::kubernetes::models::ResourceApiScope;

pub const DELETE_ACTION_TIMEOUT: Duration = Duration::from_secs(15);

pub fn resolve_delete_scope(kind: &str) -> Option<ResourceApiScope> {
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
        | "LimitRange"
        | "Event"
        | "Policy" => Some(ResourceApiScope::Namespaced),
        "Namespace"
        | "PersistentVolume"
        | "Node"
        | "StorageClass"
        | "ValidatingWebhookConfiguration"
        | "MutatingWebhookConfiguration" => Some(ResourceApiScope::ClusterScoped),
        _ => None,
    }
}

pub fn is_not_found_error(error: &kube::Error) -> bool {
    matches!(error, kube::Error::Api(err) if err.code == 404 || err.reason == "NotFound")
}

pub fn format_target_label(namespace: Option<&str>, name: &str) -> String {
    match namespace.map(str::trim).filter(|ns| !ns.is_empty()) {
        Some(ns) => format!("{ns}/{name}"),
        None => name.to_string(),
    }
}

pub fn validate_delete_input(
    namespace: Option<&str>,
    kind: &str,
    name: &str,
) -> Result<ResourceApiScope, String> {
    let trimmed_kind = kind.trim();
    if trimmed_kind.is_empty() {
        return Err("Resource kind is required for deletion.".to_string());
    }
    let scope = resolve_delete_scope(trimmed_kind)
        .ok_or_else(|| format!("Unsupported delete resource kind: {trimmed_kind}"))?;

    if name.trim().is_empty() {
        return Err(format!("Cannot delete {trimmed_kind}: resource name is empty."));
    }

    if scope == ResourceApiScope::Namespaced
        && namespace.map(str::trim).unwrap_or_default().is_empty()
    {
        return Err(format!(
            "Cannot delete {trimmed_kind} '{}': namespace is required for namespaced resources.",
            name.trim()
        ));
    }

    Ok(scope)
}

pub fn format_delete_error(error: &kube::Error, kind: &str, target: &str) -> String {
    match error {
        kube::Error::Api(api_err) if api_err.code == 403 || api_err.reason == "Forbidden" => {
            if api_err.message.trim().is_empty() {
                format!("Permission denied deleting {kind} '{target}'.")
            } else {
                format!("Permission denied: {}", api_err.message)
            }
        }
        kube::Error::Api(api_err) if api_err.code == 404 || api_err.reason == "NotFound" => {
            if api_err.message.trim().is_empty() {
                format!("{kind} '{target}' was not found (it may have already been deleted).")
            } else {
                api_err.message.clone()
            }
        }
        kube::Error::Api(api_err) if api_err.code == 409 || api_err.reason == "Conflict" => {
            if api_err.message.trim().is_empty() {
                format!("Conflict deleting {kind} '{target}' (check active finalizers).")
            } else {
                api_err.message.clone()
            }
        }
        kube::Error::Api(api_err) => {
            if api_err.message.trim().is_empty() {
                format!("Kubernetes API error ({}) deleting {kind} '{target}'.", api_err.code)
            } else {
                api_err.message.clone()
            }
        }
        other => other.to_string(),
    }
}

pub async fn delete_resource(
    client: &Client,
    namespace: &str,
    kind: &str,
    name: &str,
) -> Result<(), kube::Error> {
    if resolve_delete_scope(kind).is_none() {
        return Err(kube::Error::Api(kube::error::ErrorResponse {
            status: "Failure".to_string(),
            message: format!("Unsupported delete resource kind: {}", kind),
            reason: "BadRequest".to_string(),
            code: 400,
        }));
    }

    let delete_params = DeleteParams::default();

    match kind {
        "Pod" => {
            let api: Api<Pod> = Api::namespaced(client.clone(), namespace);
            api.delete(name, &delete_params).await?;
        }
        "Deployment" => {
            let api: Api<Deployment> = Api::namespaced(client.clone(), namespace);
            api.delete(name, &delete_params).await?;
        }
        "StatefulSet" => {
            let api: Api<StatefulSet> = Api::namespaced(client.clone(), namespace);
            api.delete(name, &delete_params).await?;
        }
        "DaemonSet" => {
            let api: Api<DaemonSet> = Api::namespaced(client.clone(), namespace);
            api.delete(name, &delete_params).await?;
        }
        "ReplicaSet" => {
            let api: Api<ReplicaSet> = Api::namespaced(client.clone(), namespace);
            api.delete(name, &delete_params).await?;
        }
        "Job" => {
            let api: Api<Job> = Api::namespaced(client.clone(), namespace);
            api.delete(name, &delete_params).await?;
        }
        "CronJob" => {
            let api: Api<CronJob> = Api::namespaced(client.clone(), namespace);
            api.delete(name, &delete_params).await?;
        }
        "Service" => {
            let api: Api<Service> = Api::namespaced(client.clone(), namespace);
            api.delete(name, &delete_params).await?;
        }
        "ConfigMap" => {
            let api: Api<ConfigMap> = Api::namespaced(client.clone(), namespace);
            api.delete(name, &delete_params).await?;
        }
        "Secret" => {
            let api: Api<Secret> = Api::namespaced(client.clone(), namespace);
            api.delete(name, &delete_params).await?;
        }
        "PersistentVolumeClaim" => {
            let api: Api<PersistentVolumeClaim> = Api::namespaced(client.clone(), namespace);
            api.delete(name, &delete_params).await?;
        }
        "NetworkPolicy" => {
            let api: Api<NetworkPolicy> = Api::namespaced(client.clone(), namespace);
            api.delete(name, &delete_params).await?;
        }
        "Ingress" => {
            let api: Api<Ingress> = Api::namespaced(client.clone(), namespace);
            api.delete(name, &delete_params).await?;
        }
        "HorizontalPodAutoscaler" => {
            let api: Api<HorizontalPodAutoscaler> = Api::namespaced(client.clone(), namespace);
            api.delete(name, &delete_params).await?;
        }
        "Namespace" => {
            let api: Api<Namespace> = Api::all(client.clone());
            api.delete(name, &delete_params).await?;
        }
        "Node" => {
            let api: Api<Node> = Api::all(client.clone());
            api.delete(name, &delete_params).await?;
        }
        "ResourceQuota" => {
            let api: Api<ResourceQuota> = Api::namespaced(client.clone(), namespace);
            api.delete(name, &delete_params).await?;
        }
        "LimitRange" => {
            let api: Api<LimitRange> = Api::namespaced(client.clone(), namespace);
            api.delete(name, &delete_params).await?;
        }
        "Event" => {
            let api: Api<Event> = Api::namespaced(client.clone(), namespace);
            api.delete(name, &delete_params).await?;
        }
        "ValidatingWebhookConfiguration" => {
            let api: Api<ValidatingWebhookConfiguration> = Api::all(client.clone());
            api.delete(name, &delete_params).await?;
        }
        "MutatingWebhookConfiguration" => {
            let api: Api<MutatingWebhookConfiguration> = Api::all(client.clone());
            api.delete(name, &delete_params).await?;
        }
        "PersistentVolume" => {
            let api: Api<PersistentVolume> = Api::all(client.clone());
            api.delete(name, &delete_params).await?;
        }
        "StorageClass" => {
            let api: Api<StorageClass> = Api::all(client.clone());
            api.delete(name, &delete_params).await?;
        }
        "Policy" => {
            let np_api: Api<NetworkPolicy> = Api::namespaced(client.clone(), namespace);
            match np_api.delete(name, &delete_params).await {
                Ok(_) => return Ok(()),
                Err(e) if is_not_found_error(&e) => {}
                Err(e) => return Err(e),
            }
            let rq_api: Api<ResourceQuota> = Api::namespaced(client.clone(), namespace);
            match rq_api.delete(name, &delete_params).await {
                Ok(_) => return Ok(()),
                Err(e) if is_not_found_error(&e) => {}
                Err(e) => return Err(e),
            }
            let lr_api: Api<LimitRange> = Api::namespaced(client.clone(), namespace);
            match lr_api.delete(name, &delete_params).await {
                Ok(_) => return Ok(()),
                Err(e) if is_not_found_error(&e) => {}
                Err(e) => return Err(e),
            }
            let val_api: Api<ValidatingWebhookConfiguration> = Api::all(client.clone());
            match val_api.delete(name, &delete_params).await {
                Ok(_) => return Ok(()),
                Err(e) if is_not_found_error(&e) => {}
                Err(e) => return Err(e),
            }
            let mut_api: Api<MutatingWebhookConfiguration> = Api::all(client.clone());
            match mut_api.delete(name, &delete_params).await {
                Ok(_) => return Ok(()),
                Err(e) if is_not_found_error(&e) => {}
                Err(e) => return Err(e),
            }
            return Err(kube::Error::Api(kube::error::ErrorResponse {
                status: "Failure".to_string(),
                message: format!("Policy resource not found: {}", name),
                reason: "NotFound".to_string(),
                code: 404,
            }));
        }
        _ => {
            return Err(kube::Error::Api(kube::error::ErrorResponse {
                status: "Failure".to_string(),
                message: format!("Unsupported delete resource kind: {}", kind),
                reason: "BadRequest".to_string(),
                code: 400,
            }))
        }
    }
    Ok(())
}

pub async fn execute_delete_resource(
    client: &Client,
    namespace: Option<&str>,
    kind: &str,
    name: &str,
) -> Result<(), String> {
    let scope = validate_delete_input(namespace, kind, name)?;
    let ns = match scope {
        ResourceApiScope::Namespaced => namespace.unwrap_or_default().trim(),
        ResourceApiScope::ClusterScoped => "",
    };
    let trimmed_name = name.trim();
    let target = format_target_label(
        (scope == ResourceApiScope::Namespaced).then_some(ns),
        trimmed_name,
    );

    tracing::info!(
        kind = %kind,
        namespace = %ns,
        name = %trimmed_name,
        "Sending request to Kubernetes: delete resource"
    );

    match tokio::time::timeout(
        DELETE_ACTION_TIMEOUT,
        delete_resource(client, ns, kind.trim(), trimmed_name),
    )
    .await
    {
        Ok(Ok(())) => {
            tracing::info!(
                kind = %kind,
                namespace = %ns,
                name = %trimmed_name,
                "Kubernetes request completed: delete resource"
            );
            Ok(())
        }
        Ok(Err(err)) => Err(format_delete_error(&err, kind.trim(), &target)),
        Err(_) => Err(format!(
            "Timed out deleting {} '{}' after {}s",
            kind.trim(),
            target,
            DELETE_ACTION_TIMEOUT.as_secs()
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use kube::error::ErrorResponse;

    #[test]
    fn test_delete_resource_scope_all_supported_kinds() {
        for kind in [
            "Pod",
            "Deployment",
            "StatefulSet",
            "DaemonSet",
            "ReplicaSet",
            "Job",
            "CronJob",
            "Service",
            "Ingress",
            "ConfigMap",
            "Secret",
            "PersistentVolumeClaim",
            "NetworkPolicy",
            "ResourceQuota",
            "LimitRange",
            "Event",
            "HorizontalPodAutoscaler",
            "Policy",
        ] {
            assert_eq!(
                resolve_delete_scope(kind),
                Some(ResourceApiScope::Namespaced),
                "expected Namespaced for {kind}"
            );
        }

        for kind in [
            "Namespace",
            "PersistentVolume",
            "Node",
            "StorageClass",
            "ValidatingWebhookConfiguration",
            "MutatingWebhookConfiguration",
        ] {
            assert_eq!(
                resolve_delete_scope(kind),
                Some(ResourceApiScope::ClusterScoped),
                "expected ClusterScoped for {kind}"
            );
        }

        assert_eq!(resolve_delete_scope("Unknown"), None);
    }

    #[test]
    fn test_validate_delete_input_rejects_empty_name_or_missing_namespace_for_namespaced_kind() {
        assert_eq!(
            validate_delete_input(Some("default"), "Pod", "web-0"),
            Ok(ResourceApiScope::Namespaced)
        );
        assert_eq!(
            validate_delete_input(None, "Node", "worker-1"),
            Ok(ResourceApiScope::ClusterScoped)
        );
        assert_eq!(
            validate_delete_input(Some(""), "Namespace", "staging"),
            Ok(ResourceApiScope::ClusterScoped)
        );

        let missing_ns = validate_delete_input(None, "Pod", "web-0").unwrap_err();
        assert!(missing_ns.contains("namespace is required"));

        let empty_ns = validate_delete_input(Some("  "), "Deployment", "api").unwrap_err();
        assert!(empty_ns.contains("namespace is required"));

        let empty_name = validate_delete_input(Some("default"), "Pod", "  ").unwrap_err();
        assert!(empty_name.contains("resource name is empty"));

        let unsupported = validate_delete_input(Some("default"), "CustomThing", "foo").unwrap_err();
        assert!(unsupported.contains("Unsupported delete resource kind"));
    }

    #[test]
    fn test_is_not_found_error_distinguishes_errors() {
        let not_found_404 = kube::Error::Api(ErrorResponse {
            status: "Failure".to_string(),
            message: "not found".to_string(),
            reason: "NotFound".to_string(),
            code: 404,
        });
        assert!(is_not_found_error(&not_found_404));

        let forbidden_403 = kube::Error::Api(ErrorResponse {
            status: "Failure".to_string(),
            message: "forbidden".to_string(),
            reason: "Forbidden".to_string(),
            code: 403,
        });
        assert!(!is_not_found_error(&forbidden_403));

        let conflict_409 = kube::Error::Api(ErrorResponse {
            status: "Failure".to_string(),
            message: "conflict".to_string(),
            reason: "Conflict".to_string(),
            code: 409,
        });
        assert!(!is_not_found_error(&conflict_409));
    }

    #[test]
    fn test_format_delete_error_messages() {
        let forbidden = kube::Error::Api(ErrorResponse {
            status: "Failure".to_string(),
            message: "pods \"web-0\" is forbidden".to_string(),
            reason: "Forbidden".to_string(),
            code: 403,
        });
        assert_eq!(
            format_delete_error(&forbidden, "Pod", "default/web-0"),
            "Permission denied: pods \"web-0\" is forbidden"
        );

        let not_found = kube::Error::Api(ErrorResponse {
            status: "Failure".to_string(),
            message: "pods \"web-0\" not found".to_string(),
            reason: "NotFound".to_string(),
            code: 404,
        });
        assert_eq!(
            format_delete_error(&not_found, "Pod", "default/web-0"),
            "pods \"web-0\" not found"
        );

        let empty_not_found = kube::Error::Api(ErrorResponse {
            status: "Failure".to_string(),
            message: "".to_string(),
            reason: "NotFound".to_string(),
            code: 404,
        });
        assert!(format_delete_error(&empty_not_found, "Node", "worker-1")
            .contains("Node 'worker-1' was not found"));

        let generic = kube::Error::Api(ErrorResponse {
            status: "Failure".to_string(),
            message: "internal server error".to_string(),
            reason: "InternalError".to_string(),
            code: 500,
        });
        assert_eq!(
            format_delete_error(&generic, "Deployment", "prod/api"),
            "internal server error"
        );
    }
}
