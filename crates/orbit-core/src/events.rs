use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum WatcherConnectionState {
    Connected,
    Degraded,
    Suspended,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum WatchAction<T> {
    Applied(T),
    Deleted(T),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TypedResourceBatch {
    Namespaces(Vec<WatchAction<crate::kubernetes::models::NamespaceInfo>>),
    Pods(Vec<WatchAction<crate::kubernetes::models::PodInfo>>),
    Deployments(Vec<WatchAction<crate::kubernetes::models::DeploymentInfo>>),
    StatefulSets(Vec<WatchAction<crate::kubernetes::models::StatefulSetInfo>>),
    DaemonSets(Vec<WatchAction<crate::kubernetes::models::DaemonSetInfo>>),
    ReplicaSets(Vec<WatchAction<crate::kubernetes::models::ReplicaSetInfo>>),
    Jobs(Vec<WatchAction<crate::kubernetes::models::JobInfo>>),
    CronJobs(Vec<WatchAction<crate::kubernetes::models::CronJobInfo>>),
    Hpas(Vec<WatchAction<crate::kubernetes::models::HpaInfo>>),
    Nodes(Vec<WatchAction<crate::kubernetes::models::NodeInfo>>),
    Services(Vec<WatchAction<crate::kubernetes::models::ServiceInfo>>),
    Ingresses(Vec<WatchAction<crate::kubernetes::models::IngressInfo>>),
    ConfigMaps(Vec<WatchAction<crate::kubernetes::models::ConfigMapInfo>>),
    Secrets(Vec<WatchAction<crate::kubernetes::models::SecretInfo>>),
    Events(Vec<WatchAction<crate::kubernetes::models::EventInfo>>),
    PersistentVolumes(Vec<WatchAction<crate::kubernetes::models::PersistentVolumeInfo>>),
    PersistentVolumeClaims(Vec<WatchAction<crate::kubernetes::models::PersistentVolumeClaimInfo>>),
    StorageClasses(Vec<WatchAction<crate::kubernetes::models::StorageClassInfo>>),
    Policies(Vec<WatchAction<crate::kubernetes::models::PolicyInfo>>),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CoreEvent {
    EngineReady {
        status: String,
        message: String,
        clusters: Vec<crate::kubernetes::models::ClusterInfo>,
        active_cluster_id: Option<String>,
        settings: HashMap<String, serde_json::Value>,
    },
    Ping,
    Pong,
    ClustersUpdated {
        clusters: Vec<crate::kubernetes::models::ClusterInfo>,
    },
    ActiveClusterChanged {
        active_cluster_id: Option<String>,
    },
    ClusterAdded {
        cluster_id: String,
        request_id: Option<String>,
    },
    ClusterAddFailed {
        error: String,
        request_id: Option<String>,
    },
    UserProfileUpdated {
        profile: crate::kubernetes::models::UserProfileInfo,
    },
    ResourceBatchUpdated(TypedResourceBatch),
    PodMetricsUpdated {
        metrics: Vec<crate::kubernetes::metrics::PodMetricInfo>,
    },
    WatcherConnectionChanged {
        state: WatcherConnectionState,
        failing_watchers: usize,
        message: String,
    },
    RawResourceLoaded {
        kind: String,
        name: String,
        namespace: Option<String>,
        yaml: String,
        request_id: Option<String>,
    },
    ResourceApplied {
        kind: String,
        name: String,
        namespace: Option<String>,
        request_id: Option<String>,
    },
    ResourceCreated {
        kind: String,
        name: String,
        namespace: Option<String>,
        request_id: Option<String>,
    },
    LogStreamReceived {
        chunk: String,
    },
    LogStreamStopped,
    PortForwardsUpdated {
        forwards: Vec<crate::kubernetes::port_forward::PortForwardState>,
    },
    UpdateAvailable {
        version: String,
        notes: String,
        download_url: String,
    },
    UpdateNotAvailable,
    UpdateError {
        error: String,
    },
    ActionSucceeded {
        action: String,
        message: String,
    },
    ErrorOccurred {
        action: String,
        message: String,
    },
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sink::{CoreEventSink, MpscEventSink};
    use tokio::sync::mpsc;

    #[tokio::test]
    async fn test_mpsc_event_sink_delivers_typed_core_events() {
        let (tx, mut rx) = mpsc::unbounded_channel::<CoreEvent>();
        let sink = MpscEventSink::new(tx);

        sink.send(CoreEvent::Pong).expect("send Pong");
        sink.send(CoreEvent::ResourceBatchUpdated(TypedResourceBatch::Pods(vec![]))).expect("send batch");

        match rx.recv().await {
            Some(CoreEvent::Pong) => {}
            other => panic!("expected Pong, got {:?}", other),
        }

        match rx.recv().await {
            Some(CoreEvent::ResourceBatchUpdated(TypedResourceBatch::Pods(p))) => {
                assert!(p.is_empty());
            }
            other => panic!("expected Pods batch, got {:?}", other),
        }
    }
}
