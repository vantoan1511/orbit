use std::sync::Arc;
use serde_json::Value;
use tokio::sync::{Mutex, RwLock};
use crate::ipc::bridge::{Bridge, WsWriter};
use crate::ipc::events::OrbitEvent;
use crate::kubernetes;
use crate::kubernetes::manager::KubeManager;
use super::utils::get_string;

pub fn get_config_maps(
    data: Option<Value>,
    writer: Arc<Mutex<WsWriter>>,
    token: String,
    manager: Arc<RwLock<KubeManager>>,
) {
    tokio::spawn(async move {
        let namespace = get_string(&data, "namespace");
        tracing::debug!(namespace = ?namespace, "Fetching configmaps");

        let client = {
            let r_manager = manager.read().await;
            r_manager.active_client.clone()
        };
        if let Some(ref client) = client {
            match kubernetes::list_configmaps(client, namespace.clone()).await {
                Ok(config_maps) => {
                    let _ = Bridge::send_event(&writer, &token, &OrbitEvent::ConfigMapsUpdated { config_maps }).await;
                }
                Err(e) => {
                    tracing::error!(namespace = ?namespace, error = %e, "Failed to list configmaps");
                    let _ = Bridge::send_event(
                        &writer,
                        &token,
                        &OrbitEvent::ErrorOccurred {
                            message: format!("Failed to list configmaps: {}", crate::kubernetes::format_error(&e)),
                        },
                    ).await;
                }
            }
        }
    });
}

pub fn get_secrets(
    data: Option<Value>,
    writer: Arc<Mutex<WsWriter>>,
    token: String,
    manager: Arc<RwLock<KubeManager>>,
) {
    tokio::spawn(async move {
        let namespace = get_string(&data, "namespace");
        tracing::debug!(namespace = ?namespace, "Fetching secrets");

        let client = {
            let r_manager = manager.read().await;
            r_manager.active_client.clone()
        };
        if let Some(ref client) = client {
            match kubernetes::list_secrets(client, namespace.clone()).await {
                Ok(secrets) => {
                    let _ = Bridge::send_event(&writer, &token, &OrbitEvent::SecretsUpdated { secrets }).await;
                }
                Err(e) => {
                    tracing::error!(namespace = ?namespace, error = %e, "Failed to list secrets");
                    let _ = Bridge::send_event(
                        &writer,
                        &token,
                        &OrbitEvent::ErrorOccurred {
                            message: format!("Failed to list secrets: {}", crate::kubernetes::format_error(&e)),
                        },
                    ).await;
                }
            }
        }
    });
}
