use std::path::PathBuf;
use std::sync::Arc;

use serde::{Deserialize, Serialize};
use tokio::sync::{mpsc, RwLock};

use crate::config::OrbitConfig;
use crate::kubernetes::manager::KubeManager;

static FILE_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct PersistedPortForward {
    pub id: String,
    pub cluster_id: String,
    pub namespace: String,
    pub kind: String,
    pub name: String,
    pub local_port: u16,
    pub remote_port: u16,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PortForwardRequest {
    pub namespace: String,
    pub kind: String,
    pub name: String,
    pub local_port: u16,
    pub remote_port: u16,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PortForwardLifecycleEvent {
    Started(PersistedPortForward),
    Stopped { id: String },
    Error { id: Option<String>, message: String },
}

pub fn format_forward_id(
    namespace: &str,
    kind: &str,
    name: &str,
    local_port: u16,
    remote_port: u16,
) -> String {
    format!("{namespace}/{kind}/{name}:{local_port}:{remote_port}")
}

#[cfg(windows)]
const CREATE_NO_WINDOW: u32 = 0x0800_0000;

#[cfg(windows)]
pub fn new_hidden_command(program: &str) -> tokio::process::Command {
    let mut cmd = tokio::process::Command::new(program);
    cmd.creation_flags(CREATE_NO_WINDOW);
    cmd
}

#[cfg(not(windows))]
pub fn new_hidden_command(program: &str) -> tokio::process::Command {
    tokio::process::Command::new(program)
}

pub fn port_forwards_file_path() -> Option<PathBuf> {
    let mut path = OrbitConfig::config_dir()?;
    path.push("port_forwards.json");
    Some(path)
}

pub fn load_persisted_port_forwards() -> Vec<PersistedPortForward> {
    let _guard = FILE_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    load_persisted_port_forwards_unlocked()
}

fn load_persisted_port_forwards_unlocked() -> Vec<PersistedPortForward> {
    let Some(path) = port_forwards_file_path() else {
        return Vec::new();
    };
    if !path.exists() {
        return Vec::new();
    }
    let content = match std::fs::read_to_string(&path) {
        Ok(c) => c,
        Err(e) => {
            tracing::warn!(path = ?path, error = %e, "Failed to read port forwards file");
            return Vec::new();
        }
    };
    serde_json::from_str(&content).unwrap_or_default()
}

pub fn save_persisted_port_forwards(forwards: &[PersistedPortForward]) -> Result<(), String> {
    let _guard = FILE_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    save_persisted_port_forwards_unlocked(forwards)
}

fn save_persisted_port_forwards_unlocked(forwards: &[PersistedPortForward]) -> Result<(), String> {
    let Some(path) = port_forwards_file_path() else {
        return Err("Could not determine config directory".to_string());
    };
    if let Some(parent) = path.parent().filter(|p| !p.exists()) {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("Failed to create config directory {:?}: {}", parent, e))?;
    }
    let content = serde_json::to_string_pretty(forwards)
        .map_err(|e| format!("Failed to serialize port forwards: {}", e))?;
    let temp_path = path.with_extension("tmp");
    std::fs::write(&temp_path, &content)
        .map_err(|e| format!("Failed to write temporary port forwards file {:?}: {}", temp_path, e))?;
    std::fs::rename(&temp_path, &path)
        .map_err(|e| format!("Failed to atomically rename port forwards file to {:?}: {}", path, e))?;
    Ok(())
}

pub fn add_persisted_port_forward(forward: PersistedPortForward) {
    let _guard = FILE_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let mut current = load_persisted_port_forwards_unlocked();
    current.retain(|f| !(f.cluster_id == forward.cluster_id && f.id == forward.id));
    current.push(forward);
    if let Err(e) = save_persisted_port_forwards_unlocked(&current) {
        tracing::error!(error = %e, "Failed to save added port forward to disk");
    }
}

pub fn remove_persisted_port_forward(id: &str) {
    let _guard = FILE_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let mut current = load_persisted_port_forwards_unlocked();
    let prev_len = current.len();
    current.retain(|f| f.id != id);
    if current.len() == prev_len {
        return;
    }
    if let Err(e) = save_persisted_port_forwards_unlocked(&current) {
        tracing::error!(error = %e, "Failed to save updated port forwards after removal");
    }
}

pub fn remove_persisted_port_forward_for_cluster(cluster_id: &str, id: &str) {
    let _guard = FILE_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let mut current = load_persisted_port_forwards_unlocked();
    let prev_len = current.len();
    current.retain(|f| !(f.cluster_id == cluster_id && f.id == id));
    if current.len() == prev_len {
        return;
    }
    if let Err(e) = save_persisted_port_forwards_unlocked(&current) {
        tracing::error!(error = %e, "Failed to save updated port forwards after removal");
    }
}

pub fn remove_persisted_port_forwards_for_cluster(cluster_id: &str) {
    let _guard = FILE_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let mut current = load_persisted_port_forwards_unlocked();
    let prev_len = current.len();
    current.retain(|f| f.cluster_id != cluster_id);
    if current.len() == prev_len {
        return;
    }
    if let Err(e) = save_persisted_port_forwards_unlocked(&current) {
        tracing::error!(error = %e, "Failed to save updated port forwards after cluster clear");
    }
}

pub fn get_persisted_for_cluster(cluster_id: &str) -> Vec<PersistedPortForward> {
    let _guard = FILE_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    load_persisted_port_forwards_unlocked()
        .into_iter()
        .filter(|f| f.cluster_id == cluster_id)
        .collect()
}

pub async fn start_port_forward_tunnel(
    manager: Arc<RwLock<KubeManager>>,
    request: PortForwardRequest,
    event_tx: mpsc::UnboundedSender<PortForwardLifecycleEvent>,
) {
    if request.name.trim().is_empty() {
        let _ = event_tx.send(PortForwardLifecycleEvent::Error {
            id: None,
            message: "Resource name is required for port forwarding".to_string(),
        });
        return;
    }

    let forward_id = format_forward_id(
        &request.namespace,
        &request.kind,
        &request.name,
        request.local_port,
        request.remote_port,
    );

    tracing::info!(
        event = "startPortForward",
        forward_id = %forward_id,
        namespace = %request.namespace,
        kind = %request.kind,
        name = %request.name,
        local_port = request.local_port,
        remote_port = request.remote_port,
        "Starting port forwarding"
    );

    let (kubeconfig_paths, active_context) = {
        let r_manager = manager.read().await;
        (
            r_manager.get_user_profile().kubeconfig_paths,
            r_manager.active_context.clone(),
        )
    };

    let previous_task = {
        let mut w_manager = manager.write().await;
        w_manager.port_forward_cancel.remove(&forward_id)
    };
    if let Some((cancel, join_handle)) = previous_task {
        let _ = cancel.send(());
        let _ = join_handle.await;
    }

    let mut cmd = new_hidden_command("kubectl");
    if !kubeconfig_paths.is_empty() {
        let separator = if cfg!(windows) { ";" } else { ":" };
        cmd.env("KUBECONFIG", kubeconfig_paths.join(separator));
    }
    if let Some(ref ctx) = active_context {
        cmd.arg("--context").arg(ctx);
    }
    cmd.arg("port-forward");
    cmd.arg("-n").arg(&request.namespace);

    let resource_target = format!("{}/{}", request.kind.to_lowercase(), request.name);
    cmd.arg(&resource_target);
    cmd.arg(format!("{}:{}", request.local_port, request.remote_port));
    cmd.stdin(std::process::Stdio::null());
    cmd.stdout(std::process::Stdio::piped());
    cmd.stderr(std::process::Stdio::piped());

    let mut child = match cmd.spawn() {
        Ok(c) => c,
        Err(e) => {
            tracing::error!(forward_id = %forward_id, error = ?e, "Failed to spawn kubectl port-forward");
            let _ = event_tx.send(PortForwardLifecycleEvent::Error {
                id: Some(forward_id),
                message: format!(
                    "Failed to start port forwarding (is kubectl installed?): {}",
                    e
                ),
            });
            return;
        }
    };

    let stderr_reader = child.stderr.take();
    let stdout_reader = child.stdout.take();

    let (started_tx, mut started_rx) = tokio::sync::oneshot::channel::<bool>();
    let started_tx = Arc::new(tokio::sync::Mutex::new(Some(started_tx)));

    if let Some(stdout) = stdout_reader {
        let started_tx_clone = started_tx.clone();
        let forward_id_stdout = forward_id.clone();
        tokio::spawn(async move {
            use tokio::io::{AsyncBufReadExt, BufReader};
            let mut reader = BufReader::new(stdout).lines();
            while let Ok(Some(line)) = reader.next_line().await {
                tracing::debug!(forward_id = %forward_id_stdout, stdout = %line, "kubectl port-forward stdout");
                if line.contains("Forwarding from") {
                    let mut lock = started_tx_clone.lock().await;
                    if let Some(tx) = lock.take() {
                        let _ = tx.send(true);
                    }
                }
            }
        });
    }

    let stderr_output = Arc::new(tokio::sync::Mutex::new(String::new()));
    let stderr_output_clone = stderr_output.clone();
    if let Some(mut stderr) = stderr_reader {
        tokio::spawn(async move {
            use tokio::io::AsyncReadExt;
            let mut buf = [0u8; 1024];
            while let Ok(n) = stderr.read(&mut buf).await {
                if n == 0 {
                    break;
                }
                if let Ok(s) = std::str::from_utf8(&buf[..n]) {
                    let mut lock = stderr_output_clone.lock().await;
                    lock.push_str(s);
                }
            }
        });
    }

    let (cancel_tx, mut cancel_rx) = tokio::sync::oneshot::channel();
    let forward_id_clone = forward_id.clone();
    let manager_clone = manager.clone();

    let mut w_manager = manager.write().await;
    let join_handle = tokio::spawn(async move {
        let mut has_started = false;

        tokio::select! {
            ready = &mut started_rx => {
                if ready.unwrap_or(false) {
                    has_started = true;
                    tracing::info!(forward_id = %forward_id_clone, "Port forwarding connected successfully");

                    let persisted = PersistedPortForward {
                        id: forward_id_clone.clone(),
                        cluster_id: active_context.clone().unwrap_or_default(),
                        namespace: request.namespace.clone(),
                        kind: request.kind.clone(),
                        name: request.name.clone(),
                        local_port: request.local_port,
                        remote_port: request.remote_port,
                    };

                    if active_context.is_some() {
                        add_persisted_port_forward(persisted.clone());
                    }

                    let _ = event_tx.send(PortForwardLifecycleEvent::Started(persisted));
                }
            }
            res = child.wait() => {
                handle_port_forward_exit(res, &stderr_output, &forward_id_clone, &event_tx).await;
                if let Some(ref ctx) = active_context {
                    remove_persisted_port_forward_for_cluster(ctx, &forward_id_clone);
                } else {
                    remove_persisted_port_forward(&forward_id_clone);
                }
                let mut w_manager = manager_clone.write().await;
                w_manager.port_forward_cancel.remove(&forward_id_clone);
                let _ = event_tx.send(PortForwardLifecycleEvent::Stopped {
                    id: forward_id_clone,
                });
                return;
            }
            _ = &mut cancel_rx => {
                kill_port_forward_child(&mut child, &forward_id_clone).await;
                let mut w_manager = manager_clone.write().await;
                w_manager.port_forward_cancel.remove(&forward_id_clone);
                return;
            }
        }

        tokio::select! {
            res = child.wait() => {
                handle_port_forward_exit(res, &stderr_output, &forward_id_clone, &event_tx).await;
                if let Some(ref ctx) = active_context {
                    remove_persisted_port_forward_for_cluster(ctx, &forward_id_clone);
                } else {
                    remove_persisted_port_forward(&forward_id_clone);
                }
            }
            _ = &mut cancel_rx => {
                kill_port_forward_child(&mut child, &forward_id_clone).await;
            }
        }

        {
            let mut w_manager = manager_clone.write().await;
            w_manager.port_forward_cancel.remove(&forward_id_clone);
        }

        if has_started {
            tracing::info!(forward_id = %forward_id_clone, "Port forwarding stopped");
            let _ = event_tx.send(PortForwardLifecycleEvent::Stopped {
                id: forward_id_clone,
            });
        }
    });

    w_manager
        .port_forward_cancel
        .insert(forward_id, (cancel_tx, join_handle));
}

async fn handle_port_forward_exit(
    res: Result<std::process::ExitStatus, std::io::Error>,
    stderr_output: &Arc<tokio::sync::Mutex<String>>,
    forward_id: &str,
    event_tx: &mpsc::UnboundedSender<PortForwardLifecycleEvent>,
) {
    match res {
        Ok(status) => {
            if !status.success() {
                let err_msg = {
                    let lock = stderr_output.lock().await;
                    lock.trim().to_string()
                };
                let message = if !err_msg.is_empty() {
                    format!("Port forwarding failed for {}: {}", forward_id, err_msg)
                } else {
                    format!(
                        "kubectl port-forward for {} exited with status {:?}",
                        forward_id, status
                    )
                };
                tracing::error!(
                    forward_id = %forward_id,
                    error = %message,
                    "kubectl port-forward process exited with error"
                );
                let _ = event_tx.send(PortForwardLifecycleEvent::Error {
                    id: Some(forward_id.to_string()),
                    message,
                });
            } else {
                tracing::info!(forward_id = %forward_id, "kubectl port-forward exited cleanly");
            }
        }
        Err(e) => {
            tracing::error!(forward_id = %forward_id, error = ?e, "kubectl port-forward wait error");
        }
    }
}

async fn kill_port_forward_child(child: &mut tokio::process::Child, forward_id: &str) {
    tracing::info!(forward_id = %forward_id, "Killing kubectl port-forward process");

    #[cfg(windows)]
    {
        if let Some(pid) = child.id() {
            let _ = new_hidden_command("taskkill")
                .args(["/F", "/T", "/PID", &pid.to_string()])
                .output()
                .await;
        }
    }

    let _ = child.kill().await;
    let _ = child.wait().await;
}

/// Probes each port in `local_ports`. For any that is already occupied,
/// attempts to kill the owning process if it is a `kubectl` process.
/// Called before restoring persisted port forwards to clear stale processes
/// from a previous ungraceful shutdown.
pub async fn kill_orphaned_port_forward_processes(local_ports: &[u16]) {
    let mut killed_any = false;
    for &port in local_ports {
        if std::net::TcpListener::bind(("127.0.0.1", port)).is_ok() {
            continue;
        }
        tracing::info!(
            port = port,
            "Local port is occupied; attempting to kill orphaned kubectl process"
        );
        if kill_process_on_port(port).await {
            killed_any = true;
        }
    }
    if killed_any {
        tokio::time::sleep(std::time::Duration::from_millis(200)).await;
    }
}

#[cfg(windows)]
async fn kill_process_on_port(port: u16) -> bool {
    let Ok(output) = new_hidden_command("netstat")
        .args(["-ano"])
        .output()
        .await
    else {
        return false;
    };
    let mut killed = false;
    let stdout = String::from_utf8_lossy(&output.stdout);
    for line in stdout.lines() {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 5 {
            continue;
        }
        if !parts[3].eq_ignore_ascii_case("LISTENING") {
            continue;
        }
        let local_addr = parts[1];
        let Some(addr_port_str) = local_addr.rsplit(':').next() else {
            continue;
        };
        let Ok(addr_port) = addr_port_str.parse::<u16>() else {
            continue;
        };
        if addr_port != port {
            continue;
        }
        let Ok(pid) = parts[4].parse::<u32>() else {
            continue;
        };
        let Ok(check) = new_hidden_command("tasklist")
            .args(["/FI", &format!("PID eq {}", pid), "/NH", "/FO", "CSV"])
            .output()
            .await
        else {
            continue;
        };
        let check_out = String::from_utf8_lossy(&check.stdout).to_ascii_lowercase();
        if check_out.contains("kubectl") {
            tracing::info!(
                pid = pid,
                port = port,
                "Killing orphaned kubectl port-forward process"
            );
            let _ = new_hidden_command("taskkill")
                .args(["/F", "/T", "/PID", &pid.to_string()])
                .output()
                .await;
            killed = true;
        }
    }
    killed
}

#[cfg(not(windows))]
async fn kill_process_on_port(port: u16) -> bool {
    let Ok(output) = tokio::process::Command::new("lsof")
        .args(["-ti", &format!("tcp:{}", port)])
        .output()
        .await
    else {
        return false;
    };
    let mut killed = false;
    let stdout = String::from_utf8_lossy(&output.stdout);
    for pid_str in stdout.split_whitespace() {
        let Ok(pid) = pid_str.trim().parse::<u32>() else {
            continue;
        };
        let Ok(check) = tokio::process::Command::new("ps")
            .args(["-p", &pid.to_string(), "-o", "comm="])
            .output()
            .await
        else {
            continue;
        };
        let comm = String::from_utf8_lossy(&check.stdout)
            .trim()
            .to_ascii_lowercase();
        if comm.contains("kubectl") {
            tracing::info!(
                pid = pid,
                port = port,
                "Killing orphaned kubectl port-forward process"
            );
            let _ = tokio::process::Command::new("kill")
                .args([&pid.to_string()])
                .output()
                .await;
            tokio::time::sleep(std::time::Duration::from_millis(300)).await;
            let _ = tokio::process::Command::new("kill")
                .args(["-9", &pid.to_string()])
                .output()
                .await;
            killed = true;
        }
    }
    killed
}

pub async fn stop_port_forward_tunnel(
    manager: &Arc<RwLock<KubeManager>>,
    cluster_id: Option<&str>,
    id: &str,
) {
    tracing::info!(forward_id = %id, "Stopping port forward");
    if let Some(ctx) = cluster_id {
        remove_persisted_port_forward_for_cluster(ctx, id);
    } else {
        remove_persisted_port_forward(id);
    }
    let task = {
        let mut w_manager = manager.write().await;
        w_manager.port_forward_cancel.remove(id)
    };
    if let Some((cancel, join_handle)) = task {
        let _ = cancel.send(());
        let _ = join_handle.await;
    }
}

pub async fn stop_cluster_port_forwards(manager: &Arc<RwLock<KubeManager>>, cluster_id: &str) {
    tracing::info!(cluster_id = %cluster_id, "Stopping all port forwards for cluster");
    remove_persisted_port_forwards_for_cluster(cluster_id);
    stop_all_active_port_forwards(manager).await;
}

pub async fn stop_all_active_port_forwards(manager: &Arc<RwLock<KubeManager>>) {
    tracing::info!("Stopping all active port forwards");
    let mut tasks_to_cancel = Vec::new();
    {
        let mut w_manager = manager.write().await;
        for (_id, task) in w_manager.port_forward_cancel.drain() {
            tasks_to_cancel.push(task);
        }
    }
    for (cancel, join_handle) in tasks_to_cancel {
        let _ = cancel.send(());
        let _ = join_handle.await;
    }
}

pub async fn restore_cluster_port_forwards(
    manager: Arc<RwLock<KubeManager>>,
    cluster_id: String,
    event_tx: mpsc::UnboundedSender<PortForwardLifecycleEvent>,
) {
    let persisted = get_persisted_for_cluster(&cluster_id);
    if persisted.is_empty() {
        return;
    }

    let pending_forwards: Vec<PersistedPortForward> = {
        let r_manager = manager.read().await;
        persisted
            .into_iter()
            .filter(|f| !r_manager.port_forward_cancel.contains_key(&f.id))
            .collect()
    };
    if pending_forwards.is_empty() {
        return;
    }

    tracing::info!(
        cluster_id = %cluster_id,
        count = pending_forwards.len(),
        "Restoring persisted port forwards for cluster"
    );

    let mut local_ports: Vec<u16> = pending_forwards.iter().map(|f| f.local_port).collect();
    local_ports.sort_unstable();
    local_ports.dedup();
    kill_orphaned_port_forward_processes(&local_ports).await;

    for forward in pending_forwards {
        let request = PortForwardRequest {
            namespace: forward.namespace,
            kind: forward.kind,
            name: forward.name,
            local_port: forward.local_port,
            remote_port: forward.remote_port,
        };
        let manager_clone = manager.clone();
        let event_tx_clone = event_tx.clone();
        tokio::spawn(async move {
            start_port_forward_tunnel(manager_clone, request, event_tx_clone).await;
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::TestConfigDirGuard;

    #[test]
    fn test_format_forward_id_canonical_format() {
        assert_eq!(
            format_forward_id("default", "Deployment", "web", 8080, 80),
            "default/Deployment/web:8080:80"
        );
    }

    #[test]
    fn test_persisted_port_forward_camel_case_roundtrip() {
        let pf = PersistedPortForward {
            id: "default/Deployment/web:8080:80".to_string(),
            cluster_id: "minikube".to_string(),
            namespace: "default".to_string(),
            kind: "Deployment".to_string(),
            name: "web".to_string(),
            local_port: 8080,
            remote_port: 80,
        };

        let Ok(serialized) = serde_json::to_string(&pf) else {
            panic!("failed to serialize PersistedPortForward");
        };
        assert!(serialized.contains("\"clusterId\":\"minikube\""));
        assert!(serialized.contains("\"localPort\":8080"));
        assert!(serialized.contains("\"remotePort\":80"));

        let Ok(deserialized) = serde_json::from_str::<PersistedPortForward>(&serialized) else {
            panic!("failed to deserialize PersistedPortForward");
        };
        assert_eq!(deserialized, pf);
    }

    #[test]
    fn test_persistence_add_get_remove_and_clear_by_cluster() {
        let _guard = TestConfigDirGuard::new("port_forward_persistence");

        let pf_a1 = PersistedPortForward {
            id: format_forward_id("default", "Service", "api", 8080, 80),
            cluster_id: "cluster-a".to_string(),
            namespace: "default".to_string(),
            kind: "Service".to_string(),
            name: "api".to_string(),
            local_port: 8080,
            remote_port: 80,
        };
        let pf_a2 = PersistedPortForward {
            id: format_forward_id("kube-system", "Pod", "coredns", 9153, 9153),
            cluster_id: "cluster-a".to_string(),
            namespace: "kube-system".to_string(),
            kind: "Pod".to_string(),
            name: "coredns".to_string(),
            local_port: 9153,
            remote_port: 9153,
        };
        let pf_b1 = PersistedPortForward {
            id: format_forward_id("default", "Deployment", "web", 3000, 3000),
            cluster_id: "cluster-b".to_string(),
            namespace: "default".to_string(),
            kind: "Deployment".to_string(),
            name: "web".to_string(),
            local_port: 3000,
            remote_port: 3000,
        };

        add_persisted_port_forward(pf_a1.clone());
        add_persisted_port_forward(pf_a1.clone()); // deduplicate by id
        add_persisted_port_forward(pf_a2.clone());
        add_persisted_port_forward(pf_b1.clone());

        let cluster_a = get_persisted_for_cluster("cluster-a");
        assert_eq!(cluster_a.len(), 2);
        assert_eq!(get_persisted_for_cluster("cluster-b").len(), 1);

        remove_persisted_port_forward(&pf_a1.id);
        let cluster_a_after_remove = get_persisted_for_cluster("cluster-a");
        assert_eq!(cluster_a_after_remove, vec![pf_a2]);

        remove_persisted_port_forwards_for_cluster("cluster-a");
        assert!(get_persisted_for_cluster("cluster-a").is_empty());
        assert_eq!(get_persisted_for_cluster("cluster-b"), vec![pf_b1]);
    }

    #[tokio::test]
    async fn test_kill_orphaned_skips_free_ports() {
        let Ok(listener) = std::net::TcpListener::bind(("127.0.0.1", 0)) else {
            panic!("failed to bind ephemeral port");
        };
        let Ok(addr) = listener.local_addr() else {
            panic!("failed to get local addr");
        };
        let port = addr.port();
        drop(listener);

        kill_orphaned_port_forward_processes(&[port]).await;
    }

    #[tokio::test]
    async fn test_kill_orphaned_does_not_kill_non_kubectl_process() {
        let Ok(listener) = std::net::TcpListener::bind(("127.0.0.1", 0)) else {
            panic!("failed to bind ephemeral port");
        };
        let Ok(addr) = listener.local_addr() else {
            panic!("failed to get local addr");
        };
        let port = addr.port();

        kill_orphaned_port_forward_processes(&[port]).await;

        assert!(
            std::net::TcpListener::bind(("127.0.0.1", port)).is_err(),
            "Non-kubectl process should NOT have been killed"
        );
        drop(listener);
    }

    #[tokio::test]
    async fn test_stop_all_active_port_forwards_drains_manager_map() {
        let manager = Arc::new(RwLock::new(KubeManager::new_empty(OrbitConfig::default())));
        let (tx, mut rx) = tokio::sync::oneshot::channel::<()>();
        let handle = tokio::spawn(async move {
            let _ = (&mut rx).await;
        });

        {
            let mut w_manager = manager.write().await;
            w_manager
                .port_forward_cancel
                .insert("default/Pod/demo:8080:80".to_string(), (tx, handle));
        }

        stop_all_active_port_forwards(&manager).await;

        let r_manager = manager.read().await;
        assert!(r_manager.port_forward_cancel.is_empty());
    }

    #[test]
    fn test_persistence_scopes_by_cluster_id_and_forward_id() {
        let _guard = TestConfigDirGuard::new("port_forward_cluster_scoping");
        let shared_id = format_forward_id("default", "Service", "api", 8080, 80);
        let pf_a = PersistedPortForward {
            id: shared_id.clone(),
            cluster_id: "cluster-a".to_string(),
            namespace: "default".to_string(),
            kind: "Service".to_string(),
            name: "api".to_string(),
            local_port: 8080,
            remote_port: 80,
        };
        let pf_b = PersistedPortForward {
            cluster_id: "cluster-b".to_string(),
            ..pf_a.clone()
        };

        add_persisted_port_forward(pf_a.clone());
        add_persisted_port_forward(pf_b.clone());
        assert_eq!(get_persisted_for_cluster("cluster-a"), vec![pf_a]);
        assert_eq!(get_persisted_for_cluster("cluster-b"), vec![pf_b.clone()]);

        remove_persisted_port_forward_for_cluster("cluster-a", &shared_id);
        assert!(get_persisted_for_cluster("cluster-a").is_empty());
        assert_eq!(get_persisted_for_cluster("cluster-b"), vec![pf_b]);
    }

    #[tokio::test]
    async fn test_restore_cluster_port_forwards_skips_already_active_tunnels() {
        let _guard = TestConfigDirGuard::new("port_forward_restore_skip_active");
        let Ok(listener) = std::net::TcpListener::bind(("127.0.0.1", 0)) else {
            panic!("failed to bind ephemeral port");
        };
        let Ok(addr) = listener.local_addr() else {
            panic!("failed to get local addr");
        };
        let port = addr.port();
        let pf = PersistedPortForward {
            id: format_forward_id("default", "Service", "api", port, 80),
            cluster_id: "cluster-a".to_string(),
            namespace: "default".to_string(),
            kind: "Service".to_string(),
            name: "api".to_string(),
            local_port: port,
            remote_port: 80,
        };
        add_persisted_port_forward(pf.clone());

        let manager = Arc::new(RwLock::new(KubeManager::new_empty(OrbitConfig::default())));
        let (tx, mut rx) = tokio::sync::oneshot::channel::<()>();
        let handle = tokio::spawn(async move {
            let _ = (&mut rx).await;
        });
        {
            let mut w_manager = manager.write().await;
            w_manager
                .port_forward_cancel
                .insert(pf.id.clone(), (tx, handle));
        }

        let (event_tx, mut event_rx) = mpsc::unbounded_channel();
        restore_cluster_port_forwards(manager.clone(), "cluster-a".to_string(), event_tx).await;

        assert!(event_rx.try_recv().is_err());
        assert!(
            std::net::TcpListener::bind(("127.0.0.1", port)).is_err(),
            "Active listener port should remain untouched"
        );
        stop_all_active_port_forwards(&manager).await;
        drop(listener);
    }
}
