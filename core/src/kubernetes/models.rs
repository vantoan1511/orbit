use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PodInfo {
    pub name: String,
    pub namespace: String,
    pub status: String,
    pub age: String,
    pub cpu: Option<String>,
    pub memory: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ClusterInfo {
    pub id: String,
    pub name: String,
    pub status: String,
}

