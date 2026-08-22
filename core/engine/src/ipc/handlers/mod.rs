use std::sync::Arc;
use serde_json::Value;
use tokio::sync::{Mutex, RwLock};
use crate::ipc::bridge::WsWriter;
use crate::kubernetes::manager::KubeManager;

pub mod cluster;
pub mod config;
pub mod logs;
pub mod network;
pub mod resource;
pub mod storage;
pub mod system;
pub mod update;
pub mod utils;
pub mod watchers;
pub mod workloads;

pub use watchers::spawn_watchers;

/// Dispatches an IPC event from the frontend to the appropriate Kubernetes handler.
/// Each arm spawns an async task so the message loop is never blocked.
pub fn dispatch(
    event_name: &str,
    data: Option<Value>,
    writer: Arc<Mutex<WsWriter>>,
    token: String,
    manager: Arc<RwLock<KubeManager>>,
) {
    match event_name {
        "getClusters" => cluster::get_clusters(writer, token, manager),
        "getUserProfile" => cluster::get_user_profile(writer, token, manager),
        "switchCluster" => cluster::switch_cluster(data, writer, token, manager),
        "addCluster" => cluster::add_cluster(data, writer, token, manager),
        "getNamespaces" => system::get_namespaces(writer, token, manager),
        "getPods" => workloads::get_pods(data, writer, token, manager),
        "getDeployments" => workloads::get_deployments(data, writer, token, manager),
        "getStatefulSets" => workloads::get_statefulsets(data, writer, token, manager),
        "getDaemonSets" => workloads::get_daemonsets(data, writer, token, manager),
        "getReplicaSets" => workloads::get_replicasets(data, writer, token, manager),
        "getJobs" => workloads::get_jobs(data, writer, token, manager),
        "getCronJobs" => workloads::get_cronjobs(data, writer, token, manager),
        "getServices" => network::get_services(data, writer, token, manager),
        "getIngresses" => network::get_ingresses(data, writer, token, manager),
        "getConfigMaps" => config::get_config_maps(data, writer, token, manager),
        "getEvents" => system::get_events(data, writer, token, manager),
        "getSecrets" => config::get_secrets(data, writer, token, manager),
        "getPersistentVolumes" => storage::get_persistent_volumes(writer, token, manager),
        "getPersistentVolumeClaims" => storage::get_persistent_volume_claims(data, writer, token, manager),
        "getStorageClasses" => storage::get_storage_classes(writer, token, manager),
        "getNodes" => system::get_nodes(writer, token, manager),
        "getPolicies" => system::get_policies(data, writer, token, manager),
        "checkForUpdates" => update::check_for_updates(data, writer, token),
        "applyUpdate" => update::apply_update(data, writer, token),
        "streamLogs" => logs::stream_logs(data, writer, token, manager),
        "stopLogs" => logs::stop_logs(manager),
        "scaleResource" => workloads::scale_resource(data, writer, token, manager),
        "updateResourceImages" => workloads::update_resource_images(data, writer, token, manager),
        "redeployResource" => workloads::redeploy_resource(data, writer, token, manager),
        "deleteResource" => resource::delete_resource(data, writer, token, manager),
        "restartPod" => workloads::restart_pod(data, writer, token, manager),
        "getResourceRaw" => resource::get_resource_raw(data, writer, token, manager),
        "applyResource" => resource::apply_resource(data, writer, token, manager),
        "cloneIngress" => network::clone_ingress(data, writer, token, manager),
        _ => {}
    }
}
