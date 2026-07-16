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
        tokio::select! {
            _ = cancel_rx.changed() => {
                if *cancel_rx.borrow() {
                    log::debug!("Pod metrics poller cancelled.");
                    break;
                }
            }
            _ = tokio::time::sleep(tokio::time::Duration::from_secs(15)) => {
                match api.list(&ListParams::default()).await {
                    Ok(metric_list) => {
                        let metrics: Vec<PodMetricItem> = metric_list
                            .items
                            .into_iter()
                            .filter_map(|obj| {
                                let name = obj.metadata.name.clone()?;
                                let namespace = obj.metadata.namespace.clone().unwrap_or_default();

                                // Parse the containers array from the raw JSON data.
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

                                let cpu_str = format!("{}m", (total_cpu_cores * 1000.0).round() as i64);
                                let mem_mib = (total_mem_gib * 1024.0).round() as i64;
                                let memory_str = format!("{}Mi", mem_mib);

                                Some(PodMetricItem { name, namespace, cpu: cpu_str, memory: memory_str })
                            })
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
                        log::debug!("Pod metrics unavailable (Metrics Server not installed?): {:?}", e);
                    }
                }
            }
        }
    }
}
