use std::sync::Arc;
use tokio::sync::Mutex;
use crate::ipc::bridge::{Bridge, WsWriter};
use crate::ipc::events::OrbitEvent;
use crate::kubernetes::watch_health::{WatcherHealthTracker, WATCHER_GIVE_UP_AFTER};
use crate::kubernetes::watchers::WatcherContext;

const GIVE_UP_CHECK_INTERVAL: std::time::Duration = std::time::Duration::from_secs(5);

/// Derives a child cancellation channel whose state mirrors parent cancellation,
/// or can be cancelled independently (e.g. on tracker suspend).
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
                    let _ = child_tx_clone.send(true);
                    break;
                }
            }
        });
    }
    (child_tx, child_rx)
}

/// Spawns all background watchers and the metrics poller for a connected cluster.
/// Called when the engine starts up and whenever the active cluster changes.
pub fn spawn_watchers(
    client: &kube::Client,
    writer: Arc<Mutex<WsWriter>>,
    token: String,
    rx: tokio::sync::watch::Receiver<bool>,
    cluster_id: Option<String>,
) {
    let (child_tx, child_rx) = derive_cancel_channel(rx);
    let tracker = WatcherHealthTracker::new(WATCHER_GIVE_UP_AFTER, child_tx);

    let ticker_tracker = tracker.clone();
    let ticker_writer = writer.clone();
    let ticker_token = token.clone();
    let ticker_cluster_id = cluster_id.clone();
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
                        let event = OrbitEvent::WatcherConnectionChanged {
                            cluster_id: ticker_cluster_id.clone(),
                            state,
                            failing_watchers: ticker_tracker.failing_count(),
                        };
                        if let Err(e) = Bridge::send_event(&ticker_writer, &ticker_token, &event).await {
                            tracing::debug!(error = %e, "Failed to emit suspended state event");
                        }
                        break;
                    }
                }
            }
        }
    });

    let make_ctx = |rx: tokio::sync::watch::Receiver<bool>| WatcherContext {
        client: client.clone(),
        writer: writer.clone(),
        ipc_token: token.clone(),
        cluster_id: cluster_id.clone(),
        tracker: tracker.clone(),
        cancel_rx: rx,
    };

    macro_rules! spawn_watcher {
        ($kind_type:ty, $label:expr, $mapper:path) => {
            tokio::spawn(crate::kubernetes::watchers::watch_resource::<$kind_type, _, _>(
                $label.to_string(),
                make_ctx(child_rx.clone()),
                $mapper,
            ));
        };
    }

    spawn_watcher!(k8s_openapi::api::core::v1::Service, "Service", crate::kubernetes::services::map_service);
    spawn_watcher!(k8s_openapi::api::networking::v1::Ingress, "Ingress", crate::kubernetes::ingresses::map_ingress);
    spawn_watcher!(k8s_openapi::api::apps::v1::Deployment, "Deployment", crate::kubernetes::workloads::map_deployment);
    spawn_watcher!(k8s_openapi::api::core::v1::Pod, "Pod", crate::kubernetes::workloads::map_pod);
    spawn_watcher!(k8s_openapi::api::apps::v1::StatefulSet, "StatefulSet", crate::kubernetes::workloads::map_statefulset);
    spawn_watcher!(k8s_openapi::api::apps::v1::DaemonSet, "DaemonSet", crate::kubernetes::workloads::map_daemonset);
    spawn_watcher!(k8s_openapi::api::apps::v1::ReplicaSet, "ReplicaSet", crate::kubernetes::workloads::map_replicaset);
    spawn_watcher!(k8s_openapi::api::batch::v1::Job, "Job", crate::kubernetes::batch::map_job);
    spawn_watcher!(k8s_openapi::api::batch::v1::CronJob, "CronJob", crate::kubernetes::batch::map_cronjob);
    spawn_watcher!(k8s_openapi::api::core::v1::Namespace, "Namespace", crate::kubernetes::namespaces::map_namespace);
    spawn_watcher!(k8s_openapi::api::core::v1::ConfigMap, "ConfigMap", crate::kubernetes::configmaps::format_configmap);
    spawn_watcher!(k8s_openapi::api::core::v1::Secret, "Secret", crate::kubernetes::secrets::format_secret);
    spawn_watcher!(k8s_openapi::api::core::v1::Event, "Event", crate::kubernetes::events::format_event);
    spawn_watcher!(k8s_openapi::api::core::v1::PersistentVolume, "PersistentVolume", crate::kubernetes::storage::format_pv);
    spawn_watcher!(k8s_openapi::api::core::v1::PersistentVolumeClaim, "PersistentVolumeClaim", crate::kubernetes::storage::format_pvc);
    spawn_watcher!(k8s_openapi::api::storage::v1::StorageClass, "StorageClass", crate::kubernetes::storage::format_storage_class);
    spawn_watcher!(k8s_openapi::api::networking::v1::NetworkPolicy, "Policy", crate::kubernetes::policies::format_network_policy);
    spawn_watcher!(k8s_openapi::api::core::v1::ResourceQuota, "Policy", crate::kubernetes::policies::format_resource_quota);
    spawn_watcher!(k8s_openapi::api::core::v1::LimitRange, "Policy", crate::kubernetes::policies::format_limit_range);
    spawn_watcher!(k8s_openapi::api::admissionregistration::v1::ValidatingWebhookConfiguration, "Policy", crate::kubernetes::policies::format_val_webhook);
    spawn_watcher!(k8s_openapi::api::admissionregistration::v1::MutatingWebhookConfiguration, "Policy", crate::kubernetes::policies::format_mut_webhook);

    tokio::spawn(crate::kubernetes::metrics::poll_pod_metrics(
        client.clone(), writer, token, cluster_id, child_rx,
    ));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_parent_cancel_propagates_to_child_channel() {
        let (parent_tx, parent_rx) = tokio::sync::watch::channel(false);
        let (_child_tx, child_rx) = derive_cancel_channel(parent_rx);
        assert!(!*child_rx.borrow());

        parent_tx.send(true).unwrap();
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;
        assert!(*child_rx.borrow());
    }

    #[tokio::test]
    async fn test_tracker_suspend_cancels_child() {
        let (_parent_tx, parent_rx) = tokio::sync::watch::channel(false);
        let (child_tx, child_rx) = derive_cancel_channel(parent_rx);
        let tracker = WatcherHealthTracker::new(std::time::Duration::from_secs(1), child_tx);
        let t0 = std::time::Instant::now();
        tracker.record_failure("test-watcher", t0);
        let res = tracker.check_give_up(t0 + std::time::Duration::from_secs(1));
        assert!(res.is_some());
        assert!(*child_rx.borrow());
    }
}
