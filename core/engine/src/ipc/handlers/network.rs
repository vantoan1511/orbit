use std::sync::Arc;
use serde_json::Value;
use tokio::sync::{Mutex, RwLock};
use crate::ipc::bridge::{Bridge, WsWriter};
use crate::ipc::events::OrbitEvent;
use crate::kubernetes;
use crate::kubernetes::manager::KubeManager;
use super::utils::get_string;

pub fn get_services(
    data: Option<Value>,
    writer: Arc<Mutex<WsWriter>>,
    token: String,
    manager: Arc<RwLock<KubeManager>>,
) {
    tokio::spawn(async move {
        let namespace = get_string(&data, "namespace");

        let client = {
            let r_manager = manager.read().await;
            r_manager.active_client.clone()
        };
        if let Some(ref client) = client {
            match kubernetes::list_services(client, namespace).await {
                Ok(services) => {
                    let _ = Bridge::send_event(&writer, &token, &OrbitEvent::ServicesUpdated { services }).await;
                }
                Err(e) => {
                    log::error!("Error listing services: {:?}", e);
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

        let client = {
            let r_manager = manager.read().await;
            r_manager.active_client.clone()
        };
        if let Some(ref client) = client {
            match kubernetes::list_ingresses(client, namespace).await {
                Ok(ingresses) => {
                    let _ = Bridge::send_event(&writer, &token, &OrbitEvent::IngressesUpdated { ingresses }).await;
                }
                Err(e) => {
                    log::error!("Error listing ingresses: {:?}", e);
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
                    log::error!("Error cloning Ingress {}: {:?}", source_name, e);
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
                log::error!("Failed to spawn kubectl port-forward: {:?}", e);
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
            tokio::spawn(async move {
                use tokio::io::{AsyncBufReadExt, BufReader};
                let mut reader = BufReader::new(stdout).lines();
                while let Ok(Some(line)) = reader.next_line().await {
                    log::debug!("kubectl port-forward stdout: {}", line);
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

        {
            let mut w_manager = manager.write().await;

            let join_handle = tokio::spawn(async move {
                let mut has_started = false;

                tokio::select! {
                    ready = &mut started_rx => {
                        if ready.unwrap_or(false) {
                            has_started = true;
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
                        let mut w_manager = manager_clone.write().await;
                        w_manager.port_forward_cancel.remove(&forward_id_clone);
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
                log::error!("{}", message);
                let _ = Bridge::send_event(
                    writer,
                    token,
                    &OrbitEvent::ErrorOccurred { message },
                ).await;
            } else {
                log::info!("kubectl port-forward for {} exited cleanly", forward_id);
            }
        }
        Err(e) => {
            log::error!("kubectl port-forward for {} wait error: {:?}", forward_id, e);
        }
    }
}

async fn kill_port_forward_child(child: &mut tokio::process::Child, forward_id: &str) {
    log::info!("Killing kubectl port-forward for {}", forward_id);

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

pub fn stop_port_forward(
    data: Option<Value>,
    manager: Arc<RwLock<KubeManager>>,
) {
    tokio::spawn(async move {
        let forward_id = get_string(&data, "id");
        let mut tasks_to_cancel = Vec::new();

        {
            let mut w_manager = manager.write().await;
            if let Some(id) = forward_id {
                if let Some(task) = w_manager.port_forward_cancel.remove(&id) {
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
    });
}

