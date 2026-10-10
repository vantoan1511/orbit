use std::path::PathBuf;
use serde::{Deserialize, Serialize};
use crate::config::OrbitConfig;

static FILE_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct PersistedPortForward {
    pub id: String,
    pub cluster_id: String,
    pub namespace: String,
    pub kind: String,
    pub name: String,
    pub local_port: u16,
    pub remote_port: u16,
}

pub fn port_forwards_file_path() -> Option<PathBuf> {
    let mut path = OrbitConfig::config_dir()?;
    path.push("port_forwards.json");
    Some(path)
}

#[allow(dead_code)]
pub fn load_persisted_port_forwards() -> Vec<PersistedPortForward> {
    let _guard = FILE_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    load_persisted_port_forwards_unlocked()
}

fn load_persisted_port_forwards_unlocked() -> Vec<PersistedPortForward> {
    let Some(path) = port_forwards_file_path() else {
        return Vec::new();
    };
    if !path.exists() {
        return Vec::new();
    }
    let content = match std::fs::read_to_string(&path) {
        Ok(c) => c,
        Err(e) => {
            tracing::warn!(path = ?path, error = %e, "Failed to read port forwards file");
            return Vec::new();
        }
    };
    serde_json::from_str(&content).unwrap_or_default()
}

#[allow(dead_code)]
pub fn save_persisted_port_forwards(forwards: &[PersistedPortForward]) -> Result<(), String> {
    let _guard = FILE_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    save_persisted_port_forwards_unlocked(forwards)
}

fn save_persisted_port_forwards_unlocked(forwards: &[PersistedPortForward]) -> Result<(), String> {
    let Some(path) = port_forwards_file_path() else {
        return Err("Could not determine config directory".to_string());
    };
    if let Some(parent) = path.parent().filter(|p| !p.exists()) {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("Failed to create config directory {:?}: {}", parent, e))?;
    }
    let content = serde_json::to_string_pretty(forwards)
        .map_err(|e| format!("Failed to serialize port forwards: {}", e))?;
    let temp_path = path.with_extension("tmp");
    std::fs::write(&temp_path, &content)
        .map_err(|e| format!("Failed to write temporary port forwards file {:?}: {}", temp_path, e))?;
    std::fs::rename(&temp_path, &path)
        .map_err(|e| format!("Failed to atomically rename port forwards file to {:?}: {}", path, e))?;
    Ok(())
}

pub fn add_persisted_port_forward(forward: PersistedPortForward) {
    let _guard = FILE_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let mut current = load_persisted_port_forwards_unlocked();
    current.retain(|f| f.id != forward.id);
    current.push(forward);
    if let Err(e) = save_persisted_port_forwards_unlocked(&current) {
        tracing::error!(error = %e, "Failed to save added port forward to disk");
    }
}

pub fn remove_persisted_port_forward(id: &str) {
    let _guard = FILE_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let mut current = load_persisted_port_forwards_unlocked();
    let prev_len = current.len();
    current.retain(|f| f.id != id);
    if current.len() == prev_len {
        return;
    }
    if let Err(e) = save_persisted_port_forwards_unlocked(&current) {
        tracing::error!(error = %e, "Failed to save updated port forwards after removal");
    }
}

pub fn remove_persisted_port_forwards_for_cluster(cluster_id: &str) {
    let _guard = FILE_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let mut current = load_persisted_port_forwards_unlocked();
    let prev_len = current.len();
    current.retain(|f| f.cluster_id != cluster_id);
    if current.len() == prev_len {
        return;
    }
    if let Err(e) = save_persisted_port_forwards_unlocked(&current) {
        tracing::error!(error = %e, "Failed to save updated port forwards after cluster clear");
    }
}

pub fn get_persisted_for_cluster(cluster_id: &str) -> Vec<PersistedPortForward> {
    let _guard = FILE_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    load_persisted_port_forwards_unlocked()
        .into_iter()
        .filter(|f| f.cluster_id == cluster_id)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_port_forward_serialization() {
        let pf = PersistedPortForward {
            id: "default/deployment/web:8080:80".to_string(),
            cluster_id: "minikube".to_string(),
            namespace: "default".to_string(),
            kind: "deployment".to_string(),
            name: "web".to_string(),
            local_port: 8080,
            remote_port: 80,
        };

        let serialized = serde_json::to_string(&pf).expect("serialize");
        assert!(serialized.contains("\"localPort\":8080"));
        assert!(serialized.contains("\"remotePort\":80"));
        assert!(serialized.contains("\"clusterId\":\"minikube\""));

        let deserialized: PersistedPortForward = serde_json::from_str(&serialized).expect("deserialize");
        assert_eq!(deserialized, pf);
    }

    #[test]
    fn test_filter_by_cluster() {
        let forwards = vec![
            PersistedPortForward {
                id: "ns1/pod/p1:8080:80".to_string(),
                cluster_id: "cluster-a".to_string(),
                namespace: "ns1".to_string(),
                kind: "pod".to_string(),
                name: "p1".to_string(),
                local_port: 8080,
                remote_port: 80,
            },
            PersistedPortForward {
                id: "ns2/svc/s2:3000:3000".to_string(),
                cluster_id: "cluster-b".to_string(),
                namespace: "ns2".to_string(),
                kind: "service".to_string(),
                name: "s2".to_string(),
                local_port: 3000,
                remote_port: 3000,
            },
        ];

        let cluster_a_only: Vec<_> = forwards.into_iter().filter(|f| f.cluster_id == "cluster-a").collect();
        assert_eq!(cluster_a_only.len(), 1);
        assert_eq!(cluster_a_only[0].name, "p1");
    }
}
