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
    #[serde(rename = "clustersUpdated")]
    ClustersUpdated {
        clusters: Vec<crate::kubernetes::models::ClusterInfo>,
    },
    #[serde(rename = "activeClusterChanged")]
    ActiveClusterChanged {
        active_cluster_id: Option<String>,
    },
    #[serde(rename = "errorOccurred")]
    ErrorOccurred {
        message: String,
    },
}
