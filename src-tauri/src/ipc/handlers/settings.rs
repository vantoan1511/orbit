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
