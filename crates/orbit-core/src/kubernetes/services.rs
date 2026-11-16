use std::collections::HashMap;
use std::time::Duration;

use k8s_openapi::api::core::v1::{Endpoints, Service};
use kube::{
    api::{Api, ListParams},
    Client,
};

use crate::kubernetes::{models, namespaces::format_age};

pub const SERVICE_FETCH_TIMEOUT: Duration = Duration::from_secs(10);

pub fn format_service_port_target(port: &k8s_openapi::api::core::v1::ServicePort) -> String {
    port.target_port
        .as_ref()
        .map(|tp| match tp {
            k8s_openapi::apimachinery::pkg::util::intstr::IntOrString::Int(i) => i.to_string(),
            k8s_openapi::apimachinery::pkg::util::intstr::IntOrString::String(s) => s.clone(),
        })
        .unwrap_or_else(|| "-".to_string())
}

pub fn map_service(s: &Service) -> models::ServiceInfo {
    let name = s.metadata.name.clone().unwrap_or_default();
    let namespace_name = s.metadata.namespace.clone().unwrap_or_default();
    let uid = s.metadata.uid.clone().unwrap_or_default();

    let age = format_age(&s.metadata.creation_timestamp);
    let created = s
        .metadata
        .creation_timestamp
        .as_ref()
        .map(|t| t.0.format("%b %d, %Y, %I:%M %p").to_string())
        .unwrap_or_default();

    let spec = s.spec.as_ref();

    let service_type = spec
        .and_then(|sp| sp.type_.clone())
        .unwrap_or_else(|| "ClusterIP".to_string());
    let cluster_ip = spec
        .and_then(|sp| sp.cluster_ip.clone())
        .unwrap_or_else(|| "-".to_string());

    let session_affinity = spec
        .and_then(|sp| sp.session_affinity.clone())
        .unwrap_or_else(|| "None".to_string());
    let internal_traffic_policy = spec.and_then(|sp| sp.internal_traffic_policy.clone());

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
            let ips: Vec<String> = s
                .status
                .as_ref()
                .and_then(|status| status.load_balancer.as_ref())
                .and_then(|lb| lb.ingress.as_ref())
                .map(|ingress| {
                    ingress
                        .iter()
                        .filter_map(|ing| ing.ip.clone().or_else(|| ing.hostname.clone()))
                        .collect()
                })
                .unwrap_or_default();

            if !ips.is_empty() {
                external_ip = ips.join(", ");
            } else {
                external_ip = "<pending>".to_string();
            }
        }
    }

    let mut ports_str = Vec::new();
    let ports_list = spec
        .and_then(|sp| sp.ports.as_ref())
        .cloned()
        .unwrap_or_default();

    for p in &ports_list {
        let port_val = p.port;
        let protocol = p.protocol.as_deref().unwrap_or("TCP");
        let node_port = p.node_port;

        let mut p_str = format!("{}/{}", port_val, protocol);
        if let Some(np) = node_port {
            p_str = format!("{}:{}/{}", port_val, np, protocol);
        }
        ports_str.push(p_str);
    }
    let ports = if ports_str.is_empty() {
        "-".to_string()
    } else {
        ports_str.join("\n")
    };

    let endpoints = "-".to_string();
    let endpoints_list = Vec::new();

    let selector = spec.and_then(|sp| sp.selector.clone()).unwrap_or_default();
    let labels = s.metadata.labels.clone().unwrap_or_default();
    let events = Vec::new();

    models::ServiceInfo {
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
    }
}

/// Formats ready addresses from a Kubernetes `Endpoints` object into `"ip:port"` strings
/// and a compact summary string (e.g. `"2 ready"` or `"-"` when empty).
pub fn apply_endpoints_to_service(service: &mut models::ServiceInfo, endpoints: &Endpoints) {
    let mut formatted = Vec::new();
    if let Some(subsets) = &endpoints.subsets {
        for subset in subsets {
            let Some(addresses) = &subset.addresses else {
                continue;
            };
            let ports = subset.ports.as_deref().unwrap_or(&[]);
            if ports.is_empty() {
                for addr in addresses {
                    formatted.push(addr.ip.clone());
                }
            } else {
                for addr in addresses {
                    for port in ports {
                        formatted.push(format!("{}:{}", addr.ip, port.port));
                    }
                }
            }
        }
    }

    service.endpoints = if formatted.is_empty() {
        "-".to_string()
    } else {
        format!("{} ready", formatted.len())
    };
    service.endpoints_list = formatted;
}

pub async fn list_services(
    client: &Client,
    namespace: Option<String>,
) -> Result<Vec<models::ServiceInfo>, kube::Error> {
    let api: Api<Service> = if let Some(ref ns) = namespace {
        Api::namespaced(client.clone(), ns)
    } else {
        Api::all(client.clone())
    };

    let mut list: Vec<models::ServiceInfo> = api
        .list(&ListParams::default())
        .await?
        .iter()
        .map(map_service)
        .collect();

    let ep_api: Api<Endpoints> = if let Some(ref ns) = namespace {
        Api::namespaced(client.clone(), ns)
    } else {
        Api::all(client.clone())
    };

    if let Ok(ep_list) = ep_api.list(&ListParams::default()).await {
        let ep_map: HashMap<(String, String), Endpoints> = ep_list
            .items
            .into_iter()
            .filter_map(|ep| {
                let name = ep.metadata.name.clone()?;
                let ns = ep.metadata.namespace.clone().unwrap_or_default();
                Some(((ns, name), ep))
            })
            .collect();

        for svc in &mut list {
            if let Some(ep) = ep_map.get(&(svc.namespace.clone(), svc.name.clone())) {
                apply_endpoints_to_service(svc, ep);
            }
        }
    }

    Ok(list)
}

pub async fn fetch_services(
    client: &Client,
    namespace: Option<String>,
) -> Result<Vec<models::ServiceInfo>, String> {
    match tokio::time::timeout(SERVICE_FETCH_TIMEOUT, list_services(client, namespace)).await {
        Ok(Ok(list)) => Ok(list),
        Ok(Err(kube::Error::Api(api_err))) => Err(api_err.message),
        Ok(Err(err)) => Err(err.to_string()),
        Err(_) => Err("Timed out fetching services after 10s".to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use k8s_openapi::api::core::v1::{
        EndpointAddress, EndpointPort, EndpointSubset, LoadBalancerIngress, LoadBalancerStatus,
        ServicePort, ServiceSpec, ServiceStatus,
    };
    use k8s_openapi::apimachinery::pkg::apis::meta::v1::ObjectMeta;
    use std::collections::BTreeMap;

    #[test]
    fn test_map_service_handles_clusterip_nodeport_loadbalancer_externalname() {
        let mut selector = BTreeMap::new();
        selector.insert("app".to_string(), "api".to_string());
        let mut labels = BTreeMap::new();
        labels.insert("tier".to_string(), "backend".to_string());

        let cluster_ip_svc = Service {
            metadata: ObjectMeta {
                name: Some("api-svc".to_string()),
                namespace: Some("prod".to_string()),
                uid: Some("uid-1".to_string()),
                labels: Some(labels.clone()),
                ..Default::default()
            },
            spec: Some(ServiceSpec {
                type_: Some("ClusterIP".to_string()),
                cluster_ip: Some("10.96.0.10".to_string()),
                selector: Some(selector.clone()),
                ports: Some(vec![ServicePort {
                    port: 80,
                    protocol: Some("TCP".to_string()),
                    ..Default::default()
                }]),
                ..Default::default()
            }),
            ..Default::default()
        };

        let mapped = map_service(&cluster_ip_svc);
        assert_eq!(mapped.name, "api-svc");
        assert_eq!(mapped.namespace, "prod");
        assert_eq!(mapped.r#type, "ClusterIP");
        assert_eq!(mapped.cluster_ip, "10.96.0.10");
        assert_eq!(mapped.external_ip, "-");
        assert_eq!(mapped.ports, "80/TCP");
        assert_eq!(mapped.endpoints, "-");
        assert_eq!(mapped.selector, selector);
        assert_eq!(mapped.labels, labels);
        assert_eq!(mapped.ports_list.len(), 1);

        let nodeport_svc = Service {
            metadata: ObjectMeta {
                name: Some("np-svc".to_string()),
                namespace: Some("default".to_string()),
                ..Default::default()
            },
            spec: Some(ServiceSpec {
                type_: Some("NodePort".to_string()),
                cluster_ip: Some("10.96.0.20".to_string()),
                ports: Some(vec![ServicePort {
                    port: 443,
                    node_port: Some(30443),
                    protocol: Some("TCP".to_string()),
                    ..Default::default()
                }]),
                ..Default::default()
            }),
            ..Default::default()
        };
        let np_mapped = map_service(&nodeport_svc);
        assert_eq!(np_mapped.r#type, "NodePort");
        assert_eq!(np_mapped.ports, "443:30443/TCP");

        let lb_ready_svc = Service {
            metadata: ObjectMeta {
                name: Some("lb-ready".to_string()),
                namespace: Some("default".to_string()),
                ..Default::default()
            },
            spec: Some(ServiceSpec {
                type_: Some("LoadBalancer".to_string()),
                cluster_ip: Some("10.96.0.30".to_string()),
                ..Default::default()
            }),
            status: Some(ServiceStatus {
                load_balancer: Some(LoadBalancerStatus {
                    ingress: Some(vec![LoadBalancerIngress {
                        ip: Some("34.120.0.1".to_string()),
                        ..Default::default()
                    }]),
                }),
                ..Default::default()
            }),
        };
        assert_eq!(map_service(&lb_ready_svc).external_ip, "34.120.0.1");

        let lb_pending_svc = Service {
            metadata: ObjectMeta {
                name: Some("lb-pending".to_string()),
                namespace: Some("default".to_string()),
                ..Default::default()
            },
            spec: Some(ServiceSpec {
                type_: Some("LoadBalancer".to_string()),
                cluster_ip: Some("10.96.0.31".to_string()),
                ..Default::default()
            }),
            ..Default::default()
        };
        assert_eq!(map_service(&lb_pending_svc).external_ip, "<pending>");

        let ext_name_svc = Service {
            metadata: ObjectMeta {
                name: Some("ext-svc".to_string()),
                namespace: Some("default".to_string()),
                ..Default::default()
            },
            spec: Some(ServiceSpec {
                type_: Some("ExternalName".to_string()),
                external_name: Some("db.example.com".to_string()),
                ..Default::default()
            }),
            ..Default::default()
        };
        let ext_mapped = map_service(&ext_name_svc);
        assert_eq!(ext_mapped.r#type, "ExternalName");
        assert_eq!(ext_mapped.external_ip, "db.example.com");
    }

    #[test]
    fn test_apply_endpoints_to_service_formats_ready_addresses_and_summary() {
        let mut svc = map_service(&Service {
            metadata: ObjectMeta {
                name: Some("api-svc".to_string()),
                namespace: Some("prod".to_string()),
                ..Default::default()
            },
            ..Default::default()
        });

        let endpoints = Endpoints {
            metadata: ObjectMeta {
                name: Some("api-svc".to_string()),
                namespace: Some("prod".to_string()),
                ..Default::default()
            },
            subsets: Some(vec![EndpointSubset {
                addresses: Some(vec![
                    EndpointAddress {
                        ip: "10.244.0.5".to_string(),
                        ..Default::default()
                    },
                    EndpointAddress {
                        ip: "10.244.0.6".to_string(),
                        ..Default::default()
                    },
                ]),
                ports: Some(vec![EndpointPort {
                    port: 8080,
                    ..Default::default()
                }]),
                ..Default::default()
            }]),
        };

        apply_endpoints_to_service(&mut svc, &endpoints);
        assert_eq!(
            svc.endpoints_list,
            vec!["10.244.0.5:8080".to_string(), "10.244.0.6:8080".to_string()]
        );
        assert_eq!(svc.endpoints, "2 ready");

        let empty_endpoints = Endpoints::default();
        apply_endpoints_to_service(&mut svc, &empty_endpoints);
        assert!(svc.endpoints_list.is_empty());
        assert_eq!(svc.endpoints, "-");
    }
}
