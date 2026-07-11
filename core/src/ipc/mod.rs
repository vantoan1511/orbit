pub mod models;

use futures_util::{Sink, SinkExt, StreamExt};
use models::{ExtensionEvent, NeutralinoConfig};
use std::error::Error;
use tokio::net::TcpStream;
use tokio_tungstenite::{
    connect_async,
    tungstenite::protocol::Message,
    MaybeTlsStream,
    WebSocketStream,
};

pub type WsStream = WebSocketStream<MaybeTlsStream<TcpStream>>;

pub async fn connect(config: &NeutralinoConfig) -> Result<WsStream, Box<dyn Error>> {
    let ws_url = format!(
        "ws://localhost:{}?extensionId={}&connectToken={}",
        config.nl_port, config.nl_extension_id, config.nl_connect_token
    );

    let (ws_stream, _) = connect_async(ws_url).await?;
    Ok(ws_stream)
}

pub async fn run_loop(ws_stream: WsStream) -> Result<(), Box<dyn Error>> {
    let (mut write, mut read) = ws_stream.split();

    println!("Orbit Core Engine connected to Neutralinojs WebSocket server.");

    // Simple test message dispatch back to frontend to verify connection
    let init_event = ExtensionEvent {
        event: "coreConnected".to_string(),
        data: Some(serde_json::json!({
            "status": "ready",
            "message": "Rust Core Engine is connected and ready."
        })),
    };
    let init_msg = serde_json::to_string(&init_event)?;
    write.send(Message::Text(init_msg.into())).await?;

    while let Some(message) = read.next().await {
        match message {
            Ok(Message::Text(text)) => {
                match serde_json::from_str::<ExtensionEvent>(text.as_str()) {
                    Ok(event) => {
                        println!("Received event: {}", event.event);
                        // Route the event
                        handle_event(&event, &mut write).await?;
                    }
                    Err(err) => {
                        eprintln!("Failed to deserialize extension event: {:?}", err);
                    }
                }
            }
            Ok(Message::Close(_)) => {
                println!("WebSocket connection closed by host.");
                break;
            }
            Err(e) => {
                eprintln!("WebSocket error occurred: {:?}", e);
                return Err(e.into());
            }
            _ => {}
        }
    }

    Ok(())
}

async fn handle_event<S>(event: &ExtensionEvent, writer: &mut S) -> Result<(), Box<dyn Error>>
where
    S: Sink<Message> + Unpin,
    <S as Sink<Message>>::Error: std::fmt::Debug + Error + 'static,
{
    match event.event.as_str() {
        "ping" => {
            let response = ExtensionEvent {
                event: "pong".to_string(),
                data: Some(serde_json::json!({ "reply": "Hello from Rust Core!" })),
            };
            let response_msg = serde_json::to_string(&response)?;
            writer.send(Message::Text(response_msg.into())).await?;
        }
        _ => {
            println!("Unhandled event: {}", event.event);
        }
    }
    Ok(())
}
