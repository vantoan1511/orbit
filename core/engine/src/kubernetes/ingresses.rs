use kube::{
    api::{Api, ListParams, PostParams},
    Client,
};
use k8s_openapi::api::networking::v1::Ingress;
use crate::kubernetes::{models, format_age};

pub fn map_ingress(ing: &Ingress) -> models::IngressInfo {
    let name = ing.metadata.name.clone().unwrap_or_default();
    let namespace = ing.metadata.namespace.clone().unwrap_or_default();
    let uid = ing.metadata.uid.clone().unwrap_or_default();

    let age = format_age(&ing.metadata.creation_timestamp);
    let created = ing.metadata.creation_timestamp.as_ref()
        .map(|t| t.0.format("%b %d, %Y, %I:%M %p").to_string())
        .unwrap_or_default();

    let spec = ing.spec.as_ref();
    let class_name = spec.and_then(|sp| sp.ingress_class_name.clone());

    // Extract hosts and rules summary
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

    // Extract LoadBalancer address or IP
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

    // Extract ports (80, 443 if TLS present)
    let mut ports_vec = vec!["80"];
    if let Some(_tls) = spec.and_then(|sp| sp.tls.as_ref()).filter(|t| !t.is_empty()) {
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

pub async fn list_ingresses(client: &Client, namespace: Option<String>) -> Result<Vec<models::IngressInfo>, kube::Error> {
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

pub async fn clone_ingress(
    client: &Client,
    source_namespace: &str,
    source_name: &str,
    new_name: &str,
    new_namespace: &str,
    new_hosts: Vec<String>,
) -> Result<(), kube::Error> {
    let source_api: Api<Ingress> = Api::namespaced(client.clone(), source_namespace);
    let mut cloned = source_api.get(source_name).await?;

    // Strip server-managed metadata
    cloned.metadata.uid = None;
    cloned.metadata.resource_version = None;
    cloned.metadata.creation_timestamp = None;
    cloned.metadata.generation = None;
    cloned.metadata.managed_fields = None;
    cloned.metadata.owner_references = None;
    cloned.metadata.finalizers = None;
    cloned.status = None;

    cloned.metadata.name = Some(new_name.to_string());
    cloned.metadata.namespace = Some(new_namespace.to_string());

    // Patch hosts by index — only overwrite rules that have a corresponding new host
    if let Some(rules) = cloned.spec.as_mut().and_then(|s| s.rules.as_mut()) {
        for (rule, new_host) in rules.iter_mut().zip(new_hosts.iter()) {
            if !new_host.is_empty() {
                rule.host = Some(new_host.clone());
            }
        }
    }

    let target_api: Api<Ingress> = Api::namespaced(client.clone(), new_namespace);
    target_api.create(&PostParams::default(), &cloned).await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use k8s_openapi::api::networking::v1::{IngressRule, IngressSpec};
    use k8s_openapi::apimachinery::pkg::apis::meta::v1::ObjectMeta;

    #[test]
    fn test_map_ingress_basic() {
        let ing = Ingress {
            metadata: ObjectMeta {
                name: Some("test-ingress".to_string()),
                namespace: Some("default".to_string()),
                uid: Some("uid-123".to_string()),
                ..Default::default()
            },
            spec: Some(IngressSpec {
                ingress_class_name: Some("nginx".to_string()),
                rules: Some(vec![
                    IngressRule {
                        host: Some("example.com".to_string()),
                        http: None,
                    }
                ]),
                ..Default::default()
            }),
            status: None,
        };

        let mapped = map_ingress(&ing);
        assert_eq!(mapped.name, "test-ingress");
        assert_eq!(mapped.namespace, "default");
        assert_eq!(mapped.class_name, Some("nginx".to_string()));
        assert_eq!(mapped.hosts, "example.com");
    }

    #[test]
    fn test_patch_hosts_logic() {
        let mut ing = Ingress {
            metadata: ObjectMeta {
                name: Some("orig".to_string()),
                namespace: Some("ns1".to_string()),
                ..Default::default()
            },
            spec: Some(IngressSpec {
                rules: Some(vec![
                    IngressRule {
                        host: Some("old1.com".to_string()),
                        http: None,
                    },
                    IngressRule {
                        host: Some("old2.com".to_string()),
                        http: None,
                    },
                ]),
                ..Default::default()
            }),
            status: None,
        };

        let new_hosts = vec!["new1.com".to_string()];
        if let Some(rules) = ing.spec.as_mut().and_then(|s| s.rules.as_mut()) {
            for (rule, new_host) in rules.iter_mut().zip(new_hosts.iter()) {
                if !new_host.is_empty() {
                    rule.host = Some(new_host.clone());
                }
            }
        }

        let rules = ing.spec.unwrap().rules.unwrap();
        assert_eq!(rules[0].host, Some("new1.com".to_string()));
        assert_eq!(rules[1].host, Some("old2.com".to_string()));
    }
}
