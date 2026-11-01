use kube::{
    api::{Api, DeleteParams},
    Client,
};
use k8s_openapi::api::core::v1::{Pod, Service, ConfigMap, Secret, PersistentVolume, PersistentVolumeClaim, Namespace, ResourceQuota, LimitRange};
use k8s_openapi::api::apps::v1::{Deployment, StatefulSet, DaemonSet, ReplicaSet};
use k8s_openapi::api::batch::v1::{Job, CronJob};
use k8s_openapi::api::networking::v1::{NetworkPolicy, Ingress};
use k8s_openapi::api::admissionregistration::v1::{ValidatingWebhookConfiguration, MutatingWebhookConfiguration};
use k8s_openapi::api::storage::v1::StorageClass;
use crate::kubernetes::models::ResourceApiScope;

pub(crate) fn resolve_delete_scope(kind: &str) -> Option<ResourceApiScope> {
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
        | "LimitRange"
        | "Policy" => Some(ResourceApiScope::Namespaced),
        "Namespace"
        | "PersistentVolume"
        | "StorageClass"
        | "ValidatingWebhookConfiguration"
        | "MutatingWebhookConfiguration" => Some(ResourceApiScope::ClusterScoped),
        _ => None,
    }
}

pub(crate) fn is_not_found_error(error: &kube::Error) -> bool {
    matches!(error, kube::Error::Api(err) if err.code == 404 || err.reason == "NotFound")
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
        "Namespace" => {
            let api: Api<Namespace> = Api::all(client.clone());
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
        _ => return Err(kube::Error::Api(kube::error::ErrorResponse {
            status: "Failure".to_string(),
            message: format!("Unsupported delete resource kind: {}", kind),
            reason: "BadRequest".to_string(),
            code: 400,
        })),
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_delete_resource_scope() {
        assert_eq!(resolve_delete_scope("PersistentVolume"), Some(ResourceApiScope::ClusterScoped));
        assert_eq!(resolve_delete_scope("StorageClass"), Some(ResourceApiScope::ClusterScoped));
        assert_eq!(resolve_delete_scope("PersistentVolumeClaim"), Some(ResourceApiScope::Namespaced));
        assert_eq!(resolve_delete_scope("Pod"), Some(ResourceApiScope::Namespaced));
        assert_eq!(resolve_delete_scope("Namespace"), Some(ResourceApiScope::ClusterScoped));
        assert_eq!(resolve_delete_scope("Unknown"), None);
    }

    #[test]
    fn test_resolve_delete_scope_policy_kinds() {
        assert_eq!(resolve_delete_scope("NetworkPolicy"), Some(ResourceApiScope::Namespaced));
        assert_eq!(resolve_delete_scope("ResourceQuota"), Some(ResourceApiScope::Namespaced));
        assert_eq!(resolve_delete_scope("LimitRange"), Some(ResourceApiScope::Namespaced));
        assert_eq!(resolve_delete_scope("ValidatingWebhookConfiguration"), Some(ResourceApiScope::ClusterScoped));
        assert_eq!(resolve_delete_scope("MutatingWebhookConfiguration"), Some(ResourceApiScope::ClusterScoped));
        assert_eq!(resolve_delete_scope("Policy"), Some(ResourceApiScope::Namespaced));
    }

    #[test]
    fn test_is_not_found_error_distinguishes_errors() {
        use kube::error::ErrorResponse;

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
}
