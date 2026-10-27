use std::collections::HashMap;
use std::sync::Arc;
use serde_json::Value;
use tokio::sync::{Mutex, RwLock};
use crate::config::{Configuration, OrbitConfig};
use crate::ipc::bridge::{Bridge, WsWriter};
use crate::ipc::events::OrbitEvent;
use crate::kubernetes::manager::KubeManager;

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
    manager: Arc<RwLock<KubeManager>>,
) {
    tokio::spawn(async move {
        tracing::debug!("Updating application settings");
        if let Some(data) = data {
            let mut current_config = OrbitConfig::load();
            let prev_autostart = current_config.launch_on_startup();

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
                    if prev_autostart != autostart_enabled {
                        let _ = tokio::task::spawn_blocking(move || {
                            match crate::autostart::service().and_then(|s| s.set_enabled(autostart_enabled)) {
                                Ok(outcome) => {
                                    tracing::info!(outcome = ?outcome, enabled = autostart_enabled, "Autostart updated via settings");
                                }
                                Err(e) => {
                                    tracing::warn!(code = e.code(), error = %e, "Failed to apply autostart setting to operating system");
                                }
                            }
                        }).await;
                    }

                    let mut w_manager = manager.write().await;
                    w_manager.config = current_config.clone();
                    drop(w_manager);

                    let settings = current_config.get_configurations();
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
    manager: Arc<RwLock<KubeManager>>,
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
            match crate::autostart::service().and_then(|s| s.set_enabled(enabled)) {
                Ok(outcome) => {
                    tracing::info!(outcome = ?outcome, enabled, "Autostart set via set_launch_on_startup");
                }
                Err(e) => {
                    tracing::warn!(code = e.code(), error = %e, "Failed to apply autostart setting to operating system");
                }
            }
        }).await;

        let mut w_manager = manager.write().await;
        w_manager.config = current_config.clone();
        drop(w_manager);

        let settings = current_config.get_configurations();
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
    use std::path::PathBuf;
    use crate::autostart::AutostartBackend;
    use crate::ipc::emitter::{set_global_emitter, MockEventEmitter};

    #[tokio::test]
    async fn test_set_launch_on_startup_emits_app_settings_updated() {
        let _guard = crate::ipc::emitter::TEST_EMITTER_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let _config_guard = crate::config::TestConfigDirGuard::new("set_launch_on_startup");
        let in_memory_backend = Arc::new(crate::autostart::InMemoryBackend::new());
        let test_service = Arc::new(crate::autostart::AutostartService::new(
            in_memory_backend.clone(),
            crate::autostart::RunMode::Prod,
            "OrbitTest".to_string(),
            PathBuf::from("C:\\fake\\orbit.exe"),
        ));
        let _autostart_guard = crate::autostart::AutostartTestGuard::install(test_service);

        let mock_emitter = Arc::new(MockEventEmitter::new());
        set_global_emitter(mock_emitter.clone());

        let writer = Arc::new(Mutex::new(WsWriter::Emitter));
        let manager = Arc::new(RwLock::new(KubeManager::new().await));
        let payload = Some(serde_json::json!({ "enabled": true }));

        set_launch_on_startup(payload, writer, "token".to_string(), manager);

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
        assert!(
            in_memory_backend.is_registered("OrbitTest").unwrap(),
            "in_memory_backend must be registered"
        );
    }

    #[tokio::test]
    async fn test_set_launch_on_startup_in_dev_mode_does_not_register() {
        let _guard = crate::ipc::emitter::TEST_EMITTER_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let _config_guard = crate::config::TestConfigDirGuard::new("set_launch_dev_mode");
        let in_memory_backend = Arc::new(crate::autostart::InMemoryBackend::new());
        let test_service = Arc::new(crate::autostart::AutostartService::new(
            in_memory_backend.clone(),
            crate::autostart::RunMode::Dev,
            "Orbit Dev".to_string(),
            PathBuf::from("C:\\fake\\orbit.exe"),
        ));
        let _autostart_guard = crate::autostart::AutostartTestGuard::install(test_service);

        let mock_emitter = Arc::new(MockEventEmitter::new());
        set_global_emitter(mock_emitter.clone());

        let writer = Arc::new(Mutex::new(WsWriter::Emitter));
        let manager = Arc::new(RwLock::new(KubeManager::new().await));
        let payload = Some(serde_json::json!({ "enabled": true }));

        set_launch_on_startup(payload, writer, "token".to_string(), manager);

        for _ in 0..20 {
            tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
        }

        assert!(
            in_memory_backend.list().unwrap().is_empty(),
            "Dev mode service must not register autostart"
        );
    }

    #[tokio::test]
    async fn test_update_app_settings_toggling_launch_on_startup_calls_backend() {
        let _guard = crate::ipc::emitter::TEST_EMITTER_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let _config_guard = crate::config::TestConfigDirGuard::new("toggle_launch_on_startup");
        let in_memory_backend = Arc::new(crate::autostart::InMemoryBackend::new());
        let test_service = Arc::new(crate::autostart::AutostartService::new(
            in_memory_backend.clone(),
            crate::autostart::RunMode::Prod,
            "OrbitTest".to_string(),
            PathBuf::from("C:\\fake\\orbit.exe"),
        ));
        let _autostart_guard = crate::autostart::AutostartTestGuard::install(test_service);

        let mock_emitter = Arc::new(MockEventEmitter::new());
        set_global_emitter(mock_emitter.clone());

        let writer = Arc::new(Mutex::new(WsWriter::Emitter));
        let manager = Arc::new(RwLock::new(KubeManager::new().await));
        let payload = Some(serde_json::json!({ "launchOnStartup": true }));

        update_app_settings(payload, writer, "token".to_string(), manager);

        let mut registered = false;
        for _ in 0..40 {
            tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
            if in_memory_backend.is_registered("OrbitTest").unwrap() {
                registered = true;
                break;
            }
        }
        assert!(registered, "in_memory_backend must be registered via update_app_settings");
    }

    #[tokio::test]
    #[cfg(target_os = "windows")]
    async fn test_handlers_do_not_touch_production_run_key() {
        use crate::autostart::AutostartBackend;
        let _guard = crate::ipc::emitter::TEST_EMITTER_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let _config_guard = crate::config::TestConfigDirGuard::new("prod_run_key_untouched");
        let _autostart_lock = crate::autostart::AUTOSTART_TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());

        let prod_backend = crate::autostart::WindowsRunKeyBackend::production();
        let prod_before = prod_backend.list().unwrap();

        let mock_emitter = Arc::new(MockEventEmitter::new());
        set_global_emitter(mock_emitter.clone());

        let writer = Arc::new(Mutex::new(WsWriter::Emitter));
        let manager = Arc::new(RwLock::new(KubeManager::new().await));
        let payload = Some(serde_json::json!({ "enabled": true }));

        set_launch_on_startup(payload, writer, "token".to_string(), manager);

        for _ in 0..20 {
            tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
        }

        let prod_after = prod_backend.list().unwrap();
        assert_eq!(prod_before, prod_after, "Production Run key must not be modified by handlers in tests");
    }

    #[tokio::test]
    async fn test_update_app_settings_emits_app_settings_updated() {
        let _guard = crate::ipc::emitter::TEST_EMITTER_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let _config_guard = crate::config::TestConfigDirGuard::new("update_app_settings");
        let mock_emitter = Arc::new(MockEventEmitter::new());
        set_global_emitter(mock_emitter.clone());

        let writer = Arc::new(Mutex::new(WsWriter::Emitter));
        let manager = Arc::new(RwLock::new(KubeManager::new().await));
        let payload = Some(serde_json::json!({ "themeMode": "light" }));

        update_app_settings(payload, writer, "token".to_string(), manager);

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
    }

    #[tokio::test]
    async fn test_update_app_settings_syncs_manager_config() {
        let _guard = crate::ipc::emitter::TEST_EMITTER_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let _config_guard = crate::config::TestConfigDirGuard::new("sync_manager_config");
        let mock_emitter = Arc::new(MockEventEmitter::new());
        set_global_emitter(mock_emitter.clone());

        let writer = Arc::new(Mutex::new(WsWriter::Emitter));
        let manager = Arc::new(RwLock::new(KubeManager::new().await));
        let payload = Some(serde_json::json!({ "themeMode": "light" }));

        update_app_settings(payload, writer, "token".to_string(), manager.clone());

        let mut found = false;
        for _ in 0..40 {
            tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
            if manager.read().await.config.theme_mode() == "light" {
                found = true;
                break;
            }
        }
        assert!(found, "manager.config must be updated in memory");
    }
}

