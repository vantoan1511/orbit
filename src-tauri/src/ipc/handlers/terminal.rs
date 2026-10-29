use std::collections::HashMap;
use std::io::{Read, Write};
use std::sync::{Arc, OnceLock};
use portable_pty::{native_pty_system, CommandBuilder, MasterPty, PtySize};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::sync::{Mutex, RwLock};
use crate::ipc::bridge::{Bridge, WsWriter};
use crate::ipc::events::OrbitEvent;
use crate::kubernetes::manager::KubeManager;

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct OpenPodTerminalRequest {
    pub session_id: String,
    pub namespace: String,
    pub pod: String,
    pub container: Option<String>,
    pub cols: Option<u16>,
    pub rows: Option<u16>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct OpenLocalTerminalRequest {
    pub session_id: String,
    pub shell: Option<String>,
    pub cols: Option<u16>,
    pub rows: Option<u16>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TerminalDataRequest {
    pub session_id: String,
    pub data: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TerminalResizeRequest {
    pub session_id: String,
    pub cols: u16,
    pub rows: u16,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CloseTerminalRequest {
    pub session_id: String,
}

pub enum TerminalSender {
    Local {
        writer: Arc<std::sync::Mutex<Box<dyn Write + Send>>>,
        master: Arc<std::sync::Mutex<Box<dyn MasterPty + Send>>>,
        child: Arc<std::sync::Mutex<Box<dyn portable_pty::Child + Send>>>,
    },
    Pod {
        stdin_tx: tokio::sync::mpsc::Sender<Vec<u8>>,
    },
}

pub struct TerminalSession {
    pub sender: TerminalSender,
    pub cancel_tx: Option<tokio::sync::oneshot::Sender<()>>,
    pub last_size: Arc<std::sync::Mutex<PtySize>>,
}

#[derive(Debug, Clone)]
pub enum LocalTerminalStreamEvent {
    Data(String),
    Closed(Option<i32>),
}

type SessionMap = Arc<RwLock<HashMap<String, TerminalSession>>>;

fn get_sessions() -> &'static SessionMap {
    static SESSIONS: OnceLock<SessionMap> = OnceLock::new();
    SESSIONS.get_or_init(|| Arc::new(RwLock::new(HashMap::new())))
}

pub fn find_in_path(exe: &str) -> Option<std::path::PathBuf> {
    let path_var = std::env::var_os("PATH")?;
    for dir in std::env::split_paths(&path_var) {
        let full = dir.join(exe);
        if full.is_file() {
            return Some(full);
        }
        #[cfg(target_os = "windows")]
        if !exe.ends_with(".exe") {
            let with_exe = dir.join(format!("{}.exe", exe));
            if with_exe.is_file() {
                return Some(with_exe);
            }
        }
    }
    None
}

pub fn is_executable_in_path(exe: &str) -> bool {
    find_in_path(exe).is_some()
}

#[cfg(target_os = "windows")]
pub fn find_windows_pwsh() -> Option<String> {
    // 1. Search PATH first
    if let Some(path) = find_in_path("pwsh.exe").or_else(|| find_in_path("pwsh")) {
        return Some(path.to_string_lossy().to_string());
    }

    // 2. Program Files candidates (64-bit and 32-bit)
    let pf_candidates = [
        ("ProgramFiles", "PowerShell\\7\\pwsh.exe"),
        ("ProgramFiles", "PowerShell\\7-preview\\pwsh.exe"),
        ("ProgramFiles(x86)", "PowerShell\\7\\pwsh.exe"),
        ("ProgramFiles(x86)", "PowerShell\\7-preview\\pwsh.exe"),
        ("ProgramW6432", "PowerShell\\7\\pwsh.exe"),
    ];
    for (env_var, rel_path) in pf_candidates {
        if let Some(base) = std::env::var_os(env_var) {
            let full = std::path::PathBuf::from(base).join(rel_path);
            if full.is_file() {
                return Some(full.to_string_lossy().to_string());
            }
        }
    }

    // 3. Local AppData candidates (User MSI install & WindowsApps execution alias)
    if let Some(local_app_data) = std::env::var_os("LOCALAPPDATA") {
        let base = std::path::PathBuf::from(local_app_data);
        let user_candidates = [
            base.join("Programs").join("PowerShell").join("7").join("pwsh.exe"),
            base.join("Programs").join("PowerShell").join("7-preview").join("pwsh.exe"),
            base.join("Microsoft").join("WindowsApps").join("pwsh.exe"),
        ];
        for candidate in user_candidates {
            if candidate.is_file() {
                return Some(candidate.to_string_lossy().to_string());
            }
        }
    }

    // 4. Scoop and Chocolatey shims
    if let Some(user_profile) = std::env::var_os("USERPROFILE") {
        let scoop_shim = std::path::PathBuf::from(user_profile)
            .join("scoop")
            .join("shims")
            .join("pwsh.exe");
        if scoop_shim.is_file() {
            return Some(scoop_shim.to_string_lossy().to_string());
        }
    }
    if let Some(prog_data) = std::env::var_os("ProgramData") {
        let choco_bin = std::path::PathBuf::from(prog_data)
            .join("chocolatey")
            .join("bin")
            .join("pwsh.exe");
        if choco_bin.is_file() {
            return Some(choco_bin.to_string_lossy().to_string());
        }
    }

    None
}

#[cfg(target_os = "windows")]
pub fn find_windows_powershell() -> Option<String> {
    if let Some(path) = find_in_path("powershell.exe").or_else(|| find_in_path("powershell")) {
        return Some(path.to_string_lossy().to_string());
    }
    if let Some(sys_root) = std::env::var_os("SystemRoot") {
        let full = std::path::PathBuf::from(sys_root)
            .join("System32")
            .join("WindowsPowerShell")
            .join("v1.0")
            .join("powershell.exe");
        if full.is_file() {
            return Some(full.to_string_lossy().to_string());
        }
    }
    Some("powershell.exe".to_string())
}

#[cfg(target_os = "windows")]
pub fn find_windows_cmd() -> Option<String> {
    if let Some(comspec) = std::env::var_os("COMSPEC") {
        let p = std::path::PathBuf::from(comspec);
        if p.is_file() {
            return Some(p.to_string_lossy().to_string());
        }
    }
    if let Some(sys_root) = std::env::var_os("SystemRoot") {
        let full = std::path::PathBuf::from(sys_root)
            .join("System32")
            .join("cmd.exe");
        if full.is_file() {
            return Some(full.to_string_lossy().to_string());
        }
    }
    find_in_path("cmd.exe")
        .map(|p| p.to_string_lossy().to_string())
        .or_else(|| Some("cmd.exe".to_string()))
}

pub fn detect_default_shell() -> String {
    #[cfg(target_os = "windows")]
    {
        if let Some(pwsh) = find_windows_pwsh() {
            return pwsh;
        }
        if let Some(ps) = find_windows_powershell() {
            return ps;
        }
        find_windows_cmd().unwrap_or_else(|| "cmd.exe".to_string())
    }
    #[cfg(not(target_os = "windows"))]
    {
        if let Ok(shell) = std::env::var("SHELL") {
            if !shell.trim().is_empty() && std::path::Path::new(&shell).is_file() {
                return shell;
            }
        }
        for fallback in &["/bin/zsh", "/bin/bash", "/usr/bin/zsh", "/usr/bin/bash", "/bin/sh"] {
            if std::path::Path::new(fallback).is_file() {
                return fallback.to_string();
            }
        }
        "/bin/sh".to_string()
    }
}

pub fn is_allowed_shell(shell: &str) -> bool {
    const ALLOWED_SHELLS: &[&str] = &[
        "pwsh", "pwsh.exe",
        "powershell", "powershell.exe",
        "cmd", "cmd.exe",
        "bash", "zsh", "sh", "fish",
    ];
    let file_name = std::path::Path::new(shell)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or(shell);
    ALLOWED_SHELLS.iter().any(|allowed| allowed.eq_ignore_ascii_case(file_name))
}

pub fn resolve_shell(req_shell: Option<String>, config_shell: Option<String>) -> String {
    let chosen = req_shell
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty() && is_allowed_shell(s))
        .or_else(|| {
            config_shell
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty() && is_allowed_shell(s))
        });

    match chosen {
        Some(shell) => {
            #[cfg(target_os = "windows")]
            {
                let path = std::path::Path::new(&shell);
                if path.is_file() || is_executable_in_path(&shell) {
                    return shell;
                }
                // Try resolving known shell names to their full paths if not directly in PATH
                let file_name = path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or(&shell);
                let lower = file_name.to_ascii_lowercase();
                if lower == "pwsh" || lower == "pwsh.exe" {
                    if let Some(resolved) = find_windows_pwsh() {
                        return resolved;
                    }
                } else if lower == "powershell" || lower == "powershell.exe" {
                    if let Some(resolved) = find_windows_powershell() {
                        return resolved;
                    }
                } else if lower == "cmd" || lower == "cmd.exe" {
                    if let Some(resolved) = find_windows_cmd() {
                        return resolved;
                    }
                } else if let Some(in_path) = find_in_path(&shell) {
                    return in_path.to_string_lossy().to_string();
                }
                shell
            }
            #[cfg(not(target_os = "windows"))]
            {
                let path = std::path::Path::new(&shell);
                if path.is_file() {
                    shell
                } else if let Some(in_path) = find_in_path(&shell) {
                    in_path.to_string_lossy().to_string()
                } else {
                    shell
                }
            }
        }
        None => detect_default_shell(),
    }
}

pub fn open_local_terminal(
    data: Option<Value>,
    writer: Arc<Mutex<WsWriter>>,
    token: String,
) {
    tokio::spawn(async move {
        let req = match data {
            Some(v) => match serde_json::from_value::<OpenLocalTerminalRequest>(v) {
                Ok(r) => r,
                Err(e) => {
                    tracing::error!(error = %e, "Failed to parse OpenLocalTerminalRequest");
                    return;
                }
            },
            None => return,
        };

        let initial_size = PtySize {
            rows: req.rows.unwrap_or(24),
            cols: req.cols.unwrap_or(80),
            pixel_width: 0,
            pixel_height: 0,
        };

        let pty_system = native_pty_system();
        let pair = match pty_system.openpty(initial_size) {
            Ok(p) => p,
            Err(e) => {
                let _ = Bridge::send_event(
                    &writer,
                    &token,
                    &OrbitEvent::ErrorOccurred {
                        message: format!("Failed to create local PTY: {}", e),
                    },
                ).await;
                return;
            }
        };

        let config_shell = crate::config::OrbitConfig::load().default_terminal_shell();
        let shell = resolve_shell(req.shell, config_shell);
        let cmd = CommandBuilder::new(&shell);

        let child = match pair.slave.spawn_command(cmd) {
            Ok(c) => c,
            Err(e) => {
                let _ = Bridge::send_event(
                    &writer,
                    &token,
                    &OrbitEvent::ErrorOccurred {
                        message: format!("Failed to spawn shell '{}': {}", shell, e),
                    },
                ).await;
                return;
            }
        };
        let child_arc: Arc<std::sync::Mutex<Box<dyn portable_pty::Child + Send>>> =
            Arc::new(std::sync::Mutex::new(child));

        let mut reader = match pair.master.try_clone_reader() {
            Ok(r) => r,
            Err(e) => {
                let _ = Bridge::send_event(
                    &writer,
                    &token,
                    &OrbitEvent::ErrorOccurred {
                        message: format!("Failed to clone PTY reader: {}", e),
                    },
                ).await;
                return;
            }
        };

        let pty_writer = match pair.master.take_writer() {
            Ok(w) => Arc::new(std::sync::Mutex::new(w)),
            Err(e) => {
                let _ = Bridge::send_event(
                    &writer,
                    &token,
                    &OrbitEvent::ErrorOccurred {
                        message: format!("Failed to take PTY writer: {}", e),
                    },
                ).await;
                return;
            }
        };

        let master = Arc::new(std::sync::Mutex::new(pair.master));

        let (cancel_tx, _cancel_rx) = tokio::sync::oneshot::channel::<()>();

        {
            let mut sessions = get_sessions().write().await;
            sessions.insert(
                req.session_id.clone(),
                TerminalSession {
                    sender: TerminalSender::Local {
                        writer: pty_writer,
                        master,
                        child: child_arc.clone(),
                    },
                    cancel_tx: Some(cancel_tx),
                    last_size: Arc::new(std::sync::Mutex::new(initial_size)),
                },
            );
        }

        let display_shell = std::path::Path::new(&shell)
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or(&shell)
            .to_string();

        let _ = Bridge::send_event(
            &writer,
            &token,
            &OrbitEvent::TerminalStarted {
                session_id: req.session_id.clone(),
                shell: display_shell,
            },
        ).await;

        let (stream_tx, mut stream_rx) = tokio::sync::mpsc::channel::<LocalTerminalStreamEvent>(100);

        let writer_for_stream = writer.clone();
        let token_for_stream = token.clone();
        let session_id_for_stream = req.session_id.clone();
        tokio::spawn(async move {
            while let Some(evt) = stream_rx.recv().await {
                match evt {
                    LocalTerminalStreamEvent::Data(text) => {
                        let _ = Bridge::send_event(
                            &writer_for_stream,
                            &token_for_stream,
                            &OrbitEvent::TerminalData {
                                session_id: session_id_for_stream.clone(),
                                data: text,
                            },
                        ).await;
                    }
                    LocalTerminalStreamEvent::Closed(exit_code) => {
                        let _ = Bridge::send_event(
                            &writer_for_stream,
                            &token_for_stream,
                            &OrbitEvent::TerminalClosed {
                                session_id: session_id_for_stream.clone(),
                                exit_code,
                            },
                        ).await;
                        let mut sessions = get_sessions().write().await;
                        sessions.remove(&session_id_for_stream);
                        break;
                    }
                }
            }
        });

        let child_for_wait = child_arc.clone();
        let stream_tx_for_wait = stream_tx.clone();

        // Spawn a dedicated thread to wait for child process termination without holding mutex indefinitely
        std::thread::spawn(move || {
            let exit_code = loop {
                match child_for_wait.lock() {
                    Ok(mut c) => match c.try_wait() {
                        Ok(Some(s)) => break Some(s.exit_code() as i32),
                        Ok(None) => {}
                        Err(_) => break None,
                    },
                    Err(_) => break None,
                }
                std::thread::sleep(std::time::Duration::from_millis(50));
            };

            // Brief pause to allow the reader thread to drain any trailing buffered output
            std::thread::sleep(std::time::Duration::from_millis(50));

            let _ = stream_tx_for_wait.blocking_send(LocalTerminalStreamEvent::Closed(exit_code));
        });

        // Spawn a dedicated thread for blocking PTY read operations
        std::thread::spawn(move || {
            let mut buf = [0u8; 4096];
            loop {
                match reader.read(&mut buf) {
                    Ok(0) => break,
                    Ok(n) => {
                        let text = String::from_utf8_lossy(&buf[..n]).to_string();
                        if stream_tx.blocking_send(LocalTerminalStreamEvent::Data(text)).is_err() {
                            break;
                        }
                    }
                    Err(_) => break,
                }
            }
        });
    });
}

pub fn open_pod_terminal(
    data: Option<Value>,
    writer: Arc<Mutex<WsWriter>>,
    token: String,
    manager: Arc<RwLock<KubeManager>>,
) {
    tokio::spawn(async move {
        let req = match data {
            Some(v) => match serde_json::from_value::<OpenPodTerminalRequest>(v) {
                Ok(r) => r,
                Err(e) => {
                    tracing::error!(error = %e, "Failed to parse OpenPodTerminalRequest");
                    return;
                }
            },
            None => return,
        };

        let client = {
            let mgr = manager.read().await;
            match mgr.active_client.clone() {
                Some(c) => c,
                None => {
                    let _ = Bridge::send_event(
                        &writer,
                        &token,
                        &OrbitEvent::ErrorOccurred {
                            message: "No active cluster client found for Pod terminal".to_string(),
                        },
                    ).await;
                    return;
                }
            }
        };

        let pods_api: kube::Api<k8s_openapi::api::core::v1::Pod> =
            kube::Api::namespaced(client, &req.namespace);

        let mut params = kube::api::AttachParams::default()
            .stdin(true)
            .stdout(true)
            .stderr(false)
            .tty(true);

        if let Some(ref c) = req.container {
            params = params.container(c);
        }

        // Try /bin/sh first, then /bin/bash as fallback
        let mut attached = match pods_api.exec(&req.pod, vec!["/bin/sh"], &params).await {
            Ok(proc) => proc,
            Err(_) => match pods_api.exec(&req.pod, vec!["/bin/bash"], &params).await {
                Ok(proc) => proc,
                Err(e) => {
                    let _ = Bridge::send_event(
                        &writer,
                        &token,
                        &OrbitEvent::ErrorOccurred {
                            message: format!("Failed to exec into pod {}: {}", req.pod, e),
                        },
                    ).await;
                    return;
                }
            },
        };

        let initial_size = PtySize {
            rows: req.rows.unwrap_or(24),
            cols: req.cols.unwrap_or(80),
            pixel_width: 0,
            pixel_height: 0,
        };

        let (stdin_tx, mut stdin_rx) = tokio::sync::mpsc::channel::<Vec<u8>>(100);
        let (cancel_tx, mut cancel_rx) = tokio::sync::oneshot::channel::<()>();

        {
            let mut sessions = get_sessions().write().await;
            sessions.insert(
                req.session_id.clone(),
                TerminalSession {
                    sender: TerminalSender::Pod { stdin_tx },
                    cancel_tx: Some(cancel_tx),
                    last_size: Arc::new(std::sync::Mutex::new(initial_size)),
                },
            );
        }

        // Handle writing to Pod stdin
        if let Some(mut stdin) = attached.stdin() {
            tokio::spawn(async move {
                while let Some(data) = stdin_rx.recv().await {
                    if stdin.write_all(&data).await.is_err() {
                        break;
                    }
                    let _ = stdin.flush().await;
                }
            });
        }

        let (done_tx, mut done_rx) = tokio::sync::oneshot::channel::<()>();

        // Handle reading from Pod stdout
        if let Some(mut stdout) = attached.stdout() {
            let writer_clone = writer.clone();
            let token_clone = token.clone();
            let sid = req.session_id.clone();
            tokio::spawn(async move {
                let mut buf = [0u8; 4096];
                loop {
                    match stdout.read(&mut buf).await {
                        Ok(0) => break,
                        Ok(n) => {
                            let text = String::from_utf8_lossy(&buf[..n]).to_string();
                            let _ = Bridge::send_event(
                                &writer_clone,
                                &token_clone,
                                &OrbitEvent::TerminalData {
                                    session_id: sid.clone(),
                                    data: text,
                                },
                            ).await;
                        }
                        Err(_) => break,
                    }
                }
                let _ = done_tx.send(());
            });
        }

        // Handle reading from Pod stderr
        if let Some(mut stderr) = attached.stderr() {
            let writer_clone = writer.clone();
            let token_clone = token.clone();
            let sid = req.session_id.clone();
            tokio::spawn(async move {
                let mut buf = [0u8; 4096];
                loop {
                    match stderr.read(&mut buf).await {
                        Ok(0) => break,
                        Ok(n) => {
                            let text = String::from_utf8_lossy(&buf[..n]).to_string();
                            let _ = Bridge::send_event(
                                &writer_clone,
                                &token_clone,
                                &OrbitEvent::TerminalData {
                                    session_id: sid.clone(),
                                    data: text,
                                },
                            ).await;
                        }
                        Err(_) => break,
                    }
                }
            });
        }

        // Wait for process exit or session cancellation
        let writer_clone = writer.clone();
        let token_clone = token.clone();
        let sid = req.session_id.clone();
        tokio::spawn(async move {
            tokio::select! {
                _ = &mut done_rx => {},
                _ = &mut cancel_rx => {},
            }
            let _ = Bridge::send_event(
                &writer_clone,
                &token_clone,
                &OrbitEvent::TerminalClosed {
                    session_id: sid,
                    exit_code: Some(0),
                },
            ).await;
        });
    });
}

pub fn send_terminal_data(data: Option<Value>) {
    tokio::spawn(async move {
        let req = match data {
            Some(v) => match serde_json::from_value::<TerminalDataRequest>(v) {
                Ok(r) => r,
                Err(e) => {
                    tracing::error!(error = %e, "Failed to parse TerminalDataRequest");
                    return;
                }
            },
            None => return,
        };

        let sessions = get_sessions().read().await;
        if let Some(session) = sessions.get(&req.session_id) {
            match &session.sender {
                TerminalSender::Local { writer, .. } => {
                    if let Ok(mut w) = writer.lock() {
                        let _ = w.write_all(req.data.as_bytes());
                        let _ = w.flush();
                    }
                }
                TerminalSender::Pod { stdin_tx } => {
                    let _ = stdin_tx.send(req.data.into_bytes()).await;
                }
            }
        }
    });
}

pub fn resize_terminal(data: Option<Value>) {
    tokio::spawn(async move {
        let req = match data {
            Some(v) => match serde_json::from_value::<TerminalResizeRequest>(v) {
                Ok(r) => r,
                Err(e) => {
                    tracing::error!(error = %e, "Failed to parse TerminalResizeRequest");
                    return;
                }
            },
            None => return,
        };

        let sessions = get_sessions().read().await;
        if let Some(session) = sessions.get(&req.session_id)
            && let TerminalSender::Local { master, .. } = &session.sender
        {
            if let Ok(mut last) = session.last_size.lock() {
                if last.cols == req.cols && last.rows == req.rows {
                    return; // Skip duplicate resize to prevent redundant ConPTY reflow
                }
                last.cols = req.cols;
                last.rows = req.rows;
            }

            if let Ok(m) = master.lock() {
                let _ = m.resize(PtySize {
                    rows: req.rows,
                    cols: req.cols,
                    pixel_width: 0,
                    pixel_height: 0,
                });
            }
        }
    });
}

pub fn close_terminal(data: Option<Value>) {
    tokio::spawn(async move {
        let req = match data {
            Some(v) => match serde_json::from_value::<CloseTerminalRequest>(v) {
                Ok(r) => r,
                Err(e) => {
                    tracing::error!(error = %e, "Failed to parse CloseTerminalRequest");
                    return;
                }
            },
            None => return,
        };

        let mut sessions = get_sessions().write().await;
        if let Some(mut session) = sessions.remove(&req.session_id) {
            if let Some(cancel) = session.cancel_tx.take() {
                let _ = cancel.send(());
            }
            if let TerminalSender::Local { child, .. } = &session.sender
                && let Ok(mut c) = child.lock() {
                let _ = c.kill();
            }
        }
    });
}

/// Closes all active Pod terminal sessions (e.g. on cluster switch).
pub async fn close_all_pod_terminals() {
    let mut sessions = get_sessions().write().await;
    let pod_ids: Vec<String> = sessions
        .iter()
        .filter_map(|(id, session)| {
            if matches!(session.sender, TerminalSender::Pod { .. }) {
                Some(id.clone())
            } else {
                None
            }
        })
        .collect();
    for id in pod_ids {
        if let Some(mut session) = sessions.remove(&id)
            && let Some(cancel) = session.cancel_tx.take() {
            let _ = cancel.send(());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_deserialize_open_pod_terminal_request() {
        let val = json!({
            "sessionId": "term-123",
            "namespace": "default",
            "pod": "web-pod-1",
            "container": "nginx"
        });
        let req: Result<OpenPodTerminalRequest, _> = serde_json::from_value(val);
        assert!(req.is_ok());
        let req = req.unwrap();
        assert_eq!(req.session_id, "term-123");
        assert_eq!(req.namespace, "default");
        assert_eq!(req.pod, "web-pod-1");
        assert_eq!(req.container, Some("nginx".to_string()));
        assert_eq!(req.cols, None);
        assert_eq!(req.rows, None);
    }

    #[test]
    fn test_deserialize_open_pod_terminal_request_with_dimensions() {
        let val = json!({
            "sessionId": "term-123",
            "namespace": "default",
            "pod": "web-pod-1",
            "container": "nginx",
            "cols": 120,
            "rows": 35
        });
        let req: Result<OpenPodTerminalRequest, _> = serde_json::from_value(val);
        assert!(req.is_ok());
        let req = req.unwrap();
        assert_eq!(req.session_id, "term-123");
        assert_eq!(req.cols, Some(120));
        assert_eq!(req.rows, Some(35));
    }

    #[test]
    fn test_deserialize_open_local_terminal_request() {
        let val = json!({
            "sessionId": "term-local-1",
            "shell": "powershell.exe"
        });
        let req: Result<OpenLocalTerminalRequest, _> = serde_json::from_value(val);
        assert!(req.is_ok());
        let req = req.unwrap();
        assert_eq!(req.session_id, "term-local-1");
        assert_eq!(req.shell, Some("powershell.exe".to_string()));
        assert_eq!(req.cols, None);
        assert_eq!(req.rows, None);
    }

    #[test]
    fn test_deserialize_open_local_terminal_request_with_dimensions() {
        let val = json!({
            "sessionId": "term-local-1",
            "shell": "powershell.exe",
            "cols": 100,
            "rows": 30
        });
        let req: Result<OpenLocalTerminalRequest, _> = serde_json::from_value(val);
        assert!(req.is_ok());
        let req = req.unwrap();
        assert_eq!(req.session_id, "term-local-1");
        assert_eq!(req.cols, Some(100));
        assert_eq!(req.rows, Some(30));
    }

    #[test]
    fn test_deserialize_terminal_data_request() {
        let val = json!({
            "sessionId": "term-local-1",
            "data": "ls -la\n"
        });
        let req: Result<TerminalDataRequest, _> = serde_json::from_value(val);
        assert!(req.is_ok());
        let req = req.unwrap();
        assert_eq!(req.session_id, "term-local-1");
        assert_eq!(req.data, "ls -la\n");
    }

    #[test]
    fn test_deserialize_terminal_resize_request() {
        let val = json!({
            "sessionId": "term-local-1",
            "cols": 120,
            "rows": 40
        });
        let req: Result<TerminalResizeRequest, _> = serde_json::from_value(val);
        assert!(req.is_ok());
        let req = req.unwrap();
        assert_eq!(req.session_id, "term-local-1");
        assert_eq!(req.cols, 120);
        assert_eq!(req.rows, 40);
    }

    #[test]
    fn test_deserialize_close_terminal_request() {
        let val = json!({
            "sessionId": "term-local-1"
        });
        let req: Result<CloseTerminalRequest, _> = serde_json::from_value(val);
        assert!(req.is_ok());
        let req = req.unwrap();
        assert_eq!(req.session_id, "term-local-1");
    }

    #[test]
    fn test_kube_attach_params() {
        use kube::api::AttachParams;
        let params = AttachParams::default()
            .stdin(true)
            .stdout(true)
            .stderr(false)
            .tty(true);
        assert!(params.stdin);
        assert!(params.stdout);
        assert!(!params.stderr);
        assert!(params.tty);
    }

    #[test]
    fn test_resolve_shell() {
        // When request specifies shell, request shell takes precedence
        assert_eq!(
            resolve_shell(Some("cmd.exe".to_string()), Some("pwsh.exe".to_string())),
            "cmd.exe"
        );

        // When request shell is empty/none, config shell is used
        assert_eq!(
            resolve_shell(None, Some("pwsh.exe".to_string())),
            "pwsh.exe"
        );
        assert_eq!(
            resolve_shell(Some("   ".to_string()), Some("pwsh.exe".to_string())),
            "pwsh.exe"
        );

        // When both are None/empty, fallback to default shell
        #[cfg(target_os = "windows")]
        assert_eq!(resolve_shell(None, None), detect_default_shell());
    }

    #[test]
    fn test_is_allowed_shell() {
        assert!(is_allowed_shell("pwsh"));
        assert!(is_allowed_shell("pwsh.exe"));
        assert!(is_allowed_shell("powershell.exe"));
        assert!(is_allowed_shell("cmd.exe"));
        assert!(is_allowed_shell("cmd"));
        assert!(is_allowed_shell("/bin/bash"));
        assert!(is_allowed_shell("/usr/bin/zsh"));
        assert!(is_allowed_shell("/bin/sh"));
        assert!(is_allowed_shell("fish"));
        assert!(is_allowed_shell("C:\\Windows\\System32\\cmd.exe"));
        assert!(is_allowed_shell("C:\\Program Files\\PowerShell\\7\\pwsh.exe"));

        assert!(!is_allowed_shell("curl.exe"));
        assert!(!is_allowed_shell("/usr/bin/curl"));
        assert!(!is_allowed_shell("malicious.bat"));
        assert!(!is_allowed_shell(""));
    }

    #[test]
    fn test_resolve_shell_untrusted_fallback() {
        // Disallowed shell in request falls back to config shell if valid
        assert_eq!(
            resolve_shell(Some("curl.exe".to_string()), Some("cmd.exe".to_string())),
            "cmd.exe"
        );

        // Disallowed shell in both falls back to default detected shell
        assert_eq!(
            resolve_shell(Some("curl.exe".to_string()), Some("malicious.bat".to_string())),
            detect_default_shell()
        );
    }

    #[test]
    fn test_detect_default_shell_validity() {
        let shell = detect_default_shell();
        assert!(!shell.trim().is_empty(), "Default shell must not be empty");

        #[cfg(target_os = "windows")]
        {
            let p = std::path::Path::new(&shell);
            let exists_as_file = p.is_file();
            let in_path = is_executable_in_path(&shell);
            assert!(
                exists_as_file || in_path,
                "Detected shell '{}' must either be an existing file path or resolvable in PATH",
                shell
            );
        }

        #[cfg(not(target_os = "windows"))]
        {
            let p = std::path::Path::new(&shell);
            assert!(
                p.exists() || is_executable_in_path(&shell),
                "Detected shell '{}' must exist or be in PATH",
                shell
            );
        }
    }

    #[test]
    #[cfg(target_os = "windows")]
    fn test_resolve_shell_windows_known_shells() {
        // cmd should resolve to an existing path or runnable binary
        let resolved_cmd = resolve_shell(Some("cmd".to_string()), None);
        let p = std::path::Path::new(&resolved_cmd);
        assert!(
            p.is_file() || is_executable_in_path(&resolved_cmd),
            "Resolved cmd '{}' must exist or be in PATH",
            resolved_cmd
        );
    }

    #[tokio::test]
    async fn test_local_terminal_stream_channel() {
        let (tx, mut rx) = tokio::sync::mpsc::channel::<LocalTerminalStreamEvent>(10);

        // Spawn a plain OS thread without Tokio runtime context
        let handle = std::thread::spawn(move || {
            tx.blocking_send(LocalTerminalStreamEvent::Data("hello from OS thread".to_string()))
                .expect("blocking_send must succeed from plain OS thread");
            tx.blocking_send(LocalTerminalStreamEvent::Closed(Some(0)))
                .expect("blocking_send must succeed from plain OS thread");
        });

        handle.join().unwrap();

        match rx.recv().await {
            Some(LocalTerminalStreamEvent::Data(msg)) => assert_eq!(msg, "hello from OS thread"),
            other => panic!("Expected Data event, got {:?}", other),
        }

        match rx.recv().await {
            Some(LocalTerminalStreamEvent::Closed(code)) => assert_eq!(code, Some(0)),
            other => panic!("Expected Closed event, got {:?}", other),
        }
    }

    #[test]
    fn test_pty_child_waiter_thread_sends_closed() {
        let pty_system = native_pty_system();
        let pair = pty_system
            .openpty(PtySize {
                rows: 24,
                cols: 80,
                pixel_width: 0,
                pixel_height: 0,
            })
            .expect("openpty should succeed");

        #[cfg(target_os = "windows")]
        let mut cmd = CommandBuilder::new("cmd.exe");
        #[cfg(target_os = "windows")]
        cmd.args(["/c", "exit 0"]);

        #[cfg(not(target_os = "windows"))]
        let mut cmd = CommandBuilder::new("sh");
        #[cfg(not(target_os = "windows"))]
        cmd.args(["-c", "exit 0"]);

        let child = pair.slave.spawn_command(cmd).expect("spawn child");
        let child_arc = Arc::new(std::sync::Mutex::new(child));

        let (tx, rx) = std::sync::mpsc::channel();

        let child_wait = child_arc.clone();
        let tx_wait = tx.clone();
        let waiter_handle = std::thread::spawn(move || {
            let exit_code = loop {
                match child_wait.lock() {
                    Ok(mut c) => match c.try_wait() {
                        Ok(Some(s)) => break Some(s.exit_code() as i32),
                        Ok(None) => {}
                        Err(_) => break None,
                    },
                    Err(_) => break None,
                }
                std::thread::sleep(std::time::Duration::from_millis(50));
            };
            let _ = tx_wait.send(LocalTerminalStreamEvent::Closed(exit_code));
        });

        let evt = rx
            .recv_timeout(std::time::Duration::from_secs(3))
            .expect("Must receive Closed event from waiter thread within 3 seconds");

        match evt {
            LocalTerminalStreamEvent::Closed(code) => assert_eq!(code, Some(0)),
            other => panic!("Expected Closed, got {:?}", other),
        }

        waiter_handle.join().unwrap();
    }

    #[test]
    fn test_pty_child_waiter_allows_concurrent_kill() {
        let pty_system = native_pty_system();
        let pair = pty_system
            .openpty(PtySize {
                rows: 24,
                cols: 80,
                pixel_width: 0,
                pixel_height: 0,
            })
            .expect("openpty should succeed");

        #[cfg(target_os = "windows")]
        let mut cmd = CommandBuilder::new("cmd.exe");
        #[cfg(target_os = "windows")]
        cmd.args(["/c", "ping -n 5 127.0.0.1 >nul"]);

        #[cfg(not(target_os = "windows"))]
        let mut cmd = CommandBuilder::new("sh");
        #[cfg(not(target_os = "windows"))]
        cmd.args(["-c", "sleep 5"]);

        let child = pair.slave.spawn_command(cmd).expect("spawn child");
        let child_arc = Arc::new(std::sync::Mutex::new(child));

        let (tx, rx) = std::sync::mpsc::channel();

        let child_wait = child_arc.clone();
        let tx_wait = tx.clone();
        let waiter_handle = std::thread::spawn(move || {
            let exit_code = loop {
                match child_wait.lock() {
                    Ok(mut c) => match c.try_wait() {
                        Ok(Some(s)) => break Some(s.exit_code() as i32),
                        Ok(None) => {}
                        Err(_) => break None,
                    },
                    Err(_) => break None,
                }
                std::thread::sleep(std::time::Duration::from_millis(50));
            };
            let _ = tx_wait.send(LocalTerminalStreamEvent::Closed(exit_code));
        });

        // Concurrently kill the child from another thread (simulates close_terminal)
        let child_kill = child_arc.clone();
        let killer_handle = std::thread::spawn(move || {
            std::thread::sleep(std::time::Duration::from_millis(50));
            let mut c = child_kill.lock().expect("must acquire lock to kill child");
            let _ = c.kill();
        });

        killer_handle.join().unwrap();

        let evt = rx
            .recv_timeout(std::time::Duration::from_secs(3))
            .expect("Must receive Closed event from waiter thread within 3 seconds after killing");

        match evt {
            LocalTerminalStreamEvent::Closed(_) => {}
            other => panic!("Expected Closed, got {:?}", other),
        }

        waiter_handle.join().unwrap();
    }
}

