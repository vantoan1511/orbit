use std::sync::Arc;
use serde_json::Value;
use tokio::sync::{Mutex, RwLock};
use crate::ipc::bridge::{Bridge, WsWriter};
use crate::ipc::events::OrbitEvent;
use crate::kubernetes::manager::KubeManager;
use super::utils::{get_i64, get_string};

pub fn stream_logs(
    data: Option<Value>,
    writer: Arc<Mutex<WsWriter>>,
    token: String,
    manager: Arc<RwLock<KubeManager>>,
) {
    tokio::spawn(async move {
        let namespace = get_string(&data, "namespace").unwrap_or_else(|| "default".to_string());
        let workload_name = get_string(&data, "workload").unwrap_or_default();
        let workload_kind = get_string(&data, "kind").unwrap_or_else(|| "Deployment".to_string());
        let container = get_string(&data, "container").filter(|s| !s.is_empty() && s != "All" && s != "all");
        let pod_name = get_string(&data, "pod").filter(|s| !s.is_empty() && s != "All" && s != "all");
        // A tailLines value <= 0 is the frontend sentinel for "fetch all lines"
        // (TAIL_ALL_LINES = -1). Mapping to None instructs the Kubernetes log API
        // to return the full log history without a tail cap.
        let tail_lines = match get_i64(&data, "tailLines") {
            Some(v) if v > 0 => Some(v),
            _ => None,
        };

        let mut w_manager = manager.write().await;
        for cancel in w_manager.log_cancel.drain(..) {
            let _ = cancel.send(());
        }

        let client = match w_manager.active_client.clone() {
            Some(c) => c,
            None => {
                let _ = Bridge::send_event(
                    &writer,
                    &token,
                    &OrbitEvent::ErrorOccurred {
                        message: "No active cluster client found".to_string(),
                    },
                ).await;
                return;
            }
        };

        let pods_to_stream = if let Some(specific_pod) = pod_name {
            vec![specific_pod]
        } else {
            match crate::kubernetes::get_workload_pods(&client, &namespace, &workload_name, &workload_kind).await {
                Ok(p) => p,
                Err(e) => {
                    let _ = Bridge::send_event(
                        &writer,
                        &token,
                        &OrbitEvent::ErrorOccurred {
                            message: format!("Failed to find pods for workload: {}", crate::kubernetes::format_error(&e)),
                        },
                    ).await;
                    return;
                }
            }
        };

        for pod in pods_to_stream {
            let (cancel_tx, cancel_rx) = tokio::sync::oneshot::channel();
            w_manager.log_cancel.push(cancel_tx);

            let client_clone = client.clone();
            let writer_clone = writer.clone();
            let token_clone = token.clone();
            let ns_clone = namespace.clone();
            let container_clone = container.clone();

            tokio::spawn(async move {
                crate::kubernetes::stream_pod_logs(
                    client_clone,
                    writer_clone,
                    token_clone,
                    ns_clone,
                    pod,
                    container_clone,
                    tail_lines,
                    cancel_rx,
                ).await;
            });
        }
    });
}

pub fn stop_logs(
    manager: Arc<RwLock<KubeManager>>,
) {
    tokio::spawn(async move {
        let mut w_manager = manager.write().await;
        for cancel in w_manager.log_cancel.drain(..) {
            let _ = cancel.send(());
        }
    });
}
