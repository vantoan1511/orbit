use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;
use serde_json::Value;
use tauri::Manager;
use tokio::io::{AsyncRead, AsyncReadExt};
use tokio::sync::{Mutex, RwLock};
use crate::config::{Configuration, OrbitConfig};
use crate::ipc::bridge::{Bridge, WsWriter};
use crate::get_global_app_handle;
use crate::ipc::events::OrbitEvent;
use crate::kubernetes::manager::KubeManager;

static GPUI_LAUNCHER_STATE: GpuiLauncherState = GpuiLauncherState::new();

pub trait MainWindowController: Send + Sync {
    fn hide_main_window(&self);
    fn restore_main_window(&self);
}

pub struct TauriMainWindowController;

impl MainWindowController for TauriMainWindowController {
    fn hide_main_window(&self) {
        if let Some(app_handle) = get_global_app_handle()
            && let Some(window) = app_handle.get_webview_window("main")
            && let Err(err) = window.hide()
        {
            tracing::warn!(error = %err, "Failed to hide main window while GPUI shell runs");
        }
    }

    fn restore_main_window(&self) {
        if let Some(app_handle) = get_global_app_handle() {
            crate::tray::show_main_window(&app_handle);
        }
    }
}

pub struct GpuiLauncherState {
    running: AtomicBool,
    window_hidden: AtomicBool,
}

impl Default for GpuiLauncherState {
    fn default() -> Self {
        Self::new()
    }
}

impl GpuiLauncherState {
    pub const fn new() -> Self {
        Self {
            running: AtomicBool::new(false),
            window_hidden: AtomicBool::new(false),
        }
    }

    pub fn is_running(&self) -> bool {
        self.running.load(Ordering::SeqCst)
    }

    pub fn is_window_hidden(&self) -> bool {
        self.window_hidden.load(Ordering::SeqCst)
    }

    pub fn try_acquire<'a>(
        &'a self,
        window_controller: Arc<dyn MainWindowController>,
    ) -> Option<GpuiSessionGuard<'a>> {
        self.running
            .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
            .ok()?;
        self.window_hidden.store(false, Ordering::SeqCst);
        Some(GpuiSessionGuard {
            state: self,
            window_controller,
            released: AtomicBool::new(false),
        })
    }

    pub fn notify_main_window_shown(&self) {
        self.window_hidden.store(false, Ordering::SeqCst);
    }
}

pub struct GpuiSessionGuard<'a> {
    state: &'a GpuiLauncherState,
    window_controller: Arc<dyn MainWindowController>,
    released: AtomicBool,
}

impl GpuiSessionGuard<'_> {
    pub fn hide_main_window(&self) {
        self.state.window_hidden.store(true, Ordering::SeqCst);
        self.window_controller.hide_main_window();
    }

    pub fn finish_and_restore(self) {
        self.release_inner();
    }

    fn release_inner(&self) {
        if self.released.swap(true, Ordering::SeqCst) {
            return;
        }
        self.state.running.store(false, Ordering::SeqCst);
        if self.state.window_hidden.swap(false, Ordering::SeqCst) {
            self.window_controller.restore_main_window();
        }
    }
}

impl Drop for GpuiSessionGuard<'_> {
    fn drop(&mut self) {
        self.release_inner();
    }
}

/// Returns whether a GPUI shell child process is currently tracked as running.
pub fn is_gpui_shell_running() -> bool {
    GPUI_LAUNCHER_STATE.is_running()
}

/// Notifies the GPUI launcher state that the main Tauri window was revealed externally
/// (e.g. via system tray click or single-instance reactivation).
pub fn notify_main_window_shown() {
    GPUI_LAUNCHER_STATE.notify_main_window_shown();
}

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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GpuiCandidateMode {
    Release,
    Debug,
}

#[cfg(debug_assertions)]
fn compile_time_workspace_root() -> Option<&'static Path> {
    Path::new(env!("CARGO_MANIFEST_DIR")).parent()
}

#[cfg(not(debug_assertions))]
fn compile_time_workspace_root() -> Option<&'static Path> {
    None
}

#[cfg(debug_assertions)]
const fn compile_time_candidate_mode() -> GpuiCandidateMode {
    GpuiCandidateMode::Debug
}

#[cfg(not(debug_assertions))]
const fn compile_time_candidate_mode() -> GpuiCandidateMode {
    GpuiCandidateMode::Release
}

/// Returns ordered candidate paths for locating the GPUI binary for the given resolution mode:
/// - `GpuiCandidateMode::Release`: ONLY `<current_exe_dir>/orbit-gpui[.exe]` (installed sidecar).
/// - `GpuiCandidateMode::Debug`:
///   1. `<current_exe_dir>/orbit-gpui[.exe]`
///   2. `<current_exe_dir>/orbit[.exe]`
///   3. `<workspace_root>/target/{debug,release}/orbit-gpui[.exe]` and `orbit[.exe]`
///
/// Never returns a candidate whose path matches `current_exe` (case-insensitive on Windows) to prevent self-spawn loops.
pub fn resolve_gpui_binary_candidates_for_mode(
    current_exe: &Path,
    mode: GpuiCandidateMode,
    workspace_root: Option<&Path>,
) -> Vec<PathBuf> {
    let sidecar_bin = gpui_sidecar_binary_name();
    let mut candidates = Vec::new();

    if let Some(dir) = current_exe.parent() {
        push_candidate_if_distinct(&mut candidates, dir.join(sidecar_bin), current_exe);
        if mode == GpuiCandidateMode::Debug {
            push_candidate_if_distinct(
                &mut candidates,
                dir.join(gpui_dev_binary_name()),
                current_exe,
            );
        }
    }

    if mode == GpuiCandidateMode::Debug
        && let Some(workspace_root) = workspace_root
    {
        let dev_bin = gpui_dev_binary_name();
        let target_dir = workspace_root.join("target");
        for profile in ["debug", "release"] {
            let profile_dir = target_dir.join(profile);
            push_candidate_if_distinct(&mut candidates, profile_dir.join(sidecar_bin), current_exe);
            push_candidate_if_distinct(&mut candidates, profile_dir.join(dev_bin), current_exe);
        }
    }

    candidates
}

/// Returns ordered candidate paths for locating the GPUI binary using the compile-time build mode.
pub fn resolve_gpui_binary_candidates(current_exe: &Path) -> Vec<PathBuf> {
    resolve_gpui_binary_candidates_for_mode(
        current_exe,
        compile_time_candidate_mode(),
        compile_time_workspace_root(),
    )
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

pub const GPUI_STARTUP_GRACE_WINDOW: Duration = Duration::from_millis(300);
pub const GPUI_STARTUP_CRASH_THRESHOLD: Duration = Duration::from_secs(3);
pub const GPUI_STDERR_CAPTURE_LIMIT: usize = 4096;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GpuiLaunchFailure {
    FeatureDisabled,
    AlreadyRunning,
    BinaryNotFound,
    SpawnFailed {
        kind: std::io::ErrorKind,
        detail: String,
    },
    StartupCrashed {
        exit_code: Option<i32>,
        stderr_snippet: Option<String>,
    },
    RuntimeExitedWithError {
        exit_code: Option<i32>,
        stderr_snippet: Option<String>,
    },
    WaitFailed {
        detail: String,
    },
}

fn is_graphics_backend_error(stderr: &str) -> bool {
    let lower = stderr.to_lowercase();
    lower.contains("failed to open orbit gpui window")
        || lower.contains("vulkan")
        || lower.contains("dx12")
        || lower.contains("directx")
        || lower.contains("d3d12")
        || lower.contains("metal")
        || lower.contains("gpu")
        || lower.contains("graphics")
        || lower.contains("wgpu")
        || lower.contains("adapter")
}

fn has_fatal_startup_stderr(stderr: &str) -> bool {
    let lower = stderr.to_lowercase();
    lower.contains("failed to open orbit gpui window")
        || lower.contains("panicked at")
        || lower.contains("fatal error")
}

pub fn classify_gpui_exit(
    exit_code: Option<i32>,
    succeeded: bool,
    elapsed: Duration,
    stderr_snippet: Option<String>,
    startup_crash_threshold: Duration,
) -> Option<GpuiLaunchFailure> {
    let normalized_stderr = stderr_snippet
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty());

    if succeeded {
        if elapsed < startup_crash_threshold
            && let Some(ref stderr) = normalized_stderr
            && has_fatal_startup_stderr(stderr)
        {
            return Some(GpuiLaunchFailure::StartupCrashed {
                exit_code,
                stderr_snippet: normalized_stderr,
            });
        }
        return None;
    }

    if elapsed < startup_crash_threshold {
        Some(GpuiLaunchFailure::StartupCrashed {
            exit_code,
            stderr_snippet: normalized_stderr,
        })
    } else {
        Some(GpuiLaunchFailure::RuntimeExitedWithError {
            exit_code,
            stderr_snippet: normalized_stderr,
        })
    }
}

pub fn format_gpui_failure_message(failure: &GpuiLaunchFailure) -> String {
    match failure {
        GpuiLaunchFailure::FeatureDisabled => {
            "Experience Features must be enabled in Settings to launch the Native UI (GPUI) shell."
                .to_string()
        }
        GpuiLaunchFailure::AlreadyRunning => {
            "The Native UI (GPUI) shell is already running.".to_string()
        }
        GpuiLaunchFailure::BinaryNotFound => {
            "The Native UI (GPUI) shell is not available in this installation. Please reinstall or update Orbit to restore it."
                .to_string()
        }
        GpuiLaunchFailure::SpawnFailed { kind, detail } => match kind {
            std::io::ErrorKind::PermissionDenied => {
                "Permission denied while starting the Native UI (GPUI) shell. Check security or antivirus settings and try again."
                    .to_string()
            }
            std::io::ErrorKind::NotFound => {
                "The Native UI (GPUI) shell executable could not be found. Please reinstall or update Orbit."
                    .to_string()
            }
            _ => format!(
                "Unable to start the Native UI (GPUI) shell ({detail}). Check application logs for details."
            ),
        },
        GpuiLaunchFailure::StartupCrashed {
            exit_code,
            stderr_snippet,
        } => {
            if stderr_snippet
                .as_deref()
                .is_some_and(is_graphics_backend_error)
            {
                return "The Native UI (GPUI) shell could not open a window on your system's graphics backend. Update your GPU drivers or continue using the standard Orbit window."
                    .to_string();
            }
            match exit_code {
                Some(code) => format!(
                    "The Native UI (GPUI) shell closed unexpectedly during startup (exit code {code}). Check application logs for details."
                ),
                None => "The Native UI (GPUI) shell terminated unexpectedly during startup. Check application logs for details."
                    .to_string(),
            }
        }
        GpuiLaunchFailure::RuntimeExitedWithError { exit_code, .. } => match exit_code {
            Some(code) => format!(
                "The Native UI (GPUI) shell exited unexpectedly (exit code {code}). Restored the main Orbit window."
            ),
            None => "The Native UI (GPUI) shell terminated unexpectedly. Restored the main Orbit window."
                .to_string(),
        },
        GpuiLaunchFailure::WaitFailed { .. } => {
            "Lost track of the Native UI (GPUI) shell process. Restored the main Orbit window."
                .to_string()
        }
    }
}

/// Drains an async reader completely to EOF while retaining at most `max_bytes` of UTF-8 text.
/// Draining to EOF prevents child process pipe deadlocks even when output exceeds `max_bytes`.
pub async fn drain_stderr_bounded<R>(mut reader: R, max_bytes: usize) -> Option<String>
where
    R: AsyncRead + Unpin,
{
    let mut captured = Vec::with_capacity(max_bytes.min(1024));
    let mut chunk = [0u8; 1024];

    loop {
        match reader.read(&mut chunk).await {
            Ok(0) => break,
            Ok(n) => {
                if captured.len() < max_bytes {
                    let remaining = max_bytes - captured.len();
                    let take = n.min(remaining);
                    captured.extend_from_slice(&chunk[..take]);
                }
            }
            Err(err) => {
                tracing::debug!(error = %err, "Error while reading GPUI stderr stream");
                break;
            }
        }
    }

    if captured.is_empty() {
        return None;
    }

    let text = String::from_utf8_lossy(&captured);
    let trimmed = text.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}

async fn emit_gpui_failure(
    writer: &Arc<Mutex<WsWriter>>,
    token: &str,
    failure: &GpuiLaunchFailure,
) {
    let _ = Bridge::send_event(
        writer,
        token,
        &OrbitEvent::ErrorOccurred {
            message: format_gpui_failure_message(failure),
        },
    )
    .await;
}

async fn await_stderr_handle(
    stderr_handle: Option<tokio::task::JoinHandle<Option<String>>>,
) -> Option<String> {
    match stderr_handle {
        Some(handle) => match tokio::time::timeout(Duration::from_secs(2), handle).await {
            Ok(Ok(snippet)) => snippet,
            Ok(Err(err)) => {
                tracing::debug!(error = %err, "GPUI stderr drain task failed");
                None
            }
            Err(_) => {
                tracing::debug!("Timed out waiting for GPUI stderr drain task");
                None
            }
        },
        None => None,
    }
}

async fn report_gpui_exit(
    wait_result: std::io::Result<std::process::ExitStatus>,
    elapsed: Duration,
    stderr_snippet: Option<String>,
    startup_crash_threshold: Duration,
    binary_path: &Path,
    writer: &Arc<Mutex<WsWriter>>,
    token: &str,
) {
    match wait_result {
        Ok(status) => {
            if let Some(failure) = classify_gpui_exit(
                status.code(),
                status.success(),
                elapsed,
                stderr_snippet.clone(),
                startup_crash_threshold,
            ) {
                tracing::error!(
                    path = %binary_path.display(),
                    exit_code = ?status.code(),
                    elapsed_ms = elapsed.as_millis(),
                    stderr = ?stderr_snippet,
                    "Orbit GPUI shell exited abnormally"
                );
                emit_gpui_failure(writer, token, &failure).await;
            }
        }
        Err(err) => {
            tracing::error!(
                error = %err,
                path = %binary_path.display(),
                "Failed while waiting for orbit GPUI process exit"
            );
            let failure = GpuiLaunchFailure::WaitFailed {
                detail: err.to_string(),
            };
            emit_gpui_failure(writer, token, &failure).await;
        }
    }
}

pub async fn run_gpui_launch_flow<F>(
    state: &GpuiLauncherState,
    window_controller: Arc<dyn MainWindowController>,
    command_builder: Option<(PathBuf, F)>,
    grace_window: Duration,
    startup_crash_threshold: Duration,
    writer: Arc<Mutex<WsWriter>>,
    token: String,
) where
    F: FnOnce(&Path) -> tokio::process::Command,
{
    let Some(guard) = state.try_acquire(window_controller) else {
        tracing::debug!("GPUI shell is already running; notifying user of duplicate launch request");
        emit_gpui_failure(&writer, &token, &GpuiLaunchFailure::AlreadyRunning).await;
        return;
    };

    let Some((binary_path, build_cmd)) = command_builder else {
        guard.finish_and_restore();
        emit_gpui_failure(&writer, &token, &GpuiLaunchFailure::BinaryNotFound).await;
        return;
    };

    let mut cmd = build_cmd(&binary_path);
    cmd.stderr(std::process::Stdio::piped());

    let started_at = std::time::Instant::now();
    let mut child = match cmd.spawn() {
        Ok(child) => child,
        Err(err) => {
            guard.finish_and_restore();
            tracing::error!(
                error = %err,
                path = %binary_path.display(),
                "Failed to spawn orbit GPUI binary"
            );
            let failure = GpuiLaunchFailure::SpawnFailed {
                kind: err.kind(),
                detail: err.to_string(),
            };
            emit_gpui_failure(&writer, &token, &failure).await;
            return;
        }
    };

    let stderr_handle = child.stderr.take().map(|stderr| {
        tokio::spawn(async move { drain_stderr_bounded(stderr, GPUI_STDERR_CAPTURE_LIMIT).await })
    });

    let wait_result = tokio::select! {
        res = child.wait() => res,
        () = tokio::time::sleep(grace_window) => {
            guard.hide_main_window();
            child.wait().await
        }
    };

    let elapsed = started_at.elapsed();
    let stderr_snippet = await_stderr_handle(stderr_handle).await;
    guard.finish_and_restore();

    report_gpui_exit(
        wait_result,
        elapsed,
        stderr_snippet,
        startup_crash_threshold,
        &binary_path,
        &writer,
        &token,
    )
    .await;
}

#[cfg(windows)]
const CREATE_NO_WINDOW: u32 = 0x0800_0000;

#[cfg(windows)]
fn build_gpui_launch_command(path: &Path) -> tokio::process::Command {
    let mut cmd = tokio::process::Command::new(path);
    cmd.creation_flags(CREATE_NO_WINDOW);
    cmd
}

#[cfg(not(windows))]
fn build_gpui_launch_command(path: &Path) -> tokio::process::Command {
    tokio::process::Command::new(path)
}

fn find_gpui_binary_with_candidates() -> (Option<PathBuf>, Vec<PathBuf>) {
    let Ok(current_exe) = std::env::current_exe() else {
        return (None, Vec::new());
    };
    let candidates = resolve_gpui_binary_candidates(&current_exe);
    let found = find_gpui_binary_from_candidates(&candidates);
    (found, candidates)
}

pub fn launch_gpui_shell(writer: Arc<Mutex<WsWriter>>, token: String) {
    if !OrbitConfig::load().enable_experience_features() {
        tracing::warn!("Ignoring launchGpuiShell because enableExperienceFeatures is disabled");
        tokio::spawn(async move {
            emit_gpui_failure(&writer, &token, &GpuiLaunchFailure::FeatureDisabled).await;
        });
        return;
    }

    tokio::spawn(async move {
        let command_builder = if GPUI_LAUNCHER_STATE.is_running() {
            None
        } else {
            let (binary_path, candidates) = find_gpui_binary_with_candidates();
            if binary_path.is_none() {
                tracing::error!(
                    ?candidates,
                    "Could not locate orbit GPUI binary in resolved candidate paths"
                );
            }
            binary_path.map(|path| (path, build_gpui_launch_command))
        };

        run_gpui_launch_flow(
            &GPUI_LAUNCHER_STATE,
            Arc::new(TauriMainWindowController),
            command_builder,
            GPUI_STARTUP_GRACE_WINDOW,
            GPUI_STARTUP_CRASH_THRESHOLD,
            writer,
            token,
        )
        .await;
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
        assert_eq!(
            candidates[0],
            PathBuf::from("D:/Projects/orbit/target/debug").join(gpui_sidecar_binary_name())
        );
        if cfg!(debug_assertions) {
            assert!(candidates.len() >= 2);
            assert_eq!(
                candidates[1],
                PathBuf::from("D:/Projects/orbit/target/debug").join(gpui_dev_binary_name())
            );
            assert!(candidates.iter().any(|p| p.ends_with(Path::new("target").join("debug").join(gpui_sidecar_binary_name()))));
            assert!(candidates.iter().any(|p| p.ends_with(Path::new("target").join("debug").join(gpui_dev_binary_name()))));
            assert!(candidates.iter().any(|p| p.ends_with(Path::new("target").join("release").join(gpui_sidecar_binary_name()))));
            assert!(candidates.iter().any(|p| p.ends_with(Path::new("target").join("release").join(gpui_dev_binary_name()))));
        } else {
            assert_eq!(candidates.len(), 1);
        }
    }

    #[test]
    fn test_resolve_gpui_binary_candidates_release_mode_excludes_workspace_and_dev_paths() {
        let exe_path = if cfg!(windows) {
            PathBuf::from("C:/Program Files/Orbit/Orbit.exe")
        } else {
            PathBuf::from("/opt/orbit/orbit")
        };
        let workspace_root = Path::new("D:/a/orbit/orbit");
        let candidates = resolve_gpui_binary_candidates_for_mode(
            &exe_path,
            GpuiCandidateMode::Release,
            Some(workspace_root),
        );

        let expected_dir = exe_path.parent().unwrap();
        assert_eq!(
            candidates,
            vec![expected_dir.join(gpui_sidecar_binary_name())],
            "Release mode must only resolve the bundled sidecar next to current_exe"
        );
        assert!(
            !candidates
                .iter()
                .any(|p| p.to_string_lossy().contains("target")),
            "Release mode must never include Cargo target/ workspace paths"
        );
        assert!(
            !candidates
                .iter()
                .any(|p| p.file_name().and_then(|n| n.to_str()) == Some(gpui_dev_binary_name())),
            "Release mode must never include dev binary fallback names"
        );
    }

    #[test]
    fn test_resolve_gpui_binary_candidates_debug_mode_includes_workspace_fallbacks() {
        let exe_path = PathBuf::from("D:/Projects/orbit/target/debug/orbit-app.exe");
        let workspace_root = Path::new("D:/Projects/orbit");
        let candidates = resolve_gpui_binary_candidates_for_mode(
            &exe_path,
            GpuiCandidateMode::Debug,
            Some(workspace_root),
        );

        assert!(candidates.len() >= 2);
        assert_eq!(
            candidates[0],
            PathBuf::from("D:/Projects/orbit/target/debug").join(gpui_sidecar_binary_name())
        );
        assert_eq!(
            candidates[1],
            PathBuf::from("D:/Projects/orbit/target/debug").join(gpui_dev_binary_name())
        );
        assert!(candidates.iter().any(|p| p.ends_with(
            Path::new("target")
                .join("debug")
                .join(gpui_sidecar_binary_name())
        )));
        assert!(candidates.iter().any(|p| p.ends_with(
            Path::new("target")
                .join("debug")
                .join(gpui_dev_binary_name())
        )));
        assert!(candidates.iter().any(|p| p.ends_with(
            Path::new("target")
                .join("release")
                .join(gpui_sidecar_binary_name())
        )));
        assert!(candidates.iter().any(|p| p.ends_with(
            Path::new("target")
                .join("release")
                .join(gpui_dev_binary_name())
        )));
    }

    #[test]
    fn test_resolve_gpui_binary_candidates_never_returns_current_exe() {
        let exe_path = if cfg!(windows) {
            PathBuf::from("C:/Users/dev/AppData/Local/Orbit/Orbit.exe")
        } else {
            PathBuf::from("/opt/orbit/orbit")
        };
        let exe_dir = exe_path.parent().unwrap();
        for mode in [GpuiCandidateMode::Release, GpuiCandidateMode::Debug] {
            let candidates =
                resolve_gpui_binary_candidates_for_mode(&exe_path, mode, Some(exe_dir));

            assert_eq!(
                candidates[0],
                exe_dir.join(gpui_sidecar_binary_name()),
                "Bundled sidecar must be candidate 0 for {mode:?}"
            );
            assert!(
                !candidates
                    .iter()
                    .any(|p| p == &exe_dir.join(gpui_dev_binary_name())),
                "Candidate matching current_exe must be excluded for {mode:?}"
            );
        }

        // Also ensure default resolve_gpui_binary_candidates excludes current_exe
        let default_candidates = resolve_gpui_binary_candidates(&exe_path);
        assert_eq!(default_candidates[0], exe_dir.join(gpui_sidecar_binary_name()));
        assert!(
            !default_candidates
                .iter()
                .any(|p| p == &exe_dir.join(gpui_dev_binary_name()))
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

    #[test]
    fn test_format_gpui_failure_messages_are_user_facing_without_dev_commands() {
        let variants = vec![
            GpuiLaunchFailure::FeatureDisabled,
            GpuiLaunchFailure::AlreadyRunning,
            GpuiLaunchFailure::BinaryNotFound,
            GpuiLaunchFailure::SpawnFailed {
                kind: std::io::ErrorKind::PermissionDenied,
                detail: "access denied".to_string(),
            },
            GpuiLaunchFailure::SpawnFailed {
                kind: std::io::ErrorKind::NotFound,
                detail: "file not found".to_string(),
            },
            GpuiLaunchFailure::StartupCrashed {
                exit_code: Some(1),
                stderr_snippet: None,
            },
            GpuiLaunchFailure::StartupCrashed {
                exit_code: Some(1),
                stderr_snippet: Some(
                    "Failed to open Orbit GPUI window: No supported Vulkan/DX12 adapter"
                        .to_string(),
                ),
            },
            GpuiLaunchFailure::RuntimeExitedWithError {
                exit_code: Some(101),
                stderr_snippet: Some("panic: unexpected state".to_string()),
            },
            GpuiLaunchFailure::WaitFailed {
                detail: "wait error".to_string(),
            },
        ];

        for failure in &variants {
            let msg = format_gpui_failure_message(failure);
            assert!(
                !msg.trim().is_empty(),
                "Failure message must be non-empty for {failure:?}"
            );
            let lower = msg.to_lowercase();
            assert!(
                !lower.contains("cargo build"),
                "User-facing message must never mention `cargo build`: {msg}"
            );
            assert!(
                !msg.contains("CARGO_MANIFEST_DIR"),
                "User-facing message must never mention CARGO_MANIFEST_DIR: {msg}"
            );
            assert!(
                !lower.contains("-p orbit-gpui"),
                "User-facing message must never mention `-p orbit-gpui`: {msg}"
            );
        }

        let gpu_msg = format_gpui_failure_message(&GpuiLaunchFailure::StartupCrashed {
            exit_code: Some(1),
            stderr_snippet: Some(
                "Failed to open Orbit GPUI window: No supported Vulkan/DX12 adapter".to_string(),
            ),
        });
        assert!(
            gpu_msg.to_lowercase().contains("graphics")
                || gpu_msg.to_lowercase().contains("gpu"),
            "Graphics backend failure should mention graphics/GPU guidance: {gpu_msg}"
        );
    }

    #[test]
    fn test_classify_gpui_exit_handles_normal_crash_and_immediate_startup_failures() {
        let threshold = GPUI_STARTUP_CRASH_THRESHOLD;

        // Exit code 0 after 10s -> None (normal exit)
        assert_eq!(
            classify_gpui_exit(Some(0), true, std::time::Duration::from_secs(10), None, threshold),
            None
        );

        // Exit code 0 after 200ms with empty stderr -> None
        assert_eq!(
            classify_gpui_exit(
                Some(0),
                true,
                std::time::Duration::from_millis(200),
                Some("   ".to_string()),
                threshold,
            ),
            None
        );

        // Exit code 0 after 200ms with fatal window-open stderr -> StartupCrashed
        assert_eq!(
            classify_gpui_exit(
                Some(0),
                true,
                std::time::Duration::from_millis(200),
                Some("Failed to open Orbit GPUI window: DX12 unavailable".to_string()),
                threshold,
            ),
            Some(GpuiLaunchFailure::StartupCrashed {
                exit_code: Some(0),
                stderr_snippet: Some(
                    "Failed to open Orbit GPUI window: DX12 unavailable".to_string()
                ),
            })
        );

        // Exit code 1 after 150ms -> StartupCrashed
        assert_eq!(
            classify_gpui_exit(
                Some(1),
                false,
                std::time::Duration::from_millis(150),
                None,
                threshold,
            ),
            Some(GpuiLaunchFailure::StartupCrashed {
                exit_code: Some(1),
                stderr_snippet: None,
            })
        );

        // Exit code 101 after 30s -> RuntimeExitedWithError
        assert_eq!(
            classify_gpui_exit(
                Some(101),
                false,
                std::time::Duration::from_secs(30),
                None,
                threshold,
            ),
            Some(GpuiLaunchFailure::RuntimeExitedWithError {
                exit_code: Some(101),
                stderr_snippet: None,
            })
        );

        // Signal termination (exit_code: None, succeeded: false) after 500ms -> StartupCrashed
        assert_eq!(
            classify_gpui_exit(
                None,
                false,
                std::time::Duration::from_millis(500),
                None,
                threshold,
            ),
            Some(GpuiLaunchFailure::StartupCrashed {
                exit_code: None,
                stderr_snippet: None,
            })
        );
    }

    #[tokio::test]
    async fn test_drain_stderr_bounded_truncates_output_without_blocking() {
        let large_payload = vec![b'x'; GPUI_STDERR_CAPTURE_LIMIT * 2 + 512];
        let mut cursor: &[u8] = &large_payload;
        let captured = drain_stderr_bounded(&mut cursor, GPUI_STDERR_CAPTURE_LIMIT).await;

        assert!(captured.is_some(), "Non-empty stderr must return Some");
        let text = captured.unwrap();
        assert!(
            text.len() <= GPUI_STDERR_CAPTURE_LIMIT,
            "Captured stderr ({} bytes) must not exceed limit ({} bytes)",
            text.len(),
            GPUI_STDERR_CAPTURE_LIMIT
        );
        assert!(
            cursor.is_empty(),
            "Entire stderr stream must be drained to EOF to avoid child pipe deadlock"
        );
    }

    struct MockMainWindowController {
        hide_count: std::sync::atomic::AtomicUsize,
        restore_count: std::sync::atomic::AtomicUsize,
    }

    impl MockMainWindowController {
        fn new() -> Self {
            Self {
                hide_count: std::sync::atomic::AtomicUsize::new(0),
                restore_count: std::sync::atomic::AtomicUsize::new(0),
            }
        }

        fn hides(&self) -> usize {
            self.hide_count.load(Ordering::SeqCst)
        }

        fn restores(&self) -> usize {
            self.restore_count.load(Ordering::SeqCst)
        }
    }

    impl MainWindowController for MockMainWindowController {
        fn hide_main_window(&self) {
            self.hide_count.fetch_add(1, Ordering::SeqCst);
        }

        fn restore_main_window(&self) {
            self.restore_count.fetch_add(1, Ordering::SeqCst);
        }
    }

    fn platform_shell_command(windows_cmd: &str, unix_cmd: &str) -> tokio::process::Command {
        if cfg!(windows) {
            let mut cmd = tokio::process::Command::new("cmd");
            cmd.args(["/C", windows_cmd]);
            cmd
        } else {
            let mut cmd = tokio::process::Command::new("sh");
            cmd.args(["-c", unix_cmd]);
            cmd
        }
    }

    #[test]
    fn test_gpui_session_guard_drop_recovers_state_and_restores_hidden_window() {
        let state = GpuiLauncherState::new();
        let controller = Arc::new(MockMainWindowController::new());

        {
            let guard = state
                .try_acquire(controller.clone())
                .expect("Guard acquisition must succeed when idle");
            assert!(state.is_running());
            guard.hide_main_window();
            assert!(state.is_window_hidden());
            assert_eq!(controller.hides(), 1);
            // Drop guard without calling finish_and_restore
        }

        assert!(!state.is_running(), "Drop must reset running to false");
        assert!(
            !state.is_window_hidden(),
            "Drop must clear window_hidden flag"
        );
        assert_eq!(
            controller.restores(),
            1,
            "Drop must restore hidden main window"
        );
    }

    #[test]
    fn test_gpui_launcher_single_instance_reactivation_clears_hidden_flag_while_preserving_running() {
        let state = GpuiLauncherState::new();
        let controller = Arc::new(MockMainWindowController::new());

        let guard = state
            .try_acquire(controller.clone())
            .expect("Guard acquisition must succeed");
        guard.hide_main_window();
        assert!(state.is_running());
        assert!(state.is_window_hidden());

        // Simulate user clicking tray or second-instance reactivation
        state.notify_main_window_shown();
        assert!(
            state.is_running(),
            "Reactivating main window must keep running = true while GPUI child is alive"
        );
        assert!(
            !state.is_window_hidden(),
            "Reactivating main window must clear window_hidden"
        );

        guard.finish_and_restore();
        assert!(!state.is_running());
        assert!(!state.is_window_hidden());
        assert_eq!(
            controller.restores(),
            0,
            "No redundant restore needed when window was already shown"
        );
    }

    #[tokio::test]
    async fn test_gpui_launch_flow_spawn_failure_recovers_state_and_emits_user_error() {
        let _guard = crate::ipc::emitter::TEST_EMITTER_LOCK.lock().await;
        let mock_emitter = Arc::new(MockEventEmitter::new());
        set_global_emitter(mock_emitter.clone());

        let state = GpuiLauncherState::new();
        let controller = Arc::new(MockMainWindowController::new());
        let writer = Arc::new(Mutex::new(WsWriter::Emitter));

        let nonexistent = PathBuf::from("nonexistent_orbit_gpui_binary_489_test");
        run_gpui_launch_flow(
            &state,
            controller.clone(),
            Some((nonexistent, |path: &Path| {
                tokio::process::Command::new(path)
            })),
            Duration::from_millis(50),
            GPUI_STARTUP_CRASH_THRESHOLD,
            writer,
            "token".to_string(),
        )
        .await;

        assert!(!state.is_running());
        assert_eq!(controller.hides(), 0);
        assert_eq!(controller.restores(), 0);

        let emitted = mock_emitter.get_emitted_events().await;
        let error_msg = emitted
            .iter()
            .find(|(name, _)| name == "errorOccurred")
            .and_then(|(_, data)| data.get("message").and_then(|m| m.as_str()));
        assert!(
            error_msg.is_some(),
            "Spawn failure must emit errorOccurred event"
        );
        assert!(
            !error_msg.unwrap().to_lowercase().contains("cargo build"),
            "Error message must not contain dev build commands"
        );
    }

    #[tokio::test]
    async fn test_gpui_launch_flow_immediate_crash_skips_hide_and_emits_error() {
        let _guard = crate::ipc::emitter::TEST_EMITTER_LOCK.lock().await;
        let mock_emitter = Arc::new(MockEventEmitter::new());
        set_global_emitter(mock_emitter.clone());

        let state = GpuiLauncherState::new();
        let controller = Arc::new(MockMainWindowController::new());
        let writer = Arc::new(Mutex::new(WsWriter::Emitter));

        run_gpui_launch_flow(
            &state,
            controller.clone(),
            Some((PathBuf::from("mock-gpui"), |_path: &Path| {
                platform_shell_command(
                    "echo Failed to open Orbit GPUI window: DX12 unavailable 1>&2 & exit /B 1",
                    "echo 'Failed to open Orbit GPUI window: Vulkan unavailable' >&2; exit 1",
                )
            })),
            Duration::from_secs(2),
            GPUI_STARTUP_CRASH_THRESHOLD,
            writer,
            "token".to_string(),
        )
        .await;

        assert!(!state.is_running());
        assert_eq!(
            controller.hides(),
            0,
            "Immediate crash within grace window must NOT hide main window"
        );
        assert_eq!(controller.restores(), 0);

        let emitted = mock_emitter.get_emitted_events().await;
        let error_msg = emitted
            .iter()
            .find(|(name, _)| name == "errorOccurred")
            .and_then(|(_, data)| data.get("message").and_then(|m| m.as_str()));
        assert!(
            error_msg.is_some(),
            "Immediate crash must emit errorOccurred event"
        );
        let lower = error_msg.unwrap().to_lowercase();
        assert!(
            lower.contains("graphics") || lower.contains("gpu"),
            "Graphics startup crash should inform user about graphics backend: {lower}"
        );
    }

    #[tokio::test]
    async fn test_gpui_launch_flow_nonzero_exit_after_hide_restores_window_and_emits_error() {
        let _guard = crate::ipc::emitter::TEST_EMITTER_LOCK.lock().await;
        let mock_emitter = Arc::new(MockEventEmitter::new());
        set_global_emitter(mock_emitter.clone());

        let state = GpuiLauncherState::new();
        let controller = Arc::new(MockMainWindowController::new());
        let writer = Arc::new(Mutex::new(WsWriter::Emitter));

        run_gpui_launch_flow(
            &state,
            controller.clone(),
            Some((PathBuf::from("mock-gpui"), |_path: &Path| {
                platform_shell_command(
                    "ping -n 2 127.0.0.1 >nul & exit /B 2",
                    "sleep 0.15; exit 2",
                )
            })),
            Duration::from_millis(20),
            GPUI_STARTUP_CRASH_THRESHOLD,
            writer,
            "token".to_string(),
        )
        .await;

        assert!(!state.is_running());
        assert_eq!(controller.hides(), 1);
        assert_eq!(controller.restores(), 1);

        let emitted = mock_emitter.get_emitted_events().await;
        assert!(
            emitted.iter().any(|(name, _)| name == "errorOccurred"),
            "Non-zero exit after hide must emit errorOccurred"
        );
    }

    #[tokio::test]
    async fn test_gpui_launch_flow_clean_exit_restores_window_without_error() {
        let _guard = crate::ipc::emitter::TEST_EMITTER_LOCK.lock().await;
        let mock_emitter = Arc::new(MockEventEmitter::new());
        set_global_emitter(mock_emitter.clone());

        let state = GpuiLauncherState::new();
        let controller = Arc::new(MockMainWindowController::new());
        let writer = Arc::new(Mutex::new(WsWriter::Emitter));

        run_gpui_launch_flow(
            &state,
            controller.clone(),
            Some((PathBuf::from("mock-gpui"), |_path: &Path| {
                platform_shell_command(
                    "ping -n 2 127.0.0.1 >nul & exit /B 0",
                    "sleep 0.15; exit 0",
                )
            })),
            Duration::from_millis(20),
            GPUI_STARTUP_CRASH_THRESHOLD,
            writer,
            "token".to_string(),
        )
        .await;

        assert!(!state.is_running());
        assert_eq!(controller.hides(), 1);
        assert_eq!(controller.restores(), 1);

        let emitted = mock_emitter.get_emitted_events().await;
        assert!(
            !emitted.iter().any(|(name, _)| name == "errorOccurred"),
            "Clean exit (code 0) must not emit errorOccurred"
        );
    }

    #[tokio::test]
    async fn test_gpui_launch_flow_duplicate_request_while_running_emits_feedback() {
        let _guard = crate::ipc::emitter::TEST_EMITTER_LOCK.lock().await;
        let mock_emitter = Arc::new(MockEventEmitter::new());
        set_global_emitter(mock_emitter.clone());

        let state = GpuiLauncherState::new();
        let controller = Arc::new(MockMainWindowController::new());
        let writer = Arc::new(Mutex::new(WsWriter::Emitter));

        let _active_guard = state
            .try_acquire(controller.clone())
            .expect("Initial acquire must succeed");

        run_gpui_launch_flow(
            &state,
            controller.clone(),
            None::<(PathBuf, fn(&Path) -> tokio::process::Command)>,
            Duration::from_millis(20),
            GPUI_STARTUP_CRASH_THRESHOLD,
            writer,
            "token".to_string(),
        )
        .await;

        assert!(
            state.is_running(),
            "Duplicate launch must not clear active running state"
        );
        let emitted = mock_emitter.get_emitted_events().await;
        let error_msg = emitted
            .iter()
            .find(|(name, _)| name == "errorOccurred")
            .and_then(|(_, data)| data.get("message").and_then(|m| m.as_str()));
        assert!(
            error_msg.is_some_and(|m| m.to_lowercase().contains("already running")),
            "Duplicate launch must emit already-running feedback, got: {error_msg:?}"
        );
    }

    #[tokio::test]
    async fn test_build_gpui_launch_command_preserves_program_and_stderr_capture() {
        let _guard = crate::ipc::emitter::TEST_EMITTER_LOCK.lock().await;
        let mock_emitter = Arc::new(MockEventEmitter::new());
        set_global_emitter(mock_emitter.clone());

        let expected_path = Path::new("orbit-gpui.exe");
        let cmd = build_gpui_launch_command(expected_path);
        assert_eq!(cmd.as_std().get_program(), expected_path.as_os_str());

        let state = GpuiLauncherState::new();
        let controller = Arc::new(MockMainWindowController::new());
        let writer = Arc::new(Mutex::new(WsWriter::Emitter));

        run_gpui_launch_flow(
            &state,
            controller.clone(),
            Some((PathBuf::from("mock-gpui"), |_path: &Path| {
                #[cfg(windows)]
                {
                    let mut cmd = build_gpui_launch_command(Path::new("cmd"));
                    cmd.args([
                        "/C",
                        "echo Failed to open Orbit GPUI window: DX12 unavailable 1>&2 & exit /B 1",
                    ]);
                    cmd
                }
                #[cfg(not(windows))]
                {
                    let mut cmd = build_gpui_launch_command(Path::new("sh"));
                    cmd.args([
                        "-c",
                        "echo 'Failed to open Orbit GPUI window: Vulkan unavailable' >&2; exit 1",
                    ]);
                    cmd
                }
            })),
            Duration::from_secs(2),
            GPUI_STARTUP_CRASH_THRESHOLD,
            writer,
            "token".to_string(),
        )
        .await;

        assert!(!state.is_running());
        assert_eq!(controller.hides(), 0);

        let emitted = mock_emitter.get_emitted_events().await;
        let error_msg = emitted
            .iter()
            .find(|(name, _)| name == "errorOccurred")
            .and_then(|(_, data)| data.get("message").and_then(|m| m.as_str()));
        assert!(
            error_msg.is_some(),
            "Crash via build_gpui_launch_command must emit errorOccurred event"
        );
        let lower = error_msg.unwrap().to_lowercase();
        assert!(
            lower.contains("graphics") || lower.contains("gpu"),
            "Piped stderr diagnostic must be captured with CREATE_NO_WINDOW: {lower}"
        );
    }
}

