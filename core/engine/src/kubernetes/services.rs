use kube::{
    api::{Api, ListParams},
    Client,
};
use k8s_openapi::api::core::v1::Service;
use crate::kubernetes::{models, format_age};

pub async fn list_services(client: &Client, namespace: Option<String>) -> Result<Vec<models::ServiceInfo>, kube::Error> {
    let api: Api<Service> = if let Some(ns) = namespace {
        Api::namespaced(client.clone(), &ns)
    } else {
        Api::all(client.clone())
    };

    let mut list = Vec::new();
    for s in api.list(&ListParams::default()).await? {
        let name = s.metadata.name.clone().unwrap_or_default();
        let namespace_name = s.metadata.namespace.clone().unwrap_or_default();
        let uid = s.metadata.uid.clone().unwrap_or_default();

        let age = format_age(&s.metadata.creation_timestamp);
        let created = s.metadata.creation_timestamp.as_ref()
            .map(|t| t.0.format("%b %d, %Y, %I:%M %p").to_string())
            .unwrap_or_default();

        let spec = s.spec.as_ref();

        let service_type = spec.and_then(|sp| sp.type_.clone()).unwrap_or_else(|| "ClusterIP".to_string());
        let cluster_ip = spec.and_then(|sp| sp.cluster_ip.clone()).unwrap_or_else(|| "-".to_string());

        let session_affinity = spec.and_then(|sp| sp.session_affinity.clone()).unwrap_or_else(|| "None".to_string());
        let internal_traffic_policy = spec.and_then(|sp| sp.internal_traffic_policy.clone());

        // Parse external IP
        let mut external_ip = "-".to_string();
        if let Some(sp) = spec {
            if let Some(ext_ips) = &sp.external_ips {
                if !ext_ips.is_empty() {
                    external_ip = ext_ips.join(", ");
                }
            } else if service_type == "ExternalName" {
                if let Some(ext_name) = &sp.external_name {
                    external_ip = ext_name.clone();
                }
            } else if service_type == "LoadBalancer" {
                let ips: Vec<String> = s.status.as_ref()
                    .and_then(|status| status.load_balancer.as_ref())
                    .and_then(|lb| lb.ingress.as_ref())
                    .map(|ingress| {
                        ingress.iter().filter_map(|ing| {
                            ing.ip.clone().or_else(|| ing.hostname.clone())
                        }).collect()
                    })
                    .unwrap_or_default();

                if !ips.is_empty() {
                    external_ip = ips.join(", ");
                } else {
                    external_ip = "<pending>".to_string();
                }
            }
        }

        // Ports formatting and parsing
        let mut ports_str = Vec::new();
        let mut ports_list = Vec::new();
        if let Some(ports) = spec.and_then(|sp| sp.ports.as_ref()) {
            for p in ports {
                let port_val = p.port;
                let target_port_val = p.target_port.as_ref()
                    .map(|tp| match tp {
                        k8s_openapi::apimachinery::pkg::util::intstr::IntOrString::Int(i) => i.to_string(),
                        k8s_openapi::apimachinery::pkg::util::intstr::IntOrString::String(s) => s.clone(),
                    })
                    .unwrap_or_else(|| port_val.to_string());
                let protocol = p.protocol.clone().unwrap_or_else(|| "TCP".to_string());
                let node_port = p.node_port;

                let mut p_str = format!("{}/{}", port_val, protocol);
                if let Some(np) = node_port {
                    p_str = format!("{}:{}/{}", port_val, np, protocol);
                }
                ports_str.push(p_str);

                ports_list.push(models::ServicePort {
                    port: port_val,
                    target_port: target_port_val,
                    protocol,
                    node_port,
                });
            }
        }
        let ports = if ports_str.is_empty() { "-".to_string() } else { ports_str.join("\n") };

        let endpoints = "-".to_string();
        let endpoints_list = Vec::new();

        let selector = spec.and_then(|sp| sp.selector.clone()).unwrap_or_default();
        let labels = s.metadata.labels.clone().unwrap_or_default();
        let events = Vec::new();

        list.push(models::ServiceInfo {
            name,
            namespace: namespace_name,
            r#type: service_type,
            cluster_ip,
            external_ip,
            ports,
            endpoints,
            age,
            session_affinity,
            internal_traffic_policy,
            created,
            uid,
            selector,
            labels,
            ports_list,
            endpoints_list,
            events,
        });
    }

    Ok(list)
}
