use std::sync::Arc;
use futures_util::{AsyncBufReadExt, StreamExt};
use kube::{api::{Api, LogParams}, Client};
use k8s_openapi::api::core::v1::Pod;
use k8s_openapi::api::apps::v1::{Deployment, StatefulSet, DaemonSet, ReplicaSet};
use k8s_openapi::api::batch::v1::Job;
use tokio::sync::Mutex;
use crate::ipc::bridge::{Bridge, WsWriter};
use crate::ipc::events::OrbitEvent;

#[allow(clippy::too_many_arguments)]
pub async fn stream_pod_logs(
    client: Client,
    writer: Arc<Mutex<WsWriter>>,
    token: String,
    namespace: String,
    pod_name: String,
    container: Option<String>,
    tail_lines: Option<i64>,
    mut cancel_rx: tokio::sync::oneshot::Receiver<()>,
) {
    let pods: Api<Pod> = Api::namespaced(client, &namespace);
    let lp = LogParams {
        follow: true,
        container: container.clone(),
        tail_lines,
        timestamps: true,
        ..Default::default()
    };

    let log_stream = match pods.log_stream(&pod_name, &lp).await {
        Ok(s) => s,
        Err(e) => {
            log::error!("Failed to open log stream for {}: {}", pod_name, e);
            let _ = Bridge::send_event(
                &writer,
                &token,
                &OrbitEvent::ErrorOccurred {
                    message: format!("Failed to stream logs for pod {}: {}", pod_name, e),
                },
            ).await;
            return;
        }
    };

    let mut lines = log_stream.lines();

    loop {
        tokio::select! {
            _ = &mut cancel_rx => {
                break;
            }
            res = lines.next() => {
                match res {
                    Some(Ok(line)) => {
                        let _ = Bridge::send_event(
                            &writer,
                            &token,
                            &OrbitEvent::LogLineReceived {
                                pod: pod_name.clone(),
                                container: container.clone().unwrap_or_default(),
                                line,
                            },
                        ).await;
                    }
                    Some(Err(e)) => {
                        log::error!("Error reading log stream: {}", e);
                        break;
                    }
                    None => {
                        break;
                    }
                }
            }
        }
    }
}

pub async fn get_workload_pods(
    client: &Client,
    namespace: &str,
    workload_name: &str,
    workload_kind: &str,
) -> Result<Vec<String>, kube::Error> {
    if workload_kind == "Pod" {
        return Ok(vec![workload_name.to_string()]);
    }

    let pods_api: Api<Pod> = Api::namespaced(client.clone(), namespace);
    let mut pod_names = Vec::new();

    let selector = match workload_kind {
        "Deployment" => {
            let api: Api<Deployment> = Api::namespaced(client.clone(), namespace);
            api.get(workload_name).await?.spec.map(|s| s.selector)
        }
        "StatefulSet" => {
            let api: Api<StatefulSet> = Api::namespaced(client.clone(), namespace);
            api.get(workload_name).await?.spec.map(|s| s.selector)
        }
        "DaemonSet" => {
            let api: Api<DaemonSet> = Api::namespaced(client.clone(), namespace);
            api.get(workload_name).await?.spec.map(|s| s.selector)
        }
        "ReplicaSet" => {
            let api: Api<ReplicaSet> = Api::namespaced(client.clone(), namespace);
            api.get(workload_name).await?.spec.map(|s| s.selector)
        }
        "Job" => {
            let api: Api<Job> = Api::namespaced(client.clone(), namespace);
            api.get(workload_name).await?.spec.and_then(|s| s.selector)
        }
        _ => None,
    };

    if let Some(sel) = selector {
        let mut parts = Vec::new();
        if let Some(match_labels) = sel.match_labels {
            for (k, v) in match_labels {
                parts.push(format!("{}={}", k, v));
            }
        }
        if let Some(match_expressions) = sel.match_expressions {
            for req in match_expressions {
                let key = req.key;
                match req.operator.as_str() {
                    "In" => {
                        if let Some(values) = req.values {
                            parts.push(format!("{} in ({})", key, values.join(",")));
                        }
                    }
                    "NotIn" => {
                        if let Some(values) = req.values {
                            parts.push(format!("{} notin ({})", key, values.join(",")));
                        }
                    }
                    "Exists" => {
                        parts.push(key);
                    }
                    "DoesNotExist" => {
                        parts.push(format!("!{}", key));
                    }
                    _ => {}
                }
            }
        }
        let selector_str = parts.join(",");
        let lp = kube::api::ListParams::default().labels(&selector_str);
        let list = pods_api.list(&lp).await?;
        for p in list {
            if let Some(name) = p.metadata.name {
                pod_names.push(name);
            }
        }
    }

    Ok(pod_names)
}
