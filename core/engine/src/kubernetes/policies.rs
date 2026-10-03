use kube::{
    api::{Api, ListParams},
    Client,
};
use k8s_openapi::api::networking::v1::NetworkPolicy;
use k8s_openapi::api::admissionregistration::v1::{ValidatingWebhookConfiguration, MutatingWebhookConfiguration};
use k8s_openapi::api::core::v1::{ResourceQuota, LimitRange};
use crate::kubernetes::models::PolicyInfo;
use crate::kubernetes::format_age;
use tokio::join;

pub async fn list_policies(client: &Client, namespace: Option<String>) -> Result<Vec<PolicyInfo>, kube::Error> {
    let mut all_policies = Vec::new();

    let net_api: Api<NetworkPolicy> = if let Some(ref ns) = namespace {
        Api::namespaced(client.clone(), ns)
    } else {
        Api::all(client.clone())
    };

    let quota_api: Api<ResourceQuota> = if let Some(ref ns) = namespace {
        Api::namespaced(client.clone(), ns)
    } else {
        Api::all(client.clone())
    };

    let limit_api: Api<LimitRange> = if let Some(ref ns) = namespace {
        Api::namespaced(client.clone(), ns)
    } else {
        Api::all(client.clone())
    };

    let val_api: Api<ValidatingWebhookConfiguration> = Api::all(client.clone());
    let mut_api: Api<MutatingWebhookConfiguration> = Api::all(client.clone());
    let lp = ListParams::default();

    let (net_res, quota_res, limit_res, val_res, mut_res) = join!(
        net_api.list(&lp),
        quota_api.list(&lp),
        limit_api.list(&lp),
        val_api.list(&lp),
        mut_api.list(&lp)
    );

    if let Ok(list) = net_res {
        for p in list {
            all_policies.push(format_network_policy(&p));
        }
    } else if let Err(e) = net_res {
        tracing::warn!(error = %e, "Failed to list NetworkPolicies");
    }

    if let Ok(list) = quota_res {
        for p in list {
            all_policies.push(format_resource_quota(&p));
        }
    } else if let Err(e) = quota_res {
        tracing::warn!(error = %e, "Failed to list ResourceQuotas");
    }

    if let Ok(list) = limit_res {
        for p in list {
            all_policies.push(format_limit_range(&p));
        }
    } else if let Err(e) = limit_res {
        tracing::warn!(error = %e, "Failed to list LimitRanges");
    }

    if namespace.is_none() {
        if let Ok(list) = val_res {
            for p in list {
                all_policies.push(format_val_webhook(&p));
            }
        } else if let Err(e) = val_res {
            tracing::warn!(error = %e, "Failed to list ValidatingWebhookConfigurations");
        }

        if let Ok(list) = mut_res {
            for p in list {
                all_policies.push(format_mut_webhook(&p));
            }
        } else if let Err(e) = mut_res {
            tracing::warn!(error = %e, "Failed to list MutatingWebhookConfigurations");
        }
    }

    Ok(all_policies)
}

pub fn format_network_policy(p: &NetworkPolicy) -> PolicyInfo {
    let name = p.metadata.name.clone().unwrap_or_default();
    let ns = p.metadata.namespace.clone().unwrap_or_else(|| "-".to_string());
    let uid = p.metadata.uid.clone().unwrap_or_default();
    let age = format_age(&p.metadata.creation_timestamp);
    let rules = serde_yaml::to_string(&p.spec).unwrap_or_default();

    PolicyInfo {
        uid,
        name,
        r#type: "Network Policy".to_string(),
        scope: "Namespaced".to_string(),
        namespace: ns,
        status: "Enforced".to_string(),
        mode: "enforce".to_string(),
        violations: 0,
        last_updated: age,
        description: "Kubernetes NetworkPolicy".to_string(),
        rules,
        kind: "NetworkPolicy".to_string(),
    }
}

pub fn format_resource_quota(p: &ResourceQuota) -> PolicyInfo {
    let name = p.metadata.name.clone().unwrap_or_default();
    let ns = p.metadata.namespace.clone().unwrap_or_else(|| "-".to_string());
    let uid = p.metadata.uid.clone().unwrap_or_default();
    let age = format_age(&p.metadata.creation_timestamp);
    let rules = serde_yaml::to_string(&p.spec).unwrap_or_default();

    PolicyInfo {
        uid,
        name,
        r#type: "Resource Quota".to_string(),
        scope: "Namespaced".to_string(),
        namespace: ns,
        status: "Enforced".to_string(),
        mode: "enforce".to_string(),
        violations: 0,
        last_updated: age,
        description: "Kubernetes ResourceQuota".to_string(),
        rules,
        kind: "ResourceQuota".to_string(),
    }
}

pub fn format_limit_range(p: &LimitRange) -> PolicyInfo {
    let name = p.metadata.name.clone().unwrap_or_default();
    let ns = p.metadata.namespace.clone().unwrap_or_else(|| "-".to_string());
    let uid = p.metadata.uid.clone().unwrap_or_default();
    let age = format_age(&p.metadata.creation_timestamp);
    let rules = serde_yaml::to_string(&p.spec).unwrap_or_default();

    PolicyInfo {
        uid,
        name,
        r#type: "Limit Range".to_string(),
        scope: "Namespaced".to_string(),
        namespace: ns,
        status: "Enforced".to_string(),
        mode: "enforce".to_string(),
        violations: 0,
        last_updated: age,
        description: "Kubernetes LimitRange".to_string(),
        rules,
        kind: "LimitRange".to_string(),
    }
}

pub fn format_val_webhook(p: &ValidatingWebhookConfiguration) -> PolicyInfo {
    let name = p.metadata.name.clone().unwrap_or_default();
    let uid = p.metadata.uid.clone().unwrap_or_default();
    let age = format_age(&p.metadata.creation_timestamp);
    let rules = serde_yaml::to_string(&p.webhooks).unwrap_or_default();

    PolicyInfo {
        uid,
        name,
        r#type: "Admission Policy".to_string(),
        scope: "Cluster".to_string(),
        namespace: "-".to_string(),
        status: "Enforced".to_string(),
        mode: "enforce".to_string(),
        violations: 0,
        last_updated: age,
        description: "Kubernetes ValidatingWebhookConfiguration".to_string(),
        rules,
        kind: "ValidatingWebhookConfiguration".to_string(),
    }
}

pub fn format_mut_webhook(p: &MutatingWebhookConfiguration) -> PolicyInfo {
    let name = p.metadata.name.clone().unwrap_or_default();
    let uid = p.metadata.uid.clone().unwrap_or_default();
    let age = format_age(&p.metadata.creation_timestamp);
    let rules = serde_yaml::to_string(&p.webhooks).unwrap_or_default();

    PolicyInfo {
        uid,
        name,
        r#type: "Admission Policy".to_string(),
        scope: "Cluster".to_string(),
        namespace: "-".to_string(),
        status: "Enforced".to_string(),
        mode: "enforce".to_string(),
        violations: 0,
        last_updated: age,
        description: "Kubernetes MutatingWebhookConfiguration".to_string(),
        rules,
        kind: "MutatingWebhookConfiguration".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use k8s_openapi::apimachinery::pkg::apis::meta::v1::ObjectMeta;

    #[test]
    fn test_format_network_policy_sets_kind() {
        let p = NetworkPolicy {
            metadata: ObjectMeta {
                name: Some("test-netpol".to_string()),
                namespace: Some("default".to_string()),
                ..Default::default()
            },
            ..Default::default()
        };
        let info = format_network_policy(&p);
        assert_eq!(info.kind, "NetworkPolicy");
        assert_eq!(info.name, "test-netpol");
    }

    #[test]
    fn test_format_resource_quota_sets_kind() {
        let p = ResourceQuota {
            metadata: ObjectMeta {
                name: Some("test-quota".to_string()),
                namespace: Some("default".to_string()),
                ..Default::default()
            },
            ..Default::default()
        };
        let info = format_resource_quota(&p);
        assert_eq!(info.kind, "ResourceQuota");
        assert_eq!(info.name, "test-quota");
    }

    #[test]
    fn test_format_limit_range_sets_kind() {
        let p = LimitRange {
            metadata: ObjectMeta {
                name: Some("test-limit".to_string()),
                namespace: Some("default".to_string()),
                ..Default::default()
            },
            ..Default::default()
        };
        let info = format_limit_range(&p);
        assert_eq!(info.kind, "LimitRange");
        assert_eq!(info.name, "test-limit");
    }

    #[test]
    fn test_format_val_webhook_sets_kind() {
        let p = ValidatingWebhookConfiguration {
            metadata: ObjectMeta {
                name: Some("test-val-webhook".to_string()),
                ..Default::default()
            },
            ..Default::default()
        };
        let info = format_val_webhook(&p);
        assert_eq!(info.kind, "ValidatingWebhookConfiguration");
        assert_eq!(info.name, "test-val-webhook");
    }

    #[test]
    fn test_format_mut_webhook_sets_kind() {
        let p = MutatingWebhookConfiguration {
            metadata: ObjectMeta {
                name: Some("test-mut-webhook".to_string()),
                ..Default::default()
            },
            ..Default::default()
        };
        let info = format_mut_webhook(&p);
        assert_eq!(info.kind, "MutatingWebhookConfiguration");
        assert_eq!(info.name, "test-mut-webhook");
    }
}
