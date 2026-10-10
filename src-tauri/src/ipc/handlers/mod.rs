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
pub mod settings;
pub mod storage;
pub mod system;
pub mod terminal;
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
    tracing::debug!(event = %event_name, "Dispatching UI event");
    match event_name {
        "getAppSettings" => settings::get_app_settings(writer, token),
        "updateAppSettings" => settings::update_app_settings(data, writer, token),
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
        "createResource" => resource::create_resource(data, writer, token, manager),
        "cloneIngress" => network::clone_ingress(data, writer, token, manager),
        "cloneDeployment" => workloads::clone_deployment(data, writer, token, manager),
        "rollbackDeployment" => workloads::rollback_deployment(data, writer, token, manager),
        "startPortForward" => network::start_port_forward(data, writer, token, manager),
        "stopPortForward" => network::stop_port_forward(data, writer, token, manager),
        "getPortForwards" => network::get_port_forwards(writer, token, manager),
        "openLocalTerminal" => terminal::open_local_terminal(data, writer, token),
        "openPodTerminal" => terminal::open_pod_terminal(data, writer, token, manager),
        "sendTerminalData" => terminal::send_terminal_data(data),
        "resizeTerminal" => terminal::resize_terminal(data),
        "closeTerminal" => terminal::close_terminal(data),
        "clientConnect" | "appClientConnect" => {
            let writer = writer.clone();
            let token = token.clone();
            let manager = manager.clone();
            tokio::spawn(async move {
                cluster::broadcast_engine_ready(&writer, &token, &manager).await;
            });
        }
        "ping" => {
            let writer = writer.clone();
            let token = token.clone();
            tokio::spawn(async move {
                let _ = crate::ipc::bridge::Bridge::send_event(
                    &writer,
                    &token,
                    &crate::ipc::events::OrbitEvent::Pong { reply: "pong".to_string() },
                ).await;
            });
        }
        other => {
            tracing::debug!(event = %other, "Unhandled UI event in dispatcher");
        }
    }
}

/// Dispatches an IPC event from the Tauri frontend to the appropriate Kubernetes handler.
pub fn dispatch_tauri(
    event_name: &str,
    data: Option<Value>,
    manager: Arc<RwLock<KubeManager>>,
) {
    static NATIVE_WRITER: std::sync::OnceLock<Arc<Mutex<WsWriter>>> = std::sync::OnceLock::new();
    let writer = NATIVE_WRITER.get_or_init(|| Arc::new(Mutex::new(WsWriter::Emitter)));
    dispatch(event_name, data, writer.clone(), String::new(), manager);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ipc::emitter::{set_global_emitter, MockEventEmitter};

    #[tokio::test]
    async fn test_dispatch_ping_responds_with_pong() {
        let _guard = crate::ipc::emitter::TEST_EMITTER_LOCK.lock().unwrap();
        let mock_emitter = Arc::new(MockEventEmitter::new());
        set_global_emitter(mock_emitter.clone());

        let writer = Arc::new(Mutex::new(WsWriter::Emitter));
        let manager = Arc::new(RwLock::new(KubeManager::new().await));

        dispatch("ping", None, writer, "token".to_string(), manager);

        tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
        let emitted = mock_emitter.get_emitted_events().await;
        let pong_event = emitted.iter().find(|(name, _)| name == "pong");
        assert!(pong_event.is_some());
    }
}
