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
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct OpenLocalTerminalRequest {
    pub session_id: String,
    pub shell: Option<String>,
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
    },
    Pod {
        stdin_tx: tokio::sync::mpsc::Sender<Vec<u8>>,
    },
}

pub struct TerminalSession {
    pub sender: TerminalSender,
    pub cancel_tx: Option<tokio::sync::oneshot::Sender<()>>,
}

type SessionMap = Arc<RwLock<HashMap<String, TerminalSession>>>;

fn get_sessions() -> &'static SessionMap {
    static SESSIONS: OnceLock<SessionMap> = OnceLock::new();
    SESSIONS.get_or_init(|| Arc::new(RwLock::new(HashMap::new())))
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

        let pty_system = native_pty_system();
        let pair = match pty_system.openpty(PtySize {
            rows: 24,
            cols: 80,
            pixel_width: 0,
            pixel_height: 0,
        }) {
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

        #[cfg(target_os = "windows")]
        let default_shell = "powershell.exe".to_string();
        #[cfg(not(target_os = "windows"))]
        let default_shell = std::env::var("SHELL").unwrap_or_else(|_| "/bin/sh".to_string());

        let shell = req.shell.unwrap_or(default_shell);
        let cmd = CommandBuilder::new(&shell);

        let mut child = match pair.slave.spawn_command(cmd) {
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
                    },
                    cancel_tx: Some(cancel_tx),
                },
            );
        }

        let writer_clone = writer.clone();
        let token_clone = token.clone();
        let session_id_clone = req.session_id.clone();

        // Spawn a dedicated thread for blocking PTY read operations
        std::thread::spawn(move || {
            let mut buf = [0u8; 4096];
            loop {
                match reader.read(&mut buf) {
                    Ok(0) => break,
                    Ok(n) => {
                        let text = String::from_utf8_lossy(&buf[..n]).to_string();
                        let w = writer_clone.clone();
                        let tok = token_clone.clone();
                        let sid = session_id_clone.clone();
                        tokio::spawn(async move {
                            let _ = Bridge::send_event(
                                &w,
                                &tok,
                                &OrbitEvent::TerminalData {
                                    session_id: sid,
                                    data: text,
                                },
                            ).await;
                        });
                    }
                    Err(_) => break,
                }
            }

            let exit_code = child.wait().ok().map(|s| s.exit_code() as i32);
            let w = writer_clone;
            let tok = token_clone;
            let sid = session_id_clone;
            tokio::spawn(async move {
                let _ = Bridge::send_event(
                    &w,
                    &tok,
                    &OrbitEvent::TerminalClosed {
                        session_id: sid,
                        exit_code,
                    },
                ).await;
            });
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
            .stderr(true)
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

        let (stdin_tx, mut stdin_rx) = tokio::sync::mpsc::channel::<Vec<u8>>(100);
        let (cancel_tx, mut cancel_rx) = tokio::sync::oneshot::channel::<()>();

        {
            let mut sessions = get_sessions().write().await;
            sessions.insert(
                req.session_id.clone(),
                TerminalSession {
                    sender: TerminalSender::Pod { stdin_tx },
                    cancel_tx: Some(cancel_tx),
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
        if let Some(session) = sessions.get(&req.session_id) {
            if let TerminalSender::Local { master, .. } = &session.sender {
                if let Ok(m) = master.lock() {
                    let _ = m.resize(PtySize {
                        rows: req.rows,
                        cols: req.cols,
                        pixel_width: 0,
                        pixel_height: 0,
                    });
                }
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
        }
    });
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
            .stderr(true)
            .tty(true);
        assert!(params.stdin);
        assert!(params.stdout);
        assert!(params.tty);
    }
}
