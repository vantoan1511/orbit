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
    let api = Api::<K>::all(client);
    let mut stream = watcher(api, watcher::Config::default()).boxed();

    log::info!("Starting watcher for {}", kind);

    loop {
        tokio::select! {
            res = cancel_rx.changed() => {
                if res.is_ok() && *cancel_rx.borrow() {
                    log::info!("Stopping watcher for {}", kind);
                    break;
                }
            }
            event = stream.next() => {
                match event {
                    Some(Ok(watcher::Event::Apply(obj))) | Some(Ok(watcher::Event::InitApply(obj))) => {
                        let mapped = mapper(&obj);
                        if let Ok(data) = serde_json::to_value(&mapped) {
                            let _ = std::fs::write("orbit_watcher_debug.log", format!("Applied: {:?}", data));
                            let _ = Bridge::send_event(
                                &writer,
                                &ipc_token,
                                &OrbitEvent::ResourceUpdated {
                                    kind: kind.clone(),
                                    action: "Applied".to_string(),
                                    data,
                                }
                            ).await;
                        }
                    }
                    Some(Ok(watcher::Event::Delete(obj))) => {
                        let mapped = mapper(&obj);
                        if let Ok(data) = serde_json::to_value(&mapped) {
                            let _ = std::fs::write("orbit_watcher_debug.log", format!("Deleted: {:?}", data));
                            let _ = Bridge::send_event(
                                &writer,
                                &ipc_token,
                                &OrbitEvent::ResourceUpdated {
                                    kind: kind.clone(),
                                    action: "Deleted".to_string(),
                                    data,
                                }
                            ).await;
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
                        log::info!("Watcher stream ended for {}", kind);
                        break;
                    }
                }
            }
        }
    }
}
