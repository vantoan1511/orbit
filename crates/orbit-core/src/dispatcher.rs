use std::sync::Arc;
use tokio::sync::RwLock;

use crate::command::{AddClusterMode, CoreCommand};
use crate::events::CoreEvent;
use crate::kubernetes::manager::KubeManager;
use crate::sink::CoreEventSink;

pub struct CoreDispatcher {
    manager: Arc<RwLock<KubeManager>>,
    sink: Arc<dyn CoreEventSink>,
}

impl CoreDispatcher {
    pub fn new(manager: Arc<RwLock<KubeManager>>, sink: Arc<dyn CoreEventSink>) -> Self {
        Self { manager, sink }
    }

    pub async fn dispatch(&self, command: CoreCommand) {
        match command {
            CoreCommand::Ping => {
                let _ = self.sink.send(CoreEvent::Pong);
            }
            CoreCommand::Initialize => {
                let mgr = self.manager.read().await;
                let clusters = mgr.get_clusters();
                let active_cluster_id = mgr.active_context.clone();
                let configs = mgr.config.get_configurations();
                let settings = configs.into_iter()
                    .map(|c| (c.key, c.default_value.unwrap_or(serde_json::Value::Null)))
                    .collect();

                let _ = self.sink.send(CoreEvent::EngineReady {
                    status: "ready".to_string(),
                    message: "Orbit Native Core Engine Connected".to_string(),
                    clusters,
                    active_cluster_id,
                    settings,
                });
            }
            CoreCommand::GetAppSettings => {
                let mgr = self.manager.read().await;
                let configs = mgr.config.get_configurations();
                let settings = configs.into_iter()
                    .map(|c| (c.key, c.default_value.unwrap_or(serde_json::Value::Null)))
                    .collect();
                let clusters = mgr.get_clusters();
                let _ = self.sink.send(CoreEvent::EngineReady {
                    status: "ready".to_string(),
                    message: "Settings loaded".to_string(),
                    clusters,
                    active_cluster_id: mgr.active_context.clone(),
                    settings,
                });
            }
            CoreCommand::UpdateAppSettings { values } => {
                let mut mgr = self.manager.write().await;
                for (k, v) in values {
                    mgr.config.values.insert(k, v);
                }
                let _ = mgr.config.save();
                let _ = self.sink.send(CoreEvent::ActionSucceeded {
                    action: "update_app_settings".to_string(),
                    message: "Settings updated successfully".to_string(),
                });
            }
            CoreCommand::SetLaunchOnStartup { enabled } => {
                let mut mgr = self.manager.write().await;
                mgr.config.values.insert("launchOnStartup".to_string(), serde_json::Value::Bool(enabled));
                let _ = mgr.config.save();
                let _ = tokio::task::spawn_blocking(move || {
                    crate::autostart::service().and_then(|s| s.set_enabled(enabled))
                }).await;
                let _ = self.sink.send(CoreEvent::ActionSucceeded {
                    action: "set_launch_on_startup".to_string(),
                    message: format!("Launch on startup set to {}", enabled),
                });
            }
            CoreCommand::GetClusters => {
                let mgr = self.manager.read().await;
                let clusters = mgr.get_clusters();
                let _ = self.sink.send(CoreEvent::ClustersUpdated { clusters });
            }
            CoreCommand::GetUserProfile => {
                let mgr = self.manager.read().await;
                let profile = mgr.get_user_profile();
                let _ = self.sink.send(CoreEvent::UserProfileUpdated { profile });
            }
            CoreCommand::SwitchCluster { context_name } => {
                let mut mgr = self.manager.write().await;
                let connected = if let Some(ref name) = context_name {
                    mgr.switch_context(name).await.is_ok()
                } else {
                    mgr.active_context = None;
                    mgr.active_client = None;
                    false
                };
                let active_id = mgr.active_context.clone();
                let clusters = mgr.get_clusters();
                let _ = self.sink.send(CoreEvent::ClustersUpdated { clusters });
                let _ = self.sink.send(CoreEvent::ActiveClusterChanged {
                    active_cluster_id: active_id.clone(),
                });

                if connected {
                    let client = mgr.active_client.clone();
                    if let Some(ref client) = client {
                        let (cancel_tx, cancel_rx) = tokio::sync::watch::channel(false);
                        mgr.watch_cancel = Some(cancel_tx);
                        crate::kubernetes::watchers::spawn_watchers(
                            client,
                            self.sink.clone(),
                            cancel_rx,
                        );
                    }
                }
            }
            CoreCommand::RestartWatchers => {
                let mut mgr = self.manager.write().await;
                if let Some(ref cancel) = mgr.watch_cancel {
                    let _ = cancel.send(true);
                }
                let client = mgr.active_client.clone();
                if let Some(ref client) = client {
                    let (cancel_tx, cancel_rx) = tokio::sync::watch::channel(false);
                    mgr.watch_cancel = Some(cancel_tx);
                    crate::kubernetes::watchers::spawn_watchers(
                        client,
                        self.sink.clone(),
                        cancel_rx,
                    );
                }
            }
            CoreCommand::AddCluster { mode, request_id } => {
                let mut mgr = self.manager.write().await;
                let result: Result<(), String> = match mode {
                    AddClusterMode::FilePath { path } => mgr.merge_kubeconfig_from_path(&path),
                    AddClusterMode::RawYaml { yaml } => mgr.add_kubeconfig_content(&yaml).await.map(|_| ()),
                    AddClusterMode::ManualEndpoint { name, server_url, bearer_token, insecure_skip_tls_verify, ca_cert_pem } => {
                        let ca_field = ca_cert_pem.map(|ca| format!("\n    certificate-authority-data: {}", base64::Engine::encode(&base64::engine::general_purpose::STANDARD, ca.as_bytes()))).unwrap_or_default();
                        let insecure_field = if insecure_skip_tls_verify { "\n    insecure-skip-tls-verify: true" } else { "" };
                        let synthetic_yaml = format!(
r#"apiVersion: v1
kind: Config
clusters:
- cluster:
    server: {}{}{}
  name: {}
contexts:
- context:
    cluster: {}
    user: {}
  name: {}
current-context: {}
users:
- name: {}
  user:
    token: {}
"#,
                            server_url, ca_field, insecure_field, name, name, name, name, name, name, bearer_token
                        );
                        mgr.add_kubeconfig_content(&synthetic_yaml).await.map(|_| ())
                    }
                };

                match result {
                    Ok(_) => {
                        let clusters = mgr.get_clusters();
                        let _ = self.sink.send(CoreEvent::ClustersUpdated { clusters });
                        let _ = self.sink.send(CoreEvent::ClusterAdded {
                            cluster_id: "cluster-added".to_string(),
                            request_id,
                        });
                    }
                    Err(e) => {
                        let _ = self.sink.send(CoreEvent::ClusterAddFailed {
                            error: e,
                            request_id,
                        });
                    }
                }
            }
            CoreCommand::UpdateClusterMetadata { cluster_id, customization } => {
                let mut mgr = self.manager.write().await;
                mgr.config.set_cluster_customization(&cluster_id, customization);
                let _ = mgr.config.save();
                let clusters = mgr.get_clusters();
                let _ = self.sink.send(CoreEvent::ClustersUpdated { clusters });
            }
            CoreCommand::RefreshResourceList { .. } => {
                let mgr = self.manager.read().await;
                let client = mgr.active_client.clone();
                if let Some(ref client) = client {
                    let (_tx, cancel_rx) = tokio::sync::watch::channel(false);
                    crate::kubernetes::watchers::spawn_watchers(
                        client,
                        self.sink.clone(),
                        cancel_rx,
                    );
                }
            }
            CoreCommand::GetResourceRaw { kind, name, namespace, request_id } => {
                let mgr = self.manager.read().await;
                let client = mgr.active_client.clone();
                if let Some(ref client) = client {
                    let ns = namespace.as_deref().unwrap_or("default");
                    match crate::kubernetes::edit::get_resource_raw(client, ns, &kind, &name).await {
                        Ok(val) => {
                            let yaml = serde_yaml::to_string(&val).unwrap_or_else(|_| val.to_string());
                            let _ = self.sink.send(CoreEvent::RawResourceLoaded {
                                kind,
                                name,
                                namespace,
                                yaml,
                                request_id,
                            });
                        }
                        Err(e) => {
                            let _ = self.sink.send(CoreEvent::ErrorOccurred {
                                action: "get_resource_raw".to_string(),
                                message: e.to_string(),
                            });
                        }
                    }
                }
            }
            CoreCommand::ApplyResource { kind, name, namespace, yaml, request_id } => {
                let mgr = self.manager.read().await;
                let client = mgr.active_client.clone();
                if let Some(ref client) = client {
                    let ns = namespace.as_deref().unwrap_or("default");
                    let val: serde_json::Value = match serde_yaml::from_str(&yaml) {
                        Ok(v) => v,
                        Err(e) => {
                            let _ = self.sink.send(CoreEvent::ErrorOccurred {
                                action: "apply_resource".to_string(),
                                message: format!("Invalid YAML: {}", e),
                            });
                            return;
                        }
                    };
                    match crate::kubernetes::edit::apply_resource(client, ns, &kind, &name, val).await {
                        Ok(_) => {
                            let _ = self.sink.send(CoreEvent::ResourceApplied {
                                kind,
                                name,
                                namespace,
                                request_id,
                            });
                        }
                        Err(e) => {
                            let _ = self.sink.send(CoreEvent::ErrorOccurred {
                                action: "apply_resource".to_string(),
                                message: e.to_string(),
                            });
                        }
                    }
                }
            }
            CoreCommand::CreateResource { yaml, namespace, request_id } => {
                let mgr = self.manager.read().await;
                let client = mgr.active_client.clone();
                if let Some(ref client) = client {
                    let ns = namespace.as_deref().unwrap_or("default");
                    let val: serde_json::Value = match serde_yaml::from_str(&yaml) {
                        Ok(v) => v,
                        Err(e) => {
                            let _ = self.sink.send(CoreEvent::ErrorOccurred {
                                action: "create_resource".to_string(),
                                message: format!("Invalid YAML: {}", e),
                            });
                            return;
                        }
                    };
                    let kind = val.get("kind").and_then(|k| k.as_str()).unwrap_or("Resource").to_string();
                    let name = val.get("metadata").and_then(|m| m.get("name")).and_then(|n| n.as_str()).unwrap_or("new-resource").to_string();
                    match crate::kubernetes::edit::create_resource(client, ns, &kind, val).await {
                        Ok(_) => {
                            let _ = self.sink.send(CoreEvent::ResourceCreated {
                                kind,
                                name,
                                namespace,
                                request_id,
                            });
                        }
                        Err(e) => {
                            let _ = self.sink.send(CoreEvent::ErrorOccurred {
                                action: "create_resource".to_string(),
                                message: e.to_string(),
                            });
                        }
                    }
                }
            }
            CoreCommand::DeleteResource { kind, name, namespace, .. } => {
                let mgr = self.manager.read().await;
                let client = mgr.active_client.clone();
                if let Some(ref client) = client {
                    let ns = namespace.as_deref().unwrap_or("default");
                    match crate::kubernetes::delete::delete_resource(client, ns, &kind, &name).await {
                        Ok(_) => {
                            let _ = self.sink.send(CoreEvent::ActionSucceeded {
                                action: "delete_resource".to_string(),
                                message: format!("Deleted {} {}", kind, name),
                            });
                        }
                        Err(e) => {
                            let _ = self.sink.send(CoreEvent::ErrorOccurred {
                                action: "delete_resource".to_string(),
                                message: e.to_string(),
                            });
                        }
                    }
                }
            }
            CoreCommand::ScaleResource { kind, name, namespace, replicas } => {
                let mgr = self.manager.read().await;
                let client = mgr.active_client.clone();
                if let Some(ref client) = client {
                    match crate::kubernetes::workloads::scale_resource(client, &namespace, &kind, &name, replicas).await {
                        Ok(_) => {
                            let _ = self.sink.send(CoreEvent::ActionSucceeded {
                                action: "scale_resource".to_string(),
                                message: format!("Scaled {} {} to {} replicas", kind, name, replicas),
                            });
                        }
                        Err(e) => {
                            let _ = self.sink.send(CoreEvent::ErrorOccurred {
                                action: "scale_resource".to_string(),
                                message: e.to_string(),
                            });
                        }
                    }
                }
            }
            CoreCommand::UpdateHpaReplicas { name, namespace, min_replicas, max_replicas } => {
                let mgr = self.manager.read().await;
                let client = mgr.active_client.clone();
                if let Some(ref client) = client {
                    let min_r = min_replicas.unwrap_or(1);
                    match crate::kubernetes::hpa::update_hpa_replicas(client, &name, &namespace, min_r, max_replicas).await {
                        Ok(_) => {
                            let _ = self.sink.send(CoreEvent::ActionSucceeded {
                                action: "update_hpa_replicas".to_string(),
                                message: format!("Updated HPA {} replicas", name),
                            });
                        }
                        Err(e) => {
                            let _ = self.sink.send(CoreEvent::ErrorOccurred {
                                action: "update_hpa_replicas".to_string(),
                                message: e.to_string(),
                            });
                        }
                    }
                }
            }
            CoreCommand::UpdateResourceImages { kind, name, namespace, containers } => {
                let mgr = self.manager.read().await;
                let client = mgr.active_client.clone();
                if let Some(ref client) = client {
                    match crate::kubernetes::workloads::update_images_resource(client, &kind, &namespace, &name, containers).await {
                        Ok(_) => {
                            let _ = self.sink.send(CoreEvent::ActionSucceeded {
                                action: "update_resource_images".to_string(),
                                message: format!("Updated images for {} {}", kind, name),
                            });
                        }
                        Err(e) => {
                            let _ = self.sink.send(CoreEvent::ErrorOccurred {
                                action: "update_resource_images".to_string(),
                                message: e.to_string(),
                            });
                        }
                    }
                }
            }
            CoreCommand::RedeployResource { kind, name, namespace } => {
                let mgr = self.manager.read().await;
                let client = mgr.active_client.clone();
                if let Some(ref client) = client {
                    match crate::kubernetes::workloads::redeploy_resource(client, &namespace, &kind, &name).await {
                        Ok(_) => {
                            let _ = self.sink.send(CoreEvent::ActionSucceeded {
                                action: "redeploy_resource".to_string(),
                                message: format!("Triggered rollout restart for {} {}", kind, name),
                            });
                        }
                        Err(e) => {
                            let _ = self.sink.send(CoreEvent::ErrorOccurred {
                                action: "redeploy_resource".to_string(),
                                message: e.to_string(),
                            });
                        }
                    }
                }
            }
            CoreCommand::RestartPod { name, namespace } => {
                let mgr = self.manager.read().await;
                let client = mgr.active_client.clone();
                if let Some(ref client) = client {
                    match crate::kubernetes::workloads::delete_pod(client, &namespace, &name).await {
                        Ok(_) => {
                            let _ = self.sink.send(CoreEvent::ActionSucceeded {
                                action: "restart_pod".to_string(),
                                message: format!("Restarted pod {}", name),
                            });
                        }
                        Err(e) => {
                            let _ = self.sink.send(CoreEvent::ErrorOccurred {
                                action: "restart_pod".to_string(),
                                message: e.to_string(),
                            });
                        }
                    }
                }
            }
            CoreCommand::RollbackDeployment { name, namespace, revision } => {
                let mgr = self.manager.read().await;
                let client = mgr.active_client.clone();
                if let Some(ref client) = client {
                    match crate::kubernetes::workloads::rollback_deployment(client, &name, &namespace, revision).await {
                        Ok(_) => {
                            let _ = self.sink.send(CoreEvent::ActionSucceeded {
                                action: "rollback_deployment".to_string(),
                                message: format!("Rolled back deployment {}", name),
                            });
                        }
                        Err(e) => {
                            let _ = self.sink.send(CoreEvent::ErrorOccurred {
                                action: "rollback_deployment".to_string(),
                                message: e.to_string(),
                            });
                        }
                    }
                }
            }
            CoreCommand::TriggerCronJob { name, namespace } => {
                if name.trim().is_empty() {
                    let _ = self.sink.send(CoreEvent::ErrorOccurred {
                        action: "trigger_cronjob".to_string(),
                        message: "Name cannot be empty".to_string(),
                    });
                    return;
                }
                let mgr = self.manager.read().await;
                let client = mgr.active_client.clone();
                if let Some(ref client) = client {
                    match crate::kubernetes::workloads::trigger_cronjob(client, &namespace, &name).await {
                        Ok(job_name) => {
                            let _ = self.sink.send(CoreEvent::ActionSucceeded {
                                action: "trigger_cronjob".to_string(),
                                message: format!("Created Job {} from CronJob {}", job_name, name),
                            });
                        }
                        Err(e) => {
                            let _ = self.sink.send(CoreEvent::ErrorOccurred {
                                action: "trigger_cronjob".to_string(),
                                message: e.to_string(),
                            });
                        }
                    }
                }
            }
            CoreCommand::SetCronJobSuspend { name, namespace, suspend } => {
                let mgr = self.manager.read().await;
                let client = mgr.active_client.clone();
                if let Some(ref client) = client {
                    match crate::kubernetes::workloads::set_cronjob_suspend(client, &namespace, &name, suspend).await {
                        Ok(_) => {
                            let _ = self.sink.send(CoreEvent::ActionSucceeded {
                                action: "set_cronjob_suspend".to_string(),
                                message: format!("Set suspend = {} on CronJob {}", suspend, name),
                            });
                        }
                        Err(e) => {
                            let _ = self.sink.send(CoreEvent::ErrorOccurred {
                                action: "set_cronjob_suspend".to_string(),
                                message: e.to_string(),
                            });
                        }
                    }
                }
            }
            CoreCommand::CloneDeployment { source_name, source_namespace, target_name, target_namespace } => {
                let mgr = self.manager.read().await;
                let client = mgr.active_client.clone();
                if let Some(ref client) = client {
                    match crate::kubernetes::workloads::clone_deployment(client, &source_namespace, &source_name, &target_name, &target_namespace).await {
                        Ok(_) => {
                            let _ = self.sink.send(CoreEvent::ActionSucceeded {
                                action: "clone_deployment".to_string(),
                                message: format!("Cloned deployment {} into {}", source_name, target_name),
                            });
                        }
                        Err(e) => {
                            let _ = self.sink.send(CoreEvent::ErrorOccurred {
                                action: "clone_deployment".to_string(),
                                message: e.to_string(),
                            });
                        }
                    }
                }
            }
            CoreCommand::CloneIngress { source_name, source_namespace, target_name, target_namespace } => {
                let mgr = self.manager.read().await;
                let client = mgr.active_client.clone();
                if let Some(ref client) = client {
                    match crate::kubernetes::ingresses::clone_ingress(client, &source_namespace, &source_name, &target_name, &target_namespace, vec![]).await {
                        Ok(_) => {
                            let _ = self.sink.send(CoreEvent::ActionSucceeded {
                                action: "clone_ingress".to_string(),
                                message: format!("Cloned ingress {} into {}", source_name, target_name),
                            });
                        }
                        Err(e) => {
                            let _ = self.sink.send(CoreEvent::ErrorOccurred {
                                action: "clone_ingress".to_string(),
                                message: e.to_string(),
                            });
                        }
                    }
                }
            }
            CoreCommand::CordonNode { name } => {
                let mgr = self.manager.read().await;
                let client = mgr.active_client.clone();
                if let Some(ref client) = client {
                    match crate::kubernetes::nodes::set_node_schedulable(client, &name, false).await {
                        Ok(_) => {
                            let _ = self.sink.send(CoreEvent::ActionSucceeded {
                                action: "cordon_node".to_string(),
                                message: format!("Cordoned node {}", name),
                            });
                        }
                        Err(e) => {
                            let _ = self.sink.send(CoreEvent::ErrorOccurred {
                                action: "cordon_node".to_string(),
                                message: e.to_string(),
                            });
                        }
                    }
                }
            }
            CoreCommand::UncordonNode { name } => {
                let mgr = self.manager.read().await;
                let client = mgr.active_client.clone();
                if let Some(ref client) = client {
                    match crate::kubernetes::nodes::set_node_schedulable(client, &name, true).await {
                        Ok(_) => {
                            let _ = self.sink.send(CoreEvent::ActionSucceeded {
                                action: "uncordon_node".to_string(),
                                message: format!("Uncordoned node {}", name),
                            });
                        }
                        Err(e) => {
                            let _ = self.sink.send(CoreEvent::ErrorOccurred {
                                action: "uncordon_node".to_string(),
                                message: e.to_string(),
                            });
                        }
                    }
                }
            }
            CoreCommand::DrainNode { name } => {
                let mgr = self.manager.read().await;
                let client = mgr.active_client.clone();
                if let Some(ref client) = client {
                    match crate::kubernetes::nodes::drain_node(client, &name).await {
                        Ok(_) => {
                            let _ = self.sink.send(CoreEvent::ActionSucceeded {
                                action: "drain_node".to_string(),
                                message: format!("Drained node {}", name),
                            });
                        }
                        Err(e) => {
                            let _ = self.sink.send(CoreEvent::ErrorOccurred {
                                action: "drain_node".to_string(),
                                message: e.to_string(),
                            });
                        }
                    }
                }
            }
            CoreCommand::StreamLogs { namespace, pod, workload_kind, workload_name, container, tail_lines } => {
                let mut mgr = self.manager.write().await;
                for cancel in mgr.log_cancel.drain(..) {
                    let _ = cancel.send(());
                }

                let client = mgr.active_client.clone();
                if let Some(ref client) = client {
                    let pod_name = if let Some(p) = pod {
                        p
                    } else if let (Some(wk), Some(wn)) = (workload_kind, workload_name) {
                        match crate::kubernetes::logs::get_workload_pods(client, &namespace, &wk, &wn).await {
                            Ok(pods) if !pods.is_empty() => pods[0].clone(),
                            _ => {
                                let _ = self.sink.send(CoreEvent::ErrorOccurred {
                                    action: "stream_logs".to_string(),
                                    message: "No pods found for workload".to_string(),
                                });
                                return;
                            }
                        }
                    } else {
                        let _ = self.sink.send(CoreEvent::ErrorOccurred {
                            action: "stream_logs".to_string(),
                            message: "Missing pod or workload name".to_string(),
                        });
                        return;
                    };

                    let (cancel_tx, cancel_rx) = tokio::sync::oneshot::channel();
                    mgr.log_cancel.push(cancel_tx);

                    tokio::spawn(crate::kubernetes::logs::stream_pod_logs(
                        client.clone(),
                        self.sink.clone(),
                        namespace,
                        pod_name,
                        container,
                        tail_lines,
                        cancel_rx,
                    ));
                }
            }
            CoreCommand::StopLogs => {
                let mut mgr = self.manager.write().await;
                for cancel in mgr.log_cancel.drain(..) {
                    let _ = cancel.send(());
                }
                let _ = self.sink.send(CoreEvent::LogStreamStopped);
            }
            CoreCommand::GetPortForwards => {
                let forwards = crate::kubernetes::port_forward::load_persisted_port_forwards();
                let _ = self.sink.send(CoreEvent::PortForwardsUpdated { forwards });
            }
            CoreCommand::StartPortForward { namespace, kind, name, local_port, remote_port } => {
                let mgr = self.manager.read().await;
                let id = format!("{}-{}-{}-{}", namespace, kind, name, local_port);
                let cluster_id = mgr.active_context.clone().unwrap_or_else(|| "default".to_string());
                let forward = crate::kubernetes::port_forward::PersistedPortForward {
                    id: id.clone(),
                    cluster_id: cluster_id.clone(),
                    namespace,
                    kind,
                    name,
                    local_port,
                    remote_port,
                };
                crate::kubernetes::port_forward::add_persisted_port_forward(forward);
                let forwards = crate::kubernetes::port_forward::load_persisted_port_forwards();
                let _ = self.sink.send(CoreEvent::PortForwardsUpdated { forwards });
            }
            CoreCommand::StopPortForward { id } => {
                let mut mgr = self.manager.write().await;
                if let Some((cancel, _)) = mgr.port_forward_cancel.remove(&id) {
                    let _ = cancel.send(());
                }
                crate::kubernetes::port_forward::remove_persisted_port_forward(&id);
                let forwards = crate::kubernetes::port_forward::load_persisted_port_forwards();
                let _ = self.sink.send(CoreEvent::PortForwardsUpdated { forwards });
            }
            CoreCommand::CheckForUpdates => {
                match crate::updater::UpdateManifest::fetch_from_github("0.20.0").await {
                    Ok(Some((manifest, has_update))) => {
                        if has_update {
                            let _ = self.sink.send(CoreEvent::UpdateAvailable {
                                version: manifest.version,
                                notes: manifest.release_notes.unwrap_or_default(),
                                download_url: manifest.url,
                            });
                        } else {
                            let _ = self.sink.send(CoreEvent::UpdateNotAvailable);
                        }
                    }
                    Ok(None) => {
                        let _ = self.sink.send(CoreEvent::UpdateNotAvailable);
                    }
                    Err(e) => {
                        let _ = self.sink.send(CoreEvent::UpdateError { error: e.to_string() });
                    }
                }
            }
            CoreCommand::ApplyUpdate { download_url } => {
                if !crate::updater::is_trusted_update_url(&download_url) {
                    let _ = self.sink.send(CoreEvent::UpdateError {
                        error: "Untrusted update URL".to_string(),
                    });
                    return;
                }
                match crate::updater::UpdateManifest::download_installer(&download_url, "latest", None).await {
                    Ok(_) => {
                        let _ = self.sink.send(CoreEvent::ActionSucceeded {
                            action: "apply_update".to_string(),
                            message: "Update applied successfully".to_string(),
                        });
                    }
                    Err(e) => {
                        let _ = self.sink.send(CoreEvent::UpdateError { error: e.to_string() });
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sink::MockEventSink;

    #[tokio::test]
    async fn test_dispatcher_ping_emits_pong() {
        let manager = Arc::new(RwLock::new(KubeManager::new().await));
        let sink = Arc::new(MockEventSink::new());
        let dispatcher = CoreDispatcher::new(manager, sink.clone());

        dispatcher.dispatch(CoreCommand::Ping).await;

        let events = sink.get_events();
        assert_eq!(events.len(), 1);
        match &events[0] {
            CoreEvent::Pong => {}
            other => panic!("expected Pong, got {:?}", other),
        }
    }

    #[tokio::test]
    async fn test_dispatcher_validation_error_emits_error_occurred() {
        let manager = Arc::new(RwLock::new(KubeManager::new().await));
        let sink = Arc::new(MockEventSink::new());
        let dispatcher = CoreDispatcher::new(manager, sink.clone());

        dispatcher.dispatch(CoreCommand::TriggerCronJob {
            name: "".to_string(),
            namespace: "default".to_string(),
        }).await;

        let events = sink.get_events();
        assert_eq!(events.len(), 1);
        match &events[0] {
            CoreEvent::ErrorOccurred { action, message } => {
                assert_eq!(action, "trigger_cronjob");
                assert!(message.contains("empty"));
            }
            other => panic!("expected ErrorOccurred, got {:?}", other),
        }
    }
}
