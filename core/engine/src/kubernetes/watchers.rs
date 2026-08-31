use std::sync::Arc;
use std::fmt::Debug;
use futures_util::StreamExt;
use kube::{Api, Client, Resource};
use kube::runtime::watcher;
use serde::Serialize;
use tokio::sync::Mutex;
use tokio::sync::watch;
use crate::ipc::bridge::{Bridge, WsWriter};
use crate::ipc::events::{OrbitEvent, ResourceUpdate};

pub async fn watch_resource<K, M, F>(
    client: Client,
    writer: Arc<Mutex<WsWriter>>,
    ipc_token: String,
    kind: String,
    mut cancel_rx: watch::Receiver<bool>,
    mapper: F,
) where
    K: Resource + Clone + serde::de::DeserializeOwned + Debug + Send + Sync + 'static,
    K::DynamicType: Default,
    M: Serialize + Send + Sync + 'static,
    F: Fn(&K) -> M + Send + Sync + 'static,
{
    log::info!("Starting watcher for {}", kind);

    'outer: loop {
        let api = Api::<K>::all(client.clone());
        let mut stream = watcher(api, watcher::Config::default()).boxed();

        // 50ms coalescing window buffer to prevent flooding IPC on high frequency events
        let mut buffer: Vec<ResourceUpdate> = Vec::new();
        let mut flush_interval = tokio::time::interval(tokio::time::Duration::from_millis(50));
        flush_interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

        async fn flush_buffer(
            buffer: &mut Vec<ResourceUpdate>,
            writer: &Arc<Mutex<WsWriter>>,
            ipc_token: &str,
            kind: &str,
        ) {
            if !buffer.is_empty() {
                let updates = std::mem::take(buffer);
                let event = OrbitEvent::ResourceBatchUpdated {
                    kind: kind.to_string(),
                    updates,
                };
                let _ = Bridge::send_event(writer, ipc_token, &event).await;
            }
        }

        loop {
            tokio::select! {
                res = cancel_rx.changed() => {
                    if res.is_ok() && *cancel_rx.borrow() {
                        log::info!("Stopping watcher for {}", kind);
                        flush_buffer(&mut buffer, &writer, &ipc_token, &kind).await;
                        break 'outer;
                    }
                }
                _ = flush_interval.tick() => {
                    flush_buffer(&mut buffer, &writer, &ipc_token, &kind).await;
                }
                event = stream.next() => {
                    match event {
                        Some(Ok(watcher::Event::Apply(obj))) | Some(Ok(watcher::Event::InitApply(obj))) => {
                            let mapped = mapper(&obj);
                            if let Ok(data) = serde_json::to_value(&mapped) {
                                log::debug!("Watcher Applied {} {:?}", kind, data);
                                buffer.push(ResourceUpdate {
                                    action: "Applied".to_string(),
                                    data,
                                });
                                if buffer.len() >= 100 {
                                    flush_buffer(&mut buffer, &writer, &ipc_token, &kind).await;
                                }
                            }
                        }
                        Some(Ok(watcher::Event::Delete(obj))) => {
                            let mapped = mapper(&obj);
                            if let Ok(data) = serde_json::to_value(&mapped) {
                                log::debug!("Watcher Deleted {} {:?}", kind, data);
                                buffer.push(ResourceUpdate {
                                    action: "Deleted".to_string(),
                                    data,
                                });
                                if buffer.len() >= 100 {
                                    flush_buffer(&mut buffer, &writer, &ipc_token, &kind).await;
                                }
                            }
                        }
                        Some(Ok(watcher::Event::InitDone)) => {
                            flush_buffer(&mut buffer, &writer, &ipc_token, &kind).await;
                            log::info!("Watcher initial sync done for {}", kind);
                        }
                        Some(Ok(_)) => {}
                        Some(Err(e)) => {
                            log::error!("Watcher error for {}: {:?}", kind, e);
                        }
                        None => {
                            log::info!("Watcher stream ended for {}. Reconnecting...", kind);
                            flush_buffer(&mut buffer, &writer, &ipc_token, &kind).await;
                            break;
                        }
                    }
                }
            }
        }

        tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;
    }
}
