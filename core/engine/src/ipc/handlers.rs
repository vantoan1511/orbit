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

    tokio::spawn(crate::kubernetes::watchers::watch_resource::<
        k8s_openapi::api::apps::v1::StatefulSet, _, _,
    >(
        client.clone(), writer.clone(), token.clone(), "StatefulSet".to_string(), rx.clone(),
        crate::kubernetes::workloads::map_statefulset,
    ));

    tokio::spawn(crate::kubernetes::watchers::watch_resource::<
        k8s_openapi::api::apps::v1::DaemonSet, _, _,
    >(
        client.clone(), writer.clone(), token.clone(), "DaemonSet".to_string(), rx.clone(),
        crate::kubernetes::workloads::map_daemonset,
    ));

    tokio::spawn(crate::kubernetes::watchers::watch_resource::<
        k8s_openapi::api::apps::v1::ReplicaSet, _, _,
    >(
        client.clone(), writer.clone(), token.clone(), "ReplicaSet".to_string(), rx.clone(),
        crate::kubernetes::workloads::map_replicaset,
    ));

    tokio::spawn(crate::kubernetes::watchers::watch_resource::<
        k8s_openapi::api::batch::v1::Job, _, _,
    >(
        client.clone(), writer.clone(), token.clone(), "Job".to_string(), rx.clone(),
        crate::kubernetes::batch::map_job,
    ));

    tokio::spawn(crate::kubernetes::watchers::watch_resource::<
        k8s_openapi::api::batch::v1::CronJob, _, _,
    >(
        client.clone(), writer.clone(), token.clone(), "CronJob".to_string(), rx.clone(),
        crate::kubernetes::batch::map_cronjob,
    ));

    tokio::spawn(crate::kubernetes::watchers::watch_resource::<
        k8s_openapi::api::core::v1::Namespace, _, _,
    >(
        client.clone(), writer.clone(), token.clone(), "Namespace".to_string(), rx.clone(),
        crate::kubernetes::namespaces::map_namespace,
    ));

    tokio::spawn(crate::kubernetes::watchers::watch_resource::<
        k8s_openapi::api::core::v1::ConfigMap, _, _,
    >(
        client.clone(), writer.clone(), token.clone(), "ConfigMap".to_string(), rx.clone(),
        crate::kubernetes::configmaps::format_configmap,
    ));

    tokio::spawn(crate::kubernetes::watchers::watch_resource::<
        k8s_openapi::api::core::v1::Secret, _, _,
    >(
        client.clone(), writer.clone(), token.clone(), "Secret".to_string(), rx.clone(),
        crate::kubernetes::secrets::format_secret,
    ));

    tokio::spawn(crate::kubernetes::watchers::watch_resource::<
        k8s_openapi::api::core::v1::Event, _, _,
    >(
        client.clone(), writer.clone(), token.clone(), "Event".to_string(), rx.clone(),
        crate::kubernetes::events::format_event,
    ));

    tokio::spawn(crate::kubernetes::watchers::watch_resource::<
        k8s_openapi::api::core::v1::PersistentVolume, _, _,
    >(
        client.clone(), writer.clone(), token.clone(), "PersistentVolume".to_string(), rx.clone(),
        crate::kubernetes::storage::format_pv,
    ));

    tokio::spawn(crate::kubernetes::watchers::watch_resource::<
        k8s_openapi::api::core::v1::PersistentVolumeClaim, _, _,
    >(
        client.clone(), writer.clone(), token.clone(), "PersistentVolumeClaim".to_string(), rx.clone(),
        crate::kubernetes::storage::format_pvc,
    ));

    tokio::spawn(crate::kubernetes::watchers::watch_resource::<
        k8s_openapi::api::storage::v1::StorageClass, _, _,
    >(
        client.clone(), writer.clone(), token.clone(), "StorageClass".to_string(), rx.clone(),
        crate::kubernetes::storage::format_storage_class,
    ));

    tokio::spawn(crate::kubernetes::watchers::watch_resource::<
        k8s_openapi::api::networking::v1::NetworkPolicy, _, _,
    >(
        client.clone(), writer.clone(), token.clone(), "Policy".to_string(), rx.clone(),
        crate::kubernetes::policies::format_network_policy,
    ));

    tokio::spawn(crate::kubernetes::watchers::watch_resource::<
        k8s_openapi::api::core::v1::ResourceQuota, _, _,
    >(
        client.clone(), writer.clone(), token.clone(), "Policy".to_string(), rx.clone(),
        crate::kubernetes::policies::format_resource_quota,
    ));

    tokio::spawn(crate::kubernetes::watchers::watch_resource::<
        k8s_openapi::api::core::v1::LimitRange, _, _,
    >(
        client.clone(), writer.clone(), token.clone(), "Policy".to_string(), rx.clone(),
        crate::kubernetes::policies::format_limit_range,
    ));

    tokio::spawn(crate::kubernetes::watchers::watch_resource::<
        k8s_openapi::api::admissionregistration::v1::ValidatingWebhookConfiguration, _, _,
    >(
        client.clone(), writer.clone(), token.clone(), "Policy".to_string(), rx.clone(),
        crate::kubernetes::policies::format_val_webhook,
    ));

    tokio::spawn(crate::kubernetes::watchers::watch_resource::<
        k8s_openapi::api::admissionregistration::v1::MutatingWebhookConfiguration, _, _,
    >(
        client.clone(), writer.clone(), token.clone(), "Policy".to_string(), rx.clone(),
        crate::kubernetes::policies::format_mut_webhook,
    ));

    tokio::spawn(crate::kubernetes::metrics::poll_pod_metrics(
        client.clone(), writer, token, rx,
    ));
}


fn get_string(data: &Option<Value>, key: &str) -> Option<String> {
    data.as_ref()
        .and_then(|d| d.get(key))
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
}

fn get_i64(data: &Option<Value>, key: &str) -> Option<i64> {
    data.as_ref()
        .and_then(|d| d.get(key))
        .and_then(|v| v.as_i64())
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
        "switchCluster" => {
            tokio::spawn(async move {
                let cluster_id = get_string(&data, "clusterId");

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
                let namespace = get_string(&data, "namespace");

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
                let namespace = get_string(&data, "namespace");

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
                let namespace = get_string(&data, "namespace");

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
                let namespace = get_string(&data, "namespace");

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
                let namespace = get_string(&data, "namespace");

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
                let namespace = get_string(&data, "namespace");

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
                let namespace = get_string(&data, "namespace");

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
                let namespace = get_string(&data, "namespace");

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
                let namespace = get_string(&data, "namespace");

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
                let namespace = get_string(&data, "namespace");

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
                let namespace = get_string(&data, "namespace");

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
                let namespace = get_string(&data, "namespace");

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
                let namespace = get_string(&data, "namespace");

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
                let url = get_string(&data, "manifestUrl")
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
                let url = get_string(&data, "url");
                    
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
        "streamLogs" => {
            tokio::spawn(async move {
                let namespace = get_string(&data, "namespace").unwrap_or_else(|| "default".to_string());
                let workload_name = get_string(&data, "workload").unwrap_or_default();
                let workload_kind = get_string(&data, "kind").unwrap_or_else(|| "Deployment".to_string());
                let container = get_string(&data, "container").filter(|s| !s.is_empty() && s != "All" && s != "all");
                let pod_name = get_string(&data, "pod").filter(|s| !s.is_empty() && s != "All" && s != "all");
                let tail_lines = get_i64(&data, "tailLines");

                let mut w_manager = manager.write().await;
                for cancel in w_manager.log_cancel.drain(..) {
                    let _ = cancel.send(());
                }

                let client = match w_manager.active_client.clone() {
                    Some(c) => c,
                    None => {
                        let _ = Bridge::send_event(
                            &writer,
                            &token,
                            &OrbitEvent::ErrorOccurred {
                                message: "No active cluster client found".to_string(),
                            },
                        ).await;
                        return;
                    }
                };

                let pods_to_stream = if let Some(specific_pod) = pod_name {
                    vec![specific_pod]
                } else {
                    match crate::kubernetes::get_workload_pods(&client, &namespace, &workload_name, &workload_kind).await {
                        Ok(p) => p,
                        Err(e) => {
                            let _ = Bridge::send_event(
                                &writer,
                                &token,
                                &OrbitEvent::ErrorOccurred {
                                    message: format!("Failed to find pods for workload: {}", e),
                                },
                            ).await;
                            return;
                        }
                    }
                };

                for pod in pods_to_stream {
                    let (cancel_tx, cancel_rx) = tokio::sync::oneshot::channel();
                    w_manager.log_cancel.push(cancel_tx);

                    let client_clone = client.clone();
                    let writer_clone = writer.clone();
                    let token_clone = token.clone();
                    let ns_clone = namespace.clone();
                    let container_clone = container.clone();

                    tokio::spawn(async move {
                        crate::kubernetes::stream_pod_logs(
                            client_clone,
                            writer_clone,
                            token_clone,
                            ns_clone,
                            pod,
                            container_clone,
                            tail_lines,
                            cancel_rx,
                        ).await;
                    });
                }
            });
        }
        "stopLogs" => {
            tokio::spawn(async move {
                let mut w_manager = manager.write().await;
                for cancel in w_manager.log_cancel.drain(..) {
                    let _ = cancel.send(());
                }
            });
        }
        "scaleResource" => {
            tokio::spawn(async move {
                let namespace = get_string(&data, "namespace").unwrap_or_else(|| "default".to_string());
                let kind = get_string(&data, "kind").unwrap_or_default();
                let name = get_string(&data, "name").unwrap_or_default();
                let replicas = get_i64(&data, "replicas").unwrap_or(1) as i32;

                let client = {
                    let r_manager = manager.read().await;
                    r_manager.active_client.clone()
                };

                if let Some(ref client) = client {
                    match crate::kubernetes::workloads::scale_resource(client, &namespace, &kind, &name, replicas).await {
                        Ok(()) => {
                            let _ = Bridge::send_event(
                                &writer,
                                &token,
                                &OrbitEvent::CommandSucceeded {
                                    message: format!("Scaled {} {} to {} replicas", kind, name, replicas),
                                },
                            ).await;
                        }
                        Err(e) => {
                            log::error!("Error scaling {}: {:?}", kind, e);
                            let _ = Bridge::send_event(
                                &writer,
                                &token,
                                &OrbitEvent::ErrorOccurred {
                                    message: format!("Failed to scale: {}", e),
                                },
                            ).await;
                        }
                    }
                }
            });
        }
        "redeployResource" => {
            tokio::spawn(async move {
                let namespace = get_string(&data, "namespace").unwrap_or_else(|| "default".to_string());
                let kind = get_string(&data, "kind").unwrap_or_default();
                let name = get_string(&data, "name").unwrap_or_default();

                let client = {
                    let r_manager = manager.read().await;
                    r_manager.active_client.clone()
                };

                if let Some(ref client) = client {
                    match crate::kubernetes::workloads::redeploy_resource(client, &namespace, &kind, &name).await {
                        Ok(()) => {
                            let _ = Bridge::send_event(
                                &writer,
                                &token,
                                &OrbitEvent::CommandSucceeded {
                                    message: format!("Redeployed {} {}", kind, name),
                                },
                            ).await;
                        }
                        Err(e) => {
                            log::error!("Error redeploying {}: {:?}", kind, e);
                            let _ = Bridge::send_event(
                                &writer,
                                &token,
                                &OrbitEvent::ErrorOccurred {
                                    message: format!("Failed to redeploy: {}", e),
                                },
                            ).await;
                        }
                    }
                }
            });
        }
        "deleteResource" => {
            tokio::spawn(async move {
                let namespace = get_string(&data, "namespace").unwrap_or_else(|| "default".to_string());
                let kind = get_string(&data, "kind").unwrap_or_default();
                let name = get_string(&data, "name").unwrap_or_default();

                let client = {
                    let r_manager = manager.read().await;
                    r_manager.active_client.clone()
                };

                if let Some(ref client) = client {
                    match crate::kubernetes::delete_resource(client, &namespace, &kind, &name).await {
                        Ok(()) => {
                            let _ = Bridge::send_event(
                                &writer,
                                &token,
                                &OrbitEvent::CommandSucceeded {
                                    message: format!("Deleted {} {}", kind, name),
                                },
                            ).await;
                        }
                        Err(e) => {
                            log::error!("Error deleting {} {}: {:?}", kind, name, e);
                            let _ = Bridge::send_event(
                                &writer,
                                &token,
                                &OrbitEvent::ErrorOccurred {
                                    message: format!("Failed to delete: {}", e),
                                },
                            ).await;
                        }
                    }
                }
            });
        }
        "restartPod" => {
            tokio::spawn(async move {
                let namespace = get_string(&data, "namespace").unwrap_or_else(|| "default".to_string());
                let name = get_string(&data, "name").unwrap_or_default();

                let client = {
                    let r_manager = manager.read().await;
                    r_manager.active_client.clone()
                };

                if let Some(ref client) = client {
                    match crate::kubernetes::workloads::delete_pod(client, &namespace, &name).await {
                        Ok(()) => {
                            let _ = Bridge::send_event(
                                &writer,
                                &token,
                                &OrbitEvent::CommandSucceeded {
                                    message: format!("Restarted Pod {}", name),
                                },
                            ).await;
                        }
                        Err(e) => {
                            log::error!("Error restarting Pod {}: {:?}", name, e);
                            let _ = Bridge::send_event(
                                &writer,
                                &token,
                                &OrbitEvent::ErrorOccurred {
                                    message: format!("Failed to restart pod: {}", e),
                                },
                            ).await;
                        }
                    }
                }
            });
        }
        _ => {}
    }
}
