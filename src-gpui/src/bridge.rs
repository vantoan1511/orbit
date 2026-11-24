use std::sync::Arc;

use orbit_core::kubernetes::batch::{fetch_cronjobs, fetch_jobs};
use orbit_core::kubernetes::configmaps::fetch_configmaps;
use orbit_core::kubernetes::events::fetch_events;
use orbit_core::kubernetes::hpa::fetch_hpas;
use orbit_core::kubernetes::ingresses::fetch_ingresses;
use orbit_core::kubernetes::manager::KubeManager;
use orbit_core::kubernetes::namespaces::fetch_namespaces;
use orbit_core::kubernetes::nodes::fetch_nodes;
use orbit_core::kubernetes::overview::fetch_cluster_overview;
use orbit_core::kubernetes::policies::{
    fetch_admission_webhooks, fetch_network_policies, fetch_pod_security_policies, fetch_policies,
    fetch_resource_quotas_and_limits,
};
use orbit_core::kubernetes::secrets::fetch_secrets;
use orbit_core::kubernetes::services::fetch_services;
use orbit_core::kubernetes::storage::{fetch_pvcs, fetch_pvs, fetch_storage_classes};
use orbit_core::kubernetes::storage_overview::fetch_storage_overview;
use orbit_core::kubernetes::workloads::{
    fetch_daemonsets, fetch_deployments, fetch_pods, fetch_replicasets, fetch_statefulsets,
};
use orbit_core::kubernetes::workloads_overview::fetch_workloads_overview;
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
                tokio::spawn(async move {
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
                tokio::spawn(async move {
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
                tokio::spawn(async move {
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
                tokio::spawn(async move {
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
                tokio::spawn(async move {
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
                tokio::spawn(async move {
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
                tokio::spawn(async move {
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
                tokio::spawn(async move {
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
                tokio::spawn(async move {
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
                tokio::spawn(async move {
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
                tokio::spawn(async move {
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
                tokio::spawn(async move {
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
                tokio::spawn(async move {
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
                tokio::spawn(async move {
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
                tokio::spawn(async move {
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
                tokio::spawn(async move {
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
                tokio::spawn(async move {
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
                tokio::spawn(async move {
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
                tokio::spawn(async move {
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
                tokio::spawn(async move {
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
        ClusterBridgeCommand::SaveThemeMode { mode } => {
            let mut guard = manager.write().await;
            guard.config.update_theme_mode(&mode);
            if let Err(message) = guard.config.save() {
                tracing::warn!(error = %message, "Failed to persist theme mode");
                let _ = event_tx.send(ClusterBridgeEvent::SettingsSaveFailed { message });
            }
        }
        ClusterBridgeCommand::SaveUiStyle { style } => {
            let mut guard = manager.write().await;
            guard.config.update_ui_style(&style);
            if let Err(message) = guard.config.save() {
                tracing::warn!(error = %message, "Failed to persist UI style");
                let _ = event_tx.send(ClusterBridgeEvent::SettingsSaveFailed { message });
            }
        }
        ClusterBridgeCommand::SaveAccentColor { accent } => {
            let mut guard = manager.write().await;
            guard.config.update_accent_color(&accent);
            if let Err(message) = guard.config.save() {
                tracing::warn!(error = %message, "Failed to persist accent color");
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
        ClusterBridgeCommand::ResetAppearanceDefaults => {
            let mut guard = manager.write().await;
            guard.config.reset_appearance_preferences();
            if let Err(message) = guard.config.save() {
                tracing::warn!(error = %message, "Failed to reset appearance preferences");
                let _ = event_tx.send(ClusterBridgeEvent::SettingsSaveFailed { message });
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
        let manager = Arc::new(RwLock::new(KubeManager::new_empty(OrbitConfig::default())));
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
            ClusterBridgeCommand::SaveUiStyle {
                style: "glass".to_string(),
            },
            &manager,
            &event_tx,
        )
        .await;
        handle_bridge_command(
            ClusterBridgeCommand::SaveAccentColor {
                accent: "violet".to_string(),
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
        assert_eq!(loaded.ui_style(), "glass");
        assert_eq!(loaded.accent_color(), "violet");
        assert!(loaded.enable_experience_features());

        handle_bridge_command(
            ClusterBridgeCommand::ResetAppearanceDefaults,
            &manager,
            &event_tx,
        )
        .await;

        assert!(event_rx.try_recv().is_err());

        let after_reset = OrbitConfig::load();
        assert_eq!(after_reset.theme_mode(), "dark");
        assert_eq!(after_reset.ui_style(), "flat");
        assert_eq!(after_reset.accent_color(), "blue");
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
}
