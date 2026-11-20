use k8s_openapi::api::networking::v1::Ingress;
use kube::{
    api::{Api, PostParams},
    Client,
};

pub use orbit_core::kubernetes::ingresses::{list_ingresses, map_ingress};

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

        let new_hosts = ["new1.com".to_string()];
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
