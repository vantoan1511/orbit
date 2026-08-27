mod ipc;
mod kubernetes;
pub mod updater;
pub mod config;
pub mod logger;

use std::error::Error;
use std::sync::Arc;
use std::time::Duration;
use futures_util::SinkExt;
use tokio::sync::{Mutex, RwLock};
use tokio::time::MissedTickBehavior;
use tokio_tungstenite::tungstenite::Message;
use ipc::bridge::{AuthInfo, Bridge, PING_INTERVAL_SECS, WsWriter};
use ipc::events::OrbitEvent;
use kubernetes::manager::KubeManager;

async fn broadcast_engine_ready(
    writer: &Arc<Mutex<WsWriter>>,
    token: &str,
    kube_manager: &Arc<RwLock<KubeManager>>,
) {
    let _ = Bridge::send_event(
        writer,
        token,
        &OrbitEvent::EngineConnected {
            status: "ready".to_string(),
            message: "Orbit Engine is connected and ready.".to_string(),
        },
    ).await;

    let r_manager = kube_manager.read().await;
    let clusters = r_manager.get_clusters();
    let active_cluster_id = r_manager.active_context.clone();
    drop(r_manager);

    let _ = Bridge::send_event(writer, token, &OrbitEvent::ClustersUpdated { clusters }).await;
    let _ = Bridge::send_event(writer, token, &OrbitEvent::ActiveClusterChanged { active_cluster_id }).await;
}

async fn restart_watchers(
    bridge: &Bridge,
    kube_manager: &Arc<RwLock<KubeManager>>,
) {
    let mut w_manager = kube_manager.write().await;

    // Cancel any existing watcher tasks
    if let Some(cancel) = w_manager.watch_cancel.take() {
        let _ = cancel.send(true);
    }

    let client = w_manager.active_client.clone();
    let (tx, rx) = tokio::sync::watch::channel(false);
    w_manager.watch_cancel = Some(tx);
    drop(w_manager);

    if let Some(ref client) = client {
        ipc::handlers::spawn_watchers(
            client,
            bridge.writer.clone(),
            bridge.token.clone(),
            rx,
        );
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Initialize persistent activity logging
    if let Err(e) = logger::init() {
        eprintln!("Warning: Failed to initialize file logger: {}", e);
    }

    tracing::info!("Orbit Core Engine starting up...");

    // Retrieve authentication details from stdin
    let mut auth = AuthInfo::from_stdin();
    // Allow overriding authentication details from command-line arguments
    auth.override_from_cli();

    println!("Connecting to port: {}", auth.nl_port);

    // Initialize KubeManager
    let kube_manager = Arc::new(RwLock::new(KubeManager::new().await));

    let mut backoff_secs = 1u64;

    'reconnect: loop {
        let mut bridge = match Bridge::connect(&auth).await {
            Ok(b) => {
                tracing::info!("Orbit Core Engine connected to Neutralinojs WebSocket server.");
                backoff_secs = 1;
                b
            }
            Err(e) => {
                tracing::error!("Failed to connect to Neutralino WebSocket server: {:?}", e);
                tokio::time::sleep(Duration::from_secs(backoff_secs)).await;
                backoff_secs = next_backoff(backoff_secs);
                continue 'reconnect;
            }
        };

        // Broadcast that the core is connected and ready along with initial clusters & context
        broadcast_engine_ready(&bridge.writer, &bridge.token, &kube_manager).await;

        // Restart watchers with the new bridge writer
        restart_watchers(&bridge, &kube_manager).await;

        let mut ping_interval = tokio::time::interval(Duration::from_secs(PING_INTERVAL_SECS));
        ping_interval.set_missed_tick_behavior(MissedTickBehavior::Skip);

        // Message processing loop
        loop {
            tokio::select! {
                _ = ping_interval.tick() => {
                    let mut w = bridge.writer.lock().await;
                    if let Err(e) = w.send(Message::Ping(vec![].into())).await {
                        tracing::warn!("Ping failed (bridge dead), reconnecting: {:?}", e);
                        break;
                    }
                }
                result = Bridge::read_message(&mut bridge.reader, &bridge.writer) => {
                    let msg = match result {
                        Ok(msg) => msg,
                        Err(e) => {
                            tracing::warn!("WebSocket error occurred or connection closed: {:?}. Reconnecting...", e);
                            break;
                        }
                    };

                    if msg.event.as_deref() == Some("windowClose") {
                        tracing::info!("Received windowClose, shutting down.");
                        break 'reconnect;
                    }

                    // Re-broadcast connection status when a client connects to ensure the frontend receives it
                    if msg.event.as_deref() == Some("appClientConnect") || msg.event.as_deref() == Some("clientConnect") {
                        let writer = bridge.writer.clone();
                        let token = bridge.token.clone();
                        let manager = kube_manager.clone();
                        tokio::spawn(async move {
                            broadcast_engine_ready(&writer, &token, &manager).await;
                        });
                    }

                    // Dispatch all Kubernetes resource events to the handler module
                    if let Some(event_name) = msg.event.as_deref() {
                        ipc::handlers::dispatch(
                            event_name,
                            msg.data.clone(),
                            bridge.writer.clone(),
                            bridge.token.clone(),
                            kube_manager.clone(),
                        );
                    }
                }
            }
        }

        tracing::info!("Bridge disconnected. Reconnecting in {}s...", backoff_secs);
        tokio::time::sleep(Duration::from_secs(backoff_secs)).await;
        backoff_secs = next_backoff(backoff_secs);
    }

    Ok(())
}

fn next_backoff(current_backoff: u64) -> u64 {
    (current_backoff * 2).min(30)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_next_backoff() {
        assert_eq!(next_backoff(1), 2);
        assert_eq!(next_backoff(2), 4);
        assert_eq!(next_backoff(4), 8);
        assert_eq!(next_backoff(8), 16);
        assert_eq!(next_backoff(16), 30);
        assert_eq!(next_backoff(30), 30);
    }

    #[test]
    fn test_ping_interval_constant() {
        assert_eq!(PING_INTERVAL_SECS, 30);
    }
}

