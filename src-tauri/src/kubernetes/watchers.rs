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
use crate::kubernetes::watch_error::classify_watcher_error;
use crate::kubernetes::watch_health::WatcherHealthTracker;

const COALESCE_INTERVAL: Duration = Duration::from_millis(50);
const COALESCE_BUFFER_MAX: usize = 100;
const STREAM_RETRY_INTERVAL: Duration = Duration::from_secs(3);
const ACTION_APPLIED: &str = "Applied";
const ACTION_DELETED: &str = "Deleted";

#[derive(Clone)]
pub struct ConnectionReporter {
    pub(crate) writer: Arc<Mutex<WsWriter>>,
    pub(crate) ipc_token: String,
    pub(crate) cluster_id: Option<String>,
}

impl ConnectionReporter {
    pub fn new(writer: Arc<Mutex<WsWriter>>, ipc_token: String, cluster_id: Option<String>) -> Self {
        Self {
            writer,
            ipc_token,
            cluster_id,
        }
    }

    pub async fn report(&self, state: WatcherConnectionState, failing_watchers: usize) {
        tracing::info!(
            state = ?state,
            failing_watchers = failing_watchers,
            cluster_id = self.cluster_id.as_deref().unwrap_or(""),
            "Watcher aggregate connection state changed"
        );
        let event = OrbitEvent::WatcherConnectionChanged {
            cluster_id: self.cluster_id.clone(),
            state,
            failing_watchers,
        };
        if let Err(e) = Bridge::send_event(&self.writer, &self.ipc_token, &event).await {
            tracing::debug!(error = %e, "Failed to emit watcherConnectionChanged event");
        }
    }
}

pub struct WatcherContext {
    pub client: Client,
    pub reporter: ConnectionReporter,
    pub tracker: Arc<WatcherHealthTracker>,
    pub cancel_rx: watch::Receiver<bool>,
}

struct WatchBuffer {
    updates: Vec<ResourceUpdate>,
    key_index: HashMap<String, usize>,
}

impl WatchBuffer {
    fn new() -> Self {
        Self {
            updates: Vec::new(),
            key_index: HashMap::new(),
        }
    }

    fn upsert(&mut self, key: String, update: ResourceUpdate) -> bool {
        if let Some(&idx) = self.key_index.get(&key) {
            self.updates[idx] = update;
        } else {
            self.key_index.insert(key, self.updates.len());
            self.updates.push(update);
        }
        self.updates.len() >= COALESCE_BUFFER_MAX
    }

    fn take_batch(&mut self) -> Option<Vec<ResourceUpdate>> {
        if self.updates.is_empty() {
            None
        } else {
            self.key_index.clear();
            Some(std::mem::take(&mut self.updates))
        }
    }
}

async fn flush(buffer: &mut WatchBuffer, ctx: &WatcherContext, kind: &str) {
    if let Some(updates) = buffer.take_batch() {
        let event = OrbitEvent::ResourceBatchUpdated {
            kind: kind.to_string(),
            cluster_id: ctx.reporter.cluster_id.clone(),
            updates,
        };
        if let Err(e) = Bridge::send_event(&ctx.reporter.writer, &ctx.reporter.ipc_token, &event).await {
            tracing::debug!(error = %e, "Failed to send batch update");
        }
    }
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

enum StreamStep<K> {
    Item { action: &'static str, obj: K },
    SyncDone,
    Continue,
    Reconnect,
}

fn process_event<K>(
    event: Option<Result<watcher::Event<K>, watcher::Error>>,
) -> (StreamStep<K>, Option<watcher::Error>) {
    match event {
        Some(Ok(watcher::Event::Apply(obj))) | Some(Ok(watcher::Event::InitApply(obj))) => {
            (StreamStep::Item { action: ACTION_APPLIED, obj }, None)
        }
        Some(Ok(watcher::Event::Delete(obj))) => {
            (StreamStep::Item { action: ACTION_DELETED, obj }, None)
        }
        Some(Ok(watcher::Event::InitDone)) => (StreamStep::SyncDone, None),
        Some(Ok(_)) => (StreamStep::Continue, None),
        Some(Err(e)) => (StreamStep::Continue, Some(e)),
        None => (StreamStep::Reconnect, None),
    }
}

struct WatcherRuntime {
    has_failed: bool,
}

async fn on_success(ctx: &WatcherContext, watcher_id: &str, runtime: &mut WatcherRuntime) {
    if let Some(state) = ctx.tracker.record_success(watcher_id, std::time::Instant::now()) {
        ctx.reporter.report(state, ctx.tracker.failing_count()).await;
    }
    runtime.has_failed = false;
}

async fn on_failure(
    ctx: &WatcherContext,
    watcher_id: &str,
    kind: &str,
    err: &watcher::Error,
    runtime: &mut WatcherRuntime,
) {
    let failure_class = classify_watcher_error(err);
    let transition = ctx.tracker.record_failure(watcher_id, failure_class, std::time::Instant::now());

    if !runtime.has_failed {
        tracing::warn!(kind = %kind, error = %err, "Watcher error (backoff active)");
        runtime.has_failed = true;
    } else {
        tracing::debug!(kind = %kind, error = %err, "Watcher retry failed");
    }

    if let Some(state) = transition {
        ctx.reporter.report(state, ctx.tracker.failing_count()).await;
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

    let mut runtime = WatcherRuntime { has_failed: false };

    'outer: loop {
        let api = Api::<K>::all(ctx.client.clone());
        let mut stream = watcher(api, watcher::Config::default())
            .default_backoff()
            .boxed();

        let mut buffer = WatchBuffer::new();
        let mut flush_interval = tokio::time::interval(COALESCE_INTERVAL);
        flush_interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

        loop {
            tokio::select! {
                res = ctx.cancel_rx.changed() => {
                    if res.is_err() || *ctx.cancel_rx.borrow() {
                        tracing::info!(kind = %kind, "Stopping watcher");
                        flush(&mut buffer, &ctx, &kind).await;
                        break 'outer;
                    }
                }
                _ = flush_interval.tick() => {
                    flush(&mut buffer, &ctx, &kind).await;
                }
                event = stream.next() => {
                    let (step, err) = process_event(event);
                    if let Some(ref e) = err {
                        on_failure(&ctx, &watcher_id, &kind, e, &mut runtime).await;
                    }
                    match step {
                        StreamStep::Item { action, obj } => {
                            on_success(&ctx, &watcher_id, &mut runtime).await;
                            let key = get_resource_key(&obj);
                            let mapped = mapper(&obj);
                            if let Ok(data) = serde_json::to_value(&mapped) {
                                tracing::debug!(kind = %kind, action = %action, data = ?data, "Watcher event processed");
                                let update = ResourceUpdate {
                                    action: action.to_string(),
                                    data,
                                };
                                if buffer.upsert(key, update) {
                                    flush(&mut buffer, &ctx, &kind).await;
                                }
                            }
                        }
                        StreamStep::SyncDone => {
                            flush(&mut buffer, &ctx, &kind).await;
                            on_success(&ctx, &watcher_id, &mut runtime).await;
                            tracing::info!(kind = %kind, "Watcher initial sync done");
                        }
                        StreamStep::Continue => {}
                        StreamStep::Reconnect => {
                            tracing::info!(kind = %kind, "Watcher stream ended, reconnecting...");
                            flush(&mut buffer, &ctx, &kind).await;
                            break;
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
    use crate::ipc::emitter::{set_global_emitter, MockEventEmitter};
    use crate::kubernetes::watch_health::WATCHER_GIVE_UP_AFTER;

    const HOT_LOOP_FAILURE_BOUND: usize = 50;

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

    #[test]
    fn watch_buffer_coalesces_same_key() {
        let mut buf = WatchBuffer::new();
        let u1 = ResourceUpdate {
            action: ACTION_APPLIED.into(),
            data: serde_json::json!({"v": 1}),
        };
        let u2 = ResourceUpdate {
            action: ACTION_APPLIED.into(),
            data: serde_json::json!({"v": 2}),
        };
        assert!(!buf.upsert("k1".into(), u1));
        assert!(!buf.upsert("k1".into(), u2));

        let batch = buf.take_batch().expect("batch should be Some");
        assert_eq!(batch.len(), 1);
        assert_eq!(batch[0].data, serde_json::json!({"v": 2}));
    }

    #[test]
    fn watch_buffer_signals_flush_at_max() {
        let mut buf = WatchBuffer::new();
        for i in 0..(COALESCE_BUFFER_MAX - 1) {
            let u = ResourceUpdate {
                action: ACTION_APPLIED.into(),
                data: serde_json::json!(i),
            };
            assert!(!buf.upsert(format!("k{i}"), u));
        }
        let last = ResourceUpdate {
            action: ACTION_APPLIED.into(),
            data: serde_json::json!("last"),
        };
        assert!(buf.upsert("k_last".into(), last));
    }

    #[test]
    fn watch_buffer_take_batch_clears_index_and_returns_none_when_empty() {
        let mut buf = WatchBuffer::new();
        assert!(buf.take_batch().is_none());

        buf.upsert(
            "k1".into(),
            ResourceUpdate {
                action: ACTION_APPLIED.into(),
                data: serde_json::json!(1),
            },
        );
        assert!(buf.take_batch().is_some());
        assert!(buf.take_batch().is_none());
    }

    #[tokio::test]
    async fn connection_reporter_emits_event() {
        let _guard = crate::ipc::emitter::TEST_EMITTER_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let mock_emitter = Arc::new(MockEventEmitter::new());
        set_global_emitter(mock_emitter.clone());

        let writer = Arc::new(Mutex::new(WsWriter::Emitter));
        let reporter = ConnectionReporter::new(
            writer,
            "token-abc".into(),
            Some("cluster-123".into()),
        );

        reporter.report(WatcherConnectionState::Degraded, 2).await;

        let emitted = mock_emitter.get_emitted_events().await;
        assert_eq!(emitted.len(), 1);
        let (name, payload) = &emitted[0];
        assert_eq!(name, "watcherConnectionChanged");
        assert_eq!(payload["clusterId"], "cluster-123");
        assert_eq!(payload["state"], "degraded");
        assert_eq!(payload["failingWatchers"], 2);
    }

    #[tokio::test]
    async fn test_unreachable_api_server_does_not_hot_loop() {
        let mut config = kube::Config::new("http://127.0.0.1:1".parse().unwrap());
        config.connect_timeout = Some(Duration::from_millis(200));
        let client = kube::Client::try_from(config).unwrap();
        let (tx, _rx) = tokio::sync::watch::channel(false);
        let (cancel_tx, cancel_rx) = tokio::sync::watch::channel(false);
        let tracker = crate::kubernetes::watch_health::WatcherHealthTracker::new(
            WATCHER_GIVE_UP_AFTER,
            tx,
        );
        let writer = Arc::new(Mutex::new(WsWriter::Emitter));
        let reporter = ConnectionReporter::new(
            writer,
            "test".to_string(),
            Some("test-cluster".to_string()),
        );
        let ctx = WatcherContext {
            client,
            reporter,
            tracker: tracker.clone(),
            cancel_rx,
        };

        let handle = tokio::spawn(watch_resource::<Pod, _, _>(
            "Pod".to_string(),
            ctx,
            |_| (),
        ));

        // 1-second window (satisfies AC-12: runs <= 1.5s).
        // With a 200ms connect timeout and default backoff, failures occur but stay far below
        // the unthrottled hot loop bound of 50.
        tokio::time::sleep(Duration::from_secs(1)).await;
        let _ = cancel_tx.send(true);
        let _ = handle.await;

        assert!(tracker.total_failure_records() >= 1);
        assert!(
            tracker.total_failure_records() <= HOT_LOOP_FAILURE_BOUND,
            "Failure count {} exceeded hot-loop bound {}",
            tracker.total_failure_records(),
            HOT_LOOP_FAILURE_BOUND
        );
    }
}
