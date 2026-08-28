use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "event", content = "data")]
pub enum OrbitEvent {
    #[serde(rename = "engineConnected")]
    EngineConnected {
        status: String,
        message: String,
    },
    #[serde(rename = "ping")]
    Ping {
        message: String,
    },
    #[serde(rename = "pong")]
    Pong {
        reply: String,
    },
    #[serde(rename = "namespacesUpdated")]
    NamespacesUpdated {
        namespaces: Vec<crate::kubernetes::models::NamespaceInfo>,
    },
    #[serde(rename = "podsUpdated")]
    PodsUpdated {
        pods: Vec<crate::kubernetes::models::PodInfo>,
    },
    #[serde(rename = "deploymentsUpdated")]
    DeploymentsUpdated {
        deployments: Vec<crate::kubernetes::models::DeploymentInfo>,
    },
    #[serde(rename = "statefulSetsUpdated")]
    StatefulSetsUpdated {
        stateful_sets: Vec<crate::kubernetes::models::StatefulSetInfo>,
    },
    #[serde(rename = "daemonSetsUpdated")]
    DaemonSetsUpdated {
        daemon_sets: Vec<crate::kubernetes::models::DaemonSetInfo>,
    },
    #[serde(rename = "replicaSetsUpdated")]
    ReplicaSetsUpdated {
        replica_sets: Vec<crate::kubernetes::models::ReplicaSetInfo>,
    },
    #[serde(rename = "jobsUpdated")]
    JobsUpdated {
        jobs: Vec<crate::kubernetes::models::JobInfo>,
    },
    #[serde(rename = "cronJobsUpdated")]
    CronJobsUpdated {
        cron_jobs: Vec<crate::kubernetes::models::CronJobInfo>,
    },
    #[serde(rename = "clustersUpdated")]
    ClustersUpdated {
        clusters: Vec<crate::kubernetes::models::ClusterInfo>,
    },
    #[serde(rename = "activeClusterChanged")]
    ActiveClusterChanged {
        active_cluster_id: Option<String>,
    },
    #[serde(rename = "userProfileUpdated")]
    UserProfileUpdated {
        profile: crate::kubernetes::models::UserProfileInfo,
    },
    #[serde(rename = "nodesUpdated")]
    NodesUpdated {
        nodes: Vec<crate::kubernetes::models::NodeInfo>,
    },
    #[serde(rename = "servicesUpdated")]
    ServicesUpdated {
        services: Vec<crate::kubernetes::models::ServiceInfo>,
    },
    #[serde(rename = "ingressesUpdated")]
    IngressesUpdated {
        ingresses: Vec<crate::kubernetes::models::IngressInfo>,
    },
    #[serde(rename = "configMapsUpdated")]
    ConfigMapsUpdated {
        config_maps: Vec<crate::kubernetes::models::ConfigMapInfo>,
    },
    #[serde(rename = "secretsUpdated")]
    SecretsUpdated {
        secrets: Vec<crate::kubernetes::models::SecretInfo>,
    },
    #[serde(rename = "eventsUpdated")]
    EventsUpdated {
        events: Vec<crate::kubernetes::models::EventInfo>,
    },
    #[serde(rename = "persistentVolumesUpdated")]
    PersistentVolumesUpdated {
        persistent_volumes: Vec<crate::kubernetes::models::PersistentVolumeInfo>,
    },
    #[serde(rename = "persistentVolumeClaimsUpdated")]
    PersistentVolumeClaimsUpdated {
        persistent_volume_claims: Vec<crate::kubernetes::models::PersistentVolumeClaimInfo>,
    },
    #[serde(rename = "storageClassesUpdated")]
    StorageClassesUpdated {
        storage_classes: Vec<crate::kubernetes::models::StorageClassInfo>,
    },
    #[serde(rename = "policiesUpdated")]
    PoliciesUpdated {
        policies: Vec<crate::kubernetes::models::PolicyInfo>,
    },
    #[serde(rename = "resourceUpdated")]
    ResourceUpdated {
        kind: String,
        action: String,
        data: serde_json::Value,
    },
    #[serde(rename = "podMetricsUpdated")]
    PodMetricsUpdated {
        metrics: Vec<crate::kubernetes::models::PodMetricItem>,
    },
    #[serde(rename = "errorOccurred")]
    ErrorOccurred {
        message: String,
    },
    #[serde(rename = "logLineReceived")]
    LogLineReceived {
        pod: String,
        container: String,
        line: String,
    },
    #[serde(rename = "logLinesChunkReceived")]
    LogLinesChunkReceived {
        pod: String,
        container: String,
        lines: Vec<String>,
    },
    #[serde(rename = "updateCheckFinished")]
    UpdateCheckFinished {
        has_update: bool,
        manifest: crate::updater::UpdateManifest,
    },
    #[serde(rename = "commandSucceeded")]
    CommandSucceeded {
        message: String,
    },
    #[serde(rename = "updateDownloadProgress")]
    UpdateDownloadProgress {
        component: String,
        progress_percentage: u8,
    },
    #[serde(rename = "updateReady")]
    UpdateReady {
        component: String,
    },
    #[serde(rename = "resourceRawData")]
    ResourceRawData {
        kind: String,
        name: String,
        data: serde_json::Value,
    },
    #[serde(rename = "portForwardStarted")]
    #[serde(rename_all = "camelCase")]
    PortForwardStarted {
        id: String,
        namespace: String,
        kind: String,
        name: String,
        local_port: u16,
        remote_port: u16,
    },
    #[serde(rename = "portForwardStopped")]
    #[serde(rename_all = "camelCase")]
    PortForwardStopped {
        id: String,
    },
}

impl OrbitEvent {
    pub fn event_name(&self) -> &'static str {
        match self {
            OrbitEvent::EngineConnected { .. } => "engineConnected",
            OrbitEvent::Ping { .. } => "ping",
            OrbitEvent::Pong { .. } => "pong",
            OrbitEvent::NamespacesUpdated { .. } => "namespacesUpdated",
            OrbitEvent::PodsUpdated { .. } => "podsUpdated",
            OrbitEvent::DeploymentsUpdated { .. } => "deploymentsUpdated",
            OrbitEvent::StatefulSetsUpdated { .. } => "statefulSetsUpdated",
            OrbitEvent::DaemonSetsUpdated { .. } => "daemonSetsUpdated",
            OrbitEvent::ReplicaSetsUpdated { .. } => "replicaSetsUpdated",
            OrbitEvent::JobsUpdated { .. } => "jobsUpdated",
            OrbitEvent::CronJobsUpdated { .. } => "cronJobsUpdated",
            OrbitEvent::ClustersUpdated { .. } => "clustersUpdated",
            OrbitEvent::ActiveClusterChanged { .. } => "activeClusterChanged",
            OrbitEvent::UserProfileUpdated { .. } => "userProfileUpdated",
            OrbitEvent::NodesUpdated { .. } => "nodesUpdated",
            OrbitEvent::ServicesUpdated { .. } => "servicesUpdated",
            OrbitEvent::IngressesUpdated { .. } => "ingressesUpdated",
            OrbitEvent::ConfigMapsUpdated { .. } => "configMapsUpdated",
            OrbitEvent::SecretsUpdated { .. } => "secretsUpdated",
            OrbitEvent::EventsUpdated { .. } => "eventsUpdated",
            OrbitEvent::PersistentVolumesUpdated { .. } => "persistentVolumesUpdated",
            OrbitEvent::PersistentVolumeClaimsUpdated { .. } => "persistentVolumeClaimsUpdated",
            OrbitEvent::StorageClassesUpdated { .. } => "storageClassesUpdated",
            OrbitEvent::PoliciesUpdated { .. } => "policiesUpdated",
            OrbitEvent::ResourceUpdated { .. } => "resourceUpdated",
            OrbitEvent::PodMetricsUpdated { .. } => "podMetricsUpdated",
            OrbitEvent::ErrorOccurred { .. } => "errorOccurred",
            OrbitEvent::LogLineReceived { .. } => "logLineReceived",
            OrbitEvent::LogLinesChunkReceived { .. } => "logLinesChunkReceived",
            OrbitEvent::UpdateCheckFinished { .. } => "updateCheckFinished",
            OrbitEvent::CommandSucceeded { .. } => "commandSucceeded",
            OrbitEvent::UpdateDownloadProgress { .. } => "updateDownloadProgress",
            OrbitEvent::UpdateReady { .. } => "updateReady",
            OrbitEvent::ResourceRawData { .. } => "resourceRawData",
            OrbitEvent::PortForwardStarted { .. } => "portForwardStarted",
            OrbitEvent::PortForwardStopped { .. } => "portForwardStopped",
        }
    }
}
