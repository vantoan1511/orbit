use std::time::Duration;

use k8s_openapi::api::admissionregistration::v1::{
    MutatingWebhookConfiguration, ValidatingWebhookConfiguration,
};
use k8s_openapi::api::core::v1::{LimitRange, Namespace, ResourceQuota};
use k8s_openapi::api::networking::v1::NetworkPolicy;
use kube::{
    api::{Api, ListParams},
    Client,
};
use tokio::join;

use crate::kubernetes::models::PolicyInfo;
use crate::kubernetes::namespaces::format_age;
use crate::kubernetes::nodes::{parse_cpu_quantity, parse_memory_quantity};

pub const POLICIES_FETCH_TIMEOUT: Duration = Duration::from_secs(10);

fn parse_quota_quantity(key: &str, q: &str) -> f64 {
    let lower = key.to_ascii_lowercase();
    if lower.contains("cpu") {
        parse_cpu_quantity(q)
    } else if lower.contains("memory") || lower.contains("storage") {
        parse_memory_quantity(q)
    } else {
        q.trim()
            .parse::<f64>()
            .unwrap_or_else(|_| parse_cpu_quantity(q))
    }
}

pub(crate) async fn query_cluster_scoped_if_applicable<T, F, Fut>(
    is_cluster: bool,
    fetch: F,
) -> Option<T>
where
    F: FnOnce() -> Fut,
    Fut: std::future::Future<Output = T>,
{
    if is_cluster {
        Some(fetch().await)
    } else {
        None
    }
}

pub async fn list_network_policies(
    client: &Client,
    namespace: Option<String>,
) -> Result<Vec<PolicyInfo>, kube::Error> {
    let net_api: Api<NetworkPolicy> = if let Some(ref ns) = namespace {
        Api::namespaced(client.clone(), ns)
    } else {
        Api::all(client.clone())
    };

    let list = net_api.list(&ListParams::default()).await?;
    Ok(list.iter().map(format_network_policy).collect())
}

pub async fn fetch_network_policies(
    client: &Client,
    namespace: Option<String>,
) -> Result<Vec<PolicyInfo>, String> {
    match tokio::time::timeout(
        POLICIES_FETCH_TIMEOUT,
        list_network_policies(client, namespace),
    )
    .await
    {
        Ok(Ok(list)) => Ok(list),
        Ok(Err(kube::Error::Api(api_err))) => Err(api_err.message),
        Ok(Err(err)) => Err(err.to_string()),
        Err(_) => Err("Timed out fetching network policies after 10s".to_string()),
    }
}

/// Lists Pod Security Admission policies extracted from Namespace `pod-security.kubernetes.io/*` labels.
pub async fn list_pod_security_policies(
    client: &Client,
    namespace: Option<String>,
) -> Result<Vec<PolicyInfo>, kube::Error> {
    let ns_api: Api<Namespace> = Api::all(client.clone());
    let namespaces = if let Some(ref ns) = namespace {
        vec![ns_api.get(ns).await?]
    } else {
        ns_api.list(&ListParams::default()).await?.items
    };

    Ok(namespaces
        .iter()
        .filter_map(format_pod_security_namespace)
        .collect())
}

pub async fn fetch_pod_security_policies(
    client: &Client,
    namespace: Option<String>,
) -> Result<Vec<PolicyInfo>, String> {
    match tokio::time::timeout(
        POLICIES_FETCH_TIMEOUT,
        list_pod_security_policies(client, namespace),
    )
    .await
    {
        Ok(Ok(list)) => Ok(list),
        Ok(Err(kube::Error::Api(api_err))) => Err(api_err.message),
        Ok(Err(err)) => Err(err.to_string()),
        Err(_) => Err("Timed out fetching pod security policies after 10s".to_string()),
    }
}

pub(crate) fn combine_resource_policy_results<E: std::fmt::Display>(
    quota_res: Result<Vec<ResourceQuota>, E>,
    limit_res: Result<Vec<LimitRange>, E>,
) -> Result<Vec<PolicyInfo>, E> {
    match (quota_res, limit_res) {
        (Ok(quotas), Ok(limits)) => {
            let mut policies = Vec::with_capacity(quotas.len() + limits.len());
            policies.extend(quotas.iter().map(format_resource_quota));
            policies.extend(limits.iter().map(format_limit_range));
            Ok(policies)
        }
        (Ok(quotas), Err(err)) => {
            tracing::warn!(error = %err, "Failed to list LimitRanges");
            Ok(quotas.iter().map(format_resource_quota).collect())
        }
        (Err(err), Ok(limits)) => {
            tracing::warn!(error = %err, "Failed to list ResourceQuotas");
            Ok(limits.iter().map(format_limit_range).collect())
        }
        (Err(quota_err), Err(limit_err)) => {
            tracing::warn!(error = %limit_err, "Failed to list LimitRanges");
            Err(quota_err)
        }
    }
}

/// Lists ResourceQuotas and LimitRanges in parallel.
/// If both fail, returns Err with the primary API/transport error.
/// If one succeeds and the other fails, logs a warning and returns the available rows.
pub async fn list_resource_quotas_and_limits(
    client: &Client,
    namespace: Option<String>,
) -> Result<Vec<PolicyInfo>, kube::Error> {
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

    let lp = ListParams::default();
    let (quota_res, limit_res) = join!(quota_api.list(&lp), limit_api.list(&lp));

    combine_resource_policy_results(
        quota_res.map(|list| list.items),
        limit_res.map(|list| list.items),
    )
}

pub async fn fetch_resource_quotas_and_limits(
    client: &Client,
    namespace: Option<String>,
) -> Result<Vec<PolicyInfo>, String> {
    match tokio::time::timeout(
        POLICIES_FETCH_TIMEOUT,
        list_resource_quotas_and_limits(client, namespace),
    )
    .await
    {
        Ok(Ok(list)) => Ok(list),
        Ok(Err(kube::Error::Api(api_err))) => Err(api_err.message),
        Ok(Err(err)) => Err(err.to_string()),
        Err(_) => Err("Timed out fetching resource quotas and limits after 10s".to_string()),
    }
}

pub async fn fetch_policies(
    client: &Client,
    namespace: Option<String>,
) -> Result<Vec<PolicyInfo>, String> {
    match tokio::time::timeout(POLICIES_FETCH_TIMEOUT, list_policies(client, namespace)).await {
        Ok(Ok(list)) => Ok(list),
        Ok(Err(kube::Error::Api(api_err))) => Err(api_err.message),
        Ok(Err(err)) => Err(err.to_string()),
        Err(_) => Err("Timed out fetching policies after 10s".to_string()),
    }
}

pub async fn list_policies(
    client: &Client,
    namespace: Option<String>,
) -> Result<Vec<PolicyInfo>, kube::Error> {
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

    let lp = ListParams::default();
    let is_cluster = namespace.is_none();

    let val_future = query_cluster_scoped_if_applicable(is_cluster, {
        let client_clone = client.clone();
        let lp_clone = lp.clone();
        move || async move {
            let val_api: Api<ValidatingWebhookConfiguration> = Api::all(client_clone);
            val_api.list(&lp_clone).await
        }
    });

    let mut_future = query_cluster_scoped_if_applicable(is_cluster, {
        let client_clone = client.clone();
        let lp_clone = lp.clone();
        move || async move {
            let mut_api: Api<MutatingWebhookConfiguration> = Api::all(client_clone);
            mut_api.list(&lp_clone).await
        }
    });

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
        val_future,
        mut_future,
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

    if let Some(res) = val_res {
        if let Ok(list) = res {
            for p in list {
                all_policies.push(format_val_webhook(&p));
            }
        } else if let Err(e) = res {
            tracing::warn!(error = %e, "Failed to list ValidatingWebhookConfigurations");
        }
    }

    if let Some(res) = mut_res {
        if let Ok(list) = res {
            for p in list {
                all_policies.push(format_mut_webhook(&p));
            }
        } else if let Err(e) = res {
            tracing::warn!(error = %e, "Failed to list MutatingWebhookConfigurations");
        }
    }

    Ok(all_policies)
}

pub fn format_network_policy(p: &NetworkPolicy) -> PolicyInfo {
    let name = p.metadata.name.clone().unwrap_or_default();
    let ns = p
        .metadata
        .namespace
        .clone()
        .unwrap_or_else(|| "-".to_string());
    let uid = p.metadata.uid.clone().unwrap_or_default();
    let age = format_age(&p.metadata.creation_timestamp);
    let rules = serde_yaml_ng::to_string(&p.spec).unwrap_or_default();

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
    let ns = p
        .metadata
        .namespace
        .clone()
        .unwrap_or_else(|| "-".to_string());
    let uid = p.metadata.uid.clone().unwrap_or_default();
    let age = format_age(&p.metadata.creation_timestamp);
    let hard = p
        .status
        .as_ref()
        .and_then(|s| s.hard.as_ref())
        .or_else(|| p.spec.as_ref().and_then(|s| s.hard.as_ref()));
    let used = p.status.as_ref().and_then(|s| s.used.as_ref());

    let quota_payload = serde_json::json!({
        "hard": hard,
        "used": used,
        "spec": &p.spec,
        "status": &p.status,
    });
    let rules = serde_yaml_ng::to_string(&quota_payload).unwrap_or_default();

    let mut violations = 0;
    let hard_count = hard.map(|h| h.len()).unwrap_or(0);
    let used_count = used.map(|u| u.len()).unwrap_or(0);

    if let (Some(hard_map), Some(used_map)) = (hard, used) {
        for (k, hard_q) in hard_map {
            if let Some(used_q) = used_map.get(k) {
                let hard_val = parse_quota_quantity(k, &hard_q.0);
                let used_val = parse_quota_quantity(k, &used_q.0);
                if hard_val > 0.0 && used_val > hard_val {
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
    let ns = p
        .metadata
        .namespace
        .clone()
        .unwrap_or_else(|| "-".to_string());
    let uid = p.metadata.uid.clone().unwrap_or_default();
    let age = format_age(&p.metadata.creation_timestamp);
    let rules = serde_yaml_ng::to_string(&p.spec).unwrap_or_default();

    let limits = p
        .spec
        .as_ref()
        .map(|s| s.limits.as_slice())
        .unwrap_or_default();
    let (status, mode, description) = if limits.is_empty() {
        (
            "Disabled".to_string(),
            "disabled".to_string(),
            "No limits defined".to_string(),
        )
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
    let rules = serde_yaml_ng::to_string(&p.webhooks).unwrap_or_default();

    let webhooks = p.webhooks.as_deref().unwrap_or_default();
    let count = webhooks.len();
    let (status, mode) = if count == 0 {
        ("Disabled".to_string(), "disabled".to_string())
    } else {
        let has_fail = webhooks.iter().any(|w| {
            w.failure_policy
                .as_deref()
                .map(|fp| fp.eq_ignore_ascii_case("Fail"))
                .unwrap_or(true)
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
    let rules = serde_yaml_ng::to_string(&p.webhooks).unwrap_or_default();

    let webhooks = p.webhooks.as_deref().unwrap_or_default();
    let count = webhooks.len();
    let (status, mode) = if count == 0 {
        ("Disabled".to_string(), "disabled".to_string())
    } else {
        let has_fail = webhooks.iter().any(|w| {
            w.failure_policy
                .as_deref()
                .map(|fp| fp.eq_ignore_ascii_case("Fail"))
                .unwrap_or(true)
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
    let rules = serde_yaml_ng::to_string(&psa_labels).unwrap_or_default();

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
    fn test_format_network_policy_defaults_when_spec_and_policy_types_missing() {
        use k8s_openapi::api::networking::v1::{NetworkPolicyEgressRule, NetworkPolicySpec};
        use k8s_openapi::apimachinery::pkg::apis::meta::v1::{
            LabelSelector, LabelSelectorRequirement,
        };

        // Missing namespace and spec
        let minimal = NetworkPolicy {
            metadata: ObjectMeta {
                name: Some("minimal-netpol".to_string()),
                namespace: None,
                ..Default::default()
            },
            spec: None,
        };
        let info = format_network_policy(&minimal);
        assert_eq!(info.namespace, "-");
        assert_eq!(info.scope, "Namespaced");
        assert_eq!(info.status, "Enforced");
        assert_eq!(info.mode, "Ingress");
        assert_eq!(info.description, "Kubernetes NetworkPolicy");

        // Spec present without policy_types and without egress -> Ingress + All pods in namespace
        let all_pods = NetworkPolicy {
            metadata: ObjectMeta {
                name: Some("default-deny".to_string()),
                namespace: Some("prod".to_string()),
                ..Default::default()
            },
            spec: Some(NetworkPolicySpec {
                policy_types: None,
                egress: None,
                pod_selector: LabelSelector::default(),
                ..Default::default()
            }),
        };
        let info_all = format_network_policy(&all_pods);
        assert_eq!(info_all.mode, "Ingress");
        assert_eq!(info_all.description, "All pods in namespace");

        // Spec with egress rules and match_expressions -> Ingress, Egress + expression count
        let with_egress = NetworkPolicy {
            metadata: ObjectMeta {
                name: Some("egress-expr".to_string()),
                namespace: Some("prod".to_string()),
                ..Default::default()
            },
            spec: Some(NetworkPolicySpec {
                policy_types: None,
                egress: Some(vec![NetworkPolicyEgressRule::default()]),
                pod_selector: LabelSelector {
                    match_labels: None,
                    match_expressions: Some(vec![
                        LabelSelectorRequirement {
                            key: "tier".to_string(),
                            operator: "In".to_string(),
                            values: Some(vec!["backend".to_string()]),
                        },
                        LabelSelectorRequirement {
                            key: "env".to_string(),
                            operator: "Exists".to_string(),
                            values: None,
                        },
                    ]),
                },
                ..Default::default()
            }),
        };
        let info_egress = format_network_policy(&with_egress);
        assert_eq!(info_egress.mode, "Ingress, Egress");
        assert_eq!(info_egress.description, "Pods matching 2 expression(s)");
    }

    #[tokio::test]
    async fn test_fetch_network_policies_propagates_connection_error() {
        let Ok(cluster_url) = "http://127.0.0.1:1".parse::<http::Uri>() else {
            panic!("static URI should parse");
        };
        let config = kube::Config::new(cluster_url);
        let Ok(client) = Client::try_from(config) else {
            panic!("client should build from static config");
        };

        let result = fetch_network_policies(&client, None).await;
        match result {
            Ok(_) => panic!("expected connection error against closed local port"),
            Err(message) => assert!(!message.trim().is_empty()),
        }
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
            webhooks: Some(vec![ValidatingWebhook {
                failure_policy: Some("Ignore".to_string()),
                name: "hook1.example.com".to_string(),
                ..Default::default()
            }]),
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
            webhooks: Some(vec![ValidatingWebhook {
                failure_policy: Some("Fail".to_string()),
                name: "hook2.example.com".to_string(),
                ..Default::default()
            }]),
        };
        let info = format_val_webhook(&p);
        assert_eq!(info.status, "Enforced");
        assert_eq!(info.mode, "Fail");
        assert_eq!(info.description, "1 webhook(s)");
    }

    #[tokio::test]
    async fn test_query_cluster_scoped_if_applicable_skips_when_namespaced() {
        let called = std::sync::atomic::AtomicBool::new(false);
        let result = query_cluster_scoped_if_applicable(false, || async {
            called.store(true, std::sync::atomic::Ordering::SeqCst);
            "webhooks"
        })
        .await;

        assert_eq!(result, None);
        assert!(
            !called.load(std::sync::atomic::Ordering::SeqCst),
            "fetch closure should not be executed when not cluster-scoped"
        );

        let result_cluster = query_cluster_scoped_if_applicable(true, || async {
            called.store(true, std::sync::atomic::Ordering::SeqCst);
            "webhooks"
        })
        .await;

        assert_eq!(result_cluster, Some("webhooks"));
        assert!(called.load(std::sync::atomic::Ordering::SeqCst));
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
        used.insert("cpu".to_string(), Quantity("5".to_string()));
        used.insert("memory".to_string(), Quantity("4Gi".to_string()));
        used.insert("pods".to_string(), Quantity("10".to_string()));

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
        assert_eq!(info.violations, 1);
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
            spec: Some(LimitRangeSpec { limits: vec![] }),
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
        labels.insert(
            "pod-security.kubernetes.io/enforce".to_string(),
            "restricted".to_string(),
        );
        labels.insert(
            "pod-security.kubernetes.io/audit".to_string(),
            "baseline".to_string(),
        );

        let ns = Namespace {
            metadata: ObjectMeta {
                name: Some("secure-ns".to_string()),
                labels: Some(labels),
                ..Default::default()
            },
            ..Default::default()
        };

        let Some(info) = format_pod_security_namespace(&ns) else {
            panic!("should format PSA info");
        };
        assert_eq!(info.r#type, "Pod Security");
        assert_eq!(info.kind, "Namespace");
        assert_eq!(info.status, "Enforced");
        assert_eq!(info.mode, "enforce: restricted");
        assert_eq!(info.name, "psa-secure-ns");
        assert_eq!(info.namespace, "secure-ns");

        let empty_ns = Namespace {
            metadata: ObjectMeta {
                name: Some("plain-ns".to_string()),
                ..Default::default()
            },
            ..Default::default()
        };
        assert!(format_pod_security_namespace(&empty_ns).is_none());
    }

    #[test]
    fn test_format_resource_quota_preserves_used_and_hard_in_rules() {
        use k8s_openapi::api::core::v1::{ResourceQuotaSpec, ResourceQuotaStatus};
        use k8s_openapi::apimachinery::pkg::api::resource::Quantity;
        use std::collections::BTreeMap;

        let mut hard = BTreeMap::new();
        hard.insert("cpu".to_string(), Quantity("4".to_string()));
        hard.insert("memory".to_string(), Quantity("8Gi".to_string()));

        let mut used = BTreeMap::new();
        used.insert("cpu".to_string(), Quantity("1".to_string()));
        used.insert("memory".to_string(), Quantity("2Gi".to_string()));

        let p = ResourceQuota {
            metadata: ObjectMeta {
                name: Some("compute-quota".to_string()),
                namespace: Some("prod".to_string()),
                ..Default::default()
            },
            spec: Some(ResourceQuotaSpec {
                hard: Some(hard.clone()),
                ..Default::default()
            }),
            status: Some(ResourceQuotaStatus {
                hard: Some(hard),
                used: Some(used),
            }),
        };

        let info = format_resource_quota(&p);
        assert!(info.rules.contains("hard:"), "rules should contain hard:");
        assert!(info.rules.contains("used:"), "rules should contain used:");
        assert!(info.rules.contains('4'), "rules should contain 4");
        assert!(info.rules.contains("2Gi"), "rules should contain 2Gi");
    }

    #[test]
    fn test_policy_formatters_serialize_rules_to_yaml() {
        use k8s_openapi::api::admissionregistration::v1::{MutatingWebhook, ValidatingWebhook};
        use k8s_openapi::api::core::v1::{LimitRangeItem, LimitRangeSpec};
        use k8s_openapi::api::networking::v1::NetworkPolicySpec;
        use std::collections::BTreeMap;

        let netpol = NetworkPolicy {
            metadata: ObjectMeta {
                name: Some("yaml-netpol".to_string()),
                ..Default::default()
            },
            spec: Some(NetworkPolicySpec {
                policy_types: Some(vec!["Ingress".to_string()]),
                ..Default::default()
            }),
        };
        assert!(format_network_policy(&netpol)
            .rules
            .contains("policyTypes:"));

        let limit_range = LimitRange {
            metadata: ObjectMeta {
                name: Some("yaml-limit".to_string()),
                ..Default::default()
            },
            spec: Some(LimitRangeSpec {
                limits: vec![LimitRangeItem {
                    type_: "Container".to_string(),
                    ..Default::default()
                }],
            }),
        };
        assert!(format_limit_range(&limit_range).rules.contains("Container"));

        let val_wh = ValidatingWebhookConfiguration {
            metadata: ObjectMeta {
                name: Some("yaml-val".to_string()),
                ..Default::default()
            },
            webhooks: Some(vec![ValidatingWebhook {
                name: "val.example.com".to_string(),
                ..Default::default()
            }]),
        };
        assert!(format_val_webhook(&val_wh)
            .rules
            .contains("val.example.com"));

        let mut_wh = MutatingWebhookConfiguration {
            metadata: ObjectMeta {
                name: Some("yaml-mut".to_string()),
                ..Default::default()
            },
            webhooks: Some(vec![MutatingWebhook {
                name: "mut.example.com".to_string(),
                ..Default::default()
            }]),
        };
        assert!(format_mut_webhook(&mut_wh)
            .rules
            .contains("mut.example.com"));

        let mut labels = BTreeMap::new();
        labels.insert(
            "pod-security.kubernetes.io/enforce".to_string(),
            "restricted".to_string(),
        );
        let ns = Namespace {
            metadata: ObjectMeta {
                name: Some("yaml-psa".to_string()),
                labels: Some(labels),
                ..Default::default()
            },
            ..Default::default()
        };
        let Some(psa) = format_pod_security_namespace(&ns) else {
            panic!("should format PSA policy");
        };
        assert!(psa
            .rules
            .contains("pod-security.kubernetes.io/enforce: restricted"));
    }

    #[test]
    fn test_combine_resource_policy_results_merges_quotas_and_limits_and_surfaces_dual_error() {
        let quota = ResourceQuota {
            metadata: ObjectMeta {
                name: Some("compute-quota".to_string()),
                namespace: Some("prod".to_string()),
                ..Default::default()
            },
            ..Default::default()
        };
        let limit = LimitRange {
            metadata: ObjectMeta {
                name: Some("default-limits".to_string()),
                namespace: Some("prod".to_string()),
                ..Default::default()
            },
            ..Default::default()
        };

        // Both Ok -> merges into 2 PolicyInfo rows
        let combined = combine_resource_policy_results::<&str>(
            Ok(vec![quota.clone()]),
            Ok(vec![limit.clone()]),
        );
        let Ok(rows) = combined else {
            panic!("expected Ok when both succeed");
        };
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0].kind, "ResourceQuota");
        assert_eq!(rows[1].kind, "LimitRange");

        // Partial error (quota Ok, limit Err) -> preserves quota row
        let partial = combine_resource_policy_results::<&str>(Ok(vec![quota]), Err("limit denied"));
        let Ok(partial_rows) = partial else {
            panic!("expected Ok when one source succeeds");
        };
        assert_eq!(partial_rows.len(), 1);
        assert_eq!(partial_rows[0].kind, "ResourceQuota");

        // Both Err -> returns Err
        let dual_err =
            combine_resource_policy_results::<&str>(Err("quota denied"), Err("limit denied"));
        match dual_err {
            Ok(_) => panic!("expected Err when both sources fail"),
            Err(err) => assert_eq!(err, "quota denied"),
        }
    }

    #[tokio::test]
    async fn test_fetch_pod_security_policies_propagates_connection_error() {
        let Ok(cluster_url) = "http://127.0.0.1:1".parse::<http::Uri>() else {
            panic!("static URI should parse");
        };
        let config = kube::Config::new(cluster_url);
        let Ok(client) = Client::try_from(config) else {
            panic!("client should build from static config");
        };

        let result = fetch_pod_security_policies(&client, None).await;
        match result {
            Ok(_) => panic!("expected connection error against closed local port"),
            Err(message) => assert!(!message.trim().is_empty()),
        }
    }

    #[tokio::test]
    async fn test_fetch_resource_quotas_and_limits_propagates_connection_error() {
        let Ok(cluster_url) = "http://127.0.0.1:1".parse::<http::Uri>() else {
            panic!("static URI should parse");
        };
        let config = kube::Config::new(cluster_url);
        let Ok(client) = Client::try_from(config) else {
            panic!("client should build from static config");
        };

        let result = fetch_resource_quotas_and_limits(&client, None).await;
        match result {
            Ok(_) => panic!("expected connection error against closed local port"),
            Err(message) => assert!(!message.trim().is_empty()),
        }
    }
}
