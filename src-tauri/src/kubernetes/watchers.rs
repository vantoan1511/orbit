use std::sync::Arc;
use std::fmt::Debug;
use std::collections::HashMap;
use std::time::Duration;
use futures_util::StreamExt;
use kube::{Api, Client, Resource};
use kube::runtime::{watcher, WatchStreamExt};
use serde::Serialize;
use tokio::sync::Mutex;
use tokio::sync::watch;
use crate::ipc::bridge::{Bridge, WsWriter};
use crate::ipc::events::{OrbitEvent, ResourceUpdate, WatcherConnectionState};
use crate::kubernetes::watch_health::WatcherHealthTracker;

const COALESCE_INTERVAL: Duration = Duration::from_millis(50);
const COALESCE_BUFFER_MAX: usize = 100;
const STREAM_RETRY_INTERVAL: Duration = Duration::from_secs(3);

pub struct WatcherContext {
    pub client: Client,
    pub writer: Arc<Mutex<WsWriter>>,
    pub ipc_token: String,
    pub cluster_id: Option<String>,
    pub tracker: Arc<WatcherHealthTracker>,
    pub cancel_rx: watch::Receiver<bool>,
}

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

async fn emit_connection_state(
    writer: &Arc<Mutex<WsWriter>>,
    ipc_token: &str,
    cluster_id: Option<&str>,
    state: WatcherConnectionState,
    failing: usize,
) {
    let event = OrbitEvent::WatcherConnectionChanged {
        cluster_id: cluster_id.map(|s| s.to_string()),
        state,
        failing_watchers: failing,
    };
    if let Err(e) = Bridge::send_event(writer, ipc_token, &event).await {
        tracing::debug!(error = %e, "Failed to emit watcherConnectionChanged event");
    }
}

pub async fn watch_resource<K, M, F>(
    kind: String,
    mut ctx: WatcherContext,
    mapper: F,
) where
    K: Resource + Clone + serde::de::DeserializeOwned + Debug + Send + Sync + 'static,
    K::DynamicType: Default,
    M: Serialize + Send + Sync + 'static,
    F: Fn(&K) -> M + Send + Sync + 'static,
{
    let watcher_id = format!("{kind}:{}", K::kind(&Default::default()));
    tracing::info!(kind = %kind, watcher_id = %watcher_id, "Starting watcher");

    let mut has_failed = false;

    'outer: loop {
        let api = Api::<K>::all(ctx.client.clone());
        let mut stream = watcher(api, watcher::Config::default())
            .default_backoff()
            .boxed();

        let mut key_index: HashMap<String, usize> = HashMap::new();
        let mut buffer: Vec<ResourceUpdate> = Vec::new();
        let mut flush_interval = tokio::time::interval(COALESCE_INTERVAL);
        flush_interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

        async fn flush_buffer(
            buffer: &mut Vec<ResourceUpdate>,
            key_index: &mut HashMap<String, usize>,
            writer: &Arc<Mutex<WsWriter>>,
            ipc_token: &str,
            cluster_id: Option<String>,
            kind: &str,
        ) {
            if !buffer.is_empty() {
                let updates = std::mem::take(buffer);
                key_index.clear();
                let event = OrbitEvent::ResourceBatchUpdated {
                    kind: kind.to_string(),
                    cluster_id,
                    updates,
                };
                if let Err(e) = Bridge::send_event(writer, ipc_token, &event).await {
                    tracing::debug!(error = %e, "Failed to send batch update");
                }
            }
        }

        loop {
            tokio::select! {
                res = ctx.cancel_rx.changed() => {
                    if res.is_err() || *ctx.cancel_rx.borrow() {
                        tracing::info!(kind = %kind, "Stopping watcher");
                        flush_buffer(&mut buffer, &mut key_index, &ctx.writer, &ctx.ipc_token, ctx.cluster_id.clone(), &kind).await;
                        break 'outer;
                    }
                }
                _ = flush_interval.tick() => {
                    flush_buffer(&mut buffer, &mut key_index, &ctx.writer, &ctx.ipc_token, ctx.cluster_id.clone(), &kind).await;
                }
                event = stream.next() => {
                    let maybe_item = match event {
                        Some(Ok(watcher::Event::Apply(obj))) | Some(Ok(watcher::Event::InitApply(obj))) => {
                            if let Some(state) = ctx.tracker.record_success(&watcher_id) {
                                emit_connection_state(
                                    &ctx.writer,
                                    &ctx.ipc_token,
                                    ctx.cluster_id.as_deref(),
                                    state,
                                    ctx.tracker.failing_count(),
                                ).await;
                                tracing::info!(kind = %kind, "Watcher recovered");
                            }
                            has_failed = false;
                            Some(("Applied", obj))
                        }
                        Some(Ok(watcher::Event::Delete(obj))) => {
                            if let Some(state) = ctx.tracker.record_success(&watcher_id) {
                                emit_connection_state(
                                    &ctx.writer,
                                    &ctx.ipc_token,
                                    ctx.cluster_id.as_deref(),
                                    state,
                                    ctx.tracker.failing_count(),
                                ).await;
                                tracing::info!(kind = %kind, "Watcher recovered");
                            }
                            has_failed = false;
                            Some(("Deleted", obj))
                        }
                        Some(Ok(watcher::Event::InitDone)) => {
                            flush_buffer(&mut buffer, &mut key_index, &ctx.writer, &ctx.ipc_token, ctx.cluster_id.clone(), &kind).await;
                            if let Some(state) = ctx.tracker.record_success(&watcher_id) {
                                emit_connection_state(
                                    &ctx.writer,
                                    &ctx.ipc_token,
                                    ctx.cluster_id.as_deref(),
                                    state,
                                    ctx.tracker.failing_count(),
                                ).await;
                                tracing::info!(kind = %kind, "Watcher recovered");
                            }
                            has_failed = false;
                            tracing::info!(kind = %kind, "Watcher initial sync done");
                            None
                        }
                        Some(Ok(_)) => None,
                        Some(Err(e)) => {
                            let transition = ctx.tracker.record_failure(&watcher_id, std::time::Instant::now());
                            if !has_failed {
                                tracing::warn!(kind = %kind, error = %e, "Watcher error (backoff active)");
                                has_failed = true;
                            } else {
                                tracing::debug!(kind = %kind, error = %e, "Watcher retry failed");
                            }
                            if let Some(state) = transition {
                                emit_connection_state(
                                    &ctx.writer,
                                    &ctx.ipc_token,
                                    ctx.cluster_id.as_deref(),
                                    state,
                                    ctx.tracker.failing_count(),
                                ).await;
                            }
                            None
                        }
                        None => {
                            tracing::info!(kind = %kind, "Watcher stream ended, reconnecting...");
                            flush_buffer(&mut buffer, &mut key_index, &ctx.writer, &ctx.ipc_token, ctx.cluster_id.clone(), &kind).await;
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
                            if buffer.len() >= COALESCE_BUFFER_MAX {
                                flush_buffer(&mut buffer, &mut key_index, &ctx.writer, &ctx.ipc_token, ctx.cluster_id.clone(), &kind).await;
                            }
                        }
                    }
                }
            }
        }

        tokio::select! {
            res = ctx.cancel_rx.changed() => {
                if res.is_err() || *ctx.cancel_rx.borrow() {
                    break 'outer;
                }
            }
            _ = tokio::time::sleep(STREAM_RETRY_INTERVAL) => {}
        }
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

    #[tokio::test]
    async fn test_watch_cancellation_logic() {
        let (tx, mut rx) = tokio::sync::watch::channel(false);
        assert!(!*rx.borrow());

        tx.send(true).unwrap();
        assert!(rx.changed().await.is_ok());
        assert!(*rx.borrow());

        drop(tx);
        assert!(rx.changed().await.is_err());
    }

    #[tokio::test]
    async fn test_unreachable_api_server_does_not_hot_loop() {
        let config = kube::Config::new("http://127.0.0.1:1".parse().unwrap());
        let client = kube::Client::try_from(config).unwrap();
        let (tx, _rx) = tokio::sync::watch::channel(false);
        let (cancel_tx, cancel_rx) = tokio::sync::watch::channel(false);
        let tracker = crate::kubernetes::watch_health::WatcherHealthTracker::new(
            Duration::from_secs(300),
            tx,
        );
        let writer = Arc::new(Mutex::new(WsWriter::Emitter));
        let ctx = WatcherContext {
            client,
            writer,
            ipc_token: "test".to_string(),
            cluster_id: Some("test-cluster".to_string()),
            tracker: tracker.clone(),
            cancel_rx,
        };

        let handle = tokio::spawn(watch_resource::<Pod, _, _>(
            "Pod".to_string(),
            ctx,
            |_| (),
        ));

        tokio::time::sleep(Duration::from_secs(3)).await;
        let _ = cancel_tx.send(true);
        let _ = handle.await;

        assert!(tracker.total_failure_records() >= 1);
        assert!(tracker.total_failure_records() <= 6);
    }
}
