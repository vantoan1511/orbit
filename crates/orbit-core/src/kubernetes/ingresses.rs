use std::time::Duration;

use k8s_openapi::api::networking::v1::Ingress;
use kube::{
    api::{Api, ListParams},
    Client,
};

use crate::kubernetes::{models, namespaces::format_age};

pub const INGRESS_FETCH_TIMEOUT: Duration = Duration::from_secs(10);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedIngressRule {
    pub host: String,
    pub path: String,
    pub backend: String,
}

pub fn parse_ingress_rule_summary(rule: &str, fallback_host: &str) -> ParsedIngressRule {
    let trimmed = rule.trim();
    if let Some((host_part, rest)) = trimmed.split_once("->") {
        let host = host_part.trim();
        let resolved_host = if host.is_empty() { fallback_host } else { host };
        let rest_trimmed = rest.trim();

        if let (Some(open_idx), Some(close_idx)) = (rest_trimmed.rfind('('), rest_trimmed.rfind(')'))
        {
            if open_idx < close_idx {
                let path = rest_trimmed[..open_idx].trim();
                let backend = rest_trimmed[open_idx + 1..close_idx].trim();
                return ParsedIngressRule {
                    host: resolved_host.to_string(),
                    path: if path.is_empty() {
                        "/".to_string()
                    } else {
                        path.to_string()
                    },
                    backend: if backend.is_empty() {
                        "-".to_string()
                    } else {
                        backend.to_string()
                    },
                };
            }
        }

        return ParsedIngressRule {
            host: resolved_host.to_string(),
            path: if rest_trimmed.is_empty() {
                "/".to_string()
            } else {
                rest_trimmed.to_string()
            },
            backend: "-".to_string(),
        };
    }

    ParsedIngressRule {
        host: fallback_host.to_string(),
        path: "/".to_string(),
        backend: if trimmed.is_empty() {
            "-".to_string()
        } else {
            trimmed.to_string()
        },
    }
}

pub fn map_ingress(ing: &Ingress) -> models::IngressInfo {
    let name = ing.metadata.name.clone().unwrap_or_default();
    let namespace = ing.metadata.namespace.clone().unwrap_or_default();
    let uid = ing.metadata.uid.clone().unwrap_or_default();

    let age = format_age(&ing.metadata.creation_timestamp);
    let created = ing
        .metadata
        .creation_timestamp
        .as_ref()
        .map(|t| t.0.format("%b %d, %Y, %I:%M %p").to_string())
        .unwrap_or_default();

    let spec = ing.spec.as_ref();
    let class_name = spec
        .and_then(|sp| sp.ingress_class_name.clone())
        .filter(|c| !c.trim().is_empty());

    let mut hosts_vec: Vec<String> = Vec::new();
    let mut rules_summary = Vec::new();

    if let Some(rules) = spec.and_then(|sp| sp.rules.as_ref()) {
        for rule in rules {
            if let Some(host) = rule.host.as_ref().filter(|h| !hosts_vec.contains(h)) {
                hosts_vec.push(host.clone());
            }
            if let Some(http) = &rule.http {
                for path in &http.paths {
                    let p = path.path.as_deref().unwrap_or("/");
                    let port_str = path
                        .backend
                        .service
                        .as_ref()
                        .and_then(|svc| svc.port.as_ref())
                        .and_then(|p| p.number.map(|n| n.to_string()).or_else(|| p.name.clone()))
                        .unwrap_or_else(|| "*".to_string());

                    let backend = match &path.backend.service {
                        Some(svc) => format!("{}:{}", svc.name, port_str),
                        None => "unknown".to_string(),
                    };
                    let host_str = rule.host.as_deref().unwrap_or("*");
                    rules_summary.push(format!("{} -> {} ({})", host_str, p, backend));
                }
            }
        }
    }

    let hosts = if hosts_vec.is_empty() {
        "*".to_string()
    } else {
        hosts_vec.join(", ")
    };

    let mut address_vec = Vec::new();
    if let Some(ingress_status) = ing
        .status
        .as_ref()
        .and_then(|s| s.load_balancer.as_ref())
        .and_then(|lb| lb.ingress.as_ref())
    {
        for ing_stat in ingress_status {
            if let Some(ip) = &ing_stat.ip {
                address_vec.push(ip.clone());
            } else if let Some(hostname) = &ing_stat.hostname {
                address_vec.push(hostname.clone());
            }
        }
    }

    let address = if address_vec.is_empty() {
        "-".to_string()
    } else {
        address_vec.join(", ")
    };

    let mut ports_vec = vec!["80"];
    if spec
        .and_then(|sp| sp.tls.as_ref())
        .is_some_and(|t| !t.is_empty())
    {
        ports_vec.push("443");
    }
    let ports = ports_vec.join(", ");

    let labels = ing.metadata.labels.clone().unwrap_or_default();
    let annotations = ing.metadata.annotations.clone().unwrap_or_default();

    models::IngressInfo {
        name,
        namespace,
        class_name,
        hosts,
        address,
        ports,
        age,
        created,
        uid,
        labels,
        annotations,
        rules_summary,
    }
}

pub async fn list_ingresses(
    client: &Client,
    namespace: Option<String>,
) -> Result<Vec<models::IngressInfo>, kube::Error> {
    let api: Api<Ingress> = if let Some(ns) = namespace {
        Api::namespaced(client.clone(), &ns)
    } else {
        Api::all(client.clone())
    };

    let mut list = Vec::new();
    for ing in api.list(&ListParams::default()).await? {
        list.push(map_ingress(&ing));
    }

    Ok(list)
}

pub async fn fetch_ingresses(
    client: &Client,
    namespace: Option<String>,
) -> Result<Vec<models::IngressInfo>, String> {
    match tokio::time::timeout(INGRESS_FETCH_TIMEOUT, list_ingresses(client, namespace)).await {
        Ok(Ok(list)) => Ok(list),
        Ok(Err(kube::Error::Api(api_err))) => Err(api_err.message),
        Ok(Err(err)) => Err(err.to_string()),
        Err(_) => Err("Timed out fetching ingresses after 10s".to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use k8s_openapi::api::networking::v1::{
        HTTPIngressPath, HTTPIngressRuleValue, IngressBackend, IngressLoadBalancerIngress,
        IngressLoadBalancerStatus, IngressRule, IngressServiceBackend, IngressSpec, IngressStatus,
        IngressTLS, ServiceBackendPort,
    };
    use k8s_openapi::apimachinery::pkg::apis::meta::v1::ObjectMeta;
    use std::collections::BTreeMap;

    #[test]
    fn test_map_ingress_extracts_hosts_rules_address_tls_ports_and_metadata() {
        let mut labels = BTreeMap::new();
        labels.insert("app".to_string(), "gateway".to_string());
        let mut annotations = BTreeMap::new();
        annotations.insert("nginx.ingress.kubernetes.io/rewrite-target".to_string(), "/".to_string());

        let ing = Ingress {
            metadata: ObjectMeta {
                name: Some("web-ingress".to_string()),
                namespace: Some("prod".to_string()),
                uid: Some("uid-ing-1".to_string()),
                labels: Some(labels.clone()),
                annotations: Some(annotations.clone()),
                ..Default::default()
            },
            spec: Some(IngressSpec {
                ingress_class_name: Some("nginx".to_string()),
                tls: Some(vec![IngressTLS {
                    hosts: Some(vec!["app.example.com".to_string()]),
                    secret_name: Some("app-tls".to_string()),
                }]),
                rules: Some(vec![
                    IngressRule {
                        host: Some("app.example.com".to_string()),
                        http: Some(HTTPIngressRuleValue {
                            paths: vec![
                                HTTPIngressPath {
                                    path: Some("/api".to_string()),
                                    path_type: "Prefix".to_string(),
                                    backend: IngressBackend {
                                        service: Some(IngressServiceBackend {
                                            name: "api-svc".to_string(),
                                            port: Some(ServiceBackendPort {
                                                number: Some(8080),
                                                name: None,
                                            }),
                                        }),
                                        resource: None,
                                    },
                                },
                                HTTPIngressPath {
                                    path: Some("/web".to_string()),
                                    path_type: "Prefix".to_string(),
                                    backend: IngressBackend {
                                        service: Some(IngressServiceBackend {
                                            name: "web-svc".to_string(),
                                            port: Some(ServiceBackendPort {
                                                number: None,
                                                name: Some("http".to_string()),
                                            }),
                                        }),
                                        resource: None,
                                    },
                                },
                            ],
                        }),
                    },
                    IngressRule {
                        host: Some("app.example.com".to_string()),
                        http: None,
                    },
                    IngressRule {
                        host: Some("admin.example.com".to_string()),
                        http: None,
                    },
                ]),
                ..Default::default()
            }),
            status: Some(IngressStatus {
                load_balancer: Some(IngressLoadBalancerStatus {
                    ingress: Some(vec![
                        IngressLoadBalancerIngress {
                            ip: Some("34.120.10.1".to_string()),
                            hostname: None,
                            ports: None,
                        },
                        IngressLoadBalancerIngress {
                            ip: None,
                            hostname: Some("lb.example.com".to_string()),
                            ports: None,
                        },
                    ]),
                }),
            }),
        };

        let mapped = map_ingress(&ing);
        assert_eq!(mapped.name, "web-ingress");
        assert_eq!(mapped.namespace, "prod");
        assert_eq!(mapped.uid, "uid-ing-1");
        assert_eq!(mapped.class_name.as_deref(), Some("nginx"));
        assert_eq!(mapped.hosts, "app.example.com, admin.example.com");
        assert_eq!(mapped.address, "34.120.10.1, lb.example.com");
        assert_eq!(mapped.ports, "80, 443");
        assert_eq!(
            mapped.rules_summary,
            vec![
                "app.example.com -> /api (api-svc:8080)".to_string(),
                "app.example.com -> /web (web-svc:http)".to_string(),
            ]
        );
        assert_eq!(mapped.labels, labels);
        assert_eq!(mapped.annotations, annotations);
    }

    #[test]
    fn test_map_ingress_defaults_empty_hosts_address_and_http_only_ports() {
        let ing = Ingress {
            metadata: ObjectMeta {
                name: Some("minimal".to_string()),
                namespace: Some("default".to_string()),
                ..Default::default()
            },
            spec: None,
            status: None,
        };

        let mapped = map_ingress(&ing);
        assert_eq!(mapped.name, "minimal");
        assert_eq!(mapped.namespace, "default");
        assert_eq!(mapped.class_name, None);
        assert_eq!(mapped.hosts, "*");
        assert_eq!(mapped.address, "-");
        assert_eq!(mapped.ports, "80");
        assert!(mapped.rules_summary.is_empty());
    }

    #[test]
    fn test_parse_ingress_rule_summary_parses_structured_and_fallback_rules() {
        assert_eq!(
            parse_ingress_rule_summary("app.example.com -> /api (api-svc:8080)", "*"),
            ParsedIngressRule {
                host: "app.example.com".to_string(),
                path: "/api".to_string(),
                backend: "api-svc:8080".to_string(),
            }
        );

        assert_eq!(
            parse_ingress_rule_summary(" -> /health (health-svc:http)", "fallback.local"),
            ParsedIngressRule {
                host: "fallback.local".to_string(),
                path: "/health".to_string(),
                backend: "health-svc:http".to_string(),
            }
        );

        assert_eq!(
            parse_ingress_rule_summary("malformed-rule", "fallback.local"),
            ParsedIngressRule {
                host: "fallback.local".to_string(),
                path: "/".to_string(),
                backend: "malformed-rule".to_string(),
            }
        );
    }
}
