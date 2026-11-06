use kube::{
    api::{Api, ApiResource, DynamicObject, GroupVersionKind, ListParams, Patch, PatchParams},
    Client,
};
use k8s_openapi::api::core::v1::{Node, Pod};
use crate::kubernetes::{models, format_age};

/// Maps a raw Kubernetes `Node` into a `models::NodeInfo` representation.
pub fn map_node(node: &Node) -> models::NodeInfo {
    let name = node.metadata.name.clone().unwrap_or_default();

    let status = if let Some(status_ref) = &node.status {
        if let Some(conditions) = &status_ref.conditions {
            conditions.iter()
                .find(|c| c.type_ == "Ready")
                .map(|c| if c.status == "True" { "Ready".to_string() } else { "NotReady".to_string() })
                .unwrap_or_else(|| "Unknown".to_string())
        } else {
            "Unknown".to_string()
        }
    } else {
        "Unknown".to_string()
    };

    let mut role = "worker".to_string();
    let has_control_plane_role = node.metadata.labels.as_ref().is_some_and(|labels_map| {
        labels_map.contains_key("node-role.kubernetes.io/control-plane")
            || labels_map.contains_key("node-role.kubernetes.io/master")
    });
    if has_control_plane_role {
        role = "control-plane".to_string();
    }

    let version = node.status.as_ref()
        .and_then(|s| s.node_info.as_ref())
        .map(|ni| ni.kubelet_version.clone())
        .unwrap_or_else(|| "Unknown".to_string());

    let uptime = format_age(&node.metadata.creation_timestamp);
    let created_at = node.metadata.creation_timestamp.as_ref().map(|t| t.0.to_rfc3339());

    let mut labels = Vec::new();
    if let Some(map) = &node.metadata.labels {
        for (k, v) in map {
            labels.push(format!("{}={}", k, v));
        }
    }
    labels.sort();
    let labels_map = node.metadata.labels.clone().unwrap_or_default();
    let annotations = node.metadata.annotations.clone().unwrap_or_default();

    let pods_limit = node.status.as_ref()
        .and_then(|s| s.capacity.as_ref())
        .and_then(|c| c.get("pods"))
        .and_then(|q| q.0.parse::<i32>().ok())
        .unwrap_or(110);

    let cpu_total_str = node.status.as_ref()
        .and_then(|s| s.capacity.as_ref())
        .and_then(|c| c.get("cpu"))
        .map(|q| q.0.clone())
        .unwrap_or_else(|| "0".to_string());
    let cpu_total_val = parse_cpu_quantity(&cpu_total_str);

    let mem_total_str = node.status.as_ref()
        .and_then(|s| s.capacity.as_ref())
        .and_then(|c| c.get("memory"))
        .map(|q| q.0.clone())
        .unwrap_or_else(|| "0".to_string());
    let mem_total_val = parse_memory_quantity(&mem_total_str);

    let is_cordoned = node.spec.as_ref().and_then(|s| s.unschedulable).unwrap_or(false);

    let system_info = node.status.as_ref()
        .and_then(|s| s.node_info.as_ref())
        .cloned();

    let addresses = node.status.as_ref()
        .and_then(|s| s.addresses.as_ref())
        .cloned()
        .unwrap_or_default();

    let conditions = node.status.as_ref()
        .and_then(|s| s.conditions.as_ref())
        .cloned()
        .unwrap_or_default();

    let taints = node.spec.as_ref()
        .and_then(|s| s.taints.as_ref())
        .cloned()
        .unwrap_or_default();

    let capacity = node.status.as_ref()
        .and_then(|s| s.capacity.as_ref())
        .map(|m| models::NodeResources {
            cpu: m.get("cpu").map(|q| q.0.clone()).unwrap_or_else(|| "-".to_string()),
            memory: m.get("memory").map(|q| q.0.clone()).unwrap_or_else(|| "-".to_string()),
            pods: m.get("pods").map(|q| q.0.clone()).unwrap_or_else(|| "-".to_string()),
            ephemeral_storage: m.get("ephemeral-storage").map(|q| q.0.clone()),
        });

    let allocatable = node.status.as_ref()
        .and_then(|s| s.allocatable.as_ref())
        .map(|m| models::NodeResources {
            cpu: m.get("cpu").map(|q| q.0.clone()).unwrap_or_else(|| "-".to_string()),
            memory: m.get("memory").map(|q| q.0.clone()).unwrap_or_else(|| "-".to_string()),
            pods: m.get("pods").map(|q| q.0.clone()).unwrap_or_else(|| "-".to_string()),
            ephemeral_storage: m.get("ephemeral-storage").map(|q| q.0.clone()),
        });

    let images_count = node.status.as_ref()
        .and_then(|s| s.images.as_ref())
        .map(|imgs| imgs.len())
        .unwrap_or(0);

    models::NodeInfo {
        name,
        status,
        role,
        version,
        cpu_pct: 0.0,
        cpu_used: "0".to_string(),
        cpu_total: format!("{:.0}", cpu_total_val),
        mem_pct: 0.0,
        mem_used: "0".to_string(),
        mem_total: format!("{:.0}", mem_total_val),
        pods_count: 0,
        pods_limit,
        uptime,
        created_at,
        labels,
        labels_map,
        annotations,
        is_cordoned,
        node_info: system_info,
        addresses,
        conditions,
        taints,
        capacity,
        allocatable,
        images_count,
    }
}

/// Builds the merge patch JSON for changing a node's schedulable (cordoned) status.
pub fn build_node_schedulable_patch(unschedulable: bool) -> serde_json::Value {
    serde_json::json!({
        "spec": {
            "unschedulable": unschedulable
        }
    })
}

/// Updates the `unschedulable` field of a Kubernetes node via Merge Patch.
pub async fn set_node_schedulable(client: &Client, name: &str, unschedulable: bool) -> Result<(), kube::Error> {
    let nodes_api: Api<Node> = Api::all(client.clone());
    let patch = build_node_schedulable_patch(unschedulable);
    nodes_api
        .patch(name, &PatchParams::default(), &Patch::Merge(&patch))
        .await?;
    Ok(())
}

/// Determines whether a Pod should be evicted during a Node drain operation.
/// Excludes DaemonSet-managed pods, mirror pods, and pods in Succeeded/Failed phases.
pub fn is_pod_evictable(pod: &Pod) -> bool {
    // 1. Exclude terminal pods (Succeeded or Failed)
    if let Some(status) = &pod.status {
        if let Some(phase) = &status.phase {
            if phase == "Succeeded" || phase == "Failed" {
                return false;
            }
        }
    }

    // 2. Exclude mirror pods with kubernetes.io/config.mirror annotation
    if let Some(annotations) = &pod.metadata.annotations {
        if annotations.contains_key("kubernetes.io/config.mirror") {
            return false;
        }
    }

    // 3. Exclude DaemonSet-owned pods
    if let Some(owners) = &pod.metadata.owner_references {
        for owner in owners {
            if owner.kind == "DaemonSet" {
                return false;
            }
        }
    }

    true
}

/// Cordons the target node and evicts all eligible scheduled pods.
pub async fn drain_node(client: &Client, name: &str) -> Result<usize, kube::Error> {
    // Step 1: Cordon node first to prevent new pods from scheduling
    set_node_schedulable(client, name, true).await?;

    // Step 2: List all pods currently bound to this node
    let pods_api: Api<Pod> = Api::all(client.clone());
    let list_params = ListParams::default().fields(&format!("spec.nodeName={}", name));
    let pods = pods_api.list(&list_params).await?;

    let mut evicted_count = 0;
    for pod in pods.items {
        if !is_pod_evictable(&pod) {
            continue;
        }

        let pod_name = pod.metadata.name.as_deref().unwrap_or_default();
        let namespace = pod.metadata.namespace.as_deref().unwrap_or("default");
        if pod_name.is_empty() {
            continue;
        }

        let namespaced_pods: Api<Pod> = Api::namespaced(client.clone(), namespace);
        let eviction_payload = serde_json::json!({
            "apiVersion": "policy/v1",
            "kind": "Eviction",
            "metadata": {
                "name": pod_name,
                "namespace": namespace
            }
        });

        let eviction_bytes = match serde_json::to_vec(&eviction_payload) {
            Ok(b) => b,
            Err(e) => {
                tracing::warn!(error = %e, "Failed to serialize eviction payload");
                continue;
            }
        };

        match namespaced_pods.create_subresource::<serde_json::Value>("eviction", pod_name, &kube::api::PostParams::default(), eviction_bytes).await {
            Ok(_) => {
                evicted_count += 1;
            }
            Err(e) => {
                tracing::warn!(pod = %pod_name, namespace = %namespace, error = %e, "Failed to evict pod during node drain");
            }
        }
    }

    Ok(evicted_count)
}

pub async fn list_nodes(client: &Client) -> Result<Vec<models::NodeInfo>, kube::Error> {
    let nodes_api: Api<Node> = Api::all(client.clone());
    let pods_api: Api<Pod> = Api::all(client.clone());

    let nodes = nodes_api.list(&ListParams::default()).await?;
    let pods = match pods_api.list(&ListParams::default()).await {
        Ok(list) => list.items,
        Err(e) => {
            tracing::warn!(error = %e, "Could not list pods for node metrics calculation (RBAC?)");
            Vec::new()
        }
    };

    let gvk = GroupVersionKind::gvk("metrics.k8s.io", "v1beta1", "NodeMetrics");
    let ar = ApiResource::from_gvk(&gvk);
    let metrics_api: Api<DynamicObject> = Api::all_with(client.clone(), &ar);
    let node_metrics = match metrics_api.list(&ListParams::default()).await {
        Ok(list) => Some(list.items),
        Err(e) => {
            tracing::warn!(error = %e, "Could not fetch node metrics (Metrics Server installed?)");
            None
        }
    };

    let mut pods_by_node: std::collections::HashMap<String, Vec<Pod>> = std::collections::HashMap::new();
    for pod in pods {
        if let Some(node_name) = pod.spec.as_ref().and_then(|spec| spec.node_name.as_ref()) {
            pods_by_node.entry(node_name.clone()).or_default().push(pod);
        }
    }

    let mut list = Vec::new();
    for node in nodes {
        let name = node.metadata.name.clone().unwrap_or_default();
        let mut info = map_node(&node);

        let node_pods = pods_by_node.get(&name).cloned().unwrap_or_default();
        let pods_count = node_pods.len() as i32;

        let cpu_total_str = node.status.as_ref()
            .and_then(|s| s.capacity.as_ref())
            .and_then(|c| c.get("cpu"))
            .map(|q| q.0.clone())
            .unwrap_or_else(|| "0".to_string());
        let cpu_total_val = parse_cpu_quantity(&cpu_total_str);

        let mem_total_str = node.status.as_ref()
            .and_then(|s| s.capacity.as_ref())
            .and_then(|c| c.get("memory"))
            .map(|q| q.0.clone())
            .unwrap_or_else(|| "0".to_string());
        let mem_total_val = parse_memory_quantity(&mem_total_str);

        let mut actual_cpu_cores = 0.0;
        let mut actual_mem_gib = 0.0;
        let mut has_metrics = false;

        // Usage calculation precedence:
        // 1. Real NodeMetrics from Metrics Server (metrics.k8s.io/v1beta1 schema: { "usage": { "cpu": "...", "memory": "..." } })
        // 2. Sum of Pod resource requests (fallback when Metrics Server is not installed or returns an error)
        // 3. 0.0 default when no metrics or pods exist for this node
        if let Some(usage) = node_metrics.as_ref().and_then(|list| {
            list.iter()
                .find(|m| m.metadata.name.as_deref() == Some(&name))
                .and_then(|metric| metric.data.get("usage"))
        }) {
            has_metrics = true;
            if let Some(cpu) = usage.get("cpu").and_then(|v| v.as_str()) {
                actual_cpu_cores = parse_cpu_quantity(cpu);
            }
            if let Some(mem) = usage.get("memory").and_then(|v| v.as_str()) {
                actual_mem_gib = parse_memory_quantity(mem);
            }
        }

        if !has_metrics {
            for pod in &node_pods {
                if let Some(spec) = &pod.spec {
                    for container in &spec.containers {
                        if let Some(requests) = container.resources.as_ref().and_then(|r| r.requests.as_ref()) {
                            if let Some(cpu) = requests.get("cpu") {
                                actual_cpu_cores += parse_cpu_quantity(&cpu.0);
                            }
                            if let Some(mem) = requests.get("memory") {
                                actual_mem_gib += parse_memory_quantity(&mem.0);
                            }
                        }
                    }
                }
            }
        }

        let cpu_pct = if cpu_total_val > 0.0 { ((actual_cpu_cores / cpu_total_val) * 1000.0).round() / 10.0 } else { 0.0 };
        let mem_pct = if mem_total_val > 0.0 { ((actual_mem_gib / mem_total_val) * 1000.0).round() / 10.0 } else { 0.0 };

        info.cpu_pct = cpu_pct;
        info.cpu_used = crate::kubernetes::metrics::format_decimal(actual_cpu_cores, 2);
        info.mem_pct = mem_pct;
        info.mem_used = crate::kubernetes::metrics::format_decimal(actual_mem_gib, 2);
        info.pods_count = pods_count;

        list.push(info);
    }

    Ok(list)
}

/// Parses a Kubernetes CPU quantity string (e.g. "2", "250m", "500u", "3141592n") into fractional CPU cores.
/// Returns `0.0` for empty, unrecognized, or invalid quantity formats.
pub(crate) fn parse_cpu_quantity(q: &str) -> f64 {
    let q = q.trim();
    if let Some(stripped) = q.strip_suffix('n') {
        stripped.parse::<f64>().unwrap_or(0.0) / 1_000_000_000.0
    } else if let Some(stripped) = q.strip_suffix('u') {
        stripped.parse::<f64>().unwrap_or(0.0) / 1_000_000.0
    } else if let Some(stripped) = q.strip_suffix('m') {
        stripped.parse::<f64>().unwrap_or(0.0) / 1000.0
    } else {
        q.parse::<f64>().unwrap_or(0.0)
    }
}

pub(crate) fn parse_memory_quantity(q: &str) -> f64 {
    let q = q.trim();
    if q.is_empty() {
        return 0.0;
    }

    if let Some(stripped) = q.strip_suffix("Ki") {
        let val = stripped.parse::<f64>().unwrap_or(0.0);
        val * 1024.0 / (1024.0 * 1024.0 * 1024.0)
    } else if let Some(stripped) = q.strip_suffix("Mi") {
        let val = stripped.parse::<f64>().unwrap_or(0.0);
        val * 1024.0 * 1024.0 / (1024.0 * 1024.0 * 1024.0)
    } else if let Some(stripped) = q.strip_suffix("Gi") {
        stripped.parse::<f64>().unwrap_or(0.0)
    } else if let Some(stripped) = q.strip_suffix("Ti") {
        let val = stripped.parse::<f64>().unwrap_or(0.0);
        val * 1024.0
    } else if let Some(stripped) = q.strip_suffix("Pi") {
        let val = stripped.parse::<f64>().unwrap_or(0.0);
        val * 1024.0 * 1024.0
    } else if let Some(stripped) = q.strip_suffix("Ei") {
        let val = stripped.parse::<f64>().unwrap_or(0.0);
        val * 1024.0 * 1024.0 * 1024.0
    } else if let Some(stripped) = q.strip_suffix('k') {
        let val = stripped.parse::<f64>().unwrap_or(0.0);
        val * 1000.0 / (1024.0 * 1024.0 * 1024.0)
    } else if let Some(stripped) = q.strip_suffix('M') {
        let val = stripped.parse::<f64>().unwrap_or(0.0);
        val * 1000.0 * 1000.0 / (1024.0 * 1024.0 * 1024.0)
    } else if let Some(stripped) = q.strip_suffix('G') {
        let val = stripped.parse::<f64>().unwrap_or(0.0);
        val * 1000.0 * 1000.0 * 1000.0 / (1024.0 * 1024.0 * 1024.0)
    } else if let Some(stripped) = q.strip_suffix('T') {
        let val = stripped.parse::<f64>().unwrap_or(0.0);
        val * 1000.0 * 1000.0 * 1000.0 * 1000.0 / (1024.0 * 1024.0 * 1024.0)
    } else if let Some(stripped) = q.strip_suffix('P') {
        let val = stripped.parse::<f64>().unwrap_or(0.0);
        val * 1000.0 * 1000.0 * 1000.0 * 1000.0 * 1000.0 / (1024.0 * 1024.0 * 1024.0)
    } else if let Some(stripped) = q.strip_suffix('E') {
        let val = stripped.parse::<f64>().unwrap_or(0.0);
        val * 1000.0 * 1000.0 * 1000.0 * 1000.0 * 1000.0 * 1000.0 / (1024.0 * 1024.0 * 1024.0)
    } else {
        let val = q.parse::<f64>().unwrap_or(0.0);
        if q.parse::<f64>().is_err() {
            tracing::warn!(quantity = %q, "parse_memory_quantity: unrecognised suffix, treating as bytes");
        }
        val / (1024.0 * 1024.0 * 1024.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_cpu_quantity() {
        assert_eq!(parse_cpu_quantity("2"), 2.0);
        assert_eq!(parse_cpu_quantity("250m"), 0.25);
        assert_eq!(parse_cpu_quantity("500u"), 0.0005);
        assert_eq!(parse_cpu_quantity("3141592n"), 0.003141592);
        assert_eq!(parse_cpu_quantity("0"), 0.0);
        assert_eq!(parse_cpu_quantity(""), 0.0);
        assert_eq!(parse_cpu_quantity("invalid"), 0.0);
    }

    #[test]
    fn test_parse_memory_quantity() {
        assert_eq!(parse_memory_quantity("1Gi"), 1.0);
        assert_eq!(parse_memory_quantity("1024Mi"), 1.0);
        assert_eq!(parse_memory_quantity("1048576Ki"), 1.0);
        assert_eq!(parse_memory_quantity(""), 0.0);
    }

    #[test]
    fn test_map_node_extracts_status_role_and_cordoned_state() {
        use k8s_openapi::apimachinery::pkg::apis::meta::v1::ObjectMeta;
        use k8s_openapi::api::core::v1::{NodeCondition, NodeSpec, NodeStatus};
        use std::collections::BTreeMap;

        let mut labels = BTreeMap::new();
        labels.insert("node-role.kubernetes.io/control-plane".to_string(), "".to_string());

        let node1 = Node {
            metadata: ObjectMeta {
                name: Some("node-1".to_string()),
                labels: Some(labels),
                ..Default::default()
            },
            spec: Some(NodeSpec {
                unschedulable: Some(true),
                ..Default::default()
            }),
            status: Some(NodeStatus {
                conditions: Some(vec![NodeCondition {
                    type_: "Ready".to_string(),
                    status: "True".to_string(),
                    ..Default::default()
                }]),
                ..Default::default()
            }),
        };

        let mapped1 = map_node(&node1);
        assert_eq!(mapped1.name, "node-1");
        assert_eq!(mapped1.status, "Ready");
        assert_eq!(mapped1.role, "control-plane");
        assert!(mapped1.is_cordoned);

        let node2 = Node {
            metadata: ObjectMeta {
                name: Some("node-2".to_string()),
                ..Default::default()
            },
            spec: Some(NodeSpec {
                unschedulable: None,
                ..Default::default()
            }),
            status: Some(NodeStatus {
                conditions: Some(vec![NodeCondition {
                    type_: "Ready".to_string(),
                    status: "False".to_string(),
                    ..Default::default()
                }]),
                ..Default::default()
            }),
        };

        let mapped2 = map_node(&node2);
        assert_eq!(mapped2.name, "node-2");
        assert_eq!(mapped2.status, "NotReady");
        assert_eq!(mapped2.role, "worker");
        assert!(!mapped2.is_cordoned);
    }

    #[test]
    fn test_build_node_schedulable_patch() {
        let patch_cordon = build_node_schedulable_patch(true);
        assert_eq!(patch_cordon, serde_json::json!({ "spec": { "unschedulable": true } }));

        let patch_uncordon = build_node_schedulable_patch(false);
        assert_eq!(patch_uncordon, serde_json::json!({ "spec": { "unschedulable": false } }));
    }

    #[test]
    fn test_is_pod_evictable_excludes_daemonset_mirror_and_terminal_pods() {
        use k8s_openapi::apimachinery::pkg::apis::meta::v1::{ObjectMeta, OwnerReference};
        use k8s_openapi::api::core::v1::{Pod, PodStatus};
        use std::collections::BTreeMap;

        // Normal running pod is evictable
        let normal_pod = Pod {
            metadata: ObjectMeta {
                name: Some("normal-pod".to_string()),
                namespace: Some("default".to_string()),
                ..Default::default()
            },
            status: Some(PodStatus {
                phase: Some("Running".to_string()),
                ..Default::default()
            }),
            ..Default::default()
        };
        assert!(is_pod_evictable(&normal_pod));

        // DaemonSet owned pod is NOT evictable
        let daemonset_pod = Pod {
            metadata: ObjectMeta {
                name: Some("ds-pod".to_string()),
                namespace: Some("kube-system".to_string()),
                owner_references: Some(vec![OwnerReference {
                    api_version: "apps/v1".to_string(),
                    kind: "DaemonSet".to_string(),
                    name: "kube-proxy".to_string(),
                    uid: "123".to_string(),
                    ..Default::default()
                }]),
                ..Default::default()
            },
            status: Some(PodStatus {
                phase: Some("Running".to_string()),
                ..Default::default()
            }),
            ..Default::default()
        };
        assert!(!is_pod_evictable(&daemonset_pod));

        // Mirror pod (kubernetes.io/config.mirror) is NOT evictable
        let mut annotations = BTreeMap::new();
        annotations.insert("kubernetes.io/config.mirror".to_string(), "mirror-hash".to_string());
        let mirror_pod = Pod {
            metadata: ObjectMeta {
                name: Some("mirror-pod".to_string()),
                namespace: Some("kube-system".to_string()),
                annotations: Some(annotations),
                ..Default::default()
            },
            status: Some(PodStatus {
                phase: Some("Running".to_string()),
                ..Default::default()
            }),
            ..Default::default()
        };
        assert!(!is_pod_evictable(&mirror_pod));

        // Succeeded pod is NOT evictable
        let succeeded_pod = Pod {
            metadata: ObjectMeta {
                name: Some("succeeded-pod".to_string()),
                namespace: Some("default".to_string()),
                ..Default::default()
            },
            status: Some(PodStatus {
                phase: Some("Succeeded".to_string()),
                ..Default::default()
            }),
            ..Default::default()
        };
        assert!(!is_pod_evictable(&succeeded_pod));

        // Failed pod is NOT evictable
        let failed_pod = Pod {
            metadata: ObjectMeta {
                name: Some("failed-pod".to_string()),
                namespace: Some("default".to_string()),
                ..Default::default()
            },
            status: Some(PodStatus {
                phase: Some("Failed".to_string()),
                ..Default::default()
            }),
            ..Default::default()
        };
        assert!(!is_pod_evictable(&failed_pod));
    }
}
