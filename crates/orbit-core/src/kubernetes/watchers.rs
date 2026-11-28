use std::collections::HashMap;
use std::fmt::Debug;
use std::future::Future;
use std::hash::{DefaultHasher, Hasher};
use std::io;
use std::sync::Arc;
use std::time::{Duration, Instant};

use futures_util::StreamExt;
use kube::runtime::{WatchStreamExt, watcher};
use kube::{Api, Client, Resource};
use serde::Serialize;
use tokio::sync::watch;

use crate::kubernetes::watch_error::classify_watcher_error;
use crate::kubernetes::watch_health::WatcherHealthTracker;

pub const WATCH_COALESCE_INTERVAL: Duration = Duration::from_millis(200);
pub const WATCH_COALESCE_BUFFER_MAX: usize = 256;
pub const WATCH_STREAM_RETRY_INTERVAL: Duration = Duration::from_secs(3);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WatchAction {
    Applied,
    Deleted,
}

#[derive(Debug, Clone, PartialEq)]
pub struct WatchItem<T> {
    pub key: String,
    pub action: WatchAction,
    pub item: T,
}

#[derive(Debug, Clone, PartialEq)]
pub enum WatchUpdate<T> {
    /// Atomic full-snapshot replacement emitted on Init -> InitApply* -> InitDone
    /// (both initial sync and post-reconnect relist) to eliminate ghost entries.
    Reset(Vec<(String, T)>),
    /// Coalesced incremental upserts/deletes emitted during steady-state watching.
    Delta(Vec<WatchItem<T>>),
}

/// Canonical resource key: `"namespace/name"` for namespaced resources, `"name"` for cluster-scoped.
pub fn resource_key<K: Resource>(obj: &K) -> String {
    let name = obj.meta().name.as_deref().unwrap_or_default();
    match obj.meta().namespace.as_deref() {
        Some(ns) if !ns.is_empty() => format!("{ns}/{name}"),
        _ => name.to_string(),
    }
}

/// Composite resource key for multi-kind views: `"{prefix}:{namespace}/{name}"` or `"{prefix}:{name}"`.
pub fn prefixed_resource_key<K: Resource>(prefix: &str, obj: &K) -> String {
    format!("{prefix}:{}", resource_key(obj))
}

/// Stack-only `io::Write` sink that hashes serialized JSON bytes directly into a `DefaultHasher`
/// without heap allocations or intermediate `serde_json::Value` trees.
struct HashWriter(DefaultHasher);

impl HashWriter {
    fn new() -> Self {
        Self(DefaultHasher::new())
    }

    fn finish(&self) -> u64 {
        self.0.finish()
    }
}

impl io::Write for HashWriter {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.0.write(buf);
        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

/// Tracks last-emitted payload fingerprint per resource key without JSON heap allocations.
#[derive(Debug, Default)]
pub struct TypedEmitDedup {
    last: HashMap<String, u64>,
}

impl TypedEmitDedup {
    pub fn new() -> Self {
        Self {
            last: HashMap::new(),
        }
    }

    fn fingerprint<T: Serialize>(item: &T) -> Option<u64> {
        let mut writer = HashWriter::new();
        serde_json::to_writer(&mut writer, item).ok()?;
        Some(writer.finish())
    }

    /// Returns `true` if the update should be emitted (changed payload or `Deleted`).
    pub fn should_emit<T: Serialize>(&mut self, key: &str, action: WatchAction, item: &T) -> bool {
        if action == WatchAction::Deleted {
            self.last.remove(key);
            return true;
        }

        let Some(hash) = Self::fingerprint(item) else {
            return true;
        };

        if self.last.get(key) == Some(&hash) {
            return false;
        }

        self.last.insert(key.to_string(), hash);
        true
    }

    /// Replaces the dedup table with hashes computed from an atomic Reset snapshot.
    pub fn reset_from_snapshot<T: Serialize>(&mut self, items: &[(String, T)]) {
        self.last.clear();
        self.last.reserve(items.len());
        for (key, item) in items {
            if let Some(hash) = Self::fingerprint(item) {
                self.last.insert(key.clone(), hash);
            }
        }
    }

    pub fn contains_key(&self, key: &str) -> bool {
        self.last.contains_key(key)
    }
}

/// O(1) key-coalescing buffer for steady-state watch deltas.
#[derive(Debug)]
pub struct TypedWatchBuffer<T> {
    items: Vec<WatchItem<T>>,
    key_index: HashMap<String, usize>,
    max_items: usize,
}

impl<T> Default for TypedWatchBuffer<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> TypedWatchBuffer<T> {
    pub fn new() -> Self {
        Self::with_max(WATCH_COALESCE_BUFFER_MAX)
    }

    pub fn with_max(max_items: usize) -> Self {
        Self {
            items: Vec::new(),
            key_index: HashMap::new(),
            max_items: max_items.max(1),
        }
    }

    /// Inserts or overwrites the latest item for `item.key` in O(1).
    /// Returns `true` when the buffer reached its flush capacity.
    pub fn upsert(&mut self, item: WatchItem<T>) -> bool {
        if let Some(&idx) = self.key_index.get(&item.key) {
            self.items[idx] = item;
        } else {
            self.key_index.insert(item.key.clone(), self.items.len());
            self.items.push(item);
        }
        self.items.len() >= self.max_items
    }

    pub fn take_batch(&mut self) -> Option<Vec<WatchItem<T>>> {
        if self.items.is_empty() {
            None
        } else {
            self.key_index.clear();
            Some(std::mem::take(&mut self.items))
        }
    }

    pub fn clear(&mut self) {
        self.items.clear();
        self.key_index.clear();
    }
}

/// Accumulates `Init` -> `InitApply*` -> `InitDone` into an atomic `WatchUpdate::Reset`.
#[derive(Debug)]
pub struct InitAccumulator<T> {
    active: bool,
    items: Vec<(String, T)>,
    key_index: HashMap<String, usize>,
}

impl<T> Default for InitAccumulator<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> InitAccumulator<T> {
    pub fn new() -> Self {
        Self {
            active: false,
            items: Vec::new(),
            key_index: HashMap::new(),
        }
    }

    pub fn begin(&mut self) {
        self.active = true;
        self.items.clear();
        self.key_index.clear();
    }

    pub fn push(&mut self, key: String, item: T) {
        self.active = true;
        if let Some(&idx) = self.key_index.get(&key) {
            self.items[idx] = (key, item);
        } else {
            self.key_index.insert(key.clone(), self.items.len());
            self.items.push((key, item));
        }
    }

    pub fn finish(&mut self, dedup: &mut TypedEmitDedup) -> Option<WatchUpdate<T>>
    where
        T: Serialize,
    {
        if !self.active {
            return None;
        }
        self.active = false;
        self.key_index.clear();
        let snapshot = std::mem::take(&mut self.items);
        dedup.reset_from_snapshot(&snapshot);
        Some(WatchUpdate::Reset(snapshot))
    }
}

async fn flush_buffer<T, E, Fut>(buffer: &mut TypedWatchBuffer<T>, emit: &mut E) -> bool
where
    E: FnMut(WatchUpdate<T>) -> Fut,
    Fut: Future<Output = bool>,
{
    if let Some(batch) = buffer.take_batch() {
        emit(WatchUpdate::Delta(batch)).await
    } else {
        true
    }
}

/// Shared connection, health, cancellation, and key-prefix context for typed watchers.
pub struct TypedWatcherContext {
    pub watcher_id: String,
    pub client: Client,
    pub tracker: Arc<WatcherHealthTracker>,
    pub cancel_rx: watch::Receiver<bool>,
    pub key_prefix: Option<&'static str>,
}

/// Streams typed Kubernetes watch updates with mandatory `.default_backoff()`, atomic `Reset` on
/// initial sync/relist, zero-allocation dedup, and 200ms / 256-item delta coalescing.
///
/// The `emit` callback returns `false` when the downstream consumer has closed, stopping the task.
pub async fn watch_resource_typed<K, T, M, E, Fut>(
    ctx: TypedWatcherContext,
    mapper: M,
    emit: E,
) where
    K: Resource + Clone + serde::de::DeserializeOwned + Debug + Send + Sync + 'static,
    K::DynamicType: Default,
    T: Serialize + Send + 'static,
    M: Fn(&K) -> T + Send + Sync + 'static,
    E: FnMut(WatchUpdate<T>) -> Fut + Send,
    Fut: Future<Output = bool> + Send,
{
    watch_resource_typed_inner(ctx, move |obj| Some(mapper(obj)), None::<fn(&K) -> T>, emit).await;
}

/// Variant of `watch_resource_typed` for mappers that filter resources (e.g. namespaces carrying
/// `pod-security.kubernetes.io/*` labels). When a previously emitted resource no longer matches,
/// a `Deleted` delta is emitted using `delete_placeholder`.
pub async fn watch_resource_typed_filtered<K, T, M, D, E, Fut>(
    ctx: TypedWatcherContext,
    mapper: M,
    delete_placeholder: D,
    emit: E,
) where
    K: Resource + Clone + serde::de::DeserializeOwned + Debug + Send + Sync + 'static,
    K::DynamicType: Default,
    T: Serialize + Send + 'static,
    M: Fn(&K) -> Option<T> + Send + Sync + 'static,
    D: Fn(&K) -> T + Send + Sync + 'static,
    E: FnMut(WatchUpdate<T>) -> Fut + Send,
    Fut: Future<Output = bool> + Send,
{
    watch_resource_typed_inner(ctx, mapper, Some(delete_placeholder), emit).await;
}

async fn watch_resource_typed_inner<K, T, M, D, E, Fut>(
    ctx: TypedWatcherContext,
    mapper: M,
    delete_placeholder: Option<D>,
    mut emit: E,
) where
    K: Resource + Clone + serde::de::DeserializeOwned + Debug + Send + Sync + 'static,
    K::DynamicType: Default,
    T: Serialize + Send + 'static,
    M: Fn(&K) -> Option<T> + Send + Sync + 'static,
    D: Fn(&K) -> T + Send + Sync + 'static,
    E: FnMut(WatchUpdate<T>) -> Fut + Send,
    Fut: Future<Output = bool> + Send,
{
    let TypedWatcherContext {
        watcher_id,
        client,
        tracker,
        mut cancel_rx,
        key_prefix,
    } = ctx;
    let mut has_failed = false;

    'outer: loop {
        if *cancel_rx.borrow() {
            break 'outer;
        }

        let api = Api::<K>::all(client.clone());
        let mut stream = watcher(api, watcher::Config::default())
            .default_backoff()
            .boxed();

        let mut buffer = TypedWatchBuffer::<T>::new();
        let mut dedup = TypedEmitDedup::new();
        let mut init_acc = InitAccumulator::<T>::new();
        let mut flush_interval = tokio::time::interval(WATCH_COALESCE_INTERVAL);
        flush_interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

        loop {
            tokio::select! {
                res = cancel_rx.changed() => {
                    if res.is_err() || *cancel_rx.borrow() {
                        let _ = flush_buffer(&mut buffer, &mut emit).await;
                        break 'outer;
                    }
                }
                _ = flush_interval.tick() => {
                    if !flush_buffer(&mut buffer, &mut emit).await {
                        break 'outer;
                    }
                }
                event = stream.next() => {
                    match event {
                        Some(Ok(watcher::Event::Init)) => {
                            buffer.clear();
                            init_acc.begin();
                        }
                        Some(Ok(watcher::Event::InitApply(obj))) => {
                            tracker.record_success(&watcher_id, Instant::now());
                            has_failed = false;
                            if let Some(mapped) = mapper(&obj) {
                                let key = match key_prefix {
                                    Some(prefix) => prefixed_resource_key(prefix, &obj),
                                    None => resource_key(&obj),
                                };
                                init_acc.push(key, mapped);
                            }
                        }
                        Some(Ok(watcher::Event::InitDone)) => {
                            tracker.record_success(&watcher_id, Instant::now());
                            has_failed = false;
                            if let Some(reset) = init_acc.finish(&mut dedup)
                                && !emit(reset).await
                            {
                                break 'outer;
                            }
                        }
                        Some(Ok(watcher::Event::Apply(obj))) => {
                            tracker.record_success(&watcher_id, Instant::now());
                            has_failed = false;
                            let key = match key_prefix {
                                Some(prefix) => prefixed_resource_key(prefix, &obj),
                                None => resource_key(&obj),
                            };
                            match mapper(&obj) {
                                Some(mapped) => {
                                    if !dedup.should_emit(&key, WatchAction::Applied, &mapped) {
                                        continue;
                                    }
                                    if buffer.upsert(WatchItem {
                                        key,
                                        action: WatchAction::Applied,
                                        item: mapped,
                                    }) && !flush_buffer(&mut buffer, &mut emit).await
                                    {
                                        break 'outer;
                                    }
                                }
                                None => {
                                    if dedup.contains_key(&key)
                                        && let Some(ref make_deleted) = delete_placeholder
                                    {
                                        let placeholder = make_deleted(&obj);
                                        if dedup.should_emit(&key, WatchAction::Deleted, &placeholder)
                                            && buffer.upsert(WatchItem {
                                                key,
                                                action: WatchAction::Deleted,
                                                item: placeholder,
                                            })
                                            && !flush_buffer(&mut buffer, &mut emit).await
                                        {
                                            break 'outer;
                                        }
                                    }
                                }
                            }
                        }
                        Some(Ok(watcher::Event::Delete(obj))) => {
                            tracker.record_success(&watcher_id, Instant::now());
                            has_failed = false;
                            let key = match key_prefix {
                                Some(prefix) => prefixed_resource_key(prefix, &obj),
                                None => resource_key(&obj),
                            };
                            let mapped = mapper(&obj).or_else(|| {
                                delete_placeholder.as_ref().map(|make_deleted| make_deleted(&obj))
                            });
                            let Some(mapped) = mapped else {
                                continue;
                            };
                            if !dedup.should_emit(&key, WatchAction::Deleted, &mapped) {
                                continue;
                            }
                            if buffer.upsert(WatchItem {
                                key,
                                action: WatchAction::Deleted,
                                item: mapped,
                            }) && !flush_buffer(&mut buffer, &mut emit).await
                            {
                                break 'outer;
                            }
                        }
                        Some(Err(err)) => {
                            let class = classify_watcher_error(&err);
                            tracker.record_failure(&watcher_id, class, Instant::now());
                            if !has_failed {
                                tracing::warn!(watcher_id = %watcher_id, error = %err, "Typed watcher error (backoff active)");
                                has_failed = true;
                            } else {
                                tracing::debug!(watcher_id = %watcher_id, error = %err, "Typed watcher retry failed");
                            }
                        }
                        None => {
                            if !flush_buffer(&mut buffer, &mut emit).await {
                                break 'outer;
                            }
                            break;
                        }
                    }
                }
            }
        }

        tokio::select! {
            res = cancel_rx.changed() => {
                if res.is_err() || *cancel_rx.borrow() {
                    break 'outer;
                }
            }
            _ = tokio::time::sleep(WATCH_STREAM_RETRY_INTERVAL) => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use k8s_openapi::api::core::v1::{Node, Pod};
    use k8s_openapi::apimachinery::pkg::apis::meta::v1::ObjectMeta;

    use crate::kubernetes::models::PodInfo;
    use crate::kubernetes::watch_health::WATCHER_GIVE_UP_AFTER;

    const HOT_LOOP_FAILURE_BOUND: usize = 50;

    fn sample_pod(name: &str, status: &str) -> PodInfo {
        PodInfo {
            name: name.to_string(),
            namespace: "prod".to_string(),
            status: status.to_string(),
            age: "1m".to_string(),
            cpu: None,
            memory: None,
            cpu_cores: None,
            memory_bytes: None,
            cpu_pct: None,
            memory_pct: None,
            node: Some("node-1".to_string()),
            restarts: 0,
            images: vec!["nginx:latest".to_string()],
            labels: Default::default(),
            annotations: Default::default(),
            ip: Some("10.0.0.1".to_string()),
            node_ip: Some("192.168.1.1".to_string()),
            controlled_by: None,
            qos_class: Some("Burstable".to_string()),
            containers: Vec::new(),
        }
    }

    #[test]
    fn resource_key_uses_namespace_and_name_or_cluster_name() {
        let pod = Pod {
            metadata: ObjectMeta {
                name: Some("api-0".to_string()),
                namespace: Some("prod".to_string()),
                ..Default::default()
            },
            ..Default::default()
        };
        assert_eq!(resource_key(&pod), "prod/api-0");
        assert_eq!(prefixed_resource_key("Pod", &pod), "Pod:prod/api-0");

        let node = Node {
            metadata: ObjectMeta {
                name: Some("node-1".to_string()),
                namespace: None,
                ..Default::default()
            },
            ..Default::default()
        };
        assert_eq!(resource_key(&node), "node-1");
        assert_eq!(prefixed_resource_key("Node", &node), "Node:node-1");
    }

    #[test]
    fn typed_emit_dedup_suppresses_identical_and_emits_changed_or_deleted() {
        let mut dedup = TypedEmitDedup::new();
        let pending = sample_pod("api-0", "Pending");
        let running = sample_pod("api-0", "Running");

        assert!(dedup.should_emit("prod/api-0", WatchAction::Applied, &pending));
        assert!(!dedup.should_emit("prod/api-0", WatchAction::Applied, &pending));
        assert!(dedup.should_emit("prod/api-0", WatchAction::Applied, &running));
        assert!(!dedup.should_emit("prod/api-0", WatchAction::Applied, &running));

        assert!(dedup.should_emit("prod/api-0", WatchAction::Deleted, &running));
        assert!(dedup.should_emit("prod/api-0", WatchAction::Applied, &running));
    }

    #[test]
    fn typed_watch_buffer_coalesces_burst_for_same_key_in_o1() {
        let mut buf = TypedWatchBuffer::new();
        for i in 0..1_000 {
            let key_idx = i % 5;
            let key = format!("prod/api-{key_idx}");
            let pod = sample_pod(&format!("api-{key_idx}"), &format!("Step-{i}"));
            let _ = buf.upsert(WatchItem {
                key,
                action: WatchAction::Applied,
                item: pod,
            });
        }

        let Some(batch) = buf.take_batch() else {
            panic!("expected coalesced batch");
        };
        assert_eq!(batch.len(), 5);
        for item in &batch {
            let suffix = item.key.strip_prefix("prod/api-").unwrap_or_default();
            let expected_step = match suffix {
                "0" => "Step-995",
                "1" => "Step-996",
                "2" => "Step-997",
                "3" => "Step-998",
                "4" => "Step-999",
                _ => "",
            };
            assert_eq!(item.item.status, expected_step);
        }
        assert!(buf.take_batch().is_none());
    }

    #[test]
    fn typed_watch_buffer_signals_flush_at_capacity() {
        let mut buf = TypedWatchBuffer::new();
        for i in 0..(WATCH_COALESCE_BUFFER_MAX - 1) {
            let full = buf.upsert(WatchItem {
                key: format!("prod/pod-{i}"),
                action: WatchAction::Applied,
                item: sample_pod(&format!("pod-{i}"), "Running"),
            });
            assert!(!full);
        }

        let full = buf.upsert(WatchItem {
            key: "prod/pod-last".to_string(),
            action: WatchAction::Applied,
            item: sample_pod("pod-last", "Running"),
        });
        assert!(full);
    }

    #[test]
    fn init_accumulator_emits_atomic_reset_and_clears_dedup() {
        let mut dedup = TypedEmitDedup::new();
        let mut acc = InitAccumulator::new();
        acc.begin();
        for i in 0..3 {
            let name = format!("pod-{i}");
            acc.push(format!("prod/{name}"), sample_pod(&name, "Running"));
        }

        let Some(WatchUpdate::Reset(items)) = acc.finish(&mut dedup) else {
            panic!("expected Reset update");
        };
        assert_eq!(items.len(), 3);

        // Primed dedup suppresses identical steady-state applies after InitDone.
        assert!(!dedup.should_emit(
            "prod/pod-0",
            WatchAction::Applied,
            &sample_pod("pod-0", "Running")
        ));
        assert!(dedup.should_emit(
            "prod/pod-0",
            WatchAction::Applied,
            &sample_pod("pod-0", "CrashLoopBackOff")
        ));
    }

    #[tokio::test]
    async fn unreachable_api_server_respects_default_backoff_and_does_not_hot_loop() {
        let Ok(uri) = "http://127.0.0.1:1".parse() else {
            panic!("valid test URI");
        };
        let mut config = kube::Config::new(uri);
        config.connect_timeout = Some(Duration::from_millis(200));
        let Ok(client) = Client::try_from(config) else {
            panic!("valid kube client");
        };

        let (health_tx, _health_rx) = watch::channel(false);
        let (cancel_tx, cancel_rx) = watch::channel(false);
        let tracker = WatcherHealthTracker::new(WATCHER_GIVE_UP_AFTER, health_tx);

        let handle = tokio::spawn(watch_resource_typed::<Pod, _, _, _, _>(
            TypedWatcherContext {
                watcher_id: "pods:Pod".to_string(),
                client,
                tracker: tracker.clone(),
                cancel_rx,
                key_prefix: None,
            },
            |_| (),
            |_update| async { true },
        ));

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
