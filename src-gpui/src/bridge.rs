use std::sync::Arc;

use orbit_core::kubernetes::events::fetch_events;
use orbit_core::kubernetes::manager::KubeManager;
use orbit_core::kubernetes::namespaces::fetch_namespaces;
use orbit_core::kubernetes::nodes::fetch_nodes;
use orbit_core::kubernetes::overview::fetch_cluster_overview;
use orbit_core::kubernetes::workloads::{fetch_deployments, fetch_pods};
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
        ClusterBridgeCommand::LoadOverview {
            cluster_id,
            generation,
        } => {
            let (client, k8s_version) = {
                let guard = manager.read().await;
                let client = guard.active_client_for_context(&cluster_id);
                let k8s_version = guard.get_user_profile().k8s_version;
                (client, k8s_version)
            };

            if let Some(client) = client {
                let tx = event_tx.clone();
                tokio::spawn(async move {
                    let summary = fetch_cluster_overview(&client, k8s_version).await;
                    let _ = tx.send(ClusterBridgeEvent::OverviewLoaded {
                        cluster_id,
                        generation,
                        summary,
                    });
                });
            } else {
                let _ = event_tx.send(ClusterBridgeEvent::OverviewLoadFailed {
                    cluster_id: cluster_id.clone(),
                    generation,
                    message: format!("No active Kubernetes client for cluster '{cluster_id}'"),
                });
            }
        }
        ClusterBridgeCommand::LoadNamespaces {
            cluster_id,
            generation,
        } => {
            let client = {
                let guard = manager.read().await;
                guard.active_client_for_context(&cluster_id)
            };

            if let Some(client) = client {
                let tx = event_tx.clone();
                tokio::spawn(async move {
                    match fetch_namespaces(&client).await {
                        Ok(namespaces) => {
                            let _ = tx.send(ClusterBridgeEvent::NamespacesLoaded {
                                cluster_id,
                                generation,
                                namespaces,
                            });
                        }
                        Err(message) => {
                            let _ = tx.send(ClusterBridgeEvent::NamespacesLoadFailed {
                                cluster_id,
                                generation,
                                message,
                            });
                        }
                    }
                });
            } else {
                let _ = event_tx.send(ClusterBridgeEvent::NamespacesLoadFailed {
                    cluster_id: cluster_id.clone(),
                    generation,
                    message: format!("No active Kubernetes client for cluster '{cluster_id}'"),
                });
            }
        }
        ClusterBridgeCommand::LoadDeployments {
            cluster_id,
            generation,
        } => {
            let client = {
                let guard = manager.read().await;
                guard.active_client_for_context(&cluster_id)
            };

            if let Some(client) = client {
                let tx = event_tx.clone();
                tokio::spawn(async move {
                    match fetch_deployments(&client, None).await {
                        Ok(deployments) => {
                            let _ = tx.send(ClusterBridgeEvent::DeploymentsLoaded {
                                cluster_id,
                                generation,
                                deployments,
                            });
                        }
                        Err(message) => {
                            let _ = tx.send(ClusterBridgeEvent::DeploymentsLoadFailed {
                                cluster_id,
                                generation,
                                message,
                            });
                        }
                    }
                });
            } else {
                let _ = event_tx.send(ClusterBridgeEvent::DeploymentsLoadFailed {
                    cluster_id: cluster_id.clone(),
                    generation,
                    message: format!("No active Kubernetes client for cluster '{cluster_id}'"),
                });
            }
        }
        ClusterBridgeCommand::LoadPods {
            cluster_id,
            generation,
        } => {
            let client = {
                let guard = manager.read().await;
                guard.active_client_for_context(&cluster_id)
            };

            if let Some(client) = client {
                let tx = event_tx.clone();
                tokio::spawn(async move {
                    match fetch_pods(&client, None).await {
                        Ok(pods) => {
                            let _ = tx.send(ClusterBridgeEvent::PodsLoaded {
                                cluster_id,
                                generation,
                                pods,
                            });
                        }
                        Err(message) => {
                            let _ = tx.send(ClusterBridgeEvent::PodsLoadFailed {
                                cluster_id,
                                generation,
                                message,
                            });
                        }
                    }
                });
            } else {
                let _ = event_tx.send(ClusterBridgeEvent::PodsLoadFailed {
                    cluster_id: cluster_id.clone(),
                    generation,
                    message: format!("No active Kubernetes client for cluster '{cluster_id}'"),
                });
            }
        }
        ClusterBridgeCommand::LoadNodes {
            cluster_id,
            generation,
        } => {
            let client = {
                let guard = manager.read().await;
                guard.active_client_for_context(&cluster_id)
            };

            if let Some(client) = client {
                let tx = event_tx.clone();
                tokio::spawn(async move {
                    match fetch_nodes(&client).await {
                        Ok(nodes) => {
                            let _ = tx.send(ClusterBridgeEvent::NodesLoaded {
                                cluster_id,
                                generation,
                                nodes,
                            });
                        }
                        Err(message) => {
                            let _ = tx.send(ClusterBridgeEvent::NodesLoadFailed {
                                cluster_id,
                                generation,
                                message,
                            });
                        }
                    }
                });
            } else {
                let _ = event_tx.send(ClusterBridgeEvent::NodesLoadFailed {
                    cluster_id: cluster_id.clone(),
                    generation,
                    message: format!("No active Kubernetes client for cluster '{cluster_id}'"),
                });
            }
        }
        ClusterBridgeCommand::LoadEvents {
            cluster_id,
            generation,
        } => {
            let client = {
                let guard = manager.read().await;
                guard.active_client_for_context(&cluster_id)
            };

            if let Some(client) = client {
                let tx = event_tx.clone();
                tokio::spawn(async move {
                    match fetch_events(&client, None).await {
                        Ok(events) => {
                            let _ = tx.send(ClusterBridgeEvent::EventsLoaded {
                                cluster_id,
                                generation,
                                events,
                            });
                        }
                        Err(message) => {
                            let _ = tx.send(ClusterBridgeEvent::EventsLoadFailed {
                                cluster_id,
                                generation,
                                message,
                            });
                        }
                    }
                });
            } else {
                let _ = event_tx.send(ClusterBridgeEvent::EventsLoadFailed {
                    cluster_id: cluster_id.clone(),
                    generation,
                    message: format!("No active Kubernetes client for cluster '{cluster_id}'"),
                });
            }
        }
        ClusterBridgeCommand::LoadResourceYaml {
            cluster_id,
            generation,
            kind,
            namespace,
            name,
        } => {
            let client = {
                let guard = manager.read().await;
                guard.active_client_for_context(&cluster_id)
            };

            if let Some(client) = client {
                let tx = event_tx.clone();
                tokio::spawn(async move {
                    match orbit_core::kubernetes::raw::fetch_resource_yaml(
                        &client,
                        namespace.as_deref(),
                        &kind,
                        &name,
                    )
                    .await
                    {
                        Ok(yaml) => {
                            let _ = tx.send(ClusterBridgeEvent::ResourceYamlLoaded {
                                cluster_id,
                                generation,
                                kind,
                                namespace,
                                name,
                                yaml,
                            });
                        }
                        Err(message) => {
                            let _ = tx.send(ClusterBridgeEvent::ResourceYamlLoadFailed {
                                cluster_id,
                                generation,
                                kind,
                                namespace,
                                name,
                                message,
                            });
                        }
                    }
                });
            } else {
                let _ = event_tx.send(ClusterBridgeEvent::ResourceYamlLoadFailed {
                    cluster_id: cluster_id.clone(),
                    generation,
                    kind,
                    namespace,
                    name,
                    message: format!("No active Kubernetes client for cluster '{cluster_id}'"),
                });
            }
        }
        ClusterBridgeCommand::SaveNamespaceSelection {
            cluster_id,
            namespaces,
        } => {
            let mut guard = manager.write().await;
            if let Err(err) = guard
                .config
                .set_selected_namespaces_for_cluster(&cluster_id, namespaces)
            {
                tracing::warn!(
                    cluster_id = %cluster_id,
                    error = %err,
                    "Failed to persist selected namespaces"
                );
            }
        }
        ClusterBridgeCommand::SaveWorkspaceTabs {
            cluster_id,
            open_tabs,
            active_tab,
        } => {
            let mut guard = manager.write().await;
            if let Err(err) = guard
                .config
                .set_workspace_tabs_for_cluster(&cluster_id, open_tabs, active_tab)
            {
                tracing::warn!(
                    cluster_id = %cluster_id,
                    error = %err,
                    "Failed to persist workspace tabs"
                );
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

    #[tokio::test]
    async fn test_handle_bridge_command_load_overview_without_active_client_emits_failure() {
        let manager = Arc::new(RwLock::new(KubeManager::new_empty(OrbitConfig::default())));
        let (event_tx, mut event_rx) = mpsc::unbounded_channel();

        handle_bridge_command(
            ClusterBridgeCommand::LoadOverview {
                cluster_id: "missing-ctx".to_string(),
                generation: 7,
            },
            &manager,
            &event_tx,
        )
        .await;

        let event = event_rx
            .recv()
            .await
            .expect("Expected OverviewLoadFailed event");
        match event {
            ClusterBridgeEvent::OverviewLoadFailed {
                cluster_id,
                generation,
                message,
            } => {
                assert_eq!(cluster_id, "missing-ctx");
                assert_eq!(generation, 7);
                assert!(!message.trim().is_empty());
            }
            other => panic!("Expected OverviewLoadFailed, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn test_handle_bridge_command_load_namespaces_without_active_client_emits_failure() {
        let manager = Arc::new(RwLock::new(KubeManager::new_empty(OrbitConfig::default())));
        let (event_tx, mut event_rx) = mpsc::unbounded_channel();

        handle_bridge_command(
            ClusterBridgeCommand::LoadNamespaces {
                cluster_id: "missing-ctx".to_string(),
                generation: 5,
            },
            &manager,
            &event_tx,
        )
        .await;

        let event = event_rx
            .recv()
            .await
            .expect("Expected NamespacesLoadFailed event");
        match event {
            ClusterBridgeEvent::NamespacesLoadFailed {
                cluster_id,
                generation,
                message,
            } => {
                assert_eq!(cluster_id, "missing-ctx");
                assert_eq!(generation, 5);
                assert!(!message.trim().is_empty());
            }
            other => panic!("Expected NamespacesLoadFailed, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn test_handle_bridge_command_save_namespace_selection_persists_to_config() {
        let _guard = orbit_core::config::TestConfigDirGuard::new("gpui_ns_bridge");
        let manager = Arc::new(RwLock::new(KubeManager::new_empty(OrbitConfig::load())));
        let (event_tx, _event_rx) = mpsc::unbounded_channel();

        handle_bridge_command(
            ClusterBridgeCommand::SaveNamespaceSelection {
                cluster_id: "ctx-a".to_string(),
                namespaces: vec!["payments".to_string()],
            },
            &manager,
            &event_tx,
        )
        .await;

        let reloaded = OrbitConfig::load();
        assert_eq!(
            reloaded.selected_namespaces_for_cluster("ctx-a"),
            vec!["payments".to_string()]
        );
    }

    #[tokio::test]
    async fn test_handle_bridge_command_save_workspace_tabs_persists_to_config() {
        let _guard = orbit_core::config::TestConfigDirGuard::new("gpui_tabs_bridge");
        let manager = Arc::new(RwLock::new(KubeManager::new_empty(OrbitConfig::load())));
        let (event_tx, _event_rx) = mpsc::unbounded_channel();

        handle_bridge_command(
            ClusterBridgeCommand::SaveWorkspaceTabs {
                cluster_id: "ctx-a".to_string(),
                open_tabs: vec![
                    "core-overview".to_string(),
                    "workloads-deployments".to_string(),
                ],
                active_tab: Some("workloads-deployments".to_string()),
            },
            &manager,
            &event_tx,
        )
        .await;

        let reloaded = OrbitConfig::load();
        assert_eq!(
            reloaded.workspace_tabs_for_cluster("ctx-a"),
            Some(orbit_core::config::PersistedClusterTabs {
                open_tabs: vec![
                    "core-overview".to_string(),
                    "workloads-deployments".to_string(),
                ],
                active_tab: Some("workloads-deployments".to_string()),
            })
        );
    }

    #[tokio::test]
    async fn test_handle_bridge_command_load_deployments_without_client_emits_failure() {
        let manager = Arc::new(RwLock::new(KubeManager::new_empty(OrbitConfig::default())));
        let (event_tx, mut event_rx) = mpsc::unbounded_channel();

        handle_bridge_command(
            ClusterBridgeCommand::LoadDeployments {
                cluster_id: "missing-ctx".to_string(),
                generation: 9,
            },
            &manager,
            &event_tx,
        )
        .await;

        let event = event_rx
            .recv()
            .await
            .expect("Expected DeploymentsLoadFailed event");
        match event {
            ClusterBridgeEvent::DeploymentsLoadFailed {
                cluster_id,
                generation,
                message,
            } => {
                assert_eq!(cluster_id, "missing-ctx");
                assert_eq!(generation, 9);
                assert!(!message.trim().is_empty());
            }
            other => panic!("Expected DeploymentsLoadFailed, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn test_handle_bridge_command_load_pods_without_client_emits_failure() {
        let manager = Arc::new(RwLock::new(KubeManager::new_empty(OrbitConfig::default())));
        let (event_tx, mut event_rx) = mpsc::unbounded_channel();

        handle_bridge_command(
            ClusterBridgeCommand::LoadPods {
                cluster_id: "missing-ctx".to_string(),
                generation: 11,
            },
            &manager,
            &event_tx,
        )
        .await;

        let event = event_rx
            .recv()
            .await
            .expect("Expected PodsLoadFailed event");
        match event {
            ClusterBridgeEvent::PodsLoadFailed {
                cluster_id,
                generation,
                message,
            } => {
                assert_eq!(cluster_id, "missing-ctx");
                assert_eq!(generation, 11);
                assert!(!message.trim().is_empty());
            }
            other => panic!("Expected PodsLoadFailed, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn test_handle_bridge_command_load_nodes_without_client_emits_failure() {
        let manager = Arc::new(RwLock::new(KubeManager::new_empty(OrbitConfig::default())));
        let (event_tx, mut event_rx) = mpsc::unbounded_channel();

        handle_bridge_command(
            ClusterBridgeCommand::LoadNodes {
                cluster_id: "missing-ctx".to_string(),
                generation: 7,
            },
            &manager,
            &event_tx,
        )
        .await;

        let event = event_rx
            .recv()
            .await
            .expect("Expected NodesLoadFailed event");
        match event {
            ClusterBridgeEvent::NodesLoadFailed {
                cluster_id,
                generation,
                message,
            } => {
                assert_eq!(cluster_id, "missing-ctx");
                assert_eq!(generation, 7);
                assert!(!message.trim().is_empty());
            }
            other => panic!("Expected NodesLoadFailed, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn test_handle_bridge_command_load_resource_yaml_without_client_emits_failure() {
        let manager = Arc::new(RwLock::new(KubeManager::new_empty(OrbitConfig::default())));
        let (event_tx, mut event_rx) = mpsc::unbounded_channel();

        handle_bridge_command(
            ClusterBridgeCommand::LoadResourceYaml {
                cluster_id: "missing-ctx".to_string(),
                generation: 4,
                kind: "Deployment".to_string(),
                namespace: Some("default".to_string()),
                name: "web".to_string(),
            },
            &manager,
            &event_tx,
        )
        .await;

        let event = event_rx
            .recv()
            .await
            .expect("Expected ResourceYamlLoadFailed event");
        match event {
            ClusterBridgeEvent::ResourceYamlLoadFailed {
                cluster_id,
                generation,
                kind,
                namespace,
                name,
                message,
            } => {
                assert_eq!(cluster_id, "missing-ctx");
                assert_eq!(generation, 4);
                assert_eq!(kind, "Deployment");
                assert_eq!(namespace.as_deref(), Some("default"));
                assert_eq!(name, "web");
                assert!(!message.trim().is_empty());
            }
            other => panic!("Expected ResourceYamlLoadFailed, got {other:?}"),
        }
    }
}
