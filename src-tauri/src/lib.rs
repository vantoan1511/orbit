pub mod config;
pub mod ipc;
pub mod kubernetes;
pub mod logger;
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

pub fn run() {
    // Initialize file logger
    if let Err(e) = crate::logger::init() {
        eprintln!("Warning: Failed to initialize file logger: {}", e);
    }

    let builder = tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_store::Builder::new().build())
        .setup(|app| {
            let handle = app.handle().clone();
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

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![dispatch_engine]);

    if let Err(e) = builder.run(tauri::generate_context!()) {
        tracing::error!("error while running tauri application: {}", e);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
