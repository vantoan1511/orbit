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

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Replicas {
    pub current: i32,
    pub desired: i32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DaemonSetReplicas {
    pub desired: i32,
    pub current: i32,
    pub ready: i32,
    pub up_to_date: i32,
    pub available: i32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DeploymentInfo {
    pub name: String,
    pub namespace: String,
    pub status: String,
    pub replicas: Replicas,
    pub available: i32,
    pub up_to_date: i32,
    pub age: String,
    pub images: Vec<String>,
    pub strategy: Option<String>,
    pub min_ready_seconds: i32,
    pub revision_history: Option<i32>,
    pub labels: std::collections::BTreeMap<String, String>,
    pub annotations: std::collections::BTreeMap<String, String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct StatefulSetInfo {
    pub name: String,
    pub namespace: String,
    pub status: String,
    pub replicas: Replicas,
    pub age: String,
    pub images: Vec<String>,
    pub labels: std::collections::BTreeMap<String, String>,
    pub annotations: std::collections::BTreeMap<String, String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DaemonSetInfo {
    pub name: String,
    pub namespace: String,
    pub status: String,
    pub replicas: DaemonSetReplicas,
    pub age: String,
    pub images: Vec<String>,
    pub labels: std::collections::BTreeMap<String, String>,
    pub annotations: std::collections::BTreeMap<String, String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ReplicaSetInfo {
    pub name: String,
    pub namespace: String,
    pub status: String,
    pub replicas: Replicas,
    pub age: String,
    pub images: Vec<String>,
    pub labels: std::collections::BTreeMap<String, String>,
    pub annotations: std::collections::BTreeMap<String, String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct JobInfo {
    pub name: String,
    pub namespace: String,
    pub status: String,
    pub completions: String,
    pub duration: Option<String>,
    pub age: String,
    pub images: Vec<String>,
    pub labels: std::collections::BTreeMap<String, String>,
    pub annotations: std::collections::BTreeMap<String, String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CronJobInfo {
    pub name: String,
    pub namespace: String,
    pub schedule: String,
    pub suspend: bool,
    pub active: i32,
    pub last_schedule: Option<String>,
    pub age: String,
    pub images: Vec<String>,
    pub labels: std::collections::BTreeMap<String, String>,
    pub annotations: std::collections::BTreeMap<String, String>,
}



