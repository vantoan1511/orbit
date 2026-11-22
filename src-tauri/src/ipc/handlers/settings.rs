use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use serde_json::Value;
use tauri::Manager;
use tokio::sync::{Mutex, RwLock};
use crate::config::{Configuration, OrbitConfig};
use crate::ipc::bridge::{Bridge, WsWriter};
use crate::get_global_app_handle;
use crate::ipc::events::OrbitEvent;
use crate::kubernetes::manager::KubeManager;

static GPUI_RUNNING: AtomicBool = AtomicBool::new(false);

/// Returns the bundled Tauri externalBin sidecar filename (`orbit-gpui.exe` on Windows, `orbit-gpui` on Unix).
pub fn gpui_sidecar_binary_name() -> &'static str {
    if cfg!(windows) {
        "orbit-gpui.exe"
    } else {
        "orbit-gpui"
    }
}

/// Returns the direct `cargo build -p orbit-gpui` output filename (`orbit.exe` on Windows, `orbit` on Unix).
pub fn gpui_dev_binary_name() -> &'static str {
    if cfg!(windows) {
        "orbit.exe"
    } else {
        "orbit"
    }
}

fn paths_match_executable(a: &Path, b: &Path) -> bool {
    if cfg!(windows) {
        let norm_a = a.to_string_lossy().replace('/', "\\").to_lowercase();
        let norm_b = b.to_string_lossy().replace('/', "\\").to_lowercase();
        norm_a == norm_b
    } else {
        a == b
    }
}

fn push_candidate_if_distinct(candidates: &mut Vec<PathBuf>, candidate: PathBuf, current_exe: &Path) {
    if paths_match_executable(&candidate, current_exe) {
        return;
    }
    if candidates.iter().any(|existing| paths_match_executable(existing, &candidate)) {
        return;
    }
    candidates.push(candidate);
}

/// Returns ordered candidate paths for locating the GPUI binary:
/// 1. `<current_exe_dir>/orbit-gpui[.exe]` (installed NSIS/MSI sidecar & `tauri-build` target copy)
/// 2. `<current_exe_dir>/orbit[.exe]` (direct dev build fallback next to `orbit-app[.exe]`)
/// 3. `<workspace_root>/target/{debug,release}/orbit-gpui[.exe]` and `orbit[.exe]` (dev workspace fallback)
///
/// Never returns a candidate whose path matches `current_exe` (case-insensitive on Windows) to prevent self-spawn loops.
pub fn resolve_gpui_binary_candidates(current_exe: &Path) -> Vec<PathBuf> {
    let sidecar_bin = gpui_sidecar_binary_name();
    let dev_bin = gpui_dev_binary_name();
    let mut candidates = Vec::new();

    if let Some(dir) = current_exe.parent() {
        push_candidate_if_distinct(&mut candidates, dir.join(sidecar_bin), current_exe);
        push_candidate_if_distinct(&mut candidates, dir.join(dev_bin), current_exe);
    }

    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    if let Some(workspace_root) = manifest_dir.parent() {
        let target_dir = workspace_root.join("target");
        for profile in ["debug", "release"] {
            let profile_dir = target_dir.join(profile);
            push_candidate_if_distinct(&mut candidates, profile_dir.join(sidecar_bin), current_exe);
            push_candidate_if_distinct(&mut candidates, profile_dir.join(dev_bin), current_exe);
        }
    }

    candidates
}

/// Selects the first existing, non-empty file from candidate paths.
pub fn find_gpui_binary_from_candidates(candidates: &[PathBuf]) -> Option<PathBuf> {
    candidates
        .iter()
        .find(|path| {
            std::fs::metadata(path)
                .map(|meta| meta.is_file() && meta.len() > 0)
                .unwrap_or(false)
        })
        .cloned()
}

fn find_gpui_binary() -> Option<PathBuf> {
    let current_exe = std::env::current_exe().ok()?;
    let candidates = resolve_gpui_binary_candidates(&current_exe);
    find_gpui_binary_from_candidates(&candidates)
}

pub fn launch_gpui_shell(writer: Arc<Mutex<WsWriter>>, token: String) {
    if !OrbitConfig::load().enable_experience_features() {
        tracing::warn!("Ignoring launchGpuiShell because enableExperienceFeatures is disabled");
        tokio::spawn(async move {
            let _ = Bridge::send_event(
                &writer,
                &token,
                &OrbitEvent::ErrorOccurred {
                    message: "Experience Features must be enabled in Settings to launch the GPUI shell.".to_string(),
                },
            )
            .await;
        });
        return;
    }

    if GPUI_RUNNING
        .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
        .is_err()
    {
        tracing::debug!("GPUI shell is already running; ignoring duplicate launch request");
        return;
    }

    tokio::spawn(async move {
        let Some(binary_path) = find_gpui_binary() else {
            GPUI_RUNNING.store(false, Ordering::SeqCst);
            tracing::error!("Could not locate orbit GPUI binary");
            let _ = Bridge::send_event(
                &writer,
                &token,
                &OrbitEvent::ErrorOccurred {
                    message: "Could not locate orbit GPUI binary. Build it with `cargo build -p orbit-gpui`.".to_string(),
                },
            )
            .await;
            return;
        };

        let mut cmd = tokio::process::Command::new(&binary_path);
        match cmd.spawn() {
            Ok(mut child) => {
                if let Some(app_handle) = get_global_app_handle()
                    && let Some(window) = app_handle.get_webview_window("main")
                    && let Err(err) = window.hide()
                {
                    tracing::warn!(error = %err, "Failed to hide main window while GPUI shell runs");
                }

                if let Err(err) = child.wait().await {
                    tracing::error!(error = %err, "Failed while waiting for orbit GPUI process exit");
                }
                GPUI_RUNNING.store(false, Ordering::SeqCst);

                if let Some(app_handle) = get_global_app_handle()
                    && let Some(window) = app_handle.get_webview_window("main")
                {
                    if let Err(err) = window.show() {
                        tracing::warn!(error = %err, "Failed to show main window after GPUI shell exit");
                    }
                    if let Err(err) = window.set_focus() {
                        tracing::warn!(error = %err, "Failed to focus main window after GPUI shell exit");
                    }
                }
            }
            Err(err) => {
                GPUI_RUNNING.store(false, Ordering::SeqCst);
                tracing::error!(error = %err, path = %binary_path.display(), "Failed to spawn orbit GPUI binary");
                let _ = Bridge::send_event(
                    &writer,
                    &token,
                    &OrbitEvent::ErrorOccurred {
                        message: format!("Failed to launch GPUI shell: {err}"),
                    },
                )
                .await;
            }
        }
    });
}

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
                        let autostart_res = tokio::task::spawn_blocking(move || {
                            crate::autostart::service().and_then(|s| s.set_enabled(autostart_enabled))
                        }).await;

                        match autostart_res {
                            Ok(Ok(outcome)) => {
                                tracing::info!(outcome = ?outcome, enabled = autostart_enabled, "Autostart updated via settings");
                            }
                            Ok(Err(e)) => {
                                tracing::warn!(code = e.code(), error = %e, "Failed to apply autostart setting to operating system");
                                let _ = Bridge::send_event(
                                    &writer,
                                    &token,
                                    &OrbitEvent::ErrorOccurred {
                                        message: format!("Failed to apply autostart setting: {}", e),
                                    },
                                ).await;
                            }
                            Err(e) => {
                                tracing::error!(error = %e, "Autostart background task panicked");
                            }
                        }
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

        let autostart_res = tokio::task::spawn_blocking(move || {
            crate::autostart::service().and_then(|s| s.set_enabled(enabled))
        }).await;

        match autostart_res {
            Ok(Ok(outcome)) => {
                tracing::info!(outcome = ?outcome, enabled, "Autostart set via set_launch_on_startup");
            }
            Ok(Err(e)) => {
                tracing::warn!(code = e.code(), error = %e, "Failed to apply autostart setting to operating system");
                let _ = Bridge::send_event(
                    &writer,
                    &token,
                    &OrbitEvent::ErrorOccurred {
                        message: format!("Failed to apply autostart setting: {}", e),
                    },
                ).await;
            }
            Err(e) => {
                tracing::error!(error = %e, "Autostart background task panicked");
            }
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
        let _guard = crate::ipc::emitter::TEST_EMITTER_LOCK.lock().await;
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
    async fn test_set_launch_on_startup_failure_emits_error_occurred() {
        let _guard = crate::ipc::emitter::TEST_EMITTER_LOCK.lock().await;
        let _config_guard = crate::config::TestConfigDirGuard::new("set_launch_failure");
        let failing_service = Arc::new(crate::autostart::AutostartService::new(
            Arc::new(crate::autostart::memory::InMemoryBackend::new()),
            crate::autostart::RunMode::Prod,
            "OrbitTest".to_string(),
            PathBuf::from("D:\\Projects\\orbit\\target\\debug\\deps\\orbit_app-unsafe.exe"),
        ));
        let _autostart_guard = crate::autostart::AutostartTestGuard::install(failing_service);

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
            if emitted.iter().any(|(name, _)| name == "errorOccurred") {
                found = true;
                break;
            }
        }
        assert!(found, "errorOccurred event must be emitted when autostart fails");
    }

    #[tokio::test]
    async fn test_set_launch_on_startup_in_dev_mode_does_not_register() {
        let _guard = crate::ipc::emitter::TEST_EMITTER_LOCK.lock().await;
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
        let _guard = crate::ipc::emitter::TEST_EMITTER_LOCK.lock().await;
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
    // AUTOSTART_TEST_LOCK is shared with synchronous unit tests and AutostartTestGuard,
    // so it must remain a std::sync::Mutex held for the duration of this serialized test.
    #[allow(clippy::await_holding_lock)]
    async fn test_handlers_do_not_touch_production_run_key() {
        use crate::autostart::AutostartBackend;
        let _guard = crate::ipc::emitter::TEST_EMITTER_LOCK.lock().await;
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
        let _guard = crate::ipc::emitter::TEST_EMITTER_LOCK.lock().await;
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
        let _guard = crate::ipc::emitter::TEST_EMITTER_LOCK.lock().await;
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

    #[test]
    fn test_gpui_binary_name_and_candidate_paths() {
        assert_eq!(
            gpui_sidecar_binary_name(),
            if cfg!(windows) { "orbit-gpui.exe" } else { "orbit-gpui" }
        );
        assert_eq!(
            gpui_dev_binary_name(),
            if cfg!(windows) { "orbit.exe" } else { "orbit" }
        );
        let exe_path = PathBuf::from("D:/Projects/orbit/target/debug/orbit-app.exe");
        let candidates = resolve_gpui_binary_candidates(&exe_path);
        assert!(candidates.len() >= 2);
        assert_eq!(
            candidates[0],
            PathBuf::from("D:/Projects/orbit/target/debug").join(gpui_sidecar_binary_name())
        );
        assert_eq!(
            candidates[1],
            PathBuf::from("D:/Projects/orbit/target/debug").join(gpui_dev_binary_name())
        );
        assert!(candidates.iter().any(|p| p.ends_with(Path::new("target").join("debug").join(gpui_sidecar_binary_name()))));
        assert!(candidates.iter().any(|p| p.ends_with(Path::new("target").join("debug").join(gpui_dev_binary_name()))));
        assert!(candidates.iter().any(|p| p.ends_with(Path::new("target").join("release").join(gpui_sidecar_binary_name()))));
        assert!(candidates.iter().any(|p| p.ends_with(Path::new("target").join("release").join(gpui_dev_binary_name()))));
    }

    #[test]
    fn test_resolve_gpui_binary_candidates_never_returns_current_exe() {
        let exe_path = if cfg!(windows) {
            PathBuf::from("C:/Users/dev/AppData/Local/Orbit/Orbit.exe")
        } else {
            PathBuf::from("/opt/orbit/orbit")
        };
        let exe_dir = exe_path.parent().unwrap();
        let candidates = resolve_gpui_binary_candidates(&exe_path);

        assert_eq!(
            candidates[0],
            exe_dir.join(gpui_sidecar_binary_name()),
            "Bundled sidecar must be candidate 0"
        );
        assert!(
            !candidates.iter().any(|p| p == &exe_dir.join(gpui_dev_binary_name())),
            "Candidate matching current_exe must be excluded to prevent self-spawn loops"
        );
    }

    #[test]
    fn test_find_gpui_binary_from_candidates_prefers_bundled_sidecar_and_skips_empty_files() {
        let unique = format!(
            "orbit_gpui_candidates_test_{}_{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        );
        let temp_dir = std::env::temp_dir().join(unique);
        std::fs::create_dir_all(&temp_dir).unwrap();

        let sidecar = temp_dir.join(gpui_sidecar_binary_name());
        let dev_bin = temp_dir.join(gpui_dev_binary_name());

        // Case 1: empty 0-byte sidecar is skipped in favor of non-empty dev binary
        std::fs::write(&sidecar, b"").unwrap();
        std::fs::write(&dev_bin, b"dev-binary-bytes").unwrap();
        let selected = find_gpui_binary_from_candidates(&[sidecar.clone(), dev_bin.clone()]);
        assert_eq!(selected, Some(dev_bin.clone()));

        // Case 2: non-empty bundled sidecar wins over dev binary
        std::fs::write(&sidecar, b"sidecar-binary-bytes").unwrap();
        let selected = find_gpui_binary_from_candidates(&[sidecar.clone(), dev_bin.clone()]);
        assert_eq!(selected, Some(sidecar.clone()));

        // Case 3: all candidates empty -> None
        std::fs::write(&sidecar, b"").unwrap();
        std::fs::write(&dev_bin, b"").unwrap();
        let selected = find_gpui_binary_from_candidates(&[sidecar, dev_bin]);
        assert_eq!(selected, None);

        let _ = std::fs::remove_dir_all(&temp_dir);
    }
}

