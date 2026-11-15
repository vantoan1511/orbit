use std::collections::HashSet;

use orbit_core::config::{OrbitConfig, SelectedNamespacesMap};
use orbit_core::kubernetes::manager::ClusterSwitchOutcome;
use orbit_core::kubernetes::models::{
    ClusterInfo, DeploymentInfo, EventInfo, NamespaceInfo, NodeInfo, UserProfileInfo,
};
use orbit_core::kubernetes::overview::ClusterOverviewSummary;

use crate::components::data_table::DataTableState;

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
    LoadNamespaces { cluster_id: String, generation: u64 },
    LoadDeployments { cluster_id: String, generation: u64 },
    LoadNodes { cluster_id: String, generation: u64 },
    LoadEvents { cluster_id: String, generation: u64 },
    SaveNamespaceSelection { cluster_id: String, namespaces: Vec<String> },
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
    NamespacesLoaded {
        cluster_id: String,
        generation: u64,
        namespaces: Vec<NamespaceInfo>,
    },
    NamespacesLoadFailed {
        cluster_id: String,
        generation: u64,
        message: String,
    },
    DeploymentsLoaded {
        cluster_id: String,
        generation: u64,
        deployments: Vec<DeploymentInfo>,
    },
    DeploymentsLoadFailed {
        cluster_id: String,
        generation: u64,
        message: String,
    },
    NodesLoaded {
        cluster_id: String,
        generation: u64,
        nodes: Vec<NodeInfo>,
    },
    NodesLoadFailed {
        cluster_id: String,
        generation: u64,
        message: String,
    },
    EventsLoaded {
        cluster_id: String,
        generation: u64,
        events: Vec<EventInfo>,
    },
    EventsLoadFailed {
        cluster_id: String,
        generation: u64,
        message: String,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum NamespaceLoadStatus {
    #[default]
    Idle,
    Loading,
    Loaded,
    Error,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct NamespaceFilterState {
    pub cluster_id: Option<String>,
    pub generation: u64,
    pub status: NamespaceLoadStatus,
    pub namespaces: Vec<NamespaceInfo>,
    pub error_message: Option<String>,
    pub selected_by_cluster: SelectedNamespacesMap,
    pub dropdown_open: bool,
    pub search_query: String,
}

impl NamespaceFilterState {
    pub fn from_config(config: &OrbitConfig) -> Self {
        Self {
            selected_by_cluster: config.selected_namespaces_map(),
            ..Default::default()
        }
    }

    pub fn needs_load_for_cluster(&self, cluster_id: &str) -> bool {
        self.cluster_id.as_deref() != Some(cluster_id) || self.status == NamespaceLoadStatus::Idle
    }

    pub fn begin_load_for_cluster(&mut self, cluster_id: &str) -> ClusterBridgeCommand {
        self.generation += 1;
        self.cluster_id = Some(cluster_id.to_string());
        self.status = NamespaceLoadStatus::Loading;
        self.namespaces.clear();
        self.error_message = None;
        self.dropdown_open = false;
        self.search_query.clear();
        ClusterBridgeCommand::LoadNamespaces {
            cluster_id: cluster_id.to_string(),
            generation: self.generation,
        }
    }

    pub fn begin_refresh(&mut self) -> Option<ClusterBridgeCommand> {
        let cluster_id = self.cluster_id.clone()?;
        self.generation += 1;
        self.status = NamespaceLoadStatus::Loading;
        self.error_message = None;
        Some(ClusterBridgeCommand::LoadNamespaces {
            cluster_id,
            generation: self.generation,
        })
    }

    pub fn reset_for_disconnected(&mut self) {
        self.generation += 1;
        self.cluster_id = None;
        self.status = NamespaceLoadStatus::Idle;
        self.namespaces.clear();
        self.error_message = None;
        self.dropdown_open = false;
        self.search_query.clear();
    }

    pub fn apply_loaded(
        &mut self,
        cluster_id: &str,
        generation: u64,
        namespaces: Vec<NamespaceInfo>,
    ) -> (bool, Option<Vec<String>>) {
        if self.cluster_id.as_deref() != Some(cluster_id) || self.generation != generation {
            return (false, None);
        }
        self.status = NamespaceLoadStatus::Loaded;
        self.error_message = None;

        let valid_names: HashSet<&str> = namespaces.iter().map(|ns| ns.name.as_str()).collect();
        let mut pruned_selection = None;
        if let Some(current) = self.selected_by_cluster.get(cluster_id).cloned() {
            let filtered: Vec<String> = current
                .iter()
                .filter(|name| valid_names.contains(name.as_str()))
                .cloned()
                .collect();
            if filtered.len() != current.len() {
                if filtered.is_empty() {
                    self.selected_by_cluster.remove(cluster_id);
                } else {
                    self.selected_by_cluster
                        .insert(cluster_id.to_string(), filtered.clone());
                }
                pruned_selection = Some(filtered);
            }
        }

        self.namespaces = namespaces;
        (true, pruned_selection)
    }

    pub fn apply_failed(&mut self, cluster_id: &str, generation: u64, message: String) -> bool {
        if self.cluster_id.as_deref() != Some(cluster_id) || self.generation != generation {
            return false;
        }
        self.status = NamespaceLoadStatus::Error;
        self.error_message = Some(message);
        true
    }

    pub fn active_selection(&self) -> &[String] {
        self.cluster_id
            .as_deref()
            .and_then(|id| self.selected_by_cluster.get(id))
            .map(Vec::as_slice)
            .unwrap_or(&[])
    }

    pub fn is_all_namespaces(&self) -> bool {
        self.active_selection().is_empty()
    }

    pub fn select_all_namespaces(&mut self) -> Option<Vec<String>> {
        let cluster_id = self.cluster_id.as_deref()?;
        if self.selected_by_cluster.remove(cluster_id).is_some() {
            Some(Vec::new())
        } else {
            None
        }
    }

    pub fn toggle_namespace(&mut self, name: &str) -> Option<Vec<String>> {
        let cluster_id = self.cluster_id.clone()?;
        let trimmed = name.trim();
        if trimmed.is_empty() {
            return None;
        }

        let entry = self.selected_by_cluster.entry(cluster_id.clone()).or_default();
        if let Some(pos) = entry.iter().position(|n| n == trimmed) {
            entry.remove(pos);
        } else {
            entry.push(trimmed.to_string());
        }

        let updated = entry.clone();
        if updated.is_empty() {
            self.selected_by_cluster.remove(&cluster_id);
        }
        Some(updated)
    }

    pub fn filtered_namespaces(&self) -> Vec<&NamespaceInfo> {
        let query = self.search_query.trim().to_lowercase();
        if query.is_empty() {
            self.namespaces.iter().collect()
        } else {
            self.namespaces
                .iter()
                .filter(|ns| ns.name.to_lowercase().contains(&query))
                .collect()
        }
    }

    pub fn matches_resource_namespace(&self, resource_namespace: Option<&str>) -> bool {
        let selected = self.active_selection();
        if selected.is_empty() {
            return true;
        }
        match resource_namespace {
            Some(ns) => selected.iter().any(|s| s == ns),
            None => false,
        }
    }

    pub fn summary_label(&self) -> String {
        let selected = self.active_selection();
        match selected.len() {
            0 => "All Namespaces".to_string(),
            1 => selected[0].clone(),
            n => format!("{} +{}", selected[0], n - 1),
        }
    }

    pub fn toggle_dropdown(&mut self) {
        self.dropdown_open = !self.dropdown_open;
        if !self.dropdown_open {
            self.search_query.clear();
        }
    }

    pub fn close_dropdown(&mut self) {
        self.dropdown_open = false;
        self.search_query.clear();
    }

    pub fn handle_search_key(&mut self, key: &str, key_char: Option<&str>) -> bool {
        if key == "backspace" {
            self.search_query.pop();
            return true;
        }
        if key == "escape" {
            self.close_dropdown();
            return true;
        }
        if let Some(ch) = key_char
            && !ch.chars().any(char::is_control)
        {
            self.search_query.push_str(ch);
            return true;
        }
        false
    }
}

#[derive(Debug, Clone, Default)]
pub struct NamespacesViewState {
    pub table: DataTableState,
}

impl NamespacesViewState {
    pub fn reset_for_cluster(&mut self) {
        self.table.selected_row_key = None;
        self.table.loading = false;
    }

    pub fn reset_for_disconnected(&mut self) {
        self.table.selected_row_key = None;
        self.table.loading = false;
    }
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum DeploymentsLoadStatus {
    #[default]
    Idle,
    Loading,
    Loaded,
    Error,
}

#[derive(Debug, Clone, Default)]
pub struct DeploymentsState {
    pub cluster_id: Option<String>,
    pub generation: u64,
    pub status: DeploymentsLoadStatus,
    pub deployments: Vec<DeploymentInfo>,
    pub error_message: Option<String>,
    pub table: DataTableState,
}

impl DeploymentsState {
    pub fn begin_load_for_cluster(&mut self, cluster_id: &str) -> ClusterBridgeCommand {
        let switching_cluster = self.cluster_id.as_deref() != Some(cluster_id);
        self.generation += 1;
        self.cluster_id = Some(cluster_id.to_string());
        self.status = DeploymentsLoadStatus::Loading;
        self.error_message = None;
        if switching_cluster {
            self.deployments.clear();
            self.table.selected_row_key = None;
        }
        self.table.loading = self.deployments.is_empty();
        ClusterBridgeCommand::LoadDeployments {
            cluster_id: cluster_id.to_string(),
            generation: self.generation,
        }
    }

    pub fn begin_refresh(&mut self) -> Option<ClusterBridgeCommand> {
        let cluster_id = self.cluster_id.clone()?;
        self.generation += 1;
        self.status = DeploymentsLoadStatus::Loading;
        self.error_message = None;
        self.table.loading = self.deployments.is_empty();
        Some(ClusterBridgeCommand::LoadDeployments {
            cluster_id,
            generation: self.generation,
        })
    }

    pub fn reset_for_cluster(&mut self, cluster_id: &str) {
        self.generation += 1;
        self.cluster_id = Some(cluster_id.to_string());
        self.status = DeploymentsLoadStatus::Idle;
        self.deployments.clear();
        self.error_message = None;
        self.table.selected_row_key = None;
        self.table.loading = false;
    }

    pub fn reset_for_disconnected(&mut self) {
        self.generation += 1;
        self.cluster_id = None;
        self.status = DeploymentsLoadStatus::Idle;
        self.deployments.clear();
        self.error_message = None;
        self.table.selected_row_key = None;
        self.table.loading = false;
    }

    pub fn apply_loaded(
        &mut self,
        cluster_id: &str,
        generation: u64,
        deployments: Vec<DeploymentInfo>,
    ) -> bool {
        if self.cluster_id.as_deref() == Some(cluster_id) && self.generation == generation {
            self.status = DeploymentsLoadStatus::Loaded;
            self.deployments = deployments;
            self.error_message = None;
            self.table.loading = false;
            true
        } else {
            false
        }
    }

    pub fn apply_failed(&mut self, cluster_id: &str, generation: u64, message: String) -> bool {
        if self.cluster_id.as_deref() == Some(cluster_id) && self.generation == generation {
            self.status = DeploymentsLoadStatus::Error;
            self.error_message = Some(message);
            self.table.loading = false;
            true
        } else {
            false
        }
    }

    pub fn needs_load_for_cluster(&self, cluster_id: &str) -> bool {
        self.cluster_id.as_deref() != Some(cluster_id)
            || self.status == DeploymentsLoadStatus::Idle
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum NodesLoadStatus {
    #[default]
    Idle,
    Loading,
    Loaded,
    Error,
}

#[derive(Debug, Clone, Default)]
pub struct NodesState {
    pub cluster_id: Option<String>,
    pub generation: u64,
    pub status: NodesLoadStatus,
    pub nodes: Vec<NodeInfo>,
    pub error_message: Option<String>,
    pub table: DataTableState,
}

impl NodesState {
    pub fn begin_load_for_cluster(&mut self, cluster_id: &str) -> ClusterBridgeCommand {
        let switching_cluster = self.cluster_id.as_deref() != Some(cluster_id);
        self.generation += 1;
        self.cluster_id = Some(cluster_id.to_string());
        self.status = NodesLoadStatus::Loading;
        self.error_message = None;
        if switching_cluster {
            self.nodes.clear();
            self.table.selected_row_key = None;
        }
        self.table.loading = self.nodes.is_empty();
        ClusterBridgeCommand::LoadNodes {
            cluster_id: cluster_id.to_string(),
            generation: self.generation,
        }
    }

    pub fn begin_refresh(&mut self) -> Option<ClusterBridgeCommand> {
        let cluster_id = self.cluster_id.clone()?;
        self.generation += 1;
        self.status = NodesLoadStatus::Loading;
        self.error_message = None;
        self.table.loading = self.nodes.is_empty();
        Some(ClusterBridgeCommand::LoadNodes {
            cluster_id,
            generation: self.generation,
        })
    }

    pub fn reset_for_cluster(&mut self, cluster_id: &str) {
        self.generation += 1;
        self.cluster_id = Some(cluster_id.to_string());
        self.status = NodesLoadStatus::Idle;
        self.nodes.clear();
        self.error_message = None;
        self.table.selected_row_key = None;
        self.table.loading = false;
    }

    pub fn reset_for_disconnected(&mut self) {
        self.generation += 1;
        self.cluster_id = None;
        self.status = NodesLoadStatus::Idle;
        self.nodes.clear();
        self.error_message = None;
        self.table.selected_row_key = None;
        self.table.loading = false;
    }

    pub fn apply_loaded(
        &mut self,
        cluster_id: &str,
        generation: u64,
        nodes: Vec<NodeInfo>,
    ) -> bool {
        if self.cluster_id.as_deref() == Some(cluster_id) && self.generation == generation {
            self.status = NodesLoadStatus::Loaded;
            self.nodes = nodes;
            self.error_message = None;
            self.table.loading = false;
            true
        } else {
            false
        }
    }

    pub fn apply_failed(&mut self, cluster_id: &str, generation: u64, message: String) -> bool {
        if self.cluster_id.as_deref() == Some(cluster_id) && self.generation == generation {
            self.status = NodesLoadStatus::Error;
            self.error_message = Some(message);
            self.table.loading = false;
            true
        } else {
            false
        }
    }

    pub fn needs_load_for_cluster(&self, cluster_id: &str) -> bool {
        self.cluster_id.as_deref() != Some(cluster_id) || self.status == NodesLoadStatus::Idle
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum EventsLoadStatus {
    #[default]
    Idle,
    Loading,
    Loaded,
    Error,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum EventTypeFilter {
    #[default]
    All,
    Normal,
    Warning,
}

impl EventTypeFilter {
    pub fn matches(self, event_type: &str) -> bool {
        match self {
            Self::All => true,
            Self::Normal => event_type.eq_ignore_ascii_case("normal"),
            Self::Warning => event_type.eq_ignore_ascii_case("warning"),
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct EventsState {
    pub cluster_id: Option<String>,
    pub generation: u64,
    pub status: EventsLoadStatus,
    pub events: Vec<EventInfo>,
    pub type_filter: EventTypeFilter,
    pub error_message: Option<String>,
    pub table: DataTableState,
}

impl EventsState {
    pub fn begin_load_for_cluster(&mut self, cluster_id: &str) -> ClusterBridgeCommand {
        let switching_cluster = self.cluster_id.as_deref() != Some(cluster_id);
        self.generation += 1;
        self.cluster_id = Some(cluster_id.to_string());
        self.status = EventsLoadStatus::Loading;
        self.error_message = None;
        if switching_cluster {
            self.events.clear();
            self.type_filter = EventTypeFilter::All;
            self.table.selected_row_key = None;
        }
        self.table.loading = self.events.is_empty();
        ClusterBridgeCommand::LoadEvents {
            cluster_id: cluster_id.to_string(),
            generation: self.generation,
        }
    }

    pub fn begin_refresh(&mut self) -> Option<ClusterBridgeCommand> {
        let cluster_id = self.cluster_id.clone()?;
        self.generation += 1;
        self.status = EventsLoadStatus::Loading;
        self.error_message = None;
        self.table.loading = self.events.is_empty();
        Some(ClusterBridgeCommand::LoadEvents {
            cluster_id,
            generation: self.generation,
        })
    }

    pub fn reset_for_cluster(&mut self, cluster_id: &str) {
        self.generation += 1;
        self.cluster_id = Some(cluster_id.to_string());
        self.status = EventsLoadStatus::Idle;
        self.events.clear();
        self.type_filter = EventTypeFilter::All;
        self.error_message = None;
        self.table.selected_row_key = None;
        self.table.loading = false;
    }

    pub fn reset_for_disconnected(&mut self) {
        self.generation += 1;
        self.cluster_id = None;
        self.status = EventsLoadStatus::Idle;
        self.events.clear();
        self.type_filter = EventTypeFilter::All;
        self.error_message = None;
        self.table.selected_row_key = None;
        self.table.loading = false;
    }

    pub fn apply_loaded(
        &mut self,
        cluster_id: &str,
        generation: u64,
        events: Vec<EventInfo>,
    ) -> bool {
        if self.cluster_id.as_deref() == Some(cluster_id) && self.generation == generation {
            self.status = EventsLoadStatus::Loaded;
            self.events = events;
            self.error_message = None;
            self.table.loading = false;
            true
        } else {
            false
        }
    }

    pub fn apply_failed(&mut self, cluster_id: &str, generation: u64, message: String) -> bool {
        if self.cluster_id.as_deref() == Some(cluster_id) && self.generation == generation {
            self.status = EventsLoadStatus::Error;
            self.error_message = Some(message);
            self.table.loading = false;
            true
        } else {
            false
        }
    }

    pub fn needs_load_for_cluster(&self, cluster_id: &str) -> bool {
        self.cluster_id.as_deref() != Some(cluster_id) || self.status == EventsLoadStatus::Idle
    }

    pub fn set_type_filter(&mut self, filter: EventTypeFilter) {
        self.type_filter = filter;
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
            | ClusterBridgeEvent::OverviewLoadFailed { .. }
            | ClusterBridgeEvent::NamespacesLoaded { .. }
            | ClusterBridgeEvent::NamespacesLoadFailed { .. }
            | ClusterBridgeEvent::DeploymentsLoaded { .. }
            | ClusterBridgeEvent::DeploymentsLoadFailed { .. }
            | ClusterBridgeEvent::NodesLoaded { .. }
            | ClusterBridgeEvent::NodesLoadFailed { .. }
            | ClusterBridgeEvent::EventsLoaded { .. }
            | ClusterBridgeEvent::EventsLoadFailed { .. } => {}
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

    fn sample_namespace(name: &str, is_system: bool) -> NamespaceInfo {
        NamespaceInfo {
            name: name.to_string(),
            status: "Active".to_string(),
            is_system,
            age: "1d".to_string(),
            labels: Default::default(),
            annotations: Default::default(),
            uid: format!("uid-{name}"),
            created: "2026-01-01T00:00:00Z".to_string(),
        }
    }

    #[test]
    fn test_namespace_filter_state_tracks_generation_persists_per_cluster_and_prunes_stale_names() {
        let mut config = OrbitConfig::default();
        config.update_selected_namespaces_for_cluster(
            "ctx-a",
            vec!["default".to_string(), "deleted-ns".to_string()],
        );
        config.update_selected_namespaces_for_cluster("ctx-b", vec!["kube-system".to_string()]);

        let mut ns_state = NamespaceFilterState::from_config(&config);
        assert_eq!(
            ns_state.selected_by_cluster.get("ctx-a"),
            Some(&vec!["default".to_string(), "deleted-ns".to_string()])
        );
        assert_eq!(
            ns_state.selected_by_cluster.get("ctx-b"),
            Some(&vec!["kube-system".to_string()])
        );

        let cmd_a = ns_state.begin_load_for_cluster("ctx-a");
        assert_eq!(
            cmd_a,
            ClusterBridgeCommand::LoadNamespaces {
                cluster_id: "ctx-a".to_string(),
                generation: 1,
            }
        );
        assert_eq!(ns_state.status, NamespaceLoadStatus::Loading);

        let ns_list = vec![
            sample_namespace("default", true),
            sample_namespace("kube-system", true),
            sample_namespace("payments", false),
        ];
        let (applied, pruned) = ns_state.apply_loaded("ctx-a", 1, ns_list.clone());
        assert!(applied);
        assert_eq!(pruned, Some(vec!["default".to_string()]));
        assert_eq!(ns_state.active_selection(), &["default".to_string()]);

        let cmd_b = ns_state.begin_load_for_cluster("ctx-b");
        assert_eq!(
            cmd_b,
            ClusterBridgeCommand::LoadNamespaces {
                cluster_id: "ctx-b".to_string(),
                generation: 2,
            }
        );
        assert_eq!(ns_state.apply_loaded("ctx-a", 1, ns_list), (false, None));
        assert_eq!(ns_state.active_selection(), &["kube-system".to_string()]);
        assert_eq!(
            ns_state.selected_by_cluster.get("ctx-a"),
            Some(&vec!["default".to_string()])
        );
    }

    #[test]
    fn test_namespace_filter_state_multi_select_search_and_resource_matching() {
        let mut ns_state = NamespaceFilterState::default();
        ns_state.begin_load_for_cluster("ctx-a");
        let ns_list = vec![
            sample_namespace("default", true),
            sample_namespace("kube-system", true),
            sample_namespace("monitoring", true),
            sample_namespace("payments", false),
        ];
        ns_state.apply_loaded("ctx-a", 1, ns_list);

        assert!(ns_state.is_all_namespaces());
        assert_eq!(ns_state.summary_label(), "All Namespaces");
        assert!(ns_state.matches_resource_namespace(Some("payments")));
        assert!(ns_state.matches_resource_namespace(None));

        assert_eq!(
            ns_state.toggle_namespace("payments"),
            Some(vec!["payments".to_string()])
        );
        assert_eq!(ns_state.summary_label(), "payments");
        assert!(ns_state.matches_resource_namespace(Some("payments")));
        assert!(!ns_state.matches_resource_namespace(Some("default")));
        assert!(!ns_state.matches_resource_namespace(None));

        assert_eq!(
            ns_state.toggle_namespace("default"),
            Some(vec!["payments".to_string(), "default".to_string()])
        );
        assert_eq!(ns_state.summary_label(), "payments +1");
        assert!(ns_state.matches_resource_namespace(Some("payments")));
        assert!(ns_state.matches_resource_namespace(Some("default")));

        assert_eq!(ns_state.select_all_namespaces(), Some(Vec::new()));
        assert!(ns_state.is_all_namespaces());
        assert_eq!(ns_state.summary_label(), "All Namespaces");

        assert!(ns_state.handle_search_key("p", Some("P")));
        assert!(ns_state.handle_search_key("a", Some("a")));
        assert!(ns_state.handle_search_key("y", Some("y")));
        assert!(ns_state.handle_search_key("z", Some("z")));
        assert!(ns_state.handle_search_key("backspace", None));
        assert_eq!(ns_state.search_query, "Pay");

        let filtered: Vec<&str> = ns_state
            .filtered_namespaces()
            .into_iter()
            .map(|ns| ns.name.as_str())
            .collect();
        assert_eq!(filtered, vec!["payments"]);
    }

    fn sample_deployment(name: &str, namespace: &str) -> DeploymentInfo {
        DeploymentInfo {
            kind: "Deployment".to_string(),
            name: name.to_string(),
            namespace: namespace.to_string(),
            status: "Running".to_string(),
            conditions: None,
            replicas: orbit_core::kubernetes::models::Replicas {
                current: 2,
                desired: 2,
            },
            available: 2,
            up_to_date: 2,
            age: "1h".to_string(),
            images: vec!["nginx:1.25".to_string()],
            containers: Vec::new(),
            strategy: Some("RollingUpdate".to_string()),
            min_ready_seconds: 0,
            revision_history: Some(10),
            labels: Default::default(),
            annotations: Default::default(),
            ports: vec![80],
        }
    }

    #[test]
    fn test_deployments_state_generation_guard_and_reset() {
        let mut state = DeploymentsState::default();
        assert!(state.needs_load_for_cluster("ctx-a"));

        let cmd_a = state.begin_load_for_cluster("ctx-a");
        assert_eq!(
            cmd_a,
            ClusterBridgeCommand::LoadDeployments {
                cluster_id: "ctx-a".to_string(),
                generation: 1,
            }
        );
        assert_eq!(state.status, DeploymentsLoadStatus::Loading);
        assert!(state.table.loading);
        assert!(!state.needs_load_for_cluster("ctx-a"));

        // Stale cluster or generation is ignored.
        assert!(!state.apply_loaded(
            "ctx-b",
            1,
            vec![sample_deployment("wrong-cluster", "default")],
        ));
        assert!(!state.apply_loaded(
            "ctx-a",
            99,
            vec![sample_deployment("wrong-gen", "default")],
        ));
        assert!(state.deployments.is_empty());

        // Matching (cluster_id, generation) succeeds.
        assert!(state.apply_loaded(
            "ctx-a",
            1,
            vec![sample_deployment("api", "default")],
        ));
        assert_eq!(state.status, DeploymentsLoadStatus::Loaded);
        assert!(!state.table.loading);
        assert_eq!(state.deployments.len(), 1);
        assert_eq!(state.deployments[0].name, "api");

        // Refresh keeps existing rows while loading.
        state.table.selected_row_key = Some("default/api".to_string());
        let refresh_cmd = state.begin_refresh().expect("refresh command");
        assert_eq!(
            refresh_cmd,
            ClusterBridgeCommand::LoadDeployments {
                cluster_id: "ctx-a".to_string(),
                generation: 2,
            }
        );
        assert_eq!(state.status, DeploymentsLoadStatus::Loading);
        assert_eq!(state.deployments.len(), 1);

        assert!(state.apply_failed("ctx-a", 2, "timeout".to_string()));
        assert_eq!(state.status, DeploymentsLoadStatus::Error);
        assert_eq!(state.error_message.as_deref(), Some("timeout"));

        // Reset for new cluster clears rows, selection, and returns to Idle.
        state.reset_for_cluster("ctx-b");
        assert_eq!(state.cluster_id.as_deref(), Some("ctx-b"));
        assert_eq!(state.status, DeploymentsLoadStatus::Idle);
        assert!(state.deployments.is_empty());
        assert!(state.error_message.is_none());
        assert!(state.table.selected_row_key.is_none());
        assert!(state.needs_load_for_cluster("ctx-b"));

        // Disconnect clears cluster_id and invalidates in-flight replies.
        let gen_before_disconnect = state.generation;
        state.reset_for_disconnected();
        assert!(state.cluster_id.is_none());
        assert_eq!(state.status, DeploymentsLoadStatus::Idle);
        assert!(state.generation > gen_before_disconnect);
    }

    fn sample_node(name: &str, status: &str, is_cordoned: bool) -> NodeInfo {
        NodeInfo {
            name: name.to_string(),
            status: status.to_string(),
            role: "worker".to_string(),
            version: "v1.30.2".to_string(),
            cpu_pct: 25.0,
            cpu_used: "1".to_string(),
            cpu_total: "4".to_string(),
            mem_pct: 50.0,
            mem_used: "8".to_string(),
            mem_total: "16".to_string(),
            pods_count: 12,
            pods_limit: 110,
            uptime: "4d".to_string(),
            created_at: None,
            labels: Vec::new(),
            labels_map: Default::default(),
            annotations: Default::default(),
            is_cordoned,
            node_info: None,
            addresses: Vec::new(),
            conditions: Vec::new(),
            taints: Vec::new(),
            capacity: None,
            allocatable: None,
            images_count: 0,
        }
    }

    #[test]
    fn test_nodes_state_tracks_generations_refresh_and_cluster_switches() {
        let mut state = NodesState::default();
        assert!(state.needs_load_for_cluster("ctx-a"));

        let cmd_a = state.begin_load_for_cluster("ctx-a");
        assert_eq!(
            cmd_a,
            ClusterBridgeCommand::LoadNodes {
                cluster_id: "ctx-a".to_string(),
                generation: 1,
            }
        );
        assert_eq!(state.status, NodesLoadStatus::Loading);
        assert!(state.table.loading);
        assert!(!state.needs_load_for_cluster("ctx-a"));

        // Stale cluster or generation is ignored.
        assert!(!state.apply_loaded("ctx-b", 1, vec![sample_node("wrong-cluster", "Ready", false)]));
        assert!(!state.apply_loaded("ctx-a", 99, vec![sample_node("wrong-gen", "Ready", false)]));
        assert!(!state.apply_failed("ctx-b", 1, "wrong-cluster".to_string()));
        assert!(!state.apply_failed("ctx-a", 99, "wrong-gen".to_string()));
        assert!(state.nodes.is_empty());

        // Matching (cluster_id, generation) succeeds.
        assert!(state.apply_loaded("ctx-a", 1, vec![sample_node("node-1", "Ready", false)]));
        assert_eq!(state.status, NodesLoadStatus::Loaded);
        assert!(!state.table.loading);
        assert_eq!(state.nodes.len(), 1);
        assert_eq!(state.nodes[0].name, "node-1");

        // Refresh keeps existing rows while loading.
        state.table.selected_row_key = Some("node-1".to_string());
        let refresh_cmd = state.begin_refresh().expect("refresh command");
        assert_eq!(
            refresh_cmd,
            ClusterBridgeCommand::LoadNodes {
                cluster_id: "ctx-a".to_string(),
                generation: 2,
            }
        );
        assert_eq!(state.status, NodesLoadStatus::Loading);
        assert!(!state.table.loading);
        assert_eq!(state.nodes.len(), 1);

        assert!(state.apply_failed("ctx-a", 2, "timeout".to_string()));
        assert_eq!(state.status, NodesLoadStatus::Error);
        assert_eq!(state.error_message.as_deref(), Some("timeout"));

        // Reset for new cluster clears rows, selection, and returns to Idle.
        state.reset_for_cluster("ctx-b");
        assert_eq!(state.cluster_id.as_deref(), Some("ctx-b"));
        assert_eq!(state.status, NodesLoadStatus::Idle);
        assert!(state.nodes.is_empty());
        assert!(state.error_message.is_none());
        assert!(state.table.selected_row_key.is_none());
        assert!(state.needs_load_for_cluster("ctx-b"));

        // Disconnect clears cluster_id and invalidates in-flight replies.
        let gen_before_disconnect = state.generation;
        state.reset_for_disconnected();
        assert!(state.cluster_id.is_none());
        assert_eq!(state.status, NodesLoadStatus::Idle);
        assert!(state.generation > gen_before_disconnect);
    }

    #[test]
    fn test_namespaces_view_state_and_filter_needs_load() {
        let mut filter = NamespaceFilterState::default();
        assert!(filter.needs_load_for_cluster("ctx-a"));

        filter.begin_load_for_cluster("ctx-a");
        assert!(!filter.needs_load_for_cluster("ctx-a"));
        assert!(filter.needs_load_for_cluster("ctx-b"));

        let mut view_state = NamespacesViewState::default();
        view_state.table.selected_row_key = Some("default".into());
        view_state.table.loading = true;

        view_state.reset_for_cluster();
        assert!(view_state.table.selected_row_key.is_none());
        assert!(!view_state.table.loading);

        view_state.table.selected_row_key = Some("kube-system".into());
        view_state.table.loading = true;

        view_state.reset_for_disconnected();
        assert!(view_state.table.selected_row_key.is_none());
        assert!(!view_state.table.loading);
    }

    fn sample_event(uid: &str, event_type: &str, reason: &str, namespace: &str) -> EventInfo {
        EventInfo {
            uid: uid.to_string(),
            time: "2m".to_string(),
            r#type: event_type.to_string(),
            reason: reason.to_string(),
            object_name: "api-pod".to_string(),
            object_kind: "Pod".to_string(),
            message: "Event message".to_string(),
            namespace: namespace.to_string(),
            source: "kubelet".to_string(),
            first_seen: "Nov 15, 2026, 07:00 PM".to_string(),
            last_seen: "Nov 15, 2026, 07:05 PM".to_string(),
            count: 1,
            labels: Default::default(),
            annotations: Default::default(),
        }
    }

    #[test]
    fn test_events_state_lifecycle_and_generation_guard() {
        let mut state = EventsState::default();
        assert!(state.needs_load_for_cluster("cluster-a"));

        let cmd = state.begin_load_for_cluster("cluster-a");
        assert_eq!(
            cmd,
            ClusterBridgeCommand::LoadEvents {
                cluster_id: "cluster-a".to_string(),
                generation: 1,
            }
        );
        assert_eq!(state.status, EventsLoadStatus::Loading);
        assert!(state.table.loading);
        assert!(!state.needs_load_for_cluster("cluster-a"));

        // Stale cluster or generation is ignored.
        assert!(!state.apply_loaded(
            "cluster-b",
            1,
            vec![sample_event("ev-wrong-cluster", "Normal", "Scheduled", "default")],
        ));
        assert!(!state.apply_loaded(
            "cluster-a",
            0,
            vec![sample_event("ev-stale-gen", "Normal", "Scheduled", "default")],
        ));
        assert!(!state.apply_failed("cluster-a", 0, "stale error".to_string()));
        assert!(state.events.is_empty());

        // Matching load succeeds.
        assert!(state.apply_loaded(
            "cluster-a",
            1,
            vec![sample_event("ev-1", "Warning", "BackOff", "default")],
        ));
        assert_eq!(state.status, EventsLoadStatus::Loaded);
        assert!(!state.table.loading);
        assert!(state.error_message.is_none());
        assert_eq!(state.events.len(), 1);
        assert_eq!(state.events[0].uid, "ev-1");

        // Refresh keeps existing rows while loading and handles failure.
        let refresh_cmd = state.begin_refresh().expect("refresh command");
        assert_eq!(
            refresh_cmd,
            ClusterBridgeCommand::LoadEvents {
                cluster_id: "cluster-a".to_string(),
                generation: 2,
            }
        );
        assert_eq!(state.status, EventsLoadStatus::Loading);
        assert!(!state.table.loading);
        assert!(state.apply_failed("cluster-a", 2, "timeout".to_string()));
        assert_eq!(state.status, EventsLoadStatus::Error);
        assert_eq!(state.error_message.as_deref(), Some("timeout"));
    }

    #[test]
    fn test_events_state_cluster_switch_and_disconnect_reset() {
        let mut state = EventsState::default();
        let _ = state.begin_load_for_cluster("cluster-a");
        assert!(state.apply_loaded(
            "cluster-a",
            1,
            vec![sample_event("ev-1", "Warning", "BackOff", "default")],
        ));
        state.set_type_filter(EventTypeFilter::Warning);
        state.table.selected_row_key = Some("ev-1".to_string());
        assert_eq!(state.type_filter, EventTypeFilter::Warning);

        state.reset_for_cluster("cluster-b");
        assert_eq!(state.cluster_id.as_deref(), Some("cluster-b"));
        assert_eq!(state.status, EventsLoadStatus::Idle);
        assert!(state.events.is_empty());
        assert_eq!(state.type_filter, EventTypeFilter::All);
        assert!(state.table.selected_row_key.is_none());
        assert!(state.needs_load_for_cluster("cluster-b"));

        let gen_before = state.generation;
        state.set_type_filter(EventTypeFilter::Normal);
        state.table.selected_row_key = Some("ev-2".to_string());
        state.reset_for_disconnected();
        assert!(state.cluster_id.is_none());
        assert_eq!(state.status, EventsLoadStatus::Idle);
        assert!(state.events.is_empty());
        assert_eq!(state.type_filter, EventTypeFilter::All);
        assert!(state.table.selected_row_key.is_none());
        assert!(state.generation > gen_before);
    }
}
