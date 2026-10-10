use async_trait::async_trait;
use serde_json::Value;
use std::sync::{Arc, OnceLock};
use crate::ipc::events::OrbitEvent;

static GLOBAL_EMITTER: OnceLock<Arc<dyn EventEmitter>> = OnceLock::new();

pub fn set_global_emitter(emitter: Arc<dyn EventEmitter>) {
    let _ = GLOBAL_EMITTER.set(emitter);
}

pub fn get_global_emitter() -> Option<Arc<dyn EventEmitter>> {
    GLOBAL_EMITTER.get().cloned()
}

#[async_trait]
pub trait EventEmitter: Send + Sync + 'static {
    async fn emit(&self, event: &str, data: Value) -> Result<(), String>;

    async fn emit_orbit_event(&self, event: &OrbitEvent) -> Result<(), String> {
        let ev_name = event.event_name();
        let payload = match serde_json::to_value(event) {
            Ok(v) => v,
            Err(e) => return Err(e.to_string()),
        };
        // Extract the inner data object if present from the tagged enum
        let data = payload.get("data").cloned().unwrap_or(payload);
        self.emit(ev_name, data).await
    }
}

#[cfg(test)]
use tokio::sync::Mutex;

#[cfg(test)]
#[derive(Clone, Default)]
pub struct MockEventEmitter {
    pub emitted: Arc<Mutex<Vec<(String, Value)>>>,
}

#[cfg(test)]
impl MockEventEmitter {
    pub fn new() -> Self {
        Self {
            emitted: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub async fn get_emitted_events(&self) -> Vec<(String, Value)> {
        self.emitted.lock().await.clone()
    }
}

#[cfg(test)]
#[async_trait]
impl EventEmitter for MockEventEmitter {
    async fn emit(&self, event: &str, data: Value) -> Result<(), String> {
        self.emitted.lock().await.push((event.to_string(), data));
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_mock_emitter_records_events() {
        let emitter = MockEventEmitter::new();
        let result = emitter.emit("testEvent", serde_json::json!({ "foo": "bar" })).await;
        assert!(result.is_ok());

        let events = emitter.get_emitted_events().await;
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].0, "testEvent");
        assert_eq!(events[0].1["foo"], "bar");
    }

    #[tokio::test]
    async fn test_emit_orbit_event_extracts_data() {
        let emitter = MockEventEmitter::new();
        let event = OrbitEvent::EngineConnected {
            status: "ready".to_string(),
            message: "Connected".to_string(),
        };
        let result = emitter.emit_orbit_event(&event).await;
        assert!(result.is_ok());

        let events = emitter.get_emitted_events().await;
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].0, "engineConnected");
        assert_eq!(events[0].1["status"], "ready");
        assert_eq!(events[0].1["message"], "Connected");
    }
}
