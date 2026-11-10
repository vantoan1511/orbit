use std::sync::{Arc, Mutex};
use tokio::sync::mpsc::UnboundedSender;
use crate::events::CoreEvent;

pub trait CoreEventSink: Send + Sync + 'static {
    fn send(&self, event: CoreEvent) -> Result<(), String>;
}

pub struct MpscEventSink {
    sender: UnboundedSender<CoreEvent>,
}

impl MpscEventSink {
    pub fn new(sender: UnboundedSender<CoreEvent>) -> Self {
        Self { sender }
    }
}

impl CoreEventSink for MpscEventSink {
    fn send(&self, event: CoreEvent) -> Result<(), String> {
        self.sender.send(event).map_err(|e| e.to_string())
    }
}

#[derive(Default, Clone)]
pub struct MockEventSink {
    pub events: Arc<Mutex<Vec<CoreEvent>>>,
}

impl MockEventSink {
    pub fn new() -> Self {
        Self {
            events: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn get_events(&self) -> Vec<CoreEvent> {
        self.events.lock().unwrap_or_else(|e| e.into_inner()).clone()
    }
}

impl CoreEventSink for MockEventSink {
    fn send(&self, event: CoreEvent) -> Result<(), String> {
        let mut list = self.events.lock().map_err(|e| e.to_string())?;
        list.push(event);
        Ok(())
    }
}
