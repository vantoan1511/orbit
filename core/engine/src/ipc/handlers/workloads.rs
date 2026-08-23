use std::sync::Arc;
use serde_json::Value;
use tokio::sync::{Mutex, RwLock};
use crate::ipc::bridge::{Bridge, WsWriter};
use crate::ipc::events::OrbitEvent;
use crate::kubernetes;
use crate::kubernetes::manager::KubeManager;
use super::utils::{get_i64, get_string};

pub fn get_pods(
    data: Option<Value>,
    writer: Arc<Mutex<WsWriter>>,
    token: String,
    manager: Arc<RwLock<KubeManager>>,
) {
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
                Err(e) => {
                    log::error!("Error listing pods: {:?}", e);
                    let _ = Bridge::send_event(
                        &writer,
                        &token,
                        &OrbitEvent::ErrorOccurred {
                            message: format!("Failed to list pods: {}", crate::kubernetes::format_error(&e)),
                        },
                    ).await;
                }
            }
        }
    });
}

pub fn get_deployments(
    data: Option<Value>,
    writer: Arc<Mutex<WsWriter>>,
    token: String,
    manager: Arc<RwLock<KubeManager>>,
) {
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
                Err(e) => {
                    log::error!("Error listing deployments: {:?}", e);
                    let _ = Bridge::send_event(
                        &writer,
                        &token,
                        &OrbitEvent::ErrorOccurred {
                            message: format!("Failed to list deployments: {}", crate::kubernetes::format_error(&e)),
                        },
                    ).await;
                }
            }
        }
    });
}

pub fn get_statefulsets(
    data: Option<Value>,
    writer: Arc<Mutex<WsWriter>>,
    token: String,
    manager: Arc<RwLock<KubeManager>>,
) {
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
                Err(e) => {
                    log::error!("Error listing statefulsets: {:?}", e);
                    let _ = Bridge::send_event(
                        &writer,
                        &token,
                        &OrbitEvent::ErrorOccurred {
                            message: format!("Failed to list statefulsets: {}", crate::kubernetes::format_error(&e)),
                        },
                    ).await;
                }
            }
        }
    });
}

pub fn get_daemonsets(
    data: Option<Value>,
    writer: Arc<Mutex<WsWriter>>,
    token: String,
    manager: Arc<RwLock<KubeManager>>,
) {
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
                Err(e) => {
                    log::error!("Error listing daemonsets: {:?}", e);
                    let _ = Bridge::send_event(
                        &writer,
                        &token,
                        &OrbitEvent::ErrorOccurred {
                            message: format!("Failed to list daemonsets: {}", crate::kubernetes::format_error(&e)),
                        },
                    ).await;
                }
            }
        }
    });
}

pub fn get_replicasets(
    data: Option<Value>,
    writer: Arc<Mutex<WsWriter>>,
    token: String,
    manager: Arc<RwLock<KubeManager>>,
) {
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
                Err(e) => {
                    log::error!("Error listing replicasets: {:?}", e);
                    let _ = Bridge::send_event(
                        &writer,
                        &token,
                        &OrbitEvent::ErrorOccurred {
                            message: format!("Failed to list replicasets: {}", crate::kubernetes::format_error(&e)),
                        },
                    ).await;
                }
            }
        }
    });
}

pub fn get_jobs(
    data: Option<Value>,
    writer: Arc<Mutex<WsWriter>>,
    token: String,
    manager: Arc<RwLock<KubeManager>>,
) {
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
                Err(e) => {
                    log::error!("Error listing jobs: {:?}", e);
                    let _ = Bridge::send_event(
                        &writer,
                        &token,
                        &OrbitEvent::ErrorOccurred {
                            message: format!("Failed to list jobs: {}", crate::kubernetes::format_error(&e)),
                        },
                    ).await;
                }
            }
        }
    });
}

pub fn get_cronjobs(
    data: Option<Value>,
    writer: Arc<Mutex<WsWriter>>,
    token: String,
    manager: Arc<RwLock<KubeManager>>,
) {
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
                Err(e) => {
                    log::error!("Error listing cronjobs: {:?}", e);
                    let _ = Bridge::send_event(
                        &writer,
                        &token,
                        &OrbitEvent::ErrorOccurred {
                            message: format!("Failed to list cronjobs: {}", crate::kubernetes::format_error(&e)),
                        },
                    ).await;
                }
            }
        }
    });
}

pub fn scale_resource(
    data: Option<Value>,
    writer: Arc<Mutex<WsWriter>>,
    token: String,
    manager: Arc<RwLock<KubeManager>>,
) {
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
                            message: format!("Failed to scale: {}", crate::kubernetes::format_error(&e)),
                        },
                    ).await;
                }
            }
        }
    });
}

pub fn update_resource_images(
    data: Option<Value>,
    writer: Arc<Mutex<WsWriter>>,
    token: String,
    manager: Arc<RwLock<KubeManager>>,
) {
    tokio::spawn(async move {
        let namespace = get_string(&data, "namespace").unwrap_or_else(|| "default".to_string());
        let kind = get_string(&data, "kind").unwrap_or_default();
        let name = get_string(&data, "name").unwrap_or_default();
        let containers: Vec<crate::kubernetes::models::ContainerImageInfo> = data
            .as_ref()
            .and_then(|d| d.get("containers"))
            .and_then(|c| serde_json::from_value(c.clone()).ok())
            .unwrap_or_default();

        if containers.is_empty() {
            let _ = Bridge::send_event(
                &writer,
                &token,
                &OrbitEvent::ErrorOccurred {
                    message: "No containers provided for image update".to_string(),
                },
            ).await;
            return;
        }

        let client = {
            let r_manager = manager.read().await;
            r_manager.active_client.clone()
        };

        if let Some(ref client) = client {
            match crate::kubernetes::workloads::update_images_resource(client, &namespace, &kind, &name, containers).await {
                Ok(()) => {
                    let _ = Bridge::send_event(
                        &writer,
                        &token,
                        &OrbitEvent::CommandSucceeded {
                            message: format!("Updated images for {} {}", kind, name),
                        },
                    ).await;
                }
                Err(e) => {
                    log::error!("Error updating images for {}: {:?}", kind, e);
                    let _ = Bridge::send_event(
                        &writer,
                        &token,
                        &OrbitEvent::ErrorOccurred {
                            message: format!("Failed to update images: {}", crate::kubernetes::format_error(&e)),
                        },
                    ).await;
                }
            }
        }
    });
}

pub fn redeploy_resource(
    data: Option<Value>,
    writer: Arc<Mutex<WsWriter>>,
    token: String,
    manager: Arc<RwLock<KubeManager>>,
) {
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
                            message: format!("Failed to redeploy: {}", crate::kubernetes::format_error(&e)),
                        },
                    ).await;
                }
            }
        }
    });
}

pub fn rollback_deployment(
    data: Option<Value>,
    writer: Arc<Mutex<WsWriter>>,
    token: String,
    manager: Arc<RwLock<KubeManager>>,
) {
    tokio::spawn(async move {
        let namespace = get_string(&data, "namespace").unwrap_or_else(|| "default".to_string());
        let name = get_string(&data, "name").unwrap_or_default();
        let revision = get_i64(&data, "revision");

        let client = {
            let r_manager = manager.read().await;
            r_manager.active_client.clone()
        };

        if let Some(ref client) = client {
            match crate::kubernetes::workloads::rollback_deployment(client, &namespace, &name, revision).await {
                Ok(()) => {
                    let _ = Bridge::send_event(
                        &writer,
                        &token,
                        &OrbitEvent::CommandSucceeded {
                            message: format!("Rolled back Deployment {}", name),
                        },
                    ).await;
                }
                Err(e) => {
                    log::error!("Error rolling back deployment {}: {:?}", name, e);
                    let _ = Bridge::send_event(
                        &writer,
                        &token,
                        &OrbitEvent::ErrorOccurred {
                            message: format!("Failed to rollback: {}", crate::kubernetes::format_error(&e)),
                        },
                    ).await;
                }
            }
        }
    });
}

pub fn restart_pod(
    data: Option<Value>,
    writer: Arc<Mutex<WsWriter>>,
    token: String,
    manager: Arc<RwLock<KubeManager>>,
) {
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
                            message: format!("Failed to restart pod: {}", crate::kubernetes::format_error(&e)),
                        },
                    ).await;
                }
            }
        }
    });
}

pub fn clone_deployment(
    data: Option<Value>,
    writer: Arc<Mutex<WsWriter>>,
    token: String,
    manager: Arc<RwLock<KubeManager>>,
) {
    tokio::spawn(async move {
        let source_namespace = get_string(&data, "sourceNamespace")
            .unwrap_or_else(|| "default".to_string());
        let source_name = get_string(&data, "sourceName").unwrap_or_default();
        let new_name = get_string(&data, "newName").unwrap_or_default();
        let new_namespace = get_string(&data, "newNamespace")
            .unwrap_or_else(|| source_namespace.clone());

        let client = {
            let r_manager = manager.read().await;
            r_manager.active_client.clone()
        };

        if let Some(ref client) = client {
            match crate::kubernetes::workloads::clone_deployment(
                client,
                &source_namespace,
                &source_name,
                &new_name,
                &new_namespace,
            )
            .await
            {
                Ok(()) => {
                    let _ = Bridge::send_event(
                        &writer,
                        &token,
                        &OrbitEvent::CommandSucceeded {
                            message: format!("Cloned Deployment {} as {}", source_name, new_name),
                        },
                    )
                    .await;
                }
                Err(e) => {
                    log::error!("Error cloning Deployment {}: {:?}", source_name, e);
                    let _ = Bridge::send_event(
                        &writer,
                        &token,
                        &OrbitEvent::ErrorOccurred {
                            message: format!("Failed to clone Deployment: {}", crate::kubernetes::format_error(&e)),
                        },
                    )
                    .await;
                }
            }
        }
    });
}
