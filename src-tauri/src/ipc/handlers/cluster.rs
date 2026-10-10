use std::sync::Arc;
use serde_json::Value;
use tokio::sync::{Mutex, RwLock};
use crate::ipc::bridge::{Bridge, WsWriter};
use crate::ipc::events::OrbitEvent;
use crate::kubernetes::manager::KubeManager;
use super::network;
use super::utils::get_string;
use super::watchers::spawn_watchers;

pub fn get_clusters(
    writer: Arc<Mutex<WsWriter>>,
    token: String,
    manager: Arc<RwLock<KubeManager>>,
) {
    tokio::spawn(async move {
        tracing::debug!("Refreshing active cluster health and fetching clusters");
        {
            let mut w_manager = manager.write().await;
            w_manager.refresh_active_cluster_health().await;
        }
        let r_manager = manager.read().await;
        let clusters = r_manager.get_clusters();
        let active_cluster_id = r_manager.active_context.clone();

        let _ = Bridge::send_event(
            &writer,
            &token,
            &OrbitEvent::ClustersUpdated { clusters },
        ).await;

        let _ = Bridge::send_event(
            &writer,
            &token,
            &OrbitEvent::ActiveClusterChanged { active_cluster_id: active_cluster_id.clone() },
        ).await;
    });
}

pub fn get_user_profile(
    writer: Arc<Mutex<WsWriter>>,
    token: String,
    manager: Arc<RwLock<KubeManager>>,
) {
    tokio::spawn(async move {
        tracing::debug!("Fetching user profile");
        let r_manager = manager.read().await;
        let profile = r_manager.get_user_profile();
        let _ = Bridge::send_event(
            &writer,
            &token,
            &OrbitEvent::UserProfileUpdated { profile },
        ).await;
    });
}

pub fn switch_cluster(
    data: Option<Value>,
    writer: Arc<Mutex<WsWriter>>,
    token: String,
    manager: Arc<RwLock<KubeManager>>,
) {
    tokio::spawn(async move {
        let cluster_id = get_string(&data, "clusterId");

        if let Some(id) = cluster_id {
            tracing::info!(cluster_id = %id, "Switching cluster");
            let mut w_manager = manager.write().await;
            match w_manager.switch_context(&id).await {
                Ok(()) => {
                    tracing::info!(cluster_id = %id, "Cluster switch completed");
                    let active_cluster_id = w_manager.active_context.clone();
                    let clusters = w_manager.get_clusters();
                    let profile = w_manager.get_user_profile();
                    let client = w_manager.active_client.clone();

                    if let Some(cancel) = w_manager.watch_cancel.take() {
                        let _ = cancel.send(true);
                    }
                    for cancel in w_manager.log_cancel.drain(..) {
                        let _ = cancel.send(());
                    }
                    let (tx, rx) = tokio::sync::watch::channel(false);
                    w_manager.watch_cancel = Some(tx);
                    drop(w_manager);

                    let _ = Bridge::send_event(
                        &writer,
                        &token,
                        &OrbitEvent::ActiveClusterChanged { active_cluster_id: active_cluster_id.clone() },
                    ).await;

                    let _ = Bridge::send_event(
                        &writer,
                        &token,
                        &OrbitEvent::ClustersUpdated { clusters },
                    ).await;

                    let _ = Bridge::send_event(
                        &writer,
                        &token,
                        &OrbitEvent::UserProfileUpdated { profile },
                    ).await;

                    // Spawn watchers and metrics poller for the new cluster.
                    if let Some(ref client) = client {
                        spawn_watchers(client, writer.clone(), token.clone(), rx.clone(), active_cluster_id.clone());
                    }

                    // Stop previous active forwards and restore persisted forwards for the new cluster
                    network::stop_all_active_port_forwards(&manager).await;
                    network::restore_cluster_port_forwards(
                        writer.clone(),
                        token.clone(),
                        manager.clone(),
                        id.clone(),
                    );
                }
                Err(e) => {
                    tracing::error!(cluster_id = %id, error = %e, "Cluster switch failed");
                    let _ = Bridge::send_event(
                        &writer,
                        &token,
                        &OrbitEvent::ErrorOccurred {
                            message: format!("Failed to switch cluster: {}", e),
                        },
                    ).await;
                }
            }
        }
    });
}

pub fn add_cluster(
    data: Option<Value>,
    writer: Arc<Mutex<WsWriter>>,
    token: String,
    manager: Arc<RwLock<KubeManager>>,
) {
    tokio::spawn(async move {
        let file_path = get_string(&data, "filePath");

        if let Some(path) = file_path {
            tracing::info!(file_path = %path, "Adding cluster kubeconfig");
            let mut w_manager = manager.write().await;
            match w_manager.add_kubeconfig_file(&path).await {
                Ok(()) => {
                    tracing::info!(file_path = %path, "Cluster added successfully");
                    let clusters = w_manager.get_clusters();
                    let active_cluster_id = w_manager.active_context.clone();
                    let client = w_manager.active_client.clone();

                    if let Some(cancel) = w_manager.watch_cancel.take() {
                        let _ = cancel.send(true);
                    }
                    for cancel in w_manager.log_cancel.drain(..) {
                        let _ = cancel.send(());
                    }
                    let (tx, rx) = tokio::sync::watch::channel(false);
                    w_manager.watch_cancel = Some(tx);
                    drop(w_manager);

                    let _ = Bridge::send_event(
                        &writer,
                        &token,
                        &OrbitEvent::ClustersUpdated { clusters },
                    ).await;

                    let _ = Bridge::send_event(
                        &writer,
                        &token,
                        &OrbitEvent::ActiveClusterChanged { active_cluster_id: active_cluster_id.clone() },
                    ).await;

                    // Spawn watchers and metrics poller for the new cluster.
                    if let Some(ref client) = client {
                        spawn_watchers(client, writer.clone(), token.clone(), rx.clone(), active_cluster_id.clone());
                    }

                    // Stop previous active forwards and restore persisted forwards for the new cluster
                    network::stop_all_active_port_forwards(&manager).await;
                    if let Some(ref id) = active_cluster_id {
                        network::restore_cluster_port_forwards(
                            writer.clone(),
                            token.clone(),
                            manager.clone(),
                            id.clone(),
                        );
                    }
                }
                Err(e) => {
                    tracing::error!(file_path = %path, error = %e, "Failed to add cluster");
                    let _ = Bridge::send_event(
                        &writer,
                        &token,
                        &OrbitEvent::ErrorOccurred {
                            message: format!("Failed to add cluster: {}", e),
                        },
                    ).await;
                }
            }
        }
    });
}

pub async fn restart_watchers(
    writer: &Arc<Mutex<WsWriter>>,
    token: &str,
    manager: &Arc<RwLock<KubeManager>>,
    client: &kube::Client,
    active_cluster_id: Option<String>,
) {
    let mut w_manager = manager.write().await;
    if let Some(cancel) = w_manager.watch_cancel.take() {
        let _ = cancel.send(true);
    }
    let (tx, rx) = tokio::sync::watch::channel(false);
    w_manager.watch_cancel = Some(tx);
    drop(w_manager);

    spawn_watchers(
        client,
        writer.clone(),
        token.to_string(),
        rx,
        active_cluster_id,
    );
}

pub async fn broadcast_engine_ready(
    writer: &Arc<Mutex<WsWriter>>,
    token: &str,
    manager: &Arc<RwLock<KubeManager>>,
) {
    let _ = Bridge::send_event(
        writer,
        token,
        &OrbitEvent::EngineConnected {
            status: "ready".to_string(),
            message: "Orbit Engine is connected and ready.".to_string(),
        },
    ).await;

    let r_manager = manager.read().await;
    let clusters = r_manager.get_clusters();
    let active_cluster_id = r_manager.active_context.clone();
    let active_client = r_manager.active_client.clone();
    drop(r_manager);

    let _ = Bridge::send_event(
        writer,
        token,
        &OrbitEvent::ClustersUpdated { clusters },
    ).await;

    let _ = Bridge::send_event(
        writer,
        token,
        &OrbitEvent::ActiveClusterChanged { active_cluster_id: active_cluster_id.clone() },
    ).await;

    if let Some(ref client) = active_client {
        restart_watchers(writer, token, manager, client, active_cluster_id.clone()).await;
    }
    if let Some(ref ctx) = active_cluster_id {
        network::restore_cluster_port_forwards(writer.clone(), token.to_string(), manager.clone(), ctx.clone());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ipc::emitter::{set_global_emitter, MockEventEmitter};

    #[tokio::test]
    async fn test_drain_log_cancel_signals() {
        let (tx1, mut rx1) = tokio::sync::oneshot::channel();
        let (tx2, mut rx2) = tokio::sync::oneshot::channel();
        let mut log_cancels = vec![tx1, tx2];

        for cancel in log_cancels.drain(..) {
            let _ = cancel.send(());
        }

        assert_eq!(rx1.try_recv(), Ok(()));
        assert_eq!(rx2.try_recv(), Ok(()));
        assert!(log_cancels.is_empty());
    }

    #[tokio::test]
    async fn test_broadcast_engine_ready_emits_expected_events() {
        let _guard = crate::ipc::emitter::TEST_EMITTER_LOCK.lock().unwrap();
        let mock_emitter = Arc::new(MockEventEmitter::new());
        set_global_emitter(mock_emitter.clone());

        let writer = Arc::new(Mutex::new(WsWriter::Emitter));
        let manager = Arc::new(RwLock::new(KubeManager::new().await));

        broadcast_engine_ready(&writer, "test-token", &manager).await;

        let emitted = mock_emitter.get_emitted_events().await;
        let event_names: Vec<String> = emitted.iter().map(|(name, _)| name.clone()).collect();

        assert!(event_names.contains(&"engineConnected".to_string()));
        assert!(event_names.contains(&"clustersUpdated".to_string()));
        assert!(event_names.contains(&"activeClusterChanged".to_string()));
    }
}

