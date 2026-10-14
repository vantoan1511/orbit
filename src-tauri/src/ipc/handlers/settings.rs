use std::collections::HashMap;
use std::sync::Arc;
use serde_json::Value;
use tokio::sync::Mutex;
use crate::config::{Configuration, OrbitConfig};
use crate::ipc::bridge::{Bridge, WsWriter};
use crate::ipc::events::OrbitEvent;

pub fn get_app_settings(
    writer: Arc<Mutex<WsWriter>>,
    token: String,
) {
    tokio::spawn(async move {
        tracing::debug!("Fetching application settings");
        let settings = OrbitConfig::load().get_configurations();
        let _ = Bridge::send_event(
            &writer,
            &token,
            &OrbitEvent::AppSettingsUpdated { settings },
        ).await;
    });
}

pub fn update_app_settings(
    data: Option<Value>,
    writer: Arc<Mutex<WsWriter>>,
    token: String,
) {
    tokio::spawn(async move {
        tracing::debug!("Updating application settings");
        if let Some(data) = data {
            let mut current_config = OrbitConfig::load();

            // Support either unified Vec<Configuration> or direct HashMap<String, Value>
            let update_result = if let Ok(configs) = serde_json::from_value::<Vec<Configuration>>(data.clone()) {
                current_config.update_from_configurations(&configs);
                Ok(())
            } else if let Ok(map) = serde_json::from_value::<HashMap<String, Value>>(data.clone()) {
                for (k, v) in map {
                    current_config.values.insert(k, v);
                }
                Ok(())
            } else {
                Err("Payload must be an array of Configuration objects or a key-value map".to_string())
            };

            match update_result {
                Ok(()) => {
                    if let Err(e) = current_config.save() {
                        tracing::error!(error = %e, "Failed to save application settings");
                        let _ = Bridge::send_event(
                            &writer,
                            &token,
                            &OrbitEvent::ErrorOccurred {
                                message: format!("Failed to save settings: {}", e),
                            },
                        ).await;
                        return;
                    }

                    let autostart_enabled = current_config.launch_on_startup();
                    let _ = tokio::task::spawn_blocking(move || {
                        if let Ok(manager) = crate::autostart::AutostartManager::new(crate::autostart::DEFAULT_APP_NAME) {
                            let _ = if autostart_enabled {
                                manager.enable()
                            } else {
                                manager.disable()
                            };
                        }
                    }).await;

                    let settings = OrbitConfig::load().get_configurations();
                    let _ = Bridge::send_event(
                        &writer,
                        &token,
                        &OrbitEvent::AppSettingsUpdated { settings },
                    ).await;
                }
                Err(err_msg) => {
                    tracing::error!(error = %err_msg, "Failed to deserialize settings update payload");
                    let _ = Bridge::send_event(
                        &writer,
                        &token,
                        &OrbitEvent::ErrorOccurred {
                            message: format!("Invalid settings format: {}", err_msg),
                        },
                    ).await;
                }
            }
        }
    });
}

pub fn set_launch_on_startup(
    data: Option<Value>,
    writer: Arc<Mutex<WsWriter>>,
    token: String,
) {
    tokio::spawn(async move {
        tracing::debug!("Setting launch on startup");
        let enabled = data
            .as_ref()
            .and_then(|d| d.get("enabled"))
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        let mut current_config = OrbitConfig::load();
        current_config.values.insert("launchOnStartup".to_string(), serde_json::json!(enabled));
        if let Err(e) = current_config.save() {
            tracing::error!(error = %e, "Failed to save application settings");
            let _ = Bridge::send_event(
                &writer,
                &token,
                &OrbitEvent::ErrorOccurred {
                    message: format!("Failed to save settings: {}", e),
                },
            ).await;
            return;
        }

        let _ = tokio::task::spawn_blocking(move || {
            match crate::autostart::AutostartManager::new(crate::autostart::DEFAULT_APP_NAME) {
                Ok(manager) => {
                    let res = if enabled {
                        manager.enable()
                    } else {
                        manager.disable()
                    };
                    if let Err(e) = res {
                        tracing::warn!(error = %e, "Failed to apply autostart setting to operating system");
                    }
                }
                Err(e) => {
                    tracing::warn!(error = %e, "Failed to initialize autostart manager");
                }
            }
        }).await;

        let settings = OrbitConfig::load().get_configurations();
        let _ = Bridge::send_event(
            &writer,
            &token,
            &OrbitEvent::AppSettingsUpdated { settings },
        ).await;
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ipc::emitter::{set_global_emitter, MockEventEmitter};

    #[tokio::test]
    async fn test_set_launch_on_startup_emits_app_settings_updated() {
        let _guard = crate::ipc::emitter::TEST_EMITTER_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let mock_emitter = Arc::new(MockEventEmitter::new());
        set_global_emitter(mock_emitter.clone());

        let writer = Arc::new(Mutex::new(WsWriter::Emitter));
        let payload = Some(serde_json::json!({ "enabled": true }));

        set_launch_on_startup(payload, writer, "token".to_string());

        let mut found = false;
        for _ in 0..40 {
            tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
            let emitted = mock_emitter.get_emitted_events().await;
            if emitted.iter().any(|(name, _)| name == "appSettingsUpdated") {
                found = true;
                break;
            }
        }
        assert!(found, "appSettingsUpdated event must be emitted");

        // Clean up test registry entry
        let _ = tokio::task::spawn_blocking(|| {
            if let Ok(manager) = crate::autostart::AutostartManager::new(crate::autostart::DEFAULT_APP_NAME) {
                let _ = manager.disable();
            }
        }).await;
    }
}

