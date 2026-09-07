use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use kube::{
    api::{Api, ApiResource, DynamicObject, GroupVersionKind, ListParams},
    Client,
};

use crate::ipc::bridge::{Bridge, WsWriter};
use crate::ipc::events::OrbitEvent;
use crate::kubernetes::models::PodMetricItem;
use crate::kubernetes::nodes::{parse_cpu_quantity, parse_memory_quantity};

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

pub fn format_memory_mib(mib: f64) -> String {
    format!("{}Mi", format_decimal(mib, 2))
}

pub fn format_cpu_cores(cores: f64) -> String {
    if cores >= 1.0 {
        format!("{} cores", format_decimal(cores, 2))
    } else if cores > 0.0 {
        format!("{}m", format_decimal(cores * 1000.0, 2))
    } else {
        "0m".to_string()
    }
}

pub fn parse_pod_metric_obj(obj: &DynamicObject) -> Option<PodMetricItem> {
    let name = obj.metadata.name.clone()?;
    let namespace = obj.metadata.namespace.clone().unwrap_or_default();

    let containers = obj.data.get("containers")?.as_array()?;

    let mut total_cpu_cores: f64 = 0.0;
    let mut total_mem_gib: f64 = 0.0;

    for container in containers {
        if let Some(usage) = container.get("usage") {
            if let Some(cpu) = usage.get("cpu").and_then(|v| v.as_str()) {
                total_cpu_cores += parse_cpu_quantity(cpu);
            }
            if let Some(mem) = usage.get("memory").and_then(|v| v.as_str()) {
                total_mem_gib += parse_memory_quantity(mem);
            }
        }
    }

    let cpu_str = format_cpu_cores(total_cpu_cores);
    let mem_mib = total_mem_gib * 1024.0;
    let memory_str = format_memory_mib(mem_mib);
    let memory_bytes = total_mem_gib * 1024.0 * 1024.0 * 1024.0;

    Some(PodMetricItem {
        name,
        namespace,
        cpu: cpu_str,
        memory: memory_str,
        cpu_cores: Some(total_cpu_cores),
        memory_bytes: Some(memory_bytes),
    })
}

pub fn parse_pod_metrics_map(items: &[DynamicObject]) -> HashMap<(String, String), PodMetricItem> {
    let mut map = HashMap::new();
    for obj in items {
        if let Some(item) = parse_pod_metric_obj(obj) {
            map.insert((item.name.clone(), item.namespace.clone()), item);
        }
    }
    map
}

/// Polls PodMetrics from the Metrics Server every 15 seconds and broadcasts
/// the aggregated CPU/memory usage per pod via the `podMetricsUpdated` event.
/// Uses kube's dynamic API — no extra crate needed, and handles missing Metrics
/// Server gracefully.
/// The loop exits cleanly when `cancel_rx` receives `true`.
pub async fn poll_pod_metrics(
    client: Client,
    writer: Arc<Mutex<WsWriter>>,
    token: String,
    mut cancel_rx: tokio::sync::watch::Receiver<bool>,
) {
    let gvk = GroupVersionKind::gvk("metrics.k8s.io", "v1beta1", "PodMetrics");
    let ar = ApiResource::from_gvk(&gvk);
    let api: Api<DynamicObject> = Api::all_with(client, &ar);

    loop {
        // Poll immediately on first iteration, then wait 15 seconds before each subsequent poll.
        match api.list(&ListParams::default()).await {
            Ok(metric_list) => {
                let metrics: Vec<PodMetricItem> = metric_list
                    .items
                    .iter()
                    .filter_map(parse_pod_metric_obj)
                    .collect();

                if !metrics.is_empty() {
                    let _ = Bridge::send_event(
                        &writer,
                        &token,
                        &OrbitEvent::PodMetricsUpdated { metrics },
                    ).await;
                }
            }
            Err(e) => {
                // Metrics Server may not be installed — log at debug level and keep polling.
                tracing::debug!(error = ?e, "Pod metrics unavailable (Metrics Server not installed?)");
            }
        }

        tokio::select! {
            _ = cancel_rx.changed() => {
                if *cancel_rx.borrow() {
                    tracing::debug!("Pod metrics poller cancelled");
                    break;
                }
            }
            _ = tokio::time::sleep(tokio::time::Duration::from_secs(15)) => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
    fn test_format_memory_mib() {
        assert_eq!(format_memory_mib(300.25), "300.25Mi");
        assert_eq!(format_memory_mib(300.0), "300Mi");
        assert_eq!(format_memory_mib(45.5), "45.5Mi");
    }

    #[test]
    fn test_format_cpu_cores() {
        assert_eq!(format_cpu_cores(2.25), "2.25 cores");
        assert_eq!(format_cpu_cores(1.0), "1 cores");
        assert_eq!(format_cpu_cores(0.25), "250m");
        assert_eq!(format_cpu_cores(0.05), "50m");
        assert_eq!(format_cpu_cores(0.0), "0m");
    }

    #[test]
    fn test_parse_pod_metric_obj() {
        use serde_json::json;
        let mut obj = DynamicObject::new("test-pod", &ApiResource::from_gvk(&GroupVersionKind::gvk("metrics.k8s.io", "v1beta1", "PodMetrics")));
        obj.metadata.namespace = Some("default".to_string());
        obj.data = json!({
            "containers": [
                {
                    "name": "c1",
                    "usage": {
                        "cpu": "2250m",
                        "memory": "307200Ki" // 300 MiB
                    }
                }
            ]
        });

        let parsed = parse_pod_metric_obj(&obj).expect("should parse");
        assert_eq!(parsed.name, "test-pod");
        assert_eq!(parsed.namespace, "default");
        assert_eq!(parsed.cpu, "2.25 cores");
        assert_eq!(parsed.memory, "300Mi");
        assert_eq!(parsed.cpu_cores, Some(2.25));
        assert!(parsed.memory_bytes.is_some());
    }
}
