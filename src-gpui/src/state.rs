use orbit_core::kubernetes::manager::ClusterSwitchOutcome;
use orbit_core::kubernetes::models::{ClusterInfo, UserProfileInfo};
use orbit_core::kubernetes::overview::ClusterOverviewSummary;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ClusterConnectionState {
    Disconnected,
    Connecting {
        cluster_id: String,
        display_name: String,
    },
    Connected {
        cluster_id: String,
        display_name: String,
        k8s_version: Option<String>,
    },
    Unreachable {
        cluster_id: String,
        display_name: String,
        server_url: Option<String>,
        error_message: String,
    },
    Error {
        cluster_id: Option<String>,
        display_name: Option<String>,
        error_message: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ClusterBridgeCommand {
    LoadContexts,
    SwitchCluster { cluster_id: String },
    ReconnectActiveCluster,
    RefreshClusters,
    LoadOverview { cluster_id: String, generation: u64 },
}

#[derive(Debug, Clone)]
pub enum ClusterBridgeEvent {
    ContextsLoaded {
        clusters: Vec<ClusterInfo>,
        kubeconfig_error: Option<String>,
    },
    ClusterConnected {
        outcome: ClusterSwitchOutcome,
        clusters: Vec<ClusterInfo>,
        profile: UserProfileInfo,
    },
    ClusterUnreachable {
        outcome: ClusterSwitchOutcome,
        clusters: Vec<ClusterInfo>,
        profile: UserProfileInfo,
    },
    ConnectionError {
        cluster_id: Option<String>,
        message: String,
        clusters: Vec<ClusterInfo>,
    },
    OverviewLoaded {
        cluster_id: String,
        generation: u64,
        summary: ClusterOverviewSummary,
    },
    OverviewLoadFailed {
        cluster_id: String,
        generation: u64,
        message: String,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum OverviewLoadStatus {
    #[default]
    Idle,
    Loading,
    Loaded,
    Error,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ClusterOverviewState {
    pub cluster_id: Option<String>,
    pub generation: u64,
    pub status: OverviewLoadStatus,
    pub summary: Option<ClusterOverviewSummary>,
    pub error_message: Option<String>,
}

impl ClusterOverviewState {
    pub fn begin_load_for_cluster(&mut self, cluster_id: &str) -> ClusterBridgeCommand {
        self.generation += 1;
        self.cluster_id = Some(cluster_id.to_string());
        self.status = OverviewLoadStatus::Loading;
        self.summary = None;
        self.error_message = None;
        ClusterBridgeCommand::LoadOverview {
            cluster_id: cluster_id.to_string(),
            generation: self.generation,
        }
    }

    pub fn begin_refresh(&mut self) -> Option<ClusterBridgeCommand> {
        let cluster_id = self.cluster_id.clone()?;
        self.generation += 1;
        self.status = OverviewLoadStatus::Loading;
        self.error_message = None;
        Some(ClusterBridgeCommand::LoadOverview {
            cluster_id,
            generation: self.generation,
        })
    }

    pub fn reset_for_disconnected(&mut self) {
        self.generation += 1;
        self.cluster_id = None;
        self.status = OverviewLoadStatus::Idle;
        self.summary = None;
        self.error_message = None;
    }

    pub fn apply_loaded(
        &mut self,
        cluster_id: &str,
        generation: u64,
        summary: ClusterOverviewSummary,
    ) -> bool {
        if self.cluster_id.as_deref() == Some(cluster_id) && self.generation == generation {
            self.status = OverviewLoadStatus::Loaded;
            self.summary = Some(summary);
            self.error_message = None;
            true
        } else {
            false
        }
    }

    pub fn apply_failed(&mut self, cluster_id: &str, generation: u64, message: String) -> bool {
        if self.cluster_id.as_deref() == Some(cluster_id) && self.generation == generation {
            self.status = OverviewLoadStatus::Error;
            self.error_message = Some(message);
            true
        } else {
            false
        }
    }
}

#[derive(Debug, Clone)]
pub struct ClusterSwitcherState {
    pub clusters: Vec<ClusterInfo>,
    pub active_cluster_id: Option<String>,
    pub active_profile: Option<UserProfileInfo>,
    pub connection_state: ClusterConnectionState,
    pub dropdown_open: bool,
    pub kubeconfig_error: Option<String>,
    pub error_banner_dismissed: bool,
}

impl Default for ClusterSwitcherState {
    fn default() -> Self {
        Self::new()
    }
}

impl ClusterSwitcherState {
    pub fn new() -> Self {
        Self {
            clusters: Vec::new(),
            active_cluster_id: None,
            active_profile: None,
            connection_state: ClusterConnectionState::Disconnected,
            dropdown_open: false,
            kubeconfig_error: None,
            error_banner_dismissed: false,
        }
    }

    pub fn toggle_dropdown(&mut self) {
        self.dropdown_open = !self.dropdown_open;
    }

    pub fn close_dropdown(&mut self) {
        self.dropdown_open = false;
    }

    pub fn begin_switch(&mut self, cluster_id: &str) -> Option<ClusterBridgeCommand> {
        self.dropdown_open = false;
        if let ClusterConnectionState::Connecting {
            cluster_id: ref current,
            ..
        } = self.connection_state
            && current == cluster_id
        {
            return None;
        }

        let display_name = self.cluster_display_name(cluster_id);
        self.connection_state = ClusterConnectionState::Connecting {
            cluster_id: cluster_id.to_string(),
            display_name,
        };
        Some(ClusterBridgeCommand::SwitchCluster {
            cluster_id: cluster_id.to_string(),
        })
    }

    pub fn begin_reconnect(&mut self) -> Option<ClusterBridgeCommand> {
        if matches!(self.connection_state, ClusterConnectionState::Connecting { .. }) {
            return None;
        }

        let target_cluster_id = self.active_cluster_id.clone().or_else(|| match &self.connection_state {
            ClusterConnectionState::Unreachable { cluster_id, .. } => Some(cluster_id.clone()),
            ClusterConnectionState::Error { cluster_id, .. } => cluster_id.clone(),
            _ => None,
        })?;

        self.dropdown_open = false;
        let display_name = self.cluster_display_name(&target_cluster_id);
        self.connection_state = ClusterConnectionState::Connecting {
            cluster_id: target_cluster_id.clone(),
            display_name,
        };

        if self.active_cluster_id.is_some() {
            Some(ClusterBridgeCommand::ReconnectActiveCluster)
        } else {
            Some(ClusterBridgeCommand::SwitchCluster {
                cluster_id: target_cluster_id,
            })
        }
    }

    pub fn dismiss_error_banner(&mut self) {
        self.error_banner_dismissed = true;
    }

    pub fn apply_event(&mut self, event: ClusterBridgeEvent) {
        match event {
            ClusterBridgeEvent::ContextsLoaded {
                clusters,
                kubeconfig_error,
            } => {
                self.clusters = clusters;
                self.kubeconfig_error = kubeconfig_error;
                if self.kubeconfig_error.is_some() {
                    self.error_banner_dismissed = false;
                }
            }
            ClusterBridgeEvent::ClusterConnected {
                outcome,
                clusters,
                profile,
            } => {
                self.clusters = clusters;
                self.active_cluster_id = Some(outcome.context_name.clone());
                let display_name = self.cluster_display_name(&outcome.context_name);
                self.active_profile = Some(profile);
                self.connection_state = ClusterConnectionState::Connected {
                    cluster_id: outcome.context_name,
                    display_name,
                    k8s_version: outcome.k8s_version,
                };
                self.error_banner_dismissed = false;
            }
            ClusterBridgeEvent::ClusterUnreachable {
                outcome,
                clusters,
                profile,
            } => {
                self.clusters = clusters;
                self.active_cluster_id = Some(outcome.context_name.clone());
                let display_name = self.cluster_display_name(&outcome.context_name);
                let server_url = profile.server_url.clone().or_else(|| {
                    self.clusters
                        .iter()
                        .find(|c| c.id == outcome.context_name)
                        .and_then(|c| c.server_url.clone())
                });
                self.active_profile = Some(profile);
                self.connection_state = ClusterConnectionState::Unreachable {
                    cluster_id: outcome.context_name,
                    display_name,
                    server_url,
                    error_message: outcome
                        .error_message
                        .unwrap_or_else(|| "Cluster API server is unreachable".to_string()),
                };
                self.error_banner_dismissed = false;
            }
            ClusterBridgeEvent::ConnectionError {
                cluster_id,
                message,
                clusters,
            } => {
                self.clusters = clusters;
                let display_name = cluster_id
                    .as_deref()
                    .map(|id| self.cluster_display_name(id));
                self.connection_state = ClusterConnectionState::Error {
                    cluster_id,
                    display_name,
                    error_message: message,
                };
                self.error_banner_dismissed = false;
            }
            ClusterBridgeEvent::OverviewLoaded { .. }
            | ClusterBridgeEvent::OverviewLoadFailed { .. } => {}
        }
    }

    pub fn cluster_display_name(&self, cluster_id: &str) -> String {
        self.clusters
            .iter()
            .find(|c| c.id == cluster_id)
            .and_then(|c| {
                c.alias
                    .as_deref()
                    .map(str::trim)
                    .filter(|s| !s.is_empty())
                    .map(ToString::to_string)
                    .or_else(|| Some(c.name.clone()))
            })
            .unwrap_or_else(|| cluster_id.to_string())
    }

    pub fn active_cluster(&self) -> Option<&ClusterInfo> {
        let id = self.active_cluster_id.as_deref()?;
        self.clusters.iter().find(|c| c.id == id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_cluster(id: &str, alias: Option<&str>, server_url: Option<&str>) -> ClusterInfo {
        ClusterInfo {
            id: id.to_string(),
            name: id.to_string(),
            status: "offline".to_string(),
            alias: alias.map(ToString::to_string),
            tags: Vec::new(),
            color: None,
            description: None,
            server_url: server_url.map(ToString::to_string),
        }
    }

    fn sample_profile(context: &str, server_url: Option<&str>) -> UserProfileInfo {
        UserProfileInfo {
            active_context: Some(context.to_string()),
            user_name: Some("admin".to_string()),
            auth_type: "Certificate".to_string(),
            cluster_name: Some("prod-cluster".to_string()),
            server_url: server_url.map(ToString::to_string),
            kubeconfig_paths: vec!["~/.kube/config".to_string()],
            k8s_version: Some("Connected".to_string()),
        }
    }

    #[test]
    fn test_contexts_loaded_keeps_disconnected_state_and_captures_kubeconfig_error() {
        let mut state = ClusterSwitcherState::new();
        let clusters = vec![sample_cluster("ctx-1", Some("Prod US"), Some("https://10.0.0.1:6443"))];

        state.apply_event(ClusterBridgeEvent::ContextsLoaded {
            clusters: clusters.clone(),
            kubeconfig_error: Some("Failed to parse custom kubeconfig".to_string()),
        });

        assert_eq!(state.connection_state, ClusterConnectionState::Disconnected);
        assert_eq!(state.clusters, clusters);
        assert_eq!(
            state.kubeconfig_error.as_deref(),
            Some("Failed to parse custom kubeconfig")
        );
        assert!(!state.error_banner_dismissed);
    }

    #[test]
    fn test_begin_switch_closes_dropdown_and_enters_connecting_state() {
        let mut state = ClusterSwitcherState::new();
        state.clusters = vec![sample_cluster("ctx-1", Some("Prod US"), None)];
        state.dropdown_open = true;

        let cmd = state.begin_switch("ctx-1");
        assert!(!state.dropdown_open);
        assert_eq!(
            cmd,
            Some(ClusterBridgeCommand::SwitchCluster {
                cluster_id: "ctx-1".to_string()
            })
        );
        assert_eq!(
            state.connection_state,
            ClusterConnectionState::Connecting {
                cluster_id: "ctx-1".to_string(),
                display_name: "Prod US".to_string(),
            }
        );

        // Duplicate switch to same already-connecting context is ignored
        assert_eq!(state.begin_switch("ctx-1"), None);
    }

    #[test]
    fn test_apply_event_connected_updates_active_cluster_and_profile() {
        let mut state = ClusterSwitcherState::new();
        state.error_banner_dismissed = true;
        let clusters = vec![sample_cluster(
            "ctx-1",
            Some("Prod US"),
            Some("https://10.0.0.1:6443"),
        )];
        let profile = sample_profile("ctx-1", Some("https://10.0.0.1:6443"));

        state.apply_event(ClusterBridgeEvent::ClusterConnected {
            outcome: ClusterSwitchOutcome {
                context_name: "ctx-1".to_string(),
                healthy: true,
                k8s_version: Some("v1.31.2".to_string()),
                error_message: None,
            },
            clusters,
            profile: profile.clone(),
        });

        assert_eq!(state.active_cluster_id.as_deref(), Some("ctx-1"));
        assert!(!state.error_banner_dismissed);
        assert_eq!(
            state.active_profile.as_ref().and_then(|p| p.user_name.as_deref()),
            Some("admin")
        );
        assert_eq!(
            state.connection_state,
            ClusterConnectionState::Connected {
                cluster_id: "ctx-1".to_string(),
                display_name: "Prod US".to_string(),
                k8s_version: Some("v1.31.2".to_string()),
            }
        );
        assert_eq!(
            state.active_cluster().map(|c| c.id.as_str()),
            Some("ctx-1")
        );
    }

    #[test]
    fn test_apply_event_unreachable_and_connection_error_surface_diagnostics() {
        let mut state = ClusterSwitcherState::new();
        let clusters = vec![sample_cluster(
            "ctx-down",
            None,
            Some("http://127.0.0.1:1"),
        )];
        let profile = sample_profile("ctx-down", Some("http://127.0.0.1:1"));

        state.apply_event(ClusterBridgeEvent::ClusterUnreachable {
            outcome: ClusterSwitchOutcome {
                context_name: "ctx-down".to_string(),
                healthy: false,
                k8s_version: None,
                error_message: Some("connection refused".to_string()),
            },
            clusters: clusters.clone(),
            profile,
        });

        assert_eq!(
            state.connection_state,
            ClusterConnectionState::Unreachable {
                cluster_id: "ctx-down".to_string(),
                display_name: "ctx-down".to_string(),
                server_url: Some("http://127.0.0.1:1".to_string()),
                error_message: "connection refused".to_string(),
            }
        );

        let reconnect_cmd = state.begin_reconnect();
        assert_eq!(
            reconnect_cmd,
            Some(ClusterBridgeCommand::ReconnectActiveCluster)
        );

        state.apply_event(ClusterBridgeEvent::ConnectionError {
            cluster_id: Some("ctx-down".to_string()),
            message: "Invalid client certificate".to_string(),
            clusters,
        });

        assert!(!state.error_banner_dismissed);
        assert_eq!(
            state.connection_state,
            ClusterConnectionState::Error {
                cluster_id: Some("ctx-down".to_string()),
                display_name: Some("ctx-down".to_string()),
                error_message: "Invalid client certificate".to_string(),
            }
        );
    }

    #[test]
    fn test_cluster_display_name_prefers_non_empty_alias() {
        let mut state = ClusterSwitcherState::new();
        state.clusters = vec![
            sample_cluster("prod-ctx", Some("Prod US"), None),
            sample_cluster("dev-ctx", Some("   "), None),
            sample_cluster("staging-ctx", None, None),
        ];

        assert_eq!(state.cluster_display_name("prod-ctx"), "Prod US");
        assert_eq!(state.cluster_display_name("dev-ctx"), "dev-ctx");
        assert_eq!(state.cluster_display_name("staging-ctx"), "staging-ctx");
        assert_eq!(state.cluster_display_name("unknown-ctx"), "unknown-ctx");
    }

    #[test]
    fn test_cluster_overview_state_tracks_generation_clears_on_switch_and_ignores_stale_responses()
    {
        use orbit_core::kubernetes::overview::{ClusterOverallHealth, NodeHealthSummary};

        let mut overview = ClusterOverviewState::default();
        let summary_a = ClusterOverviewSummary {
            k8s_version: Some("v1.31.1".to_string()),
            overall_health: ClusterOverallHealth::Healthy,
            nodes: NodeHealthSummary {
                total: 3,
                ready: 3,
                ..Default::default()
            },
            ..Default::default()
        };
        let summary_b = ClusterOverviewSummary {
            k8s_version: Some("v1.30.2".to_string()),
            overall_health: ClusterOverallHealth::Degraded,
            nodes: NodeHealthSummary {
                total: 2,
                ready: 1,
                not_ready: 1,
                ..Default::default()
            },
            ..Default::default()
        };

        let cmd_a = overview.begin_load_for_cluster("ctx-a");
        assert_eq!(
            cmd_a,
            ClusterBridgeCommand::LoadOverview {
                cluster_id: "ctx-a".to_string(),
                generation: 1,
            }
        );
        assert_eq!(overview.status, OverviewLoadStatus::Loading);
        assert!(overview.summary.is_none());

        assert!(overview.apply_loaded("ctx-a", 1, summary_a.clone()));
        assert_eq!(overview.status, OverviewLoadStatus::Loaded);
        assert_eq!(overview.summary, Some(summary_a.clone()));

        let cmd_b = overview.begin_load_for_cluster("ctx-b");
        assert_eq!(
            cmd_b,
            ClusterBridgeCommand::LoadOverview {
                cluster_id: "ctx-b".to_string(),
                generation: 2,
            }
        );
        assert_eq!(overview.generation, 2);
        assert!(overview.summary.is_none());

        assert!(!overview.apply_loaded("ctx-a", 1, summary_a));
        assert!(overview.summary.is_none());

        assert!(overview.apply_loaded("ctx-b", 2, summary_b.clone()));
        assert_eq!(overview.summary, Some(summary_b));

        overview.reset_for_disconnected();
        assert_eq!(overview.generation, 3);
        assert!(overview.cluster_id.is_none());
        assert!(overview.summary.is_none());
        assert_eq!(overview.status, OverviewLoadStatus::Idle);
    }
}
