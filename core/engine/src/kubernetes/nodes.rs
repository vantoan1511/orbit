use kube::{
    api::{Api, ListParams},
    Client,
};
use k8s_openapi::api::core::v1::{Node, Pod};
use crate::kubernetes::{models, format_age};

pub async fn list_nodes(client: &Client) -> Result<Vec<models::NodeInfo>, kube::Error> {
    let nodes_api: Api<Node> = Api::all(client.clone());
    let pods_api: Api<Pod> = Api::all(client.clone());

    let nodes = nodes_api.list(&ListParams::default()).await?;
    let pods = pods_api.list(&ListParams::default()).await?;

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

        let mut labels = Vec::new();
        if let Some(map) = &node.metadata.labels {
            for (k, v) in map {
                labels.push(format!("{}={}", k, v));
            }
        }
        labels.sort();

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

        let mut cpu_req = 0.0;
        let mut mem_req = 0.0;

        for pod in &node_pods {
            if let Some(spec) = &pod.spec {
                for container in &spec.containers {
                    if let Some(requests) = container.resources.as_ref().and_then(|r| r.requests.as_ref()) {
                        if let Some(cpu) = requests.get("cpu") {
                            cpu_req += parse_cpu_quantity(&cpu.0);
                        }
                        if let Some(mem) = requests.get("memory") {
                            mem_req += parse_memory_quantity(&mem.0);
                        }
                    }
                }
            }
        }

        let cpu_pct = if cpu_total_val > 0.0 { ((cpu_req / cpu_total_val) * 1000.0).round() / 10.0 } else { 0.0 };
        let mem_pct = if mem_total_val > 0.0 { ((mem_req / mem_total_val) * 1000.0).round() / 10.0 } else { 0.0 };

        let is_cordoned = node.spec.as_ref().and_then(|s| s.unschedulable).unwrap_or(false);

        list.push(models::NodeInfo {
            name,
            status,
            role,
            version,
            cpu_pct,
            cpu_used: format!("{:.1}", cpu_req),
            cpu_total: format!("{:.0}", cpu_total_val),
            mem_pct,
            mem_used: format!("{:.1}", mem_req),
            mem_total: format!("{:.0}", mem_total_val),
            pods_count,
            pods_limit,
            uptime,
            labels,
            is_cordoned,
        });
    }

    Ok(list)
}

pub(crate) fn parse_cpu_quantity(q: &str) -> f64 {
    let q = q.trim();
    if let Some(stripped) = q.strip_suffix('m') {
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
