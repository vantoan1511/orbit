use std::sync::Arc;
use tokio::sync::Mutex;

/// Native event writer abstraction in Tauri runtime.
#[derive(Debug, Clone, Default)]
pub enum WsWriter {
    #[default]
    Emitter,
}

/// The Bridge forwards events directly to the active Tauri EventEmitter.
pub struct Bridge;

impl Bridge {
    pub async fn send_event(
        _writer: &Arc<Mutex<WsWriter>>,
        _token: &str,
        event: &super::events::OrbitEvent,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let ev_name = event.event_name();
        match event {
            super::events::OrbitEvent::ErrorOccurred { message } => {
                tracing::error!(event = ev_name, error = %message, "Response to UI (Error)");
            }
            super::events::OrbitEvent::CommandSucceeded { message } => {
                tracing::info!(event = ev_name, message = %message, "Response to UI (Command Succeeded)");
            }
            _ => {
                tracing::debug!(event = ev_name, "Response to UI (Event Broadcast)");
            }
        }

        if let Some(emitter) = super::emitter::get_global_emitter() {
            emitter
                .emit_orbit_event(event)
                .await
                .map_err(|e| Box::<dyn std::error::Error>::from(e))?;
        }
        Ok(())
    }
}
