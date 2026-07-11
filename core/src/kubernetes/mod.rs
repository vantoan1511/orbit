use crate::kubernetes::models::PodInfo;
use kube::{
    api::{Api, ListParams},
    Client,
};
use k8s_openapi::api::core::v1::{Namespace, Pod};

pub mod models;

pub async fn list_namespaces() -> Result<Vec<String>, kube::Error> {
    let client = Client::try_default().await?;
    let namespaces: Api<Namespace> = Api::all(client);
    let mut namespace_list = Vec::new();
    
    for ns in namespaces.list(&ListParams::default()).await? {
        if let Some(name) = ns.metadata.name {
            namespace_list.push(name);
        }
    }
    
    // Sort namespaces alphabetically for consistency
    namespace_list.sort();
    
    Ok(namespace_list)
}

pub async fn list_pods(namespace: Option<String>) -> Result<Vec<PodInfo>, kube::Error> {
    let client = Client::try_default().await?;
    let pods: Api<Pod> = if let Some(ns) = namespace {
        Api::namespaced(client, &ns)
    } else {
        Api::all(client)
    };
    
    let mut pod_list = Vec::new();
    for p in pods.list(&ListParams::default()).await? {
        let name = p.metadata.name.clone().unwrap_or_default();
        let namespace_name = p.metadata.namespace.clone().unwrap_or_default();
        
        let status = p.status.as_ref()
            .and_then(|s| s.phase.clone())
            .unwrap_or_else(|| "Unknown".to_string());
            
        // Calculate age
        let age = if let Some(creation) = &p.metadata.creation_timestamp {
            if let Some(time) = creation.0.to_rfc3339().split('T').next() {
                time.to_string()
            } else {
                "Unknown".to_string()
            }
        } else {
            "Unknown".to_string()
        };
        
        pod_list.push(PodInfo {
            name,
            namespace: namespace_name,
            status,
            age,
            cpu: None,
            memory: None,
        });
    }
    
    Ok(pod_list)
}
