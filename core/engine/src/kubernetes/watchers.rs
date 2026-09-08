use std::sync::Arc;
use std::fmt::Debug;
use std::collections::HashMap;
use futures_util::StreamExt;
use kube::{Api, Client, Resource};
use kube::runtime::watcher;
use serde::Serialize;
use tokio::sync::Mutex;
use tokio::sync::watch;
use crate::ipc::bridge::{Bridge, WsWriter};
use crate::ipc::events::{OrbitEvent, ResourceUpdate};

fn get_resource_key<K: Resource>(obj: &K) -> String {
    if let Some(ref uid) = obj.meta().uid {
        uid.clone()
    } else {
        format!(
            "{}/{}",
            obj.meta().namespace.as_deref().unwrap_or(""),
            obj.meta().name.as_deref().unwrap_or("")
        )
    }
}

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
    tracing::info!(kind = %kind, "Starting watcher");

    'outer: loop {
        let api = Api::<K>::all(client.clone());
        let mut stream = watcher(api, watcher::Config::default()).boxed();

        // 50ms coalescing window buffer to prevent flooding IPC on high frequency events.
        // Uses key_index to coalesce multiple intermediate transitions of the same resource
        // within the 50ms window into the latest state.
        let mut key_index: HashMap<String, usize> = HashMap::new();
        let mut buffer: Vec<ResourceUpdate> = Vec::new();
        let mut flush_interval = tokio::time::interval(tokio::time::Duration::from_millis(50));
        flush_interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

        async fn flush_buffer(
            buffer: &mut Vec<ResourceUpdate>,
            key_index: &mut HashMap<String, usize>,
            writer: &Arc<Mutex<WsWriter>>,
            ipc_token: &str,
            kind: &str,
        ) {
            if !buffer.is_empty() {
                let updates = std::mem::take(buffer);
                key_index.clear();
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
                        tracing::info!(kind = %kind, "Stopping watcher");
                        flush_buffer(&mut buffer, &mut key_index, &writer, &ipc_token, &kind).await;
                        break 'outer;
                    }
                }
                _ = flush_interval.tick() => {
                    flush_buffer(&mut buffer, &mut key_index, &writer, &ipc_token, &kind).await;
                }
                event = stream.next() => {
                    let maybe_item = match event {
                        Some(Ok(watcher::Event::Apply(obj))) | Some(Ok(watcher::Event::InitApply(obj))) => {
                            Some(("Applied", obj))
                        }
                        Some(Ok(watcher::Event::Delete(obj))) => {
                            Some(("Deleted", obj))
                        }
                        Some(Ok(watcher::Event::InitDone)) => {
                            flush_buffer(&mut buffer, &mut key_index, &writer, &ipc_token, &kind).await;
                            tracing::info!(kind = %kind, "Watcher initial sync done");
                            None
                        }
                        Some(Ok(_)) => None,
                        Some(Err(e)) => {
                            tracing::error!(kind = %kind, error = ?e, "Watcher error");
                            None
                        }
                        None => {
                            tracing::info!(kind = %kind, "Watcher stream ended, reconnecting...");
                            flush_buffer(&mut buffer, &mut key_index, &writer, &ipc_token, &kind).await;
                            break;
                        }
                    };

                    if let Some((action, obj)) = maybe_item {
                        let key = get_resource_key(&obj);
                        let mapped = mapper(&obj);
                        if let Ok(data) = serde_json::to_value(&mapped) {
                            tracing::debug!(kind = %kind, action = %action, data = ?data, "Watcher event processed");
                            let update = ResourceUpdate {
                                action: action.to_string(),
                                data,
                            };
                            if let Some(&idx) = key_index.get(&key) {
                                buffer[idx] = update;
                            } else {
                                key_index.insert(key, buffer.len());
                                buffer.push(update);
                            }
                            if buffer.len() >= 100 {
                                flush_buffer(&mut buffer, &mut key_index, &writer, &ipc_token, &kind).await;
                            }
                        }
                    }
                }
            }
        }

        tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use k8s_openapi::apimachinery::pkg::apis::meta::v1::ObjectMeta;
    use k8s_openapi::api::core::v1::Pod;

    #[test]
    fn test_get_resource_key_prefers_uid() {
        let pod = Pod {
            metadata: ObjectMeta {
                uid: Some("uid-12345".to_string()),
                name: Some("my-pod".to_string()),
                namespace: Some("prod".to_string()),
                ..Default::default()
            },
            ..Default::default()
        };
        assert_eq!(get_resource_key(&pod), "uid-12345");
    }

    #[test]
    fn test_get_resource_key_fallback_namespace_name() {
        let pod = Pod {
            metadata: ObjectMeta {
                uid: None,
                name: Some("my-pod".to_string()),
                namespace: Some("prod".to_string()),
                ..Default::default()
            },
            ..Default::default()
        };
        assert_eq!(get_resource_key(&pod), "prod/my-pod");
    }

    #[test]
    fn test_get_resource_key_cluster_scoped_fallback() {
        let pod = Pod {
            metadata: ObjectMeta {
                uid: None,
                name: Some("node-1".to_string()),
                namespace: None,
                ..Default::default()
            },
            ..Default::default()
        };
        assert_eq!(get_resource_key(&pod), "/node-1");
    }
}
