use std::sync::Arc;
use tokio::sync::Mutex;
use crate::ipc::bridge::WsWriter;

/// Spawns all background watchers and the metrics poller for a connected cluster.
/// Called when the engine starts up and whenever the active cluster changes.
pub fn spawn_watchers(
    client: &kube::Client,
    writer: Arc<Mutex<WsWriter>>,
    token: String,
    rx: tokio::sync::watch::Receiver<bool>,
) {
    tokio::spawn(crate::kubernetes::watchers::watch_resource::<
        k8s_openapi::api::core::v1::Service, _, _,
    >(
        client.clone(), writer.clone(), token.clone(), "Service".to_string(), rx.clone(),
        crate::kubernetes::services::map_service,
    ));

    tokio::spawn(crate::kubernetes::watchers::watch_resource::<
        k8s_openapi::api::networking::v1::Ingress, _, _,
    >(
        client.clone(), writer.clone(), token.clone(), "Ingress".to_string(), rx.clone(),
        crate::kubernetes::ingresses::map_ingress,
    ));

    tokio::spawn(crate::kubernetes::watchers::watch_resource::<
        k8s_openapi::api::apps::v1::Deployment, _, _,
    >(
        client.clone(), writer.clone(), token.clone(), "Deployment".to_string(), rx.clone(),
        crate::kubernetes::workloads::map_deployment,
    ));

    tokio::spawn(crate::kubernetes::watchers::watch_resource::<
        k8s_openapi::api::core::v1::Pod, _, _,
    >(
        client.clone(), writer.clone(), token.clone(), "Pod".to_string(), rx.clone(),
        crate::kubernetes::workloads::map_pod,
    ));

    tokio::spawn(crate::kubernetes::watchers::watch_resource::<
        k8s_openapi::api::apps::v1::StatefulSet, _, _,
    >(
        client.clone(), writer.clone(), token.clone(), "StatefulSet".to_string(), rx.clone(),
        crate::kubernetes::workloads::map_statefulset,
    ));

    tokio::spawn(crate::kubernetes::watchers::watch_resource::<
        k8s_openapi::api::apps::v1::DaemonSet, _, _,
    >(
        client.clone(), writer.clone(), token.clone(), "DaemonSet".to_string(), rx.clone(),
        crate::kubernetes::workloads::map_daemonset,
    ));

    tokio::spawn(crate::kubernetes::watchers::watch_resource::<
        k8s_openapi::api::apps::v1::ReplicaSet, _, _,
    >(
        client.clone(), writer.clone(), token.clone(), "ReplicaSet".to_string(), rx.clone(),
        crate::kubernetes::workloads::map_replicaset,
    ));

    tokio::spawn(crate::kubernetes::watchers::watch_resource::<
        k8s_openapi::api::batch::v1::Job, _, _,
    >(
        client.clone(), writer.clone(), token.clone(), "Job".to_string(), rx.clone(),
        crate::kubernetes::batch::map_job,
    ));

    tokio::spawn(crate::kubernetes::watchers::watch_resource::<
        k8s_openapi::api::batch::v1::CronJob, _, _,
    >(
        client.clone(), writer.clone(), token.clone(), "CronJob".to_string(), rx.clone(),
        crate::kubernetes::batch::map_cronjob,
    ));

    tokio::spawn(crate::kubernetes::watchers::watch_resource::<
        k8s_openapi::api::core::v1::Namespace, _, _,
    >(
        client.clone(), writer.clone(), token.clone(), "Namespace".to_string(), rx.clone(),
        crate::kubernetes::namespaces::map_namespace,
    ));

    tokio::spawn(crate::kubernetes::watchers::watch_resource::<
        k8s_openapi::api::core::v1::ConfigMap, _, _,
    >(
        client.clone(), writer.clone(), token.clone(), "ConfigMap".to_string(), rx.clone(),
        crate::kubernetes::configmaps::format_configmap,
    ));

    tokio::spawn(crate::kubernetes::watchers::watch_resource::<
        k8s_openapi::api::core::v1::Secret, _, _,
    >(
        client.clone(), writer.clone(), token.clone(), "Secret".to_string(), rx.clone(),
        crate::kubernetes::secrets::format_secret,
    ));

    tokio::spawn(crate::kubernetes::watchers::watch_resource::<
        k8s_openapi::api::core::v1::Event, _, _,
    >(
        client.clone(), writer.clone(), token.clone(), "Event".to_string(), rx.clone(),
        crate::kubernetes::events::format_event,
    ));

    tokio::spawn(crate::kubernetes::watchers::watch_resource::<
        k8s_openapi::api::core::v1::PersistentVolume, _, _,
    >(
        client.clone(), writer.clone(), token.clone(), "PersistentVolume".to_string(), rx.clone(),
        crate::kubernetes::storage::format_pv,
    ));

    tokio::spawn(crate::kubernetes::watchers::watch_resource::<
        k8s_openapi::api::core::v1::PersistentVolumeClaim, _, _,
    >(
        client.clone(), writer.clone(), token.clone(), "PersistentVolumeClaim".to_string(), rx.clone(),
        crate::kubernetes::storage::format_pvc,
    ));

    tokio::spawn(crate::kubernetes::watchers::watch_resource::<
        k8s_openapi::api::storage::v1::StorageClass, _, _,
    >(
        client.clone(), writer.clone(), token.clone(), "StorageClass".to_string(), rx.clone(),
        crate::kubernetes::storage::format_storage_class,
    ));

    tokio::spawn(crate::kubernetes::watchers::watch_resource::<
        k8s_openapi::api::networking::v1::NetworkPolicy, _, _,
    >(
        client.clone(), writer.clone(), token.clone(), "Policy".to_string(), rx.clone(),
        crate::kubernetes::policies::format_network_policy,
    ));

    tokio::spawn(crate::kubernetes::watchers::watch_resource::<
        k8s_openapi::api::core::v1::ResourceQuota, _, _,
    >(
        client.clone(), writer.clone(), token.clone(), "Policy".to_string(), rx.clone(),
        crate::kubernetes::policies::format_resource_quota,
    ));

    tokio::spawn(crate::kubernetes::watchers::watch_resource::<
        k8s_openapi::api::core::v1::LimitRange, _, _,
    >(
        client.clone(), writer.clone(), token.clone(), "Policy".to_string(), rx.clone(),
        crate::kubernetes::policies::format_limit_range,
    ));

    tokio::spawn(crate::kubernetes::watchers::watch_resource::<
        k8s_openapi::api::admissionregistration::v1::ValidatingWebhookConfiguration, _, _,
    >(
        client.clone(), writer.clone(), token.clone(), "Policy".to_string(), rx.clone(),
        crate::kubernetes::policies::format_val_webhook,
    ));

    tokio::spawn(crate::kubernetes::watchers::watch_resource::<
        k8s_openapi::api::admissionregistration::v1::MutatingWebhookConfiguration, _, _,
    >(
        client.clone(), writer.clone(), token.clone(), "Policy".to_string(), rx.clone(),
        crate::kubernetes::policies::format_mut_webhook,
    ));

    tokio::spawn(crate::kubernetes::metrics::poll_pod_metrics(
        client.clone(), writer, token, rx,
    ));
}
