use std::sync::Arc;
use serde_json::Value;
use tokio::sync::{Mutex, RwLock};
use crate::ipc::bridge::{Bridge, WsWriter};
use crate::ipc::events::OrbitEvent;
use crate::kubernetes;
use crate::kubernetes::manager::KubeManager;
use super::utils::get_string;

pub fn get_namespaces(
    writer: Arc<Mutex<WsWriter>>,
    token: String,
    manager: Arc<RwLock<KubeManager>>,
) {
    tokio::spawn(async move {
        tracing::debug!("Fetching namespaces");
        let client = {
            let r_manager = manager.read().await;
            r_manager.active_client.clone()
        };
        if let Some(ref client) = client {
            match kubernetes::list_namespaces(client).await {
                Ok(namespaces) => {
                    let _ = Bridge::send_event(&writer, &token, &OrbitEvent::NamespacesUpdated { namespaces }).await;
                }
                Err(e) => {
                    tracing::error!(error = %e, "Failed to list namespaces");
                    let _ = Bridge::send_event(
                        &writer,
                        &token,
                        &OrbitEvent::ErrorOccurred {
                            message: format!("Failed to list namespaces: {}", crate::kubernetes::format_error(&e)),
                        },
                    ).await;
                }
            }
        }
    });
}

pub fn get_nodes(
    writer: Arc<Mutex<WsWriter>>,
    token: String,
    manager: Arc<RwLock<KubeManager>>,
) {
    tokio::spawn(async move {
        tracing::debug!("Fetching nodes");
        let client = {
            let r_manager = manager.read().await;
            r_manager.active_client.clone()
        };
        if let Some(ref client) = client {
            match kubernetes::list_nodes(client).await {
                Ok(nodes) => {
                    let _ = Bridge::send_event(&writer, &token, &OrbitEvent::NodesUpdated { nodes }).await;
                }
                Err(e) => {
                    tracing::error!(error = %e, "Failed to list nodes");
                    let _ = Bridge::send_event(
                        &writer,
                        &token,
                        &OrbitEvent::ErrorOccurred {
                            message: format!("Failed to list nodes: {}", crate::kubernetes::format_error(&e)),
                        },
                    ).await;
                }
            }
        }
    });
}

pub fn get_events(
    data: Option<Value>,
    writer: Arc<Mutex<WsWriter>>,
    token: String,
    manager: Arc<RwLock<KubeManager>>,
) {
    tokio::spawn(async move {
        let namespace = get_string(&data, "namespace");
        tracing::debug!(namespace = ?namespace, "Fetching events");

        let client = {
            let r_manager = manager.read().await;
            r_manager.active_client.clone()
        };
        if let Some(ref client) = client {
            match kubernetes::list_events(client, namespace.clone()).await {
                Ok(events) => {
                    let _ = Bridge::send_event(&writer, &token, &OrbitEvent::EventsUpdated { events }).await;
                }
                Err(e) => {
                    tracing::error!(namespace = ?namespace, error = %e, "Failed to list events");
                    let _ = Bridge::send_event(
                        &writer,
                        &token,
                        &OrbitEvent::ErrorOccurred {
                            message: format!("Failed to list events: {}", crate::kubernetes::format_error(&e)),
                        },
                    ).await;
                }
            }
        }
    });
}

pub fn get_policies(
    data: Option<Value>,
    writer: Arc<Mutex<WsWriter>>,
    token: String,
    manager: Arc<RwLock<KubeManager>>,
) {
    tokio::spawn(async move {
        let namespace = get_string(&data, "namespace");
        tracing::debug!(namespace = ?namespace, "Fetching policies");

        let client = {
            let r_manager = manager.read().await;
            r_manager.active_client.clone()
        };
        if let Some(ref client) = client {
            match kubernetes::list_policies(client, namespace.clone()).await {
                Ok(policies) => {
                    let _ = Bridge::send_event(&writer, &token, &OrbitEvent::PoliciesUpdated { policies }).await;
                }
                Err(e) => {
                    tracing::error!(namespace = ?namespace, error = %e, "Failed to list policies");
                    let _ = Bridge::send_event(
                        &writer,
                        &token,
                        &OrbitEvent::ErrorOccurred {
                            message: format!("Failed to list policies: {}", crate::kubernetes::format_error(&e)),
                        },
                    ).await;
                }
            }
        }
    });
}

pub fn cordon_node(
    data: Option<Value>,
    writer: Arc<Mutex<WsWriter>>,
    token: String,
    manager: Arc<RwLock<KubeManager>>,
) {
    tokio::spawn(async move {
        let name = match get_string(&data, "name") {
            Some(n) if !n.trim().is_empty() => n,
            _ => {
                let _ = Bridge::send_event(
                    &writer,
                    &token,
                    &OrbitEvent::ErrorOccurred {
                        message: "Node name is required".to_string(),
                    },
                ).await;
                return;
            }
        };

        tracing::info!(name = %name, "Cordoning node");
        let client = {
            let r_manager = manager.read().await;
            r_manager.active_client.clone()
        };

        if let Some(ref client) = client {
            match kubernetes::set_node_schedulable(client, &name, true).await {
                Ok(()) => {
                    let _ = Bridge::send_event(
                        &writer,
                        &token,
                        &OrbitEvent::CommandSucceeded {
                            message: format!("Node {} cordoned", name),
                        },
                    ).await;
                }
                Err(e) => {
                    tracing::error!(name = %name, error = %e, "Failed to cordon node");
                    let _ = Bridge::send_event(
                        &writer,
                        &token,
                        &OrbitEvent::ErrorOccurred {
                            message: format!("Failed to cordon node: {}", crate::kubernetes::format_error(&e)),
                        },
                    ).await;
                }
            }
        }
    });
}

pub fn uncordon_node(
    data: Option<Value>,
    writer: Arc<Mutex<WsWriter>>,
    token: String,
    manager: Arc<RwLock<KubeManager>>,
) {
    tokio::spawn(async move {
        let name = match get_string(&data, "name") {
            Some(n) if !n.trim().is_empty() => n,
            _ => {
                let _ = Bridge::send_event(
                    &writer,
                    &token,
                    &OrbitEvent::ErrorOccurred {
                        message: "Node name is required".to_string(),
                    },
                ).await;
                return;
            }
        };

        tracing::info!(name = %name, "Uncordoning node");
        let client = {
            let r_manager = manager.read().await;
            r_manager.active_client.clone()
        };

        if let Some(ref client) = client {
            match kubernetes::set_node_schedulable(client, &name, false).await {
                Ok(()) => {
                    let _ = Bridge::send_event(
                        &writer,
                        &token,
                        &OrbitEvent::CommandSucceeded {
                            message: format!("Node {} uncordoned", name),
                        },
                    ).await;
                }
                Err(e) => {
                    tracing::error!(name = %name, error = %e, "Failed to uncordon node");
                    let _ = Bridge::send_event(
                        &writer,
                        &token,
                        &OrbitEvent::ErrorOccurred {
                            message: format!("Failed to uncordon node: {}", crate::kubernetes::format_error(&e)),
                        },
                    ).await;
                }
            }
        }
    });
}

pub fn drain_node(
    data: Option<Value>,
    writer: Arc<Mutex<WsWriter>>,
    token: String,
    manager: Arc<RwLock<KubeManager>>,
) {
    tokio::spawn(async move {
        let name = match get_string(&data, "name") {
            Some(n) if !n.trim().is_empty() => n,
            _ => {
                let _ = Bridge::send_event(
                    &writer,
                    &token,
                    &OrbitEvent::ErrorOccurred {
                        message: "Node name is required".to_string(),
                    },
                ).await;
                return;
            }
        };

        tracing::info!(name = %name, "Draining node");
        let client = {
            let r_manager = manager.read().await;
            r_manager.active_client.clone()
        };

        if let Some(ref client) = client {
            match kubernetes::drain_node(client, &name).await {
                Ok(count) => {
                    let _ = Bridge::send_event(
                        &writer,
                        &token,
                        &OrbitEvent::CommandSucceeded {
                            message: format!("Node {} drained ({} pod(s) evicted)", name, count),
                        },
                    ).await;
                }
                Err(e) => {
                    tracing::error!(name = %name, error = %e, "Failed to drain node");
                    let _ = Bridge::send_event(
                        &writer,
                        &token,
                        &OrbitEvent::ErrorOccurred {
                            message: format!("Failed to drain node: {}", crate::kubernetes::format_error(&e)),
                        },
                    ).await;
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
    async fn test_cordon_node_empty_name_emits_error() {
        let _guard = crate::ipc::emitter::TEST_EMITTER_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let mock_emitter = Arc::new(MockEventEmitter::new());
        set_global_emitter(mock_emitter.clone());

        let writer = Arc::new(Mutex::new(WsWriter::Emitter));
        let manager = Arc::new(RwLock::new(KubeManager::new().await));

        cordon_node(Some(serde_json::json!({ "name": "" })), writer, "token".to_string(), manager);

        tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
        let emitted = mock_emitter.get_emitted_events().await;
        let error_event = emitted.iter().find(|(name, _)| name == "errorOccurred");
        assert!(error_event.is_some());
    }

    #[tokio::test]
    async fn test_uncordon_node_empty_name_emits_error() {
        let _guard = crate::ipc::emitter::TEST_EMITTER_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let mock_emitter = Arc::new(MockEventEmitter::new());
        set_global_emitter(mock_emitter.clone());

        let writer = Arc::new(Mutex::new(WsWriter::Emitter));
        let manager = Arc::new(RwLock::new(KubeManager::new().await));

        uncordon_node(Some(serde_json::json!({ "name": "   " })), writer, "token".to_string(), manager);

        tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
        let emitted = mock_emitter.get_emitted_events().await;
        let error_event = emitted.iter().find(|(name, _)| name == "errorOccurred");
        assert!(error_event.is_some());
    }

    #[tokio::test]
    async fn test_drain_node_empty_name_emits_error() {
        let _guard = crate::ipc::emitter::TEST_EMITTER_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let mock_emitter = Arc::new(MockEventEmitter::new());
        set_global_emitter(mock_emitter.clone());

        let writer = Arc::new(Mutex::new(WsWriter::Emitter));
        let manager = Arc::new(RwLock::new(KubeManager::new().await));

        drain_node(None, writer, "token".to_string(), manager);

        tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
        let emitted = mock_emitter.get_emitted_events().await;
        let error_event = emitted.iter().find(|(name, _)| name == "errorOccurred");
        assert!(error_event.is_some());
    }
}
