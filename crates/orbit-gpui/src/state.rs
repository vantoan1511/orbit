use orbit_core::events::{CoreEvent, TypedResourceBatch, WatchAction};
use orbit_core::kubernetes::models::PodInfo;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActivityTab {
    Pods,
    Workloads,
    Settings,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ClusterStatus {
    Connecting,
    Connected(String),
    MockFallback,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PodRow {
    pub name: String,
    pub namespace: String,
    pub status: String,
    pub restarts: i32,
    pub age: String,
    pub node: String,
}

impl From<&PodInfo> for PodRow {
    fn from(pod: &PodInfo) -> Self {
        Self {
            name: pod.name.clone(),
            namespace: pod.namespace.clone(),
            status: pod.status.clone(),
            restarts: pod.restarts,
            age: pod.age.clone(),
            node: pod.node.clone().unwrap_or_else(|| "-".to_string()),
        }
    }
}

pub struct OrbitGpuiState {
    pub active_tab: ActivityTab,
    pub cluster_status: ClusterStatus,
    pub pods: Vec<PodRow>,
}

impl Default for OrbitGpuiState {
    fn default() -> Self {
        Self::new()
    }
}

impl OrbitGpuiState {
    pub fn new() -> Self {
        Self {
            active_tab: ActivityTab::Pods,
            cluster_status: ClusterStatus::Connecting,
            pods: Vec::new(),
        }
    }

    pub fn select_tab(&mut self, tab: ActivityTab) {
        self.active_tab = tab;
    }

    pub fn fallback_to_mock_pods(&mut self) {
        self.cluster_status = ClusterStatus::MockFallback;
        self.pods = mock_pod_rows();
    }

    pub fn apply_core_event(&mut self, event: &CoreEvent) {
        match event {
            CoreEvent::EngineReady {
                active_cluster_id: Some(id),
                ..
            } => {
                if matches!(self.cluster_status, ClusterStatus::MockFallback) {
                    self.pods.clear();
                }
                self.cluster_status = ClusterStatus::Connected(id.clone());
            }
            CoreEvent::EngineReady {
                active_cluster_id: None,
                ..
            } => {
                self.fallback_to_mock_pods();
            }
            CoreEvent::ActiveClusterChanged {
                active_cluster_id: Some(id),
            } => {
                if self.cluster_status != ClusterStatus::Connected(id.clone()) {
                    self.pods.clear();
                }
                self.cluster_status = ClusterStatus::Connected(id.clone());
            }
            CoreEvent::ActiveClusterChanged {
                active_cluster_id: None,
            } => {
                self.fallback_to_mock_pods();
            }
            CoreEvent::ResourceBatchUpdated(TypedResourceBatch::Pods(actions)) => {
                for action in actions {
                    match action {
                        WatchAction::Applied(pod) => {
                            let row = PodRow::from(pod);
                            if let Some(existing) = self
                                .pods
                                .iter_mut()
                                .find(|p| p.namespace == row.namespace && p.name == row.name)
                            {
                                *existing = row;
                            } else {
                                self.pods.push(row);
                            }
                        }
                        WatchAction::Deleted(pod) => {
                            self.pods
                                .retain(|p| !(p.namespace == pod.namespace && p.name == pod.name));
                        }
                    }
                }
            }
            CoreEvent::ErrorOccurred { .. } => {
                if matches!(self.cluster_status, ClusterStatus::Connecting) {
                    self.fallback_to_mock_pods();
                }
            }
            _ => {}
        }
    }
}

fn mock_pod_rows() -> Vec<PodRow> {
    vec![
        PodRow {
            name: "coredns-5dd5756b68-7x9k2".to_string(),
            namespace: "kube-system".to_string(),
            status: "Running".to_string(),
            restarts: 0,
            age: "14d".to_string(),
            node: "orb-control-plane".to_string(),
        },
        PodRow {
            name: "orbit-api-gateway-6c8f9d4b-m2p9q".to_string(),
            namespace: "production".to_string(),
            status: "Running".to_string(),
            restarts: 0,
            age: "3d".to_string(),
            node: "orb-worker-01".to_string(),
        },
        PodRow {
            name: "metrics-collector-74b9d8c6-v8n4w".to_string(),
            namespace: "monitoring".to_string(),
            status: "Pending".to_string(),
            restarts: 1,
            age: "12m".to_string(),
            node: "orb-worker-02".to_string(),
        },
        PodRow {
            name: "cache-warmer-cron-289104-x5k1z".to_string(),
            namespace: "jobs".to_string(),
            status: "CrashLoopBackOff".to_string(),
            restarts: 6,
            age: "45m".to_string(),
            node: "orb-worker-01".to_string(),
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn make_pod(name: &str, namespace: &str, status: &str, restarts: i32) -> PodInfo {
        PodInfo {
            name: name.to_string(),
            namespace: namespace.to_string(),
            status: status.to_string(),
            age: "2d".to_string(),
            node: Some("node-1".to_string()),
            restarts,
            ..Default::default()
        }
    }

    #[test]
    fn test_initial_state_defaults_to_pods_tab_and_connecting() {
        let state = OrbitGpuiState::new();
        assert_eq!(state.active_tab, ActivityTab::Pods);
        assert_eq!(state.cluster_status, ClusterStatus::Connecting);
        assert!(state.pods.is_empty());
    }

    #[test]
    fn test_engine_ready_without_cluster_activates_mock_fallback() {
        let mut state = OrbitGpuiState::new();
        state.apply_core_event(&CoreEvent::EngineReady {
            status: "ready".to_string(),
            message: "ok".to_string(),
            clusters: vec![],
            active_cluster_id: None,
            settings: HashMap::new(),
        });

        assert_eq!(state.cluster_status, ClusterStatus::MockFallback);
        assert!(!state.pods.is_empty());
    }

    #[test]
    fn test_resource_batch_pods_applies_and_deletes_rows() {
        let mut state = OrbitGpuiState::new();
        state.apply_core_event(&CoreEvent::ActiveClusterChanged {
            active_cluster_id: Some("prod-east".to_string()),
        });
        assert_eq!(
            state.cluster_status,
            ClusterStatus::Connected("prod-east".to_string())
        );

        let pod_a = make_pod("api-server-1", "default", "Running", 0);
        let pod_b = make_pod("worker-2", "jobs", "Pending", 1);

        state.apply_core_event(&CoreEvent::ResourceBatchUpdated(
            TypedResourceBatch::Pods(vec![
                WatchAction::Applied(pod_a.clone()),
                WatchAction::Applied(pod_b),
            ]),
        ));
        assert_eq!(state.pods.len(), 2);

        let updated_a = make_pod("api-server-1", "default", "CrashLoopBackOff", 4);
        state.apply_core_event(&CoreEvent::ResourceBatchUpdated(
            TypedResourceBatch::Pods(vec![
                WatchAction::Applied(updated_a),
                WatchAction::Deleted(make_pod("worker-2", "jobs", "Pending", 1)),
            ]),
        ));

        assert_eq!(state.pods.len(), 1);
        assert_eq!(state.pods[0].name, "api-server-1");
        assert_eq!(state.pods[0].status, "CrashLoopBackOff");
        assert_eq!(state.pods[0].restarts, 4);
    }

    #[test]
    fn test_select_tab_updates_active_tab() {
        let mut state = OrbitGpuiState::new();
        state.select_tab(ActivityTab::Workloads);
        assert_eq!(state.active_tab, ActivityTab::Workloads);
        state.select_tab(ActivityTab::Settings);
        assert_eq!(state.active_tab, ActivityTab::Settings);
    }
}
