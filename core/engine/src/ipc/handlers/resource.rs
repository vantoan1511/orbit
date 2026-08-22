use std::sync::Arc;
use serde_json::Value;
use tokio::sync::{Mutex, RwLock};
use crate::ipc::bridge::{Bridge, WsWriter};
use crate::ipc::events::OrbitEvent;
use crate::kubernetes::manager::KubeManager;
use super::utils::get_string;

pub fn get_resource_raw(
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
            match crate::kubernetes::get_resource_raw(client, &namespace, &kind, &name).await {
                Ok(raw_data) => {
                    let _ = Bridge::send_event(
                        &writer,
                        &token,
                        &OrbitEvent::ResourceRawData {
                            kind,
                            name,
                            data: raw_data,
                        },
                    ).await;
                }
                Err(e) => {
                    log::error!("Error getting raw resource {}: {:?}", name, e);
                    let _ = Bridge::send_event(
                        &writer,
                        &token,
                        &OrbitEvent::ErrorOccurred {
                            message: format!("Failed to get raw resource: {}", crate::kubernetes::format_error(&e)),
                        },
                    ).await;
                }
            }
        }
    });
}

pub fn apply_resource(
    data: Option<Value>,
    writer: Arc<Mutex<WsWriter>>,
    token: String,
    manager: Arc<RwLock<KubeManager>>,
) {
    tokio::spawn(async move {
        let namespace = get_string(&data, "namespace").unwrap_or_else(|| "default".to_string());
        let kind = get_string(&data, "kind").unwrap_or_default();
        let name = get_string(&data, "name").unwrap_or_default();
        let raw_json = data.as_ref()
            .and_then(|d| d.get("data"))
            .cloned()
            .unwrap_or(serde_json::Value::Null);

        let client = {
            let r_manager = manager.read().await;
            r_manager.active_client.clone()
        };

        if let Some(ref client) = client {
            match crate::kubernetes::apply_resource(client, &namespace, &kind, &name, raw_json).await {
                Ok(()) => {
                    let _ = Bridge::send_event(
                        &writer,
                        &token,
                        &OrbitEvent::CommandSucceeded {
                            message: format!("Applied {} {}", kind, name),
                        },
                    ).await;
                }
                Err(e) => {
                    log::error!("Error applying resource {}: {:?}", name, e);
                    let _ = Bridge::send_event(
                        &writer,
                        &token,
                        &OrbitEvent::ErrorOccurred {
                            message: format!("Failed to apply resource: {}", crate::kubernetes::format_error(&e)),
                        },
                    ).await;
                }
            }
        }
    });
}

pub fn create_resource(
    data: Option<Value>,
    writer: Arc<Mutex<WsWriter>>,
    token: String,
    manager: Arc<RwLock<KubeManager>>,
) {
    tokio::spawn(async move {
        let namespace = get_string(&data, "namespace").unwrap_or_else(|| "default".to_string());
        let kind = get_string(&data, "kind").unwrap_or_default();
        let name = get_string(&data, "name").unwrap_or_default();
        let raw_json = data.as_ref()
            .and_then(|d| d.get("data"))
            .cloned()
            .unwrap_or(serde_json::Value::Null);

        if kind.is_empty() || name.is_empty() {
            let _ = Bridge::send_event(
                &writer,
                &token,
                &OrbitEvent::ErrorOccurred {
                    message: "Failed to create: kind and name must not be empty".to_string(),
                },
            ).await;
            return;
        }

        let client = {
            let r_manager = manager.read().await;
            r_manager.active_client.clone()
        };

        if let Some(ref client) = client {
            match crate::kubernetes::create_resource(client, &namespace, &kind, raw_json).await {
                Ok(()) => {
                    let _ = Bridge::send_event(
                        &writer,
                        &token,
                        &OrbitEvent::CommandSucceeded {
                            message: format!("Created {} {}", kind, name),
                        },
                    ).await;
                }
                Err(e) => {
                    log::error!("Error creating resource {}: {:?}", name, e);
                    let _ = Bridge::send_event(
                        &writer,
                        &token,
                        &OrbitEvent::ErrorOccurred {
                            message: format!("Failed to create resource: {}", crate::kubernetes::format_error(&e)),
                        },
                    ).await;
                }
            }
        } else {
            let _ = Bridge::send_event(
                &writer,
                &token,
                &OrbitEvent::ErrorOccurred {
                    message: "Failed to create: no active Kubernetes client".to_string(),
                },
            ).await;
        }
    });
}

pub fn delete_resource(
    data: Option<Value>,
    writer: Arc<Mutex<WsWriter>>,
    token: String,
    manager: Arc<RwLock<KubeManager>>,
) {
    tokio::spawn(async move {
        let namespace = get_string(&data, "namespace").unwrap_or_else(|| "default".to_string());
        let kind = get_string(&data, "kind").unwrap_or_default();
        let name = get_string(&data, "name").unwrap_or_default();

        if kind.is_empty() || name.is_empty() {
            let _ = Bridge::send_event(
                &writer,
                &token,
                &OrbitEvent::ErrorOccurred {
                    message: "Failed to delete: kind and name must not be empty".to_string(),
                },
            ).await;
            return;
        }

        let client = {
            let r_manager = manager.read().await;
            r_manager.active_client.clone()
        };

        if let Some(client) = client {
            match crate::kubernetes::delete_resource(&client, &namespace, &kind, &name).await {
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
                            message: format!("Failed to delete: {}", crate::kubernetes::format_error(&e)),
                        },
                    ).await;
                }
            }
        } else {
            let _ = Bridge::send_event(
                &writer,
                &token,
                &OrbitEvent::ErrorOccurred {
                    message: "Failed to delete: no active Kubernetes client".to_string(),
                },
            ).await;
        }
    });
}
