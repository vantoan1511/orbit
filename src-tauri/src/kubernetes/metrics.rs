use std::sync::Arc;
use tokio::sync::Mutex;
use kube::{
    api::{Api, ApiResource, DynamicObject, GroupVersionKind, ListParams},
    Client,
};

use crate::ipc::bridge::{Bridge, WsWriter};
use crate::ipc::events::OrbitEvent;
use crate::kubernetes::models::PodMetricItem;
pub use orbit_core::kubernetes::metrics::{
    format_cpu_cores, format_memory_mib, parse_pod_metric_obj, parse_pod_metrics_map,
};
pub use orbit_core::kubernetes::nodes::format_decimal;

/// Polls PodMetrics from the Metrics Server every 15 seconds and broadcasts
/// the aggregated CPU/memory usage per pod via the `podMetricsUpdated` event.
/// Uses kube's dynamic API — no extra crate needed, and handles missing Metrics
/// Server gracefully.
/// The loop exits cleanly when `cancel_rx` receives `true`.
pub async fn poll_pod_metrics(
    client: Client,
    writer: Arc<Mutex<WsWriter>>,
    token: String,
    cluster_id: Option<String>,
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
                        &OrbitEvent::PodMetricsUpdated { cluster_id: cluster_id.clone(), metrics },
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
