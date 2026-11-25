use futures_util::{AsyncBufReadExt, StreamExt};
use k8s_openapi::api::apps::v1::{DaemonSet, Deployment, ReplicaSet, StatefulSet};
use k8s_openapi::api::batch::v1::Job;
use k8s_openapi::api::core::v1::Pod;
use k8s_openapi::apimachinery::pkg::apis::meta::v1::LabelSelector;
use kube::{
    Client,
    api::{Api, ListParams, LogParams},
};
use tokio::sync::{mpsc, oneshot};

const HTTP_STATUS_BAD_REQUEST: u16 = 400;
const CONTAINER_WAITING_TO_START_SUBSTRING: &str = "waiting to start";
const LOG_LINE_SYSTEM_PREFIX: &str = "[Orbit]";
const LOG_BATCH_INTERVAL_MS: u64 = 100;
const LOG_BATCH_MAX_LINES: usize = 500;

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct WorkloadPodTarget {
    pub name: String,
    pub containers: Vec<String>,
    pub phase: String,
    pub is_terminating: bool,
    pub is_available: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LogStreamMessage {
    Lines {
        pod: String,
        container: String,
        lines: Vec<String>,
    },
    SystemNotice {
        pod: String,
        container: String,
        message: String,
    },
    Error {
        pod: String,
        message: String,
    },
    Ended {
        pod: String,
        container: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct LogExportRow {
    pub timestamp: Option<String>,
    pub pod: String,
    pub container: String,
    pub message: String,
}

#[must_use]
pub fn is_pod_log_available(pod: &Pod) -> bool {
    if pod.metadata.deletion_timestamp.is_some() {
        return false;
    }

    let phase = pod
        .status
        .as_ref()
        .and_then(|s| s.phase.as_deref())
        .unwrap_or("");

    if phase.is_empty() || phase == "Pending" || phase == "Unknown" {
        return false;
    }

    if let Some(statuses) = pod
        .status
        .as_ref()
        .and_then(|s| s.container_statuses.as_ref())
        && !statuses.is_empty()
    {
        let all_waiting = statuses.iter().all(|cs| {
            cs.state.as_ref().is_some_and(|st| {
                st.waiting.is_some() && st.running.is_none() && st.terminated.is_none()
            })
        });
        if all_waiting {
            return false;
        }
    }

    matches!(phase, "Running" | "Succeeded" | "Failed")
}

#[must_use]
pub fn pod_to_workload_target(pod: &Pod) -> Option<WorkloadPodTarget> {
    let name = pod.metadata.name.clone()?;
    let mut containers: Vec<String> = pod
        .spec
        .as_ref()
        .map(|s| s.containers.iter().map(|c| c.name.clone()).collect())
        .unwrap_or_default();

    if containers.is_empty()
        && let Some(statuses) = pod
            .status
            .as_ref()
            .and_then(|s| s.container_statuses.as_ref())
    {
        containers = statuses.iter().map(|cs| cs.name.clone()).collect();
    }

    let phase = pod
        .status
        .as_ref()
        .and_then(|s| s.phase.clone())
        .unwrap_or_else(|| "Unknown".to_string());

    let is_terminating = pod.metadata.deletion_timestamp.is_some();
    let is_available = is_pod_log_available(pod);

    Some(WorkloadPodTarget {
        name,
        containers,
        phase,
        is_terminating,
        is_available,
    })
}

#[must_use]
pub fn select_streamable_pods(
    pods: &[WorkloadPodTarget],
    pod_filter: Option<&str>,
    include_unavailable: bool,
) -> (Vec<WorkloadPodTarget>, usize) {
    let excluded_unavailable_count = if include_unavailable {
        0
    } else {
        pods.iter().filter(|p| !p.is_available).count()
    };

    let normalized_filter = pod_filter.filter(|f| !f.is_empty() && *f != "All" && *f != "all");

    let selected = pods
        .iter()
        .filter(|p| include_unavailable || p.is_available)
        .filter(|p| normalized_filter.is_none_or(|target| p.name == target))
        .cloned()
        .collect();

    (selected, excluded_unavailable_count)
}

#[must_use]
pub fn build_label_selector(selector: &LabelSelector) -> String {
    let mut parts = Vec::new();
    if let Some(match_labels) = &selector.match_labels {
        let mut pairs: Vec<(&String, &String)> = match_labels.iter().collect();
        pairs.sort_by_key(|(a, _)| *a);
        for (k, v) in pairs {
            parts.push(format!("{k}={v}"));
        }
    }
    if let Some(match_expressions) = &selector.match_expressions {
        for req in match_expressions {
            let key = &req.key;
            match req.operator.as_str() {
                "In" => {
                    if let Some(values) = &req.values {
                        parts.push(format!("{key} in ({})", values.join(",")));
                    }
                }
                "NotIn" => {
                    if let Some(values) = &req.values {
                        parts.push(format!("{key} notin ({})", values.join(",")));
                    }
                }
                "Exists" => {
                    parts.push(key.clone());
                }
                "DoesNotExist" => {
                    parts.push(format!("!{key}"));
                }
                _ => {}
            }
        }
    }
    parts.join(",")
}

pub async fn resolve_workload_pods(
    client: &Client,
    namespace: &str,
    workload_name: &str,
    workload_kind: &str,
) -> Result<Vec<WorkloadPodTarget>, kube::Error> {
    let pods_api: Api<Pod> = Api::namespaced(client.clone(), namespace);

    if workload_kind == "Pod" {
        let pod = pods_api.get(workload_name).await?;
        return Ok(pod_to_workload_target(&pod).into_iter().collect());
    }

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

    let Some(sel) = selector else {
        return Ok(Vec::new());
    };

    let selector_str = build_label_selector(&sel);
    let lp = if selector_str.is_empty() {
        ListParams::default()
    } else {
        ListParams::default().labels(&selector_str)
    };
    let list = pods_api.list(&lp).await?;
    let mut targets: Vec<WorkloadPodTarget> = list
        .iter()
        .filter_map(pod_to_workload_target)
        .collect();
    targets.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(targets)
}

#[allow(clippy::too_many_arguments)]
pub async fn stream_pod_logs(
    client: Client,
    namespace: String,
    pod_name: String,
    container: Option<String>,
    tail_lines: Option<i64>,
    tx: mpsc::UnboundedSender<LogStreamMessage>,
    mut cancel_rx: oneshot::Receiver<()>,
) {
    let container_label = container.clone().unwrap_or_default();
    let pods: Api<Pod> = Api::namespaced(client, &namespace);
    let lp = LogParams {
        follow: true,
        container,
        tail_lines,
        timestamps: true,
        ..Default::default()
    };

    let log_stream = match pods.log_stream(&pod_name, &lp).await {
        Ok(s) => s,
        Err(e) => {
            if let Some(msg) = extract_waiting_container_message(&e) {
                let _ = tx.send(LogStreamMessage::SystemNotice {
                    pod: pod_name.clone(),
                    container: container_label.clone(),
                    message: format!("{LOG_LINE_SYSTEM_PREFIX} {msg}"),
                });
            } else {
                tracing::error!(pod = %pod_name, error = %e, "Failed to open log stream");
                let _ = tx.send(LogStreamMessage::Error {
                    pod: pod_name.clone(),
                    message: format!("Failed to stream logs for pod {pod_name}: {e}"),
                });
            }
            let _ = tx.send(LogStreamMessage::Ended {
                pod: pod_name,
                container: container_label,
            });
            return;
        }
    };

    let mut lines = log_stream.lines();
    let mut buffer: Vec<String> = Vec::new();
    let mut interval =
        tokio::time::interval(std::time::Duration::from_millis(LOG_BATCH_INTERVAL_MS));

    loop {
        tokio::select! {
            _ = &mut cancel_rx => {
                break;
            }
            _ = interval.tick() => {
                if !buffer.is_empty() {
                    let chunk = std::mem::take(&mut buffer);
                    let _ = tx.send(LogStreamMessage::Lines {
                        pod: pod_name.clone(),
                        container: container_label.clone(),
                        lines: chunk,
                    });
                }
            }
            res = lines.next() => {
                match res {
                    Some(Ok(line)) => {
                        buffer.push(line);
                        if buffer.len() >= LOG_BATCH_MAX_LINES {
                            let chunk = std::mem::take(&mut buffer);
                            let _ = tx.send(LogStreamMessage::Lines {
                                pod: pod_name.clone(),
                                container: container_label.clone(),
                                lines: chunk,
                            });
                        }
                    }
                    Some(Err(e)) => {
                        tracing::error!(pod = %pod_name, error = %e, "Error reading log stream");
                        let _ = tx.send(LogStreamMessage::Error {
                            pod: pod_name.clone(),
                            message: format!("Error reading log stream for pod {pod_name}: {e}"),
                        });
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
        let _ = tx.send(LogStreamMessage::Lines {
            pod: pod_name.clone(),
            container: container_label.clone(),
            lines: buffer,
        });
    }

    let _ = tx.send(LogStreamMessage::Ended {
        pod: pod_name,
        container: container_label,
    });
}

#[must_use]
pub fn parse_k8s_log_line(line: &str) -> (Option<&str>, &str) {
    if let Some((head, rest)) = line.split_once(' ')
        && chrono::DateTime::parse_from_rfc3339(head).is_ok()
    {
        return (Some(head), rest);
    }
    (None, line)
}

#[must_use]
pub fn extract_waiting_container_message(err: &kube::Error) -> Option<&str> {
    if let kube::Error::Api(api_err) = err
        && api_err.code == HTTP_STATUS_BAD_REQUEST
        && api_err.message.contains(CONTAINER_WAITING_TO_START_SUBSTRING)
    {
        return Some(&api_err.message);
    }
    None
}

pub fn format_logs_as_json(rows: &[LogExportRow]) -> Result<String, serde_json::Error> {
    serde_json::to_string_pretty(rows)
}

fn escape_csv_field(field: &str) -> String {
    if field.contains([',', '"', '\n', '\r']) {
        let escaped = field.replace('"', "\"\"");
        format!("\"{escaped}\"")
    } else {
        field.to_string()
    }
}

#[must_use]
pub fn format_logs_as_csv(rows: &[LogExportRow]) -> String {
    let mut out = String::from("timestamp,pod,container,message\n");
    for row in rows {
        let ts = row.timestamp.as_deref().unwrap_or("");
        out.push_str(&escape_csv_field(ts));
        out.push(',');
        out.push_str(&escape_csv_field(&row.pod));
        out.push(',');
        out.push_str(&escape_csv_field(&row.container));
        out.push(',');
        out.push_str(&escape_csv_field(&row.message));
        out.push('\n');
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use k8s_openapi::api::core::v1::{
        Container, ContainerState, ContainerStateRunning, ContainerStateWaiting, ContainerStatus,
        PodSpec, PodStatus,
    };
    use k8s_openapi::apimachinery::pkg::apis::meta::v1::{ObjectMeta, Time};
    use kube::error::ErrorResponse;

    fn make_pod(name: &str, phase: &str, terminating: bool, waiting_only: bool) -> Pod {
        let container_statuses = if waiting_only {
            Some(vec![ContainerStatus {
                name: "app".to_string(),
                state: Some(ContainerState {
                    waiting: Some(ContainerStateWaiting {
                        reason: Some("ContainerCreating".to_string()),
                        ..Default::default()
                    }),
                    ..Default::default()
                }),
                ..Default::default()
            }])
        } else {
            Some(vec![ContainerStatus {
                name: "app".to_string(),
                state: Some(ContainerState {
                    running: Some(ContainerStateRunning::default()),
                    ..Default::default()
                }),
                ..Default::default()
            }])
        };

        Pod {
            metadata: ObjectMeta {
                name: Some(name.to_string()),
                deletion_timestamp: if terminating {
                    Some(Time(chrono::Utc::now()))
                } else {
                    None
                },
                ..Default::default()
            },
            spec: Some(PodSpec {
                containers: vec![Container {
                    name: "app".to_string(),
                    ..Default::default()
                }],
                ..Default::default()
            }),
            status: Some(PodStatus {
                phase: Some(phase.to_string()),
                container_statuses,
                ..Default::default()
            }),
        }
    }

    #[test]
    fn test_is_pod_log_available_excludes_terminating_and_pending_pods() {
        let running = make_pod("pod-run", "Running", false, false);
        assert!(is_pod_log_available(&running));

        let succeeded = make_pod("pod-done", "Succeeded", false, false);
        assert!(is_pod_log_available(&succeeded));

        let terminating = make_pod("pod-term", "Running", true, false);
        assert!(!is_pod_log_available(&terminating));

        let pending = make_pod("pod-pend", "Pending", false, false);
        assert!(!is_pod_log_available(&pending));

        let waiting = make_pod("pod-wait", "Running", false, true);
        assert!(!is_pod_log_available(&waiting));
    }

    #[test]
    fn test_select_streamable_pods_respects_include_unavailable_and_pod_filter() {
        let targets = vec![
            WorkloadPodTarget {
                name: "pod-a".to_string(),
                containers: vec!["app".to_string()],
                phase: "Running".to_string(),
                is_terminating: false,
                is_available: true,
            },
            WorkloadPodTarget {
                name: "pod-b".to_string(),
                containers: vec!["app".to_string()],
                phase: "Pending".to_string(),
                is_terminating: false,
                is_available: false,
            },
            WorkloadPodTarget {
                name: "pod-c".to_string(),
                containers: vec!["app".to_string()],
                phase: "Running".to_string(),
                is_terminating: false,
                is_available: true,
            },
        ];

        let (selected, excluded) = select_streamable_pods(&targets, None, false);
        assert_eq!(excluded, 1);
        assert_eq!(selected.len(), 2);
        assert_eq!(selected[0].name, "pod-a");
        assert_eq!(selected[1].name, "pod-c");

        let (selected_all, excluded_all) = select_streamable_pods(&targets, None, true);
        assert_eq!(excluded_all, 0);
        assert_eq!(selected_all.len(), 3);

        let (filtered, excluded_filtered) = select_streamable_pods(&targets, Some("pod-a"), false);
        assert_eq!(excluded_filtered, 1);
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].name, "pod-a");
    }

    #[test]
    fn test_parse_k8s_log_line_splits_rfc3339_timestamp_and_message() {
        let (ts, msg) = parse_k8s_log_line("2026-11-25T12:00:00.123456Z hello world");
        assert_eq!(ts, Some("2026-11-25T12:00:00.123456Z"));
        assert_eq!(msg, "hello world");

        let (ts2, msg2) = parse_k8s_log_line("plain line without timestamp");
        assert_eq!(ts2, None);
        assert_eq!(msg2, "plain line without timestamp");
    }

    #[test]
    fn test_format_logs_as_json_and_csv_escapes_special_characters() {
        let rows = vec![
            LogExportRow {
                timestamp: Some("2026-11-25T12:00:00Z".to_string()),
                pod: "pod-a".to_string(),
                container: "app".to_string(),
                message: "hello, \"world\"\nline2".to_string(),
            },
            LogExportRow {
                timestamp: None,
                pod: "pod-b".to_string(),
                container: "sidecar".to_string(),
                message: "ok".to_string(),
            },
        ];

        let Ok(json) = format_logs_as_json(&rows) else {
            panic!("json serialization failed");
        };
        let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&json) else {
            panic!("invalid json output");
        };
        assert_eq!(parsed.as_array().map(Vec::len), Some(2));
        assert_eq!(parsed[0]["pod"], "pod-a");

        let csv = format_logs_as_csv(&rows);
        assert!(csv.starts_with("timestamp,pod,container,message\n"));
        assert!(csv.contains("\"hello, \"\"world\"\"\nline2\""));
        assert!(csv.contains(",pod-b,sidecar,ok\n"));
    }

    #[test]
    fn test_extract_waiting_container_message() {
        let err = kube::Error::Api(ErrorResponse {
            status: "Failure".to_string(),
            message: "container \"app\" in pod \"p\" is waiting to start: ImagePullBackOff"
                .to_string(),
            reason: "BadRequest".to_string(),
            code: 400,
        });
        assert!(extract_waiting_container_message(&err).is_some());

        let other = kube::Error::Api(ErrorResponse {
            status: "Failure".to_string(),
            message: "pods \"p\" not found".to_string(),
            reason: "NotFound".to_string(),
            code: 404,
        });
        assert!(extract_waiting_container_message(&other).is_none());
    }
}
