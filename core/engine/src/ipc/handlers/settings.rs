use std::sync::Arc;
use serde_json::Value;
use tokio::sync::Mutex;
use crate::config::OrbitConfig;
use crate::ipc::bridge::{Bridge, WsWriter};
use crate::ipc::events::OrbitEvent;

pub fn get_app_settings(
    writer: Arc<Mutex<WsWriter>>,
    token: String,
) {
    tokio::spawn(async move {
        tracing::debug!("Fetching application settings");
        let settings = OrbitConfig::load();
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
            match serde_json::from_value::<OrbitConfig>(data) {
                Ok(new_settings) => {
                    if let Err(e) = new_settings.save() {
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
                    let settings = OrbitConfig::load();
                    let _ = Bridge::send_event(
                        &writer,
                        &token,
                        &OrbitEvent::AppSettingsUpdated { settings },
                    ).await;
                }
                Err(e) => {
                    tracing::error!(error = %e, "Failed to deserialize settings update payload");
                    let _ = Bridge::send_event(
                        &writer,
                        &token,
                        &OrbitEvent::ErrorOccurred {
                            message: format!("Invalid settings format: {}", e),
                        },
                    ).await;
                }
            }
        }
    });
}
