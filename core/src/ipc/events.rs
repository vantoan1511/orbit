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
        namespaces: Vec<String>,
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
    #[serde(rename = "nodesUpdated")]
    NodesUpdated {
        nodes: Vec<crate::kubernetes::models::NodeInfo>,
    },
    #[serde(rename = "servicesUpdated")]
    ServicesUpdated {
        services: Vec<crate::kubernetes::models::ServiceInfo>,
    },
    #[serde(rename = "configMapsUpdated")]
    ConfigMapsUpdated {
        config_maps: Vec<crate::kubernetes::models::ConfigMapInfo>,
    },
    #[serde(rename = "secretsUpdated")]
    SecretsUpdated {
        secrets: Vec<crate::kubernetes::models::SecretInfo>,
    },
    #[serde(rename = "errorOccurred")]
    ErrorOccurred {
        message: String,
    },
}

