mod ipc;
mod kubernetes;

use std::error::Error;
use ipc::bridge::{AuthInfo, Bridge};
use ipc::events::OrbitEvent;

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

    // Broadcast that the core is connected and ready
    Bridge::send_event(
        &bridge.writer,
        &bridge.token,
        &OrbitEvent::EngineConnected {
            status: "ready".to_string(),
            message: "Orbit Engine is connected and ready.".to_string(),
        },
    ).await?;

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
            tokio::spawn(async move {
                let _ = Bridge::send_event(
                    &writer,
                    &token,
                    &OrbitEvent::EngineConnected {
                        status: "ready".to_string(),
                        message: "Orbit Engine is connected and ready.".to_string(),
                    },
                ).await;
            });
        }

        // Handle extension events from frontend
        if let Some(event_name) = msg.event.as_deref() {
            let writer = bridge.writer.clone();
            let token = bridge.token.clone();
            
            match event_name {
                "getNamespaces" => {
                    tokio::spawn(async move {
                        match kubernetes::list_namespaces().await {
                            Ok(namespaces) => {
                                let _ = Bridge::send_event(
                                    &writer,
                                    &token,
                                    &OrbitEvent::NamespacesUpdated { namespaces },
                                ).await;
                            }
                            Err(e) => {
                                eprintln!("Error listing namespaces: {:?}", e);
                            }
                        }
                    });
                }
                "getPods" => {
                    let ext_data = msg.data.clone();
                    tokio::spawn(async move {
                        let namespace = ext_data
                            .and_then(|d| d.get("namespace").cloned())
                            .and_then(|v| v.as_str().map(|s| s.to_string()));
                            
                        match kubernetes::list_pods(namespace).await {
                            Ok(pods) => {
                                let _ = Bridge::send_event(
                                    &writer,
                                    &token,
                                    &OrbitEvent::PodsUpdated { pods },
                                ).await;
                            }
                            Err(e) => {
                                eprintln!("Error listing pods: {:?}", e);
                            }
                        }
                    });
                }
                _ => {}
            }
        }

        tokio::spawn(async move {
            println!("Handled received message: {:?}", msg);
        });
    }

    Ok(())
}
