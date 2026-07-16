mod ipc;
mod kubernetes;
pub mod updater;

use std::error::Error;
use std::sync::Arc;
use tokio::sync::RwLock;
use ipc::bridge::{AuthInfo, Bridge};
use ipc::events::OrbitEvent;
use kubernetes::manager::KubeManager;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    println!("Orbit Core Engine starting up...");

    // Retrieve authentication details from stdin
    let mut auth = AuthInfo::from_stdin();
    // Allow overriding authentication details from command-line arguments
    auth.override_from_cli();

    println!("Connecting to port: {}", auth.nl_port);

    let mut bridge = Bridge::connect(&auth).await?;
    println!("Orbit Core Engine connected to Neutralinojs WebSocket server.");

    // Initialize KubeManager
    let kube_manager = Arc::new(RwLock::new(KubeManager::new().await));

    // Broadcast that the core is connected and ready
    Bridge::send_event(
        &bridge.writer,
        &bridge.token,
        &OrbitEvent::EngineConnected {
            status: "ready".to_string(),
            message: "Orbit Engine is connected and ready.".to_string(),
        },
    ).await?;

    // Also send initial clusters list and active context
    {
        let mut manager = kube_manager.write().await;
        let clusters = manager.get_clusters();
        let active_cluster_id = manager.active_context.clone();
        let client = manager.active_client.clone();

        let _ = Bridge::send_event(
            &bridge.writer,
            &bridge.token,
            &OrbitEvent::ClustersUpdated { clusters },
        ).await;

        let _ = Bridge::send_event(
            &bridge.writer,
            &bridge.token,
            &OrbitEvent::ActiveClusterChanged { active_cluster_id },
        ).await;

        if let Some(cancel) = manager.watch_cancel.take() {
            let _ = cancel.send(true);
        }
        let (tx, rx) = tokio::sync::watch::channel(false);
        manager.watch_cancel = Some(tx);

        if let Some(ref client) = client {
            let writer_s = bridge.writer.clone();
            let token_s = bridge.token.clone();
            let client_s = client.clone();
            let rx_s = rx.clone();
            tokio::spawn(async move {
                kubernetes::watchers::watch_resource::<k8s_openapi::api::core::v1::Service, _, _>(
                    client_s,
                    writer_s,
                    token_s,
                    "Service".to_string(),
                    rx_s,
                    kubernetes::services::map_service,
                ).await;
            });
            let writer_d = bridge.writer.clone();
            let token_d = bridge.token.clone();
            let client_d = client.clone();
            let rx_d = rx.clone();
            tokio::spawn(async move {
                kubernetes::watchers::watch_resource::<k8s_openapi::api::apps::v1::Deployment, _, _>(
                    client_d,
                    writer_d,
                    token_d,
                    "Deployment".to_string(),
                    rx_d,
                    kubernetes::workloads::map_deployment,
                ).await;
            });
            let writer_p = bridge.writer.clone();
            let token_p = bridge.token.clone();
            let client_p = client.clone();
            let rx_p = rx.clone();
            tokio::spawn(async move {
                kubernetes::watchers::watch_resource::<k8s_openapi::api::core::v1::Pod, _, _>(
                    client_p,
                    writer_p,
                    token_p,
                    "Pod".to_string(),
                    rx_p,
                    kubernetes::workloads::map_pod,
                ).await;
            });
        }
    }

    // Message processing loop
    loop {
        let msg = match Bridge::read_message(&mut bridge.reader, &bridge.writer).await {
            Ok(msg) => msg,
            Err(e) => {
                eprintln!("WebSocket error occurred or connection closed: {:?}", e);
                break;
            }
        };

        if msg.event.as_deref() == Some("windowClose") {
            log::info!("Received windowClose, shutting down.");
            break;
        }

        // Re-broadcast connection status when a client connects to ensure the frontend receives it
        if msg.event.as_deref() == Some("appClientConnect") || msg.event.as_deref() == Some("clientConnect") {
            let writer = bridge.writer.clone();
            let token = bridge.token.clone();
            let manager = kube_manager.clone();
            tokio::spawn(async move {
                let _ = Bridge::send_event(
                    &writer,
                    &token,
                    &OrbitEvent::EngineConnected {
                        status: "ready".to_string(),
                        message: "Orbit Engine is connected and ready.".to_string(),
                    },
                ).await;

                let r_manager = manager.read().await;
                let clusters = r_manager.get_clusters();
                let active_cluster_id = r_manager.active_context.clone();

                let _ = Bridge::send_event(&writer, &token, &OrbitEvent::ClustersUpdated { clusters }).await;
                let _ = Bridge::send_event(&writer, &token, &OrbitEvent::ActiveClusterChanged { active_cluster_id }).await;
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

    Ok(())
}
