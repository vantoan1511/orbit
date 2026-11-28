use std::collections::HashMap;
use std::sync::Arc;

use orbit_core::kubernetes::batch::{execute_trigger_cronjob, fetch_cronjobs, fetch_jobs};
use orbit_core::kubernetes::configmaps::fetch_configmaps;
use orbit_core::kubernetes::delete::{execute_delete_resource, format_target_label};
use orbit_core::kubernetes::events::fetch_events;
use orbit_core::kubernetes::hpa::fetch_hpas;
use orbit_core::kubernetes::ingresses::fetch_ingresses;
use orbit_core::kubernetes::logs::{
    LogStreamMessage, resolve_workload_pods, select_streamable_pods, stream_pod_logs,
};
use orbit_core::kubernetes::manager::KubeManager;
use orbit_core::kubernetes::namespaces::fetch_namespaces;
use orbit_core::kubernetes::nodes::fetch_nodes;
use orbit_core::kubernetes::overview::fetch_cluster_overview;
use orbit_core::kubernetes::policies::{
    fetch_admission_webhooks, fetch_network_policies, fetch_pod_security_policies, fetch_policies,
    fetch_resource_quotas_and_limits,
};
use orbit_core::kubernetes::port_forward::{
    PortForwardLifecycleEvent, PortForwardRequest, get_persisted_for_cluster,
    restore_cluster_port_forwards, start_port_forward_tunnel, stop_all_active_port_forwards,
    stop_cluster_port_forwards, stop_port_forward_tunnel,
};
use orbit_core::kubernetes::secrets::fetch_secrets;
use orbit_core::kubernetes::services::fetch_services;
use orbit_core::kubernetes::storage::{fetch_pvcs, fetch_pvs, fetch_storage_classes};
use orbit_core::kubernetes::storage_overview::fetch_storage_overview;
use orbit_core::kubernetes::workloads::{
    execute_restart_pod, execute_restart_resource, execute_scale_resource, fetch_daemonsets,
    fetch_deployments, fetch_pods, fetch_replicasets, fetch_statefulsets,
};
use orbit_core::kubernetes::workloads_overview::fetch_workloads_overview;
use tokio::sync::{RwLock, mpsc};

use crate::state::{ClusterBridgeCommand, ClusterBridgeEvent, WorkloadActionKind};

pub struct ClusterBridge {
    pub command_tx: mpsc::UnboundedSender<ClusterBridgeCommand>,
    pub event_rx: mpsc::UnboundedReceiver<ClusterBridgeEvent>,
}

#[derive(Default)]
pub struct ActiveLogStream {
    cancel_txs: Vec<tokio::sync::oneshot::Sender<()>>,
    abort_handles: Vec<tokio::task::AbortHandle>,
}

#[derive(Default)]
pub struct LogStreamRegistry {
    streams: HashMap<u64, ActiveLogStream>,
}

impl LogStreamRegistry {
    pub fn register(
        &mut self,
        stream_id: u64,
        cancel_txs: Vec<tokio::sync::oneshot::Sender<()>>,
        abort_handles: Vec<tokio::task::AbortHandle>,
    ) {
        self.stop(stream_id);
        self.streams.insert(
            stream_id,
            ActiveLogStream {
                cancel_txs,
                abort_handles,
            },
        );
    }

    pub fn stop(&mut self, stream_id: u64) {
        if let Some(active) = self.streams.remove(&stream_id) {
            for tx in active.cancel_txs {
                let _ = tx.send(());
            }
            for handle in active.abort_handles {
                handle.abort();
            }
        }
    }

    pub fn stop_all(&mut self) {
        let keys: Vec<u64> = self.streams.keys().copied().collect();
        for stream_id in keys {
            self.stop(stream_id);
        }
    }

    #[cfg(test)]
    pub fn active_count(&self) -> usize {
        self.streams.len()
    }
}

#[derive(Default)]
pub struct BridgeTaskTracker {
    handles: Vec<tokio::task::AbortHandle>,
    pub log_streams: LogStreamRegistry,
}

impl BridgeTaskTracker {
    pub fn spawn<F>(&mut self, future: F)
    where
        F: std::future::Future<Output = ()> + Send + 'static,
    {
        self.handles.retain(|h| !h.is_finished());
        let handle = tokio::spawn(future).abort_handle();
        self.handles.push(handle);
    }

    pub fn cancel_all(&mut self) {
        for handle in self.handles.drain(..) {
            handle.abort();
        }
        self.log_streams.stop_all();
    }
}

#[cfg(test)]
pub async fn handle_bridge_command(
    cmd: ClusterBridgeCommand,
    manager: &Arc<RwLock<KubeManager>>,
    event_tx: &mpsc::UnboundedSender<ClusterBridgeEvent>,
) {
    let mut tracker = BridgeTaskTracker::default();
    handle_bridge_command_with_tracker(cmd, manager, event_tx, &mut tracker).await;
}

pub async fn handle_bridge_command_with_tracker(
    cmd: ClusterBridgeCommand,
    manager: &Arc<RwLock<KubeManager>>,
    event_tx: &mpsc::UnboundedSender<ClusterBridgeEvent>,
    tracker: &mut BridgeTaskTracker,
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
            tracker.cancel_all();
            stop_all_active_port_forwards(manager).await;
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
            tracker.cancel_all();
            stop_all_active_port_forwards(manager).await;
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
        ClusterBridgeCommand::DisconnectCluster => {
            tracker.cancel_all();
            stop_all_active_port_forwards(manager).await;
            let mut guard = manager.write().await;
            let outcome = guard.disconnect_active_cluster();
            let clusters = guard.get_clusters();
            let _ = event_tx.send(ClusterBridgeEvent::ClusterDisconnected {
                previous_cluster_id: outcome.previous_context,
                clusters,
            });
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
                tracker.spawn(async move {
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
        ClusterBridgeCommand::LoadWorkloadsOverview {
            cluster_id,
            generation,
        } => {
            let client = {
                let guard = manager.read().await;
                guard.active_client_for_context(&cluster_id)
            };

            if let Some(client) = client {
                let tx = event_tx.clone();
                tracker.spawn(async move {
                    let snapshot = fetch_workloads_overview(&client).await;
                    let _ = tx.send(ClusterBridgeEvent::WorkloadsOverviewLoaded {
                        cluster_id,
                        generation,
                        snapshot,
                    });
                });
            } else {
                let _ = event_tx.send(ClusterBridgeEvent::WorkloadsOverviewLoadFailed {
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
                tracker.spawn(async move {
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
                tracker.spawn(async move {
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
        ClusterBridgeCommand::LoadStatefulSets {
            cluster_id,
            generation,
        } => {
            let client = {
                let guard = manager.read().await;
                guard.active_client_for_context(&cluster_id)
            };

            if let Some(client) = client {
                let tx = event_tx.clone();
                tracker.spawn(async move {
                    match fetch_statefulsets(&client, None).await {
                        Ok(statefulsets) => {
                            let _ = tx.send(ClusterBridgeEvent::StatefulSetsLoaded {
                                cluster_id,
                                generation,
                                statefulsets,
                            });
                        }
                        Err(message) => {
                            let _ = tx.send(ClusterBridgeEvent::StatefulSetsLoadFailed {
                                cluster_id,
                                generation,
                                message,
                            });
                        }
                    }
                });
            } else {
                let _ = event_tx.send(ClusterBridgeEvent::StatefulSetsLoadFailed {
                    cluster_id: cluster_id.clone(),
                    generation,
                    message: format!("No active Kubernetes client for cluster '{cluster_id}'"),
                });
            }
        }
        ClusterBridgeCommand::LoadReplicaSets {
            cluster_id,
            generation,
        } => {
            let client = {
                let guard = manager.read().await;
                guard.active_client_for_context(&cluster_id)
            };

            if let Some(client) = client {
                let tx = event_tx.clone();
                tracker.spawn(async move {
                    match fetch_replicasets(&client, None).await {
                        Ok(replicasets) => {
                            let _ = tx.send(ClusterBridgeEvent::ReplicaSetsLoaded {
                                cluster_id,
                                generation,
                                replicasets,
                            });
                        }
                        Err(message) => {
                            let _ = tx.send(ClusterBridgeEvent::ReplicaSetsLoadFailed {
                                cluster_id,
                                generation,
                                message,
                            });
                        }
                    }
                });
            } else {
                let _ = event_tx.send(ClusterBridgeEvent::ReplicaSetsLoadFailed {
                    cluster_id: cluster_id.clone(),
                    generation,
                    message: format!("No active Kubernetes client for cluster '{cluster_id}'"),
                });
            }
        }
        ClusterBridgeCommand::LoadDaemonSets {
            cluster_id,
            generation,
        } => {
            let client = {
                let guard = manager.read().await;
                guard.active_client_for_context(&cluster_id)
            };

            if let Some(client) = client {
                let tx = event_tx.clone();
                tracker.spawn(async move {
                    match fetch_daemonsets(&client, None).await {
                        Ok(daemonsets) => {
                            let _ = tx.send(ClusterBridgeEvent::DaemonSetsLoaded {
                                cluster_id,
                                generation,
                                daemonsets,
                            });
                        }
                        Err(message) => {
                            let _ = tx.send(ClusterBridgeEvent::DaemonSetsLoadFailed {
                                cluster_id,
                                generation,
                                message,
                            });
                        }
                    }
                });
            } else {
                let _ = event_tx.send(ClusterBridgeEvent::DaemonSetsLoadFailed {
                    cluster_id: cluster_id.clone(),
                    generation,
                    message: format!("No active Kubernetes client for cluster '{cluster_id}'"),
                });
            }
        }
        ClusterBridgeCommand::LoadJobs {
            cluster_id,
            generation,
        } => {
            let client = {
                let guard = manager.read().await;
                guard.active_client_for_context(&cluster_id)
            };

            if let Some(client) = client {
                let tx = event_tx.clone();
                tracker.spawn(async move {
                    match fetch_jobs(&client, None).await {
                        Ok(jobs) => {
                            let _ = tx.send(ClusterBridgeEvent::JobsLoaded {
                                cluster_id,
                                generation,
                                jobs,
                            });
                        }
                        Err(message) => {
                            let _ = tx.send(ClusterBridgeEvent::JobsLoadFailed {
                                cluster_id,
                                generation,
                                message,
                            });
                        }
                    }
                });
            } else {
                let _ = event_tx.send(ClusterBridgeEvent::JobsLoadFailed {
                    cluster_id: cluster_id.clone(),
                    generation,
                    message: format!("No active Kubernetes client for cluster '{cluster_id}'"),
                });
            }
        }
        ClusterBridgeCommand::LoadCronJobs {
            cluster_id,
            generation,
        } => {
            let client = {
                let guard = manager.read().await;
                guard.active_client_for_context(&cluster_id)
            };

            if let Some(client) = client {
                let tx = event_tx.clone();
                tracker.spawn(async move {
                    match fetch_cronjobs(&client, None).await {
                        Ok(cronjobs) => {
                            let _ = tx.send(ClusterBridgeEvent::CronJobsLoaded {
                                cluster_id,
                                generation,
                                cronjobs,
                            });
                        }
                        Err(message) => {
                            let _ = tx.send(ClusterBridgeEvent::CronJobsLoadFailed {
                                cluster_id,
                                generation,
                                message,
                            });
                        }
                    }
                });
            } else {
                let _ = event_tx.send(ClusterBridgeEvent::CronJobsLoadFailed {
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
                tracker.spawn(async move {
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
        ClusterBridgeCommand::LoadServices {
            cluster_id,
            generation,
        } => {
            let client = {
                let guard = manager.read().await;
                guard.active_client_for_context(&cluster_id)
            };

            if let Some(client) = client {
                let tx = event_tx.clone();
                tracker.spawn(async move {
                    match fetch_services(&client, None).await {
                        Ok(services) => {
                            let _ = tx.send(ClusterBridgeEvent::ServicesLoaded {
                                cluster_id,
                                generation,
                                services,
                            });
                        }
                        Err(message) => {
                            let _ = tx.send(ClusterBridgeEvent::ServicesLoadFailed {
                                cluster_id,
                                generation,
                                message,
                            });
                        }
                    }
                });
            } else {
                let _ = event_tx.send(ClusterBridgeEvent::ServicesLoadFailed {
                    cluster_id: cluster_id.clone(),
                    generation,
                    message: format!("No active Kubernetes client for cluster '{cluster_id}'"),
                });
            }
        }
        ClusterBridgeCommand::LoadIngresses {
            cluster_id,
            generation,
        } => {
            let client = {
                let guard = manager.read().await;
                guard.active_client_for_context(&cluster_id)
            };

            if let Some(client) = client {
                let tx = event_tx.clone();
                tracker.spawn(async move {
                    match fetch_ingresses(&client, None).await {
                        Ok(ingresses) => {
                            let _ = tx.send(ClusterBridgeEvent::IngressesLoaded {
                                cluster_id,
                                generation,
                                ingresses,
                            });
                        }
                        Err(message) => {
                            let _ = tx.send(ClusterBridgeEvent::IngressesLoadFailed {
                                cluster_id,
                                generation,
                                message,
                            });
                        }
                    }
                });
            } else {
                let _ = event_tx.send(ClusterBridgeEvent::IngressesLoadFailed {
                    cluster_id: cluster_id.clone(),
                    generation,
                    message: format!("No active Kubernetes client for cluster '{cluster_id}'"),
                });
            }
        }
        ClusterBridgeCommand::LoadConfigMaps {
            cluster_id,
            generation,
        } => {
            let client = {
                let guard = manager.read().await;
                guard.active_client_for_context(&cluster_id)
            };

            if let Some(client) = client {
                let tx = event_tx.clone();
                tracker.spawn(async move {
                    match fetch_configmaps(&client, None).await {
                        Ok(configmaps) => {
                            let _ = tx.send(ClusterBridgeEvent::ConfigMapsLoaded {
                                cluster_id,
                                generation,
                                configmaps,
                            });
                        }
                        Err(message) => {
                            let _ = tx.send(ClusterBridgeEvent::ConfigMapsLoadFailed {
                                cluster_id,
                                generation,
                                message,
                            });
                        }
                    }
                });
            } else {
                let _ = event_tx.send(ClusterBridgeEvent::ConfigMapsLoadFailed {
                    cluster_id: cluster_id.clone(),
                    generation,
                    message: format!("No active Kubernetes client for cluster '{cluster_id}'"),
                });
            }
        }
        ClusterBridgeCommand::LoadSecrets {
            cluster_id,
            generation,
        } => {
            let client = {
                let guard = manager.read().await;
                guard.active_client_for_context(&cluster_id)
            };

            if let Some(client) = client {
                let tx = event_tx.clone();
                tracker.spawn(async move {
                    match fetch_secrets(&client, None).await {
                        Ok(secrets) => {
                            let _ = tx.send(ClusterBridgeEvent::SecretsLoaded {
                                cluster_id,
                                generation,
                                secrets,
                            });
                        }
                        Err(message) => {
                            let _ = tx.send(ClusterBridgeEvent::SecretsLoadFailed {
                                cluster_id,
                                generation,
                                message,
                            });
                        }
                    }
                });
            } else {
                let _ = event_tx.send(ClusterBridgeEvent::SecretsLoadFailed {
                    cluster_id: cluster_id.clone(),
                    generation,
                    message: format!("No active Kubernetes client for cluster '{cluster_id}'"),
                });
            }
        }
        ClusterBridgeCommand::LoadPersistentVolumes {
            cluster_id,
            generation,
        } => {
            let client = {
                let guard = manager.read().await;
                guard.active_client_for_context(&cluster_id)
            };

            if let Some(client) = client {
                let tx = event_tx.clone();
                tracker.spawn(async move {
                    match fetch_pvs(&client).await {
                        Ok(persistent_volumes) => {
                            let _ = tx.send(ClusterBridgeEvent::PersistentVolumesLoaded {
                                cluster_id,
                                generation,
                                persistent_volumes,
                            });
                        }
                        Err(message) => {
                            let _ = tx.send(ClusterBridgeEvent::PersistentVolumesLoadFailed {
                                cluster_id,
                                generation,
                                message,
                            });
                        }
                    }
                });
            } else {
                let _ = event_tx.send(ClusterBridgeEvent::PersistentVolumesLoadFailed {
                    cluster_id: cluster_id.clone(),
                    generation,
                    message: format!("No active Kubernetes client for cluster '{cluster_id}'"),
                });
            }
        }
        ClusterBridgeCommand::LoadPersistentVolumeClaims {
            cluster_id,
            generation,
        } => {
            let client = {
                let guard = manager.read().await;
                guard.active_client_for_context(&cluster_id)
            };

            if let Some(client) = client {
                let tx = event_tx.clone();
                tracker.spawn(async move {
                    match fetch_pvcs(&client, None).await {
                        Ok(persistent_volume_claims) => {
                            let _ = tx.send(ClusterBridgeEvent::PersistentVolumeClaimsLoaded {
                                cluster_id,
                                generation,
                                persistent_volume_claims,
                            });
                        }
                        Err(message) => {
                            let _ = tx.send(ClusterBridgeEvent::PersistentVolumeClaimsLoadFailed {
                                cluster_id,
                                generation,
                                message,
                            });
                        }
                    }
                });
            } else {
                let _ = event_tx.send(ClusterBridgeEvent::PersistentVolumeClaimsLoadFailed {
                    cluster_id: cluster_id.clone(),
                    generation,
                    message: format!("No active Kubernetes client for cluster '{cluster_id}'"),
                });
            }
        }
        ClusterBridgeCommand::LoadStorageOverview {
            cluster_id,
            generation,
        } => {
            let client = {
                let guard = manager.read().await;
                guard.active_client_for_context(&cluster_id)
            };

            if let Some(client) = client {
                let tx = event_tx.clone();
                tracker.spawn(async move {
                    let snapshot = fetch_storage_overview(&client).await;
                    let _ = tx.send(ClusterBridgeEvent::StorageOverviewLoaded {
                        cluster_id,
                        generation,
                        snapshot,
                    });
                });
            } else {
                let _ = event_tx.send(ClusterBridgeEvent::StorageOverviewLoadFailed {
                    cluster_id: cluster_id.clone(),
                    generation,
                    message: format!("No active Kubernetes client for cluster '{cluster_id}'"),
                });
            }
        }
        ClusterBridgeCommand::LoadNetworkPolicies {
            cluster_id,
            generation,
        } => {
            let client = {
                let guard = manager.read().await;
                guard.active_client_for_context(&cluster_id)
            };

            if let Some(client) = client {
                let tx = event_tx.clone();
                tracker.spawn(async move {
                    match fetch_network_policies(&client, None).await {
                        Ok(network_policies) => {
                            let _ = tx.send(ClusterBridgeEvent::NetworkPoliciesLoaded {
                                cluster_id,
                                generation,
                                network_policies,
                            });
                        }
                        Err(message) => {
                            let _ = tx.send(ClusterBridgeEvent::NetworkPoliciesLoadFailed {
                                cluster_id,
                                generation,
                                message,
                            });
                        }
                    }
                });
            } else {
                let _ = event_tx.send(ClusterBridgeEvent::NetworkPoliciesLoadFailed {
                    cluster_id: cluster_id.clone(),
                    generation,
                    message: format!("No active Kubernetes client for cluster '{cluster_id}'"),
                });
            }
        }
        ClusterBridgeCommand::LoadPodSecurity {
            cluster_id,
            generation,
        } => {
            let client = {
                let guard = manager.read().await;
                guard.active_client_for_context(&cluster_id)
            };

            if let Some(client) = client {
                let tx = event_tx.clone();
                tracker.spawn(async move {
                    match fetch_pod_security_policies(&client, None).await {
                        Ok(policies) => {
                            let _ = tx.send(ClusterBridgeEvent::PodSecurityLoaded {
                                cluster_id,
                                generation,
                                policies,
                            });
                        }
                        Err(message) => {
                            let _ = tx.send(ClusterBridgeEvent::PodSecurityLoadFailed {
                                cluster_id,
                                generation,
                                message,
                            });
                        }
                    }
                });
            } else {
                let _ = event_tx.send(ClusterBridgeEvent::PodSecurityLoadFailed {
                    cluster_id: cluster_id.clone(),
                    generation,
                    message: format!("No active Kubernetes client for cluster '{cluster_id}'"),
                });
            }
        }
        ClusterBridgeCommand::LoadHpas {
            cluster_id,
            generation,
        } => {
            let client = {
                let guard = manager.read().await;
                guard.active_client_for_context(&cluster_id)
            };

            if let Some(client) = client {
                let tx = event_tx.clone();
                tracker.spawn(async move {
                    match fetch_hpas(&client, None).await {
                        Ok(hpas) => {
                            let _ = tx.send(ClusterBridgeEvent::HpasLoaded {
                                cluster_id,
                                generation,
                                hpas,
                            });
                        }
                        Err(message) => {
                            let _ = tx.send(ClusterBridgeEvent::HpasLoadFailed {
                                cluster_id,
                                generation,
                                message,
                            });
                        }
                    }
                });
            } else {
                let _ = event_tx.send(ClusterBridgeEvent::HpasLoadFailed {
                    cluster_id: cluster_id.clone(),
                    generation,
                    message: format!("No active Kubernetes client for cluster '{cluster_id}'"),
                });
            }
        }
        ClusterBridgeCommand::LoadStorageClasses {
            cluster_id,
            generation,
        } => {
            let client = {
                let guard = manager.read().await;
                guard.active_client_for_context(&cluster_id)
            };

            if let Some(client) = client {
                let tx = event_tx.clone();
                tracker.spawn(async move {
                    match fetch_storage_classes(&client).await {
                        Ok(classes) => {
                            let _ = tx.send(ClusterBridgeEvent::StorageClassesLoaded {
                                cluster_id,
                                generation,
                                classes,
                            });
                        }
                        Err(message) => {
                            let _ = tx.send(ClusterBridgeEvent::StorageClassesLoadFailed {
                                cluster_id,
                                generation,
                                message,
                            });
                        }
                    }
                });
            } else {
                let _ = event_tx.send(ClusterBridgeEvent::StorageClassesLoadFailed {
                    cluster_id: cluster_id.clone(),
                    generation,
                    message: format!("No active Kubernetes client for cluster '{cluster_id}'"),
                });
            }
        }
        ClusterBridgeCommand::LoadSecurityOverview {
            cluster_id,
            generation,
        } => {
            let client = {
                let guard = manager.read().await;
                guard.active_client_for_context(&cluster_id)
            };

            if let Some(client) = client {
                let tx = event_tx.clone();
                tracker.spawn(async move {
                    match fetch_policies(&client, None).await {
                        Ok(policies) => {
                            let _ = tx.send(ClusterBridgeEvent::SecurityOverviewLoaded {
                                cluster_id,
                                generation,
                                policies,
                            });
                        }
                        Err(message) => {
                            let _ = tx.send(ClusterBridgeEvent::SecurityOverviewLoadFailed {
                                cluster_id,
                                generation,
                                message,
                            });
                        }
                    }
                });
            } else {
                let _ = event_tx.send(ClusterBridgeEvent::SecurityOverviewLoadFailed {
                    cluster_id: cluster_id.clone(),
                    generation,
                    message: format!("No active Kubernetes client for cluster '{cluster_id}'"),
                });
            }
        }
        ClusterBridgeCommand::LoadAdmissionWebhooks {
            cluster_id,
            generation,
        } => {
            let client = {
                let guard = manager.read().await;
                guard.active_client_for_context(&cluster_id)
            };

            if let Some(client) = client {
                let tx = event_tx.clone();
                tracker.spawn(async move {
                    match fetch_admission_webhooks(&client).await {
                        Ok(policies) => {
                            let _ = tx.send(ClusterBridgeEvent::AdmissionWebhooksLoaded {
                                cluster_id,
                                generation,
                                policies,
                            });
                        }
                        Err(message) => {
                            let _ = tx.send(ClusterBridgeEvent::AdmissionWebhooksLoadFailed {
                                cluster_id,
                                generation,
                                message,
                            });
                        }
                    }
                });
            } else {
                let _ = event_tx.send(ClusterBridgeEvent::AdmissionWebhooksLoadFailed {
                    cluster_id: cluster_id.clone(),
                    generation,
                    message: format!("No active Kubernetes client for cluster '{cluster_id}'"),
                });
            }
        }
        ClusterBridgeCommand::LoadResourceQuotas {
            cluster_id,
            generation,
        } => {
            let client = {
                let guard = manager.read().await;
                guard.active_client_for_context(&cluster_id)
            };

            if let Some(client) = client {
                let tx = event_tx.clone();
                tracker.spawn(async move {
                    match fetch_resource_quotas_and_limits(&client, None).await {
                        Ok(policies) => {
                            let _ = tx.send(ClusterBridgeEvent::ResourceQuotasLoaded {
                                cluster_id,
                                generation,
                                policies,
                            });
                        }
                        Err(message) => {
                            let _ = tx.send(ClusterBridgeEvent::ResourceQuotasLoadFailed {
                                cluster_id,
                                generation,
                                message,
                            });
                        }
                    }
                });
            } else {
                let _ = event_tx.send(ClusterBridgeEvent::ResourceQuotasLoadFailed {
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
                tracker.spawn(async move {
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
                tracker.spawn(async move {
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
                tracker.spawn(async move {
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
            if let Err(err) =
                guard
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
        ClusterBridgeCommand::SaveTableConfig { table_id, config } => {
            let mut guard = manager.write().await;
            guard.config.update_table_config(&table_id, config);
            if let Err(message) = guard.config.save() {
                tracing::warn!(table_id = %table_id, error = %message, "Failed to persist table config");
                let _ = event_tx.send(ClusterBridgeEvent::SettingsSaveFailed { message });
            }
        }
        ClusterBridgeCommand::SaveClusterTableFilters {
            cluster_id,
            table_id,
            filters,
        } => {
            let mut guard = manager.write().await;
            guard
                .config
                .update_cluster_table_filters(&cluster_id, &table_id, filters);
            if let Err(message) = guard.config.save() {
                tracing::warn!(
                    cluster_id = %cluster_id,
                    table_id = %table_id,
                    error = %message,
                    "Failed to persist cluster table filters"
                );
                let _ = event_tx.send(ClusterBridgeEvent::SettingsSaveFailed { message });
            }
        }
        ClusterBridgeCommand::SaveThemeMode { mode } => {
            let mut guard = manager.write().await;
            guard.config.update_theme_mode(&mode);
            if let Err(message) = guard.config.save() {
                tracing::warn!(error = %message, "Failed to persist theme mode");
                let _ = event_tx.send(ClusterBridgeEvent::SettingsSaveFailed { message });
            }
        }
        ClusterBridgeCommand::SaveEnableExperienceFeatures { enabled } => {
            let mut guard = manager.write().await;
            guard.config.update_enable_experience_features(enabled);
            if let Err(message) = guard.config.save() {
                tracing::warn!(error = %message, "Failed to persist experience features toggle");
                let _ = event_tx.send(ClusterBridgeEvent::SettingsSaveFailed { message });
            }
        }
        ClusterBridgeCommand::AddCluster(request) => {
            let mut guard = manager.write().await;
            match guard.register_kubeconfig(request).await {
                Ok(outcome) => {
                    let _ = event_tx.send(ClusterBridgeEvent::ClusterAdded {
                        added_contexts: outcome.added_contexts,
                        clusters: outcome.clusters,
                        switch_outcome: outcome.switch_outcome,
                        profile: outcome.profile,
                    });
                }
                Err(message) => {
                    let _ = event_tx.send(ClusterBridgeEvent::AddClusterFailed { message });
                }
            }
        }
        ClusterBridgeCommand::ScaleWorkload {
            cluster_id,
            namespace,
            kind,
            name,
            replicas,
        } => {
            let client = {
                let guard = manager.read().await;
                guard.active_client_for_context(&cluster_id)
            };

            if let Some(client) = client {
                let tx = event_tx.clone();
                tracker.spawn(async move {
                    match execute_scale_resource(&client, &namespace, &kind, &name, replicas).await
                    {
                        Ok(()) => {
                            let _ = tx.send(ClusterBridgeEvent::WorkloadActionSucceeded {
                                cluster_id,
                                action: WorkloadActionKind::Scale,
                                kind: kind.clone(),
                                namespace: namespace.clone(),
                                name: name.clone(),
                                message: format!(
                                    "Scaled {kind} '{namespace}/{name}' to {replicas} replica(s)."
                                ),
                            });
                        }
                        Err(message) => {
                            let _ = tx.send(ClusterBridgeEvent::WorkloadActionFailed {
                                cluster_id,
                                action: WorkloadActionKind::Scale,
                                kind,
                                namespace,
                                name,
                                message,
                            });
                        }
                    }
                });
            } else {
                let _ = event_tx.send(ClusterBridgeEvent::WorkloadActionFailed {
                    cluster_id: cluster_id.clone(),
                    action: WorkloadActionKind::Scale,
                    kind,
                    namespace,
                    name,
                    message: format!("No active Kubernetes client for cluster '{cluster_id}'"),
                });
            }
        }
        ClusterBridgeCommand::RestartWorkload {
            cluster_id,
            namespace,
            kind,
            name,
        } => {
            let client = {
                let guard = manager.read().await;
                guard.active_client_for_context(&cluster_id)
            };

            if let Some(client) = client {
                let tx = event_tx.clone();
                tracker.spawn(async move {
                    match execute_restart_resource(&client, &namespace, &kind, &name).await {
                        Ok(()) => {
                            let _ = tx.send(ClusterBridgeEvent::WorkloadActionSucceeded {
                                cluster_id,
                                action: WorkloadActionKind::Restart,
                                kind: kind.clone(),
                                namespace: namespace.clone(),
                                name: name.clone(),
                                message: format!(
                                    "Triggered rollout restart for {kind} '{namespace}/{name}'."
                                ),
                            });
                        }
                        Err(message) => {
                            let _ = tx.send(ClusterBridgeEvent::WorkloadActionFailed {
                                cluster_id,
                                action: WorkloadActionKind::Restart,
                                kind,
                                namespace,
                                name,
                                message,
                            });
                        }
                    }
                });
            } else {
                let _ = event_tx.send(ClusterBridgeEvent::WorkloadActionFailed {
                    cluster_id: cluster_id.clone(),
                    action: WorkloadActionKind::Restart,
                    kind,
                    namespace,
                    name,
                    message: format!("No active Kubernetes client for cluster '{cluster_id}'"),
                });
            }
        }
        ClusterBridgeCommand::RestartPod {
            cluster_id,
            namespace,
            name,
        } => {
            let client = {
                let guard = manager.read().await;
                guard.active_client_for_context(&cluster_id)
            };

            if let Some(client) = client {
                let tx = event_tx.clone();
                tracker.spawn(async move {
                    match execute_restart_pod(&client, &namespace, &name).await {
                        Ok(()) => {
                            let _ = tx.send(ClusterBridgeEvent::WorkloadActionSucceeded {
                                cluster_id,
                                action: WorkloadActionKind::RestartPod,
                                kind: "Pod".to_string(),
                                namespace: namespace.clone(),
                                name: name.clone(),
                                message: format!("Deleted Pod '{namespace}/{name}' for restart."),
                            });
                        }
                        Err(message) => {
                            let _ = tx.send(ClusterBridgeEvent::WorkloadActionFailed {
                                cluster_id,
                                action: WorkloadActionKind::RestartPod,
                                kind: "Pod".to_string(),
                                namespace,
                                name,
                                message,
                            });
                        }
                    }
                });
            } else {
                let _ = event_tx.send(ClusterBridgeEvent::WorkloadActionFailed {
                    cluster_id: cluster_id.clone(),
                    action: WorkloadActionKind::RestartPod,
                    kind: "Pod".to_string(),
                    namespace,
                    name,
                    message: format!("No active Kubernetes client for cluster '{cluster_id}'"),
                });
            }
        }
        ClusterBridgeCommand::TriggerCronJob {
            cluster_id,
            namespace,
            name,
        } => {
            let client = {
                let guard = manager.read().await;
                guard.active_client_for_context(&cluster_id)
            };

            if let Some(client) = client {
                let tx = event_tx.clone();
                tracker.spawn(async move {
                    match execute_trigger_cronjob(&client, &namespace, &name).await {
                        Ok(job_name) => {
                            let _ = tx.send(ClusterBridgeEvent::WorkloadActionSucceeded {
                                cluster_id,
                                action: WorkloadActionKind::TriggerCronJob,
                                kind: "CronJob".to_string(),
                                namespace: namespace.clone(),
                                name: name.clone(),
                                message: format!(
                                    "Triggered CronJob '{namespace}/{name}' as Job '{job_name}'."
                                ),
                            });
                        }
                        Err(message) => {
                            let _ = tx.send(ClusterBridgeEvent::WorkloadActionFailed {
                                cluster_id,
                                action: WorkloadActionKind::TriggerCronJob,
                                kind: "CronJob".to_string(),
                                namespace,
                                name,
                                message,
                            });
                        }
                    }
                });
            } else {
                let _ = event_tx.send(ClusterBridgeEvent::WorkloadActionFailed {
                    cluster_id: cluster_id.clone(),
                    action: WorkloadActionKind::TriggerCronJob,
                    kind: "CronJob".to_string(),
                    namespace,
                    name,
                    message: format!("No active Kubernetes client for cluster '{cluster_id}'"),
                });
            }
        }
        ClusterBridgeCommand::DeleteResource {
            cluster_id,
            namespace,
            kind,
            name,
        } => {
            let client = {
                let guard = manager.read().await;
                guard.active_client_for_context(&cluster_id)
            };
            let normalized_ns = namespace
                .as_deref()
                .map(str::trim)
                .filter(|ns| !ns.is_empty())
                .map(str::to_string);
            let event_namespace = normalized_ns.clone().unwrap_or_default();

            if let Some(client) = client {
                let tx = event_tx.clone();
                tracker.spawn(async move {
                    let target = format_target_label(normalized_ns.as_deref(), &name);
                    match execute_delete_resource(&client, normalized_ns.as_deref(), &kind, &name)
                        .await
                    {
                        Ok(()) => {
                            let _ = tx.send(ClusterBridgeEvent::WorkloadActionSucceeded {
                                cluster_id,
                                action: WorkloadActionKind::Delete,
                                kind: kind.clone(),
                                namespace: event_namespace,
                                name: name.clone(),
                                message: format!("Deleted {kind} '{target}'."),
                            });
                        }
                        Err(message) => {
                            let _ = tx.send(ClusterBridgeEvent::WorkloadActionFailed {
                                cluster_id,
                                action: WorkloadActionKind::Delete,
                                kind,
                                namespace: event_namespace,
                                name,
                                message,
                            });
                        }
                    }
                });
            } else {
                let _ = event_tx.send(ClusterBridgeEvent::WorkloadActionFailed {
                    cluster_id: cluster_id.clone(),
                    action: WorkloadActionKind::Delete,
                    kind,
                    namespace: event_namespace,
                    name,
                    message: format!("No active Kubernetes client for cluster '{cluster_id}'"),
                });
            }
        }
        ClusterBridgeCommand::StartLogStream {
            cluster_id,
            stream_id,
            namespace,
            workload_kind,
            workload_name,
            pod_filter,
            container_filter,
            tail_lines,
            include_unavailable_pods,
        } => {
            tracker.log_streams.stop(stream_id);

            let client = {
                let guard = manager.read().await;
                guard.active_client.clone()
            };
            let Some(client) = client else {
                let _ = event_tx.send(ClusterBridgeEvent::LogStreamStatus {
                    cluster_id: cluster_id.clone(),
                    stream_id,
                    message: format!("No active Kubernetes client for cluster '{cluster_id}'"),
                    is_error: true,
                });
                let _ = event_tx.send(ClusterBridgeEvent::LogStreamEnded {
                    cluster_id,
                    stream_id,
                });
                return;
            };

            let pods = match resolve_workload_pods(&client, &namespace, &workload_name, &workload_kind).await {
                Ok(pods) => pods,
                Err(err) => {
                    let _ = event_tx.send(ClusterBridgeEvent::LogStreamStatus {
                        cluster_id: cluster_id.clone(),
                        stream_id,
                        message: format!("Failed to resolve pods for {workload_kind} '{workload_name}': {err}"),
                        is_error: true,
                    });
                    let _ = event_tx.send(ClusterBridgeEvent::LogStreamEnded {
                        cluster_id,
                        stream_id,
                    });
                    return;
                }
            };

            let (selected_pods, excluded_unavailable_count) =
                select_streamable_pods(&pods, pod_filter.as_deref(), include_unavailable_pods);

            let _ = event_tx.send(ClusterBridgeEvent::LogTargetsResolved {
                cluster_id: cluster_id.clone(),
                stream_id,
                pods: pods.clone(),
                active_pod_count: selected_pods.len(),
                excluded_unavailable_count,
            });

            if selected_pods.is_empty() {
                let msg = if excluded_unavailable_count > 0 {
                    format!(
                        "No available pods to stream ({excluded_unavailable_count} starting or terminating pod(s) excluded). Enable \"Include unavailable\" to stream them."
                    )
                } else {
                    format!("No pods found for {workload_kind} '{workload_name}'.")
                };
                let _ = event_tx.send(ClusterBridgeEvent::LogStreamStatus {
                    cluster_id: cluster_id.clone(),
                    stream_id,
                    message: msg,
                    is_error: false,
                });
                let _ = event_tx.send(ClusterBridgeEvent::LogStreamEnded {
                    cluster_id,
                    stream_id,
                });
                return;
            }

            let normalized_container = container_filter
                .filter(|c| !c.is_empty() && c != "All" && c != "all");
            let normalized_tail = (tail_lines > 0).then_some(tail_lines);

            let (msg_tx, mut msg_rx) = mpsc::unbounded_channel::<LogStreamMessage>();
            let mut cancel_txs = Vec::new();
            let mut abort_handles = Vec::new();

            for pod_target in selected_pods {
                let containers_to_stream: Vec<Option<String>> = if let Some(ref c) = normalized_container {
                    if pod_target.containers.is_empty() || pod_target.containers.contains(c) {
                        vec![Some(c.clone())]
                    } else {
                        Vec::new()
                    }
                } else if pod_target.containers.len() > 1 {
                    pod_target.containers.iter().cloned().map(Some).collect()
                } else {
                    vec![pod_target.containers.first().cloned()]
                };

                for container in containers_to_stream {
                    let (cancel_tx, cancel_rx) = tokio::sync::oneshot::channel();
                    cancel_txs.push(cancel_tx);

                    let handle = tokio::spawn(stream_pod_logs(
                        client.clone(),
                        namespace.clone(),
                        pod_target.name.clone(),
                        container,
                        normalized_tail,
                        msg_tx.clone(),
                        cancel_rx,
                    ))
                    .abort_handle();
                    abort_handles.push(handle);
                }
            }

            drop(msg_tx);

            let fwd_tx = event_tx.clone();
            let fwd_cluster_id = cluster_id.clone();
            let fwd_handle = tokio::spawn(async move {
                while let Some(msg) = msg_rx.recv().await {
                    match msg {
                        LogStreamMessage::Lines {
                            pod,
                            container,
                            lines,
                        } => {
                            let _ = fwd_tx.send(ClusterBridgeEvent::LogLinesReceived {
                                cluster_id: fwd_cluster_id.clone(),
                                stream_id,
                                pod,
                                container,
                                lines,
                            });
                        }
                        LogStreamMessage::SystemNotice {
                            pod,
                            container,
                            message,
                        } => {
                            let _ = fwd_tx.send(ClusterBridgeEvent::LogLinesReceived {
                                cluster_id: fwd_cluster_id.clone(),
                                stream_id,
                                pod,
                                container,
                                lines: vec![message],
                            });
                        }
                        LogStreamMessage::Error { message, .. } => {
                            let _ = fwd_tx.send(ClusterBridgeEvent::LogStreamStatus {
                                cluster_id: fwd_cluster_id.clone(),
                                stream_id,
                                message,
                                is_error: true,
                            });
                        }
                        LogStreamMessage::Ended { .. } => {}
                    }
                }
                let _ = fwd_tx.send(ClusterBridgeEvent::LogStreamEnded {
                    cluster_id: fwd_cluster_id,
                    stream_id,
                });
            })
            .abort_handle();
            abort_handles.push(fwd_handle);

            tracker
                .log_streams
                .register(stream_id, cancel_txs, abort_handles);
        }
        ClusterBridgeCommand::StopLogStream { stream_id } => {
            tracker.log_streams.stop(stream_id);
        }
        ClusterBridgeCommand::LoadPortForwards {
            cluster_id,
            generation,
        } => {
            let port_forwards = get_persisted_for_cluster(&cluster_id);
            let _ = event_tx.send(ClusterBridgeEvent::PortForwardsLoaded {
                cluster_id: cluster_id.clone(),
                generation,
                port_forwards,
            });
            let is_active = {
                let guard = manager.read().await;
                guard.active_context.as_deref() == Some(cluster_id.as_str())
            };
            if is_active {
                let (pf_tx, pf_rx) = mpsc::unbounded_channel();
                spawn_port_forward_event_forwarder(
                    cluster_id.clone(),
                    pf_rx,
                    event_tx.clone(),
                );
                let manager_clone = manager.clone();
                tokio::spawn(async move {
                    restore_cluster_port_forwards(manager_clone, cluster_id, pf_tx).await;
                });
            }
        }
        ClusterBridgeCommand::StartPortForward {
            cluster_id,
            namespace,
            kind,
            name,
            local_port,
            remote_port,
        } => {
            let is_active = {
                let guard = manager.read().await;
                guard.active_context.as_deref() == Some(cluster_id.as_str())
            };
            if !is_active {
                let _ = event_tx.send(ClusterBridgeEvent::PortForwardFailed {
                    cluster_id,
                    id: None,
                    message: "No active Kubernetes context for port forwarding".to_string(),
                });
                return;
            }
            let (pf_tx, pf_rx) = mpsc::unbounded_channel();
            spawn_port_forward_event_forwarder(cluster_id, pf_rx, event_tx.clone());
            let manager_clone = manager.clone();
            let request = PortForwardRequest {
                namespace,
                kind,
                name,
                local_port,
                remote_port,
            };
            tokio::spawn(async move {
                start_port_forward_tunnel(manager_clone, request, pf_tx).await;
            });
        }
        ClusterBridgeCommand::StopPortForward { cluster_id, id } => {
            stop_port_forward_tunnel(manager, Some(&cluster_id), &id).await;
            let _ = event_tx.send(ClusterBridgeEvent::PortForwardStopped { cluster_id, id });
        }
        ClusterBridgeCommand::StopSelectedPortForwards { cluster_id, ids } => {
            for id in ids {
                stop_port_forward_tunnel(manager, Some(&cluster_id), &id).await;
                let _ = event_tx.send(ClusterBridgeEvent::PortForwardStopped {
                    cluster_id: cluster_id.clone(),
                    id,
                });
            }
        }
        ClusterBridgeCommand::StopAllPortForwards { cluster_id } => {
            stop_cluster_port_forwards(manager, &cluster_id).await;
            let _ = event_tx.send(ClusterBridgeEvent::PortForwardsCleared { cluster_id });
        }
    }
}

fn spawn_port_forward_event_forwarder(
    cluster_id: String,
    mut pf_rx: mpsc::UnboundedReceiver<PortForwardLifecycleEvent>,
    event_tx: mpsc::UnboundedSender<ClusterBridgeEvent>,
) {
    tokio::spawn(async move {
        while let Some(ev) = pf_rx.recv().await {
            let mapped = match ev {
                PortForwardLifecycleEvent::Started(port_forward) => {
                    ClusterBridgeEvent::PortForwardStarted {
                        cluster_id: cluster_id.clone(),
                        port_forward,
                    }
                }
                PortForwardLifecycleEvent::Stopped { id } => {
                    ClusterBridgeEvent::PortForwardStopped {
                        cluster_id: cluster_id.clone(),
                        id,
                    }
                }
                PortForwardLifecycleEvent::Error { id, message } => {
                    ClusterBridgeEvent::PortForwardFailed {
                        cluster_id: cluster_id.clone(),
                        id,
                        message,
                    }
                }
            };
            if event_tx.send(mapped).is_err() {
                break;
            }
        }
    });
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
                let mut tracker = BridgeTaskTracker::default();
                handle_bridge_command_with_tracker(
                    ClusterBridgeCommand::LoadContexts,
                    &manager,
                    &thread_event_tx,
                    &mut tracker,
                )
                .await;

                while let Some(cmd) = command_rx.recv().await {
                    handle_bridge_command_with_tracker(
                        cmd,
                        &manager,
                        &thread_event_tx,
                        &mut tracker,
                    )
                    .await;
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
    async fn test_bridge_save_table_config_and_cluster_filters_persists_to_orbit_config() {
        use orbit_core::config::{
            PersistedClusterTableFilters, PersistedSortDirection, PersistedTableConfig,
        };
        use std::collections::HashMap;

        let _guard = orbit_core::config::TestConfigDirGuard::new("gpui_table_prefs_bridge");
        let manager = Arc::new(RwLock::new(KubeManager::new_empty(OrbitConfig::load())));
        let (event_tx, _event_rx) = mpsc::unbounded_channel();

        handle_bridge_command(
            ClusterBridgeCommand::SaveTableConfig {
                table_id: "pods".to_string(),
                config: PersistedTableConfig {
                    page_size: Some(50),
                    hidden_columns: vec!["node".to_string()],
                    sort_column: Some("name".to_string()),
                    sort_direction: Some(PersistedSortDirection::Asc),
                    ..Default::default()
                },
            },
            &manager,
            &event_tx,
        )
        .await;

        let mut extra = HashMap::new();
        extra.insert("status".to_string(), "Running".to_string());
        handle_bridge_command(
            ClusterBridgeCommand::SaveClusterTableFilters {
                cluster_id: "ctx-a".to_string(),
                table_id: "pods".to_string(),
                filters: PersistedClusterTableFilters {
                    search_query: "nginx".to_string(),
                    extra_filters: extra,
                },
            },
            &manager,
            &event_tx,
        )
        .await;

        let reloaded = OrbitConfig::load();
        let prefs = reloaded.table_preferences();
        assert_eq!(
            prefs.tables.get("pods").and_then(|c| c.page_size),
            Some(50)
        );
        assert_eq!(
            prefs
                .clusters
                .get("ctx-a")
                .and_then(|m| m.get("pods"))
                .map(|f| f.search_query.as_str()),
            Some("nginx")
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
    async fn test_handle_bridge_command_load_statefulsets_without_client_emits_failure() {
        let manager = Arc::new(RwLock::new(KubeManager::new_empty(OrbitConfig::default())));
        let (event_tx, mut event_rx) = mpsc::unbounded_channel();

        handle_bridge_command(
            ClusterBridgeCommand::LoadStatefulSets {
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
            .expect("Expected StatefulSetsLoadFailed event");
        match event {
            ClusterBridgeEvent::StatefulSetsLoadFailed {
                cluster_id,
                generation,
                message,
            } => {
                assert_eq!(cluster_id, "missing-ctx");
                assert_eq!(generation, 9);
                assert!(!message.trim().is_empty());
            }
            other => panic!("Expected StatefulSetsLoadFailed, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn test_handle_bridge_command_load_replicasets_without_client_emits_failure() {
        let manager = Arc::new(RwLock::new(KubeManager::new_empty(OrbitConfig::default())));
        let (event_tx, mut event_rx) = mpsc::unbounded_channel();

        handle_bridge_command(
            ClusterBridgeCommand::LoadReplicaSets {
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
            .expect("Expected ReplicaSetsLoadFailed event");
        match event {
            ClusterBridgeEvent::ReplicaSetsLoadFailed {
                cluster_id,
                generation,
                message,
            } => {
                assert_eq!(cluster_id, "missing-ctx");
                assert_eq!(generation, 9);
                assert!(!message.trim().is_empty());
            }
            other => panic!("Expected ReplicaSetsLoadFailed, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn test_handle_bridge_command_load_daemonsets_without_client_emits_failure() {
        let manager = Arc::new(RwLock::new(KubeManager::new_empty(OrbitConfig::default())));
        let (event_tx, mut event_rx) = mpsc::unbounded_channel();

        handle_bridge_command(
            ClusterBridgeCommand::LoadDaemonSets {
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
            .expect("Expected DaemonSetsLoadFailed event");
        match event {
            ClusterBridgeEvent::DaemonSetsLoadFailed {
                cluster_id,
                generation,
                message,
            } => {
                assert_eq!(cluster_id, "missing-ctx");
                assert_eq!(generation, 11);
                assert!(!message.trim().is_empty());
            }
            other => panic!("Expected DaemonSetsLoadFailed, got {other:?}"),
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

    #[tokio::test]
    async fn test_handle_bridge_command_load_services_without_client_emits_failure() {
        let manager = Arc::new(RwLock::new(KubeManager::new().await));
        let (event_tx, mut event_rx) = mpsc::unbounded_channel();

        handle_bridge_command(
            ClusterBridgeCommand::LoadServices {
                cluster_id: "missing-ctx".to_string(),
                generation: 3,
            },
            &manager,
            &event_tx,
        )
        .await;

        let event = event_rx
            .recv()
            .await
            .expect("Expected ServicesLoadFailed event");
        match event {
            ClusterBridgeEvent::ServicesLoadFailed {
                cluster_id,
                generation,
                message,
            } => {
                assert_eq!(cluster_id, "missing-ctx");
                assert_eq!(generation, 3);
                assert!(!message.trim().is_empty());
            }
            other => panic!("Expected ServicesLoadFailed, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn test_handle_bridge_command_persists_appearance_and_experience_settings() {
        let _guard = orbit_core::config::TestConfigDirGuard::new("bridge_settings_persist");
        let mut seeded = OrbitConfig::default();
        seeded.update_ui_style("glass");
        seeded.update_accent_color("violet");
        seeded.save().expect("seed legacy config");

        let manager = Arc::new(RwLock::new(KubeManager::new_empty(seeded)));
        let (event_tx, mut event_rx) = mpsc::unbounded_channel();

        handle_bridge_command(
            ClusterBridgeCommand::SaveThemeMode {
                mode: "light".to_string(),
            },
            &manager,
            &event_tx,
        )
        .await;
        handle_bridge_command(
            ClusterBridgeCommand::SaveEnableExperienceFeatures { enabled: true },
            &manager,
            &event_tx,
        )
        .await;

        assert!(event_rx.try_recv().is_err());

        let loaded = OrbitConfig::load();
        assert_eq!(loaded.theme_mode(), "light");
        assert!(loaded.enable_experience_features());
        assert_eq!(loaded.ui_style(), "glass");
        assert_eq!(loaded.accent_color(), "violet");

        handle_bridge_command(
            ClusterBridgeCommand::SaveThemeMode {
                mode: "dark".to_string(),
            },
            &manager,
            &event_tx,
        )
        .await;

        assert!(event_rx.try_recv().is_err());

        let after_reset = OrbitConfig::load();
        assert_eq!(after_reset.theme_mode(), "dark");
        assert!(!after_reset.values.contains_key("themeMode"));
        assert_eq!(after_reset.ui_style(), "glass");
        assert_eq!(after_reset.accent_color(), "violet");
        assert!(after_reset.enable_experience_features());
    }

    #[tokio::test]
    async fn test_handle_bridge_command_load_workloads_overview_without_active_client_emits_failure()
     {
        let manager = Arc::new(RwLock::new(KubeManager::new_empty(OrbitConfig::default())));
        let (event_tx, mut event_rx) = mpsc::unbounded_channel();

        handle_bridge_command(
            ClusterBridgeCommand::LoadWorkloadsOverview {
                cluster_id: "missing-ctx".to_string(),
                generation: 3,
            },
            &manager,
            &event_tx,
        )
        .await;

        let event = event_rx
            .recv()
            .await
            .expect("Expected WorkloadsOverviewLoadFailed event");
        match event {
            ClusterBridgeEvent::WorkloadsOverviewLoadFailed {
                cluster_id,
                generation,
                message,
            } => {
                assert_eq!(cluster_id, "missing-ctx");
                assert_eq!(generation, 3);
                assert!(!message.trim().is_empty());
            }
            other => panic!("Expected WorkloadsOverviewLoadFailed, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn test_handle_bridge_command_load_jobs_without_client_emits_failure() {
        let manager = Arc::new(RwLock::new(KubeManager::new_empty(OrbitConfig::default())));
        let (event_tx, mut event_rx) = mpsc::unbounded_channel();

        handle_bridge_command(
            ClusterBridgeCommand::LoadJobs {
                cluster_id: "missing-ctx".to_string(),
                generation: 13,
            },
            &manager,
            &event_tx,
        )
        .await;

        let event = event_rx
            .recv()
            .await
            .expect("Expected JobsLoadFailed event");
        match event {
            ClusterBridgeEvent::JobsLoadFailed {
                cluster_id,
                generation,
                message,
            } => {
                assert_eq!(cluster_id, "missing-ctx");
                assert_eq!(generation, 13);
                assert!(!message.trim().is_empty());
            }
            other => panic!("Expected JobsLoadFailed, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn test_handle_bridge_command_load_cronjobs_without_client_emits_failure() {
        let manager = Arc::new(RwLock::new(KubeManager::new_empty(OrbitConfig::default())));
        let (event_tx, mut event_rx) = mpsc::unbounded_channel();

        handle_bridge_command(
            ClusterBridgeCommand::LoadCronJobs {
                cluster_id: "missing-ctx".to_string(),
                generation: 14,
            },
            &manager,
            &event_tx,
        )
        .await;

        let event = event_rx
            .recv()
            .await
            .expect("Expected CronJobsLoadFailed event");
        match event {
            ClusterBridgeEvent::CronJobsLoadFailed {
                cluster_id,
                generation,
                message,
            } => {
                assert_eq!(cluster_id, "missing-ctx");
                assert_eq!(generation, 14);
                assert!(!message.trim().is_empty());
            }
            other => panic!("Expected CronJobsLoadFailed, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn test_handle_bridge_command_load_ingresses_without_client_emits_failure() {
        let manager = Arc::new(RwLock::new(KubeManager::new_empty(OrbitConfig::default())));
        let (event_tx, mut event_rx) = mpsc::unbounded_channel();

        handle_bridge_command(
            ClusterBridgeCommand::LoadIngresses {
                cluster_id: "missing-ctx".to_string(),
                generation: 15,
            },
            &manager,
            &event_tx,
        )
        .await;

        let event = event_rx
            .recv()
            .await
            .expect("Expected IngressesLoadFailed event");
        match event {
            ClusterBridgeEvent::IngressesLoadFailed {
                cluster_id,
                generation,
                message,
            } => {
                assert_eq!(cluster_id, "missing-ctx");
                assert_eq!(generation, 15);
                assert!(!message.trim().is_empty());
            }
            other => panic!("Expected IngressesLoadFailed, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn test_handle_bridge_command_load_persistent_volumes_without_client_emits_failure() {
        let manager = Arc::new(RwLock::new(KubeManager::new_empty(OrbitConfig::default())));
        let (event_tx, mut event_rx) = mpsc::unbounded_channel();

        handle_bridge_command(
            ClusterBridgeCommand::LoadPersistentVolumes {
                cluster_id: "missing-ctx".to_string(),
                generation: 16,
            },
            &manager,
            &event_tx,
        )
        .await;

        let event = event_rx
            .recv()
            .await
            .expect("Expected PersistentVolumesLoadFailed event");
        match event {
            ClusterBridgeEvent::PersistentVolumesLoadFailed {
                cluster_id,
                generation,
                message,
            } => {
                assert_eq!(cluster_id, "missing-ctx");
                assert_eq!(generation, 16);
                assert!(!message.trim().is_empty());
            }
            other => panic!("Expected PersistentVolumesLoadFailed, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn test_handle_bridge_command_load_persistent_volume_claims_without_client_emits_failure()
    {
        let manager = Arc::new(RwLock::new(KubeManager::new_empty(OrbitConfig::default())));
        let (event_tx, mut event_rx) = mpsc::unbounded_channel();

        handle_bridge_command(
            ClusterBridgeCommand::LoadPersistentVolumeClaims {
                cluster_id: "missing-ctx".to_string(),
                generation: 17,
            },
            &manager,
            &event_tx,
        )
        .await;

        let event = event_rx
            .recv()
            .await
            .expect("Expected PersistentVolumeClaimsLoadFailed event");
        match event {
            ClusterBridgeEvent::PersistentVolumeClaimsLoadFailed {
                cluster_id,
                generation,
                message,
            } => {
                assert_eq!(cluster_id, "missing-ctx");
                assert_eq!(generation, 17);
                assert!(!message.trim().is_empty());
            }
            other => panic!("Expected PersistentVolumeClaimsLoadFailed, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn test_handle_bridge_command_load_network_policies_without_client_emits_failure() {
        let manager = Arc::new(RwLock::new(KubeManager::new_empty(OrbitConfig::default())));
        let (event_tx, mut event_rx) = mpsc::unbounded_channel();

        handle_bridge_command(
            ClusterBridgeCommand::LoadNetworkPolicies {
                cluster_id: "missing-ctx".to_string(),
                generation: 19,
            },
            &manager,
            &event_tx,
        )
        .await;

        let Some(event) = event_rx.recv().await else {
            panic!("Expected NetworkPoliciesLoadFailed event");
        };
        match event {
            ClusterBridgeEvent::NetworkPoliciesLoadFailed {
                cluster_id,
                generation,
                message,
            } => {
                assert_eq!(cluster_id, "missing-ctx");
                assert_eq!(generation, 19);
                assert!(!message.trim().is_empty());
            }
            other => panic!("Expected NetworkPoliciesLoadFailed, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn test_handle_bridge_command_emits_load_failed_when_no_active_client_for_new_views() {
        let manager = Arc::new(RwLock::new(KubeManager::new_empty(OrbitConfig::default())));
        let (event_tx, mut event_rx) = mpsc::unbounded_channel();

        handle_bridge_command(
            ClusterBridgeCommand::LoadPodSecurity {
                cluster_id: "missing-ctx".to_string(),
                generation: 20,
            },
            &manager,
            &event_tx,
        )
        .await;
        match event_rx.recv().await {
            Some(ClusterBridgeEvent::PodSecurityLoadFailed {
                cluster_id,
                generation,
                message,
            }) => {
                assert_eq!(cluster_id, "missing-ctx");
                assert_eq!(generation, 20);
                assert!(!message.trim().is_empty());
            }
            other => panic!("Expected PodSecurityLoadFailed, got {other:?}"),
        }

        handle_bridge_command(
            ClusterBridgeCommand::LoadResourceQuotas {
                cluster_id: "missing-ctx".to_string(),
                generation: 21,
            },
            &manager,
            &event_tx,
        )
        .await;
        match event_rx.recv().await {
            Some(ClusterBridgeEvent::ResourceQuotasLoadFailed {
                cluster_id,
                generation,
                message,
            }) => {
                assert_eq!(cluster_id, "missing-ctx");
                assert_eq!(generation, 21);
                assert!(!message.trim().is_empty());
            }
            other => panic!("Expected ResourceQuotasLoadFailed, got {other:?}"),
        }

        handle_bridge_command(
            ClusterBridgeCommand::LoadStorageOverview {
                cluster_id: "missing-ctx".to_string(),
                generation: 22,
            },
            &manager,
            &event_tx,
        )
        .await;
        match event_rx.recv().await {
            Some(ClusterBridgeEvent::StorageOverviewLoadFailed {
                cluster_id,
                generation,
                message,
            }) => {
                assert_eq!(cluster_id, "missing-ctx");
                assert_eq!(generation, 22);
                assert!(!message.trim().is_empty());
            }
            other => panic!("Expected StorageOverviewLoadFailed, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn test_load_hpas_storage_classes_security_webhooks_without_client_emit_failed() {
        let manager = Arc::new(RwLock::new(KubeManager::new_empty(OrbitConfig::default())));
        let (event_tx, mut event_rx) = mpsc::unbounded_channel();
        let cases = [
            (
                ClusterBridgeCommand::LoadHpas {
                    cluster_id: "missing-ctx".to_string(),
                    generation: 30,
                },
                "HpasLoadFailed",
            ),
            (
                ClusterBridgeCommand::LoadStorageClasses {
                    cluster_id: "missing-ctx".to_string(),
                    generation: 30,
                },
                "StorageClassesLoadFailed",
            ),
            (
                ClusterBridgeCommand::LoadSecurityOverview {
                    cluster_id: "missing-ctx".to_string(),
                    generation: 30,
                },
                "SecurityOverviewLoadFailed",
            ),
            (
                ClusterBridgeCommand::LoadAdmissionWebhooks {
                    cluster_id: "missing-ctx".to_string(),
                    generation: 30,
                },
                "AdmissionWebhooksLoadFailed",
            ),
        ];
        for (command, expected) in cases {
            handle_bridge_command(command, &manager, &event_tx).await;
            let event = event_rx.recv().await;
            let rendered = format!("{event:?}");
            assert!(
                rendered.contains(expected),
                "expected {expected}, got {rendered}"
            );
            assert!(rendered.contains("missing-ctx"));
        }
    }

    #[tokio::test]
    async fn test_handle_bridge_command_add_cluster_emits_cluster_added_and_failed_events() {
        let _guard = orbit_core::config::TestConfigDirGuard::new("gpui_bridge_add_cluster");
        let manager = Arc::new(RwLock::new(KubeManager::new_empty(OrbitConfig::load())));
        let (event_tx, mut event_rx) = mpsc::unbounded_channel();

        let valid_yaml = r#"
apiVersion: v1
kind: Config
clusters:
  - name: bridge-cl
    cluster:
      server: https://10.0.0.50:6443
contexts:
  - name: bridge-ctx
    context:
      cluster: bridge-cl
      user: bridge-u
users:
  - name: bridge-u
    user:
      token: tok
"#;

        let req = orbit_core::kubernetes::kubeconfig_import::AddClusterRequest {
            source: orbit_core::kubernetes::kubeconfig_import::AddClusterSource::YamlContent(
                valid_yaml.to_string(),
            ),
            selected_contexts: vec!["bridge-ctx".to_string()],
            customization: None,
            switch_context: false,
        };

        handle_bridge_command(
            ClusterBridgeCommand::AddCluster(req.clone()),
            &manager,
            &event_tx,
        )
        .await;

        let ev1 = event_rx.recv().await.expect("ClusterAdded event expected");
        match ev1 {
            ClusterBridgeEvent::ClusterAdded {
                added_contexts,
                clusters,
                switch_outcome,
                ..
            } => {
                assert_eq!(added_contexts, vec!["bridge-ctx".to_string()]);
                assert_eq!(clusters.len(), 1);
                assert!(switch_outcome.is_none());
            }
            other => panic!("expected ClusterAdded, got {other:?}"),
        }

        // Duplicate context should emit AddClusterFailed
        handle_bridge_command(ClusterBridgeCommand::AddCluster(req), &manager, &event_tx).await;
        let ev2 = event_rx
            .recv()
            .await
            .expect("AddClusterFailed event expected");
        match ev2 {
            ClusterBridgeEvent::AddClusterFailed { message } => {
                assert!(message.contains("already exists"), "got: {message}");
            }
            other => panic!("expected AddClusterFailed, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn test_handle_bridge_command_disconnect_clears_active_context_and_emits_event() {
        let manager = Arc::new(RwLock::new(KubeManager::new_from_kubeconfig(
            Some(sample_kubeconfig()),
            OrbitConfig::default(),
        )));
        let (event_tx, mut event_rx) = mpsc::unbounded_channel();
        let mut tracker = BridgeTaskTracker::default();

        handle_bridge_command_with_tracker(
            ClusterBridgeCommand::SwitchCluster {
                cluster_id: "dev-ctx".to_string(),
            },
            &manager,
            &event_tx,
            &mut tracker,
        )
        .await;
        let _ = event_rx.recv().await.expect("switch event expected");

        handle_bridge_command_with_tracker(
            ClusterBridgeCommand::DisconnectCluster,
            &manager,
            &event_tx,
            &mut tracker,
        )
        .await;

        let ev = event_rx
            .recv()
            .await
            .expect("ClusterDisconnected event expected");
        match ev {
            ClusterBridgeEvent::ClusterDisconnected {
                previous_cluster_id,
                clusters,
            } => {
                assert_eq!(previous_cluster_id.as_deref(), Some("dev-ctx"));
                assert!(clusters.iter().all(|c| c.status == "offline"));
            }
            other => panic!("expected ClusterDisconnected, got {other:?}"),
        }

        let guard = manager.read().await;
        assert!(guard.active_context.is_none());
        assert!(guard.active_client.is_none());
        assert!(!guard.active_context_healthy);
    }

    #[tokio::test]
    async fn test_bridge_task_tracker_aborts_in_flight_requests_on_disconnect() {
        let manager = Arc::new(RwLock::new(KubeManager::new_from_kubeconfig(
            Some(sample_kubeconfig()),
            OrbitConfig::default(),
        )));
        let (event_tx, mut event_rx) = mpsc::unbounded_channel();
        let mut tracker = BridgeTaskTracker::default();

        let dropped = Arc::new(std::sync::atomic::AtomicBool::new(false));
        let dropped_clone = Arc::clone(&dropped);
        struct AbortGuard(Arc<std::sync::atomic::AtomicBool>);
        impl Drop for AbortGuard {
            fn drop(&mut self) {
                self.0.store(true, std::sync::atomic::Ordering::SeqCst);
            }
        }

        tracker.spawn(async move {
            let _guard = AbortGuard(dropped_clone);
            std::future::pending::<()>().await;
        });
        tokio::task::yield_now().await;
        assert!(!dropped.load(std::sync::atomic::Ordering::SeqCst));

        handle_bridge_command_with_tracker(
            ClusterBridgeCommand::DisconnectCluster,
            &manager,
            &event_tx,
            &mut tracker,
        )
        .await;
        tokio::task::yield_now().await;

        assert!(
            dropped.load(std::sync::atomic::Ordering::SeqCst),
            "in-flight tracked task should be aborted on DisconnectCluster"
        );
        let ev = event_rx.recv().await.expect("ClusterDisconnected expected");
        assert!(matches!(ev, ClusterBridgeEvent::ClusterDisconnected { .. }));
    }

    #[tokio::test]
    async fn test_handle_bridge_command_disconnect_then_switch_reconnects_cleanly() {
        let manager = Arc::new(RwLock::new(KubeManager::new_from_kubeconfig(
            Some(sample_kubeconfig()),
            OrbitConfig::default(),
        )));
        let (event_tx, mut event_rx) = mpsc::unbounded_channel();
        let mut tracker = BridgeTaskTracker::default();

        handle_bridge_command_with_tracker(
            ClusterBridgeCommand::SwitchCluster {
                cluster_id: "dev-ctx".to_string(),
            },
            &manager,
            &event_tx,
            &mut tracker,
        )
        .await;
        let _ = event_rx.recv().await.expect("first switch event");

        handle_bridge_command_with_tracker(
            ClusterBridgeCommand::DisconnectCluster,
            &manager,
            &event_tx,
            &mut tracker,
        )
        .await;
        let _ = event_rx.recv().await.expect("disconnect event");

        handle_bridge_command_with_tracker(
            ClusterBridgeCommand::SwitchCluster {
                cluster_id: "dev-ctx".to_string(),
            },
            &manager,
            &event_tx,
            &mut tracker,
        )
        .await;

        let ev = event_rx.recv().await.expect("second switch event");
        match ev {
            ClusterBridgeEvent::ClusterUnreachable { outcome, .. }
            | ClusterBridgeEvent::ClusterConnected { outcome, .. } => {
                assert_eq!(outcome.context_name, "dev-ctx");
            }
            other => panic!("expected ClusterConnected or ClusterUnreachable, got {other:?}"),
        }
        let guard = manager.read().await;
        assert_eq!(guard.active_context.as_deref(), Some("dev-ctx"));
    }

    #[tokio::test]
    async fn test_scale_workload_without_active_client_emits_action_failed() {
        let manager = Arc::new(RwLock::new(KubeManager::new_from_kubeconfig(
            Some(sample_kubeconfig()),
            OrbitConfig::default(),
        )));
        let (event_tx, mut event_rx) = mpsc::unbounded_channel();
        let mut tracker = BridgeTaskTracker::default();

        handle_bridge_command_with_tracker(
            ClusterBridgeCommand::ScaleWorkload {
                cluster_id: "dev-ctx".to_string(),
                namespace: "default".to_string(),
                kind: "Deployment".to_string(),
                name: "web".to_string(),
                replicas: 3,
            },
            &manager,
            &event_tx,
            &mut tracker,
        )
        .await;

        let ev = event_rx
            .recv()
            .await
            .expect("WorkloadActionFailed expected");
        match ev {
            ClusterBridgeEvent::WorkloadActionFailed {
                cluster_id,
                action,
                kind,
                namespace,
                name,
                message,
            } => {
                assert_eq!(cluster_id, "dev-ctx");
                assert_eq!(action, WorkloadActionKind::Scale);
                assert_eq!(kind, "Deployment");
                assert_eq!(namespace, "default");
                assert_eq!(name, "web");
                assert!(message.contains("No active Kubernetes client"));
            }
            other => panic!("expected WorkloadActionFailed, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn test_restart_workload_without_active_client_emits_action_failed() {
        let manager = Arc::new(RwLock::new(KubeManager::new_from_kubeconfig(
            Some(sample_kubeconfig()),
            OrbitConfig::default(),
        )));
        let (event_tx, mut event_rx) = mpsc::unbounded_channel();
        let mut tracker = BridgeTaskTracker::default();

        handle_bridge_command_with_tracker(
            ClusterBridgeCommand::RestartWorkload {
                cluster_id: "dev-ctx".to_string(),
                namespace: "prod".to_string(),
                kind: "StatefulSet".to_string(),
                name: "db".to_string(),
            },
            &manager,
            &event_tx,
            &mut tracker,
        )
        .await;

        let ev = event_rx
            .recv()
            .await
            .expect("WorkloadActionFailed expected");
        match ev {
            ClusterBridgeEvent::WorkloadActionFailed {
                cluster_id,
                action,
                kind,
                namespace,
                name,
                message,
            } => {
                assert_eq!(cluster_id, "dev-ctx");
                assert_eq!(action, WorkloadActionKind::Restart);
                assert_eq!(kind, "StatefulSet");
                assert_eq!(namespace, "prod");
                assert_eq!(name, "db");
                assert!(message.contains("No active Kubernetes client"));
            }
            other => panic!("expected WorkloadActionFailed, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn test_restart_pod_without_active_client_emits_action_failed() {
        let manager = Arc::new(RwLock::new(KubeManager::new_from_kubeconfig(
            Some(sample_kubeconfig()),
            OrbitConfig::default(),
        )));
        let (event_tx, mut event_rx) = mpsc::unbounded_channel();
        let mut tracker = BridgeTaskTracker::default();

        handle_bridge_command_with_tracker(
            ClusterBridgeCommand::RestartPod {
                cluster_id: "dev-ctx".to_string(),
                namespace: "default".to_string(),
                name: "api-0".to_string(),
            },
            &manager,
            &event_tx,
            &mut tracker,
        )
        .await;

        let ev = event_rx
            .recv()
            .await
            .expect("WorkloadActionFailed expected");
        match ev {
            ClusterBridgeEvent::WorkloadActionFailed {
                cluster_id,
                action,
                kind,
                namespace,
                name,
                message,
            } => {
                assert_eq!(cluster_id, "dev-ctx");
                assert_eq!(action, WorkloadActionKind::RestartPod);
                assert_eq!(kind, "Pod");
                assert_eq!(namespace, "default");
                assert_eq!(name, "api-0");
                assert!(message.contains("No active Kubernetes client"));
            }
            other => panic!("expected WorkloadActionFailed, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn test_trigger_cronjob_without_active_client_emits_action_failed() {
        let manager = Arc::new(RwLock::new(KubeManager::new_from_kubeconfig(
            Some(sample_kubeconfig()),
            OrbitConfig::default(),
        )));
        let (event_tx, mut event_rx) = mpsc::unbounded_channel();
        let mut tracker = BridgeTaskTracker::default();

        handle_bridge_command_with_tracker(
            ClusterBridgeCommand::TriggerCronJob {
                cluster_id: "dev-ctx".to_string(),
                namespace: "batch".to_string(),
                name: "nightly".to_string(),
            },
            &manager,
            &event_tx,
            &mut tracker,
        )
        .await;

        let ev = event_rx
            .recv()
            .await
            .expect("WorkloadActionFailed expected");
        match ev {
            ClusterBridgeEvent::WorkloadActionFailed {
                cluster_id,
                action,
                kind,
                namespace,
                name,
                message,
            } => {
                assert_eq!(cluster_id, "dev-ctx");
                assert_eq!(action, WorkloadActionKind::TriggerCronJob);
                assert_eq!(kind, "CronJob");
                assert_eq!(namespace, "batch");
                assert_eq!(name, "nightly");
                assert!(message.contains("No active Kubernetes client"));
            }
            other => panic!("expected WorkloadActionFailed, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn test_delete_resource_without_active_client_emits_action_failed() {
        let manager = Arc::new(RwLock::new(KubeManager::new_from_kubeconfig(
            Some(sample_kubeconfig()),
            OrbitConfig::default(),
        )));
        let (event_tx, mut event_rx) = mpsc::unbounded_channel();
        let mut tracker = BridgeTaskTracker::default();

        handle_bridge_command_with_tracker(
            ClusterBridgeCommand::DeleteResource {
                cluster_id: "dev-ctx".to_string(),
                namespace: Some("default".to_string()),
                kind: "Pod".to_string(),
                name: "web-0".to_string(),
            },
            &manager,
            &event_tx,
            &mut tracker,
        )
        .await;

        let ev = event_rx
            .recv()
            .await
            .expect("WorkloadActionFailed expected");
        match ev {
            ClusterBridgeEvent::WorkloadActionFailed {
                cluster_id,
                action,
                kind,
                namespace,
                name,
                message,
            } => {
                assert_eq!(cluster_id, "dev-ctx");
                assert_eq!(action, WorkloadActionKind::Delete);
                assert_eq!(kind, "Pod");
                assert_eq!(namespace, "default");
                assert_eq!(name, "web-0");
                assert!(message.contains("No active Kubernetes client"));
            }
            other => panic!("expected WorkloadActionFailed, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn test_delete_cluster_scoped_resource_without_active_client_emits_empty_namespace() {
        let manager = Arc::new(RwLock::new(KubeManager::new_from_kubeconfig(
            Some(sample_kubeconfig()),
            OrbitConfig::default(),
        )));
        let (event_tx, mut event_rx) = mpsc::unbounded_channel();
        let mut tracker = BridgeTaskTracker::default();

        handle_bridge_command_with_tracker(
            ClusterBridgeCommand::DeleteResource {
                cluster_id: "dev-ctx".to_string(),
                namespace: None,
                kind: "Node".to_string(),
                name: "worker-1".to_string(),
            },
            &manager,
            &event_tx,
            &mut tracker,
        )
        .await;

        let ev = event_rx
            .recv()
            .await
            .expect("WorkloadActionFailed expected");
        match ev {
            ClusterBridgeEvent::WorkloadActionFailed {
                cluster_id,
                action,
                kind,
                namespace,
                name,
                message,
            } => {
                assert_eq!(cluster_id, "dev-ctx");
                assert_eq!(action, WorkloadActionKind::Delete);
                assert_eq!(kind, "Node");
                assert_eq!(namespace, "");
                assert_eq!(name, "worker-1");
                assert!(message.contains("No active Kubernetes client"));
            }
            other => panic!("expected WorkloadActionFailed, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn test_log_stream_registry_stop_cancels_only_target_stream() {
        let mut registry = LogStreamRegistry::default();
        let (tx1, rx1) = tokio::sync::oneshot::channel::<()>();
        let (tx2, mut rx2) = tokio::sync::oneshot::channel::<()>();
        let h1 = tokio::spawn(std::future::pending::<()>()).abort_handle();
        let h2 = tokio::spawn(std::future::pending::<()>()).abort_handle();

        registry.register(1, vec![tx1], vec![h1]);
        registry.register(2, vec![tx2], vec![h2]);
        assert_eq!(registry.active_count(), 2);

        registry.stop(1);
        assert_eq!(registry.active_count(), 1);
        assert!(rx1.await.is_ok());
        assert!(matches!(
            rx2.try_recv(),
            Err(tokio::sync::oneshot::error::TryRecvError::Empty)
        ));
    }

    #[tokio::test]
    async fn test_log_stream_registry_stop_all_cancels_all_streams() {
        let mut registry = LogStreamRegistry::default();
        let (tx1, rx1) = tokio::sync::oneshot::channel::<()>();
        let (tx2, rx2) = tokio::sync::oneshot::channel::<()>();
        let h1 = tokio::spawn(std::future::pending::<()>()).abort_handle();
        let h2 = tokio::spawn(std::future::pending::<()>()).abort_handle();

        registry.register(10, vec![tx1], vec![h1]);
        registry.register(20, vec![tx2], vec![h2]);
        registry.stop_all();

        assert_eq!(registry.active_count(), 0);
        assert!(rx1.await.is_ok());
        assert!(rx2.await.is_ok());
    }

    #[tokio::test]
    async fn test_start_log_stream_without_active_client_emits_error_and_ended() {
        let manager = Arc::new(RwLock::new(KubeManager::new_from_kubeconfig(
            Some(sample_kubeconfig()),
            OrbitConfig::default(),
        )));
        let (event_tx, mut event_rx) = mpsc::unbounded_channel();
        let mut tracker = BridgeTaskTracker::default();

        handle_bridge_command_with_tracker(
            ClusterBridgeCommand::StartLogStream {
                cluster_id: "dev-ctx".to_string(),
                stream_id: 7,
                namespace: "default".to_string(),
                workload_kind: "Deployment".to_string(),
                workload_name: "api".to_string(),
                pod_filter: None,
                container_filter: None,
                tail_lines: 50,
                include_unavailable_pods: false,
            },
            &manager,
            &event_tx,
            &mut tracker,
        )
        .await;

        let ev1 = tokio::time::timeout(std::time::Duration::from_secs(2), event_rx.recv())
            .await
            .expect("timed out waiting for LogStreamStatus");
        assert!(matches!(
            ev1,
            Some(ClusterBridgeEvent::LogStreamStatus {
                stream_id: 7,
                is_error: true,
                ..
            })
        ));
        let ev2 = tokio::time::timeout(std::time::Duration::from_secs(2), event_rx.recv())
            .await
            .expect("timed out waiting for LogStreamEnded");
        assert!(matches!(
            ev2,
            Some(ClusterBridgeEvent::LogStreamEnded { stream_id: 7, .. })
        ));
    }

    #[tokio::test]
    async fn test_switch_and_disconnect_cluster_stops_active_log_streams() {
        let manager = Arc::new(RwLock::new(KubeManager::new_from_kubeconfig(
            Some(sample_kubeconfig()),
            OrbitConfig::default(),
        )));
        let (event_tx, _event_rx) = mpsc::unbounded_channel();
        let mut tracker = BridgeTaskTracker::default();

        let (tx1, rx1) = tokio::sync::oneshot::channel::<()>();
        let h1 = tokio::spawn(std::future::pending::<()>()).abort_handle();
        tracker.log_streams.register(99, vec![tx1], vec![h1]);
        assert_eq!(tracker.log_streams.active_count(), 1);

        handle_bridge_command_with_tracker(
            ClusterBridgeCommand::DisconnectCluster,
            &manager,
            &event_tx,
            &mut tracker,
        )
        .await;

        assert_eq!(tracker.log_streams.active_count(), 0);
        assert!(rx1.await.is_ok());
    }

    #[tokio::test]
    async fn test_bridge_load_port_forwards_emits_persisted_entries_for_active_cluster() {
        let _guard = orbit_core::config::TestConfigDirGuard::new("gpui_bridge_load_pf");
        let pf_a = orbit_core::kubernetes::port_forward::PersistedPortForward {
            id: "default/Service/api:8080:80".to_string(),
            cluster_id: "ctx-a".to_string(),
            namespace: "default".to_string(),
            kind: "Service".to_string(),
            name: "api".to_string(),
            local_port: 8080,
            remote_port: 80,
        };
        let pf_b = orbit_core::kubernetes::port_forward::PersistedPortForward {
            id: "default/Pod/web:3000:3000".to_string(),
            cluster_id: "ctx-b".to_string(),
            namespace: "default".to_string(),
            kind: "Pod".to_string(),
            name: "web".to_string(),
            local_port: 3000,
            remote_port: 3000,
        };
        orbit_core::kubernetes::port_forward::add_persisted_port_forward(pf_a.clone());
        orbit_core::kubernetes::port_forward::add_persisted_port_forward(pf_b);

        let manager = Arc::new(RwLock::new(KubeManager::new_empty(OrbitConfig::default())));
        let (event_tx, mut event_rx) = mpsc::unbounded_channel();
        let mut tracker = BridgeTaskTracker::default();

        handle_bridge_command_with_tracker(
            ClusterBridgeCommand::LoadPortForwards {
                cluster_id: "ctx-a".to_string(),
                generation: 3,
            },
            &manager,
            &event_tx,
            &mut tracker,
        )
        .await;

        let ev = tokio::time::timeout(std::time::Duration::from_secs(2), event_rx.recv())
            .await
            .expect("timed out waiting for PortForwardsLoaded");
        match ev {
            Some(ClusterBridgeEvent::PortForwardsLoaded {
                cluster_id,
                generation,
                port_forwards,
            }) => {
                assert_eq!(cluster_id, "ctx-a");
                assert_eq!(generation, 3);
                assert_eq!(port_forwards, vec![pf_a]);
            }
            other => panic!("unexpected event: {other:?}"),
        }
    }

    #[tokio::test]
    async fn test_bridge_stop_all_port_forwards_clears_persisted_and_emits_cleared() {
        let _guard = orbit_core::config::TestConfigDirGuard::new("gpui_bridge_stop_all_pf");
        let pf = orbit_core::kubernetes::port_forward::PersistedPortForward {
            id: "default/Service/api:8080:80".to_string(),
            cluster_id: "ctx-a".to_string(),
            namespace: "default".to_string(),
            kind: "Service".to_string(),
            name: "api".to_string(),
            local_port: 8080,
            remote_port: 80,
        };
        orbit_core::kubernetes::port_forward::add_persisted_port_forward(pf);

        let manager = Arc::new(RwLock::new(KubeManager::new_empty(OrbitConfig::default())));
        let (event_tx, mut event_rx) = mpsc::unbounded_channel();
        let mut tracker = BridgeTaskTracker::default();

        handle_bridge_command_with_tracker(
            ClusterBridgeCommand::StopAllPortForwards {
                cluster_id: "ctx-a".to_string(),
            },
            &manager,
            &event_tx,
            &mut tracker,
        )
        .await;

        let ev = tokio::time::timeout(std::time::Duration::from_secs(2), event_rx.recv())
            .await
            .expect("timed out waiting for PortForwardsCleared");
        assert!(matches!(
            ev,
            Some(ClusterBridgeEvent::PortForwardsCleared { ref cluster_id }) if cluster_id == "ctx-a"
        ));
        assert!(orbit_core::kubernetes::port_forward::get_persisted_for_cluster("ctx-a").is_empty());
    }

    #[tokio::test]
    async fn test_bridge_disconnect_stops_active_port_forward_tasks() {
        let manager = Arc::new(RwLock::new(KubeManager::new_from_kubeconfig(
            Some(sample_kubeconfig()),
            OrbitConfig::default(),
        )));
        let (tx, mut rx) = tokio::sync::oneshot::channel::<()>();
        let handle = tokio::spawn(async move {
            let _ = (&mut rx).await;
        });
        {
            let mut guard = manager.write().await;
            guard
                .port_forward_cancel
                .insert("default/Pod/demo:8080:80".to_string(), (tx, handle));
        }

        let (event_tx, _event_rx) = mpsc::unbounded_channel();
        let mut tracker = BridgeTaskTracker::default();
        handle_bridge_command_with_tracker(
            ClusterBridgeCommand::DisconnectCluster,
            &manager,
            &event_tx,
            &mut tracker,
        )
        .await;

        let guard = manager.read().await;
        assert!(guard.port_forward_cancel.is_empty());
    }
}
