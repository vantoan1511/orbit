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
        let current_engine = env!("CARGO_PKG_VERSION");
        let explicit_manifest_url = get_string(&data, "manifestUrl");

        tracing::info!(current_version = %current_engine, manifest_url = ?explicit_manifest_url, "Checking for updates");

        let check_res = if let Some(url) = explicit_manifest_url {
            crate::updater::UpdateManifest::fetch(&url, current_engine)
                .await
                .map(|manifest| {
                    let has_update = manifest.has_update(current_engine).unwrap_or(false);
                    Some((manifest, has_update))
                })
        } else {
            // First check GitHub Releases API directly (new zip-free updater mechanism)
            match crate::updater::UpdateManifest::fetch_from_github(current_engine).await {
                Ok(Some(result)) => Ok(Some(result)),
                Ok(None) => {
                    // Fallback to legacy manifest URL if GitHub API had no newer release or was rate-limited
                    let fallback_url = "https://raw.githubusercontent.com/vantoan1511/orbit/main/update-manifest.json";
                    crate::updater::UpdateManifest::fetch(fallback_url, current_engine)
                        .await
                        .map(|manifest| {
                            let has_update = manifest.has_update(current_engine).unwrap_or(false);
                            Some((manifest, has_update))
                        })
                }
                Err(e) => {
                    tracing::warn!(error = ?e, "GitHub direct check failed; trying fallback manifest");
                    let fallback_url = "https://raw.githubusercontent.com/vantoan1511/orbit/main/update-manifest.json";
                    crate::updater::UpdateManifest::fetch(fallback_url, current_engine)
                        .await
                        .map(|manifest| {
                            let has_update = manifest.has_update(current_engine).unwrap_or(false);
                            Some((manifest, has_update))
                        })
                }
            }
        };

        match check_res {
            Ok(Some((manifest, has_update))) => {
                tracing::info!(has_update = has_update, version = %manifest.version, "Update check finished");
                let _ = Bridge::send_event(
                    &writer,
                    &token,
                    &OrbitEvent::UpdateCheckFinished {
                        has_update,
                        manifest,
                    },
                ).await;
            }
            Ok(None) => {
                tracing::info!("No update available or check skipped");
                let _ = Bridge::send_event(
                    &writer,
                    &token,
                    &OrbitEvent::UpdateCheckFinished {
                        has_update: false,
                        manifest: crate::updater::UpdateManifest {
                            version: current_engine.to_string(),
                            url: String::new(),
                            release_notes: None,
                        },
                    },
                ).await;
            }
            Err(e) => {
                tracing::error!(error = ?e, "Failed to check for updates");
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

pub fn extract_target_version(data: &Option<Value>, fallback: &str) -> String {
    get_string(data, "version").unwrap_or_else(|| fallback.to_string())
}

pub fn apply_update(
    data: Option<Value>,
    writer: Arc<Mutex<WsWriter>>,
    token: String,
) {
    tokio::spawn(async move {
        let url = get_string(&data, "url");
        let current_engine = env!("CARGO_PKG_VERSION");
        let target_version = extract_target_version(&data, current_engine);
            
        if let Some(url) = url {
            tracing::info!(download_url = %url, target_version = %target_version, "Applying update");
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

            // Download installer directly using atomic .part staging
            let download_res = crate::updater::UpdateManifest::download_installer(&url, &target_version, Some(tx)).await;

            match download_res {
                Ok(installer_path) => {
                    tracing::info!(installer_path = ?installer_path, "Installer downloaded, launching setup");

                    let _ = Bridge::send_event(
                        &writer,
                        &token,
                        &OrbitEvent::UpdateReady {
                            component: "app".to_string(),
                        },
                    ).await;

                    // On Windows, launch the installer GUI directly
                    let spawn_res = std::process::Command::new(&installer_path).spawn();

                    match spawn_res {
                        Ok(_) => {
                            tracing::info!("Installer spawned successfully; exiting Orbit for upgrade");
                            // Give websocket bridge time to send UpdateReady event before exit
                            tokio::time::sleep(std::time::Duration::from_millis(500)).await;
                            std::process::exit(0);
                        }
                        Err(e) => {
                            tracing::error!(error = ?e, "Failed to spawn installer");
                            let _ = Bridge::send_event(
                                &writer,
                                &token,
                                &OrbitEvent::ErrorOccurred {
                                    message: format!("Failed to launch installer: {}", e),
                                },
                            ).await;
                        }
                    }
                }
                Err(e) => {
                    tracing::error!(error = ?e, "Failed to download update installer");
                    let _ = Bridge::send_event(
                        &writer,
                        &token,
                        &OrbitEvent::ErrorOccurred {
                            message: format!("Failed to download update: {}", e),
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
    use serde_json::json;

    #[test]
    fn test_extract_target_version_with_explicit_version() {
        let payload = Some(json!({
            "url": "https://example.com/Orbit_0.14.9_x64-setup.exe",
            "version": "0.14.9"
        }));
        let version = extract_target_version(&payload, "0.14.8");
        assert_eq!(version, "0.14.9");
    }

    #[test]
    fn test_extract_target_version_fallback_to_current() {
        let payload = Some(json!({
            "url": "https://example.com/Orbit_0.14.9_x64-setup.exe"
        }));
        let version = extract_target_version(&payload, "0.14.8");
        assert_eq!(version, "0.14.8");
    }
}
