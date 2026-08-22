use std::sync::Arc;
use serde_json::Value;
use tokio::sync::Mutex;
use crate::ipc::bridge::{Bridge, WsWriter};
use crate::ipc::events::OrbitEvent;
use super::utils::get_string;

pub fn check_for_updates(
    data: Option<Value>,
    writer: Arc<Mutex<WsWriter>>,
    token: String,
) {
    tokio::spawn(async move {
        let url = get_string(&data, "manifestUrl")
            .unwrap_or_else(|| "https://raw.githubusercontent.com/vantoan1511/orbit/main/update-manifest.json".to_string());
        
        match crate::updater::UpdateManifest::fetch(&url).await {
            Ok(manifest) => {
                let current_engine = env!("CARGO_PKG_VERSION");
                
                let has_update = manifest.has_update(current_engine).unwrap_or(false);

                let _ = Bridge::send_event(
                    &writer,
                    &token,
                    &OrbitEvent::UpdateCheckFinished {
                        has_update,
                        manifest,
                    },
                ).await;
            }
            Err(e) => {
                log::error!("Failed to fetch update manifest: {:?}", e);
                let _ = Bridge::send_event(
                    &writer,
                    &token,
                    &OrbitEvent::ErrorOccurred {
                        message: format!("Failed to check for updates: {}", e),
                    },
                ).await;
            }
        }
    });
}

pub fn apply_update(
    data: Option<Value>,
    writer: Arc<Mutex<WsWriter>>,
    token: String,
) {
    tokio::spawn(async move {
        let url = get_string(&data, "url");
            
        if let Some(url) = url {
            let (tx, mut rx) = tokio::sync::mpsc::channel(100);
            let writer_clone = writer.clone();
            let token_clone = token.clone();
            
            tokio::spawn(async move {
                while let Some(progress) = rx.recv().await {
                    let _ = Bridge::send_event(
                        &writer_clone,
                        &token_clone,
                        &OrbitEvent::UpdateDownloadProgress {
                            component: "app".to_string(),
                            progress_percentage: progress,
                        },
                    ).await;
                }
            });

            let download_res = crate::updater::UpdateManifest::download(&url, "orbit-update.zip", Some(tx)).await;
            if let Ok(path) = download_res {
                let current_exe_res = std::env::current_exe();
                if let Ok(current_exe) = current_exe_res {
                    let bin_dir_opt = current_exe.parent();
                    if let Some(bin_dir) = bin_dir_opt {
                        let app_dir = bin_dir.parent().unwrap_or(bin_dir);
                        
                        let updater_name = if cfg!(target_os = "windows") { "orbit-apply.exe" } else { "orbit-apply" };
                        let updater_path = bin_dir.join(updater_name);
                        
                        let os = std::env::consts::OS;
                        let arch = std::env::consts::ARCH;
                        
                        let neu_os = match os {
                            "windows" => "win",
                            "macos" => "mac",
                            "linux" => "linux",
                            _ => "linux",
                        };
                        
                        let neu_arch = match arch {
                            "x86_64" => "x64",
                            "aarch64" => "arm64",
                            _ => "x64",
                        };
                        
                        let ext = if cfg!(target_os = "windows") { ".exe" } else { "" };
                        let exe_name = format!("orbit-{}_{}{}", neu_os, neu_arch, ext);
                        log::info!("Spawning updater: {:?} with zip: {:?}, target_dir: {:?}, exe_name: {}", updater_path, path, app_dir, exe_name);
                        
                        match std::process::Command::new(&updater_path)
                            .arg("--zip-path")
                            .arg(&path)
                            .arg("--target-dir")
                            .arg(app_dir)
                            .arg("--executable-name")
                            .arg(exe_name)
                            .spawn() {
                                Ok(_) => {
                                    log::info!("Updater spawned successfully.");
                                }
                                Err(e) => {
                                    log::error!("Failed to spawn updater: {:?}", e);
                                }
                            }
                    }
                }
                
                let _ = Bridge::send_event(
                    &writer,
                    &token,
                    &OrbitEvent::UpdateReady {
                        component: "app".to_string(),
                    },
                ).await;
            }
        }
    });
}
