use kube::{
    api::{Api, ListParams},
    Client,
};
use k8s_openapi::api::core::v1::Namespace;
use crate::kubernetes::{models, format_age};

pub async fn list_namespaces(client: &Client) -> Result<Vec<models::NamespaceInfo>, kube::Error> {
    let namespaces_api: Api<Namespace> = Api::all(client.clone());
    let mut list = Vec::new();

    for ns in namespaces_api.list(&ListParams::default()).await? {
        let name = ns.metadata.name.clone().unwrap_or_default();
        let is_system = name.starts_with("kube-")
            || name == "default"
            || name.ends_with("-system")
            || name == "cert-manager"
            || name == "monitoring"
            || name == "ingress-nginx";
        
        let status = ns.status.as_ref()
            .and_then(|s| s.phase.clone())
            .unwrap_or_else(|| "Unknown".to_string());

        let age = format_age(&ns.metadata.creation_timestamp);
        
        let mut labels = std::collections::BTreeMap::new();
        if let Some(map) = ns.metadata.labels {
            labels = map;
        }

        let mut annotations = std::collections::BTreeMap::new();
        if let Some(map) = ns.metadata.annotations {
            annotations = map;
        }

        let uid = ns.metadata.uid.clone().unwrap_or_default();
        
        let created = ns.metadata.creation_timestamp.as_ref()
            .map(|t| t.0.to_rfc3339())
            .unwrap_or_default();

        list.push(models::NamespaceInfo {
            name,
            status,
            is_system,
            age,
            labels,
            annotations,
            uid,
            created,
        });
    }

    list.sort_by(|a, b| a.name.cmp(&b.name));

    Ok(list)
}
