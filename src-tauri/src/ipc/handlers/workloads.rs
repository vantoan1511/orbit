use std::sync::Arc;
use serde_json::Value;
use tokio::sync::{Mutex, RwLock};
use crate::ipc::bridge::{Bridge, WsWriter};
use crate::ipc::events::OrbitEvent;
use crate::kubernetes;
use crate::kubernetes::manager::KubeManager;
use super::utils::{get_bool, get_i64, get_string};

pub fn get_pods(
    data: Option<Value>,
    writer: Arc<Mutex<WsWriter>>,
    token: String,
    manager: Arc<RwLock<KubeManager>>,
) {
    tokio::spawn(async move {
        let namespace = get_string(&data, "namespace");
        tracing::debug!(namespace = ?namespace, "Fetching pods");

        let client = {
            let r_manager = manager.read().await;
            r_manager.active_client.clone()
        };
        if let Some(ref client) = client {
            match kubernetes::list_pods(client, namespace.clone()).await {
                Ok(pods) => {
                    let _ = Bridge::send_event(&writer, &token, &OrbitEvent::PodsUpdated { pods }).await;
                }
                Err(e) => {
                    tracing::error!(namespace = ?namespace, error = %e, "Failed to list pods");
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
        tracing::debug!(namespace = ?namespace, "Fetching deployments");

        let client = {
            let r_manager = manager.read().await;
            r_manager.active_client.clone()
        };
        if let Some(ref client) = client {
            match kubernetes::list_deployments(client, namespace.clone()).await {
                Ok(deployments) => {
                    let _ = Bridge::send_event(&writer, &token, &OrbitEvent::DeploymentsUpdated { deployments }).await;
                }
                Err(e) => {
                    tracing::error!(namespace = ?namespace, error = %e, "Failed to list deployments");
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
        tracing::debug!(namespace = ?namespace, "Fetching statefulsets");

        let client = {
            let r_manager = manager.read().await;
            r_manager.active_client.clone()
        };
        if let Some(ref client) = client {
            match kubernetes::list_statefulsets(client, namespace.clone()).await {
                Ok(stateful_sets) => {
                    let _ = Bridge::send_event(&writer, &token, &OrbitEvent::StatefulSetsUpdated { stateful_sets }).await;
                }
                Err(e) => {
                    tracing::error!(namespace = ?namespace, error = %e, "Failed to list statefulsets");
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
        tracing::debug!(namespace = ?namespace, "Fetching daemonsets");

        let client = {
            let r_manager = manager.read().await;
            r_manager.active_client.clone()
        };
        if let Some(ref client) = client {
            match kubernetes::list_daemonsets(client, namespace.clone()).await {
                Ok(daemon_sets) => {
                    let _ = Bridge::send_event(&writer, &token, &OrbitEvent::DaemonSetsUpdated { daemon_sets }).await;
                }
                Err(e) => {
                    tracing::error!(namespace = ?namespace, error = %e, "Failed to list daemonsets");
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
        tracing::debug!(namespace = ?namespace, "Fetching replicasets");

        let client = {
            let r_manager = manager.read().await;
            r_manager.active_client.clone()
        };
        if let Some(ref client) = client {
            match kubernetes::list_replicasets(client, namespace.clone()).await {
                Ok(replica_sets) => {
                    let _ = Bridge::send_event(&writer, &token, &OrbitEvent::ReplicaSetsUpdated { replica_sets }).await;
                }
                Err(e) => {
                    tracing::error!(namespace = ?namespace, error = %e, "Failed to list replicasets");
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
        tracing::debug!(namespace = ?namespace, "Fetching jobs");

        let client = {
            let r_manager = manager.read().await;
            r_manager.active_client.clone()
        };
        if let Some(ref client) = client {
            match kubernetes::list_jobs(client, namespace.clone()).await {
                Ok(jobs) => {
                    let _ = Bridge::send_event(&writer, &token, &OrbitEvent::JobsUpdated { jobs }).await;
                }
                Err(e) => {
                    tracing::error!(namespace = ?namespace, error = %e, "Failed to list jobs");
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
        tracing::debug!(namespace = ?namespace, "Fetching cronjobs");

        let client = {
            let r_manager = manager.read().await;
            r_manager.active_client.clone()
        };
        if let Some(ref client) = client {
            match kubernetes::list_cronjobs(client, namespace.clone()).await {
                Ok(cron_jobs) => {
                    let _ = Bridge::send_event(&writer, &token, &OrbitEvent::CronJobsUpdated { cron_jobs }).await;
                }
                Err(e) => {
                    tracing::error!(namespace = ?namespace, error = %e, "Failed to list cronjobs");
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

pub fn get_hpas(
    data: Option<Value>,
    writer: Arc<Mutex<WsWriter>>,
    token: String,
    manager: Arc<RwLock<KubeManager>>,
) {
    tokio::spawn(async move {
        let namespace = get_string(&data, "namespace");
        tracing::debug!(namespace = ?namespace, "Fetching hpas");

        let client = {
            let r_manager = manager.read().await;
            r_manager.active_client.clone()
        };
        if let Some(ref client) = client {
            match kubernetes::list_hpas(client, namespace.clone()).await {
                Ok(hpas) => {
                    let _ = Bridge::send_event(&writer, &token, &OrbitEvent::HpasUpdated { hpas }).await;
                }
                Err(e) => {
                    tracing::error!(namespace = ?namespace, error = %e, "Failed to list hpas");
                    let _ = Bridge::send_event(
                        &writer,
                        &token,
                        &OrbitEvent::ErrorOccurred {
                            message: format!("Failed to list hpas: {}", crate::kubernetes::format_error(&e)),
                        },
                    ).await;
                }
            }
        }
    });
}

pub fn update_hpa_replicas(
    data: Option<Value>,
    writer: Arc<Mutex<WsWriter>>,
    token: String,
    manager: Arc<RwLock<KubeManager>>,
) {
    tokio::spawn(async move {
        let namespace = get_string(&data, "namespace").unwrap_or_else(|| "default".to_string());
        let name = get_string(&data, "name").unwrap_or_default();
        let min_replicas = get_i64(&data, "minReplicas").unwrap_or(1) as i32;
        let max_replicas = get_i64(&data, "maxReplicas").unwrap_or(1) as i32;

        if name.is_empty() {
            let _ = Bridge::send_event(
                &writer,
                &token,
                &OrbitEvent::ErrorOccurred {
                    message: "HPA name is required".to_string(),
                },
            )
            .await;
            return;
        }

        if min_replicas < 1 || max_replicas < min_replicas {
            let _ = Bridge::send_event(
                &writer,
                &token,
                &OrbitEvent::ErrorOccurred {
                    message: format!(
                        "Invalid replica bounds: minReplicas ({}) must be >= 1 and <= maxReplicas ({})",
                        min_replicas, max_replicas
                    ),
                },
            )
            .await;
            return;
        }

        tracing::info!(
            event = "updateHpaReplicas",
            namespace = %namespace,
            name = %name,
            min_replicas = min_replicas,
            max_replicas = max_replicas,
            "Updating HPA replica bounds"
        );

        let client = {
            let r_manager = manager.read().await;
            r_manager.active_client.clone()
        };

        if let Some(ref client) = client {
            match crate::kubernetes::update_hpa_replicas(
                client,
                &namespace,
                &name,
                min_replicas,
                max_replicas,
            )
            .await
            {
                Ok(()) => {
                    let _ = Bridge::send_event(
                        &writer,
                        &token,
                        &OrbitEvent::CommandSucceeded {
                            message: format!(
                                "Updated HPA {} replica bounds to min: {}, max: {}",
                                name, min_replicas, max_replicas
                            ),
                        },
                    )
                    .await;
                }
                Err(e) => {
                    tracing::error!(
                        event = "updateHpaReplicas",
                        namespace = %namespace,
                        name = %name,
                        error = %e,
                        "Failed to update HPA replica bounds"
                    );
                    let _ = Bridge::send_event(
                        &writer,
                        &token,
                        &OrbitEvent::ErrorOccurred {
                            message: format!(
                                "Failed to update HPA replica bounds: {}",
                                crate::kubernetes::format_error(&e)
                            ),
                        },
                    )
                    .await;
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

        tracing::info!(
            event = "scaleResource",
            namespace = %namespace,
            kind = %kind,
            name = %name,
            replicas = replicas,
            "Kubernetes operation started"
        );

        let client = {
            let r_manager = manager.read().await;
            r_manager.active_client.clone()
        };

        if let Some(ref client) = client {
            match crate::kubernetes::workloads::scale_resource(client, &namespace, &kind, &name, replicas).await {
                Ok(()) => {
                    tracing::info!(
                        event = "scaleResource",
                        namespace = %namespace,
                        kind = %kind,
                        name = %name,
                        "Kubernetes operation completed"
                    );
                    let _ = Bridge::send_event(
                        &writer,
                        &token,
                        &OrbitEvent::CommandSucceeded {
                            message: format!("Scaled {} {} to {} replicas", kind, name, replicas),
                        },
                    ).await;
                }
                Err(e) => {
                    tracing::error!(
                        event = "scaleResource",
                        namespace = %namespace,
                        kind = %kind,
                        name = %name,
                        error = %e,
                        "Kubernetes operation failed"
                    );
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

        tracing::info!(
            event = "updateResourceImages",
            namespace = %namespace,
            kind = %kind,
            name = %name,
            containers_count = containers.len(),
            "Kubernetes operation started"
        );

        let client = {
            let r_manager = manager.read().await;
            r_manager.active_client.clone()
        };

        if let Some(ref client) = client {
            match crate::kubernetes::workloads::update_images_resource(client, &namespace, &kind, &name, containers).await {
                Ok(()) => {
                    tracing::info!(
                        event = "updateResourceImages",
                        namespace = %namespace,
                        kind = %kind,
                        name = %name,
                        "Kubernetes operation completed"
                    );
                    let _ = Bridge::send_event(
                        &writer,
                        &token,
                        &OrbitEvent::CommandSucceeded {
                            message: format!("Updated images for {} {}", kind, name),
                        },
                    ).await;
                }
                Err(e) => {
                    tracing::error!(
                        event = "updateResourceImages",
                        namespace = %namespace,
                        kind = %kind,
                        name = %name,
                        error = %e,
                        "Kubernetes operation failed"
                    );
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

        tracing::info!(
            event = "redeployResource",
            namespace = %namespace,
            kind = %kind,
            name = %name,
            "Kubernetes operation started"
        );

        let client = {
            let r_manager = manager.read().await;
            r_manager.active_client.clone()
        };

        if let Some(ref client) = client {
            match crate::kubernetes::workloads::redeploy_resource(client, &namespace, &kind, &name).await {
                Ok(()) => {
                    tracing::info!(
                        event = "redeployResource",
                        namespace = %namespace,
                        kind = %kind,
                        name = %name,
                        "Kubernetes operation completed"
                    );
                    let _ = Bridge::send_event(
                        &writer,
                        &token,
                        &OrbitEvent::CommandSucceeded {
                            message: format!("Redeployed {} {}", kind, name),
                        },
                    ).await;
                }
                Err(e) => {
                    tracing::error!(
                        event = "redeployResource",
                        namespace = %namespace,
                        kind = %kind,
                        name = %name,
                        error = %e,
                        "Kubernetes operation failed"
                    );
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

        tracing::info!(
            event = "rollbackDeployment",
            namespace = %namespace,
            name = %name,
            revision = ?revision,
            "Kubernetes operation started"
        );

        let client = {
            let r_manager = manager.read().await;
            r_manager.active_client.clone()
        };

        if let Some(ref client) = client {
            match crate::kubernetes::workloads::rollback_deployment(client, &namespace, &name, revision).await {
                Ok(()) => {
                    tracing::info!(
                        event = "rollbackDeployment",
                        namespace = %namespace,
                        name = %name,
                        "Kubernetes operation completed"
                    );
                    let _ = Bridge::send_event(
                        &writer,
                        &token,
                        &OrbitEvent::CommandSucceeded {
                            message: format!("Rolled back Deployment {}", name),
                        },
                    ).await;
                }
                Err(e) => {
                    tracing::error!(
                        event = "rollbackDeployment",
                        namespace = %namespace,
                        name = %name,
                        error = %e,
                        "Kubernetes operation failed"
                    );
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

        tracing::info!(
            event = "restartPod",
            namespace = %namespace,
            name = %name,
            "Kubernetes operation started"
        );

        let client = {
            let r_manager = manager.read().await;
            r_manager.active_client.clone()
        };

        if let Some(ref client) = client {
            match crate::kubernetes::workloads::delete_pod(client, &namespace, &name).await {
                Ok(()) => {
                    tracing::info!(
                        event = "restartPod",
                        namespace = %namespace,
                        name = %name,
                        "Kubernetes operation completed"
                    );
                    let _ = Bridge::send_event(
                        &writer,
                        &token,
                        &OrbitEvent::CommandSucceeded {
                            message: format!("Restarted Pod {}", name),
                        },
                    ).await;
                }
                Err(e) => {
                    tracing::error!(
                        event = "restartPod",
                        namespace = %namespace,
                        name = %name,
                        error = %e,
                        "Kubernetes operation failed"
                    );
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

        tracing::info!(
            event = "cloneDeployment",
            source_namespace = %source_namespace,
            source_name = %source_name,
            new_namespace = %new_namespace,
            new_name = %new_name,
            "Kubernetes operation started"
        );

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
                    tracing::info!(
                        event = "cloneDeployment",
                        new_namespace = %new_namespace,
                        new_name = %new_name,
                        "Kubernetes operation completed"
                    );
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
                    tracing::error!(
                        event = "cloneDeployment",
                        source_namespace = %source_namespace,
                        source_name = %source_name,
                        error = %e,
                        "Kubernetes operation failed"
                    );
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

pub fn trigger_cronjob(
    data: Option<Value>,
    writer: Arc<Mutex<WsWriter>>,
    token: String,
    manager: Arc<RwLock<KubeManager>>,
) {
    tokio::spawn(async move {
        let namespace = get_string(&data, "namespace").unwrap_or_else(|| "default".to_string());
        let name = get_string(&data, "name").unwrap_or_default();

        if name.trim().is_empty() {
            let _ = Bridge::send_event(
                &writer,
                &token,
                &OrbitEvent::ErrorOccurred {
                    message: "CronJob name is required to trigger manually".to_string(),
                },
            )
            .await;
            return;
        }

        tracing::info!(
            event = "triggerCronJob",
            namespace = %namespace,
            name = %name,
            "Kubernetes operation started"
        );

        let client = {
            let r_manager = manager.read().await;
            r_manager.active_client.clone()
        };

        if let Some(ref client) = client {
            match crate::kubernetes::workloads::trigger_cronjob(client, &namespace, &name).await {
                Ok(job_name) => {
                    tracing::info!(
                        event = "triggerCronJob",
                        namespace = %namespace,
                        name = %name,
                        job_name = %job_name,
                        "Kubernetes operation completed"
                    );
                    let _ = Bridge::send_event(
                        &writer,
                        &token,
                        &OrbitEvent::CommandSucceeded {
                            message: format!("Created Job {} from CronJob {}", job_name, name),
                        },
                    )
                    .await;
                }
                Err(e) => {
                    tracing::error!(
                        event = "triggerCronJob",
                        namespace = %namespace,
                        name = %name,
                        error = %e,
                        "Kubernetes operation failed"
                    );
                    let _ = Bridge::send_event(
                        &writer,
                        &token,
                        &OrbitEvent::ErrorOccurred {
                            message: format!("Failed to trigger CronJob: {}", crate::kubernetes::format_error(&e)),
                        },
                    )
                    .await;
                }
            }
        }
    });
}

pub fn set_cronjob_suspend(
    data: Option<Value>,
    writer: Arc<Mutex<WsWriter>>,
    token: String,
    manager: Arc<RwLock<KubeManager>>,
) {
    tokio::spawn(async move {
        let namespace = get_string(&data, "namespace").unwrap_or_else(|| "default".to_string());
        let name = get_string(&data, "name").unwrap_or_default();
        let suspend = get_bool(&data, "suspend").unwrap_or(true);

        if name.trim().is_empty() {
            let _ = Bridge::send_event(
                &writer,
                &token,
                &OrbitEvent::ErrorOccurred {
                    message: "CronJob name is required to update suspend state".to_string(),
                },
            )
            .await;
            return;
        }

        let action_verb = if suspend { "suspend" } else { "resume" };
        tracing::info!(
            event = "setCronJobSuspend",
            namespace = %namespace,
            name = %name,
            suspend = %suspend,
            "Kubernetes operation started"
        );

        let client = {
            let r_manager = manager.read().await;
            r_manager.active_client.clone()
        };

        if let Some(ref client) = client {
            match crate::kubernetes::workloads::set_cronjob_suspend(client, &namespace, &name, suspend).await {
                Ok(()) => {
                    tracing::info!(
                        event = "setCronJobSuspend",
                        namespace = %namespace,
                        name = %name,
                        suspend = %suspend,
                        "Kubernetes operation completed"
                    );
                    let _ = Bridge::send_event(
                        &writer,
                        &token,
                        &OrbitEvent::CommandSucceeded {
                            message: format!("Successfully {}ed CronJob {}", action_verb, name),
                        },
                    )
                    .await;
                }
                Err(e) => {
                    tracing::error!(
                        event = "setCronJobSuspend",
                        namespace = %namespace,
                        name = %name,
                        suspend = %suspend,
                        error = %e,
                        "Kubernetes operation failed"
                    );
                    let _ = Bridge::send_event(
                        &writer,
                        &token,
                        &OrbitEvent::ErrorOccurred {
                            message: format!("Failed to {} CronJob: {}", action_verb, crate::kubernetes::format_error(&e)),
                        },
                    )
                    .await;
                }
            }
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ipc::emitter::{set_global_emitter, MockEventEmitter};

    #[tokio::test]
    async fn test_trigger_cronjob_empty_name_emits_error() {
        let _guard = crate::ipc::emitter::TEST_EMITTER_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let mock_emitter = Arc::new(MockEventEmitter::new());
        set_global_emitter(mock_emitter.clone());

        let writer = Arc::new(Mutex::new(WsWriter::Emitter));
        let manager = Arc::new(RwLock::new(KubeManager::new().await));

        trigger_cronjob(
            Some(serde_json::json!({ "name": "", "namespace": "default" })),
            writer,
            "token".to_string(),
            manager,
        );

        tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
        let emitted = mock_emitter.get_emitted_events().await;
        let err_event = emitted.iter().find(|(name, _)| name == "errorOccurred");
        assert!(err_event.is_some(), "Expected errorOccurred event when name is empty");
    }

    #[tokio::test]
    async fn test_set_cronjob_suspend_empty_name_emits_error() {
        let _guard = crate::ipc::emitter::TEST_EMITTER_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let mock_emitter = Arc::new(MockEventEmitter::new());
        set_global_emitter(mock_emitter.clone());

        let writer = Arc::new(Mutex::new(WsWriter::Emitter));
        let manager = Arc::new(RwLock::new(KubeManager::new().await));

        set_cronjob_suspend(
            Some(serde_json::json!({ "name": "", "namespace": "default", "suspend": true })),
            writer,
            "token".to_string(),
            manager,
        );

        tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
        let emitted = mock_emitter.get_emitted_events().await;
        let err_event = emitted.iter().find(|(name, _)| name == "errorOccurred");
        assert!(err_event.is_some(), "Expected errorOccurred event when name is empty");
    }

    #[tokio::test]
    async fn test_update_hpa_replicas_invalid_bounds_emits_error() {
        let _guard = crate::ipc::emitter::TEST_EMITTER_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let mock_emitter = Arc::new(MockEventEmitter::new());
        set_global_emitter(mock_emitter.clone());

        let writer = Arc::new(Mutex::new(WsWriter::Emitter));
        let manager = Arc::new(RwLock::new(KubeManager::new().await));

        // Test maxReplicas < minReplicas
        update_hpa_replicas(
            Some(serde_json::json!({ "name": "test-hpa", "namespace": "default", "minReplicas": 5, "maxReplicas": 2 })),
            writer.clone(),
            "token".to_string(),
            manager.clone(),
        );

        tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
        let emitted = mock_emitter.get_emitted_events().await;
        let err_event = emitted.iter().find(|(name, _)| name == "errorOccurred");
        assert!(err_event.is_some(), "Expected errorOccurred event when bounds are invalid");

        // Test empty name
        mock_emitter.emitted.lock().await.clear();
        update_hpa_replicas(
            Some(serde_json::json!({ "name": "", "namespace": "default", "minReplicas": 1, "maxReplicas": 5 })),
            writer,
            "token".to_string(),
            manager,
        );

        tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
        let emitted = mock_emitter.get_emitted_events().await;
        let err_event = emitted.iter().find(|(name, _)| name == "errorOccurred");
        assert!(err_event.is_some(), "Expected errorOccurred event when name is empty");
    }
}
