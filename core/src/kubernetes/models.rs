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

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct NodeInfo {
    pub name: String,
    pub status: String,
    pub role: String,
    pub version: String,
    pub cpu_pct: f64,
    pub cpu_used: String,
    pub cpu_total: String,
    pub mem_pct: f64,
    pub mem_used: String,
    pub mem_total: String,
    pub pods_count: i32,
    pub pods_limit: i32,
    pub uptime: String,
    pub labels: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ServicePort {
    pub port: i32,
    pub target_port: String,
    pub protocol: String,
    pub node_port: Option<i32>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ServiceEvent {
    pub r#type: String,
    pub reason: String,
    pub message: String,
    pub age: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ServiceInfo {
    pub name: String,
    pub namespace: String,
    pub r#type: String,
    pub cluster_ip: String,
    pub external_ip: String,
    pub ports: String,
    pub endpoints: String,
    pub age: String,
    pub session_affinity: String,
    pub internal_traffic_policy: Option<String>,
    pub created: String,
    pub uid: String,
    pub selector: std::collections::BTreeMap<String, String>,
    pub labels: std::collections::BTreeMap<String, String>,
    pub ports_list: Vec<ServicePort>,
    pub endpoints_list: Vec<String>,
    pub events: Vec<ServiceEvent>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct UsedByPod {
    pub name: String,
    pub status: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ConfigMapInfo {
    pub name: String,
    pub namespace: String,
    pub labels: std::collections::BTreeMap<String, String>,
    pub annotations: i32,
    pub created: String,
    pub age: String,
    pub resource_version: String,
    pub immutable: bool,
    pub keys_count: i32,
    pub size: String,
    pub mounted_pods: i32,
    pub used_by: Vec<UsedByPod>,
    pub data: std::collections::BTreeMap<String, String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SecretInfo {
    pub name: String,
    pub namespace: String,
    pub labels: std::collections::BTreeMap<String, String>,
    pub annotations: i32,
    pub r#type: String,
    pub created: String,
    pub age: String,
    pub resource_version: String,
    pub immutable: bool,
    pub keys_count: i32,
    pub size: String,
    pub mounted_pods: i32,
    pub used_by: Vec<UsedByPod>,
    pub data: std::collections::BTreeMap<String, String>,
}






