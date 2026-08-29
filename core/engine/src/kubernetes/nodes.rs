use kube::{
    api::{Api, ApiResource, DynamicObject, GroupVersionKind, ListParams},
    Client,
};
use k8s_openapi::api::core::v1::{Node, Pod};
use crate::kubernetes::{models, format_age};

pub async fn list_nodes(client: &Client) -> Result<Vec<models::NodeInfo>, kube::Error> {
    let nodes_api: Api<Node> = Api::all(client.clone());
    let pods_api: Api<Pod> = Api::all(client.clone());

    let nodes = nodes_api.list(&ListParams::default()).await?;
    let pods = match pods_api.list(&ListParams::default()).await {
        Ok(list) => list.items,
        Err(e) => {
            log::warn!("Could not list pods for node metrics calculation (RBAC?): {}", e);
            Vec::new()
        }
    };

    let gvk = GroupVersionKind::gvk("metrics.k8s.io", "v1beta1", "NodeMetrics");
    let ar = ApiResource::from_gvk(&gvk);
    let metrics_api: Api<DynamicObject> = Api::all_with(client.clone(), &ar);
    let node_metrics = match metrics_api.list(&ListParams::default()).await {
        Ok(list) => Some(list.items),
        Err(e) => {
            log::warn!("Could not fetch node metrics (Metrics Server installed?): {}", e);
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

        let node_pods = pods_by_node.get(&name).cloned().unwrap_or_default();
        let pods_count = node_pods.len() as i32;

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

        let is_cordoned = node.spec.as_ref().and_then(|s| s.unschedulable).unwrap_or(false);

        let system_info = node.status.as_ref()
            .and_then(|s| s.node_info.as_ref())
            .map(|ni| models::NodeSystemInfo {
                machine_id: ni.machine_id.clone(),
                system_uuid: ni.system_uuid.clone(),
                boot_id: ni.boot_id.clone(),
                kernel_version: ni.kernel_version.clone(),
                os_image: ni.os_image.clone(),
                container_runtime_version: ni.container_runtime_version.clone(),
                kubelet_version: ni.kubelet_version.clone(),
                kube_proxy_version: ni.kube_proxy_version.clone(),
                operating_system: ni.operating_system.clone(),
                architecture: ni.architecture.clone(),
            });

        let addresses = node.status.as_ref()
            .and_then(|s| s.addresses.as_ref())
            .map(|addrs| {
                addrs.iter().map(|a| models::NodeAddress {
                    r#type: a.type_.clone(),
                    address: a.address.clone(),
                }).collect()
            })
            .unwrap_or_default();

        let conditions = node.status.as_ref()
            .and_then(|s| s.conditions.as_ref())
            .map(|conds| {
                conds.iter().map(|c| models::NodeCondition {
                    r#type: c.type_.clone(),
                    status: c.status.clone(),
                    reason: c.reason.clone(),
                    message: c.message.clone(),
                    last_transition_time: c.last_transition_time.as_ref().map(|t| t.0.to_rfc3339()),
                    last_heartbeat_time: c.last_heartbeat_time.as_ref().map(|t| t.0.to_rfc3339()),
                }).collect()
            })
            .unwrap_or_default();

        let taints = node.spec.as_ref()
            .and_then(|s| s.taints.as_ref())
            .map(|ts| {
                ts.iter().map(|t| models::NodeTaint {
                    key: t.key.clone(),
                    value: t.value.clone(),
                    effect: t.effect.clone(),
                    time_added: t.time_added.as_ref().map(|time| time.0.to_rfc3339()),
                }).collect()
            })
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

        list.push(models::NodeInfo {
            name,
            status,
            role,
            version,
            cpu_pct,
            cpu_used: format!("{:.1}", actual_cpu_cores),
            cpu_total: format!("{:.0}", cpu_total_val),
            mem_pct,
            mem_used: format!("{:.1}", actual_mem_gib),
            mem_total: format!("{:.0}", mem_total_val),
            pods_count,
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
        });
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
            log::warn!("parse_memory_quantity: unrecognised suffix in {:?}, treating as bytes", q);
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
}
