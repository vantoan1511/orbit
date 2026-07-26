use kube::{
    api::{Api, PostParams},
    Client,
};
use k8s_openapi::api::core::v1::{Pod, Service, ConfigMap, Secret, PersistentVolumeClaim};
use k8s_openapi::api::apps::v1::{Deployment, StatefulSet, DaemonSet, ReplicaSet};
use k8s_openapi::api::batch::v1::{Job, CronJob};
use k8s_openapi::api::networking::v1::NetworkPolicy;
use serde_json::Value;

pub async fn get_resource_raw(
    client: &Client,
    namespace: &str,
    kind: &str,
    name: &str,
) -> Result<Value, kube::Error> {
    match kind {
        "Pod" => {
            let api: Api<Pod> = Api::namespaced(client.clone(), namespace);
            let resource = api.get(name).await?;
            Ok(serde_json::to_value(&resource).unwrap())
        }
        "Deployment" => {
            let api: Api<Deployment> = Api::namespaced(client.clone(), namespace);
            let resource = api.get(name).await?;
            Ok(serde_json::to_value(&resource).unwrap())
        }
        "StatefulSet" => {
            let api: Api<StatefulSet> = Api::namespaced(client.clone(), namespace);
            let resource = api.get(name).await?;
            Ok(serde_json::to_value(&resource).unwrap())
        }
        "DaemonSet" => {
            let api: Api<DaemonSet> = Api::namespaced(client.clone(), namespace);
            let resource = api.get(name).await?;
            Ok(serde_json::to_value(&resource).unwrap())
        }
        "ReplicaSet" => {
            let api: Api<ReplicaSet> = Api::namespaced(client.clone(), namespace);
            let resource = api.get(name).await?;
            Ok(serde_json::to_value(&resource).unwrap())
        }
        "Job" => {
            let api: Api<Job> = Api::namespaced(client.clone(), namespace);
            let resource = api.get(name).await?;
            Ok(serde_json::to_value(&resource).unwrap())
        }
        "CronJob" => {
            let api: Api<CronJob> = Api::namespaced(client.clone(), namespace);
            let resource = api.get(name).await?;
            Ok(serde_json::to_value(&resource).unwrap())
        }
        "Service" => {
            let api: Api<Service> = Api::namespaced(client.clone(), namespace);
            let resource = api.get(name).await?;
            Ok(serde_json::to_value(&resource).unwrap())
        }
        "ConfigMap" => {
            let api: Api<ConfigMap> = Api::namespaced(client.clone(), namespace);
            let resource = api.get(name).await?;
            Ok(serde_json::to_value(&resource).unwrap())
        }
        "Secret" => {
            let api: Api<Secret> = Api::namespaced(client.clone(), namespace);
            let resource = api.get(name).await?;
            Ok(serde_json::to_value(&resource).unwrap())
        }
        "PersistentVolumeClaim" => {
            let api: Api<PersistentVolumeClaim> = Api::namespaced(client.clone(), namespace);
            let resource = api.get(name).await?;
            Ok(serde_json::to_value(&resource).unwrap())
        }
        "NetworkPolicy" => {
            let api: Api<NetworkPolicy> = Api::namespaced(client.clone(), namespace);
            let resource = api.get(name).await?;
            Ok(serde_json::to_value(&resource).unwrap())
        }
        _ => Err(kube::Error::Api(kube::error::ErrorResponse {
            status: "Failure".to_string(),
            message: format!("Unsupported get resource kind: {}", kind),
            reason: "BadRequest".to_string(),
            code: 400,
        })),
    }
}

pub async fn apply_resource(
    client: &Client,
    namespace: &str,
    kind: &str,
    name: &str,
    raw_json: Value,
) -> Result<(), kube::Error> {
    let post_params = PostParams::default();

    macro_rules! replace_resource {
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
            api.replace(name, &post_params, &parsed).await?;
        }};
    }

    match kind {
        "Pod" => replace_resource!(Pod),
        "Deployment" => replace_resource!(Deployment),
        "StatefulSet" => replace_resource!(StatefulSet),
        "DaemonSet" => replace_resource!(DaemonSet),
        "ReplicaSet" => replace_resource!(ReplicaSet),
        "Job" => replace_resource!(Job),
        "CronJob" => replace_resource!(CronJob),
        "Service" => replace_resource!(Service),
        "ConfigMap" => replace_resource!(ConfigMap),
        "Secret" => replace_resource!(Secret),
        "PersistentVolumeClaim" => replace_resource!(PersistentVolumeClaim),
        "NetworkPolicy" => replace_resource!(NetworkPolicy),
        _ => return Err(kube::Error::Api(kube::error::ErrorResponse {
            status: "Failure".to_string(),
            message: format!("Unsupported apply resource kind: {}", kind),
            reason: "BadRequest".to_string(),
            code: 400,
        })),
    }
    Ok(())
}
