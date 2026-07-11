use crate::kubernetes::models::PodInfo;
use kube::{
    api::{Api, ListParams},
    Client,
};
use k8s_openapi::api::core::v1::{Namespace, Pod};

pub mod models;
pub mod manager;

pub async fn list_namespaces(client: &Client) -> Result<Vec<String>, kube::Error> {
    let namespaces: Api<Namespace> = Api::all(client.clone());
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

pub async fn list_pods(client: &Client, namespace: Option<String>) -> Result<Vec<PodInfo>, kube::Error> {
    let pods: Api<Pod> = if let Some(ns) = namespace {
        Api::namespaced(client.clone(), &ns)
    } else {
        Api::all(client.clone())
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

