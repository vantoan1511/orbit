use std::time::Duration;

use k8s_openapi::api::core::v1::{Node, Pod};
use kube::{
    api::{Api, ApiResource, DynamicObject, GroupVersionKind, ListParams},
    Client,
};

use crate::kubernetes::{models, namespaces::format_age};

pub const NODE_FETCH_TIMEOUT: Duration = Duration::from_secs(10);

/// Formats a float up to `max_decimals` places, stripping unnecessary trailing zeros and decimal point.
pub fn format_decimal(val: f64, max_decimals: usize) -> String {
    if !val.is_finite() || val.abs() == 0.0 {
        return "0".to_string();
    }
    let formatted = format!("{:.prec$}", val, prec = max_decimals);
    if formatted.contains('.') {
        let trimmed = formatted.trim_end_matches('0').trim_end_matches('.');
        if trimmed == "-0" || trimmed.is_empty() {
            "0".to_string()
        } else {
            trimmed.to_string()
        }
    } else if formatted == "-0" {
        "0".to_string()
    } else {
        formatted
    }
}

/// Parses a Kubernetes CPU quantity string (e.g. "2", "250m", "500u", "3141592n") into fractional CPU cores.
/// Returns `0.0` for empty, unrecognized, or invalid quantity formats.
pub fn parse_cpu_quantity(q: &str) -> f64 {
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

/// Parses a Kubernetes memory quantity string (e.g. "1Gi", "1024Mi", "1048576Ki") into GiB.
pub fn parse_memory_quantity(q: &str) -> f64 {
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

fn map_node_resources(
    resources: Option<&std::collections::BTreeMap<String, k8s_openapi::apimachinery::pkg::api::resource::Quantity>>,
) -> Option<models::NodeResources> {
    resources.map(|m| models::NodeResources {
        cpu: m.get("cpu").map(|q| q.0.clone()).unwrap_or_else(|| "-".to_string()),
        memory: m.get("memory").map(|q| q.0.clone()).unwrap_or_else(|| "-".to_string()),
        pods: m.get("pods").map(|q| q.0.clone()).unwrap_or_else(|| "-".to_string()),
        ephemeral_storage: m.get("ephemeral-storage").map(|q| q.0.clone()),
    })
}

/// Maps a raw Kubernetes `Node` into a `models::NodeInfo` representation.
pub fn map_node(node: &Node) -> models::NodeInfo {
    let name = node.metadata.name.clone().unwrap_or_default();

    let status = if let Some(status_ref) = &node.status {
        if let Some(conditions) = &status_ref.conditions {
            conditions
                .iter()
                .find(|c| c.type_ == "Ready")
                .map(|c| {
                    if c.status == "True" {
                        "Ready".to_string()
                    } else {
                        "NotReady".to_string()
                    }
                })
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

    let version = node
        .status
        .as_ref()
        .and_then(|s| s.node_info.as_ref())
        .map(|ni| ni.kubelet_version.clone())
        .unwrap_or_else(|| "Unknown".to_string());

    let uptime = format_age(&node.metadata.creation_timestamp);
    let created_at = node
        .metadata
        .creation_timestamp
        .as_ref()
        .map(|t| t.0.to_rfc3339());

    let mut labels = Vec::new();
    if let Some(map) = &node.metadata.labels {
        for (k, v) in map {
            labels.push(format!("{}={}", k, v));
        }
    }
    labels.sort();
    let labels_map = node.metadata.labels.clone().unwrap_or_default();
    let annotations = node.metadata.annotations.clone().unwrap_or_default();

    let pods_limit = node
        .status
        .as_ref()
        .and_then(|s| s.capacity.as_ref())
        .and_then(|c| c.get("pods"))
        .and_then(|q| q.0.parse::<i32>().ok())
        .unwrap_or(110);

    let cpu_total_str = node
        .status
        .as_ref()
        .and_then(|s| s.capacity.as_ref())
        .and_then(|c| c.get("cpu"))
        .map(|q| q.0.clone())
        .unwrap_or_else(|| "0".to_string());
    let cpu_total_val = parse_cpu_quantity(&cpu_total_str);

    let mem_total_str = node
        .status
        .as_ref()
        .and_then(|s| s.capacity.as_ref())
        .and_then(|c| c.get("memory"))
        .map(|q| q.0.clone())
        .unwrap_or_else(|| "0".to_string());
    let mem_total_val = parse_memory_quantity(&mem_total_str);

    let is_cordoned = node
        .spec
        .as_ref()
        .and_then(|s| s.unschedulable)
        .unwrap_or(false);

    let system_info = node.status.as_ref().and_then(|s| s.node_info.as_ref()).cloned();

    let addresses = node
        .status
        .as_ref()
        .and_then(|s| s.addresses.as_ref())
        .cloned()
        .unwrap_or_default();

    let conditions = node
        .status
        .as_ref()
        .and_then(|s| s.conditions.as_ref())
        .cloned()
        .unwrap_or_default();

    let taints = node
        .spec
        .as_ref()
        .and_then(|s| s.taints.as_ref())
        .cloned()
        .unwrap_or_default();

    let capacity = map_node_resources(node.status.as_ref().and_then(|s| s.capacity.as_ref()));
    let allocatable = map_node_resources(node.status.as_ref().and_then(|s| s.allocatable.as_ref()));

    let images_count = node
        .status
        .as_ref()
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

    let mut pods_by_node: std::collections::HashMap<String, Vec<Pod>> =
        std::collections::HashMap::new();
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

        let cpu_total_str = node
            .status
            .as_ref()
            .and_then(|s| s.capacity.as_ref())
            .and_then(|c| c.get("cpu"))
            .map(|q| q.0.clone())
            .unwrap_or_else(|| "0".to_string());
        let cpu_total_val = parse_cpu_quantity(&cpu_total_str);

        let mem_total_str = node
            .status
            .as_ref()
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
                        if let Some(requests) =
                            container.resources.as_ref().and_then(|r| r.requests.as_ref())
                        {
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

        let cpu_pct = if cpu_total_val > 0.0 {
            ((actual_cpu_cores / cpu_total_val) * 1000.0).round() / 10.0
        } else {
            0.0
        };
        let mem_pct = if mem_total_val > 0.0 {
            ((actual_mem_gib / mem_total_val) * 1000.0).round() / 10.0
        } else {
            0.0
        };

        info.cpu_pct = cpu_pct;
        info.cpu_used = format_decimal(actual_cpu_cores, 2);
        info.mem_pct = mem_pct;
        info.mem_used = format_decimal(actual_mem_gib, 2);
        info.pods_count = pods_count;

        list.push(info);
    }

    Ok(list)
}

pub async fn fetch_nodes(client: &Client) -> Result<Vec<models::NodeInfo>, String> {
    match tokio::time::timeout(NODE_FETCH_TIMEOUT, list_nodes(client)).await {
        Ok(Ok(list)) => Ok(list),
        Ok(Err(kube::Error::Api(api_err))) => Err(api_err.message),
        Ok(Err(err)) => Err(err.to_string()),
        Err(_) => Err("Timed out fetching nodes after 10s".to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use k8s_openapi::api::core::v1::{NodeCondition, NodeSpec, NodeStatus, NodeSystemInfo};
    use k8s_openapi::apimachinery::pkg::api::resource::Quantity;
    use k8s_openapi::apimachinery::pkg::apis::meta::v1::ObjectMeta;
    use std::collections::BTreeMap;

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
    fn test_format_decimal_trimming() {
        assert_eq!(format_decimal(300.254, 2), "300.25");
        assert_eq!(format_decimal(300.200, 2), "300.2");
        assert_eq!(format_decimal(300.000, 2), "300");
        assert_eq!(format_decimal(0.0, 2), "0");
        assert_eq!(format_decimal(-0.0, 2), "0");
        assert_eq!(format_decimal(-0.0001, 2), "0");
        assert_eq!(format_decimal(2.25, 2), "2.25");
    }

    #[test]
    fn test_map_node_extracts_status_role_capacity_and_cordoned_state() {
        let mut labels = BTreeMap::new();
        labels.insert("node-role.kubernetes.io/control-plane".to_string(), "".to_string());
        labels.insert("kubernetes.io/os".to_string(), "linux".to_string());

        let mut capacity = BTreeMap::new();
        capacity.insert("cpu".to_string(), Quantity("8".to_string()));
        capacity.insert("memory".to_string(), Quantity("32Gi".to_string()));
        capacity.insert("pods".to_string(), Quantity("110".to_string()));

        let mut allocatable = BTreeMap::new();
        allocatable.insert("cpu".to_string(), Quantity("7800m".to_string()));
        allocatable.insert("memory".to_string(), Quantity("30Gi".to_string()));
        allocatable.insert("pods".to_string(), Quantity("110".to_string()));

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
                capacity: Some(capacity),
                allocatable: Some(allocatable),
                node_info: Some(NodeSystemInfo {
                    kubelet_version: "v1.30.2".to_string(),
                    ..Default::default()
                }),
                ..Default::default()
            }),
        };

        let mapped1 = map_node(&node1);
        assert_eq!(mapped1.name, "node-1");
        assert_eq!(mapped1.status, "Ready");
        assert_eq!(mapped1.role, "control-plane");
        assert_eq!(mapped1.version, "v1.30.2");
        assert_eq!(mapped1.cpu_total, "8");
        assert_eq!(mapped1.mem_total, "32");
        assert_eq!(mapped1.pods_limit, 110);
        assert!(mapped1.is_cordoned);
        assert!(mapped1.labels.contains(&"kubernetes.io/os=linux".to_string()));
        assert_eq!(mapped1.capacity.as_ref().map(|c| c.cpu.as_str()), Some("8"));
        assert_eq!(mapped1.allocatable.as_ref().map(|a| a.cpu.as_str()), Some("7800m"));

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
        assert_eq!(mapped2.version, "Unknown");
        assert!(!mapped2.is_cordoned);
    }
}
