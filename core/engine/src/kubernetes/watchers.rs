use std::sync::Arc;
use std::fmt::Debug;
use futures_util::StreamExt;
use kube::{Api, Client, Resource};
use kube::runtime::watcher;
use serde::Serialize;
use tokio::sync::Mutex;
use tokio::sync::watch;
use crate::ipc::bridge::{Bridge, WsWriter};
use crate::ipc::events::OrbitEvent;

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
        let mut buffer: Vec<OrbitEvent> = Vec::new();
        let mut flush_interval = tokio::time::interval(tokio::time::Duration::from_millis(50));
        flush_interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

        loop {
            tokio::select! {
                res = cancel_rx.changed() => {
                    if res.is_ok() && *cancel_rx.borrow() {
                        log::info!("Stopping watcher for {}", kind);
                        break 'outer;
                    }
                }
                _ = flush_interval.tick() => {
                    if !buffer.is_empty() {
                        let batch = std::mem::take(&mut buffer);
                        for event in &batch {
                            let _ = Bridge::send_event(&writer, &ipc_token, event).await;
                        }
                    }
                }
                event = stream.next() => {
                    match event {
                        Some(Ok(watcher::Event::Apply(obj))) | Some(Ok(watcher::Event::InitApply(obj))) => {
                            let mapped = mapper(&obj);
                            if let Ok(data) = serde_json::to_value(&mapped) {
                                log::debug!("Watcher Applied {} {:?}", kind, data);
                                buffer.push(OrbitEvent::ResourceUpdated {
                                    kind: kind.clone(),
                                    action: "Applied".to_string(),
                                    data,
                                });
                                if buffer.len() >= 100 {
                                    let batch = std::mem::take(&mut buffer);
                                    for evt in &batch {
                                        let _ = Bridge::send_event(&writer, &ipc_token, evt).await;
                                    }
                                }
                            }
                        }
                        Some(Ok(watcher::Event::Delete(obj))) => {
                            let mapped = mapper(&obj);
                            if let Ok(data) = serde_json::to_value(&mapped) {
                                log::debug!("Watcher Deleted {} {:?}", kind, data);
                                buffer.push(OrbitEvent::ResourceUpdated {
                                    kind: kind.clone(),
                                    action: "Deleted".to_string(),
                                    data,
                                });
                                if buffer.len() >= 100 {
                                    let batch = std::mem::take(&mut buffer);
                                    for evt in &batch {
                                        let _ = Bridge::send_event(&writer, &ipc_token, evt).await;
                                    }
                                }
                            }
                        }
                        Some(Ok(watcher::Event::InitDone)) => {
                            log::info!("Watcher initial sync done for {}", kind);
                        }
                        Some(Ok(_)) => {}
                        Some(Err(e)) => {
                            log::error!("Watcher error for {}: {:?}", kind, e);
                        }
                        None => {
                            log::info!("Watcher stream ended for {}. Reconnecting...", kind);
                            break;
                        }
                    }
                }
            }
        }

        tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;
    }
}
