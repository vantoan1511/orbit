use std::sync::Arc;
use serde_json::Value;
use tokio::sync::{Mutex, RwLock};
use crate::ipc::bridge::{Bridge, WsWriter};
use crate::ipc::events::OrbitEvent;
use crate::kubernetes::manager::KubeManager;
use super::utils::get_string;
use super::watchers::spawn_watchers;

pub fn get_clusters(
    writer: Arc<Mutex<WsWriter>>,
    token: String,
    manager: Arc<RwLock<KubeManager>>,
) {
    tokio::spawn(async move {
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
            &OrbitEvent::ActiveClusterChanged { active_cluster_id },
        ).await;
    });
}

pub fn get_user_profile(
    writer: Arc<Mutex<WsWriter>>,
    token: String,
    manager: Arc<RwLock<KubeManager>>,
) {
    tokio::spawn(async move {
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
            let mut w_manager = manager.write().await;
            match w_manager.switch_context(&id).await {
                Ok(()) => {
                    let active_cluster_id = w_manager.active_context.clone();
                    let clusters = w_manager.get_clusters();
                    let profile = w_manager.get_user_profile();
                    let client = w_manager.active_client.clone();

                    if let Some(cancel) = w_manager.watch_cancel.take() {
                        let _ = cancel.send(true);
                    }
                    let (tx, rx) = tokio::sync::watch::channel(false);
                    w_manager.watch_cancel = Some(tx);
                    drop(w_manager);

                    let _ = Bridge::send_event(
                        &writer,
                        &token,
                        &OrbitEvent::ActiveClusterChanged { active_cluster_id },
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
                        spawn_watchers(client, writer.clone(), token.clone(), rx.clone());
                    }
                }
                Err(e) => {
                    log::error!("Error switching cluster: {:?}", e);
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
            let mut w_manager = manager.write().await;
            match w_manager.add_kubeconfig_file(&path).await {
                Ok(()) => {
                    let clusters = w_manager.get_clusters();
                    let active_cluster_id = w_manager.active_context.clone();
                    let client = w_manager.active_client.clone();

                    if let Some(cancel) = w_manager.watch_cancel.take() {
                        let _ = cancel.send(true);
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
                        &OrbitEvent::ActiveClusterChanged { active_cluster_id },
                    ).await;

                    // Spawn watchers and metrics poller for the new cluster.
                    if let Some(ref client) = client {
                        spawn_watchers(client, writer.clone(), token.clone(), rx.clone());
                    }
                }
                Err(e) => {
                    log::error!("Error adding cluster: {:?}", e);
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
