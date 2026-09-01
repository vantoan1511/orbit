use std::sync::Arc;
use serde_json::Value;
use tokio::sync::{Mutex, RwLock};
use crate::ipc::bridge::{Bridge, WsWriter};
use crate::ipc::events::OrbitEvent;
use crate::kubernetes;
use crate::kubernetes::manager::KubeManager;
use super::utils::get_string;

pub fn get_persistent_volumes(
    writer: Arc<Mutex<WsWriter>>,
    token: String,
    manager: Arc<RwLock<KubeManager>>,
) {
    tokio::spawn(async move {
        tracing::debug!("Fetching persistent volumes");
        let client = {
            let r_manager = manager.read().await;
            r_manager.active_client.clone()
        };
        if let Some(ref client) = client {
            match kubernetes::list_pvs(client).await {
                Ok(persistent_volumes) => {
                    let _ = Bridge::send_event(&writer, &token, &OrbitEvent::PersistentVolumesUpdated { persistent_volumes }).await;
                }
                Err(e) => {
                    tracing::error!(error = %e, "Failed to list persistent volumes");
                    let _ = Bridge::send_event(
                        &writer,
                        &token,
                        &OrbitEvent::ErrorOccurred {
                            message: format!("Failed to list persistent volumes: {}", crate::kubernetes::format_error(&e)),
                        },
                    ).await;
                }
            }
        }
    });
}

pub fn get_persistent_volume_claims(
    data: Option<Value>,
    writer: Arc<Mutex<WsWriter>>,
    token: String,
    manager: Arc<RwLock<KubeManager>>,
) {
    tokio::spawn(async move {
        let namespace = get_string(&data, "namespace");
        tracing::debug!(namespace = ?namespace, "Fetching persistent volume claims");

        let client = {
            let r_manager = manager.read().await;
            r_manager.active_client.clone()
        };
        if let Some(ref client) = client {
            match kubernetes::list_pvcs(client, namespace.clone()).await {
                Ok(persistent_volume_claims) => {
                    let _ = Bridge::send_event(&writer, &token, &OrbitEvent::PersistentVolumeClaimsUpdated { persistent_volume_claims }).await;
                }
                Err(e) => {
                    tracing::error!(namespace = ?namespace, error = %e, "Failed to list persistent volume claims");
                    let _ = Bridge::send_event(
                        &writer,
                        &token,
                        &OrbitEvent::ErrorOccurred {
                            message: format!("Failed to list persistent volume claims: {}", crate::kubernetes::format_error(&e)),
                        },
                    ).await;
                }
            }
        }
    });
}

pub fn get_storage_classes(
    writer: Arc<Mutex<WsWriter>>,
    token: String,
    manager: Arc<RwLock<KubeManager>>,
) {
    tokio::spawn(async move {
        tracing::debug!("Fetching storage classes");
        let client = {
            let r_manager = manager.read().await;
            r_manager.active_client.clone()
        };
        if let Some(ref client) = client {
            match kubernetes::list_storage_classes(client).await {
                Ok(storage_classes) => {
                    let _ = Bridge::send_event(&writer, &token, &OrbitEvent::StorageClassesUpdated { storage_classes }).await;
                }
                Err(e) => {
                    tracing::error!(error = %e, "Failed to list storage classes");
                    let _ = Bridge::send_event(
                        &writer,
                        &token,
                        &OrbitEvent::ErrorOccurred {
                            message: format!("Failed to list storage classes: {}", crate::kubernetes::format_error(&e)),
                        },
                    ).await;
                }
            }
        }
    });
}
