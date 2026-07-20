use kube::{
    api::{Api, DeleteParams},
    Client,
};
use k8s_openapi::api::core::v1::{Pod, Service, ConfigMap, Secret, PersistentVolumeClaim};
use k8s_openapi::api::apps::v1::{Deployment, StatefulSet, DaemonSet, ReplicaSet};
use k8s_openapi::api::batch::v1::{Job, CronJob};
use k8s_openapi::api::networking::v1::NetworkPolicy;

pub async fn delete_resource(
    client: &Client,
    namespace: &str,
    kind: &str,
    name: &str,
) -> Result<(), kube::Error> {
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
        _ => return Err(kube::Error::Api(kube::error::ErrorResponse {
            status: "Failure".to_string(),
            message: format!("Unsupported delete resource kind: {}", kind),
            reason: "BadRequest".to_string(),
            code: 400,
        })),
    }
    Ok(())
}
