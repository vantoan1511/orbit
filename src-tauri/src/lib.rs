pub mod autostart;
pub mod config;
pub mod ipc;
pub mod kubernetes;
pub mod logger;
pub mod tray;
pub mod updater;

use std::sync::Arc;
use tokio::sync::RwLock;
use tauri::{AppHandle, Manager, State, Emitter};
use serde_json::Value;
use async_trait::async_trait;
use crate::kubernetes::manager::KubeManager;
use crate::ipc::emitter::{EventEmitter, set_global_emitter};

pub struct AppState {
    pub kube_manager: Arc<RwLock<KubeManager>>,
}

pub struct TauriEventEmitter {
    pub app: AppHandle,
}

#[async_trait]
impl EventEmitter for TauriEventEmitter {
    async fn emit(&self, event: &str, data: Value) -> Result<(), String> {
        self.app.emit(event, data).map_err(|e| e.to_string())
    }
}

#[tauri::command]
async fn dispatch_engine(
    state: State<'_, AppState>,
    event: String,
    data: Option<Value>,
) -> Result<(), String> {
    crate::ipc::handlers::dispatch_tauri(&event, data, state.kube_manager.clone());
    Ok(())
}

#[tauri::command]
async fn read_text_file(path: String) -> Result<String, String> {
    tokio::fs::read_to_string(&path).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn write_text_file(path: String, content: String) -> Result<(), String> {
    if let Some(parent) = std::path::Path::new(&path).parent() {
        if !parent.as_os_str().is_empty() {
            tokio::fs::create_dir_all(parent).await.map_err(|e| e.to_string())?;
        }
    }
    tokio::fs::write(&path, content).await.map_err(|e| e.to_string())
}

static GLOBAL_APP_HANDLE: std::sync::RwLock<Option<AppHandle>> = std::sync::RwLock::new(None);

pub fn set_global_app_handle(handle: AppHandle) {
    if let Ok(mut lock) = GLOBAL_APP_HANDLE.write() {
        *lock = Some(handle);
    }
}

pub fn get_global_app_handle() -> Option<AppHandle> {
    GLOBAL_APP_HANDLE.read().ok().and_then(|lock| lock.clone())
}

/// Returns whether the single-instance plugin should be registered.
/// In development mode, the single-instance guard is skipped to prevent clashes with a running production instance.
pub fn should_register_single_instance() -> bool {
    !crate::config::OrbitConfig::is_dev_mode()
}

pub fn run() {
    // Initialize file logger
    if let Err(e) = crate::logger::init() {
        eprintln!("Warning: Failed to initialize file logger: {}", e);
    }

    // In dev mode the compiled-in app identifier matches the production bundle identifier
    // (vantoan1511.orbit), so tauri-plugin-single-instance's Windows mutex / macOS socket
    // would prevent a dev instance from starting alongside a running production build.
    // We therefore only register the single-instance guard when running in production mode.
    let mut builder = tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_process::init());

    if should_register_single_instance() {
        builder = builder.plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            crate::tray::show_main_window(app);
        }));
    }

    let builder = builder
        .plugin(tauri_plugin_store::Builder::new().build())
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                let close_to_tray = crate::config::OrbitConfig::load().close_to_tray();
                match crate::tray::determine_window_close_action(close_to_tray) {
                    crate::tray::WindowCloseAction::HideToTray => {
                        api.prevent_close();
                        let _ = window.hide();
                    }
                    crate::tray::WindowCloseAction::ExitApplication => {
                        window.app_handle().exit(0);
                    }
                }
            }
        })
        .setup(|app| {
            let handle = app.handle().clone();
            set_global_app_handle(handle.clone());
            let emitter: Arc<dyn EventEmitter> = Arc::new(TauriEventEmitter { app: handle.clone() });
            set_global_emitter(emitter.clone());

            let kube_manager = Arc::new(RwLock::new(tauri::async_runtime::block_on(async {
                KubeManager::new().await
            })));
            app.manage(AppState { kube_manager: kube_manager.clone() });

            let km_clone = kube_manager.clone();
            tauri::async_runtime::spawn(async move {
                let writer = Arc::new(tokio::sync::Mutex::new(crate::ipc::bridge::WsWriter::Emitter));
                crate::ipc::handlers::cluster::broadcast_engine_ready(&writer, "", &km_clone).await;
            });

            // Set up system tray
            if let Err(e) = crate::tray::setup_tray(&handle) {
                tracing::warn!(error = %e, "Failed to initialize system tray");
            }

            // Show main window only if NOT launched with --hidden or --minimized
            if !crate::tray::is_hidden_launch(std::env::args()) {
                crate::tray::show_main_window(&handle);
            }

            // If running in development mode, ensure window title reflects dev mode even if not launched with dev config overlay
            if crate::config::OrbitConfig::is_dev_mode() {
                if let Some(window) = handle.get_webview_window("main") {
                    let _ = window.set_title("Orbit [Dev]");
                }
            }

            // Reconcile OS autostart state with saved application configuration
            tauri::async_runtime::spawn_blocking(crate::autostart::reconcile_on_startup);

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![dispatch_engine, write_text_file, read_text_file]);

    if let Err(e) = builder.run(tauri::generate_context!()) {
        tracing::error!("error while running tauri application: {}", e);
    }
}

#[cfg(test)]
mod tests {
    use super::{AppState, KubeManager, write_text_file};
    use std::sync::Arc;
    use tokio::sync::RwLock;

    #[tokio::test]
    async fn test_write_text_file_creates_file_and_dirs() {
        let temp_dir = std::env::temp_dir().join(format!("orbit_test_{}", uuid::Uuid::new_v4()));
        let file_path = temp_dir.join("subfolder").join("logs.txt");
        let sample_content = "2026-10-14T10:00:00Z [nginx-pod/nginx] Server started";

        let result = write_text_file(file_path.to_string_lossy().to_string(), sample_content.to_string()).await;
        assert!(result.is_ok());

        let read_back = std::fs::read_to_string(&file_path).unwrap();
        assert_eq!(read_back, sample_content);

        let _ = std::fs::remove_dir_all(&temp_dir);
    }

    #[tokio::test]
    async fn test_read_text_file_reads_existing_file() {
        let temp_dir = std::env::temp_dir().join(format!("orbit_test_read_{}", uuid::Uuid::new_v4()));
        let file_path = temp_dir.join("test.txt");
        let sample_content = "apiVersion: v1\nkind: Config";

        let result = write_text_file(file_path.to_string_lossy().to_string(), sample_content.to_string()).await;
        assert!(result.is_ok());

        let read_back = super::read_text_file(file_path.to_string_lossy().to_string()).await;
        assert!(read_back.is_ok());
        assert_eq!(read_back.unwrap(), sample_content);

        let _ = std::fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_app_state_creation() {
        let km = Arc::new(RwLock::new(KubeManager {
            kubeconfig: None,
            active_context: None,
            active_client: None,
            watch_cancel: None,
            active_context_healthy: false,
            log_cancel: Vec::new(),
            port_forward_cancel: std::collections::HashMap::new(),
            config: crate::config::OrbitConfig::load(),
        }));
        let state = AppState { kube_manager: km };
        assert!(state.kube_manager.blocking_read().active_context.is_none());
    }

    #[test]
    fn test_should_register_single_instance_in_dev_vs_prod() {
        let original_orbit_dev = std::env::var("ORBIT_DEV").ok();

        // 1. In dev mode: single instance plugin must be skipped
        unsafe {
            std::env::set_var("ORBIT_DEV", "1");
        }
        assert!(!super::should_register_single_instance());

        // 2. In prod mode: single instance plugin must be registered
        unsafe {
            std::env::set_var("ORBIT_DEV", "0");
        }
        assert!(super::should_register_single_instance());

        // Restore env
        unsafe {
            match original_orbit_dev {
                Some(val) => std::env::set_var("ORBIT_DEV", val),
                None => std::env::remove_var("ORBIT_DEV"),
            }
        }
    }
}


