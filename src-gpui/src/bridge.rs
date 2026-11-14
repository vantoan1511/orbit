use std::sync::Arc;

use orbit_core::kubernetes::manager::KubeManager;
use tokio::sync::{RwLock, mpsc};

use crate::state::{ClusterBridgeCommand, ClusterBridgeEvent};

pub struct ClusterBridge {
    pub command_tx: mpsc::UnboundedSender<ClusterBridgeCommand>,
    pub event_rx: mpsc::UnboundedReceiver<ClusterBridgeEvent>,
}

pub async fn handle_bridge_command(
    cmd: ClusterBridgeCommand,
    manager: &Arc<RwLock<KubeManager>>,
    event_tx: &mpsc::UnboundedSender<ClusterBridgeEvent>,
) {
    match cmd {
        ClusterBridgeCommand::LoadContexts | ClusterBridgeCommand::RefreshClusters => {
            let guard = manager.read().await;
            let clusters = guard.get_clusters();
            let kubeconfig_error = guard.kubeconfig_load_error().map(ToString::to_string);
            let _ = event_tx.send(ClusterBridgeEvent::ContextsLoaded {
                clusters,
                kubeconfig_error,
            });
        }
        ClusterBridgeCommand::SwitchCluster { cluster_id } => {
            let mut guard = manager.write().await;
            let result = guard.switch_context_with_status(&cluster_id).await;
            let clusters = guard.get_clusters();
            match result {
                Ok(outcome) if outcome.healthy => {
                    let profile = guard.get_user_profile();
                    let _ = event_tx.send(ClusterBridgeEvent::ClusterConnected {
                        outcome,
                        clusters,
                        profile,
                    });
                }
                Ok(outcome) => {
                    let profile = guard.get_user_profile();
                    let _ = event_tx.send(ClusterBridgeEvent::ClusterUnreachable {
                        outcome,
                        clusters,
                        profile,
                    });
                }
                Err(message) => {
                    let _ = event_tx.send(ClusterBridgeEvent::ConnectionError {
                        cluster_id: Some(cluster_id),
                        message,
                        clusters,
                    });
                }
            }
        }
        ClusterBridgeCommand::ReconnectActiveCluster => {
            let mut guard = manager.write().await;
            let active_id = guard.active_context.clone();
            let result = guard.reconnect_active_cluster().await;
            let clusters = guard.get_clusters();
            match result {
                Ok(outcome) if outcome.healthy => {
                    let profile = guard.get_user_profile();
                    let _ = event_tx.send(ClusterBridgeEvent::ClusterConnected {
                        outcome,
                        clusters,
                        profile,
                    });
                }
                Ok(outcome) => {
                    let profile = guard.get_user_profile();
                    let _ = event_tx.send(ClusterBridgeEvent::ClusterUnreachable {
                        outcome,
                        clusters,
                        profile,
                    });
                }
                Err(message) => {
                    let _ = event_tx.send(ClusterBridgeEvent::ConnectionError {
                        cluster_id: active_id,
                        message,
                        clusters,
                    });
                }
            }
        }
    }
}

pub fn start_cluster_bridge() -> ClusterBridge {
    let (command_tx, mut command_rx) = mpsc::unbounded_channel::<ClusterBridgeCommand>();
    let (event_tx, event_rx) = mpsc::unbounded_channel::<ClusterBridgeEvent>();
    let thread_event_tx = event_tx.clone();

    let spawn_res = std::thread::Builder::new()
        .name("orbit-gpui-kube-bridge".to_string())
        .spawn(move || {
            let rt = match tokio::runtime::Builder::new_multi_thread()
                .enable_all()
                .build()
            {
                Ok(rt) => rt,
                Err(err) => {
                    tracing::error!(error = %err, "Failed to create Tokio runtime for GPUI bridge");
                    let _ = thread_event_tx.send(ClusterBridgeEvent::ConnectionError {
                        cluster_id: None,
                        message: format!("Failed to start background Tokio runtime: {err}"),
                        clusters: Vec::new(),
                    });
                    return;
                }
            };

            rt.block_on(async move {
                let manager = Arc::new(RwLock::new(KubeManager::new().await));
                handle_bridge_command(
                    ClusterBridgeCommand::LoadContexts,
                    &manager,
                    &thread_event_tx,
                )
                .await;

                while let Some(cmd) = command_rx.recv().await {
                    handle_bridge_command(cmd, &manager, &thread_event_tx).await;
                }
            });
        });

    if let Err(err) = spawn_res {
        tracing::error!(error = %err, "Failed to spawn GPUI kube bridge thread");
        let _ = event_tx.send(ClusterBridgeEvent::ConnectionError {
            cluster_id: None,
            message: format!("Failed to spawn background bridge thread: {err}"),
            clusters: Vec::new(),
        });
    }

    ClusterBridge {
        command_tx,
        event_rx,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use kube::config::Kubeconfig;
    use orbit_core::config::OrbitConfig;

    fn sample_kubeconfig() -> Kubeconfig {
        let yaml = r#"
apiVersion: v1
kind: Config
clusters:
- name: dev-cluster
  cluster:
    server: http://127.0.0.1:1
- name: prod-cluster
  cluster:
    server: https://10.0.0.10:6443
contexts:
- name: dev-ctx
  context:
    cluster: dev-cluster
    user: dev-user
- name: prod-ctx
  context:
    cluster: prod-cluster
    user: prod-user
users:
- name: dev-user
  user: {}
- name: prod-user
  user: {}
"#;
        Kubeconfig::from_yaml(yaml).expect("valid sample kubeconfig")
    }

    #[tokio::test]
    async fn test_handle_bridge_command_loads_contexts_and_reports_missing_cluster_error() {
        let manager = Arc::new(RwLock::new(KubeManager::new_from_kubeconfig(
            Some(sample_kubeconfig()),
            OrbitConfig::default(),
        )));
        let (event_tx, mut event_rx) = mpsc::unbounded_channel();

        handle_bridge_command(ClusterBridgeCommand::RefreshClusters, &manager, &event_tx).await;
        let loaded_event = event_rx
            .recv()
            .await
            .expect("Expected ContextsLoaded event");
        match loaded_event {
            ClusterBridgeEvent::ContextsLoaded {
                clusters,
                kubeconfig_error,
            } => {
                assert!(kubeconfig_error.is_none());
                let names: Vec<&str> = clusters.iter().map(|c| c.name.as_str()).collect();
                assert_eq!(names, vec!["dev-ctx", "prod-ctx"]);
            }
            other => panic!("Expected ContextsLoaded, got {other:?}"),
        }

        handle_bridge_command(
            ClusterBridgeCommand::SwitchCluster {
                cluster_id: "missing-ctx".to_string(),
            },
            &manager,
            &event_tx,
        )
        .await;
        let err_event = event_rx
            .recv()
            .await
            .expect("Expected ConnectionError event");
        match err_event {
            ClusterBridgeEvent::ConnectionError {
                cluster_id,
                message,
                ..
            } => {
                assert_eq!(cluster_id.as_deref(), Some("missing-ctx"));
                assert!(
                    message.to_lowercase().contains("not found"),
                    "Expected 'not found' in error message: {message}"
                );
            }
            other => panic!("Expected ConnectionError, got {other:?}"),
        }
    }
}
