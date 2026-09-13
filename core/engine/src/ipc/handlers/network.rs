use std::sync::Arc;
use serde_json::Value;
use tokio::sync::{Mutex, RwLock};
use crate::ipc::bridge::{Bridge, WsWriter};
use crate::ipc::events::OrbitEvent;
use crate::kubernetes;
use crate::kubernetes::manager::KubeManager;
use crate::kubernetes::port_forward::{
    add_persisted_port_forward, get_persisted_for_cluster, remove_persisted_port_forward,
    remove_persisted_port_forwards_for_cluster, PersistedPortForward,
};
use super::utils::get_string;

pub fn get_services(
    data: Option<Value>,
    writer: Arc<Mutex<WsWriter>>,
    token: String,
    manager: Arc<RwLock<KubeManager>>,
) {
    tokio::spawn(async move {
        let namespace = get_string(&data, "namespace");
        tracing::debug!(namespace = ?namespace, "Fetching services");

        let client = {
            let r_manager = manager.read().await;
            r_manager.active_client.clone()
        };
        if let Some(ref client) = client {
            match kubernetes::list_services(client, namespace.clone()).await {
                Ok(services) => {
                    let _ = Bridge::send_event(&writer, &token, &OrbitEvent::ServicesUpdated { services }).await;
                }
                Err(e) => {
                    tracing::error!(namespace = ?namespace, error = %e, "Failed to list services");
                    let _ = Bridge::send_event(
                        &writer,
                        &token,
                        &OrbitEvent::ErrorOccurred {
                            message: format!("Failed to list services: {}", crate::kubernetes::format_error(&e)),
                        },
                    ).await;
                }
            }
        }
    });
}

pub fn get_ingresses(
    data: Option<Value>,
    writer: Arc<Mutex<WsWriter>>,
    token: String,
    manager: Arc<RwLock<KubeManager>>,
) {
    tokio::spawn(async move {
        let namespace = get_string(&data, "namespace");
        tracing::debug!(namespace = ?namespace, "Fetching ingresses");

        let client = {
            let r_manager = manager.read().await;
            r_manager.active_client.clone()
        };
        if let Some(ref client) = client {
            match kubernetes::list_ingresses(client, namespace.clone()).await {
                Ok(ingresses) => {
                    let _ = Bridge::send_event(&writer, &token, &OrbitEvent::IngressesUpdated { ingresses }).await;
                }
                Err(e) => {
                    tracing::error!(namespace = ?namespace, error = %e, "Failed to list ingresses");
                    let _ = Bridge::send_event(
                        &writer,
                        &token,
                        &OrbitEvent::ErrorOccurred {
                            message: format!("Failed to list ingresses: {}", crate::kubernetes::format_error(&e)),
                        },
                    ).await;
                }
            }
        }
    });
}

pub fn clone_ingress(
    data: Option<Value>,
    writer: Arc<Mutex<WsWriter>>,
    token: String,
    manager: Arc<RwLock<KubeManager>>,
) {
    tokio::spawn(async move {
        let source_namespace = get_string(&data, "sourceNamespace")
            .unwrap_or_else(|| "default".to_string());
        let source_name = get_string(&data, "sourceName").unwrap_or_default();
        let new_name = get_string(&data, "newName").unwrap_or_default();
        let new_namespace = get_string(&data, "newNamespace")
            .unwrap_or_else(|| source_namespace.clone());
        let new_hosts: Vec<String> = data.as_ref()
            .and_then(|d| d.get("newHosts"))
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(|s| s.to_string()))
                    .collect()
            })
            .unwrap_or_default();

        tracing::info!(
            event = "cloneIngress",
            source_namespace = %source_namespace,
            source_name = %source_name,
            new_namespace = %new_namespace,
            new_name = %new_name,
            "Kubernetes operation started"
        );

        let client = {
            let r_manager = manager.read().await;
            r_manager.active_client.clone()
        };

        if let Some(ref client) = client {
            match crate::kubernetes::clone_ingress(
                client,
                &source_namespace,
                &source_name,
                &new_name,
                &new_namespace,
                new_hosts,
            )
            .await
            {
                Ok(()) => {
                    tracing::info!(
                        event = "cloneIngress",
                        new_namespace = %new_namespace,
                        new_name = %new_name,
                        "Kubernetes operation completed"
                    );
                    let _ = Bridge::send_event(
                        &writer,
                        &token,
                        &OrbitEvent::CommandSucceeded {
                            message: format!("Cloned Ingress {} as {}", source_name, new_name),
                        },
                    )
                    .await;
                }
                Err(e) => {
                    tracing::error!(
                        event = "cloneIngress",
                        source_namespace = %source_namespace,
                        source_name = %source_name,
                        error = %e,
                        "Kubernetes operation failed"
                    );
                    let _ = Bridge::send_event(
                        &writer,
                        &token,
                        &OrbitEvent::ErrorOccurred {
                            message: format!("Failed to clone Ingress: {}", crate::kubernetes::format_error(&e)),
                        },
                    )
                    .await;
                }
            }
        }
    });
}

pub fn start_port_forward(
    data: Option<Value>,
    writer: Arc<Mutex<WsWriter>>,
    token: String,
    manager: Arc<RwLock<KubeManager>>,
) {
    tokio::spawn(async move {
        let namespace = get_string(&data, "namespace").unwrap_or_else(|| "default".to_string());
        let kind = get_string(&data, "kind").unwrap_or_else(|| "Deployment".to_string());
        let name = get_string(&data, "name").unwrap_or_default();
        let local_port = super::utils::get_i64(&data, "localPort").unwrap_or(8080) as u16;
        let remote_port = super::utils::get_i64(&data, "remotePort").unwrap_or(8080) as u16;

        if name.is_empty() {
            let _ = Bridge::send_event(
                &writer,
                &token,
                &OrbitEvent::ErrorOccurred {
                    message: "Resource name is required for port forwarding".to_string(),
                },
            )
            .await;
            return;
        }

        let forward_id = format!("{}/{}/{}:{}:{}", namespace, kind, name, local_port, remote_port);
        tracing::info!(
            event = "startPortForward",
            forward_id = %forward_id,
            namespace = %namespace,
            kind = %kind,
            name = %name,
            local_port = local_port,
            remote_port = remote_port,
            "Starting port forwarding"
        );

        let (kubeconfig_paths, active_context) = {
            let r_manager = manager.read().await;
            (
                r_manager.get_user_profile().kubeconfig_paths,
                r_manager.active_context.clone(),
            )
        };

        // If already forwarding this exact target and ports, cancel previous first and await it
        let previous_task = {
            let mut w_manager = manager.write().await;
            w_manager.port_forward_cancel.remove(&forward_id)
        };
        if let Some((cancel, join_handle)) = previous_task {
            let _ = cancel.send(());
            let _ = join_handle.await;
        }

        let mut cmd = tokio::process::Command::new("kubectl");
        if !kubeconfig_paths.is_empty() {
            let separator = if cfg!(windows) { ";" } else { ":" };
            cmd.env("KUBECONFIG", kubeconfig_paths.join(separator));
        }
        if let Some(ref ctx) = active_context {
            cmd.arg("--context").arg(ctx);
        }
        cmd.arg("port-forward");
        cmd.arg("-n").arg(&namespace);

        let resource_target = format!("{}/{}", kind.to_lowercase(), name);
        cmd.arg(&resource_target);
        cmd.arg(format!("{}:{}", local_port, remote_port));
        cmd.stdin(std::process::Stdio::null());
        cmd.stdout(std::process::Stdio::piped());
        cmd.stderr(std::process::Stdio::piped());

        // Spawn child process
        let mut child = match cmd.spawn() {
            Ok(c) => c,
            Err(e) => {
                tracing::error!(forward_id = %forward_id, error = ?e, "Failed to spawn kubectl port-forward");
                let _ = Bridge::send_event(
                    &writer,
                    &token,
                    &OrbitEvent::ErrorOccurred {
                        message: format!("Failed to start port forwarding (is kubectl installed?): {}", e),
                    },
                )
                .await;
                return;
            }
        };

        let stderr_reader = child.stderr.take();
        let stdout_reader = child.stdout.take();

        let (started_tx, mut started_rx) = tokio::sync::oneshot::channel::<bool>();
        let started_tx = Arc::new(tokio::sync::Mutex::new(Some(started_tx)));

        // Spawn background task to consume stdout and detect when port forwarding is listening
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

        // Spawn background task to capture stderr
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
        let writer_clone = writer.clone();
        let token_clone = token.clone();
        let forward_id_clone = forward_id.clone();
        let manager_clone = manager.clone();
        let namespace_clone = namespace.clone();
        let kind_clone = kind.clone();
        let name_clone = name.clone();
        let active_context_for_persist = active_context.clone();

        {
            let mut w_manager = manager.write().await;

            let join_handle = tokio::spawn(async move {
                let mut has_started = false;

                tokio::select! {
                    ready = &mut started_rx => {
                        if ready.unwrap_or(false) {
                            has_started = true;
                            tracing::info!(forward_id = %forward_id_clone, "Port forwarding connected successfully");

                            if let Some(ref ctx) = active_context_for_persist {
                                add_persisted_port_forward(PersistedPortForward {
                                    id: forward_id_clone.clone(),
                                    cluster_id: ctx.clone(),
                                    namespace: namespace_clone.clone(),
                                    kind: kind_clone.clone(),
                                    name: name_clone.clone(),
                                    local_port,
                                    remote_port,
                                });
                            }

                            let _ = Bridge::send_event(
                                &writer_clone,
                                &token_clone,
                                &OrbitEvent::PortForwardStarted {
                                    id: forward_id_clone.clone(),
                                    namespace: namespace_clone,
                                    kind: kind_clone,
                                    name: name_clone,
                                    local_port,
                                    remote_port,
                                },
                            )
                            .await;
                        }
                    }
                    res = child.wait() => {
                        handle_port_forward_exit(res, &stderr_output, &forward_id_clone, &writer_clone, &token_clone).await;
                        remove_persisted_port_forward(&forward_id_clone);
                        let mut w_manager = manager_clone.write().await;
                        w_manager.port_forward_cancel.remove(&forward_id_clone);
                        let _ = Bridge::send_event(
                            &writer_clone,
                            &token_clone,
                            &OrbitEvent::PortForwardStopped {
                                id: forward_id_clone,
                            },
                        )
                        .await;
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
                        handle_port_forward_exit(res, &stderr_output, &forward_id_clone, &writer_clone, &token_clone).await;
                    }
                    _ = &mut cancel_rx => {
                        kill_port_forward_child(&mut child, &forward_id_clone).await;
                    }
                }

                // Remove from manager if still present
                {
                    let mut w_manager = manager_clone.write().await;
                    w_manager.port_forward_cancel.remove(&forward_id_clone);
                }

                if has_started {
                    // Notify UI that port forwarding has stopped
                    tracing::info!(forward_id = %forward_id_clone, "Port forwarding stopped");
                    let _ = Bridge::send_event(
                        &writer_clone,
                        &token_clone,
                        &OrbitEvent::PortForwardStopped {
                            id: forward_id_clone,
                        },
                    )
                    .await;
                }
            });

            w_manager.port_forward_cancel.insert(forward_id.clone(), (cancel_tx, join_handle));
        }
    });
}

async fn handle_port_forward_exit(
    res: Result<std::process::ExitStatus, std::io::Error>,
    stderr_output: &Arc<tokio::sync::Mutex<String>>,
    forward_id: &str,
    writer: &Arc<Mutex<WsWriter>>,
    token: &str,
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
                    format!("kubectl port-forward for {} exited with status {:?}", forward_id, status)
                };
                tracing::error!(forward_id = %forward_id, error = %message, "kubectl port-forward process exited with error");
                let _ = Bridge::send_event(
                    writer,
                    token,
                    &OrbitEvent::ErrorOccurred { message },
                ).await;
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
            let _ = tokio::process::Command::new("taskkill")
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
async fn kill_orphaned_port_forward_processes(local_ports: &[u16]) {
    let mut killed_any = false;
    for &port in local_ports {
        // Blocking bind probe — acceptable here because this runs once at startup
        // and the number of persisted ports is always small in practice.
        if std::net::TcpListener::bind(("127.0.0.1", port)).is_ok() {
            continue;
        }
        tracing::info!(port = port, "Local port is occupied; attempting to kill orphaned kubectl process");
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
    // netstat -ano lists all TCP connections with their PIDs.
    let Ok(output) = tokio::process::Command::new("netstat")
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
        // rsplit handles both IPv4 (0.0.0.0:PORT) and IPv6 (:::PORT) address formats.
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
        // Confirm the process is kubectl before killing.
        let Ok(check) = tokio::process::Command::new("tasklist")
            .args(["/FI", &format!("PID eq {}", pid), "/NH", "/FO", "CSV"])
            .output()
            .await
        else {
            continue;
        };
        let check_out = String::from_utf8_lossy(&check.stdout).to_ascii_lowercase();
        if check_out.contains("kubectl") {
            tracing::info!(pid = pid, port = port, "Killing orphaned kubectl port-forward process");
            let _ = tokio::process::Command::new("taskkill")
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
    // lsof -ti tcp:<port> returns PIDs listening on the port.
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
        // Confirm the process is kubectl before killing.
        let Ok(check) = tokio::process::Command::new("ps")
            .args(["-p", &pid.to_string(), "-o", "comm="])
            .output()
            .await
        else {
            continue;
        };
        let comm = String::from_utf8_lossy(&check.stdout).trim().to_ascii_lowercase();
        if comm.contains("kubectl") {
            tracing::info!(pid = pid, port = port, "Killing orphaned kubectl port-forward process");
            // Send SIGTERM first to allow kubectl to clean up connections gracefully.
            let _ = tokio::process::Command::new("kill")
                .args([&pid.to_string()])
                .output()
                .await;
            tokio::time::sleep(std::time::Duration::from_millis(300)).await;
            // Escalate to SIGKILL in case SIGTERM was ignored.
            let _ = tokio::process::Command::new("kill")
                .args(["-9", &pid.to_string()])
                .output()
                .await;
            killed = true;
        }
    }
    killed
}

pub fn stop_port_forward(
    data: Option<Value>,
    writer: Arc<Mutex<WsWriter>>,
    token: String,
    manager: Arc<RwLock<KubeManager>>,
) {
    tokio::spawn(async move {
        let forward_id = get_string(&data, "id");
        tracing::info!(forward_id = ?forward_id, "Stopping port forward");

        if let Some(ref id) = forward_id {
            remove_persisted_port_forward(id);
        } else {
            let active_context = {
                let r_manager = manager.read().await;
                r_manager.active_context.clone()
            };
            if let Some(ref ctx) = active_context {
                remove_persisted_port_forwards_for_cluster(ctx);
            }
        }

        let mut tasks_to_cancel = Vec::new();

        {
            let mut w_manager = manager.write().await;
            if let Some(ref id) = forward_id {
                if let Some(task) = w_manager.port_forward_cancel.remove(id) {
                    tasks_to_cancel.push(task);
                }
            } else {
                // Stop all port forwards
                for (_id, task) in w_manager.port_forward_cancel.drain() {
                    tasks_to_cancel.push(task);
                }
            }
        }

        for (cancel, join_handle) in tasks_to_cancel {
            let _ = cancel.send(());
            let _ = join_handle.await;
        }

        if let Some(id) = forward_id {
            let _ = Bridge::send_event(
                &writer,
                &token,
                &OrbitEvent::PortForwardStopped { id },
            )
            .await;
        } else {
            let _ = Bridge::send_event(
                &writer,
                &token,
                &OrbitEvent::PortForwardsUpdated {
                    port_forwards: Vec::new(),
                },
            )
            .await;
        }
    });
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

pub fn get_port_forwards(
    writer: Arc<Mutex<WsWriter>>,
    token: String,
    manager: Arc<RwLock<KubeManager>>,
) {
    tokio::spawn(async move {
        let active_context = {
            let r_manager = manager.read().await;
            r_manager.active_context.clone()
        };

        let forwards = if let Some(ref ctx) = active_context {
            restore_cluster_port_forwards(
                writer.clone(),
                token.clone(),
                manager.clone(),
                ctx.clone(),
            );
            get_persisted_for_cluster(ctx)
        } else {
            Vec::new()
        };

        let _ = Bridge::send_event(
            &writer,
            &token,
            &OrbitEvent::PortForwardsUpdated {
                port_forwards: forwards,
            },
        )
        .await;
    });
}

pub fn restore_cluster_port_forwards(
    writer: Arc<Mutex<WsWriter>>,
    token: String,
    manager: Arc<RwLock<KubeManager>>,
    cluster_id: String,
) {
    tokio::spawn(async move {
        let persisted = get_persisted_for_cluster(&cluster_id);
        if persisted.is_empty() {
            return;
        }
        tracing::info!(cluster_id = %cluster_id, count = persisted.len(), "Restoring persisted port forwards for cluster");

        // Before restoring, kill any orphaned kubectl processes occupying the
        // persisted local ports (e.g. left behind by an ungraceful shutdown).
        let mut local_ports: Vec<u16> = persisted.iter().map(|f| f.local_port).collect();
        local_ports.sort_unstable();
        local_ports.dedup();
        kill_orphaned_port_forward_processes(&local_ports).await;

        for forward in persisted {
            let is_already_running = {
                let r_manager = manager.read().await;
                r_manager.port_forward_cancel.contains_key(&forward.id)
            };

            if !is_already_running {
                let data = serde_json::json!({
                    "namespace": forward.namespace,
                    "kind": forward.kind,
                    "name": forward.name,
                    "localPort": forward.local_port,
                    "remotePort": forward.remote_port,
                });
                start_port_forward(Some(data), writer.clone(), token.clone(), manager.clone());
            }
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_kill_orphaned_skips_free_ports() {
        let listener = std::net::TcpListener::bind(("127.0.0.1", 0)).unwrap();
        let port = listener.local_addr().unwrap().port();
        drop(listener);

        kill_orphaned_port_forward_processes(&[port]).await;
    }

    #[tokio::test]
    async fn test_kill_orphaned_does_not_kill_non_kubectl_process() {
        let listener = std::net::TcpListener::bind(("127.0.0.1", 0)).unwrap();
        let port = listener.local_addr().unwrap().port();

        kill_orphaned_port_forward_processes(&[port]).await;

        // The listener (a non-kubectl process) must still hold the port — rebinding must fail.
        assert!(
            std::net::TcpListener::bind(("127.0.0.1", port)).is_err(),
            "Non-kubectl process should NOT have been killed"
        );
        drop(listener);
    }
}

