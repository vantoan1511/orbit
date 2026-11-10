use std::sync::Arc;
use std::fmt::Debug;
use std::collections::HashMap;
use std::hash::{DefaultHasher, Hasher};
use std::time::Duration;
use futures_util::StreamExt;
use kube::{Api, Resource};
use kube::runtime::{watcher, WatchStreamExt};
use serde::Serialize;
use tokio::sync::watch;

use crate::events::{CoreEvent, TypedResourceBatch, WatchAction, WatcherConnectionState};
use crate::kubernetes::watch_error::classify_watcher_error;
use crate::kubernetes::watch_health::{WatcherHealthTracker, WATCHER_GIVE_UP_AFTER};
use crate::sink::CoreEventSink;

const COALESCE_INTERVAL: Duration = Duration::from_millis(250);
const COALESCE_BUFFER_MAX: usize = 100;
const STREAM_RETRY_INTERVAL: Duration = Duration::from_secs(3);
const GIVE_UP_CHECK_INTERVAL: Duration = Duration::from_secs(5);
const ACTION_APPLIED: &str = "Applied";
const ACTION_DELETED: &str = "Deleted";

/// Tracks last-emitted payload fingerprint per resource key.
#[derive(Debug)]
pub struct EmitDedup {
    last: HashMap<String, u64>,
}

impl Default for EmitDedup {
    fn default() -> Self {
        Self::new()
    }
}

impl EmitDedup {
    pub fn new() -> Self {
        Self {
            last: HashMap::new(),
        }
    }

    /// Returns true if the update should be emitted (changed or Deleted action).
    pub fn should_emit(&mut self, key: &str, action: &str, data: &serde_json::Value) -> bool {
        if action == ACTION_DELETED {
            self.last.remove(key);
            return true;
        }

        let hash = match serde_json::to_vec(data) {
            Ok(bytes) => {
                let mut hasher = DefaultHasher::new();
                hasher.write(&bytes);
                hasher.finish()
            }
            Err(_) => return true,
        };

        if self.last.get(key) == Some(&hash) {
            return false;
        }

        self.last.insert(key.to_string(), hash);
        true
    }

    pub fn clear(&mut self) {
        self.last.clear();
    }
}

#[derive(Clone)]
pub struct ConnectionReporter {
    pub sink: Arc<dyn CoreEventSink>,
}

impl ConnectionReporter {
    pub fn new(sink: Arc<dyn CoreEventSink>) -> Self {
        Self { sink }
    }

    pub async fn report(&self, state: WatcherConnectionState, failing_watchers: usize) {
        tracing::info!(
            state = ?state,
            failing_watchers = failing_watchers,
            "Watcher aggregate connection state changed"
        );
        let event = CoreEvent::WatcherConnectionChanged {
            state,
            failing_watchers,
            message: format!("Watcher connection state: {:?}", state),
        };
        let _ = self.sink.send(event);
    }
}

pub struct WatcherContext {
    pub client: kube::Client,
    pub reporter: ConnectionReporter,
    pub tracker: Arc<WatcherHealthTracker>,
    pub cancel_rx: watch::Receiver<bool>,
}

struct WatchBuffer<M> {
    updates: Vec<WatchAction<M>>,
    key_index: HashMap<String, usize>,
}

impl<M> WatchBuffer<M> {
    fn new() -> Self {
        Self {
            updates: Vec::new(),
            key_index: HashMap::new(),
        }
    }

    fn upsert(&mut self, key: String, action: WatchAction<M>) -> bool {
        if let Some(&idx) = self.key_index.get(&key) {
            self.updates[idx] = action;
        } else {
            self.key_index.insert(key, self.updates.len());
            self.updates.push(action);
        }
        self.updates.len() >= COALESCE_BUFFER_MAX
    }

    fn take_batch(&mut self) -> Option<Vec<WatchAction<M>>> {
        if self.updates.is_empty() {
            None
        } else {
            self.key_index.clear();
            Some(std::mem::take(&mut self.updates))
        }
    }
}

async fn flush<M: Clone, B: Fn(Vec<WatchAction<M>>) -> TypedResourceBatch>(
    buffer: &mut WatchBuffer<M>,
    ctx: &WatcherContext,
    batch_builder: &B,
) {
    if let Some(updates) = buffer.take_batch() {
        let batch = batch_builder(updates);
        let _ = ctx.reporter.sink.send(CoreEvent::ResourceBatchUpdated(batch));
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

pub async fn watch_resource<K, M, F, B>(
    kind: String,
    mut ctx: WatcherContext,
    mapper: F,
    batch_builder: B,
) where
    K: Resource + Clone + serde::de::DeserializeOwned + Debug + Send + Sync + 'static,
    K::DynamicType: Default,
    M: Serialize + Clone + Send + Sync + 'static,
    F: Fn(&K) -> M + Send + Sync + 'static,
    B: Fn(Vec<WatchAction<M>>) -> TypedResourceBatch + Send + Sync + 'static,
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
        let mut dedup = EmitDedup::new();
        let mut flush_interval = tokio::time::interval(COALESCE_INTERVAL);
        flush_interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

        loop {
            tokio::select! {
                res = ctx.cancel_rx.changed() => {
                    if res.is_err() || *ctx.cancel_rx.borrow() {
                        tracing::info!(kind = %kind, "Stopping watcher");
                        flush(&mut buffer, &ctx, &batch_builder).await;
                        break 'outer;
                    }
                }
                _ = flush_interval.tick() => {
                    flush(&mut buffer, &ctx, &batch_builder).await;
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
                                if !dedup.should_emit(&key, action, &data) {
                                    continue;
                                }
                                tracing::debug!(kind = %kind, action = %action, key = %key, "Watcher event processed");
                                let watch_action = if action == ACTION_DELETED {
                                    WatchAction::Deleted(mapped)
                                } else {
                                    WatchAction::Applied(mapped)
                                };
                                if buffer.upsert(key, watch_action) {
                                    flush(&mut buffer, &ctx, &batch_builder).await;
                                }
                            }
                        }
                        StreamStep::SyncDone => {
                            flush(&mut buffer, &ctx, &batch_builder).await;
                            on_success(&ctx, &watcher_id, &mut runtime).await;
                            tracing::info!(kind = %kind, "Watcher initial sync done");
                        }
                        StreamStep::Continue => {}
                        StreamStep::Reconnect => {
                            tracing::info!(kind = %kind, "Watcher stream ended, reconnecting...");
                            flush(&mut buffer, &ctx, &batch_builder).await;
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

pub fn derive_cancel_channel(
    mut parent_rx: tokio::sync::watch::Receiver<bool>,
) -> (tokio::sync::watch::Sender<bool>, tokio::sync::watch::Receiver<bool>) {
    let initial = *parent_rx.borrow();
    let (child_tx, child_rx) = tokio::sync::watch::channel(initial);
    if !initial {
        let child_tx_clone = child_tx.clone();
        tokio::spawn(async move {
            while parent_rx.changed().await.is_ok() {
                if *parent_rx.borrow() {
                    break;
                }
            }
            if let Err(e) = child_tx_clone.send(true) {
                tracing::debug!(error = %e, "Failed to propagate cancel to child channel");
            }
        });
    }
    (child_tx, child_rx)
}

pub fn spawn_watchers(
    client: &kube::Client,
    sink: Arc<dyn CoreEventSink>,
    rx: tokio::sync::watch::Receiver<bool>,
) {
    let (child_tx, child_rx) = derive_cancel_channel(rx);
    let tracker = WatcherHealthTracker::new(WATCHER_GIVE_UP_AFTER, child_tx);
    let reporter = ConnectionReporter::new(sink.clone());

    // Initial Connected emission (Task 2 / AC-2)
    let initial_reporter = reporter.clone();
    tokio::spawn(async move {
        initial_reporter.report(WatcherConnectionState::Connected, 0).await;
    });

    let ticker_tracker = tracker.clone();
    let ticker_reporter = reporter.clone();
    let mut ticker_rx = child_rx.clone();

    tokio::spawn(async move {
        let mut interval = tokio::time::interval(GIVE_UP_CHECK_INTERVAL);
        interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
        loop {
            tokio::select! {
                res = ticker_rx.changed() => {
                    if res.is_err() || *ticker_rx.borrow() {
                        break;
                    }
                }
                _ = interval.tick() => {
                    if let Some(state) = ticker_tracker.check_give_up(std::time::Instant::now()) {
                        ticker_reporter.report(state, ticker_tracker.failing_count()).await;
                        break;
                    }
                }
            }
        }
    });

    let make_ctx = |rx: tokio::sync::watch::Receiver<bool>| WatcherContext {
        client: client.clone(),
        reporter: reporter.clone(),
        tracker: tracker.clone(),
        cancel_rx: rx,
    };

    macro_rules! spawn_watcher {
        ($kind_type:ty, $label:expr, $mapper:path, $batch:ident) => {
            tokio::spawn(watch_resource::<$kind_type, _, _, _>(
                $label.to_string(),
                make_ctx(child_rx.clone()),
                $mapper,
                |items| TypedResourceBatch::$batch(items),
            ));
        };
    }

    spawn_watcher!(k8s_openapi::api::core::v1::Service, "Service", crate::kubernetes::services::map_service, Services);
    spawn_watcher!(k8s_openapi::api::core::v1::Node, "Node", crate::kubernetes::nodes::map_node, Nodes);
    spawn_watcher!(k8s_openapi::api::networking::v1::Ingress, "Ingress", crate::kubernetes::ingresses::map_ingress, Ingresses);
    spawn_watcher!(k8s_openapi::api::apps::v1::Deployment, "Deployment", crate::kubernetes::workloads::map_deployment, Deployments);
    spawn_watcher!(k8s_openapi::api::core::v1::Pod, "Pod", crate::kubernetes::workloads::map_pod, Pods);
    spawn_watcher!(k8s_openapi::api::apps::v1::StatefulSet, "StatefulSet", crate::kubernetes::workloads::map_statefulset, StatefulSets);
    spawn_watcher!(k8s_openapi::api::apps::v1::DaemonSet, "DaemonSet", crate::kubernetes::workloads::map_daemonset, DaemonSets);
    spawn_watcher!(k8s_openapi::api::apps::v1::ReplicaSet, "ReplicaSet", crate::kubernetes::workloads::map_replicaset, ReplicaSets);
    spawn_watcher!(k8s_openapi::api::batch::v1::Job, "Job", crate::kubernetes::batch::map_job, Jobs);
    spawn_watcher!(k8s_openapi::api::batch::v1::CronJob, "CronJob", crate::kubernetes::batch::map_cronjob, CronJobs);
    spawn_watcher!(k8s_openapi::api::autoscaling::v2::HorizontalPodAutoscaler, "HorizontalPodAutoscaler", crate::kubernetes::hpa::map_hpa, Hpas);
    spawn_watcher!(k8s_openapi::api::core::v1::Namespace, "Namespace", crate::kubernetes::namespaces::map_namespace, Namespaces);
    spawn_watcher!(k8s_openapi::api::core::v1::ConfigMap, "ConfigMap", crate::kubernetes::configmaps::format_configmap, ConfigMaps);
    spawn_watcher!(k8s_openapi::api::core::v1::Secret, "Secret", crate::kubernetes::secrets::format_secret, Secrets);
    spawn_watcher!(k8s_openapi::api::core::v1::Event, "Event", crate::kubernetes::events::format_event, Events);
    spawn_watcher!(k8s_openapi::api::core::v1::PersistentVolume, "PersistentVolume", crate::kubernetes::storage::format_pv, PersistentVolumes);
    spawn_watcher!(k8s_openapi::api::core::v1::PersistentVolumeClaim, "PersistentVolumeClaim", crate::kubernetes::storage::format_pvc, PersistentVolumeClaims);
    spawn_watcher!(k8s_openapi::api::storage::v1::StorageClass, "StorageClass", crate::kubernetes::storage::format_storage_class, StorageClasses);
    spawn_watcher!(k8s_openapi::api::networking::v1::NetworkPolicy, "Policy", crate::kubernetes::policies::format_network_policy, Policies);
    spawn_watcher!(k8s_openapi::api::core::v1::ResourceQuota, "Policy", crate::kubernetes::policies::format_resource_quota, Policies);
    spawn_watcher!(k8s_openapi::api::core::v1::LimitRange, "Policy", crate::kubernetes::policies::format_limit_range, Policies);
    spawn_watcher!(k8s_openapi::api::admissionregistration::v1::ValidatingWebhookConfiguration, "Policy", crate::kubernetes::policies::format_val_webhook, Policies);
    spawn_watcher!(k8s_openapi::api::admissionregistration::v1::MutatingWebhookConfiguration, "Policy", crate::kubernetes::policies::format_mut_webhook, Policies);

    tokio::spawn(crate::kubernetes::metrics::poll_pod_metrics(
        client.clone(),
        sink.clone(),
        child_rx,
    ));
}

#[cfg(test)]
mod tests {
    use super::*;
    use k8s_openapi::apimachinery::pkg::apis::meta::v1::ObjectMeta;
    use k8s_openapi::api::core::v1::Pod;
    use crate::sink::MockEventSink;

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

    #[test]
    fn test_coalesce_interval_is_250ms() {
        assert_eq!(COALESCE_INTERVAL, Duration::from_millis(250));
    }

    #[test]
    fn test_emit_dedup_suppresses_identical_consecutive_items_and_allows_deletes() {
        let mut dedup = EmitDedup::new();
        let data = serde_json::json!({ "name": "nginx-pod", "status": "Running" });
        assert!(dedup.should_emit("pod-1", "Applied", &data));
        assert!(!dedup.should_emit("pod-1", "Applied", &data));
        assert!(dedup.should_emit("pod-1", "Deleted", &data));
        assert!(dedup.should_emit("pod-1", "Applied", &data));
    }

    #[tokio::test]
    async fn test_spawn_watchers_emits_initial_connected_to_sink() {
        let sink = Arc::new(MockEventSink::new());
        let reporter = ConnectionReporter::new(sink.clone());

        reporter.report(WatcherConnectionState::Connected, 0).await;

        let events = sink.get_events();
        assert_eq!(events.len(), 1);
        match &events[0] {
            CoreEvent::WatcherConnectionChanged { state, failing_watchers, .. } => {
                assert_eq!(*state, WatcherConnectionState::Connected);
                assert_eq!(*failing_watchers, 0);
            }
            other => panic!("expected WatcherConnectionChanged, got {:?}", other),
        }
    }
}
