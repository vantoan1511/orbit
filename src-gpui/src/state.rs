use std::collections::{HashMap, HashSet};

use gpui::ScrollHandle;
use orbit_core::config::{OrbitConfig, SelectedNamespacesMap};
use orbit_core::kubernetes::manager::ClusterSwitchOutcome;
use orbit_core::kubernetes::models::{
    ClusterInfo, ConfigMapInfo, CronJobInfo, DaemonSetInfo, DeploymentInfo, EventInfo, IngressInfo,
    JobInfo, NamespaceInfo, NodeInfo, PodInfo, SecretInfo, ServiceInfo, StatefulSetInfo,
    UserProfileInfo,
};
use orbit_core::kubernetes::overview::ClusterOverviewSummary;
use orbit_core::kubernetes::workloads_overview::WorkloadsOverviewSnapshot;

use crate::components::data_table::DataTableState;
use crate::navigation::{SidebarCategory, WorkspaceView};

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
    SwitchCluster {
        cluster_id: String,
    },
    ReconnectActiveCluster,
    RefreshClusters,
    LoadOverview {
        cluster_id: String,
        generation: u64,
    },
    LoadWorkloadsOverview {
        cluster_id: String,
        generation: u64,
    },
    LoadNamespaces {
        cluster_id: String,
        generation: u64,
    },
    LoadDeployments {
        cluster_id: String,
        generation: u64,
    },
    LoadStatefulSets {
        cluster_id: String,
        generation: u64,
    },
    LoadDaemonSets {
        cluster_id: String,
        generation: u64,
    },
    LoadJobs {
        cluster_id: String,
        generation: u64,
    },
    LoadCronJobs {
        cluster_id: String,
        generation: u64,
    },
    LoadPods {
        cluster_id: String,
        generation: u64,
    },
    LoadServices {
        cluster_id: String,
        generation: u64,
    },
    LoadIngresses {
        cluster_id: String,
        generation: u64,
    },
    LoadConfigMaps {
        cluster_id: String,
        generation: u64,
    },
    LoadSecrets {
        cluster_id: String,
        generation: u64,
    },
    LoadNodes {
        cluster_id: String,
        generation: u64,
    },
    LoadEvents {
        cluster_id: String,
        generation: u64,
    },
    LoadResourceYaml {
        cluster_id: String,
        generation: u64,
        kind: String,
        namespace: Option<String>,
        name: String,
    },
    SaveNamespaceSelection {
        cluster_id: String,
        namespaces: Vec<String>,
    },
    SaveWorkspaceTabs {
        cluster_id: String,
        open_tabs: Vec<String>,
        active_tab: Option<String>,
    },
    SaveThemeMode {
        mode: String,
    },
    SaveUiStyle {
        style: String,
    },
    SaveAccentColor {
        accent: String,
    },
    SaveEnableExperienceFeatures {
        enabled: bool,
    },
    ResetAppearanceDefaults,
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
    WorkloadsOverviewLoaded {
        cluster_id: String,
        generation: u64,
        snapshot: WorkloadsOverviewSnapshot,
    },
    WorkloadsOverviewLoadFailed {
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
    StatefulSetsLoaded {
        cluster_id: String,
        generation: u64,
        statefulsets: Vec<StatefulSetInfo>,
    },
    StatefulSetsLoadFailed {
        cluster_id: String,
        generation: u64,
        message: String,
    },
    DaemonSetsLoaded {
        cluster_id: String,
        generation: u64,
        daemonsets: Vec<DaemonSetInfo>,
    },
    DaemonSetsLoadFailed {
        cluster_id: String,
        generation: u64,
        message: String,
    },
    JobsLoaded {
        cluster_id: String,
        generation: u64,
        jobs: Vec<JobInfo>,
    },
    JobsLoadFailed {
        cluster_id: String,
        generation: u64,
        message: String,
    },
    CronJobsLoaded {
        cluster_id: String,
        generation: u64,
        cronjobs: Vec<CronJobInfo>,
    },
    CronJobsLoadFailed {
        cluster_id: String,
        generation: u64,
        message: String,
    },
    PodsLoaded {
        cluster_id: String,
        generation: u64,
        pods: Vec<PodInfo>,
    },
    PodsLoadFailed {
        cluster_id: String,
        generation: u64,
        message: String,
    },
    ServicesLoaded {
        cluster_id: String,
        generation: u64,
        services: Vec<ServiceInfo>,
    },
    ServicesLoadFailed {
        cluster_id: String,
        generation: u64,
        message: String,
    },
    IngressesLoaded {
        cluster_id: String,
        generation: u64,
        ingresses: Vec<IngressInfo>,
    },
    IngressesLoadFailed {
        cluster_id: String,
        generation: u64,
        message: String,
    },
    ConfigMapsLoaded {
        cluster_id: String,
        generation: u64,
        configmaps: Vec<ConfigMapInfo>,
    },
    ConfigMapsLoadFailed {
        cluster_id: String,
        generation: u64,
        message: String,
    },
    SecretsLoaded {
        cluster_id: String,
        generation: u64,
        secrets: Vec<SecretInfo>,
    },
    SecretsLoadFailed {
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
    ResourceYamlLoaded {
        cluster_id: String,
        generation: u64,
        kind: String,
        namespace: Option<String>,
        name: String,
        yaml: String,
    },
    ResourceYamlLoadFailed {
        cluster_id: String,
        generation: u64,
        kind: String,
        namespace: Option<String>,
        name: String,
        message: String,
    },
    SettingsSaveFailed {
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

        let entry = self
            .selected_by_cluster
            .entry(cluster_id.clone())
            .or_default();
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
pub enum WorkloadsOverviewLoadStatus {
    #[default]
    Idle,
    Loading,
    Loaded,
    Error,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct WorkloadsOverviewState {
    pub cluster_id: Option<String>,
    pub generation: u64,
    pub status: WorkloadsOverviewLoadStatus,
    pub snapshot: Option<WorkloadsOverviewSnapshot>,
    pub error_message: Option<String>,
}

impl WorkloadsOverviewState {
    pub fn needs_load_for_cluster(&self, cluster_id: &str) -> bool {
        self.cluster_id.as_deref() != Some(cluster_id)
            || self.status == WorkloadsOverviewLoadStatus::Idle
    }

    pub fn begin_load_for_cluster(&mut self, cluster_id: &str) -> ClusterBridgeCommand {
        let switching_cluster = self.cluster_id.as_deref() != Some(cluster_id);
        self.generation += 1;
        self.cluster_id = Some(cluster_id.to_string());
        self.status = WorkloadsOverviewLoadStatus::Loading;
        self.error_message = None;
        if switching_cluster {
            self.snapshot = None;
        }
        ClusterBridgeCommand::LoadWorkloadsOverview {
            cluster_id: cluster_id.to_string(),
            generation: self.generation,
        }
    }

    pub fn begin_refresh(&mut self) -> Option<ClusterBridgeCommand> {
        let cluster_id = self.cluster_id.clone()?;
        self.generation += 1;
        self.status = WorkloadsOverviewLoadStatus::Loading;
        self.error_message = None;
        Some(ClusterBridgeCommand::LoadWorkloadsOverview {
            cluster_id,
            generation: self.generation,
        })
    }

    pub fn reset_for_cluster(&mut self, cluster_id: &str) {
        self.generation += 1;
        self.cluster_id = Some(cluster_id.to_string());
        self.status = WorkloadsOverviewLoadStatus::Idle;
        self.snapshot = None;
        self.error_message = None;
    }

    pub fn reset_for_disconnected(&mut self) {
        self.generation += 1;
        self.cluster_id = None;
        self.status = WorkloadsOverviewLoadStatus::Idle;
        self.snapshot = None;
        self.error_message = None;
    }

    pub fn apply_loaded(
        &mut self,
        cluster_id: &str,
        generation: u64,
        snapshot: WorkloadsOverviewSnapshot,
    ) -> bool {
        if self.cluster_id.as_deref() == Some(cluster_id) && self.generation == generation {
            self.status = WorkloadsOverviewLoadStatus::Loaded;
            self.snapshot = Some(snapshot);
            self.error_message = None;
            true
        } else {
            false
        }
    }

    pub fn apply_failed(&mut self, cluster_id: &str, generation: u64, message: String) -> bool {
        if self.cluster_id.as_deref() == Some(cluster_id) && self.generation == generation {
            self.status = WorkloadsOverviewLoadStatus::Error;
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
        self.cluster_id.as_deref() != Some(cluster_id) || self.status == DeploymentsLoadStatus::Idle
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum StatefulSetsLoadStatus {
    #[default]
    Idle,
    Loading,
    Loaded,
    Error,
}

#[derive(Debug, Clone, Default)]
pub struct StatefulSetsState {
    pub cluster_id: Option<String>,
    pub generation: u64,
    pub status: StatefulSetsLoadStatus,
    pub statefulsets: Vec<StatefulSetInfo>,
    pub error_message: Option<String>,
    pub table: DataTableState,
}

impl StatefulSetsState {
    pub fn begin_load_for_cluster(&mut self, cluster_id: &str) -> ClusterBridgeCommand {
        let switching_cluster = self.cluster_id.as_deref() != Some(cluster_id);
        self.generation += 1;
        self.cluster_id = Some(cluster_id.to_string());
        self.status = StatefulSetsLoadStatus::Loading;
        self.error_message = None;
        if switching_cluster {
            self.statefulsets.clear();
            self.table.selected_row_key = None;
        }
        self.table.loading = self.statefulsets.is_empty();
        ClusterBridgeCommand::LoadStatefulSets {
            cluster_id: cluster_id.to_string(),
            generation: self.generation,
        }
    }

    pub fn begin_refresh(&mut self) -> Option<ClusterBridgeCommand> {
        let cluster_id = self.cluster_id.clone()?;
        self.generation += 1;
        self.status = StatefulSetsLoadStatus::Loading;
        self.error_message = None;
        self.table.loading = self.statefulsets.is_empty();
        Some(ClusterBridgeCommand::LoadStatefulSets {
            cluster_id,
            generation: self.generation,
        })
    }

    pub fn reset_for_cluster(&mut self, cluster_id: &str) {
        self.generation += 1;
        self.cluster_id = Some(cluster_id.to_string());
        self.status = StatefulSetsLoadStatus::Idle;
        self.statefulsets.clear();
        self.error_message = None;
        self.table.selected_row_key = None;
        self.table.loading = false;
    }

    pub fn reset_for_disconnected(&mut self) {
        self.generation += 1;
        self.cluster_id = None;
        self.status = StatefulSetsLoadStatus::Idle;
        self.statefulsets.clear();
        self.error_message = None;
        self.table.selected_row_key = None;
        self.table.loading = false;
    }

    pub fn apply_loaded(
        &mut self,
        cluster_id: &str,
        generation: u64,
        statefulsets: Vec<StatefulSetInfo>,
    ) -> bool {
        if self.cluster_id.as_deref() == Some(cluster_id) && self.generation == generation {
            self.status = StatefulSetsLoadStatus::Loaded;
            self.statefulsets = statefulsets;
            self.error_message = None;
            self.table.loading = false;
            true
        } else {
            false
        }
    }

    pub fn apply_failed(&mut self, cluster_id: &str, generation: u64, message: String) -> bool {
        if self.cluster_id.as_deref() == Some(cluster_id) && self.generation == generation {
            self.status = StatefulSetsLoadStatus::Error;
            self.error_message = Some(message);
            self.table.loading = false;
            true
        } else {
            false
        }
    }

    pub fn needs_load_for_cluster(&self, cluster_id: &str) -> bool {
        self.cluster_id.as_deref() != Some(cluster_id)
            || self.status == StatefulSetsLoadStatus::Idle
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum DaemonSetsLoadStatus {
    #[default]
    Idle,
    Loading,
    Loaded,
    Error,
}

#[derive(Debug, Clone, Default)]
pub struct DaemonSetsState {
    pub cluster_id: Option<String>,
    pub generation: u64,
    pub status: DaemonSetsLoadStatus,
    pub daemonsets: Vec<DaemonSetInfo>,
    pub error_message: Option<String>,
    pub table: DataTableState,
}

impl DaemonSetsState {
    pub fn begin_load_for_cluster(&mut self, cluster_id: &str) -> ClusterBridgeCommand {
        let switching_cluster = self.cluster_id.as_deref() != Some(cluster_id);
        self.generation += 1;
        self.cluster_id = Some(cluster_id.to_string());
        self.status = DaemonSetsLoadStatus::Loading;
        self.error_message = None;
        if switching_cluster {
            self.daemonsets.clear();
            self.table.selected_row_key = None;
        }
        self.table.loading = self.daemonsets.is_empty();
        ClusterBridgeCommand::LoadDaemonSets {
            cluster_id: cluster_id.to_string(),
            generation: self.generation,
        }
    }

    pub fn begin_refresh(&mut self) -> Option<ClusterBridgeCommand> {
        let cluster_id = self.cluster_id.clone()?;
        self.generation += 1;
        self.status = DaemonSetsLoadStatus::Loading;
        self.error_message = None;
        self.table.loading = self.daemonsets.is_empty();
        Some(ClusterBridgeCommand::LoadDaemonSets {
            cluster_id,
            generation: self.generation,
        })
    }

    pub fn reset_for_cluster(&mut self, cluster_id: &str) {
        self.generation += 1;
        self.cluster_id = Some(cluster_id.to_string());
        self.status = DaemonSetsLoadStatus::Idle;
        self.daemonsets.clear();
        self.error_message = None;
        self.table.selected_row_key = None;
        self.table.loading = false;
    }

    pub fn reset_for_disconnected(&mut self) {
        self.generation += 1;
        self.cluster_id = None;
        self.status = DaemonSetsLoadStatus::Idle;
        self.daemonsets.clear();
        self.error_message = None;
        self.table.selected_row_key = None;
        self.table.loading = false;
    }

    pub fn apply_loaded(
        &mut self,
        cluster_id: &str,
        generation: u64,
        daemonsets: Vec<DaemonSetInfo>,
    ) -> bool {
        if self.cluster_id.as_deref() == Some(cluster_id) && self.generation == generation {
            self.status = DaemonSetsLoadStatus::Loaded;
            self.daemonsets = daemonsets;
            self.error_message = None;
            self.table.loading = false;
            true
        } else {
            false
        }
    }

    pub fn apply_failed(&mut self, cluster_id: &str, generation: u64, message: String) -> bool {
        if self.cluster_id.as_deref() == Some(cluster_id) && self.generation == generation {
            self.status = DaemonSetsLoadStatus::Error;
            self.error_message = Some(message);
            self.table.loading = false;
            true
        } else {
            false
        }
    }

    pub fn needs_load_for_cluster(&self, cluster_id: &str) -> bool {
        self.cluster_id.as_deref() != Some(cluster_id) || self.status == DaemonSetsLoadStatus::Idle
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum JobsLoadStatus {
    #[default]
    Idle,
    Loading,
    Loaded,
    Error,
}

#[derive(Debug, Clone, Default)]
pub struct JobsState {
    pub cluster_id: Option<String>,
    pub generation: u64,
    pub status: JobsLoadStatus,
    pub jobs: Vec<JobInfo>,
    pub error_message: Option<String>,
    pub table: DataTableState,
}

impl JobsState {
    pub fn begin_load_for_cluster(&mut self, cluster_id: &str) -> ClusterBridgeCommand {
        let switching_cluster = self.cluster_id.as_deref() != Some(cluster_id);
        self.generation += 1;
        self.cluster_id = Some(cluster_id.to_string());
        self.status = JobsLoadStatus::Loading;
        self.error_message = None;
        if switching_cluster {
            self.jobs.clear();
            self.table.selected_row_key = None;
        }
        self.table.loading = self.jobs.is_empty();
        ClusterBridgeCommand::LoadJobs {
            cluster_id: cluster_id.to_string(),
            generation: self.generation,
        }
    }

    pub fn begin_refresh(&mut self) -> Option<ClusterBridgeCommand> {
        let cluster_id = self.cluster_id.clone()?;
        self.generation += 1;
        self.status = JobsLoadStatus::Loading;
        self.error_message = None;
        self.table.loading = self.jobs.is_empty();
        Some(ClusterBridgeCommand::LoadJobs {
            cluster_id,
            generation: self.generation,
        })
    }

    pub fn reset_for_cluster(&mut self, cluster_id: &str) {
        self.generation += 1;
        self.cluster_id = Some(cluster_id.to_string());
        self.status = JobsLoadStatus::Idle;
        self.jobs.clear();
        self.error_message = None;
        self.table.selected_row_key = None;
        self.table.loading = false;
    }

    pub fn reset_for_disconnected(&mut self) {
        self.generation += 1;
        self.cluster_id = None;
        self.status = JobsLoadStatus::Idle;
        self.jobs.clear();
        self.error_message = None;
        self.table.selected_row_key = None;
        self.table.loading = false;
    }

    pub fn apply_loaded(&mut self, cluster_id: &str, generation: u64, jobs: Vec<JobInfo>) -> bool {
        if self.cluster_id.as_deref() == Some(cluster_id) && self.generation == generation {
            self.status = JobsLoadStatus::Loaded;
            self.jobs = jobs;
            self.error_message = None;
            self.table.loading = false;
            true
        } else {
            false
        }
    }

    pub fn apply_failed(&mut self, cluster_id: &str, generation: u64, message: String) -> bool {
        if self.cluster_id.as_deref() == Some(cluster_id) && self.generation == generation {
            self.status = JobsLoadStatus::Error;
            self.error_message = Some(message);
            self.table.loading = false;
            true
        } else {
            false
        }
    }

    pub fn needs_load_for_cluster(&self, cluster_id: &str) -> bool {
        self.cluster_id.as_deref() != Some(cluster_id) || self.status == JobsLoadStatus::Idle
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CronJobsLoadStatus {
    #[default]
    Idle,
    Loading,
    Loaded,
    Error,
}

#[derive(Debug, Clone, Default)]
pub struct CronJobsState {
    pub cluster_id: Option<String>,
    pub generation: u64,
    pub status: CronJobsLoadStatus,
    pub cronjobs: Vec<CronJobInfo>,
    pub error_message: Option<String>,
    pub table: DataTableState,
}

impl CronJobsState {
    pub fn begin_load_for_cluster(&mut self, cluster_id: &str) -> ClusterBridgeCommand {
        let switching_cluster = self.cluster_id.as_deref() != Some(cluster_id);
        self.generation += 1;
        self.cluster_id = Some(cluster_id.to_string());
        self.status = CronJobsLoadStatus::Loading;
        self.error_message = None;
        if switching_cluster {
            self.cronjobs.clear();
            self.table.selected_row_key = None;
        }
        self.table.loading = self.cronjobs.is_empty();
        ClusterBridgeCommand::LoadCronJobs {
            cluster_id: cluster_id.to_string(),
            generation: self.generation,
        }
    }

    pub fn begin_refresh(&mut self) -> Option<ClusterBridgeCommand> {
        let cluster_id = self.cluster_id.clone()?;
        self.generation += 1;
        self.status = CronJobsLoadStatus::Loading;
        self.error_message = None;
        self.table.loading = self.cronjobs.is_empty();
        Some(ClusterBridgeCommand::LoadCronJobs {
            cluster_id,
            generation: self.generation,
        })
    }

    pub fn reset_for_cluster(&mut self, cluster_id: &str) {
        self.generation += 1;
        self.cluster_id = Some(cluster_id.to_string());
        self.status = CronJobsLoadStatus::Idle;
        self.cronjobs.clear();
        self.error_message = None;
        self.table.selected_row_key = None;
        self.table.loading = false;
    }

    pub fn reset_for_disconnected(&mut self) {
        self.generation += 1;
        self.cluster_id = None;
        self.status = CronJobsLoadStatus::Idle;
        self.cronjobs.clear();
        self.error_message = None;
        self.table.selected_row_key = None;
        self.table.loading = false;
    }

    pub fn apply_loaded(
        &mut self,
        cluster_id: &str,
        generation: u64,
        cronjobs: Vec<CronJobInfo>,
    ) -> bool {
        if self.cluster_id.as_deref() == Some(cluster_id) && self.generation == generation {
            self.status = CronJobsLoadStatus::Loaded;
            self.cronjobs = cronjobs;
            self.error_message = None;
            self.table.loading = false;
            true
        } else {
            false
        }
    }

    pub fn apply_failed(&mut self, cluster_id: &str, generation: u64, message: String) -> bool {
        if self.cluster_id.as_deref() == Some(cluster_id) && self.generation == generation {
            self.status = CronJobsLoadStatus::Error;
            self.error_message = Some(message);
            self.table.loading = false;
            true
        } else {
            false
        }
    }

    pub fn needs_load_for_cluster(&self, cluster_id: &str) -> bool {
        self.cluster_id.as_deref() != Some(cluster_id) || self.status == CronJobsLoadStatus::Idle
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PodsLoadStatus {
    #[default]
    Idle,
    Loading,
    Loaded,
    Error,
}

#[derive(Debug, Clone, Default)]
pub struct PodsState {
    pub cluster_id: Option<String>,
    pub generation: u64,
    pub status: PodsLoadStatus,
    pub pods: Vec<PodInfo>,
    pub error_message: Option<String>,
    pub table: DataTableState,
}

impl PodsState {
    pub fn begin_load_for_cluster(&mut self, cluster_id: &str) -> ClusterBridgeCommand {
        let switching_cluster = self.cluster_id.as_deref() != Some(cluster_id);
        self.generation += 1;
        self.cluster_id = Some(cluster_id.to_string());
        self.status = PodsLoadStatus::Loading;
        self.error_message = None;
        if switching_cluster {
            self.pods.clear();
            self.table.selected_row_key = None;
        }
        self.table.loading = self.pods.is_empty();
        ClusterBridgeCommand::LoadPods {
            cluster_id: cluster_id.to_string(),
            generation: self.generation,
        }
    }

    pub fn begin_refresh(&mut self) -> Option<ClusterBridgeCommand> {
        let cluster_id = self.cluster_id.clone()?;
        self.generation += 1;
        self.status = PodsLoadStatus::Loading;
        self.error_message = None;
        self.table.loading = self.pods.is_empty();
        Some(ClusterBridgeCommand::LoadPods {
            cluster_id,
            generation: self.generation,
        })
    }

    pub fn reset_for_cluster(&mut self, cluster_id: &str) {
        self.generation += 1;
        self.cluster_id = Some(cluster_id.to_string());
        self.status = PodsLoadStatus::Idle;
        self.pods.clear();
        self.error_message = None;
        self.table.selected_row_key = None;
        self.table.loading = false;
    }

    pub fn reset_for_disconnected(&mut self) {
        self.generation += 1;
        self.cluster_id = None;
        self.status = PodsLoadStatus::Idle;
        self.pods.clear();
        self.error_message = None;
        self.table.selected_row_key = None;
        self.table.loading = false;
    }

    pub fn apply_loaded(&mut self, cluster_id: &str, generation: u64, pods: Vec<PodInfo>) -> bool {
        if self.cluster_id.as_deref() == Some(cluster_id) && self.generation == generation {
            self.status = PodsLoadStatus::Loaded;
            self.pods = pods;
            self.error_message = None;
            self.table.loading = false;
            true
        } else {
            false
        }
    }

    pub fn apply_failed(&mut self, cluster_id: &str, generation: u64, message: String) -> bool {
        if self.cluster_id.as_deref() == Some(cluster_id) && self.generation == generation {
            self.status = PodsLoadStatus::Error;
            self.error_message = Some(message);
            self.table.loading = false;
            true
        } else {
            false
        }
    }

    pub fn needs_load_for_cluster(&self, cluster_id: &str) -> bool {
        self.cluster_id.as_deref() != Some(cluster_id) || self.status == PodsLoadStatus::Idle
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ServicesLoadStatus {
    #[default]
    Idle,
    Loading,
    Loaded,
    Error,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ServiceTypeFilter {
    #[default]
    All,
    ClusterIP,
    NodePort,
    LoadBalancer,
    ExternalName,
}

impl ServiceTypeFilter {
    pub fn all() -> &'static [Self] {
        &[
            Self::All,
            Self::ClusterIP,
            Self::NodePort,
            Self::LoadBalancer,
            Self::ExternalName,
        ]
    }

    pub fn label(&self) -> &'static str {
        match self {
            Self::All => "All",
            Self::ClusterIP => "ClusterIP",
            Self::NodePort => "NodePort",
            Self::LoadBalancer => "LoadBalancer",
            Self::ExternalName => "ExternalName",
        }
    }

    pub fn matches(&self, service_type: &str) -> bool {
        match self {
            Self::All => true,
            Self::ClusterIP => service_type == "ClusterIP",
            Self::NodePort => service_type == "NodePort",
            Self::LoadBalancer => service_type == "LoadBalancer",
            Self::ExternalName => service_type == "ExternalName",
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct ServicesState {
    pub cluster_id: Option<String>,
    pub generation: u64,
    pub status: ServicesLoadStatus,
    pub services: Vec<ServiceInfo>,
    pub type_filter: ServiceTypeFilter,
    pub error_message: Option<String>,
    pub table: DataTableState,
}

impl ServicesState {
    pub fn begin_load_for_cluster(&mut self, cluster_id: &str) -> ClusterBridgeCommand {
        let switching_cluster = self.cluster_id.as_deref() != Some(cluster_id);
        self.generation += 1;
        self.cluster_id = Some(cluster_id.to_string());
        self.status = ServicesLoadStatus::Loading;
        self.error_message = None;
        if switching_cluster {
            self.services.clear();
            self.type_filter = ServiceTypeFilter::All;
            self.table.selected_row_key = None;
        }
        self.table.loading = self.services.is_empty();
        ClusterBridgeCommand::LoadServices {
            cluster_id: cluster_id.to_string(),
            generation: self.generation,
        }
    }

    pub fn begin_refresh(&mut self) -> Option<ClusterBridgeCommand> {
        let cluster_id = self.cluster_id.clone()?;
        self.generation += 1;
        self.status = ServicesLoadStatus::Loading;
        self.error_message = None;
        self.table.loading = self.services.is_empty();
        Some(ClusterBridgeCommand::LoadServices {
            cluster_id,
            generation: self.generation,
        })
    }

    pub fn reset_for_cluster(&mut self, cluster_id: &str) {
        self.generation += 1;
        self.cluster_id = Some(cluster_id.to_string());
        self.status = ServicesLoadStatus::Idle;
        self.services.clear();
        self.type_filter = ServiceTypeFilter::All;
        self.error_message = None;
        self.table.selected_row_key = None;
        self.table.loading = false;
    }

    pub fn reset_for_disconnected(&mut self) {
        self.generation += 1;
        self.cluster_id = None;
        self.status = ServicesLoadStatus::Idle;
        self.services.clear();
        self.type_filter = ServiceTypeFilter::All;
        self.error_message = None;
        self.table.selected_row_key = None;
        self.table.loading = false;
    }

    pub fn apply_loaded(
        &mut self,
        cluster_id: &str,
        generation: u64,
        services: Vec<ServiceInfo>,
    ) -> bool {
        if self.cluster_id.as_deref() == Some(cluster_id) && self.generation == generation {
            self.status = ServicesLoadStatus::Loaded;
            self.services = services;
            self.error_message = None;
            self.table.loading = false;
            true
        } else {
            false
        }
    }

    pub fn apply_failed(&mut self, cluster_id: &str, generation: u64, message: String) -> bool {
        if self.cluster_id.as_deref() == Some(cluster_id) && self.generation == generation {
            self.status = ServicesLoadStatus::Error;
            self.error_message = Some(message);
            self.table.loading = false;
            true
        } else {
            false
        }
    }

    pub fn needs_load_for_cluster(&self, cluster_id: &str) -> bool {
        self.cluster_id.as_deref() != Some(cluster_id) || self.status == ServicesLoadStatus::Idle
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum IngressesLoadStatus {
    #[default]
    Idle,
    Loading,
    Loaded,
    Error,
}

#[derive(Debug, Clone, Default)]
pub struct IngressesState {
    pub cluster_id: Option<String>,
    pub generation: u64,
    pub status: IngressesLoadStatus,
    pub ingresses: Vec<IngressInfo>,
    pub error_message: Option<String>,
    pub table: DataTableState,
}

impl IngressesState {
    pub fn begin_load_for_cluster(&mut self, cluster_id: &str) -> ClusterBridgeCommand {
        let switching_cluster = self.cluster_id.as_deref() != Some(cluster_id);
        self.generation += 1;
        self.cluster_id = Some(cluster_id.to_string());
        self.status = IngressesLoadStatus::Loading;
        self.error_message = None;
        if switching_cluster {
            self.ingresses.clear();
            self.table.selected_row_key = None;
        }
        self.table.loading = self.ingresses.is_empty();
        ClusterBridgeCommand::LoadIngresses {
            cluster_id: cluster_id.to_string(),
            generation: self.generation,
        }
    }

    pub fn begin_refresh(&mut self) -> Option<ClusterBridgeCommand> {
        let cluster_id = self.cluster_id.clone()?;
        self.generation += 1;
        self.status = IngressesLoadStatus::Loading;
        self.error_message = None;
        self.table.loading = self.ingresses.is_empty();
        Some(ClusterBridgeCommand::LoadIngresses {
            cluster_id,
            generation: self.generation,
        })
    }

    pub fn reset_for_cluster(&mut self, cluster_id: &str) {
        self.generation += 1;
        self.cluster_id = Some(cluster_id.to_string());
        self.status = IngressesLoadStatus::Idle;
        self.ingresses.clear();
        self.error_message = None;
        self.table.selected_row_key = None;
        self.table.loading = false;
    }

    pub fn reset_for_disconnected(&mut self) {
        self.generation += 1;
        self.cluster_id = None;
        self.status = IngressesLoadStatus::Idle;
        self.ingresses.clear();
        self.error_message = None;
        self.table.selected_row_key = None;
        self.table.loading = false;
    }

    pub fn apply_loaded(
        &mut self,
        cluster_id: &str,
        generation: u64,
        ingresses: Vec<IngressInfo>,
    ) -> bool {
        if self.cluster_id.as_deref() == Some(cluster_id) && self.generation == generation {
            self.status = IngressesLoadStatus::Loaded;
            self.ingresses = ingresses;
            self.error_message = None;
            self.table.loading = false;
            true
        } else {
            false
        }
    }

    pub fn apply_failed(&mut self, cluster_id: &str, generation: u64, message: String) -> bool {
        if self.cluster_id.as_deref() == Some(cluster_id) && self.generation == generation {
            self.status = IngressesLoadStatus::Error;
            self.error_message = Some(message);
            self.table.loading = false;
            true
        } else {
            false
        }
    }

    pub fn needs_load_for_cluster(&self, cluster_id: &str) -> bool {
        self.cluster_id.as_deref() != Some(cluster_id) || self.status == IngressesLoadStatus::Idle
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ConfigMapsLoadStatus {
    #[default]
    Idle,
    Loading,
    Loaded,
    Error,
}

#[derive(Debug, Clone, Default)]
pub struct ConfigMapsState {
    pub cluster_id: Option<String>,
    pub generation: u64,
    pub status: ConfigMapsLoadStatus,
    pub configmaps: Vec<ConfigMapInfo>,
    pub error_message: Option<String>,
    pub table: DataTableState,
}

impl ConfigMapsState {
    pub fn begin_load_for_cluster(&mut self, cluster_id: &str) -> ClusterBridgeCommand {
        let switching_cluster = self.cluster_id.as_deref() != Some(cluster_id);
        self.generation += 1;
        self.cluster_id = Some(cluster_id.to_string());
        self.status = ConfigMapsLoadStatus::Loading;
        self.error_message = None;
        if switching_cluster {
            self.configmaps.clear();
            self.table.selected_row_key = None;
        }
        self.table.loading = self.configmaps.is_empty();
        ClusterBridgeCommand::LoadConfigMaps {
            cluster_id: cluster_id.to_string(),
            generation: self.generation,
        }
    }

    pub fn begin_refresh(&mut self) -> Option<ClusterBridgeCommand> {
        let cluster_id = self.cluster_id.clone()?;
        self.generation += 1;
        self.status = ConfigMapsLoadStatus::Loading;
        self.error_message = None;
        self.table.loading = self.configmaps.is_empty();
        Some(ClusterBridgeCommand::LoadConfigMaps {
            cluster_id,
            generation: self.generation,
        })
    }

    pub fn reset_for_cluster(&mut self, cluster_id: &str) {
        self.generation += 1;
        self.cluster_id = Some(cluster_id.to_string());
        self.status = ConfigMapsLoadStatus::Idle;
        self.configmaps.clear();
        self.error_message = None;
        self.table.selected_row_key = None;
        self.table.loading = false;
    }

    pub fn reset_for_disconnected(&mut self) {
        self.generation += 1;
        self.cluster_id = None;
        self.status = ConfigMapsLoadStatus::Idle;
        self.configmaps.clear();
        self.error_message = None;
        self.table.selected_row_key = None;
        self.table.loading = false;
    }

    pub fn apply_loaded(
        &mut self,
        cluster_id: &str,
        generation: u64,
        configmaps: Vec<ConfigMapInfo>,
    ) -> bool {
        if self.cluster_id.as_deref() == Some(cluster_id) && self.generation == generation {
            self.status = ConfigMapsLoadStatus::Loaded;
            self.configmaps = configmaps;
            self.error_message = None;
            self.table.loading = false;
            true
        } else {
            false
        }
    }

    pub fn apply_failed(&mut self, cluster_id: &str, generation: u64, message: String) -> bool {
        if self.cluster_id.as_deref() == Some(cluster_id) && self.generation == generation {
            self.status = ConfigMapsLoadStatus::Error;
            self.error_message = Some(message);
            self.table.loading = false;
            true
        } else {
            false
        }
    }

    pub fn needs_load_for_cluster(&self, cluster_id: &str) -> bool {
        self.cluster_id.as_deref() != Some(cluster_id) || self.status == ConfigMapsLoadStatus::Idle
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SecretsLoadStatus {
    #[default]
    Idle,
    Loading,
    Loaded,
    Error,
}

#[derive(Debug, Clone, Default)]
pub struct SecretsState {
    pub cluster_id: Option<String>,
    pub generation: u64,
    pub status: SecretsLoadStatus,
    pub secrets: Vec<SecretInfo>,
    pub error_message: Option<String>,
    pub table: DataTableState,
}

impl SecretsState {
    pub fn begin_load_for_cluster(&mut self, cluster_id: &str) -> ClusterBridgeCommand {
        let switching_cluster = self.cluster_id.as_deref() != Some(cluster_id);
        self.generation += 1;
        self.cluster_id = Some(cluster_id.to_string());
        self.status = SecretsLoadStatus::Loading;
        self.error_message = None;
        if switching_cluster {
            self.secrets.clear();
            self.table.selected_row_key = None;
        }
        self.table.loading = self.secrets.is_empty();
        ClusterBridgeCommand::LoadSecrets {
            cluster_id: cluster_id.to_string(),
            generation: self.generation,
        }
    }

    pub fn begin_refresh(&mut self) -> Option<ClusterBridgeCommand> {
        let cluster_id = self.cluster_id.clone()?;
        self.generation += 1;
        self.status = SecretsLoadStatus::Loading;
        self.error_message = None;
        self.table.loading = self.secrets.is_empty();
        Some(ClusterBridgeCommand::LoadSecrets {
            cluster_id,
            generation: self.generation,
        })
    }

    pub fn reset_for_cluster(&mut self, cluster_id: &str) {
        self.generation += 1;
        self.cluster_id = Some(cluster_id.to_string());
        self.status = SecretsLoadStatus::Idle;
        self.secrets.clear();
        self.error_message = None;
        self.table.selected_row_key = None;
        self.table.loading = false;
    }

    pub fn reset_for_disconnected(&mut self) {
        self.generation += 1;
        self.cluster_id = None;
        self.status = SecretsLoadStatus::Idle;
        self.secrets.clear();
        self.error_message = None;
        self.table.selected_row_key = None;
        self.table.loading = false;
    }

    pub fn apply_loaded(
        &mut self,
        cluster_id: &str,
        generation: u64,
        secrets: Vec<SecretInfo>,
    ) -> bool {
        if self.cluster_id.as_deref() == Some(cluster_id) && self.generation == generation {
            self.status = SecretsLoadStatus::Loaded;
            self.secrets = secrets;
            self.error_message = None;
            self.table.loading = false;
            true
        } else {
            false
        }
    }

    pub fn apply_failed(&mut self, cluster_id: &str, generation: u64, message: String) -> bool {
        if self.cluster_id.as_deref() == Some(cluster_id) && self.generation == generation {
            self.status = SecretsLoadStatus::Error;
            self.error_message = Some(message);
            self.table.loading = false;
            true
        } else {
            false
        }
    }

    pub fn needs_load_for_cluster(&self, cluster_id: &str) -> bool {
        self.cluster_id.as_deref() != Some(cluster_id) || self.status == SecretsLoadStatus::Idle
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
        if matches!(
            self.connection_state,
            ClusterConnectionState::Connecting { .. }
        ) {
            return None;
        }

        let target_cluster_id =
            self.active_cluster_id
                .clone()
                .or_else(|| match &self.connection_state {
                    ClusterConnectionState::Unreachable { cluster_id, .. } => {
                        Some(cluster_id.clone())
                    }
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
            | ClusterBridgeEvent::WorkloadsOverviewLoaded { .. }
            | ClusterBridgeEvent::WorkloadsOverviewLoadFailed { .. }
            | ClusterBridgeEvent::DeploymentsLoaded { .. }
            | ClusterBridgeEvent::DeploymentsLoadFailed { .. }
            | ClusterBridgeEvent::StatefulSetsLoaded { .. }
            | ClusterBridgeEvent::StatefulSetsLoadFailed { .. }
            | ClusterBridgeEvent::DaemonSetsLoaded { .. }
            | ClusterBridgeEvent::DaemonSetsLoadFailed { .. }
            | ClusterBridgeEvent::JobsLoaded { .. }
            | ClusterBridgeEvent::JobsLoadFailed { .. }
            | ClusterBridgeEvent::CronJobsLoaded { .. }
            | ClusterBridgeEvent::CronJobsLoadFailed { .. }
            | ClusterBridgeEvent::PodsLoaded { .. }
            | ClusterBridgeEvent::PodsLoadFailed { .. }
            | ClusterBridgeEvent::ServicesLoaded { .. }
            | ClusterBridgeEvent::ServicesLoadFailed { .. }
            | ClusterBridgeEvent::IngressesLoaded { .. }
            | ClusterBridgeEvent::IngressesLoadFailed { .. }
            | ClusterBridgeEvent::ConfigMapsLoaded { .. }
            | ClusterBridgeEvent::ConfigMapsLoadFailed { .. }
            | ClusterBridgeEvent::SecretsLoaded { .. }
            | ClusterBridgeEvent::SecretsLoadFailed { .. }
            | ClusterBridgeEvent::NodesLoaded { .. }
            | ClusterBridgeEvent::NodesLoadFailed { .. }
            | ClusterBridgeEvent::EventsLoaded { .. }
            | ClusterBridgeEvent::EventsLoadFailed { .. }
            | ClusterBridgeEvent::ResourceYamlLoaded { .. }
            | ClusterBridgeEvent::ResourceYamlLoadFailed { .. }
            | ClusterBridgeEvent::SettingsSaveFailed { .. } => {}
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TabDropPosition {
    Before,
    After,
}

pub fn calculate_tab_drop_index(
    from_index: usize,
    hover_index: usize,
    position: TabDropPosition,
) -> usize {
    if from_index == hover_index {
        return from_index;
    }
    if from_index < hover_index {
        match position {
            TabDropPosition::Before => hover_index.saturating_sub(1),
            TabDropPosition::After => hover_index,
        }
    } else {
        match position {
            TabDropPosition::Before => hover_index,
            TabDropPosition::After => hover_index + 1,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ClusterTabsState {
    pub open_tabs: Vec<WorkspaceView>,
    pub active_tab: Option<WorkspaceView>,
}

#[derive(Debug, Clone)]
pub struct WorkspaceTabsState {
    by_cluster: HashMap<String, ClusterTabsState>,
    active_cluster_id: Option<String>,
    pub scroll_handle: ScrollHandle,
    pub pending_scroll_to_active: bool,
    pub picker_open: bool,
    pub picker_query: String,
    pub drop_target: Option<(usize, TabDropPosition)>,
    pub dragging_index: Option<usize>,
}

impl Default for WorkspaceTabsState {
    fn default() -> Self {
        Self::new()
    }
}

impl WorkspaceTabsState {
    pub fn new() -> Self {
        Self {
            by_cluster: HashMap::new(),
            active_cluster_id: None,
            scroll_handle: ScrollHandle::new(),
            pending_scroll_to_active: false,
            picker_open: false,
            picker_query: String::new(),
            drop_target: None,
            dragging_index: None,
        }
    }

    pub fn from_config(config: &OrbitConfig) -> Self {
        let mut state = Self::new();
        for (cluster_id, persisted) in config.workspace_tabs_map() {
            let mut seen = HashSet::new();
            let open_tabs: Vec<WorkspaceView> = persisted
                .open_tabs
                .iter()
                .filter_map(|id| WorkspaceView::from_id(id))
                .filter(|view| seen.insert(*view))
                .collect();

            let active_tab = if open_tabs.is_empty() {
                None
            } else {
                persisted
                    .active_tab
                    .as_deref()
                    .and_then(WorkspaceView::from_id)
                    .filter(|view| open_tabs.contains(view))
                    .or_else(|| open_tabs.first().copied())
            };

            state.by_cluster.insert(
                cluster_id,
                ClusterTabsState {
                    open_tabs,
                    active_tab,
                },
            );
        }
        state
    }

    pub fn active_cluster_id(&self) -> Option<&str> {
        self.active_cluster_id.as_deref()
    }

    pub fn open_tabs(&self) -> &[WorkspaceView] {
        self.active_cluster_id
            .as_deref()
            .and_then(|id| self.by_cluster.get(id))
            .map(|c| c.open_tabs.as_slice())
            .unwrap_or(&[])
    }

    pub fn active_tab(&self) -> Option<WorkspaceView> {
        self.active_cluster_id
            .as_deref()
            .and_then(|id| self.by_cluster.get(id))
            .and_then(|c| c.active_tab)
    }

    pub fn is_bar_visible(&self, has_active_cluster: bool) -> bool {
        has_active_cluster && !self.open_tabs().is_empty()
    }

    pub fn activate_cluster(&mut self, cluster_id: &str) -> Option<WorkspaceView> {
        self.active_cluster_id = Some(cluster_id.to_string());
        self.close_picker();
        self.drop_target = None;
        self.dragging_index = None;
        let entry = self
            .by_cluster
            .entry(cluster_id.to_string())
            .or_insert_with(|| ClusterTabsState {
                open_tabs: vec![WorkspaceView::Overview],
                active_tab: Some(WorkspaceView::Overview),
            });
        self.pending_scroll_to_active = entry.active_tab.is_some();
        entry.active_tab
    }

    pub fn clear_active_cluster(&mut self) {
        self.active_cluster_id = None;
        self.close_picker();
        self.drop_target = None;
        self.dragging_index = None;
    }

    fn current_cluster_mut(&mut self) -> Option<&mut ClusterTabsState> {
        let id = self.active_cluster_id.as_ref()?;
        Some(self.by_cluster.entry(id.clone()).or_default())
    }

    pub fn open_tab(&mut self, view: WorkspaceView) {
        let Some(cluster) = self.current_cluster_mut() else {
            return;
        };
        if !cluster.open_tabs.contains(&view) {
            cluster.open_tabs.push(view);
        }
        cluster.active_tab = Some(view);
        self.pending_scroll_to_active = true;
    }

    pub fn set_active_tab(&mut self, view: WorkspaceView) {
        self.open_tab(view);
    }

    pub fn activate_next_tab(&mut self) {
        let Some(cluster) = self.current_cluster_mut() else {
            return;
        };
        if cluster.open_tabs.is_empty() {
            return;
        }
        let next_idx = match cluster
            .active_tab
            .and_then(|active| cluster.open_tabs.iter().position(|v| *v == active))
        {
            Some(idx) => (idx + 1) % cluster.open_tabs.len(),
            None => 0,
        };
        cluster.active_tab = Some(cluster.open_tabs[next_idx]);
        self.pending_scroll_to_active = true;
    }

    pub fn activate_prev_tab(&mut self) {
        let Some(cluster) = self.current_cluster_mut() else {
            return;
        };
        if cluster.open_tabs.is_empty() {
            return;
        }
        let prev_idx = match cluster
            .active_tab
            .and_then(|active| cluster.open_tabs.iter().position(|v| *v == active))
        {
            Some(idx) => {
                if idx == 0 {
                    cluster.open_tabs.len() - 1
                } else {
                    idx - 1
                }
            }
            None => 0,
        };
        cluster.active_tab = Some(cluster.open_tabs[prev_idx]);
        self.pending_scroll_to_active = true;
    }

    pub fn close_tab(&mut self, view: WorkspaceView) {
        let Some(cluster) = self.current_cluster_mut() else {
            return;
        };
        let Some(idx) = cluster.open_tabs.iter().position(|v| *v == view) else {
            return;
        };
        cluster.open_tabs.remove(idx);
        if cluster.active_tab == Some(view) {
            if cluster.open_tabs.is_empty() {
                cluster.active_tab = None;
            } else if idx < cluster.open_tabs.len() {
                cluster.active_tab = Some(cluster.open_tabs[idx]);
            } else {
                cluster.active_tab = Some(cluster.open_tabs[idx - 1]);
            }
            self.pending_scroll_to_active = cluster.active_tab.is_some();
        }
    }

    pub fn close_active_tab(&mut self) {
        if let Some(active) = self.active_tab() {
            self.close_tab(active);
        }
    }

    pub fn close_other_tabs(&mut self, view: WorkspaceView) {
        let Some(cluster) = self.current_cluster_mut() else {
            return;
        };
        if cluster.open_tabs.contains(&view) {
            cluster.open_tabs = vec![view];
            cluster.active_tab = Some(view);
            self.pending_scroll_to_active = true;
        }
    }

    pub fn close_tabs_to_right(&mut self, view: WorkspaceView) {
        let Some(cluster) = self.current_cluster_mut() else {
            return;
        };
        let Some(idx) = cluster.open_tabs.iter().position(|v| *v == view) else {
            return;
        };
        cluster.open_tabs.truncate(idx + 1);
        if let Some(active) = cluster.active_tab {
            if !cluster.open_tabs.contains(&active) {
                cluster.active_tab = Some(view);
                self.pending_scroll_to_active = true;
            }
        }
    }

    pub fn close_all_tabs(&mut self) {
        let Some(cluster) = self.current_cluster_mut() else {
            return;
        };
        cluster.open_tabs.clear();
        cluster.active_tab = None;
    }

    pub fn reorder_tab(&mut self, from_index: usize, to_index: usize) {
        let Some(cluster) = self.current_cluster_mut() else {
            return;
        };
        let len = cluster.open_tabs.len();
        if from_index >= len || to_index >= len || from_index == to_index {
            return;
        }
        let moved = cluster.open_tabs.remove(from_index);
        cluster.open_tabs.insert(to_index, moved);
    }

    pub fn move_tab_left(&mut self, view: WorkspaceView) {
        let Some(cluster) = self.current_cluster_mut() else {
            return;
        };
        if let Some(idx) = cluster.open_tabs.iter().position(|v| *v == view) {
            if idx > 0 {
                cluster.open_tabs.swap(idx, idx - 1);
            }
        }
    }

    pub fn move_tab_right(&mut self, view: WorkspaceView) {
        let Some(cluster) = self.current_cluster_mut() else {
            return;
        };
        if let Some(idx) = cluster.open_tabs.iter().position(|v| *v == view) {
            if idx + 1 < cluster.open_tabs.len() {
                cluster.open_tabs.swap(idx, idx + 1);
            }
        }
    }

    pub fn open_picker(&mut self) {
        self.picker_open = true;
        self.picker_query.clear();
    }

    pub fn close_picker(&mut self) {
        self.picker_open = false;
        self.picker_query.clear();
    }

    pub fn toggle_picker(&mut self) {
        if self.picker_open {
            self.close_picker();
        } else {
            self.open_picker();
        }
    }

    pub fn handle_picker_search_key(&mut self, key: &str, key_char: Option<&str>) -> bool {
        if !self.picker_open {
            return false;
        }
        match key {
            "escape" => {
                self.close_picker();
                true
            }
            "backspace" => {
                self.picker_query.pop();
                true
            }
            _ => {
                if let Some(ch) = key_char {
                    if !ch.is_empty() && !ch.chars().all(char::is_control) {
                        self.picker_query.push_str(ch);
                        return true;
                    }
                }
                false
            }
        }
    }

    pub fn filtered_picker_views(&self) -> Vec<WorkspaceView> {
        let query = self.picker_query.trim().to_lowercase();
        let mut views = Vec::new();
        for category in SidebarCategory::ALL {
            if category == SidebarCategory::Clusters {
                continue;
            }
            views.extend_from_slice(category.nav_items());
        }
        views.push(WorkspaceView::Settings);

        if query.is_empty() {
            return views;
        }

        views
            .into_iter()
            .filter(|view| {
                view.label().to_lowercase().contains(&query)
                    || view.id().to_lowercase().contains(&query)
                    || view
                        .category()
                        .map(|cat| cat.label().to_lowercase().contains(&query))
                        .unwrap_or(false)
            })
            .collect()
    }

    pub fn persistence_command(&self) -> Option<ClusterBridgeCommand> {
        let cluster_id = self.active_cluster_id.as_ref()?;
        let cluster = self.by_cluster.get(cluster_id)?;
        Some(ClusterBridgeCommand::SaveWorkspaceTabs {
            cluster_id: cluster_id.clone(),
            open_tabs: cluster
                .open_tabs
                .iter()
                .map(|v| v.id().to_string())
                .collect(),
            active_tab: cluster.active_tab.map(|v| v.id().to_string()),
        })
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
        let clusters = vec![sample_cluster(
            "ctx-1",
            Some("Prod US"),
            Some("https://10.0.0.1:6443"),
        )];

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
            state
                .active_profile
                .as_ref()
                .and_then(|p| p.user_name.as_deref()),
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
        assert_eq!(state.active_cluster().map(|c| c.id.as_str()), Some("ctx-1"));
    }

    #[test]
    fn test_apply_event_unreachable_and_connection_error_surface_diagnostics() {
        let mut state = ClusterSwitcherState::new();
        let clusters = vec![sample_cluster("ctx-down", None, Some("http://127.0.0.1:1"))];
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
        assert!(!state.apply_loaded("ctx-a", 99, vec![sample_deployment("wrong-gen", "default")],));
        assert!(state.deployments.is_empty());

        // Matching (cluster_id, generation) succeeds.
        assert!(state.apply_loaded("ctx-a", 1, vec![sample_deployment("api", "default")],));
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

    fn sample_statefulset(name: &str, namespace: &str) -> StatefulSetInfo {
        StatefulSetInfo {
            kind: "StatefulSet".to_string(),
            name: name.to_string(),
            namespace: namespace.to_string(),
            status: "Running".to_string(),
            replicas: orbit_core::kubernetes::models::Replicas {
                current: 3,
                desired: 3,
            },
            age: "2d".to_string(),
            images: vec!["redis:7.2".to_string()],
            containers: vec![orbit_core::kubernetes::models::ContainerImageInfo {
                name: "redis".to_string(),
                image: "redis:7.2".to_string(),
            }],
            labels: Default::default(),
            annotations: Default::default(),
        }
    }

    #[test]
    fn test_statefulsets_state_lifecycle_and_generation_guard() {
        let mut state = StatefulSetsState::default();
        assert!(state.needs_load_for_cluster("ctx-a"));

        let cmd = state.begin_load_for_cluster("ctx-a");
        assert_eq!(
            cmd,
            ClusterBridgeCommand::LoadStatefulSets {
                cluster_id: "ctx-a".to_string(),
                generation: 1,
            }
        );
        assert_eq!(state.status, StatefulSetsLoadStatus::Loading);
        assert!(state.table.loading);
        assert!(!state.needs_load_for_cluster("ctx-a"));

        // Stale cluster or generation is rejected.
        assert!(!state.apply_loaded(
            "ctx-other",
            1,
            vec![sample_statefulset("wrong-cluster", "default")],
        ));
        assert!(!state.apply_loaded(
            "ctx-a",
            99,
            vec![sample_statefulset("wrong-gen", "default")],
        ));
        assert!(state.statefulsets.is_empty());

        // Matching cluster and generation is accepted.
        assert!(state.apply_loaded("ctx-a", 1, vec![sample_statefulset("redis", "default")],));
        assert_eq!(state.status, StatefulSetsLoadStatus::Loaded);
        assert!(!state.table.loading);
        assert_eq!(state.statefulsets.len(), 1);
        assert_eq!(state.statefulsets[0].name, "redis");

        // Refresh retains cached rows while loading.
        let refresh = state.begin_refresh();
        assert_eq!(
            refresh,
            Some(ClusterBridgeCommand::LoadStatefulSets {
                cluster_id: "ctx-a".to_string(),
                generation: 2,
            })
        );
        assert_eq!(state.status, StatefulSetsLoadStatus::Loading);
        assert_eq!(state.statefulsets.len(), 1);

        assert!(state.apply_failed("ctx-a", 2, "timeout".to_string()));
        assert_eq!(state.status, StatefulSetsLoadStatus::Error);
        assert_eq!(state.error_message.as_deref(), Some("timeout"));

        // Reset for new cluster clears rows, selection, and returns to Idle.
        state.reset_for_cluster("ctx-b");
        assert_eq!(state.cluster_id.as_deref(), Some("ctx-b"));
        assert_eq!(state.status, StatefulSetsLoadStatus::Idle);
        assert!(state.statefulsets.is_empty());
        assert!(state.error_message.is_none());
        assert!(state.table.selected_row_key.is_none());
        assert!(state.needs_load_for_cluster("ctx-b"));

        // Disconnect clears cluster_id and invalidates in-flight replies.
        let gen_before_disconnect = state.generation;
        state.reset_for_disconnected();
        assert!(state.cluster_id.is_none());
        assert_eq!(state.status, StatefulSetsLoadStatus::Idle);
        assert!(state.generation > gen_before_disconnect);
    }

    fn sample_daemonset(name: &str, namespace: &str) -> DaemonSetInfo {
        DaemonSetInfo {
            kind: "DaemonSet".to_string(),
            name: name.to_string(),
            namespace: namespace.to_string(),
            status: "Running".to_string(),
            replicas: orbit_core::kubernetes::models::DaemonSetReplicas {
                desired: 3,
                current: 3,
                ready: 3,
                up_to_date: 3,
                available: 3,
            },
            age: "2d".to_string(),
            images: vec!["fluent/fluent-bit:3.0".to_string()],
            containers: vec![orbit_core::kubernetes::models::ContainerImageInfo {
                name: "fluent-bit".to_string(),
                image: "fluent/fluent-bit:3.0".to_string(),
            }],
            labels: Default::default(),
            annotations: Default::default(),
        }
    }

    #[test]
    fn test_daemonsets_state_lifecycle_and_generation_guard() {
        let mut state = DaemonSetsState::default();
        assert!(state.needs_load_for_cluster("ctx-a"));

        let cmd = state.begin_load_for_cluster("ctx-a");
        assert_eq!(
            cmd,
            ClusterBridgeCommand::LoadDaemonSets {
                cluster_id: "ctx-a".to_string(),
                generation: 1,
            }
        );
        assert_eq!(state.status, DaemonSetsLoadStatus::Loading);
        assert!(state.table.loading);
        assert!(!state.needs_load_for_cluster("ctx-a"));

        // Stale cluster or generation is rejected.
        assert!(!state.apply_loaded(
            "ctx-other",
            1,
            vec![sample_daemonset("wrong-cluster", "default")],
        ));
        assert!(!state.apply_loaded("ctx-a", 99, vec![sample_daemonset("wrong-gen", "default")],));
        assert!(state.daemonsets.is_empty());

        // Matching cluster and generation is accepted.
        assert!(state.apply_loaded("ctx-a", 1, vec![sample_daemonset("fluent-bit", "logging")],));
        assert_eq!(state.status, DaemonSetsLoadStatus::Loaded);
        assert!(!state.table.loading);
        assert_eq!(state.daemonsets.len(), 1);
        assert_eq!(state.daemonsets[0].name, "fluent-bit");

        // Refresh retains cached rows while loading.
        let refresh = state.begin_refresh();
        assert_eq!(
            refresh,
            Some(ClusterBridgeCommand::LoadDaemonSets {
                cluster_id: "ctx-a".to_string(),
                generation: 2,
            })
        );
        assert_eq!(state.status, DaemonSetsLoadStatus::Loading);
        assert_eq!(state.daemonsets.len(), 1);

        assert!(state.apply_failed("ctx-a", 2, "timeout".to_string()));
        assert_eq!(state.status, DaemonSetsLoadStatus::Error);
        assert_eq!(state.error_message.as_deref(), Some("timeout"));

        // Reset for new cluster clears rows, selection, and returns to Idle.
        state.reset_for_cluster("ctx-b");
        assert_eq!(state.cluster_id.as_deref(), Some("ctx-b"));
        assert_eq!(state.status, DaemonSetsLoadStatus::Idle);
        assert!(state.daemonsets.is_empty());
        assert!(state.error_message.is_none());
        assert!(state.table.selected_row_key.is_none());
        assert!(state.needs_load_for_cluster("ctx-b"));

        // Disconnect clears cluster_id and invalidates in-flight replies.
        let gen_before_disconnect = state.generation;
        state.reset_for_disconnected();
        assert!(state.cluster_id.is_none());
        assert_eq!(state.status, DaemonSetsLoadStatus::Idle);
        assert!(state.generation > gen_before_disconnect);
    }

    fn sample_pod(name: &str, namespace: &str) -> PodInfo {
        PodInfo {
            name: name.to_string(),
            namespace: namespace.to_string(),
            status: "Running".to_string(),
            age: "5m".to_string(),
            cpu: Some("100m".to_string()),
            memory: Some("128Mi".to_string()),
            cpu_cores: Some(0.1),
            memory_bytes: Some(134_217_728.0),
            cpu_pct: None,
            memory_pct: None,
            node: Some("node-1".to_string()),
            restarts: 0,
            images: vec!["nginx:latest".to_string()],
            labels: Default::default(),
            annotations: Default::default(),
            ip: Some("10.0.0.2".to_string()),
            node_ip: Some("192.168.1.2".to_string()),
            controlled_by: None,
            qos_class: Some("Burstable".to_string()),
            containers: vec![],
        }
    }

    #[test]
    fn test_pods_state_lifecycle_and_generation_guard() {
        let mut state = PodsState::default();
        assert!(state.needs_load_for_cluster("ctx-a"));

        let cmd_a = state.begin_load_for_cluster("ctx-a");
        assert_eq!(
            cmd_a,
            ClusterBridgeCommand::LoadPods {
                cluster_id: "ctx-a".to_string(),
                generation: 1,
            }
        );
        assert_eq!(state.status, PodsLoadStatus::Loading);
        assert!(state.table.loading);
        assert!(!state.needs_load_for_cluster("ctx-a"));

        assert!(!state.apply_loaded("ctx-b", 1, vec![sample_pod("wrong-cluster", "default")]));
        assert!(!state.apply_loaded("ctx-a", 99, vec![sample_pod("wrong-gen", "default")]));
        assert!(!state.apply_failed("ctx-a", 99, "stale-err".to_string()));
        assert!(state.pods.is_empty());

        assert!(state.apply_loaded("ctx-a", 1, vec![sample_pod("web-0", "default")]));
        assert_eq!(state.status, PodsLoadStatus::Loaded);
        assert!(!state.table.loading);
        assert_eq!(state.pods.len(), 1);
        assert_eq!(state.pods[0].name, "web-0");

        state.table.selected_row_key = Some("default/web-0".to_string());
        let refresh_cmd = state.begin_refresh().expect("refresh command");
        assert_eq!(
            refresh_cmd,
            ClusterBridgeCommand::LoadPods {
                cluster_id: "ctx-a".to_string(),
                generation: 2,
            }
        );
        assert_eq!(state.status, PodsLoadStatus::Loading);
        assert!(!state.table.loading);
        assert_eq!(state.pods.len(), 1);

        // Switching cluster clears old pods and enables loading skeleton.
        let switch_cmd = state.begin_load_for_cluster("ctx-b");
        assert_eq!(
            switch_cmd,
            ClusterBridgeCommand::LoadPods {
                cluster_id: "ctx-b".to_string(),
                generation: 3,
            }
        );
        assert!(state.pods.is_empty());
        assert!(state.table.selected_row_key.is_none());
        assert!(state.table.loading);

        assert!(state.apply_failed("ctx-b", 3, "timeout".to_string()));
        assert_eq!(state.status, PodsLoadStatus::Error);
        assert_eq!(state.error_message.as_deref(), Some("timeout"));

        state.reset_for_cluster("ctx-c");
        assert_eq!(state.cluster_id.as_deref(), Some("ctx-c"));
        assert_eq!(state.status, PodsLoadStatus::Idle);
        assert!(state.pods.is_empty());
        assert!(state.error_message.is_none());
        assert!(state.needs_load_for_cluster("ctx-c"));

        let gen_before_disconnect = state.generation;
        state.reset_for_disconnected();
        assert!(state.cluster_id.is_none());
        assert_eq!(state.status, PodsLoadStatus::Idle);
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
        assert!(!state.apply_loaded(
            "ctx-b",
            1,
            vec![sample_node("wrong-cluster", "Ready", false)]
        ));
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
            vec![sample_event(
                "ev-wrong-cluster",
                "Normal",
                "Scheduled",
                "default"
            )],
        ));
        assert!(!state.apply_loaded(
            "cluster-a",
            0,
            vec![sample_event(
                "ev-stale-gen",
                "Normal",
                "Scheduled",
                "default"
            )],
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

    #[test]
    fn test_workspace_tabs_open_deduplicate_activate_and_cycle() {
        let mut tabs = WorkspaceTabsState::new();
        tabs.activate_cluster("cluster-a");
        assert_eq!(tabs.open_tabs(), &[WorkspaceView::Overview]);
        assert_eq!(tabs.active_tab(), Some(WorkspaceView::Overview));

        tabs.pending_scroll_to_active = false;
        tabs.open_tab(WorkspaceView::Pods);
        tabs.open_tab(WorkspaceView::Deployments);
        assert_eq!(
            tabs.open_tabs(),
            &[
                WorkspaceView::Overview,
                WorkspaceView::Pods,
                WorkspaceView::Deployments,
            ]
        );
        assert_eq!(tabs.active_tab(), Some(WorkspaceView::Deployments));
        assert!(tabs.pending_scroll_to_active);

        // Deduplicate existing view: activates without duplicating
        tabs.pending_scroll_to_active = false;
        tabs.open_tab(WorkspaceView::Pods);
        assert_eq!(
            tabs.open_tabs(),
            &[
                WorkspaceView::Overview,
                WorkspaceView::Pods,
                WorkspaceView::Deployments,
            ]
        );
        assert_eq!(tabs.active_tab(), Some(WorkspaceView::Pods));
        assert!(tabs.pending_scroll_to_active);

        // Cycle next / prev with wrap-around
        tabs.pending_scroll_to_active = false;
        tabs.activate_next_tab();
        assert_eq!(tabs.active_tab(), Some(WorkspaceView::Deployments));
        assert!(tabs.pending_scroll_to_active);

        tabs.activate_next_tab();
        assert_eq!(tabs.active_tab(), Some(WorkspaceView::Overview));

        tabs.pending_scroll_to_active = false;
        tabs.activate_prev_tab();
        assert_eq!(tabs.active_tab(), Some(WorkspaceView::Deployments));
        assert!(tabs.pending_scroll_to_active);
    }

    #[test]
    fn test_workspace_tabs_close_adjacent_and_empty_fallback() {
        let mut tabs = WorkspaceTabsState::new();
        tabs.activate_cluster("cluster-a");
        tabs.open_tab(WorkspaceView::Pods);
        tabs.open_tab(WorkspaceView::Deployments);
        tabs.open_tab(WorkspaceView::Nodes);
        // [Overview, Pods, Deployments, Nodes], active = Nodes

        // Closing an inactive tab keeps active_tab unchanged
        tabs.close_tab(WorkspaceView::Overview);
        assert_eq!(
            tabs.open_tabs(),
            &[
                WorkspaceView::Pods,
                WorkspaceView::Deployments,
                WorkspaceView::Nodes,
            ]
        );
        assert_eq!(tabs.active_tab(), Some(WorkspaceView::Nodes));

        // Closing rightmost active tab activates left neighbor
        tabs.close_active_tab();
        assert_eq!(
            tabs.open_tabs(),
            &[WorkspaceView::Pods, WorkspaceView::Deployments]
        );
        assert_eq!(tabs.active_tab(), Some(WorkspaceView::Deployments));

        // Closing middle active tab activates right neighbor
        tabs.open_tab(WorkspaceView::Nodes);
        tabs.set_active_tab(WorkspaceView::Deployments);
        tabs.close_tab(WorkspaceView::Deployments);
        assert_eq!(
            tabs.open_tabs(),
            &[WorkspaceView::Pods, WorkspaceView::Nodes]
        );
        assert_eq!(tabs.active_tab(), Some(WorkspaceView::Nodes));

        // Closing remaining tabs leads to empty state and hides bar
        assert!(tabs.is_bar_visible(true));
        tabs.close_active_tab();
        assert_eq!(tabs.open_tabs(), &[WorkspaceView::Pods]);
        assert_eq!(tabs.active_tab(), Some(WorkspaceView::Pods));
        assert!(tabs.is_bar_visible(true));

        tabs.close_active_tab();
        assert!(tabs.open_tabs().is_empty());
        assert_eq!(tabs.active_tab(), None);
        assert!(!tabs.is_bar_visible(true));
        assert!(!tabs.is_bar_visible(false));
    }

    #[test]
    fn test_workspace_tabs_context_actions_and_drag_reorder() {
        let mut tabs = WorkspaceTabsState::new();
        tabs.activate_cluster("cluster-a");
        tabs.open_tab(WorkspaceView::Pods);
        tabs.open_tab(WorkspaceView::Deployments);
        tabs.open_tab(WorkspaceView::Nodes);
        // [Overview, Pods, Deployments, Nodes]

        tabs.move_tab_left(WorkspaceView::Deployments);
        assert_eq!(
            tabs.open_tabs(),
            &[
                WorkspaceView::Overview,
                WorkspaceView::Deployments,
                WorkspaceView::Pods,
                WorkspaceView::Nodes,
            ]
        );

        tabs.move_tab_right(WorkspaceView::Deployments);
        assert_eq!(
            tabs.open_tabs(),
            &[
                WorkspaceView::Overview,
                WorkspaceView::Pods,
                WorkspaceView::Deployments,
                WorkspaceView::Nodes,
            ]
        );

        assert_eq!(calculate_tab_drop_index(0, 2, TabDropPosition::Before), 1);
        assert_eq!(calculate_tab_drop_index(0, 2, TabDropPosition::After), 2);
        assert_eq!(calculate_tab_drop_index(3, 1, TabDropPosition::Before), 1);
        assert_eq!(calculate_tab_drop_index(3, 1, TabDropPosition::After), 2);

        let to_idx = calculate_tab_drop_index(0, 2, TabDropPosition::After);
        tabs.reorder_tab(0, to_idx);
        assert_eq!(
            tabs.open_tabs(),
            &[
                WorkspaceView::Pods,
                WorkspaceView::Deployments,
                WorkspaceView::Overview,
                WorkspaceView::Nodes,
            ]
        );

        tabs.close_tabs_to_right(WorkspaceView::Deployments);
        assert_eq!(
            tabs.open_tabs(),
            &[WorkspaceView::Pods, WorkspaceView::Deployments]
        );

        tabs.open_tab(WorkspaceView::Services);
        tabs.close_other_tabs(WorkspaceView::Deployments);
        assert_eq!(tabs.open_tabs(), &[WorkspaceView::Deployments]);
        assert_eq!(tabs.active_tab(), Some(WorkspaceView::Deployments));

        tabs.close_all_tabs();
        assert!(tabs.open_tabs().is_empty());
        assert_eq!(tabs.active_tab(), None);
    }

    #[test]
    fn test_workspace_tabs_per_cluster_isolation_and_config_hydration() {
        let mut config = OrbitConfig::default();
        config.update_workspace_tabs_for_cluster(
            "cluster-a",
            vec![
                WorkspaceView::Pods.id().to_string(),
                "nonexistent-view".to_string(),
                WorkspaceView::Deployments.id().to_string(),
            ],
            Some(WorkspaceView::Deployments.id().to_string()),
        );
        config.update_workspace_tabs_for_cluster("cluster-empty", Vec::new(), None);

        let mut tabs = WorkspaceTabsState::from_config(&config);

        // Cluster A hydrates valid view IDs and ignores unknown IDs
        assert_eq!(
            tabs.activate_cluster("cluster-a"),
            Some(WorkspaceView::Deployments)
        );
        assert_eq!(
            tabs.open_tabs(),
            &[WorkspaceView::Pods, WorkspaceView::Deployments]
        );

        // Explicitly emptied cluster stays empty
        assert_eq!(tabs.activate_cluster("cluster-empty"), None);
        assert!(tabs.open_tabs().is_empty());

        // Never-visited cluster initializes with [Overview]
        assert_eq!(
            tabs.activate_cluster("cluster-new"),
            Some(WorkspaceView::Overview)
        );
        assert_eq!(tabs.open_tabs(), &[WorkspaceView::Overview]);
        tabs.open_tab(WorkspaceView::Nodes);

        // Switching back to cluster A preserves its independent list
        assert_eq!(
            tabs.activate_cluster("cluster-a"),
            Some(WorkspaceView::Deployments)
        );
        assert_eq!(
            tabs.open_tabs(),
            &[WorkspaceView::Pods, WorkspaceView::Deployments]
        );

        // Switching to cluster-new preserves its Nodes tab
        assert_eq!(
            tabs.activate_cluster("cluster-new"),
            Some(WorkspaceView::Nodes)
        );
        assert_eq!(
            tabs.open_tabs(),
            &[WorkspaceView::Overview, WorkspaceView::Nodes]
        );
    }

    #[test]
    fn test_workspace_tabs_picker_search_filtering() {
        let mut tabs = WorkspaceTabsState::new();
        tabs.open_picker();
        assert!(tabs.picker_open);
        assert!(!tabs.filtered_picker_views().is_empty());

        tabs.handle_picker_search_key("d", Some("d"));
        tabs.handle_picker_search_key("e", Some("e"));
        tabs.handle_picker_search_key("p", Some("p"));
        tabs.handle_picker_search_key("l", Some("l"));
        let filtered = tabs.filtered_picker_views();
        assert!(filtered.contains(&WorkspaceView::Deployments));
        assert!(!filtered.contains(&WorkspaceView::Pods));

        tabs.handle_picker_search_key("backspace", None);
        tabs.handle_picker_search_key("backspace", None);
        tabs.handle_picker_search_key("backspace", None);
        tabs.handle_picker_search_key("backspace", None);
        assert_eq!(tabs.picker_query, "");

        // Filter by category label ("network")
        for ch in ["n", "e", "t", "w", "o", "r", "k"] {
            tabs.handle_picker_search_key(ch, Some(ch));
        }
        let net_filtered = tabs.filtered_picker_views();
        assert!(net_filtered.contains(&WorkspaceView::Services));
        assert!(net_filtered.contains(&WorkspaceView::Ingresses));

        tabs.handle_picker_search_key("escape", None);
        assert!(!tabs.picker_open);
        assert_eq!(tabs.picker_query, "");
    }

    #[test]
    fn test_services_state_lifecycle_and_generation_guard() {
        let mut state = ServicesState::default();
        assert_eq!(state.status, ServicesLoadStatus::Idle);
        assert!(state.needs_load_for_cluster("prod-ctx"));

        let cmd = state.begin_load_for_cluster("prod-ctx");
        assert_eq!(
            cmd,
            ClusterBridgeCommand::LoadServices {
                cluster_id: "prod-ctx".to_string(),
                generation: 1,
            }
        );
        assert_eq!(state.status, ServicesLoadStatus::Loading);
        assert!(state.table.loading);

        let sample = ServiceInfo {
            name: "api-svc".to_string(),
            namespace: "prod".to_string(),
            r#type: "ClusterIP".to_string(),
            cluster_ip: "10.96.0.10".to_string(),
            external_ip: "-".to_string(),
            ports: "80/TCP".to_string(),
            endpoints: "2 ready".to_string(),
            age: "1h".to_string(),
            session_affinity: "None".to_string(),
            internal_traffic_policy: None,
            created: String::new(),
            uid: "uid-1".to_string(),
            selector: Default::default(),
            labels: Default::default(),
            ports_list: Vec::new(),
            endpoints_list: Vec::new(),
            events: Vec::new(),
        };

        // Stale cluster or generation must be ignored
        assert!(!state.apply_loaded("other-ctx", 1, vec![sample.clone()]));
        assert!(!state.apply_loaded("prod-ctx", 99, vec![sample.clone()]));
        assert!(!state.apply_failed("prod-ctx", 99, "boom".to_string()));
        assert!(state.services.is_empty());

        // Matching generation updates state
        assert!(state.apply_loaded("prod-ctx", 1, vec![sample]));
        assert_eq!(state.status, ServicesLoadStatus::Loaded);
        assert_eq!(state.services.len(), 1);
        assert!(!state.table.loading);
        assert!(!state.needs_load_for_cluster("prod-ctx"));

        // Disconnect clears cluster_id and services
        state.reset_for_disconnected();
        assert_eq!(state.status, ServicesLoadStatus::Idle);
        assert!(state.cluster_id.is_none());
        assert!(state.services.is_empty());
    }

    #[test]
    fn test_configmaps_state_lifecycle_and_stale_generation_guard() {
        let mut state = ConfigMapsState::default();
        assert_eq!(state.status, ConfigMapsLoadStatus::Idle);
        assert!(state.needs_load_for_cluster("ctx-a"));

        let cmd = state.begin_load_for_cluster("ctx-a");
        assert_eq!(
            cmd,
            ClusterBridgeCommand::LoadConfigMaps {
                cluster_id: "ctx-a".to_string(),
                generation: 1,
            }
        );
        assert_eq!(state.status, ConfigMapsLoadStatus::Loading);
        assert!(state.table.loading);

        let sample = ConfigMapInfo {
            name: "app-config".to_string(),
            namespace: "prod".to_string(),
            labels: Default::default(),
            annotations: 0,
            created: String::new(),
            age: "2h".to_string(),
            resource_version: "10".to_string(),
            immutable: false,
            keys_count: 2,
            size: "128 B".to_string(),
            mounted_pods: 1,
            used_by: Vec::new(),
            data: Default::default(),
        };

        // Stale generation or wrong cluster must be ignored
        assert!(!state.apply_loaded("ctx-b", 1, vec![sample.clone()]));
        assert!(!state.apply_loaded("ctx-a", 0, vec![sample.clone()]));
        assert!(!state.apply_failed("ctx-a", 0, "boom".to_string()));
        assert!(state.configmaps.is_empty());

        // Matching generation updates state
        assert!(state.apply_loaded("ctx-a", 1, vec![sample]));
        assert_eq!(state.status, ConfigMapsLoadStatus::Loaded);
        assert_eq!(state.configmaps.len(), 1);
        assert!(!state.table.loading);
        assert!(!state.needs_load_for_cluster("ctx-a"));

        // Disconnect clears cluster_id and configmaps
        state.reset_for_disconnected();
        assert_eq!(state.status, ConfigMapsLoadStatus::Idle);
        assert!(state.cluster_id.is_none());
        assert!(state.configmaps.is_empty());
    }

    #[test]
    fn test_secrets_state_lifecycle_and_generation_guard() {
        let mut state = SecretsState::default();
        assert_eq!(state.status, SecretsLoadStatus::Idle);
        assert!(state.needs_load_for_cluster("ctx-a"));

        let cmd = state.begin_load_for_cluster("ctx-a");
        assert_eq!(
            cmd,
            ClusterBridgeCommand::LoadSecrets {
                cluster_id: "ctx-a".to_string(),
                generation: 1,
            }
        );
        assert_eq!(state.status, SecretsLoadStatus::Loading);
        assert!(state.table.loading);

        let sample = SecretInfo {
            name: "app-secret".to_string(),
            namespace: "prod".to_string(),
            labels: Default::default(),
            annotations: 0,
            created: String::new(),
            age: "2h".to_string(),
            resource_version: "10".to_string(),
            r#type: "Opaque".to_string(),
            immutable: false,
            keys_count: 1,
            size: "16 B".to_string(),
            mounted_pods: 1,
            used_by: Vec::new(),
            data: Default::default(),
        };

        // Stale generation or wrong cluster must be ignored
        assert!(!state.apply_loaded("ctx-b", 1, vec![sample.clone()]));
        assert!(!state.apply_loaded("ctx-a", 0, vec![sample.clone()]));
        assert!(!state.apply_failed("ctx-a", 0, "boom".to_string()));
        assert!(state.secrets.is_empty());

        // Matching generation updates state
        assert!(state.apply_loaded("ctx-a", 1, vec![sample]));
        assert_eq!(state.status, SecretsLoadStatus::Loaded);
        assert_eq!(state.secrets.len(), 1);
        assert!(!state.table.loading);
        assert!(!state.needs_load_for_cluster("ctx-a"));

        // Refresh + failure transitions to Error while retaining cached secrets
        let refresh_cmd = state.begin_refresh();
        assert_eq!(
            refresh_cmd,
            Some(ClusterBridgeCommand::LoadSecrets {
                cluster_id: "ctx-a".to_string(),
                generation: 2,
            })
        );
        assert_eq!(state.status, SecretsLoadStatus::Loading);
        assert!(!state.table.loading);
        assert!(state.apply_failed("ctx-a", 2, "forbidden".to_string()));
        assert_eq!(state.status, SecretsLoadStatus::Error);
        assert_eq!(state.error_message.as_deref(), Some("forbidden"));
        assert_eq!(state.secrets.len(), 1);

        // Reset for cluster clears secrets and returns to Idle
        state.reset_for_cluster("ctx-b");
        assert_eq!(state.status, SecretsLoadStatus::Idle);
        assert_eq!(state.cluster_id.as_deref(), Some("ctx-b"));
        assert!(state.secrets.is_empty());

        // Disconnect clears cluster_id and secrets
        state.reset_for_disconnected();
        assert_eq!(state.status, SecretsLoadStatus::Idle);
        assert!(state.cluster_id.is_none());
        assert!(state.secrets.is_empty());
    }

    #[test]
    fn test_workloads_overview_state_lifecycle_and_stale_generation_guard() {
        use orbit_core::kubernetes::workloads_overview::{
            WorkloadHealthItem, WorkloadHealthStatus, WorkloadKindSnapshot,
        };

        let mut state = WorkloadsOverviewState::default();
        assert!(state.needs_load_for_cluster("ctx-a"));

        let cmd = state.begin_load_for_cluster("ctx-a");
        assert_eq!(
            cmd,
            ClusterBridgeCommand::LoadWorkloadsOverview {
                cluster_id: "ctx-a".to_string(),
                generation: 1,
            }
        );
        assert_eq!(state.status, WorkloadsOverviewLoadStatus::Loading);
        assert!(!state.needs_load_for_cluster("ctx-a"));
        assert!(state.needs_load_for_cluster("ctx-b"));

        let sample = WorkloadsOverviewSnapshot {
            pods: WorkloadKindSnapshot {
                items: vec![WorkloadHealthItem {
                    namespace: "default".to_string(),
                    status: WorkloadHealthStatus::Healthy,
                }],
                error: None,
            },
            ..Default::default()
        };

        // Stale cluster or generation must be ignored
        assert!(!state.apply_loaded("ctx-b", 1, sample.clone()));
        assert!(!state.apply_loaded("ctx-a", 0, sample.clone()));
        assert!(!state.apply_failed("ctx-a", 0, "stale".to_string()));
        assert!(state.snapshot.is_none());

        // Valid snapshot updates state
        assert!(state.apply_loaded("ctx-a", 1, sample.clone()));
        assert_eq!(state.status, WorkloadsOverviewLoadStatus::Loaded);
        assert_eq!(state.snapshot.as_ref(), Some(&sample));
        assert!(state.error_message.is_none());

        // Refresh preserves existing snapshot while loading
        let refresh = state.begin_refresh();
        assert_eq!(
            refresh,
            Some(ClusterBridgeCommand::LoadWorkloadsOverview {
                cluster_id: "ctx-a".to_string(),
                generation: 2,
            })
        );
        assert_eq!(state.status, WorkloadsOverviewLoadStatus::Loading);
        assert_eq!(state.snapshot.as_ref(), Some(&sample));

        // Failure on current generation sets Error and keeps existing snapshot
        assert!(state.apply_failed("ctx-a", 2, "network error".to_string()));
        assert_eq!(state.status, WorkloadsOverviewLoadStatus::Error);
        assert_eq!(state.error_message.as_deref(), Some("network error"));
        assert_eq!(state.snapshot.as_ref(), Some(&sample));

        // Reset for cluster clears snapshot and transitions to Idle
        state.reset_for_cluster("ctx-b");
        assert_eq!(state.status, WorkloadsOverviewLoadStatus::Idle);
        assert_eq!(state.cluster_id.as_deref(), Some("ctx-b"));
        assert!(state.snapshot.is_none());
        assert!(state.needs_load_for_cluster("ctx-b"));

        // Reset for disconnected clears cluster_id and snapshot
        state.reset_for_disconnected();
        assert_eq!(state.status, WorkloadsOverviewLoadStatus::Idle);
        assert!(state.cluster_id.is_none());
        assert!(state.snapshot.is_none());
    }

    fn sample_job(name: &str, namespace: &str) -> JobInfo {
        JobInfo {
            kind: "Job".to_string(),
            name: name.to_string(),
            namespace: namespace.to_string(),
            status: "Succeeded".to_string(),
            completions: "1/1".to_string(),
            duration: Some("30s".to_string()),
            age: "5m".to_string(),
            images: vec!["busybox:1.36".to_string()],
            containers: Vec::new(),
            labels: Default::default(),
            annotations: Default::default(),
        }
    }

    #[test]
    fn test_jobs_state_lifecycle_and_generation_guard() {
        let mut state = JobsState::default();
        assert!(state.needs_load_for_cluster("ctx-a"));

        let cmd = state.begin_load_for_cluster("ctx-a");
        assert_eq!(
            cmd,
            ClusterBridgeCommand::LoadJobs {
                cluster_id: "ctx-a".to_string(),
                generation: 1,
            }
        );
        assert_eq!(state.status, JobsLoadStatus::Loading);
        assert!(state.table.loading);

        let job = sample_job("db-migrate", "prod");

        // Stale cluster or generation must be rejected
        assert!(!state.apply_loaded("ctx-other", 1, vec![job.clone()]));
        assert!(!state.apply_loaded("ctx-a", 99, vec![job.clone()]));
        assert!(!state.apply_failed("ctx-other", 1, "boom".to_string()));
        assert!(!state.apply_failed("ctx-a", 99, "boom".to_string()));
        assert!(state.jobs.is_empty());

        // Matching cluster and generation loads rows
        assert!(state.apply_loaded("ctx-a", 1, vec![job]));
        assert_eq!(state.status, JobsLoadStatus::Loaded);
        assert_eq!(state.jobs.len(), 1);
        assert!(!state.table.loading);
        assert!(!state.needs_load_for_cluster("ctx-a"));

        // Refresh increments generation while retaining cached rows
        let refresh = state.begin_refresh();
        assert_eq!(
            refresh,
            Some(ClusterBridgeCommand::LoadJobs {
                cluster_id: "ctx-a".to_string(),
                generation: 2,
            })
        );
        assert_eq!(state.status, JobsLoadStatus::Loading);
        assert!(!state.table.loading);
        assert_eq!(state.jobs.len(), 1);

        // Failure on generation 2 transitions to Error while retaining cached rows
        assert!(state.apply_failed("ctx-a", 2, "timeout".to_string()));
        assert_eq!(state.status, JobsLoadStatus::Error);
        assert_eq!(state.error_message.as_deref(), Some("timeout"));
        assert_eq!(state.jobs.len(), 1);

        // Reset for another cluster clears rows/selection/error and invalidates in-flight replies
        state.table.selected_row_key = Some("prod/db-migrate".to_string());
        state.reset_for_cluster("ctx-b");
        assert_eq!(state.status, JobsLoadStatus::Idle);
        assert_eq!(state.cluster_id.as_deref(), Some("ctx-b"));
        assert!(state.jobs.is_empty());
        assert!(state.error_message.is_none());
        assert!(state.table.selected_row_key.is_none());
        assert!(!state.apply_loaded("ctx-a", 2, vec![sample_job("stale", "prod")]));

        // Reset for disconnected clears cluster_id and rows
        state.reset_for_disconnected();
        assert_eq!(state.status, JobsLoadStatus::Idle);
        assert!(state.cluster_id.is_none());
        assert!(state.jobs.is_empty());
        assert!(state.error_message.is_none());
        assert!(state.table.selected_row_key.is_none());
    }

    fn sample_cronjob(name: &str, namespace: &str) -> CronJobInfo {
        CronJobInfo {
            kind: "CronJob".to_string(),
            name: name.to_string(),
            namespace: namespace.to_string(),
            schedule: "0 2 * * *".to_string(),
            suspend: false,
            active: 1,
            last_schedule: Some("5m ago".to_string()),
            age: "2d".to_string(),
            images: vec!["busybox:1.36".to_string()],
            containers: Vec::new(),
            labels: Default::default(),
            annotations: Default::default(),
        }
    }

    #[test]
    fn test_cronjobs_state_lifecycle_and_generation_guard() {
        let mut state = CronJobsState::default();
        assert!(state.needs_load_for_cluster("ctx-a"));

        let cmd = state.begin_load_for_cluster("ctx-a");
        assert_eq!(
            cmd,
            ClusterBridgeCommand::LoadCronJobs {
                cluster_id: "ctx-a".to_string(),
                generation: 1,
            }
        );
        assert_eq!(state.status, CronJobsLoadStatus::Loading);
        assert!(state.table.loading);

        let cj = sample_cronjob("nightly-backup", "prod");

        // Stale cluster or generation must be rejected
        assert!(!state.apply_loaded("ctx-other", 1, vec![cj.clone()]));
        assert!(!state.apply_loaded("ctx-a", 99, vec![cj.clone()]));
        assert!(!state.apply_failed("ctx-other", 1, "boom".to_string()));
        assert!(!state.apply_failed("ctx-a", 99, "boom".to_string()));
        assert!(state.cronjobs.is_empty());

        // Matching cluster and generation loads rows
        assert!(state.apply_loaded("ctx-a", 1, vec![cj]));
        assert_eq!(state.status, CronJobsLoadStatus::Loaded);
        assert_eq!(state.cronjobs.len(), 1);
        assert!(!state.table.loading);
        assert!(!state.needs_load_for_cluster("ctx-a"));

        // Refresh increments generation while retaining cached rows
        let refresh = state.begin_refresh();
        assert_eq!(
            refresh,
            Some(ClusterBridgeCommand::LoadCronJobs {
                cluster_id: "ctx-a".to_string(),
                generation: 2,
            })
        );
        assert_eq!(state.status, CronJobsLoadStatus::Loading);
        assert!(!state.table.loading);
        assert_eq!(state.cronjobs.len(), 1);

        // Failure on generation 2 transitions to Error while retaining cached rows
        assert!(state.apply_failed("ctx-a", 2, "timeout".to_string()));
        assert_eq!(state.status, CronJobsLoadStatus::Error);
        assert_eq!(state.error_message.as_deref(), Some("timeout"));
        assert_eq!(state.cronjobs.len(), 1);

        // Reset for another cluster clears rows/selection/error and invalidates in-flight replies
        state.table.selected_row_key = Some("prod/nightly-backup".to_string());
        state.reset_for_cluster("ctx-b");
        assert_eq!(state.status, CronJobsLoadStatus::Idle);
        assert_eq!(state.cluster_id.as_deref(), Some("ctx-b"));
        assert!(state.cronjobs.is_empty());
        assert!(state.error_message.is_none());
        assert!(state.table.selected_row_key.is_none());
        assert!(!state.apply_loaded("ctx-a", 2, vec![sample_cronjob("stale", "prod")]));

        // Reset for disconnected clears cluster_id and rows
        state.reset_for_disconnected();
        assert_eq!(state.status, CronJobsLoadStatus::Idle);
        assert!(state.cluster_id.is_none());
        assert!(state.cronjobs.is_empty());
        assert!(state.error_message.is_none());
        assert!(state.table.selected_row_key.is_none());
    }

    fn sample_ingress(name: &str, namespace: &str) -> IngressInfo {
        IngressInfo {
            name: name.to_string(),
            namespace: namespace.to_string(),
            class_name: Some("nginx".to_string()),
            hosts: "app.example.com".to_string(),
            address: "34.120.0.1".to_string(),
            ports: "80, 443".to_string(),
            age: "2d".to_string(),
            created: "Nov 18, 2026, 10:00 AM".to_string(),
            uid: "uid-ing-1".to_string(),
            labels: Default::default(),
            annotations: Default::default(),
            rules_summary: vec!["app.example.com -> / (web-svc:80)".to_string()],
        }
    }

    #[test]
    fn test_ingresses_state_lifecycle_and_generation_guard() {
        let mut state = IngressesState::default();
        assert!(state.needs_load_for_cluster("ctx-a"));

        let cmd = state.begin_load_for_cluster("ctx-a");
        assert_eq!(
            cmd,
            ClusterBridgeCommand::LoadIngresses {
                cluster_id: "ctx-a".to_string(),
                generation: 1,
            }
        );
        assert_eq!(state.status, IngressesLoadStatus::Loading);
        assert!(state.table.loading);

        let ing = sample_ingress("web-ingress", "prod");

        // Stale cluster or generation must be rejected
        assert!(!state.apply_loaded("ctx-other", 1, vec![ing.clone()]));
        assert!(!state.apply_loaded("ctx-a", 99, vec![ing.clone()]));
        assert!(!state.apply_failed("ctx-other", 1, "boom".to_string()));
        assert!(!state.apply_failed("ctx-a", 99, "boom".to_string()));
        assert!(state.ingresses.is_empty());

        // Matching cluster and generation loads rows
        assert!(state.apply_loaded("ctx-a", 1, vec![ing]));
        assert_eq!(state.status, IngressesLoadStatus::Loaded);
        assert_eq!(state.ingresses.len(), 1);
        assert!(!state.table.loading);
        assert!(!state.needs_load_for_cluster("ctx-a"));

        // Refresh increments generation while retaining cached rows
        let refresh = state.begin_refresh();
        assert_eq!(
            refresh,
            Some(ClusterBridgeCommand::LoadIngresses {
                cluster_id: "ctx-a".to_string(),
                generation: 2,
            })
        );
        assert_eq!(state.status, IngressesLoadStatus::Loading);
        assert!(!state.table.loading);
        assert_eq!(state.ingresses.len(), 1);

        // Failure on generation 2 transitions to Error while retaining cached rows
        assert!(state.apply_failed("ctx-a", 2, "timeout".to_string()));
        assert_eq!(state.status, IngressesLoadStatus::Error);
        assert_eq!(state.error_message.as_deref(), Some("timeout"));
        assert_eq!(state.ingresses.len(), 1);

        // Reset for another cluster clears rows/selection/error and invalidates in-flight replies
        state.table.selected_row_key = Some("prod/web-ingress".to_string());
        state.reset_for_cluster("ctx-b");
        assert_eq!(state.status, IngressesLoadStatus::Idle);
        assert_eq!(state.cluster_id.as_deref(), Some("ctx-b"));
        assert!(state.ingresses.is_empty());
        assert!(state.error_message.is_none());
        assert!(state.table.selected_row_key.is_none());
        assert!(!state.apply_loaded("ctx-a", 2, vec![sample_ingress("stale", "prod")]));

        // Reset for disconnected clears cluster_id and rows
        state.reset_for_disconnected();
        assert_eq!(state.status, IngressesLoadStatus::Idle);
        assert!(state.cluster_id.is_none());
        assert!(state.ingresses.is_empty());
        assert!(state.error_message.is_none());
        assert!(state.table.selected_row_key.is_none());
    }
}
