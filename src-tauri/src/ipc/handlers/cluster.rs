use std::sync::Arc;
use serde_json::Value;
use tokio::sync::{Mutex, RwLock};
use crate::config::OrbitConfig;
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

                    // Cancel active pod terminals for previous cluster
                    super::terminal::close_all_pod_terminals().await;

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
        let kubeconfig_content = get_string(&data, "kubeconfigContent");

        if file_path.is_none() && kubeconfig_content.is_none() {
            let _ = Bridge::send_event(
                &writer,
                &token,
                &OrbitEvent::ErrorOccurred {
                    message: "No kubeconfig file path or content provided".to_string(),
                },
            ).await;
            return;
        }

        let mut w_manager = manager.write().await;
        let add_result = if let Some(ref content) = kubeconfig_content {
            tracing::info!("Adding cluster from kubeconfig content");
            w_manager.add_kubeconfig_content(content).await.map(|_| ())
        } else if let Some(ref path) = file_path {
            tracing::info!(file_path = %path, "Adding cluster from kubeconfig file");
            w_manager.add_kubeconfig_file(path).await
        } else {
            Ok(())
        };

        match add_result {
            Ok(()) => {
                let selected_contexts = data
                    .as_ref()
                    .and_then(|d| d.get("selectedContexts"))
                    .and_then(|v| v.as_array())
                    .map(|arr| {
                        arr.iter()
                            .filter_map(|t| t.as_str().map(|s| s.to_string()))
                            .collect::<Vec<String>>()
                    });

                // Apply optional metadata customization
                if let Some(meta_val) = data.as_ref().and_then(|d| d.get("metadata")) {
                    let alias = meta_val.get("alias").and_then(|v| v.as_str()).map(|s| s.to_string());
                    let color = meta_val.get("color").and_then(|v| v.as_str()).map(|s| s.to_string());
                    let description = meta_val.get("description").and_then(|v| v.as_str()).map(|s| s.to_string());
                    let tags = meta_val.get("tags")
                        .and_then(|v| v.as_array())
                        .map(|arr| {
                            arr.iter()
                                .filter_map(|t| t.as_str().map(|s| s.trim().to_string()))
                                .filter(|s| !s.is_empty())
                                .collect::<Vec<String>>()
                        })
                        .unwrap_or_default();

                    let customization = crate::kubernetes::models::ClusterCustomization {
                        alias,
                        tags,
                        color,
                        description,
                    };

                    let target_id = selected_contexts
                        .as_ref()
                        .and_then(|sc| sc.first().cloned());

                    if let Some(id) = target_id {
                        w_manager.config.set_cluster_customization(&id, customization);
                        if let Err(e) = w_manager.config.save() {
                            tracing::warn!(error = %e, "Failed to save cluster metadata during add_cluster");
                        }
                    }
                }

                // Handle auto-switch context
                let switch_context = data
                    .as_ref()
                    .and_then(|d| d.get("switchContext"))
                    .and_then(|v| v.as_bool())
                    .unwrap_or(true);

                if switch_context {
                    let target = selected_contexts.as_ref().and_then(|sc| sc.first());
                    if let Some(target_ctx) = target.filter(|ctx| w_manager.active_context.as_deref() != Some(*ctx)) {
                        let switch_res = w_manager.switch_context(target_ctx).await;
                        if let Err(e) = switch_res {
                            tracing::warn!(context = %target_ctx, error = %e, "Failed to auto-switch to added cluster context");
                        }
                    }
                }

                tracing::info!("Cluster added successfully");
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
                tracing::error!(error = %e, "Failed to add cluster");
                let _ = Bridge::send_event(
                    &writer,
                    &token,
                    &OrbitEvent::ErrorOccurred {
                        message: format!("Failed to add cluster: {}", e),
                    },
                ).await;
            }
        }
    });
}

pub fn update_cluster_metadata(
    data: Option<Value>,
    writer: Arc<Mutex<WsWriter>>,
    token: String,
    manager: Arc<RwLock<KubeManager>>,
) {
    tokio::spawn(async move {
        let cluster_id = get_string(&data, "id");
        if let Some(id) = cluster_id {
            tracing::info!(cluster_id = %id, "Updating cluster metadata");

            let alias = get_string(&data, "alias");
            let color = get_string(&data, "color");
            let description = get_string(&data, "description");
            let tags = data
                .as_ref()
                .and_then(|d| d.get("tags"))
                .and_then(|v| v.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|t| t.as_str().map(|s| s.trim().to_string()))
                        .filter(|s| !s.is_empty())
                        .collect::<Vec<String>>()
                })
                .unwrap_or_default();

            let customization = crate::kubernetes::models::ClusterCustomization {
                alias,
                tags,
                color,
                description,
            };

            let mut current_config = OrbitConfig::load();
            current_config.set_cluster_customization(&id, customization);
            if let Err(e) = current_config.save() {
                tracing::error!(error = %e, "Failed to save cluster metadata to config");
                let _ = Bridge::send_event(
                    &writer,
                    &token,
                    &OrbitEvent::ErrorOccurred {
                        message: format!("Failed to save cluster metadata: {}", e),
                    },
                ).await;
                return;
            }

            let mut w_manager = manager.write().await;
            w_manager.config = current_config.clone();
            let clusters = w_manager.get_clusters();
            let settings = current_config.get_configurations();
            drop(w_manager);

            let _ = Bridge::send_event(
                &writer,
                &token,
                &OrbitEvent::ClustersUpdated { clusters },
            ).await;

            let _ = Bridge::send_event(
                &writer,
                &token,
                &OrbitEvent::AppSettingsUpdated { settings },
            ).await;
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

    #[tokio::test]
    async fn test_update_cluster_metadata_updates_config_and_emits_event() {
        let _guard = crate::ipc::emitter::TEST_EMITTER_LOCK.lock().unwrap();
        let _config_guard = crate::config::TestConfigDirGuard::new("cluster_metadata");
        let mock_emitter = Arc::new(MockEventEmitter::new());
        set_global_emitter(mock_emitter.clone());

        let writer = Arc::new(Mutex::new(WsWriter::Emitter));
        let manager = Arc::new(RwLock::new(KubeManager::new().await));

        let payload = serde_json::json!({
            "id": "test-ctx",
            "alias": "Dev Cluster",
            "tags": ["dev", "k8s"],
            "color": "#10b981",
            "description": "Local dev test"
        });

        update_cluster_metadata(Some(payload), writer, "token-123".to_string(), manager.clone());

        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        let r_manager = manager.read().await;
        let map = r_manager.config.cluster_customizations();
        assert_eq!(map.get("test-ctx").and_then(|c| c.alias.as_deref()), Some("Dev Cluster"));
        assert_eq!(map.get("test-ctx").and_then(|c| c.color.as_deref()), Some("#10b981"));

        let emitted = mock_emitter.get_emitted_events().await;
        let event_names: Vec<String> = emitted.iter().map(|(name, _)| name.clone()).collect();
        assert!(event_names.contains(&"clustersUpdated".to_string()));
    }

    #[tokio::test]
    async fn test_update_cluster_metadata_preserves_existing_settings() {
        let _guard = crate::ipc::emitter::TEST_EMITTER_LOCK.lock().unwrap();
        let _config_guard = crate::config::TestConfigDirGuard::new("preserve_settings");
        let mock_emitter = Arc::new(MockEventEmitter::new());
        set_global_emitter(mock_emitter.clone());

        // Pre-save custom setting to disk (e.g. themeMode = "system")
        let mut initial_config = OrbitConfig::default();
        initial_config.values.insert("themeMode".to_string(), serde_json::json!("system"));
        initial_config.save().expect("initial config should save");

        let writer = Arc::new(Mutex::new(WsWriter::Emitter));
        let manager = Arc::new(RwLock::new(KubeManager::new().await));
        // Simulate manager having a stale config before reload
        {
            let mut w = manager.write().await;
            w.config = OrbitConfig::default(); // default has themeMode = "dark"
        }

        let payload = serde_json::json!({
            "id": "ctx-preserved",
            "alias": "Preserved Cluster",
        });

        update_cluster_metadata(Some(payload), writer, "token-456".to_string(), manager.clone());

        let mut found = false;
        for _ in 0..40 {
            tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
            let emitted = mock_emitter.get_emitted_events().await;
            if let Some((_, payload)) = emitted.iter().find(|(name, _)| name == "appSettingsUpdated") {
                if let Some(settings) = payload.get("settings").and_then(|s| s.as_array()) {
                    if settings.iter().any(|c| c.get("key") == Some(&serde_json::json!("themeMode")) && c.get("value") == Some(&serde_json::json!("system"))) {
                        found = true;
                        break;
                    }
                }
            }
        }
        assert!(found, "AppSettingsUpdated must preserve themeMode 'system'");

        let r_manager = manager.read().await;
        assert_eq!(r_manager.config.theme_mode(), "system");
        assert_eq!(
            r_manager.config.cluster_customizations().get("ctx-preserved").and_then(|c| c.alias.as_deref()),
            Some("Preserved Cluster")
        );
    }
}

