use std::sync::Arc;
use serde_json::Value;
use tokio::sync::RwLock;
use crate::ipc::bridge::{Bridge, WsWriter};
use crate::ipc::events::OrbitEvent;
use crate::kubernetes;
use crate::kubernetes::manager::KubeManager;
use tokio::sync::Mutex;

/// Spawns all background watchers and the metrics poller for a connected cluster.
/// Called when the engine starts up and whenever the active cluster changes.
pub fn spawn_watchers(
    client: &kube::Client,
    writer: Arc<Mutex<WsWriter>>,
    token: String,
    rx: tokio::sync::watch::Receiver<bool>,
) {
    tokio::spawn(crate::kubernetes::watchers::watch_resource::<
        k8s_openapi::api::core::v1::Service, _, _,
    >(
        client.clone(), writer.clone(), token.clone(), "Service".to_string(), rx.clone(),
        crate::kubernetes::services::map_service,
    ));

    tokio::spawn(crate::kubernetes::watchers::watch_resource::<
        k8s_openapi::api::apps::v1::Deployment, _, _,
    >(
        client.clone(), writer.clone(), token.clone(), "Deployment".to_string(), rx.clone(),
        crate::kubernetes::workloads::map_deployment,
    ));

    tokio::spawn(crate::kubernetes::watchers::watch_resource::<
        k8s_openapi::api::core::v1::Pod, _, _,
    >(
        client.clone(), writer.clone(), token.clone(), "Pod".to_string(), rx.clone(),
        crate::kubernetes::workloads::map_pod,
    ));

    tokio::spawn(crate::kubernetes::metrics::poll_pod_metrics(
        client.clone(), writer, token, rx,
    ));
}

/// Dispatches an IPC event from the frontend to the appropriate Kubernetes handler.
/// Each arm spawns an async task so the message loop is never blocked.
pub fn dispatch(
    event_name: &str,
    data: Option<Value>,
    writer: Arc<Mutex<WsWriter>>,
    token: String,
    manager: Arc<RwLock<KubeManager>>,
) {
    match event_name {
        "getClusters" => {
            tokio::spawn(async move {
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
        "switchCluster" => {
            tokio::spawn(async move {
                let cluster_id = data
                    .and_then(|d| d.get("clusterId").cloned())
                    .and_then(|v| v.as_str().map(|s| s.to_string()));

                if let Some(id) = cluster_id {
                    let mut w_manager = manager.write().await;
                    match w_manager.switch_context(&id).await {
                        Ok(()) => {
                            let active_cluster_id = w_manager.active_context.clone();
                            let clusters = w_manager.get_clusters();
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

                            // Spawn watchers and metrics poller for the new cluster.
                            if let Some(ref client) = client {
                                spawn_watchers(client, writer.clone(), token.clone(), rx.clone());
                            }

                            // Refresh all resources for the new active client
                            if let Some(ref client) = client {
                                if let Ok(namespaces) = kubernetes::list_namespaces(client).await {
                                    let _ = Bridge::send_event(&writer, &token, &OrbitEvent::NamespacesUpdated { namespaces }).await;
                                }
                                if let Ok(pods) = kubernetes::list_pods(client, None).await {
                                    let _ = Bridge::send_event(&writer, &token, &OrbitEvent::PodsUpdated { pods }).await;
                                }
                                if let Ok(persistent_volumes) = kubernetes::list_pvs(client).await {
                                    let _ = Bridge::send_event(&writer, &token, &OrbitEvent::PersistentVolumesUpdated { persistent_volumes }).await;
                                }
                                if let Ok(persistent_volume_claims) = kubernetes::list_pvcs(client, None).await {
                                    let _ = Bridge::send_event(&writer, &token, &OrbitEvent::PersistentVolumeClaimsUpdated { persistent_volume_claims }).await;
                                }
                                if let Ok(storage_classes) = kubernetes::list_storage_classes(client).await {
                                    let _ = Bridge::send_event(&writer, &token, &OrbitEvent::StorageClassesUpdated { storage_classes }).await;
                                }
                                if let Ok(nodes) = kubernetes::list_nodes(client).await {
                                    let _ = Bridge::send_event(&writer, &token, &OrbitEvent::NodesUpdated { nodes }).await;
                                }
                                if let Ok(events) = kubernetes::list_events(client, None).await {
                                    let _ = Bridge::send_event(&writer, &token, &OrbitEvent::EventsUpdated { events }).await;
                                }
                                if let Ok(policies) = kubernetes::list_policies(client, None).await {
                                    let _ = Bridge::send_event(&writer, &token, &OrbitEvent::PoliciesUpdated { policies }).await;
                                }
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
        "addCluster" => {
            tokio::spawn(async move {
                let file_path = data
                    .and_then(|d| d.get("filePath").cloned())
                    .and_then(|v| v.as_str().map(|s| s.to_string()));
 
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

                            // Refresh all resources for the new active client
                            if let Some(ref client) = client {
                                if let Ok(namespaces) = kubernetes::list_namespaces(client).await {
                                    let _ = Bridge::send_event(&writer, &token, &OrbitEvent::NamespacesUpdated { namespaces }).await;
                                }
                                if let Ok(pods) = kubernetes::list_pods(client, None).await {
                                    let _ = Bridge::send_event(&writer, &token, &OrbitEvent::PodsUpdated { pods }).await;
                                }
                                if let Ok(persistent_volumes) = kubernetes::list_pvs(client).await {
                                    let _ = Bridge::send_event(&writer, &token, &OrbitEvent::PersistentVolumesUpdated { persistent_volumes }).await;
                                }
                                if let Ok(persistent_volume_claims) = kubernetes::list_pvcs(client, None).await {
                                    let _ = Bridge::send_event(&writer, &token, &OrbitEvent::PersistentVolumeClaimsUpdated { persistent_volume_claims }).await;
                                }
                                if let Ok(storage_classes) = kubernetes::list_storage_classes(client).await {
                                    let _ = Bridge::send_event(&writer, &token, &OrbitEvent::StorageClassesUpdated { storage_classes }).await;
                                }
                                if let Ok(nodes) = kubernetes::list_nodes(client).await {
                                    let _ = Bridge::send_event(&writer, &token, &OrbitEvent::NodesUpdated { nodes }).await;
                                }
                                if let Ok(events) = kubernetes::list_events(client, None).await {
                                    let _ = Bridge::send_event(&writer, &token, &OrbitEvent::EventsUpdated { events }).await;
                                }
                                if let Ok(policies) = kubernetes::list_policies(client, None).await {
                                    let _ = Bridge::send_event(&writer, &token, &OrbitEvent::PoliciesUpdated { policies }).await;
                                }
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
        "getNamespaces" => {
            tokio::spawn(async move {
                let client = {
                    let r_manager = manager.read().await;
                    r_manager.active_client.clone()
                };
                if let Some(ref client) = client {
                    match kubernetes::list_namespaces(client).await {
                        Ok(namespaces) => {
                            let _ = Bridge::send_event(&writer, &token, &OrbitEvent::NamespacesUpdated { namespaces }).await;
                        }
                        Err(e) => { log::error!("Error listing namespaces: {:?}", e); }
                    }
                }
            });
        }
        "getPods" => {
            tokio::spawn(async move {
                let namespace = data
                    .and_then(|d| d.get("namespace").cloned())
                    .and_then(|v| v.as_str().map(|s| s.to_string()));

                let client = {
                    let r_manager = manager.read().await;
                    r_manager.active_client.clone()
                };
                if let Some(ref client) = client {
                    match kubernetes::list_pods(client, namespace).await {
                        Ok(pods) => {
                            let _ = Bridge::send_event(&writer, &token, &OrbitEvent::PodsUpdated { pods }).await;
                        }
                        Err(e) => { log::error!("Error listing pods: {:?}", e); }
                    }
                }
            });
        }
        "getDeployments" => {
            tokio::spawn(async move {
                let namespace = data
                    .and_then(|d| d.get("namespace").cloned())
                    .and_then(|v| v.as_str().map(|s| s.to_string()));

                let client = {
                    let r_manager = manager.read().await;
                    r_manager.active_client.clone()
                };
                if let Some(ref client) = client {
                    match kubernetes::list_deployments(client, namespace).await {
                        Ok(deployments) => {
                            let _ = Bridge::send_event(&writer, &token, &OrbitEvent::DeploymentsUpdated { deployments }).await;
                        }
                        Err(e) => { log::error!("Error listing deployments: {:?}", e); }
                    }
                }
            });
        }
        "getStatefulSets" => {
            tokio::spawn(async move {
                let namespace = data
                    .and_then(|d| d.get("namespace").cloned())
                    .and_then(|v| v.as_str().map(|s| s.to_string()));

                let client = {
                    let r_manager = manager.read().await;
                    r_manager.active_client.clone()
                };
                if let Some(ref client) = client {
                    match kubernetes::list_statefulsets(client, namespace).await {
                        Ok(stateful_sets) => {
                            let _ = Bridge::send_event(&writer, &token, &OrbitEvent::StatefulSetsUpdated { stateful_sets }).await;
                        }
                        Err(e) => { log::error!("Error listing statefulsets: {:?}", e); }
                    }
                }
            });
        }
        "getDaemonSets" => {
            tokio::spawn(async move {
                let namespace = data
                    .and_then(|d| d.get("namespace").cloned())
                    .and_then(|v| v.as_str().map(|s| s.to_string()));

                let client = {
                    let r_manager = manager.read().await;
                    r_manager.active_client.clone()
                };
                if let Some(ref client) = client {
                    match kubernetes::list_daemonsets(client, namespace).await {
                        Ok(daemon_sets) => {
                            let _ = Bridge::send_event(&writer, &token, &OrbitEvent::DaemonSetsUpdated { daemon_sets }).await;
                        }
                        Err(e) => { log::error!("Error listing daemonsets: {:?}", e); }
                    }
                }
            });
        }
        "getReplicaSets" => {
            tokio::spawn(async move {
                let namespace = data
                    .and_then(|d| d.get("namespace").cloned())
                    .and_then(|v| v.as_str().map(|s| s.to_string()));

                let client = {
                    let r_manager = manager.read().await;
                    r_manager.active_client.clone()
                };
                if let Some(ref client) = client {
                    match kubernetes::list_replicasets(client, namespace).await {
                        Ok(replica_sets) => {
                            let _ = Bridge::send_event(&writer, &token, &OrbitEvent::ReplicaSetsUpdated { replica_sets }).await;
                        }
                        Err(e) => { log::error!("Error listing replicasets: {:?}", e); }
                    }
                }
            });
        }
        "getJobs" => {
            tokio::spawn(async move {
                let namespace = data
                    .and_then(|d| d.get("namespace").cloned())
                    .and_then(|v| v.as_str().map(|s| s.to_string()));

                let client = {
                    let r_manager = manager.read().await;
                    r_manager.active_client.clone()
                };
                if let Some(ref client) = client {
                    match kubernetes::list_jobs(client, namespace).await {
                        Ok(jobs) => {
                            let _ = Bridge::send_event(&writer, &token, &OrbitEvent::JobsUpdated { jobs }).await;
                        }
                        Err(e) => { log::error!("Error listing jobs: {:?}", e); }
                    }
                }
            });
        }
        "getCronJobs" => {
            tokio::spawn(async move {
                let namespace = data
                    .and_then(|d| d.get("namespace").cloned())
                    .and_then(|v| v.as_str().map(|s| s.to_string()));

                let client = {
                    let r_manager = manager.read().await;
                    r_manager.active_client.clone()
                };
                if let Some(ref client) = client {
                    match kubernetes::list_cronjobs(client, namespace).await {
                        Ok(cron_jobs) => {
                            let _ = Bridge::send_event(&writer, &token, &OrbitEvent::CronJobsUpdated { cron_jobs }).await;
                        }
                        Err(e) => { log::error!("Error listing cronjobs: {:?}", e); }
                    }
                }
            });
        }
        "getServices" => {
            tokio::spawn(async move {
                let namespace = data
                    .and_then(|d| d.get("namespace").cloned())
                    .and_then(|v| v.as_str().map(|s| s.to_string()));

                let client = {
                    let r_manager = manager.read().await;
                    r_manager.active_client.clone()
                };
                if let Some(ref client) = client {
                    match kubernetes::list_services(client, namespace).await {
                        Ok(services) => {
                            let _ = Bridge::send_event(&writer, &token, &OrbitEvent::ServicesUpdated { services }).await;
                        }
                        Err(e) => { log::error!("Error listing services: {:?}", e); }
                    }
                }
            });
        }
        "getConfigMaps" => {
            tokio::spawn(async move {
                let namespace = data
                    .and_then(|d| d.get("namespace").cloned())
                    .and_then(|v| v.as_str().map(|s| s.to_string()));

                let client = {
                    let r_manager = manager.read().await;
                    r_manager.active_client.clone()
                };
                if let Some(ref client) = client {
                    match kubernetes::list_configmaps(client, namespace).await {
                        Ok(config_maps) => {
                            let _ = Bridge::send_event(&writer, &token, &OrbitEvent::ConfigMapsUpdated { config_maps }).await;
                        }
                        Err(e) => { log::error!("Error listing configmaps: {:?}", e); }
                    }
                }
            });
        }
        "getEvents" => {
            tokio::spawn(async move {
                let namespace = data
                    .and_then(|d| d.get("namespace").cloned())
                    .and_then(|v| v.as_str().map(|s| s.to_string()));

                let client = {
                    let r_manager = manager.read().await;
                    r_manager.active_client.clone()
                };
                if let Some(ref client) = client {
                    match kubernetes::list_events(client, namespace).await {
                        Ok(events) => {
                            let _ = Bridge::send_event(&writer, &token, &OrbitEvent::EventsUpdated { events }).await;
                        }
                        Err(e) => { log::error!("Error listing events: {:?}", e); }
                    }
                }
            });
        }
        "getSecrets" => {
            tokio::spawn(async move {
                let namespace = data
                    .and_then(|d| d.get("namespace").cloned())
                    .and_then(|v| v.as_str().map(|s| s.to_string()));

                let client = {
                    let r_manager = manager.read().await;
                    r_manager.active_client.clone()
                };
                if let Some(ref client) = client {
                    match kubernetes::list_secrets(client, namespace).await {
                        Ok(secrets) => {
                            let _ = Bridge::send_event(&writer, &token, &OrbitEvent::SecretsUpdated { secrets }).await;
                        }
                        Err(e) => { log::error!("Error listing secrets: {:?}", e); }
                    }
                }
            });
        }
        "getPersistentVolumes" => {
            tokio::spawn(async move {
                let client = {
                    let r_manager = manager.read().await;
                    r_manager.active_client.clone()
                };
                if let Some(ref client) = client {
                    match kubernetes::list_pvs(client).await {
                        Ok(persistent_volumes) => {
                            let _ = Bridge::send_event(&writer, &token, &OrbitEvent::PersistentVolumesUpdated { persistent_volumes }).await;
                        }
                        Err(e) => { log::error!("Error listing persistent volumes: {:?}", e); }
                    }
                }
            });
        }
        "getPersistentVolumeClaims" => {
            tokio::spawn(async move {
                let namespace = data
                    .and_then(|d| d.get("namespace").cloned())
                    .and_then(|v| v.as_str().map(|s| s.to_string()));

                let client = {
                    let r_manager = manager.read().await;
                    r_manager.active_client.clone()
                };
                if let Some(ref client) = client {
                    match kubernetes::list_pvcs(client, namespace).await {
                        Ok(persistent_volume_claims) => {
                            let _ = Bridge::send_event(&writer, &token, &OrbitEvent::PersistentVolumeClaimsUpdated { persistent_volume_claims }).await;
                        }
                        Err(e) => { log::error!("Error listing persistent volume claims: {:?}", e); }
                    }
                }
            });
        }
        "getStorageClasses" => {
            tokio::spawn(async move {
                let client = {
                    let r_manager = manager.read().await;
                    r_manager.active_client.clone()
                };
                if let Some(ref client) = client {
                    match kubernetes::list_storage_classes(client).await {
                        Ok(storage_classes) => {
                            let _ = Bridge::send_event(&writer, &token, &OrbitEvent::StorageClassesUpdated { storage_classes }).await;
                        }
                        Err(e) => { log::error!("Error listing storage classes: {:?}", e); }
                    }
                }
            });
        }
        "getNodes" => {
            tokio::spawn(async move {
                let client = {
                    let r_manager = manager.read().await;
                    r_manager.active_client.clone()
                };
                if let Some(ref client) = client {
                    match kubernetes::list_nodes(client).await {
                        Ok(nodes) => {
                            let _ = Bridge::send_event(&writer, &token, &OrbitEvent::NodesUpdated { nodes }).await;
                        }
                        Err(e) => { log::error!("Error listing nodes: {:?}", e); }
                    }
                }
            });
        }
        "getPolicies" => {
            tokio::spawn(async move {
                let namespace = data
                    .and_then(|d| d.get("namespace").cloned())
                    .and_then(|v| v.as_str().map(|s| s.to_string()));

                let client = {
                    let r_manager = manager.read().await;
                    r_manager.active_client.clone()
                };
                if let Some(ref client) = client {
                    match kubernetes::list_policies(client, namespace).await {
                        Ok(policies) => {
                            let _ = Bridge::send_event(&writer, &token, &OrbitEvent::PoliciesUpdated { policies }).await;
                        }
                        Err(e) => { log::error!("Error listing policies: {:?}", e); }
                    }
                }
            });
        }
        "checkForUpdates" => {
            tokio::spawn(async move {
                let url = data
                    .as_ref()
                    .and_then(|d| d.get("manifestUrl").cloned())
                    .and_then(|v| v.as_str().map(|s| s.to_string()))
                    .unwrap_or_else(|| "https://raw.githubusercontent.com/vantoan1511/orbit/main/update-manifest.json".to_string());
                
                match crate::updater::UpdateManifest::fetch(&url).await {
                    Ok(manifest) => {
                        let current_engine = env!("CARGO_PKG_VERSION");
                        
                        let has_update = manifest.has_update(current_engine).unwrap_or(false);

                        let _ = Bridge::send_event(
                            &writer,
                            &token,
                            &OrbitEvent::UpdateCheckFinished {
                                has_update,
                                manifest,
                            },
                        ).await;
                    }
                    Err(e) => {
                        log::error!("Failed to fetch update manifest: {:?}", e);
                        let _ = Bridge::send_event(
                            &writer,
                            &token,
                            &OrbitEvent::ErrorOccurred {
                                message: format!("Failed to check for updates: {}", e),
                            },
                        ).await;
                    }
                }
            });
        }
        "applyUpdate" => {
            tokio::spawn(async move {
                let url = data
                    .and_then(|d| d.get("url").cloned())
                    .and_then(|v| v.as_str().map(|s| s.to_string()));
                    
                if let Some(url) = url {
                    let (tx, mut rx) = tokio::sync::mpsc::channel(100);
                    let writer_clone = writer.clone();
                    let token_clone = token.clone();
                    
                    tokio::spawn(async move {
                        while let Some(progress) = rx.recv().await {
                            let _ = Bridge::send_event(
                                &writer_clone,
                                &token_clone,
                                &OrbitEvent::UpdateDownloadProgress {
                                    component: "app".to_string(),
                                    progress_percentage: progress,
                                },
                            ).await;
                        }
                    });

                    let download_res = crate::updater::UpdateManifest::download(&url, "orbit-update.zip", Some(tx)).await;
                    if let Ok(path) = download_res {
                        let current_exe_res = std::env::current_exe();
                        if let Ok(current_exe) = current_exe_res {
                            let bin_dir_opt = current_exe.parent();
                            if let Some(bin_dir) = bin_dir_opt {
                                let app_dir = bin_dir.parent().unwrap_or(bin_dir);
                                
                                let updater_name = if cfg!(target_os = "windows") { "orbit-apply.exe" } else { "orbit-apply" };
                                let updater_path = bin_dir.join(updater_name);
                                
                                let os = std::env::consts::OS;
                                let arch = std::env::consts::ARCH;
                                
                                let neu_os = match os {
                                    "windows" => "win",
                                    "macos" => "mac",
                                    "linux" => "linux",
                                    _ => "linux",
                                };
                                
                                let neu_arch = match arch {
                                    "x86_64" => "x64",
                                    "aarch64" => "arm64",
                                    _ => "x64",
                                };
                                
                                let ext = if cfg!(target_os = "windows") { ".exe" } else { "" };
                                let exe_name = format!("orbit-{}_{}{}", neu_os, neu_arch, ext);
                                log::info!("Spawning updater: {:?} with zip: {:?}, target_dir: {:?}, exe_name: {}", updater_path, path, app_dir, exe_name);
                                
                                match std::process::Command::new(&updater_path)
                                    .arg("--zip-path")
                                    .arg(&path)
                                    .arg("--target-dir")
                                    .arg(app_dir)
                                    .arg("--executable-name")
                                    .arg(exe_name)
                                    .spawn() {
                                        Ok(_) => {
                                            log::info!("Updater spawned successfully.");
                                        }
                                        Err(e) => {
                                            log::error!("Failed to spawn updater: {:?}", e);
                                        }
                                    }
                            }
                        }
                        
                        let _ = Bridge::send_event(
                            &writer,
                            &token,
                            &OrbitEvent::UpdateReady {
                                component: "app".to_string(),
                            },
                        ).await;
                    }
                }
            });
        }
        _ => {}
    }
}
