mod ipc;

use std::error::Error;
use ipc::bridge::{AuthInfo, Bridge};

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
    Bridge::broadcast(
        &bridge.writer,
        &bridge.token,
        "engineConnected",
        serde_json::json!({
            "status": "ready",
            "message": "Orbit Engine is connected and ready."
        }),
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

        tokio::spawn(async move {
            println!("Handled received message: {:?}", msg);
        });
    }

    Ok(())
}
