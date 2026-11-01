use kube::{
    api::{Api, ListParams},
    Client,
};
use k8s_openapi::api::networking::v1::NetworkPolicy;
use k8s_openapi::api::admissionregistration::v1::{ValidatingWebhookConfiguration, MutatingWebhookConfiguration};
use k8s_openapi::api::core::v1::{ResourceQuota, LimitRange, Namespace};
use crate::kubernetes::models::PolicyInfo;
use crate::kubernetes::format_age;
use crate::kubernetes::nodes::{parse_cpu_quantity, parse_memory_quantity};
use tokio::join;

fn parse_quota_quantity(key: &str, q: &str) -> f64 {
    let lower = key.to_ascii_lowercase();
    if lower.contains("cpu") {
        parse_cpu_quantity(q)
    } else if lower.contains("memory") || lower.contains("storage") {
        parse_memory_quantity(q)
    } else {
        q.trim().parse::<f64>().unwrap_or_else(|_| parse_cpu_quantity(q))
    }
}

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

    let ns_future = {
        let client_clone = client.clone();
        let ns_opt = namespace.clone();
        let lp_clone = lp.clone();
        async move {
            let ns_api: Api<Namespace> = Api::all(client_clone);
            if let Some(ref ns) = ns_opt {
                match ns_api.get(ns).await {
                    Ok(n) => Ok(vec![n]),
                    Err(e) => Err(e),
                }
            } else {
                match ns_api.list(&lp_clone).await {
                    Ok(list) => Ok(list.items),
                    Err(e) => Err(e),
                }
            }
        }
    };

    let (net_res, quota_res, limit_res, val_res, mut_res, ns_res) = join!(
        net_api.list(&lp),
        quota_api.list(&lp),
        limit_api.list(&lp),
        val_api.list(&lp),
        mut_api.list(&lp),
        ns_future
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

    if let Ok(namespaces) = ns_res {
        for ns in namespaces {
            if let Some(psa) = format_pod_security_namespace(&ns) {
                all_policies.push(psa);
            }
        }
    } else if let Err(e) = ns_res {
        tracing::warn!(error = %e, "Failed to list Namespaces for Pod Security Admission");
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

    let mode = match p.spec.as_ref().and_then(|s| s.policy_types.as_ref()) {
        Some(types) if !types.is_empty() => types.join(", "),
        _ => {
            if p.spec.as_ref().and_then(|s| s.egress.as_ref()).is_some() {
                "Ingress, Egress".to_string()
            } else {
                "Ingress".to_string()
            }
        }
    };

    let description = if let Some(spec) = &p.spec {
        let mut selectors = Vec::new();
        if let Some(match_labels) = &spec.pod_selector.match_labels {
            for (k, v) in match_labels {
                selectors.push(format!("{}={}", k, v));
            }
        }
        if selectors.is_empty() {
            if let Some(exprs) = &spec.pod_selector.match_expressions {
                if !exprs.is_empty() {
                    format!("Pods matching {} expression(s)", exprs.len())
                } else {
                    "All pods in namespace".to_string()
                }
            } else {
                "All pods in namespace".to_string()
            }
        } else {
            format!("Pods matching: {}", selectors.join(", "))
        }
    } else {
        "Kubernetes NetworkPolicy".to_string()
    };

    PolicyInfo {
        uid,
        name,
        r#type: "Network Policy".to_string(),
        scope: "Namespaced".to_string(),
        namespace: ns,
        status: "Enforced".to_string(),
        mode,
        violations: 0,
        last_updated: age,
        description,
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

    let hard = p
        .status
        .as_ref()
        .and_then(|s| s.hard.as_ref())
        .or_else(|| p.spec.as_ref().and_then(|s| s.hard.as_ref()));
    let used = p.status.as_ref().and_then(|s| s.used.as_ref());

    let mut violations = 0;
    let hard_count = hard.map(|h| h.len()).unwrap_or(0);
    let used_count = used.map(|u| u.len()).unwrap_or(0);

    if let (Some(hard_map), Some(used_map)) = (hard, used) {
        for (k, hard_q) in hard_map {
            if let Some(used_q) = used_map.get(k) {
                let hard_val = parse_quota_quantity(k, &hard_q.0);
                let used_val = parse_quota_quantity(k, &used_q.0);
                if hard_val > 0.0 && used_val >= hard_val {
                    violations += 1;
                }
            }
        }
    }

    PolicyInfo {
        uid,
        name,
        r#type: "Resource Quota".to_string(),
        scope: "Namespaced".to_string(),
        namespace: ns,
        status: "Enforced".to_string(),
        mode: "enforce".to_string(),
        violations,
        last_updated: age,
        description: format!("Resource limits ({}/{} tracked)", used_count, hard_count),
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

    let limits = p.spec.as_ref().map(|s| s.limits.as_slice()).unwrap_or_default();
    let (status, mode, description) = if limits.is_empty() {
        ("Disabled".to_string(), "disabled".to_string(), "No limits defined".to_string())
    } else {
        let mut item_types: Vec<String> = limits.iter().map(|l| l.type_.clone()).collect();
        item_types.sort();
        item_types.dedup();
        (
            "Enforced".to_string(),
            "enforce".to_string(),
            format!("Limits for: {}", item_types.join(", ")),
        )
    };

    PolicyInfo {
        uid,
        name,
        r#type: "Limit Range".to_string(),
        scope: "Namespaced".to_string(),
        namespace: ns,
        status,
        mode,
        violations: 0,
        last_updated: age,
        description,
        rules,
        kind: "LimitRange".to_string(),
    }
}

pub fn format_val_webhook(p: &ValidatingWebhookConfiguration) -> PolicyInfo {
    let name = p.metadata.name.clone().unwrap_or_default();
    let uid = p.metadata.uid.clone().unwrap_or_default();
    let age = format_age(&p.metadata.creation_timestamp);
    let rules = serde_yaml::to_string(&p.webhooks).unwrap_or_default();

    let webhooks = p.webhooks.as_deref().unwrap_or_default();
    let count = webhooks.len();
    let (status, mode) = if count == 0 {
        ("Disabled".to_string(), "disabled".to_string())
    } else {
        let has_fail = webhooks.iter().any(|w| {
            w.failure_policy.as_deref().map(|fp| fp.eq_ignore_ascii_case("Fail")).unwrap_or(true)
        });
        if has_fail {
            ("Enforced".to_string(), "Fail".to_string())
        } else {
            ("Audit".to_string(), "Ignore".to_string())
        }
    };

    PolicyInfo {
        uid,
        name,
        r#type: "Admission Policy".to_string(),
        scope: "Cluster".to_string(),
        namespace: "-".to_string(),
        status,
        mode,
        violations: 0,
        last_updated: age,
        description: format!("{} webhook(s)", count),
        rules,
        kind: "ValidatingWebhookConfiguration".to_string(),
    }
}

pub fn format_mut_webhook(p: &MutatingWebhookConfiguration) -> PolicyInfo {
    let name = p.metadata.name.clone().unwrap_or_default();
    let uid = p.metadata.uid.clone().unwrap_or_default();
    let age = format_age(&p.metadata.creation_timestamp);
    let rules = serde_yaml::to_string(&p.webhooks).unwrap_or_default();

    let webhooks = p.webhooks.as_deref().unwrap_or_default();
    let count = webhooks.len();
    let (status, mode) = if count == 0 {
        ("Disabled".to_string(), "disabled".to_string())
    } else {
        let has_fail = webhooks.iter().any(|w| {
            w.failure_policy.as_deref().map(|fp| fp.eq_ignore_ascii_case("Fail")).unwrap_or(true)
        });
        if has_fail {
            ("Enforced".to_string(), "Fail".to_string())
        } else {
            ("Audit".to_string(), "Ignore".to_string())
        }
    };

    PolicyInfo {
        uid,
        name,
        r#type: "Admission Policy".to_string(),
        scope: "Cluster".to_string(),
        namespace: "-".to_string(),
        status,
        mode,
        violations: 0,
        last_updated: age,
        description: format!("{} webhook(s)", count),
        rules,
        kind: "MutatingWebhookConfiguration".to_string(),
    }
}

pub fn format_pod_security_namespace(ns: &Namespace) -> Option<PolicyInfo> {
    let labels = ns.metadata.labels.as_ref()?;
    let enforce_lvl = labels.get("pod-security.kubernetes.io/enforce");
    let audit_lvl = labels.get("pod-security.kubernetes.io/audit");
    let warn_lvl = labels.get("pod-security.kubernetes.io/warn");

    if enforce_lvl.is_none() && audit_lvl.is_none() && warn_lvl.is_none() {
        return None;
    }

    let ns_name = ns.metadata.name.clone().unwrap_or_default();
    let uid = ns.metadata.uid.clone().unwrap_or_default();
    let age = format_age(&ns.metadata.creation_timestamp);

    let status = if enforce_lvl.is_some() {
        "Enforced".to_string()
    } else {
        "Audit".to_string()
    };

    let mode = if let Some(e) = enforce_lvl {
        format!("enforce: {}", e)
    } else if let Some(a) = audit_lvl {
        format!("audit: {}", a)
    } else if let Some(w) = warn_lvl {
        format!("warn: {}", w)
    } else {
        "audit".to_string()
    };

    let mut parts = Vec::new();
    if let Some(e) = enforce_lvl {
        parts.push(format!("enforce={}", e));
    }
    if let Some(a) = audit_lvl {
        parts.push(format!("audit={}", a));
    }
    if let Some(w) = warn_lvl {
        parts.push(format!("warn={}", w));
    }
    let description = format!("PSA: {}", parts.join(", "));

    let psa_labels: std::collections::BTreeMap<_, _> = labels
        .iter()
        .filter(|(k, _)| k.starts_with("pod-security.kubernetes.io/"))
        .map(|(k, v)| (k.clone(), v.clone()))
        .collect();
    let rules = serde_yaml::to_string(&psa_labels).unwrap_or_default();

    Some(PolicyInfo {
        uid,
        name: format!("psa-{}", ns_name),
        r#type: "Pod Security".to_string(),
        scope: "Namespaced".to_string(),
        namespace: ns_name,
        status,
        mode,
        violations: 0,
        last_updated: age,
        description,
        rules,
        kind: "Namespace".to_string(),
    })
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

    #[test]
    fn test_format_val_webhook_failure_policy_ignore_is_audit() {
        use k8s_openapi::api::admissionregistration::v1::ValidatingWebhook;
        let p = ValidatingWebhookConfiguration {
            metadata: ObjectMeta {
                name: Some("audit-webhook".to_string()),
                ..Default::default()
            },
            webhooks: Some(vec![
                ValidatingWebhook {
                    failure_policy: Some("Ignore".to_string()),
                    name: "hook1.example.com".to_string(),
                    ..Default::default()
                }
            ]),
        };
        let info = format_val_webhook(&p);
        assert_eq!(info.status, "Audit");
        assert_eq!(info.mode, "Ignore");
        assert_eq!(info.description, "1 webhook(s)");
    }

    #[test]
    fn test_format_val_webhook_failure_policy_fail_is_enforced() {
        use k8s_openapi::api::admissionregistration::v1::ValidatingWebhook;
        let p = ValidatingWebhookConfiguration {
            metadata: ObjectMeta {
                name: Some("enforce-webhook".to_string()),
                ..Default::default()
            },
            webhooks: Some(vec![
                ValidatingWebhook {
                    failure_policy: Some("Fail".to_string()),
                    name: "hook2.example.com".to_string(),
                    ..Default::default()
                }
            ]),
        };
        let info = format_val_webhook(&p);
        assert_eq!(info.status, "Enforced");
        assert_eq!(info.mode, "Fail");
        assert_eq!(info.description, "1 webhook(s)");
    }

    #[test]
    fn test_format_resource_quota_violations_calculation() {
        use k8s_openapi::api::core::v1::ResourceQuotaStatus;
        use k8s_openapi::apimachinery::pkg::api::resource::Quantity;
        use std::collections::BTreeMap;

        let mut hard = BTreeMap::new();
        hard.insert("cpu".to_string(), Quantity("4".to_string()));
        hard.insert("memory".to_string(), Quantity("8Gi".to_string()));
        hard.insert("pods".to_string(), Quantity("10".to_string()));

        let mut used = BTreeMap::new();
        used.insert("cpu".to_string(), Quantity("5".to_string())); // over quota
        used.insert("memory".to_string(), Quantity("4Gi".to_string())); // within quota
        used.insert("pods".to_string(), Quantity("10".to_string())); // at limit: violation

        let p = ResourceQuota {
            metadata: ObjectMeta {
                name: Some("compute-quota".to_string()),
                namespace: Some("prod".to_string()),
                ..Default::default()
            },
            status: Some(ResourceQuotaStatus {
                hard: Some(hard),
                used: Some(used),
            }),
            ..Default::default()
        };

        let info = format_resource_quota(&p);
        assert_eq!(info.status, "Enforced");
        assert_eq!(info.violations, 2);
        assert_eq!(info.description, "Resource limits (3/3 tracked)");
    }

    #[test]
    fn test_format_network_policy_modes_and_description() {
        use k8s_openapi::api::networking::v1::NetworkPolicySpec;
        use k8s_openapi::apimachinery::pkg::apis::meta::v1::LabelSelector;
        use std::collections::BTreeMap;

        let mut match_labels = BTreeMap::new();
        match_labels.insert("app".to_string(), "api".to_string());

        let p = NetworkPolicy {
            metadata: ObjectMeta {
                name: Some("api-netpol".to_string()),
                namespace: Some("default".to_string()),
                ..Default::default()
            },
            spec: Some(NetworkPolicySpec {
                policy_types: Some(vec!["Ingress".to_string(), "Egress".to_string()]),
                pod_selector: LabelSelector {
                    match_labels: Some(match_labels),
                    ..Default::default()
                },
                ..Default::default()
            }),
        };

        let info = format_network_policy(&p);
        assert_eq!(info.status, "Enforced");
        assert_eq!(info.mode, "Ingress, Egress");
        assert_eq!(info.description, "Pods matching: app=api");
    }

    #[test]
    fn test_format_limit_range_empty_is_disabled() {
        use k8s_openapi::api::core::v1::LimitRangeSpec;

        let p = LimitRange {
            metadata: ObjectMeta {
                name: Some("empty-limits".to_string()),
                namespace: Some("dev".to_string()),
                ..Default::default()
            },
            spec: Some(LimitRangeSpec {
                limits: vec![],
            }),
        };

        let info = format_limit_range(&p);
        assert_eq!(info.status, "Disabled");
        assert_eq!(info.mode, "disabled");
    }

    #[test]
    fn test_format_pod_security_namespace_labels() {
        use k8s_openapi::api::core::v1::Namespace;
        use std::collections::BTreeMap;

        let mut labels = BTreeMap::new();
        labels.insert("pod-security.kubernetes.io/enforce".to_string(), "restricted".to_string());
        labels.insert("pod-security.kubernetes.io/audit".to_string(), "baseline".to_string());

        let ns = Namespace {
            metadata: ObjectMeta {
                name: Some("secure-ns".to_string()),
                labels: Some(labels),
                ..Default::default()
            },
            ..Default::default()
        };

        let info = format_pod_security_namespace(&ns).expect("should format PSA info");
        assert_eq!(info.r#type, "Pod Security");
        assert_eq!(info.kind, "Namespace");
        assert_eq!(info.status, "Enforced");
        assert_eq!(info.mode, "enforce: restricted");
        assert_eq!(info.name, "psa-secure-ns");
        assert_eq!(info.namespace, "secure-ns");

        // Non-PSA namespace returns None
        let empty_ns = Namespace {
            metadata: ObjectMeta {
                name: Some("plain-ns".to_string()),
                ..Default::default()
            },
            ..Default::default()
        };
        assert!(format_pod_security_namespace(&empty_ns).is_none());
    }
}
