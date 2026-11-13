use std::sync::Arc;
use futures_util::{AsyncBufReadExt, StreamExt};
use kube::{api::{Api, LogParams}, Client};
use k8s_openapi::api::core::v1::Pod;
use k8s_openapi::api::apps::v1::{Deployment, StatefulSet, DaemonSet, ReplicaSet};
use k8s_openapi::api::batch::v1::Job;
use crate::events::CoreEvent;
use crate::sink::CoreEventSink;

const HTTP_STATUS_BAD_REQUEST: u16 = 400;
const CONTAINER_WAITING_TO_START_SUBSTRING: &str = "waiting to start";
const LOG_LINE_SYSTEM_PREFIX: &str = "[Orbit]";

#[allow(clippy::too_many_arguments)]
pub async fn stream_pod_logs(
    client: Client,
    sink: Arc<dyn CoreEventSink>,
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
            if let Some(msg) = extract_waiting_container_message(&e) {
                let _ = sink.send(CoreEvent::LogStreamReceived {
                    chunk: format!("{} {}\n", LOG_LINE_SYSTEM_PREFIX, msg),
                });
                return;
            }

            tracing::error!(pod = %pod_name, error = %e, "Failed to open log stream");
            let _ = sink.send(CoreEvent::ErrorOccurred {
                action: "stream_logs".to_string(),
                message: format!("Failed to stream logs for pod {}: {}", pod_name, e),
            });
            return;
        }
    };

    let mut lines = log_stream.lines();
    let mut buffer: Vec<String> = Vec::new();
    let mut interval = tokio::time::interval(std::time::Duration::from_millis(100));

    loop {
        tokio::select! {
            _ = &mut cancel_rx => {
                break;
            }
            _ = interval.tick() => {
                if !buffer.is_empty() {
                    let chunk = std::mem::take(&mut buffer).join("\n") + "\n";
                    let _ = sink.send(CoreEvent::LogStreamReceived { chunk });
                }
            }
            res = lines.next() => {
                match res {
                    Some(Ok(line)) => {
                        buffer.push(line);
                        if buffer.len() >= 500 {
                            let chunk = std::mem::take(&mut buffer).join("\n") + "\n";
                            let _ = sink.send(CoreEvent::LogStreamReceived { chunk });
                        }
                    }
                    Some(Err(e)) => {
                        tracing::error!(pod = %pod_name, error = %e, "Error reading log stream");
                        break;
                    }
                    None => {
                        break;
                    }
                }
            }
        }
    }

    if !buffer.is_empty() {
        let chunk = buffer.join("\n") + "\n";
        let _ = sink.send(CoreEvent::LogStreamReceived { chunk });
    }
    let _ = sink.send(CoreEvent::LogStreamStopped);
}

pub async fn get_workload_pods(
    client: &Client,
    namespace: &str,
    workload_kind: &str,
    workload_name: &str,
) -> Result<Vec<String>, String> {
    let pods_api: Api<Pod> = Api::namespaced(client.clone(), namespace);

    let match_labels = match workload_kind.to_ascii_lowercase().as_str() {
        "deployment" => {
            let api: Api<Deployment> = Api::namespaced(client.clone(), namespace);
            let d = api.get(workload_name).await.map_err(|e| e.to_string())?;
            d.spec.and_then(|s| s.selector.match_labels)
        }
        "statefulset" => {
            let api: Api<StatefulSet> = Api::namespaced(client.clone(), namespace);
            let s = api.get(workload_name).await.map_err(|e| e.to_string())?;
            s.spec.and_then(|s| s.selector.match_labels)
        }
        "daemonset" => {
            let api: Api<DaemonSet> = Api::namespaced(client.clone(), namespace);
            let d = api.get(workload_name).await.map_err(|e| e.to_string())?;
            d.spec.and_then(|s| s.selector.match_labels)
        }
        "replicaset" => {
            let api: Api<ReplicaSet> = Api::namespaced(client.clone(), namespace);
            let r = api.get(workload_name).await.map_err(|e| e.to_string())?;
            r.spec.and_then(|s| s.selector.match_labels)
        }
        "job" => {
            let api: Api<Job> = Api::namespaced(client.clone(), namespace);
            let j = api.get(workload_name).await.map_err(|e| e.to_string())?;
            j.spec.and_then(|s| s.selector.and_then(|sel| sel.match_labels))
        }
        _ => return Err(format!("Unsupported workload kind: {}", workload_kind)),
    };

    let Some(labels) = match_labels else {
        return Ok(Vec::new());
    };

    let selector = labels
        .into_iter()
        .map(|(k, v)| format!("{}={}", k, v))
        .collect::<Vec<_>>()
        .join(",");

    let lp = kube::api::ListParams::default().labels(&selector);
    let pod_list = pods_api.list(&lp).await.map_err(|e| e.to_string())?;

    let mut pod_names: Vec<String> = pod_list
        .items
        .into_iter()
        .filter_map(|p| p.metadata.name)
        .collect();
    pod_names.sort();
    Ok(pod_names)
}

fn extract_waiting_container_message(err: &kube::Error) -> Option<String> {
    match err {
        kube::Error::Api(response)
            if response.code == HTTP_STATUS_BAD_REQUEST
                && response.message.to_ascii_lowercase().contains(CONTAINER_WAITING_TO_START_SUBSTRING) =>
        {
            Some(response.message.clone())
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use kube::core::ErrorResponse;

    #[test]
    fn test_extract_waiting_container_message_matches() {
        let err = kube::Error::Api(ErrorResponse {
            status: "Failure".to_string(),
            message: "container \"web\" in pod \"pod-xyz\" is waiting to start: ContainerCreating".to_string(),
            reason: "BadRequest".to_string(),
            code: 400,
        });

        assert_eq!(
            extract_waiting_container_message(&err),
            Some("container \"web\" in pod \"pod-xyz\" is waiting to start: ContainerCreating".to_string())
        );
    }

    #[test]
    fn test_extract_waiting_container_message_ignores_other_status() {
        let err = kube::Error::Api(ErrorResponse {
            status: "Failure".to_string(),
            message: "container \"web\" in pod \"pod-xyz\" is waiting to start: ContainerCreating".to_string(),
            reason: "NotFound".to_string(),
            code: 404,
        });

        assert_eq!(extract_waiting_container_message(&err), None);
    }

    #[test]
    fn test_extract_waiting_container_message_ignores_non_matching_message() {
        let err = kube::Error::Api(ErrorResponse {
            status: "Failure".to_string(),
            message: "pod not found".to_string(),
            reason: "BadRequest".to_string(),
            code: 400,
        });

        assert_eq!(extract_waiting_container_message(&err), None);
    }
}
