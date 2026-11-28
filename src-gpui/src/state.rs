use std::collections::{HashMap, HashSet};

use gpui::ScrollHandle;
use orbit_core::config::{OrbitConfig, SelectedNamespacesMap};
use orbit_core::kubernetes::kubeconfig_import::AddClusterRequest;
use orbit_core::kubernetes::logs::WorkloadPodTarget;
use orbit_core::kubernetes::manager::ClusterSwitchOutcome;
use orbit_core::kubernetes::models::{
    ClusterInfo, ConfigMapInfo, CronJobInfo, DaemonSetInfo, DeploymentInfo, EventInfo, HpaInfo,
    IngressInfo, JobInfo, NamespaceInfo, NodeInfo, PersistentVolumeClaimInfo, PersistentVolumeInfo,
    PodInfo, PolicyInfo, ReplicaSetInfo, SecretInfo, ServiceInfo, StatefulSetInfo,
    StorageClassInfo, UserProfileInfo,
};
use orbit_core::kubernetes::overview::ClusterOverviewSummary;
use orbit_core::kubernetes::port_forward::PersistedPortForward;
use orbit_core::kubernetes::storage_overview::StorageOverviewSnapshot;
use orbit_core::kubernetes::workloads_overview::WorkloadsOverviewSnapshot;

pub use crate::add_cluster_state::*;
use crate::components::data_table::DataTableState;
use crate::navigation::{LogsTabTarget, SidebarCategory, WorkspaceTab, WorkspaceView};

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
    DisconnectCluster,
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
    LoadReplicaSets {
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
    LoadPersistentVolumes {
        cluster_id: String,
        generation: u64,
    },
    LoadPersistentVolumeClaims {
        cluster_id: String,
        generation: u64,
    },
    LoadStorageOverview {
        cluster_id: String,
        generation: u64,
    },
    LoadNetworkPolicies {
        cluster_id: String,
        generation: u64,
    },
    LoadPodSecurity {
        cluster_id: String,
        generation: u64,
    },
    LoadResourceQuotas {
        cluster_id: String,
        generation: u64,
    },
    LoadHpas {
        cluster_id: String,
        generation: u64,
    },
    LoadStorageClasses {
        cluster_id: String,
        generation: u64,
    },
    LoadSecurityOverview {
        cluster_id: String,
        generation: u64,
    },
    LoadAdmissionWebhooks {
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
    SaveTableConfig {
        table_id: String,
        config: orbit_core::config::PersistedTableConfig,
    },
    SaveClusterTableFilters {
        cluster_id: String,
        table_id: String,
        filters: orbit_core::config::PersistedClusterTableFilters,
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
    AddCluster(AddClusterRequest),
    ScaleWorkload {
        cluster_id: String,
        namespace: String,
        kind: String,
        name: String,
        replicas: i32,
    },
    RestartWorkload {
        cluster_id: String,
        namespace: String,
        kind: String,
        name: String,
    },
    RestartPod {
        cluster_id: String,
        namespace: String,
        name: String,
    },
    TriggerCronJob {
        cluster_id: String,
        namespace: String,
        name: String,
    },
    DeleteResource {
        cluster_id: String,
        namespace: Option<String>,
        kind: String,
        name: String,
    },
    StartLogStream {
        cluster_id: String,
        stream_id: u64,
        namespace: String,
        workload_kind: String,
        workload_name: String,
        pod_filter: Option<String>,
        container_filter: Option<String>,
        tail_lines: i64,
        include_unavailable_pods: bool,
    },
    StopLogStream {
        stream_id: u64,
    },
    LoadPortForwards {
        cluster_id: String,
        generation: u64,
    },
    StartPortForward {
        cluster_id: String,
        namespace: String,
        kind: String,
        name: String,
        local_port: u16,
        remote_port: u16,
    },
    StopPortForward {
        cluster_id: String,
        id: String,
    },
    StopSelectedPortForwards {
        cluster_id: String,
        ids: Vec<String>,
    },
    StopAllPortForwards {
        cluster_id: String,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WorkloadActionKind {
    Scale,
    Restart,
    RestartPod,
    TriggerCronJob,
    Delete,
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
    ClusterDisconnected {
        previous_cluster_id: Option<String>,
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
    ReplicaSetsLoaded {
        cluster_id: String,
        generation: u64,
        replicasets: Vec<ReplicaSetInfo>,
    },
    ReplicaSetsLoadFailed {
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
    PersistentVolumesLoaded {
        cluster_id: String,
        generation: u64,
        persistent_volumes: Vec<PersistentVolumeInfo>,
    },
    PersistentVolumesLoadFailed {
        cluster_id: String,
        generation: u64,
        message: String,
    },
    PersistentVolumeClaimsLoaded {
        cluster_id: String,
        generation: u64,
        persistent_volume_claims: Vec<PersistentVolumeClaimInfo>,
    },
    PersistentVolumeClaimsLoadFailed {
        cluster_id: String,
        generation: u64,
        message: String,
    },
    StorageOverviewLoaded {
        cluster_id: String,
        generation: u64,
        snapshot: StorageOverviewSnapshot,
    },
    StorageOverviewLoadFailed {
        cluster_id: String,
        generation: u64,
        message: String,
    },
    NetworkPoliciesLoaded {
        cluster_id: String,
        generation: u64,
        network_policies: Vec<PolicyInfo>,
    },
    NetworkPoliciesLoadFailed {
        cluster_id: String,
        generation: u64,
        message: String,
    },
    PodSecurityLoaded {
        cluster_id: String,
        generation: u64,
        policies: Vec<PolicyInfo>,
    },
    PodSecurityLoadFailed {
        cluster_id: String,
        generation: u64,
        message: String,
    },
    HpasLoaded {
        cluster_id: String,
        generation: u64,
        hpas: Vec<HpaInfo>,
    },
    HpasLoadFailed {
        cluster_id: String,
        generation: u64,
        message: String,
    },
    StorageClassesLoaded {
        cluster_id: String,
        generation: u64,
        classes: Vec<StorageClassInfo>,
    },
    StorageClassesLoadFailed {
        cluster_id: String,
        generation: u64,
        message: String,
    },
    SecurityOverviewLoaded {
        cluster_id: String,
        generation: u64,
        policies: Vec<PolicyInfo>,
    },
    SecurityOverviewLoadFailed {
        cluster_id: String,
        generation: u64,
        message: String,
    },
    AdmissionWebhooksLoaded {
        cluster_id: String,
        generation: u64,
        policies: Vec<PolicyInfo>,
    },
    AdmissionWebhooksLoadFailed {
        cluster_id: String,
        generation: u64,
        message: String,
    },
    ResourceQuotasLoaded {
        cluster_id: String,
        generation: u64,
        policies: Vec<PolicyInfo>,
    },
    ResourceQuotasLoadFailed {
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
    ClusterAdded {
        added_contexts: Vec<String>,
        clusters: Vec<ClusterInfo>,
        switch_outcome: Option<ClusterSwitchOutcome>,
        profile: Option<UserProfileInfo>,
    },
    AddClusterFailed {
        message: String,
    },
    WorkloadActionSucceeded {
        cluster_id: String,
        action: WorkloadActionKind,
        kind: String,
        namespace: String,
        name: String,
        message: String,
    },
    WorkloadActionFailed {
        cluster_id: String,
        action: WorkloadActionKind,
        kind: String,
        namespace: String,
        name: String,
        message: String,
    },
    LogTargetsResolved {
        cluster_id: String,
        stream_id: u64,
        pods: Vec<WorkloadPodTarget>,
        active_pod_count: usize,
        excluded_unavailable_count: usize,
    },
    LogLinesReceived {
        cluster_id: String,
        stream_id: u64,
        pod: String,
        container: String,
        lines: Vec<String>,
    },
    LogStreamStatus {
        cluster_id: String,
        stream_id: u64,
        message: String,
        is_error: bool,
    },
    LogStreamEnded {
        cluster_id: String,
        stream_id: u64,
    },
    PortForwardsLoaded {
        cluster_id: String,
        generation: u64,
        port_forwards: Vec<PersistedPortForward>,
    },
    PortForwardStarted {
        cluster_id: String,
        port_forward: PersistedPortForward,
    },
    PortForwardStopped {
        cluster_id: String,
        id: String,
    },
    PortForwardsCleared {
        cluster_id: String,
    },
    PortForwardFailed {
        cluster_id: String,
        id: Option<String>,
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
        self.table.refreshing = false;
    }

    pub fn reset_for_disconnected(&mut self) {
        self.table.selected_row_key = None;
        self.table.loading = false;
        self.table.refreshing = false;
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

/// Defines a cluster-scoped, generation-guarded list state machine.
/// Namespace filtering is applied client-side by the views (see `NamespaceFilterState`).
macro_rules! cluster_list_state {
    (@methods $state:ident, $status:ident, $field:ident, $item:ty, $cmd:ident) => {
        pub fn begin_load_for_cluster(&mut self, cluster_id: &str) -> ClusterBridgeCommand {
            let switching_cluster = self.cluster_id.as_deref() != Some(cluster_id);
            self.generation += 1;
            self.cluster_id = Some(cluster_id.to_string());
            self.status = $status::Loading;
            self.error_message = None;
            if switching_cluster {
                self.$field.clear();
                self.table.selected_row_key = None;
            }
            self.table.loading = self.$field.is_empty();
            self.table.refreshing = !self.$field.is_empty();
            ClusterBridgeCommand::$cmd {
                cluster_id: cluster_id.to_string(),
                generation: self.generation,
            }
        }

        pub fn begin_refresh(&mut self) -> Option<ClusterBridgeCommand> {
            let cluster_id = self.cluster_id.clone()?;
            self.generation += 1;
            self.status = $status::Loading;
            self.error_message = None;
            self.table.loading = self.$field.is_empty();
            self.table.refreshing = !self.$field.is_empty();
            Some(ClusterBridgeCommand::$cmd {
                cluster_id,
                generation: self.generation,
            })
        }

        pub fn reset_for_cluster(&mut self, cluster_id: &str) {
            self.generation += 1;
            self.cluster_id = Some(cluster_id.to_string());
            self.clear_to_idle();
        }

        pub fn reset_for_disconnected(&mut self) {
            self.generation += 1;
            self.cluster_id = None;
            self.clear_to_idle();
        }

        fn clear_to_idle(&mut self) {
            self.status = $status::Idle;
            self.$field.clear();
            self.error_message = None;
            self.table.selected_row_key = None;
            self.table.loading = false;
            self.table.refreshing = false;
        }

        pub fn apply_loaded(
            &mut self,
            cluster_id: &str,
            generation: u64,
            items: Vec<$item>,
        ) -> bool {
            if self.cluster_id.as_deref() == Some(cluster_id) && self.generation == generation {
                self.status = $status::Loaded;
                self.$field = items;
                self.error_message = None;
                self.table.loading = false;
                self.table.refreshing = false;
                true
            } else {
                false
            }
        }

        pub fn apply_failed(
            &mut self,
            cluster_id: &str,
            generation: u64,
            message: String,
        ) -> bool {
            if self.cluster_id.as_deref() == Some(cluster_id) && self.generation == generation {
                self.status = $status::Error;
                self.error_message = Some(message);
                self.table.loading = false;
                self.table.refreshing = false;
                true
            } else {
                false
            }
        }

        pub fn needs_load_for_cluster(&self, cluster_id: &str) -> bool {
            self.cluster_id.as_deref() != Some(cluster_id) || self.status == $status::Idle
        }
    };
    ($state:ident, $status:ident, $field:ident, $item:ty, $cmd:ident) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
        pub enum $status {
            #[default]
            Idle,
            Loading,
            Loaded,
            Error,
        }

        #[derive(Debug, Clone, Default)]
        pub struct $state {
            pub cluster_id: Option<String>,
            pub generation: u64,
            pub status: $status,
            pub $field: Vec<$item>,
            pub error_message: Option<String>,
            pub table: DataTableState,
        }

        impl $state {
            cluster_list_state!(@methods $state, $status, $field, $item, $cmd);
        }
    };
    (paginated: $table_id:literal, $state:ident, $status:ident, $field:ident, $item:ty, $cmd:ident) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
        pub enum $status {
            #[default]
            Idle,
            Loading,
            Loaded,
            Error,
        }

        #[derive(Debug, Clone)]
        pub struct $state {
            pub cluster_id: Option<String>,
            pub generation: u64,
            pub status: $status,
            pub $field: Vec<$item>,
            pub error_message: Option<String>,
            pub table: DataTableState,
        }

        impl Default for $state {
            fn default() -> Self {
                Self::new()
            }
        }

        impl $state {
            pub fn new() -> Self {
                Self {
                    cluster_id: None,
                    generation: 0,
                    status: $status::Idle,
                    $field: Vec::new(),
                    error_message: None,
                    table: DataTableState::new().with_pagination(),
                }
            }

            pub fn hydrate_table_preferences(
                &mut self,
                cluster_id: Option<&str>,
                prefs: &orbit_core::config::PersistedTablePreferences,
                columns: &[crate::components::TableColumn],
            ) {
                self.table
                    .hydrate_from_preferences($table_id, cluster_id, prefs, columns);
            }

            cluster_list_state!(@methods $state, $status, $field, $item, $cmd);
        }
    };
}

cluster_list_state!(
    paginated: "deployments",
    DeploymentsState,
    DeploymentsLoadStatus,
    deployments,
    DeploymentInfo,
    LoadDeployments
);
cluster_list_state!(
    StatefulSetsState,
    StatefulSetsLoadStatus,
    statefulsets,
    StatefulSetInfo,
    LoadStatefulSets
);
cluster_list_state!(
    ReplicaSetsState,
    ReplicaSetsLoadStatus,
    replicasets,
    ReplicaSetInfo,
    LoadReplicaSets
);
cluster_list_state!(
    DaemonSetsState,
    DaemonSetsLoadStatus,
    daemonsets,
    DaemonSetInfo,
    LoadDaemonSets
);
cluster_list_state!(JobsState, JobsLoadStatus, jobs, JobInfo, LoadJobs);
cluster_list_state!(
    CronJobsState,
    CronJobsLoadStatus,
    cronjobs,
    CronJobInfo,
    LoadCronJobs
);
cluster_list_state!(
    paginated: "pods",
    PodsState,
    PodsLoadStatus,
    pods,
    PodInfo,
    LoadPods
);

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

#[derive(Debug, Clone)]
pub struct ServicesState {
    pub cluster_id: Option<String>,
    pub generation: u64,
    pub status: ServicesLoadStatus,
    pub services: Vec<ServiceInfo>,
    pub type_filter: ServiceTypeFilter,
    pub error_message: Option<String>,
    pub table: DataTableState,
}

impl Default for ServicesState {
    fn default() -> Self {
        Self::new()
    }
}

impl ServicesState {
    pub fn new() -> Self {
        Self {
            cluster_id: None,
            generation: 0,
            status: ServicesLoadStatus::Idle,
            services: Vec::new(),
            type_filter: ServiceTypeFilter::All,
            error_message: None,
            table: DataTableState::new().with_pagination(),
        }
    }

    pub fn hydrate_table_preferences(
        &mut self,
        cluster_id: Option<&str>,
        prefs: &orbit_core::config::PersistedTablePreferences,
        columns: &[crate::components::TableColumn],
    ) {
        self.table
            .hydrate_from_preferences("services", cluster_id, prefs, columns);
    }
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
        self.table.refreshing = !self.services.is_empty();
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
        self.table.refreshing = !self.services.is_empty();
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
        self.table.refreshing = false;
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
        self.table.refreshing = false;
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
            self.table.refreshing = false;
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
            self.table.refreshing = false;
            true
        } else {
            false
        }
    }

    pub fn needs_load_for_cluster(&self, cluster_id: &str) -> bool {
        self.cluster_id.as_deref() != Some(cluster_id) || self.status == ServicesLoadStatus::Idle
    }
}

cluster_list_state!(
    IngressesState,
    IngressesLoadStatus,
    ingresses,
    IngressInfo,
    LoadIngresses
);
cluster_list_state!(
    ConfigMapsState,
    ConfigMapsLoadStatus,
    configmaps,
    ConfigMapInfo,
    LoadConfigMaps
);
cluster_list_state!(
    SecretsState,
    SecretsLoadStatus,
    secrets,
    SecretInfo,
    LoadSecrets
);
cluster_list_state!(
    PersistentVolumesState,
    PersistentVolumesLoadStatus,
    persistent_volumes,
    PersistentVolumeInfo,
    LoadPersistentVolumes
);
cluster_list_state!(
    PersistentVolumeClaimsState,
    PersistentVolumeClaimsLoadStatus,
    persistent_volume_claims,
    PersistentVolumeClaimInfo,
    LoadPersistentVolumeClaims
);
cluster_list_state!(
    NetworkPoliciesState,
    NetworkPoliciesLoadStatus,
    network_policies,
    PolicyInfo,
    LoadNetworkPolicies
);
cluster_list_state!(
    PodSecurityState,
    PodSecurityLoadStatus,
    policies,
    PolicyInfo,
    LoadPodSecurity
);
cluster_list_state!(HpasState, HpasLoadStatus, hpas, HpaInfo, LoadHpas);
cluster_list_state!(
    StorageClassesState,
    StorageClassesLoadStatus,
    classes,
    StorageClassInfo,
    LoadStorageClasses
);
cluster_list_state!(
    SecurityOverviewState,
    SecurityOverviewLoadStatus,
    policies,
    PolicyInfo,
    LoadSecurityOverview
);
cluster_list_state!(
    AdmissionWebhooksState,
    AdmissionWebhooksLoadStatus,
    policies,
    PolicyInfo,
    LoadAdmissionWebhooks
);

/// Read-only list views that share the same cluster-scoped load lifecycle
/// (HPAs, StorageClasses, Security Overview, Admission Webhooks).
#[derive(Debug, Clone, Default)]
pub struct ClusterListViews {
    pub hpas: HpasState,
    pub storage_classes: StorageClassesState,
    pub security_overview: SecurityOverviewState,
    pub admission_webhooks: AdmissionWebhooksState,
}

impl ClusterListViews {
    /// Returns the load command when `view` is one of these views and its data is stale.
    pub fn load_command(
        &mut self,
        view: WorkspaceView,
        cluster_id: &str,
    ) -> Option<ClusterBridgeCommand> {
        match view {
            WorkspaceView::Hpas if self.hpas.needs_load_for_cluster(cluster_id) => {
                Some(self.hpas.begin_load_for_cluster(cluster_id))
            }
            WorkspaceView::StorageClasses
                if self.storage_classes.needs_load_for_cluster(cluster_id) =>
            {
                Some(self.storage_classes.begin_load_for_cluster(cluster_id))
            }
            WorkspaceView::SecurityOverview
                if self.security_overview.needs_load_for_cluster(cluster_id) =>
            {
                Some(self.security_overview.begin_load_for_cluster(cluster_id))
            }
            WorkspaceView::AdmissionWebhooks
                if self.admission_webhooks.needs_load_for_cluster(cluster_id) =>
            {
                Some(self.admission_webhooks.begin_load_for_cluster(cluster_id))
            }
            _ => None,
        }
    }

    /// Loads the active view and resets the others when a cluster connects.
    pub fn on_cluster_connected(
        &mut self,
        active_view: WorkspaceView,
        cluster_id: &str,
        commands: &mut Vec<ClusterBridgeCommand>,
    ) {
        macro_rules! connect {
            ($state:expr, $view:expr) => {
                if active_view == $view {
                    commands.push($state.begin_load_for_cluster(cluster_id));
                } else {
                    $state.reset_for_cluster(cluster_id);
                }
            };
        }
        connect!(self.hpas, WorkspaceView::Hpas);
        connect!(self.storage_classes, WorkspaceView::StorageClasses);
        connect!(self.security_overview, WorkspaceView::SecurityOverview);
        connect!(self.admission_webhooks, WorkspaceView::AdmissionWebhooks);
    }

    pub fn reset_for_disconnected(&mut self) {
        self.hpas.reset_for_disconnected();
        self.storage_classes.reset_for_disconnected();
        self.security_overview.reset_for_disconnected();
        self.admission_webhooks.reset_for_disconnected();
    }

    /// Refresh commands for views that are active or already loaded for this cluster.
    pub fn refresh_commands(
        &mut self,
        active_view: WorkspaceView,
        cluster_id: &str,
    ) -> Vec<ClusterBridgeCommand> {
        let mut commands = Vec::new();
        macro_rules! refresh {
            ($state:expr, $view:expr) => {
                if active_view == $view || $state.cluster_id.is_some() {
                    commands.push(
                        $state
                            .begin_refresh()
                            .unwrap_or_else(|| $state.begin_load_for_cluster(cluster_id)),
                    );
                }
            };
        }
        refresh!(self.hpas, WorkspaceView::Hpas);
        refresh!(self.storage_classes, WorkspaceView::StorageClasses);
        refresh!(self.security_overview, WorkspaceView::SecurityOverview);
        refresh!(self.admission_webhooks, WorkspaceView::AdmissionWebhooks);
        commands
    }

    /// Applies a bridge event. Returns `true` when the event belonged to one of these views.
    pub fn apply_event(&mut self, event: &ClusterBridgeEvent) -> bool {
        match event {
            ClusterBridgeEvent::HpasLoaded {
                cluster_id,
                generation,
                hpas,
            } => {
                self.hpas
                    .apply_loaded(cluster_id, *generation, hpas.clone());
            }
            ClusterBridgeEvent::HpasLoadFailed {
                cluster_id,
                generation,
                message,
            } => {
                self.hpas
                    .apply_failed(cluster_id, *generation, message.clone());
            }
            ClusterBridgeEvent::StorageClassesLoaded {
                cluster_id,
                generation,
                classes,
            } => {
                self.storage_classes
                    .apply_loaded(cluster_id, *generation, classes.clone());
            }
            ClusterBridgeEvent::StorageClassesLoadFailed {
                cluster_id,
                generation,
                message,
            } => {
                self.storage_classes
                    .apply_failed(cluster_id, *generation, message.clone());
            }
            ClusterBridgeEvent::SecurityOverviewLoaded {
                cluster_id,
                generation,
                policies,
            } => {
                self.security_overview
                    .apply_loaded(cluster_id, *generation, policies.clone());
            }
            ClusterBridgeEvent::SecurityOverviewLoadFailed {
                cluster_id,
                generation,
                message,
            } => {
                self.security_overview
                    .apply_failed(cluster_id, *generation, message.clone());
            }
            ClusterBridgeEvent::AdmissionWebhooksLoaded {
                cluster_id,
                generation,
                policies,
            } => {
                self.admission_webhooks
                    .apply_loaded(cluster_id, *generation, policies.clone());
            }
            ClusterBridgeEvent::AdmissionWebhooksLoadFailed {
                cluster_id,
                generation,
                message,
            } => {
                self.admission_webhooks
                    .apply_failed(cluster_id, *generation, message.clone());
            }
            _ => return false,
        }
        true
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ResourceQuotasLoadStatus {
    #[default]
    Idle,
    Loading,
    Loaded,
    Error,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ResourceQuotaKindFilter {
    #[default]
    All,
    ResourceQuota,
    LimitRange,
}

impl ResourceQuotaKindFilter {
    pub const ALL_OPTIONS: [Self; 3] = [Self::All, Self::ResourceQuota, Self::LimitRange];

    pub fn label(self) -> &'static str {
        match self {
            Self::All => "All",
            Self::ResourceQuota => "ResourceQuota",
            Self::LimitRange => "LimitRange",
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct ResourceQuotasState {
    pub cluster_id: Option<String>,
    pub generation: u64,
    pub status: ResourceQuotasLoadStatus,
    pub policies: Vec<PolicyInfo>,
    pub kind_filter: ResourceQuotaKindFilter,
    pub error_message: Option<String>,
    pub table: DataTableState,
}

impl ResourceQuotasState {
    pub fn set_kind_filter(&mut self, filter: ResourceQuotaKindFilter) {
        self.kind_filter = filter;
    }

    pub fn begin_load_for_cluster(&mut self, cluster_id: &str) -> ClusterBridgeCommand {
        let switching_cluster = self.cluster_id.as_deref() != Some(cluster_id);
        self.generation += 1;
        self.cluster_id = Some(cluster_id.to_string());
        self.status = ResourceQuotasLoadStatus::Loading;
        self.error_message = None;
        if switching_cluster {
            self.policies.clear();
            self.kind_filter = ResourceQuotaKindFilter::All;
            self.table.selected_row_key = None;
        }
        self.table.loading = self.policies.is_empty();
        self.table.refreshing = !self.policies.is_empty();
        ClusterBridgeCommand::LoadResourceQuotas {
            cluster_id: cluster_id.to_string(),
            generation: self.generation,
        }
    }

    pub fn begin_refresh(&mut self) -> Option<ClusterBridgeCommand> {
        let cluster_id = self.cluster_id.clone()?;
        self.generation += 1;
        self.status = ResourceQuotasLoadStatus::Loading;
        self.error_message = None;
        self.table.loading = self.policies.is_empty();
        self.table.refreshing = !self.policies.is_empty();
        Some(ClusterBridgeCommand::LoadResourceQuotas {
            cluster_id,
            generation: self.generation,
        })
    }

    pub fn reset_for_cluster(&mut self, cluster_id: &str) {
        self.generation += 1;
        self.cluster_id = Some(cluster_id.to_string());
        self.status = ResourceQuotasLoadStatus::Idle;
        self.policies.clear();
        self.kind_filter = ResourceQuotaKindFilter::All;
        self.error_message = None;
        self.table.selected_row_key = None;
        self.table.loading = false;
        self.table.refreshing = false;
    }

    pub fn reset_for_disconnected(&mut self) {
        self.generation += 1;
        self.cluster_id = None;
        self.status = ResourceQuotasLoadStatus::Idle;
        self.policies.clear();
        self.kind_filter = ResourceQuotaKindFilter::All;
        self.error_message = None;
        self.table.selected_row_key = None;
        self.table.loading = false;
        self.table.refreshing = false;
    }

    pub fn apply_loaded(
        &mut self,
        cluster_id: &str,
        generation: u64,
        policies: Vec<PolicyInfo>,
    ) -> bool {
        if self.cluster_id.as_deref() == Some(cluster_id) && self.generation == generation {
            self.status = ResourceQuotasLoadStatus::Loaded;
            self.policies = policies;
            self.error_message = None;
            self.table.loading = false;
            self.table.refreshing = false;
            true
        } else {
            false
        }
    }

    pub fn apply_failed(&mut self, cluster_id: &str, generation: u64, message: String) -> bool {
        if self.cluster_id.as_deref() == Some(cluster_id) && self.generation == generation {
            self.status = ResourceQuotasLoadStatus::Error;
            self.error_message = Some(message);
            self.table.loading = false;
            self.table.refreshing = false;
            true
        } else {
            false
        }
    }

    pub fn needs_load_for_cluster(&self, cluster_id: &str) -> bool {
        self.cluster_id.as_deref() != Some(cluster_id)
            || self.status == ResourceQuotasLoadStatus::Idle
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum StorageOverviewLoadStatus {
    #[default]
    Idle,
    Loading,
    Loaded,
    Error,
}

#[derive(Debug, Clone, Default)]
pub struct StorageOverviewState {
    pub cluster_id: Option<String>,
    pub generation: u64,
    pub status: StorageOverviewLoadStatus,
    pub snapshot: Option<StorageOverviewSnapshot>,
    pub error_message: Option<String>,
}

impl StorageOverviewState {
    pub fn begin_load_for_cluster(&mut self, cluster_id: &str) -> ClusterBridgeCommand {
        let switching_cluster = self.cluster_id.as_deref() != Some(cluster_id);
        self.generation += 1;
        self.cluster_id = Some(cluster_id.to_string());
        self.status = StorageOverviewLoadStatus::Loading;
        self.error_message = None;
        if switching_cluster {
            self.snapshot = None;
        }
        ClusterBridgeCommand::LoadStorageOverview {
            cluster_id: cluster_id.to_string(),
            generation: self.generation,
        }
    }

    pub fn begin_refresh(&mut self) -> Option<ClusterBridgeCommand> {
        let cluster_id = self.cluster_id.clone()?;
        self.generation += 1;
        self.status = StorageOverviewLoadStatus::Loading;
        self.error_message = None;
        Some(ClusterBridgeCommand::LoadStorageOverview {
            cluster_id,
            generation: self.generation,
        })
    }

    pub fn reset_for_cluster(&mut self, cluster_id: &str) {
        self.generation += 1;
        self.cluster_id = Some(cluster_id.to_string());
        self.status = StorageOverviewLoadStatus::Idle;
        self.snapshot = None;
        self.error_message = None;
    }

    pub fn reset_for_disconnected(&mut self) {
        self.generation += 1;
        self.cluster_id = None;
        self.status = StorageOverviewLoadStatus::Idle;
        self.snapshot = None;
        self.error_message = None;
    }

    pub fn apply_loaded(
        &mut self,
        cluster_id: &str,
        generation: u64,
        snapshot: StorageOverviewSnapshot,
    ) -> bool {
        if self.cluster_id.as_deref() == Some(cluster_id) && self.generation == generation {
            self.status = StorageOverviewLoadStatus::Loaded;
            self.snapshot = Some(snapshot);
            self.error_message = None;
            true
        } else {
            false
        }
    }

    pub fn apply_failed(&mut self, cluster_id: &str, generation: u64, message: String) -> bool {
        if self.cluster_id.as_deref() == Some(cluster_id) && self.generation == generation {
            self.status = StorageOverviewLoadStatus::Error;
            self.error_message = Some(message);
            true
        } else {
            false
        }
    }

    pub fn needs_load_for_cluster(&self, cluster_id: &str) -> bool {
        self.cluster_id.as_deref() != Some(cluster_id)
            || self.status == StorageOverviewLoadStatus::Idle
    }
}

cluster_list_state!(NodesState, NodesLoadStatus, nodes, NodeInfo, LoadNodes);

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

#[derive(Debug, Clone)]
pub struct EventsState {
    pub cluster_id: Option<String>,
    pub generation: u64,
    pub status: EventsLoadStatus,
    pub events: Vec<EventInfo>,
    pub type_filter: EventTypeFilter,
    pub error_message: Option<String>,
    pub table: DataTableState,
}

impl Default for EventsState {
    fn default() -> Self {
        Self::new()
    }
}

impl EventsState {
    pub fn new() -> Self {
        Self {
            cluster_id: None,
            generation: 0,
            status: EventsLoadStatus::Idle,
            events: Vec::new(),
            type_filter: EventTypeFilter::All,
            error_message: None,
            table: DataTableState::new().with_pagination(),
        }
    }

    pub fn hydrate_table_preferences(
        &mut self,
        cluster_id: Option<&str>,
        prefs: &orbit_core::config::PersistedTablePreferences,
        columns: &[crate::components::TableColumn],
    ) {
        self.table
            .hydrate_from_preferences("events", cluster_id, prefs, columns);
    }
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
        self.table.refreshing = !self.events.is_empty();
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
        self.table.refreshing = !self.events.is_empty();
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
        self.table.refreshing = false;
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
        self.table.refreshing = false;
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
            self.table.refreshing = false;
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
            self.table.refreshing = false;
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PortForwardsLoadStatus {
    #[default]
    Idle,
    Loading,
    Loaded,
    Error,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PortForwardKindFilter {
    #[default]
    All,
    Service,
    Pod,
    Deployment,
    StatefulSet,
    ReplicaSet,
}

impl PortForwardKindFilter {
    pub fn all() -> &'static [Self] {
        &[
            Self::All,
            Self::Service,
            Self::Pod,
            Self::Deployment,
            Self::StatefulSet,
            Self::ReplicaSet,
        ]
    }

    pub fn label(&self) -> &'static str {
        match self {
            Self::All => "All Kinds",
            Self::Service => "Service",
            Self::Pod => "Pod",
            Self::Deployment => "Deployment",
            Self::StatefulSet => "StatefulSet",
            Self::ReplicaSet => "ReplicaSet",
        }
    }

    pub fn matches_kind(&self, kind: &str) -> bool {
        match self {
            Self::All => true,
            Self::Service => kind.eq_ignore_ascii_case("Service"),
            Self::Pod => kind.eq_ignore_ascii_case("Pod"),
            Self::Deployment => kind.eq_ignore_ascii_case("Deployment"),
            Self::StatefulSet => kind.eq_ignore_ascii_case("StatefulSet"),
            Self::ReplicaSet => kind.eq_ignore_ascii_case("ReplicaSet"),
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct PortForwardsState {
    pub cluster_id: Option<String>,
    pub generation: u64,
    pub status: PortForwardsLoadStatus,
    pub port_forwards: Vec<PersistedPortForward>,
    pub stopping_ids: HashSet<String>,
    pub selected_ids: HashSet<String>,
    pub kind_filter: PortForwardKindFilter,
    pub search_query: String,
    pub search_focused: bool,
    pub table: DataTableState,
    pub error_message: Option<String>,
}

impl PortForwardsState {
    pub fn needs_load_for_cluster(&self, cluster_id: &str) -> bool {
        self.cluster_id.as_deref() != Some(cluster_id)
            || self.status == PortForwardsLoadStatus::Idle
    }

    pub fn begin_load_for_cluster(&mut self, cluster_id: &str) -> ClusterBridgeCommand {
        let switching_cluster = self.cluster_id.as_deref() != Some(cluster_id);
        self.generation += 1;
        self.cluster_id = Some(cluster_id.to_string());
        self.status = PortForwardsLoadStatus::Loading;
        self.error_message = None;
        if switching_cluster {
            self.port_forwards.clear();
            self.stopping_ids.clear();
            self.selected_ids.clear();
            self.kind_filter = PortForwardKindFilter::All;
            self.search_query.clear();
            self.search_focused = false;
            self.table.selected_row_key = None;
        }
        self.table.loading = self.port_forwards.is_empty();
        self.table.refreshing = !self.port_forwards.is_empty();
        ClusterBridgeCommand::LoadPortForwards {
            cluster_id: cluster_id.to_string(),
            generation: self.generation,
        }
    }

    pub fn begin_refresh(&mut self) -> Option<ClusterBridgeCommand> {
        let cluster_id = self.cluster_id.clone()?;
        self.generation += 1;
        self.status = PortForwardsLoadStatus::Loading;
        self.error_message = None;
        self.table.loading = self.port_forwards.is_empty();
        self.table.refreshing = !self.port_forwards.is_empty();
        Some(ClusterBridgeCommand::LoadPortForwards {
            cluster_id,
            generation: self.generation,
        })
    }

    pub fn reset_for_cluster(&mut self, cluster_id: &str) {
        self.generation += 1;
        self.cluster_id = Some(cluster_id.to_string());
        self.status = PortForwardsLoadStatus::Idle;
        self.port_forwards.clear();
        self.stopping_ids.clear();
        self.selected_ids.clear();
        self.kind_filter = PortForwardKindFilter::All;
        self.search_query.clear();
        self.search_focused = false;
        self.error_message = None;
        self.table.selected_row_key = None;
        self.table.loading = false;
        self.table.refreshing = false;
    }

    pub fn reset_for_disconnected(&mut self) {
        self.generation += 1;
        self.cluster_id = None;
        self.status = PortForwardsLoadStatus::Idle;
        self.port_forwards.clear();
        self.stopping_ids.clear();
        self.selected_ids.clear();
        self.kind_filter = PortForwardKindFilter::All;
        self.search_query.clear();
        self.search_focused = false;
        self.error_message = None;
        self.table.selected_row_key = None;
        self.table.loading = false;
        self.table.refreshing = false;
    }

    pub fn apply_loaded(
        &mut self,
        cluster_id: &str,
        generation: u64,
        port_forwards: Vec<PersistedPortForward>,
    ) -> bool {
        if self.cluster_id.as_deref() == Some(cluster_id) && self.generation == generation {
            self.status = PortForwardsLoadStatus::Loaded;
            self.port_forwards = port_forwards;
            let active_ids: HashSet<&str> =
                self.port_forwards.iter().map(|pf| pf.id.as_str()).collect();
            self.stopping_ids
                .retain(|id| active_ids.contains(id.as_str()));
            self.selected_ids
                .retain(|id| active_ids.contains(id.as_str()));
            self.error_message = None;
            self.table.loading = false;
            self.table.refreshing = false;
            true
        } else {
            false
        }
    }

    pub fn apply_started(&mut self, cluster_id: &str, port_forward: PersistedPortForward) -> bool {
        if self.cluster_id.as_deref() != Some(cluster_id) {
            return false;
        }
        self.stopping_ids.remove(&port_forward.id);
        self.port_forwards.retain(|pf| pf.id != port_forward.id);
        self.port_forwards.push(port_forward);
        self.status = PortForwardsLoadStatus::Loaded;
        self.error_message = None;
        self.table.loading = false;
        self.table.refreshing = false;
        true
    }

    pub fn apply_stopped(&mut self, cluster_id: &str, id: &str) -> bool {
        if self.cluster_id.as_deref() != Some(cluster_id) {
            return false;
        }
        self.stopping_ids.remove(id);
        self.selected_ids.remove(id);
        self.port_forwards.retain(|pf| pf.id != id);
        if self.table.selected_row_key.as_deref() == Some(id) {
            self.table.selected_row_key = None;
        }
        true
    }

    pub fn apply_cleared(&mut self, cluster_id: &str) -> bool {
        if self.cluster_id.as_deref() != Some(cluster_id) {
            return false;
        }
        self.stopping_ids.clear();
        self.selected_ids.clear();
        self.port_forwards.clear();
        self.table.selected_row_key = None;
        self.status = PortForwardsLoadStatus::Loaded;
        self.error_message = None;
        self.table.loading = false;
        self.table.refreshing = false;
        true
    }

    pub fn apply_failed(&mut self, cluster_id: &str, id: Option<&str>, message: String) -> bool {
        if self.cluster_id.as_deref() != Some(cluster_id) {
            return false;
        }
        if let Some(forward_id) = id {
            self.stopping_ids.remove(forward_id);
        }
        self.error_message = Some(message);
        self.table.loading = false;
        self.table.refreshing = false;
        true
    }

    pub fn mark_stopping(&mut self, id: &str) {
        self.stopping_ids.insert(id.to_string());
    }

    pub fn toggle_select(&mut self, id: &str) {
        if !self.selected_ids.remove(id) {
            self.selected_ids.insert(id.to_string());
        }
    }

    pub fn toggle_select_all_visible(&mut self, visible_ids: &[String]) {
        if visible_ids.is_empty() {
            return;
        }
        let all_selected = visible_ids.iter().all(|id| self.selected_ids.contains(id));
        if all_selected {
            for id in visible_ids {
                self.selected_ids.remove(id);
            }
        } else {
            for id in visible_ids {
                self.selected_ids.insert(id.clone());
            }
        }
    }

    pub fn clear_selection(&mut self) {
        self.selected_ids.clear();
    }

    pub fn set_kind_filter(&mut self, filter: PortForwardKindFilter) {
        self.kind_filter = filter;
    }

    pub fn handle_search_key(&mut self, key: &str, key_char: Option<&str>) -> bool {
        if !self.search_focused {
            return false;
        }
        if key == "escape" {
            if self.search_query.is_empty() {
                self.search_focused = false;
            } else {
                self.search_query.clear();
            }
            return true;
        }
        if key == "backspace" {
            self.search_query.pop();
            return true;
        }
        if let Some(ch_str) = key_char {
            let mut chars = ch_str.chars();
            if let (Some(ch), None) = (chars.next(), chars.next())
                && !ch.is_control()
            {
                self.search_query.push(ch);
                return true;
            }
        }
        false
    }

    pub fn has_local_port(&self, local_port: u16) -> bool {
        self.port_forwards
            .iter()
            .any(|pf| pf.local_port == local_port)
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

    pub fn begin_disconnect(&mut self) -> Option<ClusterBridgeCommand> {
        if self.connection_state == ClusterConnectionState::Disconnected
            && self.active_cluster_id.is_none()
        {
            return None;
        }

        self.dropdown_open = false;
        self.active_cluster_id = None;
        self.active_profile = None;
        self.connection_state = ClusterConnectionState::Disconnected;
        self.error_banner_dismissed = false;
        Some(ClusterBridgeCommand::DisconnectCluster)
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
            | ClusterBridgeEvent::ReplicaSetsLoaded { .. }
            | ClusterBridgeEvent::ReplicaSetsLoadFailed { .. }
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
            | ClusterBridgeEvent::PersistentVolumesLoaded { .. }
            | ClusterBridgeEvent::PersistentVolumesLoadFailed { .. }
            | ClusterBridgeEvent::PersistentVolumeClaimsLoaded { .. }
            | ClusterBridgeEvent::PersistentVolumeClaimsLoadFailed { .. }
            | ClusterBridgeEvent::StorageOverviewLoaded { .. }
            | ClusterBridgeEvent::StorageOverviewLoadFailed { .. }
            | ClusterBridgeEvent::NetworkPoliciesLoaded { .. }
            | ClusterBridgeEvent::NetworkPoliciesLoadFailed { .. }
            | ClusterBridgeEvent::PodSecurityLoaded { .. }
            | ClusterBridgeEvent::PodSecurityLoadFailed { .. }
            | ClusterBridgeEvent::HpasLoaded { .. }
            | ClusterBridgeEvent::HpasLoadFailed { .. }
            | ClusterBridgeEvent::StorageClassesLoaded { .. }
            | ClusterBridgeEvent::StorageClassesLoadFailed { .. }
            | ClusterBridgeEvent::SecurityOverviewLoaded { .. }
            | ClusterBridgeEvent::SecurityOverviewLoadFailed { .. }
            | ClusterBridgeEvent::AdmissionWebhooksLoaded { .. }
            | ClusterBridgeEvent::AdmissionWebhooksLoadFailed { .. }
            | ClusterBridgeEvent::ResourceQuotasLoaded { .. }
            | ClusterBridgeEvent::ResourceQuotasLoadFailed { .. }
            | ClusterBridgeEvent::NodesLoaded { .. }
            | ClusterBridgeEvent::NodesLoadFailed { .. }
            | ClusterBridgeEvent::EventsLoaded { .. }
            | ClusterBridgeEvent::EventsLoadFailed { .. }
            | ClusterBridgeEvent::ResourceYamlLoaded { .. }
            | ClusterBridgeEvent::ResourceYamlLoadFailed { .. }
            | ClusterBridgeEvent::WorkloadActionSucceeded { .. }
            | ClusterBridgeEvent::WorkloadActionFailed { .. }
            | ClusterBridgeEvent::LogTargetsResolved { .. }
            | ClusterBridgeEvent::LogLinesReceived { .. }
            | ClusterBridgeEvent::LogStreamStatus { .. }
            | ClusterBridgeEvent::LogStreamEnded { .. } => {}
            ClusterBridgeEvent::ClusterAdded {
                clusters,
                switch_outcome,
                profile,
                ..
            } => {
                self.clusters = clusters;
                if let Some(outcome) = switch_outcome {
                    self.active_cluster_id = Some(outcome.context_name.clone());
                    let display_name = self.cluster_display_name(&outcome.context_name);
                    if let Some(p) = profile.clone() {
                        self.active_profile = Some(p);
                    }
                    if outcome.healthy {
                        self.connection_state = ClusterConnectionState::Connected {
                            cluster_id: outcome.context_name,
                            display_name,
                            k8s_version: outcome.k8s_version,
                        };
                    } else {
                        let server_url = profile.and_then(|p| p.server_url).or_else(|| {
                            self.clusters
                                .iter()
                                .find(|c| c.id == outcome.context_name)
                                .and_then(|c| c.server_url.clone())
                        });
                        self.connection_state = ClusterConnectionState::Unreachable {
                            cluster_id: outcome.context_name,
                            display_name,
                            server_url,
                            error_message: outcome
                                .error_message
                                .unwrap_or_else(|| "Cluster API server is unreachable".to_string()),
                        };
                    }
                    self.error_banner_dismissed = false;
                }
            }
            ClusterBridgeEvent::ClusterDisconnected { clusters, .. } => {
                self.clusters = clusters;
                self.active_cluster_id = None;
                self.active_profile = None;
                self.connection_state = ClusterConnectionState::Disconnected;
                self.dropdown_open = false;
            }
            ClusterBridgeEvent::AddClusterFailed { .. }
            | ClusterBridgeEvent::SettingsSaveFailed { .. }
            | ClusterBridgeEvent::PortForwardsLoaded { .. }
            | ClusterBridgeEvent::PortForwardStarted { .. }
            | ClusterBridgeEvent::PortForwardStopped { .. }
            | ClusterBridgeEvent::PortForwardsCleared { .. }
            | ClusterBridgeEvent::PortForwardFailed { .. } => {}
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

pub fn resolve_tab_drop_target(
    from_index: usize,
    hover_index: usize,
    position: TabDropPosition,
    tab_count: usize,
) -> Option<(usize, TabDropPosition)> {
    if from_index >= tab_count || hover_index >= tab_count {
        return None;
    }
    let to_index = calculate_tab_drop_index(from_index, hover_index, position);
    if to_index == from_index || to_index >= tab_count {
        return None;
    }
    let (canonical_hover, canonical_pos) = match position {
        TabDropPosition::After if hover_index + 1 < tab_count => {
            (hover_index + 1, TabDropPosition::Before)
        }
        _ => (hover_index, position),
    };
    Some((canonical_hover, canonical_pos))
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ClusterTabsState {
    pub open_tabs: Vec<WorkspaceTab>,
    pub active_tab: Option<WorkspaceTab>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PickerKeyAction {
    Ignored,
    Updated,
    Closed,
    Selected(WorkspaceView),
}

#[derive(Debug, Clone)]
pub struct WorkspaceTabsState {
    by_cluster: HashMap<String, ClusterTabsState>,
    active_cluster_id: Option<String>,
    pub scroll_handle: ScrollHandle,
    pub pending_scroll_to_active: bool,
    pub picker_open: bool,
    pub picker_query: String,
    pub picker_selected_index: usize,
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
            picker_selected_index: 0,
            drop_target: None,
            dragging_index: None,
        }
    }

    pub fn from_config(config: &OrbitConfig) -> Self {
        let mut state = Self::new();
        for (cluster_id, persisted) in config.workspace_tabs_map() {
            let mut seen = HashSet::new();
            let open_tabs: Vec<WorkspaceTab> = persisted
                .open_tabs
                .iter()
                .filter_map(|id| WorkspaceView::from_id(id))
                .filter(|view| seen.insert(*view))
                .map(WorkspaceTab::View)
                .collect();

            let active_tab = if open_tabs.is_empty() {
                None
            } else {
                persisted
                    .active_tab
                    .as_deref()
                    .and_then(WorkspaceView::from_id)
                    .map(WorkspaceTab::View)
                    .filter(|tab| open_tabs.contains(tab))
                    .or_else(|| open_tabs.first().cloned())
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

    pub fn open_tabs(&self) -> &[WorkspaceTab] {
        self.active_cluster_id
            .as_deref()
            .and_then(|id| self.by_cluster.get(id))
            .map(|c| c.open_tabs.as_slice())
            .unwrap_or(&[])
    }

    pub fn open_views(&self) -> Vec<WorkspaceView> {
        self.open_tabs()
            .iter()
            .filter_map(WorkspaceTab::as_view)
            .collect()
    }

    pub fn active_tab(&self) -> Option<WorkspaceTab> {
        self.active_cluster_id
            .as_deref()
            .and_then(|id| self.by_cluster.get(id))
            .and_then(|c| c.active_tab.clone())
    }

    pub fn active_view(&self) -> Option<WorkspaceView> {
        self.active_tab().and_then(|t| t.as_view())
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
                open_tabs: vec![WorkspaceTab::View(WorkspaceView::Overview)],
                active_tab: Some(WorkspaceTab::View(WorkspaceView::Overview)),
            });
        self.pending_scroll_to_active = entry.active_tab.is_some();
        entry.active_tab.as_ref().and_then(WorkspaceTab::as_view)
    }

    pub fn clear_active_cluster(&mut self) {
        self.active_cluster_id = None;
        self.close_picker();
        self.drop_target = None;
        self.dragging_index = None;
    }

    pub fn strip_logs_tabs(&mut self) {
        for cluster in self.by_cluster.values_mut() {
            cluster
                .open_tabs
                .retain(|t| !matches!(t, WorkspaceTab::Logs(_)));
            if matches!(cluster.active_tab, Some(WorkspaceTab::Logs(_))) {
                cluster.active_tab = cluster.open_tabs.first().cloned();
            }
        }
    }

    fn current_cluster_mut(&mut self) -> Option<&mut ClusterTabsState> {
        let id = self.active_cluster_id.as_ref()?;
        Some(self.by_cluster.entry(id.clone()).or_default())
    }

    pub fn open_tab(&mut self, tab: impl Into<WorkspaceTab>) {
        let tab = tab.into();
        let Some(cluster) = self.current_cluster_mut() else {
            return;
        };
        if !cluster.open_tabs.contains(&tab) {
            cluster.open_tabs.push(tab.clone());
        }
        cluster.active_tab = Some(tab);
        self.pending_scroll_to_active = true;
    }

    pub fn open_logs_tab(&mut self, target: LogsTabTarget) -> bool {
        let tab = WorkspaceTab::Logs(target);
        let Some(cluster) = self.current_cluster_mut() else {
            return false;
        };
        let is_new = !cluster.open_tabs.contains(&tab);
        if is_new {
            cluster.open_tabs.push(tab.clone());
        }
        cluster.active_tab = Some(tab);
        self.pending_scroll_to_active = true;
        is_new
    }

    pub fn set_active_tab(&mut self, tab: impl Into<WorkspaceTab>) {
        self.open_tab(tab);
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
            .as_ref()
            .and_then(|active| cluster.open_tabs.iter().position(|t| t == active))
        {
            Some(idx) => (idx + 1) % cluster.open_tabs.len(),
            None => 0,
        };
        cluster.active_tab = Some(cluster.open_tabs[next_idx].clone());
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
            .as_ref()
            .and_then(|active| cluster.open_tabs.iter().position(|t| t == active))
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
        cluster.active_tab = Some(cluster.open_tabs[prev_idx].clone());
        self.pending_scroll_to_active = true;
    }

    pub fn close_tab(&mut self, tab: impl Into<WorkspaceTab>) -> Vec<LogsTabTarget> {
        let tab = tab.into();
        let Some(cluster) = self.current_cluster_mut() else {
            return Vec::new();
        };
        let Some(idx) = cluster.open_tabs.iter().position(|t| *t == tab) else {
            return Vec::new();
        };
        let removed = cluster.open_tabs.remove(idx);
        if cluster.active_tab.as_ref() == Some(&removed) {
            if cluster.open_tabs.is_empty() {
                cluster.active_tab = None;
            } else if idx < cluster.open_tabs.len() {
                cluster.active_tab = Some(cluster.open_tabs[idx].clone());
            } else {
                cluster.active_tab = Some(cluster.open_tabs[idx - 1].clone());
            }
            self.pending_scroll_to_active = cluster.active_tab.is_some();
        }
        match removed {
            WorkspaceTab::Logs(target) => vec![target],
            WorkspaceTab::View(_) => Vec::new(),
        }
    }

    pub fn close_active_tab(&mut self) -> Vec<LogsTabTarget> {
        if let Some(active) = self.active_tab() {
            self.close_tab(active)
        } else {
            Vec::new()
        }
    }

    pub fn close_other_tabs(&mut self, tab: impl Into<WorkspaceTab>) -> Vec<LogsTabTarget> {
        let tab = tab.into();
        let Some(cluster) = self.current_cluster_mut() else {
            return Vec::new();
        };
        if !cluster.open_tabs.contains(&tab) {
            return Vec::new();
        }
        let closed: Vec<LogsTabTarget> = cluster
            .open_tabs
            .iter()
            .filter(|t| **t != tab)
            .filter_map(|t| t.as_logs().cloned())
            .collect();
        cluster.open_tabs = vec![tab.clone()];
        cluster.active_tab = Some(tab);
        self.pending_scroll_to_active = true;
        closed
    }

    pub fn close_tabs_to_right(&mut self, tab: impl Into<WorkspaceTab>) -> Vec<LogsTabTarget> {
        let tab = tab.into();
        let Some(cluster) = self.current_cluster_mut() else {
            return Vec::new();
        };
        let Some(idx) = cluster.open_tabs.iter().position(|t| *t == tab) else {
            return Vec::new();
        };
        let closed: Vec<LogsTabTarget> = cluster
            .open_tabs
            .drain((idx + 1)..)
            .filter_map(|t| match t {
                WorkspaceTab::Logs(target) => Some(target),
                WorkspaceTab::View(_) => None,
            })
            .collect();
        if let Some(active) = &cluster.active_tab
            && !cluster.open_tabs.contains(active)
        {
            cluster.active_tab = Some(tab);
            self.pending_scroll_to_active = true;
        }
        closed
    }

    pub fn close_all_tabs(&mut self) -> Vec<LogsTabTarget> {
        let Some(cluster) = self.current_cluster_mut() else {
            return Vec::new();
        };
        let closed: Vec<LogsTabTarget> = cluster
            .open_tabs
            .drain(..)
            .filter_map(|t| match t {
                WorkspaceTab::Logs(target) => Some(target),
                WorkspaceTab::View(_) => None,
            })
            .collect();
        cluster.active_tab = None;
        closed
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

    pub fn update_drop_target(&mut self, hover_index: usize, position: TabDropPosition) -> bool {
        let tab_count = self.open_tabs().len();
        let next = self
            .dragging_index
            .and_then(|from| resolve_tab_drop_target(from, hover_index, position, tab_count));
        if self.drop_target != next {
            self.drop_target = next;
            true
        } else {
            false
        }
    }

    pub fn clear_drop_target(&mut self) -> bool {
        self.drop_target.take().is_some()
    }

    pub fn clear_drag_state(&mut self) -> bool {
        let had_drag = self.dragging_index.take().is_some();
        let had_drop = self.drop_target.take().is_some();
        had_drag || had_drop
    }

    pub fn apply_drop(
        &mut self,
        from_index: usize,
        fallback_hover_index: usize,
        fallback_position: TabDropPosition,
    ) -> bool {
        let (hover_idx, pos) = self
            .drop_target
            .unwrap_or((fallback_hover_index, fallback_position));
        self.clear_drag_state();
        let tab_count = self.open_tabs().len();
        let Some((resolved_hover, resolved_pos)) =
            resolve_tab_drop_target(from_index, hover_idx, pos, tab_count)
        else {
            return false;
        };
        let to_index = calculate_tab_drop_index(from_index, resolved_hover, resolved_pos);
        self.reorder_tab(from_index, to_index);
        true
    }

    pub fn move_tab_left(&mut self, tab: impl Into<WorkspaceTab>) {
        let tab = tab.into();
        let Some(cluster) = self.current_cluster_mut() else {
            return;
        };
        if let Some(idx) = cluster.open_tabs.iter().position(|t| *t == tab)
            && idx > 0
        {
            cluster.open_tabs.swap(idx, idx - 1);
        }
    }

    pub fn move_tab_right(&mut self, tab: impl Into<WorkspaceTab>) {
        let tab = tab.into();
        let Some(cluster) = self.current_cluster_mut() else {
            return;
        };
        if let Some(idx) = cluster.open_tabs.iter().position(|t| *t == tab)
            && idx + 1 < cluster.open_tabs.len()
        {
            cluster.open_tabs.swap(idx, idx + 1);
        }
    }

    pub fn open_picker(&mut self) {
        self.picker_open = true;
        self.picker_query.clear();
        self.picker_selected_index = 0;
    }

    pub fn close_picker(&mut self) {
        self.picker_open = false;
        self.picker_query.clear();
        self.picker_selected_index = 0;
    }

    pub fn clear_picker_query(&mut self) {
        self.picker_query.clear();
        self.picker_selected_index = 0;
    }

    pub fn toggle_picker(&mut self) {
        if self.picker_open {
            self.close_picker();
        } else {
            self.open_picker();
        }
    }

    pub fn handle_picker_key(
        &mut self,
        key: &str,
        key_char: Option<&str>,
        has_shortcut_modifier: bool,
    ) -> PickerKeyAction {
        if !self.picker_open {
            return PickerKeyAction::Ignored;
        }
        let normalized_key = key.to_ascii_lowercase();
        if normalized_key == "escape" {
            self.close_picker();
            return PickerKeyAction::Closed;
        }
        if has_shortcut_modifier {
            return PickerKeyAction::Ignored;
        }
        match normalized_key.as_str() {
            "backspace" => {
                self.picker_query.pop();
                self.picker_selected_index = 0;
                PickerKeyAction::Updated
            }
            "down" => {
                let len = self.filtered_picker_views().len();
                if len > 0 {
                    self.picker_selected_index = (self.picker_selected_index + 1) % len;
                }
                PickerKeyAction::Updated
            }
            "up" => {
                let len = self.filtered_picker_views().len();
                if len > 0 {
                    self.picker_selected_index = if self.picker_selected_index == 0 {
                        len - 1
                    } else {
                        self.picker_selected_index - 1
                    };
                }
                PickerKeyAction::Updated
            }
            "enter" | "return" => {
                let filtered = self.filtered_picker_views();
                let selected = filtered
                    .get(self.picker_selected_index)
                    .copied()
                    .or_else(|| filtered.first().copied());
                if let Some(view) = selected {
                    self.close_picker();
                    PickerKeyAction::Selected(view)
                } else {
                    PickerKeyAction::Updated
                }
            }
            "space" => {
                self.picker_query.push(' ');
                self.picker_selected_index = 0;
                PickerKeyAction::Updated
            }
            _ => {
                if let Some(ch) = key_char {
                    if !ch.is_empty() && !ch.chars().any(char::is_control) {
                        self.picker_query.push_str(ch);
                        self.picker_selected_index = 0;
                        return PickerKeyAction::Updated;
                    }
                }
                if key.chars().count() == 1 && !key.chars().any(char::is_control) {
                    self.picker_query.push_str(key);
                    self.picker_selected_index = 0;
                    return PickerKeyAction::Updated;
                }
                PickerKeyAction::Ignored
            }
        }
    }

    pub fn handle_picker_search_key(&mut self, key: &str, key_char: Option<&str>) -> bool {
        !matches!(
            self.handle_picker_key(key, key_char, false),
            PickerKeyAction::Ignored
        )
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
                .filter_map(|t| t.as_view().map(|v| v.id().to_string()))
                .collect(),
            active_tab: cluster
                .active_tab
                .as_ref()
                .and_then(WorkspaceTab::as_view)
                .map(|v| v.id().to_string()),
        })
    }
}

/// Aggregates all cluster-scoped Kubernetes resource view states owned by `OrbitShell`.
#[derive(Debug, Clone, Default)]
pub struct WorkspaceResourcesState {
    pub overview: ClusterOverviewState,
    pub workloads_overview: WorkloadsOverviewState,
    pub namespaces: NamespaceFilterState,
    pub namespaces_view: NamespacesViewState,
    pub deployments: DeploymentsState,
    pub statefulsets: StatefulSetsState,
    pub daemonsets: DaemonSetsState,
    pub replicasets: ReplicaSetsState,
    pub jobs: JobsState,
    pub cronjobs: CronJobsState,
    pub pods: PodsState,
    pub services: ServicesState,
    pub ingresses: IngressesState,
    pub port_forwards: PortForwardsState,
    pub configmaps: ConfigMapsState,
    pub secrets: SecretsState,
    pub persistent_volumes: PersistentVolumesState,
    pub persistent_volume_claims: PersistentVolumeClaimsState,
    pub network_policies: NetworkPoliciesState,
    pub storage_overview: StorageOverviewState,
    pub pod_security: PodSecurityState,
    pub resource_quotas: ResourceQuotasState,
    pub list_views: ClusterListViews,
    pub nodes: NodesState,
    pub events: EventsState,
}

impl WorkspaceResourcesState {
    pub fn from_config(config: &OrbitConfig) -> Self {
        Self {
            namespaces: NamespaceFilterState::from_config(config),
            ..Self::default()
        }
    }

    pub fn reset_all_for_disconnect(&mut self) {
        self.overview.reset_for_disconnected();
        self.workloads_overview.reset_for_disconnected();
        self.namespaces.reset_for_disconnected();
        self.namespaces_view.reset_for_disconnected();
        self.deployments.reset_for_disconnected();
        self.statefulsets.reset_for_disconnected();
        self.daemonsets.reset_for_disconnected();
        self.replicasets.reset_for_disconnected();
        self.jobs.reset_for_disconnected();
        self.cronjobs.reset_for_disconnected();
        self.pods.reset_for_disconnected();
        self.services.reset_for_disconnected();
        self.ingresses.reset_for_disconnected();
        self.port_forwards.reset_for_disconnected();
        self.configmaps.reset_for_disconnected();
        self.secrets.reset_for_disconnected();
        self.persistent_volumes.reset_for_disconnected();
        self.persistent_volume_claims.reset_for_disconnected();
        self.network_policies.reset_for_disconnected();
        self.storage_overview.reset_for_disconnected();
        self.pod_security.reset_for_disconnected();
        self.resource_quotas.reset_for_disconnected();
        self.nodes.reset_for_disconnected();
        self.events.reset_for_disconnected();
        self.list_views.reset_for_disconnected();
    }

    pub fn load_command_for_active_view(
        &mut self,
        state: &ClusterSwitcherState,
        active_view: Option<WorkspaceView>,
    ) -> Option<ClusterBridgeCommand> {
        let view = active_view?;
        let ClusterConnectionState::Connected { cluster_id, .. } = &state.connection_state else {
            return None;
        };
        match view {
            WorkspaceView::WorkloadsOverview
                if self.workloads_overview.needs_load_for_cluster(cluster_id) =>
            {
                Some(self.workloads_overview.begin_load_for_cluster(cluster_id))
            }
            WorkspaceView::Namespaces if self.namespaces.needs_load_for_cluster(cluster_id) => {
                Some(self.namespaces.begin_load_for_cluster(cluster_id))
            }
            WorkspaceView::Deployments if self.deployments.needs_load_for_cluster(cluster_id) => {
                Some(self.deployments.begin_load_for_cluster(cluster_id))
            }
            WorkspaceView::StatefulSets if self.statefulsets.needs_load_for_cluster(cluster_id) => {
                Some(self.statefulsets.begin_load_for_cluster(cluster_id))
            }
            WorkspaceView::DaemonSets if self.daemonsets.needs_load_for_cluster(cluster_id) => {
                Some(self.daemonsets.begin_load_for_cluster(cluster_id))
            }
            WorkspaceView::ReplicaSets if self.replicasets.needs_load_for_cluster(cluster_id) => {
                Some(self.replicasets.begin_load_for_cluster(cluster_id))
            }
            WorkspaceView::Jobs if self.jobs.needs_load_for_cluster(cluster_id) => {
                Some(self.jobs.begin_load_for_cluster(cluster_id))
            }
            WorkspaceView::CronJobs if self.cronjobs.needs_load_for_cluster(cluster_id) => {
                Some(self.cronjobs.begin_load_for_cluster(cluster_id))
            }
            WorkspaceView::Pods if self.pods.needs_load_for_cluster(cluster_id) => {
                Some(self.pods.begin_load_for_cluster(cluster_id))
            }
            WorkspaceView::Services if self.services.needs_load_for_cluster(cluster_id) => {
                Some(self.services.begin_load_for_cluster(cluster_id))
            }
            WorkspaceView::Ingresses if self.ingresses.needs_load_for_cluster(cluster_id) => {
                Some(self.ingresses.begin_load_for_cluster(cluster_id))
            }
            WorkspaceView::ConfigMaps if self.configmaps.needs_load_for_cluster(cluster_id) => {
                Some(self.configmaps.begin_load_for_cluster(cluster_id))
            }
            WorkspaceView::Secrets if self.secrets.needs_load_for_cluster(cluster_id) => {
                Some(self.secrets.begin_load_for_cluster(cluster_id))
            }
            WorkspaceView::PersistentVolumes
                if self.persistent_volumes.needs_load_for_cluster(cluster_id) =>
            {
                Some(self.persistent_volumes.begin_load_for_cluster(cluster_id))
            }
            WorkspaceView::PersistentVolumeClaims
                if self
                    .persistent_volume_claims
                    .needs_load_for_cluster(cluster_id) =>
            {
                Some(
                    self.persistent_volume_claims
                        .begin_load_for_cluster(cluster_id),
                )
            }
            WorkspaceView::NetworkPolicies
                if self.network_policies.needs_load_for_cluster(cluster_id) =>
            {
                Some(self.network_policies.begin_load_for_cluster(cluster_id))
            }
            WorkspaceView::StorageOverview
                if self.storage_overview.needs_load_for_cluster(cluster_id) =>
            {
                Some(self.storage_overview.begin_load_for_cluster(cluster_id))
            }
            WorkspaceView::PodSecurity if self.pod_security.needs_load_for_cluster(cluster_id) => {
                Some(self.pod_security.begin_load_for_cluster(cluster_id))
            }
            WorkspaceView::ResourceQuotas
                if self.resource_quotas.needs_load_for_cluster(cluster_id) =>
            {
                Some(self.resource_quotas.begin_load_for_cluster(cluster_id))
            }
            WorkspaceView::Nodes if self.nodes.needs_load_for_cluster(cluster_id) => {
                Some(self.nodes.begin_load_for_cluster(cluster_id))
            }
            WorkspaceView::Events if self.events.needs_load_for_cluster(cluster_id) => {
                Some(self.events.begin_load_for_cluster(cluster_id))
            }
            _ => self.list_views.load_command(view, cluster_id),
        }
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
    fn test_cluster_switcher_begin_disconnect_clears_active_state_and_is_idempotent() {
        let mut state = ClusterSwitcherState::new();
        assert_eq!(state.begin_disconnect(), None);

        let clusters = vec![sample_cluster(
            "ctx-1",
            Some("Prod"),
            Some("https://10.0.0.1"),
        )];
        let profile = sample_profile("ctx-1", Some("https://10.0.0.1"));
        state.apply_event(ClusterBridgeEvent::ClusterConnected {
            outcome: ClusterSwitchOutcome {
                context_name: "ctx-1".to_string(),
                healthy: true,
                k8s_version: Some("v1.31.2".to_string()),
                error_message: None,
            },
            clusters: clusters.clone(),
            profile: profile.clone(),
        });
        state.dropdown_open = true;
        state.error_banner_dismissed = true;

        assert_eq!(
            state.begin_disconnect(),
            Some(ClusterBridgeCommand::DisconnectCluster)
        );
        assert!(state.active_cluster_id.is_none());
        assert!(state.active_profile.is_none());
        assert!(!state.dropdown_open);
        assert!(!state.error_banner_dismissed);
        assert_eq!(state.connection_state, ClusterConnectionState::Disconnected);
        assert_eq!(state.begin_disconnect(), None);

        // Connecting, Unreachable, and Error states with a cluster_id can also disconnect
        state.connection_state = ClusterConnectionState::Connecting {
            cluster_id: "ctx-1".to_string(),
            display_name: "Prod".to_string(),
        };
        assert_eq!(
            state.begin_disconnect(),
            Some(ClusterBridgeCommand::DisconnectCluster)
        );

        state.apply_event(ClusterBridgeEvent::ClusterUnreachable {
            outcome: ClusterSwitchOutcome {
                context_name: "ctx-1".to_string(),
                healthy: false,
                k8s_version: None,
                error_message: Some("timeout".to_string()),
            },
            clusters: clusters.clone(),
            profile,
        });
        assert_eq!(
            state.begin_disconnect(),
            Some(ClusterBridgeCommand::DisconnectCluster)
        );

        state.apply_event(ClusterBridgeEvent::ConnectionError {
            cluster_id: Some("ctx-1".to_string()),
            message: "bad cert".to_string(),
            clusters,
        });
        assert_eq!(
            state.begin_disconnect(),
            Some(ClusterBridgeCommand::DisconnectCluster)
        );
    }

    #[test]
    fn test_cluster_switcher_applies_cluster_disconnected_and_allows_reconnect() {
        let mut state = ClusterSwitcherState::new();
        let clusters = vec![sample_cluster(
            "ctx-1",
            Some("Prod"),
            Some("https://10.0.0.1"),
        )];
        let profile = sample_profile("ctx-1", Some("https://10.0.0.1"));

        state.apply_event(ClusterBridgeEvent::ClusterConnected {
            outcome: ClusterSwitchOutcome {
                context_name: "ctx-1".to_string(),
                healthy: true,
                k8s_version: Some("v1.31.2".to_string()),
                error_message: None,
            },
            clusters: clusters.clone(),
            profile,
        });
        state.dropdown_open = true;

        state.apply_event(ClusterBridgeEvent::ClusterDisconnected {
            previous_cluster_id: Some("ctx-1".to_string()),
            clusters: clusters.clone(),
        });

        assert!(state.active_cluster_id.is_none());
        assert!(state.active_profile.is_none());
        assert!(!state.dropdown_open);
        assert_eq!(state.connection_state, ClusterConnectionState::Disconnected);
        assert_eq!(state.clusters.len(), 1);

        let cmd = state.begin_switch("ctx-1");
        assert_eq!(
            cmd,
            Some(ClusterBridgeCommand::SwitchCluster {
                cluster_id: "ctx-1".to_string()
            })
        );
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

    fn sample_replicaset(name: &str, namespace: &str) -> ReplicaSetInfo {
        ReplicaSetInfo {
            kind: "ReplicaSet".to_string(),
            name: name.to_string(),
            namespace: namespace.to_string(),
            status: "Running".to_string(),
            replicas: orbit_core::kubernetes::models::Replicas {
                current: 3,
                desired: 3,
            },
            age: "2d".to_string(),
            images: vec!["nginx:1.25".to_string()],
            containers: vec![orbit_core::kubernetes::models::ContainerImageInfo {
                name: "nginx".to_string(),
                image: "nginx:1.25".to_string(),
            }],
            labels: Default::default(),
            annotations: Default::default(),
        }
    }

    #[test]
    fn test_replicasets_state_lifecycle_and_generation_guard() {
        let mut state = ReplicaSetsState::default();
        assert!(state.needs_load_for_cluster("ctx-a"));

        let cmd = state.begin_load_for_cluster("ctx-a");
        assert_eq!(
            cmd,
            ClusterBridgeCommand::LoadReplicaSets {
                cluster_id: "ctx-a".to_string(),
                generation: 1,
            }
        );
        assert_eq!(state.status, ReplicaSetsLoadStatus::Loading);
        assert!(state.table.loading);
        assert!(!state.needs_load_for_cluster("ctx-a"));

        // Stale cluster or generation is rejected.
        assert!(!state.apply_loaded(
            "ctx-other",
            1,
            vec![sample_replicaset("wrong-cluster", "default")],
        ));
        assert!(!state.apply_loaded("ctx-a", 99, vec![sample_replicaset("wrong-gen", "default")],));
        assert!(state.replicasets.is_empty());

        // Matching cluster and generation is accepted.
        assert!(state.apply_loaded("ctx-a", 1, vec![sample_replicaset("web-rs", "default")]));
        assert_eq!(state.status, ReplicaSetsLoadStatus::Loaded);
        assert!(!state.table.loading);
        assert_eq!(state.replicasets.len(), 1);
        assert_eq!(state.replicasets[0].name, "web-rs");

        // Refresh retains cached rows while loading.
        let refresh = state.begin_refresh();
        assert_eq!(
            refresh,
            Some(ClusterBridgeCommand::LoadReplicaSets {
                cluster_id: "ctx-a".to_string(),
                generation: 2,
            })
        );
        assert_eq!(state.status, ReplicaSetsLoadStatus::Loading);
        assert_eq!(state.replicasets.len(), 1);

        assert!(state.apply_failed("ctx-a", 2, "timeout".to_string()));
        assert_eq!(state.status, ReplicaSetsLoadStatus::Error);
        assert_eq!(state.error_message.as_deref(), Some("timeout"));

        // Reset for new cluster clears rows, selection, and returns to Idle.
        state.reset_for_cluster("ctx-b");
        assert_eq!(state.cluster_id.as_deref(), Some("ctx-b"));
        assert_eq!(state.status, ReplicaSetsLoadStatus::Idle);
        assert!(state.replicasets.is_empty());
        assert!(state.error_message.is_none());
        assert!(state.table.selected_row_key.is_none());
        assert!(state.needs_load_for_cluster("ctx-b"));

        // Disconnect clears cluster_id and invalidates in-flight replies.
        let gen_before_disconnect = state.generation;
        state.reset_for_disconnected();
        assert!(state.cluster_id.is_none());
        assert_eq!(state.status, ReplicaSetsLoadStatus::Idle);
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
            name: String::new(),
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
        assert_eq!(tabs.open_views(), vec![WorkspaceView::Overview]);
        assert_eq!(tabs.active_view(), Some(WorkspaceView::Overview));

        tabs.pending_scroll_to_active = false;
        tabs.open_tab(WorkspaceView::Pods);
        tabs.open_tab(WorkspaceView::Deployments);
        assert_eq!(
            tabs.open_views(),
            vec![
                WorkspaceView::Overview,
                WorkspaceView::Pods,
                WorkspaceView::Deployments,
            ]
        );
        assert_eq!(tabs.active_view(), Some(WorkspaceView::Deployments));
        assert!(tabs.pending_scroll_to_active);

        // Deduplicate existing view: activates without duplicating
        tabs.pending_scroll_to_active = false;
        tabs.open_tab(WorkspaceView::Pods);
        assert_eq!(
            tabs.open_views(),
            vec![
                WorkspaceView::Overview,
                WorkspaceView::Pods,
                WorkspaceView::Deployments,
            ]
        );
        assert_eq!(tabs.active_view(), Some(WorkspaceView::Pods));
        assert!(tabs.pending_scroll_to_active);

        // Cycle next / prev with wrap-around
        tabs.pending_scroll_to_active = false;
        tabs.activate_next_tab();
        assert_eq!(tabs.active_view(), Some(WorkspaceView::Deployments));
        assert!(tabs.pending_scroll_to_active);

        tabs.activate_next_tab();
        assert_eq!(tabs.active_view(), Some(WorkspaceView::Overview));

        tabs.pending_scroll_to_active = false;
        tabs.activate_prev_tab();
        assert_eq!(tabs.active_view(), Some(WorkspaceView::Deployments));
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
            tabs.open_views(),
            vec![
                WorkspaceView::Pods,
                WorkspaceView::Deployments,
                WorkspaceView::Nodes,
            ]
        );
        assert_eq!(tabs.active_view(), Some(WorkspaceView::Nodes));

        // Closing rightmost active tab activates left neighbor
        tabs.close_active_tab();
        assert_eq!(
            tabs.open_views(),
            vec![WorkspaceView::Pods, WorkspaceView::Deployments]
        );
        assert_eq!(tabs.active_view(), Some(WorkspaceView::Deployments));

        // Closing middle active tab activates right neighbor
        tabs.open_tab(WorkspaceView::Nodes);
        tabs.set_active_tab(WorkspaceView::Deployments);
        tabs.close_tab(WorkspaceView::Deployments);
        assert_eq!(
            tabs.open_views(),
            vec![WorkspaceView::Pods, WorkspaceView::Nodes]
        );
        assert_eq!(tabs.active_view(), Some(WorkspaceView::Nodes));

        // Closing remaining tabs leads to empty state and hides bar
        assert!(tabs.is_bar_visible(true));
        tabs.close_active_tab();
        assert_eq!(tabs.open_views(), vec![WorkspaceView::Pods]);
        assert_eq!(tabs.active_view(), Some(WorkspaceView::Pods));
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
            tabs.open_views(),
            vec![
                WorkspaceView::Overview,
                WorkspaceView::Deployments,
                WorkspaceView::Pods,
                WorkspaceView::Nodes,
            ]
        );

        tabs.move_tab_right(WorkspaceView::Deployments);
        assert_eq!(
            tabs.open_views(),
            vec![
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
            tabs.open_views(),
            vec![
                WorkspaceView::Pods,
                WorkspaceView::Deployments,
                WorkspaceView::Overview,
                WorkspaceView::Nodes,
            ]
        );

        tabs.close_tabs_to_right(WorkspaceView::Deployments);
        assert_eq!(
            tabs.open_views(),
            vec![WorkspaceView::Pods, WorkspaceView::Deployments]
        );

        tabs.open_tab(WorkspaceView::Services);
        tabs.close_other_tabs(WorkspaceView::Deployments);
        assert_eq!(tabs.open_views(), vec![WorkspaceView::Deployments]);
        assert_eq!(tabs.active_view(), Some(WorkspaceView::Deployments));

        tabs.close_all_tabs();
        assert!(tabs.open_tabs().is_empty());
        assert_eq!(tabs.active_tab(), None);
    }

    #[test]
    fn test_resolve_tab_drop_target_suppresses_noop_and_out_of_bounds_positions() {
        // Self drops
        assert_eq!(
            resolve_tab_drop_target(1, 1, TabDropPosition::Before, 4),
            None
        );
        assert_eq!(
            resolve_tab_drop_target(1, 1, TabDropPosition::After, 4),
            None
        );
        // Adjacent no-ops
        assert_eq!(
            resolve_tab_drop_target(1, 2, TabDropPosition::Before, 4),
            None
        );
        assert_eq!(
            resolve_tab_drop_target(1, 0, TabDropPosition::After, 4),
            None
        );
        // Valid moves
        assert_eq!(
            resolve_tab_drop_target(0, 2, TabDropPosition::Before, 4),
            Some((2, TabDropPosition::Before))
        );
        assert_eq!(calculate_tab_drop_index(0, 2, TabDropPosition::Before), 1);

        assert_eq!(
            resolve_tab_drop_target(0, 3, TabDropPosition::After, 4),
            Some((3, TabDropPosition::After))
        );
        assert_eq!(calculate_tab_drop_index(0, 3, TabDropPosition::After), 3);

        assert_eq!(
            resolve_tab_drop_target(3, 0, TabDropPosition::Before, 4),
            Some((0, TabDropPosition::Before))
        );
        assert_eq!(calculate_tab_drop_index(3, 0, TabDropPosition::Before), 0);

        // Interior After canonicalizes to next tab's Before so a single seam never has 2 indicators
        assert_eq!(
            resolve_tab_drop_target(3, 0, TabDropPosition::After, 4),
            Some((1, TabDropPosition::Before))
        );
        assert_eq!(
            resolve_tab_drop_target(3, 1, TabDropPosition::Before, 4),
            Some((1, TabDropPosition::Before))
        );
        assert_eq!(
            resolve_tab_drop_target(0, 2, TabDropPosition::After, 4),
            Some((3, TabDropPosition::Before))
        );

        // Out of bounds
        assert_eq!(
            resolve_tab_drop_target(0, 4, TabDropPosition::Before, 4),
            None
        );
        assert_eq!(
            resolve_tab_drop_target(4, 0, TabDropPosition::Before, 4),
            None
        );
    }

    #[test]
    fn test_workspace_tabs_drag_drop_lifecycle_and_cancel_clears_indicator() {
        let mut tabs = WorkspaceTabsState::new();
        tabs.activate_cluster("cluster-a");
        tabs.open_tab(WorkspaceView::Pods);
        tabs.open_tab(WorkspaceView::Deployments);
        assert_eq!(
            tabs.open_views(),
            vec![
                WorkspaceView::Overview,
                WorkspaceView::Pods,
                WorkspaceView::Deployments,
            ]
        );

        tabs.dragging_index = Some(0);
        assert!(tabs.update_drop_target(2, TabDropPosition::Before));
        assert_eq!(tabs.drop_target, Some((2, TabDropPosition::Before)));

        assert!(tabs.clear_drop_target());
        assert_eq!(tabs.drop_target, None);
        assert_eq!(tabs.dragging_index, Some(0));

        assert!(tabs.update_drop_target(2, TabDropPosition::After));
        assert!(tabs.apply_drop(0, 2, TabDropPosition::Before));
        assert_eq!(
            tabs.open_views(),
            vec![
                WorkspaceView::Pods,
                WorkspaceView::Deployments,
                WorkspaceView::Overview,
            ]
        );
        assert_eq!(tabs.dragging_index, None);
        assert_eq!(tabs.drop_target, None);

        tabs.dragging_index = Some(1);
        tabs.update_drop_target(0, TabDropPosition::Before);
        assert!(tabs.clear_drag_state());
        assert_eq!(tabs.dragging_index, None);
        assert_eq!(tabs.drop_target, None);
        assert_eq!(
            tabs.open_views(),
            vec![
                WorkspaceView::Pods,
                WorkspaceView::Deployments,
                WorkspaceView::Overview,
            ]
        );
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
            tabs.open_views(),
            vec![WorkspaceView::Pods, WorkspaceView::Deployments]
        );

        // Explicitly emptied cluster stays empty
        assert_eq!(tabs.activate_cluster("cluster-empty"), None);
        assert!(tabs.open_tabs().is_empty());

        // Never-visited cluster initializes with [Overview]
        assert_eq!(
            tabs.activate_cluster("cluster-new"),
            Some(WorkspaceView::Overview)
        );
        assert_eq!(tabs.open_views(), vec![WorkspaceView::Overview]);
        tabs.open_tab(WorkspaceView::Nodes);

        // Switching back to cluster A preserves its independent list
        assert_eq!(
            tabs.activate_cluster("cluster-a"),
            Some(WorkspaceView::Deployments)
        );
        assert_eq!(
            tabs.open_views(),
            vec![WorkspaceView::Pods, WorkspaceView::Deployments]
        );

        // Switching to cluster-new preserves its Nodes tab
        assert_eq!(
            tabs.activate_cluster("cluster-new"),
            Some(WorkspaceView::Nodes)
        );
        assert_eq!(
            tabs.open_views(),
            vec![WorkspaceView::Overview, WorkspaceView::Nodes]
        );
    }

    #[test]
    fn test_open_logs_tab_deduplicates_by_kind_namespace_and_name() {
        let mut tabs = WorkspaceTabsState::new();
        tabs.activate_cluster("cluster-a");

        let api_default = LogsTabTarget {
            kind: "Deployment".to_string(),
            namespace: "default".to_string(),
            name: "api".to_string(),
        };
        let api_staging = LogsTabTarget {
            kind: "Deployment".to_string(),
            namespace: "staging".to_string(),
            name: "api".to_string(),
        };

        assert!(tabs.open_logs_tab(api_default.clone()));
        assert_eq!(tabs.open_tabs().len(), 2);
        assert_eq!(
            tabs.active_tab(),
            Some(WorkspaceTab::Logs(api_default.clone()))
        );

        // Switch away, then re-open same target: focuses without duplicating
        tabs.open_tab(WorkspaceView::Overview);
        assert!(!tabs.open_logs_tab(api_default.clone()));
        assert_eq!(tabs.open_tabs().len(), 2);
        assert_eq!(tabs.active_tab(), Some(WorkspaceTab::Logs(api_default)));

        // Different namespace opens a separate tab
        assert!(tabs.open_logs_tab(api_staging.clone()));
        assert_eq!(tabs.open_tabs().len(), 3);
        assert_eq!(tabs.active_tab(), Some(WorkspaceTab::Logs(api_staging)));
    }

    #[test]
    fn test_close_logs_tabs_returns_closed_targets_across_all_close_actions() {
        let mut tabs = WorkspaceTabsState::new();
        tabs.activate_cluster("cluster-a");

        let t1 = LogsTabTarget {
            kind: "Deployment".to_string(),
            namespace: "default".to_string(),
            name: "api-1".to_string(),
        };
        let t2 = LogsTabTarget {
            kind: "Pod".to_string(),
            namespace: "default".to_string(),
            name: "pod-2".to_string(),
        };
        let t3 = LogsTabTarget {
            kind: "Job".to_string(),
            namespace: "batch".to_string(),
            name: "job-3".to_string(),
        };

        tabs.open_logs_tab(t1.clone());
        tabs.open_logs_tab(t2.clone());
        tabs.open_logs_tab(t3.clone());
        // [Overview, t1, t2, t3]

        let closed_right = tabs.close_tabs_to_right(WorkspaceTab::Logs(t1.clone()));
        assert_eq!(closed_right, vec![t2.clone(), t3.clone()]);

        tabs.open_logs_tab(t2.clone());
        let closed_others = tabs.close_other_tabs(WorkspaceView::Overview);
        assert_eq!(closed_others, vec![t1.clone(), t2.clone()]);

        tabs.open_logs_tab(t1.clone());
        let closed_single = tabs.close_tab(WorkspaceTab::Logs(t1.clone()));
        assert_eq!(closed_single, vec![t1]);

        tabs.open_logs_tab(t2.clone());
        let closed_all = tabs.close_all_tabs();
        assert_eq!(closed_all, vec![t2]);
    }

    #[test]
    fn test_persist_workspace_tabs_excludes_ephemeral_logs_tabs() {
        let mut tabs = WorkspaceTabsState::new();
        tabs.activate_cluster("cluster-a");
        tabs.open_tab(WorkspaceView::Pods);
        tabs.open_logs_tab(LogsTabTarget {
            kind: "Deployment".to_string(),
            namespace: "default".to_string(),
            name: "api".to_string(),
        });

        let Some(ClusterBridgeCommand::SaveWorkspaceTabs {
            open_tabs,
            active_tab,
            ..
        }) = tabs.persistence_command()
        else {
            panic!("expected SaveWorkspaceTabs command");
        };

        assert_eq!(
            open_tabs,
            vec![
                WorkspaceView::Overview.id().to_string(),
                WorkspaceView::Pods.id().to_string(),
            ]
        );
        assert_eq!(active_tab, None);
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
    fn test_workspace_tabs_picker_handles_space_key_and_key_fallback_without_key_char() {
        let mut tabs = WorkspaceTabsState::new();
        tabs.open_picker();

        assert_eq!(
            tabs.handle_picker_key("p", None, false),
            PickerKeyAction::Updated
        );
        assert_eq!(
            tabs.handle_picker_key("o", None, false),
            PickerKeyAction::Updated
        );
        assert_eq!(
            tabs.handle_picker_key("d", None, false),
            PickerKeyAction::Updated
        );
        assert_eq!(
            tabs.handle_picker_key("space", None, false),
            PickerKeyAction::Updated
        );
        assert_eq!(tabs.picker_query, "pod ");

        // Shortcut-modified keys must be ignored and must not mutate picker_query
        assert_eq!(
            tabs.handle_picker_key("w", Some("w"), true),
            PickerKeyAction::Ignored
        );
        assert_eq!(tabs.picker_query, "pod ");
    }

    #[test]
    fn test_workspace_tabs_picker_arrow_navigation_and_enter_selects_filtered_view() {
        let mut tabs = WorkspaceTabsState::new();
        tabs.open_picker();

        for ch in ["d", "e", "p", "l"] {
            assert_eq!(
                tabs.handle_picker_key(ch, Some(ch), false),
                PickerKeyAction::Updated
            );
        }
        assert_eq!(
            tabs.handle_picker_key("enter", None, false),
            PickerKeyAction::Selected(WorkspaceView::Deployments)
        );
        assert!(!tabs.picker_open);
        assert!(tabs.picker_query.is_empty());
        assert_eq!(tabs.picker_selected_index, 0);

        // Multi-match navigation wrapping with Down / Up
        tabs.open_picker();
        for ch in ["n", "e", "t", "w", "o", "r", "k"] {
            assert_eq!(
                tabs.handle_picker_key(ch, Some(ch), false),
                PickerKeyAction::Updated
            );
        }
        let filtered = tabs.filtered_picker_views();
        assert!(filtered.len() >= 2);
        assert_eq!(tabs.picker_selected_index, 0);

        // Up from 0 wraps to last item
        assert_eq!(
            tabs.handle_picker_key("up", None, false),
            PickerKeyAction::Updated
        );
        assert_eq!(tabs.picker_selected_index, filtered.len() - 1);

        // Down from last wraps back to 0
        assert_eq!(
            tabs.handle_picker_key("down", None, false),
            PickerKeyAction::Updated
        );
        assert_eq!(tabs.picker_selected_index, 0);

        // Down to index 1 and Enter selects filtered[1]
        assert_eq!(
            tabs.handle_picker_key("down", None, false),
            PickerKeyAction::Updated
        );
        assert_eq!(tabs.picker_selected_index, 1);
        assert_eq!(
            tabs.handle_picker_key("enter", None, false),
            PickerKeyAction::Selected(filtered[1])
        );
        assert!(!tabs.picker_open);
        assert_eq!(tabs.picker_selected_index, 0);

        // Zero matches + Enter keeps picker open
        tabs.open_picker();
        for ch in ["z", "z", "z", "z"] {
            tabs.handle_picker_key(ch, Some(ch), false);
        }
        assert!(tabs.filtered_picker_views().is_empty());
        assert_eq!(
            tabs.handle_picker_key("enter", None, false),
            PickerKeyAction::Updated
        );
        assert!(tabs.picker_open);
    }

    #[test]
    fn test_workspace_tabs_picker_clear_query_resets_text_and_selection() {
        let mut tabs = WorkspaceTabsState::new();
        tabs.open_picker();
        tabs.handle_picker_key("n", Some("n"), false);
        tabs.handle_picker_key("down", None, false);
        assert_eq!(tabs.picker_query, "n");
        assert_eq!(tabs.picker_selected_index, 1);

        tabs.clear_picker_query();
        assert_eq!(tabs.picker_query, "");
        assert_eq!(tabs.picker_selected_index, 0);
        assert!(tabs.picker_open);
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

    fn sample_persistent_volume(name: &str) -> PersistentVolumeInfo {
        PersistentVolumeInfo {
            name: name.to_string(),
            capacity: "20Gi".to_string(),
            access_mode: "ReadWriteOnce".to_string(),
            reclaim_policy: "Retain".to_string(),
            status: "Bound".to_string(),
            storage_class: "gp3".to_string(),
            age: "4d".to_string(),
            volume_mode: "Filesystem".to_string(),
            reason: None,
        }
    }

    #[test]
    fn test_persistent_volumes_state_lifecycle_and_generation_guard() {
        let mut state = PersistentVolumesState::default();
        assert!(state.needs_load_for_cluster("ctx-a"));

        let cmd = state.begin_load_for_cluster("ctx-a");
        assert_eq!(
            cmd,
            ClusterBridgeCommand::LoadPersistentVolumes {
                cluster_id: "ctx-a".to_string(),
                generation: 1,
            }
        );
        assert_eq!(state.status, PersistentVolumesLoadStatus::Loading);
        assert!(state.table.loading);

        let pv = sample_persistent_volume("pv-data-01");

        // Stale cluster or generation must be rejected
        assert!(!state.apply_loaded("ctx-other", 1, vec![pv.clone()]));
        assert!(!state.apply_loaded("ctx-a", 99, vec![pv.clone()]));
        assert!(!state.apply_failed("ctx-other", 1, "boom".to_string()));
        assert!(!state.apply_failed("ctx-a", 99, "boom".to_string()));
        assert!(state.persistent_volumes.is_empty());

        // Matching cluster and generation loads rows
        assert!(state.apply_loaded("ctx-a", 1, vec![pv]));
        assert_eq!(state.status, PersistentVolumesLoadStatus::Loaded);
        assert_eq!(state.persistent_volumes.len(), 1);
        assert!(!state.table.loading);
        assert!(!state.needs_load_for_cluster("ctx-a"));

        // Refresh increments generation while retaining cached rows
        let refresh = state.begin_refresh();
        assert_eq!(
            refresh,
            Some(ClusterBridgeCommand::LoadPersistentVolumes {
                cluster_id: "ctx-a".to_string(),
                generation: 2,
            })
        );
        assert_eq!(state.status, PersistentVolumesLoadStatus::Loading);
        assert!(!state.table.loading);
        assert_eq!(state.persistent_volumes.len(), 1);

        // Failure on generation 2 transitions to Error while retaining cached rows
        assert!(state.apply_failed("ctx-a", 2, "timeout".to_string()));
        assert_eq!(state.status, PersistentVolumesLoadStatus::Error);
        assert_eq!(state.error_message.as_deref(), Some("timeout"));
        assert_eq!(state.persistent_volumes.len(), 1);

        // Reset for another cluster clears rows/selection/error and invalidates in-flight replies
        state.table.selected_row_key = Some("pv-data-01".to_string());
        state.reset_for_cluster("ctx-b");
        assert_eq!(state.status, PersistentVolumesLoadStatus::Idle);
        assert_eq!(state.cluster_id.as_deref(), Some("ctx-b"));
        assert!(state.persistent_volumes.is_empty());
        assert!(state.error_message.is_none());
        assert!(state.table.selected_row_key.is_none());
        assert!(!state.apply_loaded("ctx-a", 2, vec![sample_persistent_volume("stale")]));

        // Reset for disconnected clears cluster_id and rows
        state.reset_for_disconnected();
        assert_eq!(state.status, PersistentVolumesLoadStatus::Idle);
        assert!(state.cluster_id.is_none());
        assert!(state.persistent_volumes.is_empty());
        assert!(state.error_message.is_none());
        assert!(state.table.selected_row_key.is_none());
    }

    fn sample_persistent_volume_claim(name: &str, namespace: &str) -> PersistentVolumeClaimInfo {
        PersistentVolumeClaimInfo {
            name: name.to_string(),
            namespace: namespace.to_string(),
            status: "Bound".to_string(),
            volume: "pv-data-01".to_string(),
            capacity: "20Gi".to_string(),
            access_mode: "ReadWriteOnce".to_string(),
            storage_class: "gp3".to_string(),
            age: "4d".to_string(),
        }
    }

    #[test]
    fn test_persistent_volume_claims_state_lifecycle_and_generation_guard() {
        let mut state = PersistentVolumeClaimsState::default();
        assert!(state.needs_load_for_cluster("ctx-a"));

        let cmd = state.begin_load_for_cluster("ctx-a");
        assert_eq!(
            cmd,
            ClusterBridgeCommand::LoadPersistentVolumeClaims {
                cluster_id: "ctx-a".to_string(),
                generation: 1,
            }
        );
        assert_eq!(state.status, PersistentVolumeClaimsLoadStatus::Loading);
        assert!(state.table.loading);

        let pvc = sample_persistent_volume_claim("data-pg-0", "prod");

        // Stale cluster or generation must be rejected
        assert!(!state.apply_loaded("ctx-other", 1, vec![pvc.clone()]));
        assert!(!state.apply_loaded("ctx-a", 99, vec![pvc.clone()]));
        assert!(!state.apply_failed("ctx-other", 1, "boom".to_string()));
        assert!(!state.apply_failed("ctx-a", 99, "boom".to_string()));
        assert!(state.persistent_volume_claims.is_empty());

        // Matching cluster and generation loads rows
        assert!(state.apply_loaded("ctx-a", 1, vec![pvc]));
        assert_eq!(state.status, PersistentVolumeClaimsLoadStatus::Loaded);
        assert_eq!(state.persistent_volume_claims.len(), 1);
        assert!(!state.table.loading);
        assert!(!state.needs_load_for_cluster("ctx-a"));

        // Refresh increments generation while retaining cached rows
        let refresh = state.begin_refresh();
        assert_eq!(
            refresh,
            Some(ClusterBridgeCommand::LoadPersistentVolumeClaims {
                cluster_id: "ctx-a".to_string(),
                generation: 2,
            })
        );
        assert_eq!(state.status, PersistentVolumeClaimsLoadStatus::Loading);
        assert!(!state.table.loading);
        assert_eq!(state.persistent_volume_claims.len(), 1);

        // Failure on generation 2 transitions to Error while retaining cached rows
        assert!(state.apply_failed("ctx-a", 2, "timeout".to_string()));
        assert_eq!(state.status, PersistentVolumeClaimsLoadStatus::Error);
        assert_eq!(state.error_message.as_deref(), Some("timeout"));
        assert_eq!(state.persistent_volume_claims.len(), 1);

        // Reset for another cluster clears rows/selection/error and invalidates in-flight replies
        state.table.selected_row_key = Some("prod/data-pg-0".to_string());
        state.reset_for_cluster("ctx-b");
        assert_eq!(state.status, PersistentVolumeClaimsLoadStatus::Idle);
        assert_eq!(state.cluster_id.as_deref(), Some("ctx-b"));
        assert!(state.persistent_volume_claims.is_empty());
        assert!(state.error_message.is_none());
        assert!(state.table.selected_row_key.is_none());
        assert!(!state.apply_loaded(
            "ctx-a",
            2,
            vec![sample_persistent_volume_claim("stale", "prod")]
        ));

        // Reset for disconnected clears cluster_id and rows
        state.reset_for_disconnected();
        assert_eq!(state.status, PersistentVolumeClaimsLoadStatus::Idle);
        assert!(state.cluster_id.is_none());
        assert!(state.persistent_volume_claims.is_empty());
        assert!(state.error_message.is_none());
        assert!(state.table.selected_row_key.is_none());
    }

    fn sample_network_policy(name: &str, namespace: &str) -> PolicyInfo {
        PolicyInfo {
            uid: format!("uid-{name}"),
            name: name.to_string(),
            r#type: "Network Policy".to_string(),
            scope: "Namespaced".to_string(),
            namespace: namespace.to_string(),
            status: "Enforced".to_string(),
            mode: "Ingress, Egress".to_string(),
            violations: 0,
            last_updated: "2d".to_string(),
            description: "Pods matching: app=api".to_string(),
            rules: "policyTypes:\n- Ingress\n".to_string(),
            kind: "NetworkPolicy".to_string(),
        }
    }

    #[test]
    fn test_network_policies_state_lifecycle_and_generation_guard() {
        let mut state = NetworkPoliciesState::default();
        assert!(state.needs_load_for_cluster("ctx-a"));

        let cmd = state.begin_load_for_cluster("ctx-a");
        assert_eq!(
            cmd,
            ClusterBridgeCommand::LoadNetworkPolicies {
                cluster_id: "ctx-a".to_string(),
                generation: 1,
            }
        );
        assert_eq!(state.status, NetworkPoliciesLoadStatus::Loading);
        assert!(state.table.loading);

        let policy = sample_network_policy("allow-api", "prod");

        // Stale cluster or generation must be rejected
        assert!(!state.apply_loaded("ctx-other", 1, vec![policy.clone()]));
        assert!(!state.apply_loaded("ctx-a", 99, vec![policy.clone()]));
        assert!(!state.apply_failed("ctx-other", 1, "boom".to_string()));
        assert!(!state.apply_failed("ctx-a", 99, "boom".to_string()));
        assert!(state.network_policies.is_empty());

        // Matching cluster and generation loads rows
        assert!(state.apply_loaded("ctx-a", 1, vec![policy]));
        assert_eq!(state.status, NetworkPoliciesLoadStatus::Loaded);
        assert_eq!(state.network_policies.len(), 1);
        assert!(!state.table.loading);
        assert!(!state.needs_load_for_cluster("ctx-a"));

        // Refresh increments generation while retaining cached rows
        let refresh = state.begin_refresh();
        assert_eq!(
            refresh,
            Some(ClusterBridgeCommand::LoadNetworkPolicies {
                cluster_id: "ctx-a".to_string(),
                generation: 2,
            })
        );
        assert_eq!(state.status, NetworkPoliciesLoadStatus::Loading);
        assert!(!state.table.loading);
        assert_eq!(state.network_policies.len(), 1);

        // Failure on generation 2 transitions to Error while retaining cached rows
        assert!(state.apply_failed("ctx-a", 2, "timeout".to_string()));
        assert_eq!(state.status, NetworkPoliciesLoadStatus::Error);
        assert_eq!(state.error_message.as_deref(), Some("timeout"));
        assert_eq!(state.network_policies.len(), 1);

        // Reset for another cluster clears rows/selection/error and invalidates in-flight replies
        state.table.selected_row_key = Some("prod/allow-api".to_string());
        state.reset_for_cluster("ctx-b");
        assert_eq!(state.status, NetworkPoliciesLoadStatus::Idle);
        assert_eq!(state.cluster_id.as_deref(), Some("ctx-b"));
        assert!(state.network_policies.is_empty());
        assert!(state.error_message.is_none());
        assert!(state.table.selected_row_key.is_none());
        assert!(!state.apply_loaded("ctx-a", 2, vec![sample_network_policy("stale", "prod")]));

        // Reset for disconnected clears cluster_id and rows
        state.reset_for_disconnected();
        assert_eq!(state.status, NetworkPoliciesLoadStatus::Idle);
        assert!(state.cluster_id.is_none());
        assert!(state.network_policies.is_empty());
        assert!(state.error_message.is_none());
        assert!(state.table.selected_row_key.is_none());
    }

    fn sample_pod_security_policy(namespace: &str) -> PolicyInfo {
        PolicyInfo {
            uid: format!("uid-psa-{namespace}"),
            name: format!("psa-{namespace}"),
            r#type: "Pod Security".to_string(),
            scope: "Namespaced".to_string(),
            namespace: namespace.to_string(),
            status: "Enforced".to_string(),
            mode: "enforce: restricted".to_string(),
            violations: 0,
            last_updated: "5d".to_string(),
            description: "PSA: enforce=restricted".to_string(),
            rules: "pod-security.kubernetes.io/enforce: restricted\n".to_string(),
            kind: "Namespace".to_string(),
        }
    }

    fn sample_resource_quota_policy(name: &str, namespace: &str, kind: &str) -> PolicyInfo {
        PolicyInfo {
            uid: format!("uid-{namespace}-{name}"),
            name: name.to_string(),
            r#type: if kind == "LimitRange" {
                "Limit Range".to_string()
            } else {
                "Resource Quota".to_string()
            },
            scope: "Namespaced".to_string(),
            namespace: namespace.to_string(),
            status: "Enforced".to_string(),
            mode: "enforce".to_string(),
            violations: 0,
            last_updated: "1d".to_string(),
            description: "Resource limits (2/4 tracked)".to_string(),
            rules: "hard:\n  cpu: '4'\n".to_string(),
            kind: kind.to_string(),
        }
    }

    fn sample_hpa(name: &str, namespace: &str) -> HpaInfo {
        HpaInfo {
            kind: "HorizontalPodAutoscaler".to_string(),
            name: name.to_string(),
            namespace: namespace.to_string(),
            status: "Active".to_string(),
            scale_target_ref: orbit_core::kubernetes::models::HpaTargetRef {
                kind: "Deployment".to_string(),
                name: name.to_string(),
                api_version: None,
            },
            min_replicas: 1,
            max_replicas: 5,
            current_replicas: 2,
            desired_replicas: 2,
            metrics: Vec::new(),
            metrics_summary: Vec::new(),
            conditions: Vec::new(),
            last_scale_time: None,
            age: "1d".to_string(),
            created: String::new(),
            labels: Default::default(),
            annotations: Default::default(),
        }
    }

    fn sample_storage_class(name: &str) -> StorageClassInfo {
        StorageClassInfo {
            name: name.to_string(),
            provisioner: "kubernetes.io/aws-ebs".to_string(),
            reclaim_policy: "Delete".to_string(),
            volume_binding_mode: "Immediate".to_string(),
            allow_volume_expansion: true,
            age: "3d".to_string(),
        }
    }

    macro_rules! list_state_test {
        ($name:ident, $state:ident, $status:ident, $field:ident, $cmd:ident, $item:expr) => {
            #[test]
            fn $name() {
                let mut state = $state::default();
                assert!(state.needs_load_for_cluster("ctx-a"));
                let cmd = state.begin_load_for_cluster("ctx-a");
                assert_eq!(
                    cmd,
                    ClusterBridgeCommand::$cmd {
                        cluster_id: "ctx-a".to_string(),
                        generation: 1,
                    }
                );
                assert_eq!(state.status, $status::Loading);
                assert!(state.table.loading);

                assert!(!state.apply_loaded("ctx-other", 1, vec![$item]));
                assert!(!state.apply_loaded("ctx-a", 99, vec![$item]));
                assert!(!state.apply_failed("ctx-a", 99, "stale".to_string()));

                assert!(state.apply_loaded("ctx-a", 1, vec![$item]));
                assert_eq!(state.status, $status::Loaded);
                assert_eq!(state.$field.len(), 1);
                assert!(!state.needs_load_for_cluster("ctx-a"));

                assert!(state.begin_refresh().is_some());
                assert!(state.apply_failed("ctx-a", 2, "forbidden".to_string()));
                assert_eq!(state.status, $status::Error);
                assert_eq!(state.error_message.as_deref(), Some("forbidden"));
                assert_eq!(state.$field.len(), 1);

                state.reset_for_cluster("ctx-b");
                assert_eq!(state.status, $status::Idle);
                assert!(state.$field.is_empty());
                state.reset_for_disconnected();
                assert!(state.cluster_id.is_none());
            }
        };
    }

    list_state_test!(
        test_hpas_state_load_success_error_and_stale_generation,
        HpasState,
        HpasLoadStatus,
        hpas,
        LoadHpas,
        sample_hpa("web", "prod")
    );
    list_state_test!(
        test_storage_classes_state_load_success_error_and_stale_generation,
        StorageClassesState,
        StorageClassesLoadStatus,
        classes,
        LoadStorageClasses,
        sample_storage_class("fast")
    );
    list_state_test!(
        test_security_overview_state_load_success_error_and_stale_generation,
        SecurityOverviewState,
        SecurityOverviewLoadStatus,
        policies,
        LoadSecurityOverview,
        sample_pod_security_policy("prod")
    );
    list_state_test!(
        test_admission_webhooks_state_load_success_error_and_stale_generation,
        AdmissionWebhooksState,
        AdmissionWebhooksLoadStatus,
        policies,
        LoadAdmissionWebhooks,
        sample_pod_security_policy("prod")
    );

    #[test]
    fn test_cluster_list_views_load_apply_and_cluster_switch_reset() {
        let mut views = ClusterListViews::default();
        assert!(views.load_command(WorkspaceView::Pods, "ctx-a").is_none());
        let cmd = views.load_command(WorkspaceView::Hpas, "ctx-a");
        assert_eq!(
            cmd,
            Some(ClusterBridgeCommand::LoadHpas {
                cluster_id: "ctx-a".to_string(),
                generation: 1,
            })
        );
        assert!(views.apply_event(&ClusterBridgeEvent::HpasLoaded {
            cluster_id: "ctx-a".to_string(),
            generation: 1,
            hpas: vec![sample_hpa("web", "prod")],
        }));
        assert!(!views.apply_event(&ClusterBridgeEvent::NodesLoadFailed {
            cluster_id: "ctx-a".to_string(),
            generation: 1,
            message: "x".to_string(),
        }));
        assert_eq!(views.hpas.status, HpasLoadStatus::Loaded);

        let mut commands = Vec::new();
        views.on_cluster_connected(WorkspaceView::StorageClasses, "ctx-b", &mut commands);
        assert_eq!(commands.len(), 1);
        assert_eq!(views.hpas.status, HpasLoadStatus::Idle);
        assert!(views.hpas.hpas.is_empty());
        assert_eq!(
            views.admission_webhooks.status,
            AdmissionWebhooksLoadStatus::Idle
        );
        assert_eq!(
            views.storage_classes.status,
            StorageClassesLoadStatus::Loading
        );

        views.reset_for_disconnected();
        assert_eq!(views.storage_classes.status, StorageClassesLoadStatus::Idle);
        assert!(
            views
                .refresh_commands(WorkspaceView::Pods, "ctx-b")
                .is_empty()
        );
    }

    #[test]
    fn test_pod_security_state_lifecycle_and_generation_guard() {
        let mut state = PodSecurityState::default();
        assert!(state.needs_load_for_cluster("ctx-a"));

        let cmd = state.begin_load_for_cluster("ctx-a");
        assert_eq!(
            cmd,
            ClusterBridgeCommand::LoadPodSecurity {
                cluster_id: "ctx-a".to_string(),
                generation: 1,
            }
        );
        assert_eq!(state.status, PodSecurityLoadStatus::Loading);
        assert!(state.table.loading);

        let policy = sample_pod_security_policy("prod");
        assert!(!state.apply_loaded("ctx-other", 1, vec![policy.clone()]));
        assert!(!state.apply_loaded("ctx-a", 99, vec![policy.clone()]));
        assert!(!state.apply_failed("ctx-other", 1, "err".to_string()));

        assert!(state.apply_loaded("ctx-a", 1, vec![policy]));
        assert_eq!(state.status, PodSecurityLoadStatus::Loaded);
        assert_eq!(state.policies.len(), 1);
        assert!(!state.table.loading);
        assert!(!state.needs_load_for_cluster("ctx-a"));

        let refresh = state.begin_refresh();
        assert_eq!(
            refresh,
            Some(ClusterBridgeCommand::LoadPodSecurity {
                cluster_id: "ctx-a".to_string(),
                generation: 2,
            })
        );
        assert!(!state.table.loading);
        assert!(state.apply_failed("ctx-a", 2, "forbidden".to_string()));
        assert_eq!(state.status, PodSecurityLoadStatus::Error);
        assert_eq!(state.error_message.as_deref(), Some("forbidden"));
        assert_eq!(state.policies.len(), 1);

        state.reset_for_cluster("ctx-b");
        assert_eq!(state.status, PodSecurityLoadStatus::Idle);
        assert_eq!(state.cluster_id.as_deref(), Some("ctx-b"));
        assert!(state.policies.is_empty());

        state.reset_for_disconnected();
        assert!(state.cluster_id.is_none());
        assert!(state.policies.is_empty());
    }

    #[test]
    fn test_resource_quotas_state_lifecycle_kind_filter_and_generation_guard() {
        let mut state = ResourceQuotasState::default();
        assert_eq!(state.kind_filter, ResourceQuotaKindFilter::All);
        assert!(state.needs_load_for_cluster("ctx-a"));

        let cmd = state.begin_load_for_cluster("ctx-a");
        assert_eq!(
            cmd,
            ClusterBridgeCommand::LoadResourceQuotas {
                cluster_id: "ctx-a".to_string(),
                generation: 1,
            }
        );

        let rq = sample_resource_quota_policy("compute-quota", "prod", "ResourceQuota");
        let lr = sample_resource_quota_policy("default-limits", "prod", "LimitRange");
        assert!(!state.apply_loaded("ctx-a", 2, vec![rq.clone()]));
        assert!(state.apply_loaded("ctx-a", 1, vec![rq, lr]));
        assert_eq!(state.policies.len(), 2);

        state.set_kind_filter(ResourceQuotaKindFilter::LimitRange);
        assert_eq!(state.kind_filter, ResourceQuotaKindFilter::LimitRange);

        let refresh = state.begin_refresh();
        assert_eq!(
            refresh,
            Some(ClusterBridgeCommand::LoadResourceQuotas {
                cluster_id: "ctx-a".to_string(),
                generation: 2,
            })
        );
        assert!(state.apply_failed("ctx-a", 2, "timeout".to_string()));
        assert_eq!(state.status, ResourceQuotasLoadStatus::Error);

        // Switching cluster resets kind_filter to All
        state.reset_for_cluster("ctx-b");
        assert_eq!(state.kind_filter, ResourceQuotaKindFilter::All);
        assert!(state.policies.is_empty());

        state.set_kind_filter(ResourceQuotaKindFilter::ResourceQuota);
        state.reset_for_disconnected();
        assert_eq!(state.kind_filter, ResourceQuotaKindFilter::All);
        assert!(state.cluster_id.is_none());
    }

    #[test]
    fn test_storage_overview_state_lifecycle_and_generation_guard() {
        let mut state = StorageOverviewState::default();
        assert!(state.needs_load_for_cluster("ctx-a"));

        let cmd = state.begin_load_for_cluster("ctx-a");
        assert_eq!(
            cmd,
            ClusterBridgeCommand::LoadStorageOverview {
                cluster_id: "ctx-a".to_string(),
                generation: 1,
            }
        );
        assert_eq!(state.status, StorageOverviewLoadStatus::Loading);

        let snapshot = StorageOverviewSnapshot::default();
        assert!(!state.apply_loaded("ctx-other", 1, snapshot.clone()));
        assert!(!state.apply_loaded("ctx-a", 99, snapshot.clone()));
        assert!(state.apply_loaded("ctx-a", 1, snapshot));
        assert_eq!(state.status, StorageOverviewLoadStatus::Loaded);
        assert!(state.snapshot.is_some());
        assert!(!state.needs_load_for_cluster("ctx-a"));

        let refresh = state.begin_refresh();
        assert_eq!(
            refresh,
            Some(ClusterBridgeCommand::LoadStorageOverview {
                cluster_id: "ctx-a".to_string(),
                generation: 2,
            })
        );
        assert!(state.apply_failed("ctx-a", 2, "rpc error".to_string()));
        assert_eq!(state.status, StorageOverviewLoadStatus::Error);
        assert_eq!(state.error_message.as_deref(), Some("rpc error"));
        assert!(state.snapshot.is_some());

        state.reset_for_cluster("ctx-b");
        assert_eq!(state.status, StorageOverviewLoadStatus::Idle);
        assert!(state.snapshot.is_none());

        state.reset_for_disconnected();
        assert!(state.cluster_id.is_none());
        assert!(state.snapshot.is_none());
    }

    #[test]
    fn test_port_forwards_state_load_start_stop_and_clear_lifecycle() {
        let mut state = PortForwardsState::default();
        assert!(state.needs_load_for_cluster("ctx-a"));

        let cmd = state.begin_load_for_cluster("ctx-a");
        assert_eq!(
            cmd,
            ClusterBridgeCommand::LoadPortForwards {
                cluster_id: "ctx-a".to_string(),
                generation: 1,
            }
        );

        let pf1 = PersistedPortForward {
            id: "default/Service/api:8080:80".to_string(),
            cluster_id: "ctx-a".to_string(),
            namespace: "default".to_string(),
            kind: "Service".to_string(),
            name: "api".to_string(),
            local_port: 8080,
            remote_port: 80,
        };
        let pf2 = PersistedPortForward {
            id: "default/Pod/web:3000:3000".to_string(),
            cluster_id: "ctx-a".to_string(),
            namespace: "default".to_string(),
            kind: "Pod".to_string(),
            name: "web".to_string(),
            local_port: 3000,
            remote_port: 3000,
        };

        // Stale generation is ignored
        assert!(!state.apply_loaded("ctx-a", 99, vec![pf1.clone()]));
        assert!(state.apply_loaded("ctx-a", 1, vec![pf1.clone()]));
        assert_eq!(state.status, PortForwardsLoadStatus::Loaded);
        assert_eq!(state.port_forwards.len(), 1);
        assert!(!state.needs_load_for_cluster("ctx-a"));

        // apply_started deduplicates existing entry and adds new one
        assert!(state.apply_started("ctx-a", pf1.clone()));
        assert!(state.apply_started("ctx-a", pf2.clone()));
        assert_eq!(state.port_forwards.len(), 2);

        // apply_stopped cleans stopping_ids, selected_ids, and entry
        state.mark_stopping(&pf1.id);
        state.toggle_select(&pf1.id);
        assert!(state.apply_stopped("ctx-a", &pf1.id));
        assert!(!state.stopping_ids.contains(&pf1.id));
        assert!(!state.selected_ids.contains(&pf1.id));
        assert_eq!(state.port_forwards, vec![pf2]);

        // apply_cleared empties everything
        assert!(state.apply_cleared("ctx-a"));
        assert!(state.port_forwards.is_empty());
        assert!(state.selected_ids.is_empty());
    }

    #[test]
    fn test_port_forwards_state_selection_and_local_port_collision() {
        let mut state = PortForwardsState::default();
        let _ = state.begin_load_for_cluster("ctx-a");
        let pf1 = PersistedPortForward {
            id: "default/Service/api:8080:80".to_string(),
            cluster_id: "ctx-a".to_string(),
            namespace: "default".to_string(),
            kind: "Service".to_string(),
            name: "api".to_string(),
            local_port: 8080,
            remote_port: 80,
        };
        let pf2 = PersistedPortForward {
            id: "default/Pod/web:9090:90".to_string(),
            cluster_id: "ctx-a".to_string(),
            namespace: "default".to_string(),
            kind: "Pod".to_string(),
            name: "web".to_string(),
            local_port: 9090,
            remote_port: 90,
        };
        assert!(state.apply_loaded("ctx-a", 1, vec![pf1.clone(), pf2.clone()]));
        assert!(state.has_local_port(8080));
        assert!(!state.has_local_port(7777));

        let visible = vec![pf1.id.clone(), pf2.id.clone()];
        state.toggle_select_all_visible(&visible);
        assert_eq!(state.selected_ids.len(), 2);
        state.toggle_select(&pf1.id);
        assert_eq!(state.selected_ids.len(), 1);
        state.toggle_select_all_visible(&visible);
        assert_eq!(state.selected_ids.len(), 2);
        state.toggle_select_all_visible(&visible);
        assert!(state.selected_ids.is_empty());
    }

    #[test]
    fn test_exemplar_states_hydrate_table_preferences_and_switch_cluster_filters() {
        use orbit_core::config::{
            PersistedClusterTableFilters, PersistedSortDirection, PersistedTableConfig,
            PersistedTablePreferences,
        };
        use std::collections::HashMap;

        let mut prefs = PersistedTablePreferences::default();
        prefs.tables.insert(
            "pods".to_string(),
            PersistedTableConfig {
                page_size: Some(50),
                hidden_columns: vec!["node".to_string()],
                sort_column: Some("name".to_string()),
                sort_direction: Some(PersistedSortDirection::Asc),
                ..Default::default()
            },
        );
        let mut extra_a = HashMap::new();
        extra_a.insert("status".to_string(), "Running".to_string());
        let mut cluster_a = HashMap::new();
        cluster_a.insert(
            "pods".to_string(),
            PersistedClusterTableFilters {
                search_query: "api".to_string(),
                extra_filters: extra_a,
            },
        );
        prefs.clusters.insert("ctx-a".to_string(), cluster_a);

        let mut cluster_b = HashMap::new();
        cluster_b.insert(
            "pods".to_string(),
            PersistedClusterTableFilters {
                search_query: "worker".to_string(),
                extra_filters: HashMap::new(),
            },
        );
        prefs.clusters.insert("ctx-b".to_string(), cluster_b);

        let mut pods = PodsState::new();
        assert!(pods.table.pagination.enabled);
        let deps = DeploymentsState::new();
        assert!(deps.table.pagination.enabled);
        let svcs = ServicesState::new();
        assert!(svcs.table.pagination.enabled);
        let evs = EventsState::new();
        assert!(evs.table.pagination.enabled);

        pods.hydrate_table_preferences(Some("ctx-a"), &prefs, &[]);
        assert_eq!(pods.table.pagination.page_size, 50);
        assert_eq!(pods.table.filters.search_query, "api");

        // Switching cluster swaps per-cluster filters while preserving table config
        pods.hydrate_table_preferences(Some("ctx-b"), &prefs, &[]);
        assert_eq!(pods.table.pagination.page_size, 50);
        assert_eq!(pods.table.filters.search_query, "worker");
        assert!(pods.table.filters.extra_filters.is_empty());
    }

    #[test]
    fn test_resource_state_begin_refresh_preserves_rows_and_sets_refreshing_flag() {
        use crate::components::data_table::TableBodyMode;

        // 1. PodsState: cold load -> loaded -> warm refresh -> fail (preserves rows) -> loaded
        let mut pods = PodsState::new();
        let cmd = pods.begin_load_for_cluster("ctx-a");
        let ClusterBridgeCommand::LoadPods { generation, .. } = cmd else {
            panic!("expected LoadPods");
        };
        assert!(pods.table.loading);
        assert!(!pods.table.refreshing);
        assert_eq!(pods.table.body_mode(0), TableBodyMode::LoadingSkeleton);

        assert!(pods.apply_loaded("ctx-a", generation, vec![sample_pod("web-1", "default")]));
        assert!(!pods.table.loading);
        assert!(!pods.table.refreshing);
        assert_eq!(pods.table.body_mode(pods.pods.len()), TableBodyMode::Rows);

        let refresh_cmd = pods.begin_refresh().expect("refresh command");
        let ClusterBridgeCommand::LoadPods { generation: g2, .. } = refresh_cmd else {
            panic!("expected LoadPods");
        };
        assert_eq!(pods.pods.len(), 1);
        assert!(!pods.table.loading);
        assert!(pods.table.refreshing);
        assert_eq!(
            pods.table.body_mode(pods.pods.len()),
            TableBodyMode::RefreshingRows
        );

        assert!(pods.apply_failed("ctx-a", g2, "timeout".to_string()));
        assert_eq!(
            pods.pods.len(),
            1,
            "warm refresh failure must preserve live rows"
        );
        assert!(!pods.table.loading);
        assert!(!pods.table.refreshing);
        assert!(pods.error_message.is_some());

        // 2. DeploymentsState
        let mut deps = DeploymentsState::new();
        let ClusterBridgeCommand::LoadDeployments {
            generation: dg1, ..
        } = deps.begin_load_for_cluster("ctx-a")
        else {
            panic!("expected LoadDeployments");
        };
        assert!(deps.apply_loaded("ctx-a", dg1, vec![sample_deployment("api", "default")]));
        let ClusterBridgeCommand::LoadDeployments {
            generation: dg2, ..
        } = deps.begin_refresh().expect("deployments refresh")
        else {
            panic!("expected LoadDeployments");
        };
        assert_eq!(deps.deployments.len(), 1);
        assert!(!deps.table.loading);
        assert!(deps.table.refreshing);
        assert_eq!(
            deps.table.body_mode(deps.deployments.len()),
            TableBodyMode::RefreshingRows
        );
        assert!(deps.apply_loaded(
            "ctx-a",
            dg2,
            vec![
                sample_deployment("api", "default"),
                sample_deployment("worker", "default"),
            ]
        ));
        assert_eq!(deps.deployments.len(), 2);
        assert!(!deps.table.loading);
        assert!(!deps.table.refreshing);
        assert!(deps.error_message.is_none());

        // 3. Macro-generated cluster_list_state! (HpasState)
        let mut hpas = HpasState::default();
        let ClusterBridgeCommand::LoadHpas {
            generation: hg1, ..
        } = hpas.begin_load_for_cluster("ctx-a")
        else {
            panic!("expected LoadHpas");
        };
        assert!(hpas.table.loading);
        assert!(!hpas.table.refreshing);
        assert!(hpas.apply_loaded("ctx-a", hg1, vec![sample_hpa("api-hpa", "default")]));
        let ClusterBridgeCommand::LoadHpas {
            generation: hg2, ..
        } = hpas.begin_refresh().expect("hpas refresh")
        else {
            panic!("expected LoadHpas");
        };
        assert_eq!(hpas.hpas.len(), 1);
        assert!(!hpas.table.loading);
        assert!(hpas.table.refreshing);
        assert_eq!(
            hpas.table.body_mode(hpas.hpas.len()),
            TableBodyMode::RefreshingRows
        );
        assert!(hpas.apply_failed("ctx-a", hg2, "rbac denied".to_string()));
        assert_eq!(hpas.hpas.len(), 1);
        assert!(!hpas.table.loading);
        assert!(!hpas.table.refreshing);
        assert!(hpas.error_message.is_some());
    }

    #[test]
    fn deployments_and_pods_states_default_with_pagination_enabled() {
        use orbit_core::config::{
            PersistedSortDirection, PersistedTableConfig, PersistedTablePreferences,
        };

        let mut deployments = DeploymentsState::default();
        let mut pods = PodsState::default();
        assert!(deployments.table.pagination.enabled);
        assert!(pods.table.pagination.enabled);

        let mut prefs = PersistedTablePreferences::default();
        prefs.tables.insert(
            "deployments".to_string(),
            PersistedTableConfig {
                page_size: Some(25),
                hidden_columns: vec!["age".to_string()],
                sort_column: Some("name".to_string()),
                sort_direction: Some(PersistedSortDirection::Desc),
                ..Default::default()
            },
        );
        prefs.tables.insert(
            "pods".to_string(),
            PersistedTableConfig {
                page_size: Some(50),
                hidden_columns: vec!["node".to_string()],
                sort_column: Some("status".to_string()),
                sort_direction: Some(PersistedSortDirection::Asc),
                ..Default::default()
            },
        );

        deployments.hydrate_table_preferences(Some("ctx-a"), &prefs, &[]);
        pods.hydrate_table_preferences(Some("ctx-a"), &prefs, &[]);

        assert_eq!(deployments.table.pagination.page_size, 25);
        assert!(deployments.table.hidden_columns.contains("age"));
        assert_eq!(
            deployments
                .table
                .sort
                .as_ref()
                .map(|s| s.column_id.as_str()),
            Some("name")
        );
        assert_eq!(pods.table.pagination.page_size, 50);
        assert!(pods.table.hidden_columns.contains("node"));
        assert_eq!(
            pods.table.sort.as_ref().map(|s| s.column_id.as_str()),
            Some("status")
        );
    }

    #[test]
    fn macro_generated_list_states_preserve_generation_guard_and_reset() {
        macro_rules! verify_state {
            ($state:ty, $status:ident, $field:ident, $cmd:ident, $item:expr) => {{
                let mut s = <$state>::default();
                assert!(s.needs_load_for_cluster("ctx-a"));
                let cmd = s.begin_load_for_cluster("ctx-a");
                assert_eq!(
                    cmd,
                    ClusterBridgeCommand::$cmd {
                        cluster_id: "ctx-a".to_string(),
                        generation: 1,
                    }
                );
                assert_eq!(s.status, $status::Loading);
                assert!(s.table.loading);
                assert!(!s.apply_loaded("ctx-other", 1, vec![$item]));
                assert!(!s.apply_loaded("ctx-a", 99, vec![$item]));
                assert!(!s.apply_failed("ctx-a", 99, "stale".to_string()));

                assert!(s.apply_loaded("ctx-a", 1, vec![$item]));
                assert_eq!(s.status, $status::Loaded);
                assert_eq!(s.$field.len(), 1);
                assert!(!s.needs_load_for_cluster("ctx-a"));

                assert_eq!(
                    s.begin_refresh(),
                    Some(ClusterBridgeCommand::$cmd {
                        cluster_id: "ctx-a".to_string(),
                        generation: 2,
                    })
                );
                assert!(s.apply_failed("ctx-a", 2, "timeout".to_string()));
                assert_eq!(s.status, $status::Error);
                assert_eq!(s.error_message.as_deref(), Some("timeout"));

                s.reset_for_cluster("ctx-b");
                assert_eq!(s.status, $status::Idle);
                assert_eq!(s.cluster_id.as_deref(), Some("ctx-b"));
                assert!(s.$field.is_empty());

                s.reset_for_disconnected();
                assert_eq!(s.status, $status::Idle);
                assert!(s.cluster_id.is_none());
                assert!(s.$field.is_empty());
            }};
        }

        verify_state!(
            StatefulSetsState,
            StatefulSetsLoadStatus,
            statefulsets,
            LoadStatefulSets,
            sample_statefulset("db", "prod")
        );
        verify_state!(
            NodesState,
            NodesLoadStatus,
            nodes,
            LoadNodes,
            sample_node("node-1", "Ready", false)
        );
        verify_state!(
            PodSecurityState,
            PodSecurityLoadStatus,
            policies,
            LoadPodSecurity,
            sample_pod_security_policy("prod")
        );
        verify_state!(
            DeploymentsState,
            DeploymentsLoadStatus,
            deployments,
            LoadDeployments,
            sample_deployment("api", "prod")
        );
        verify_state!(
            PodsState,
            PodsLoadStatus,
            pods,
            LoadPods,
            sample_pod("api-0", "prod")
        );
    }

    #[test]
    fn workspace_resources_state_from_config_hydrates_namespace_selection() {
        let mut config = OrbitConfig::default();
        config.update_selected_namespaces_for_cluster(
            "ctx-prod",
            vec!["payments".to_string(), "checkout".to_string()],
        );

        let resources = WorkspaceResourcesState::from_config(&config);
        assert_eq!(
            resources.namespaces.selected_by_cluster.get("ctx-prod"),
            Some(&vec!["payments".to_string(), "checkout".to_string()])
        );
        assert_eq!(resources.overview.status, OverviewLoadStatus::Idle);
        assert_eq!(resources.deployments.status, DeploymentsLoadStatus::Idle);
        assert_eq!(resources.pods.status, PodsLoadStatus::Idle);
        assert_eq!(resources.nodes.status, NodesLoadStatus::Idle);
        assert_eq!(resources.events.status, EventsLoadStatus::Idle);
        assert!(resources.deployments.table.pagination.enabled);
        assert!(resources.pods.table.pagination.enabled);
        assert!(resources.services.table.pagination.enabled);
        assert!(resources.events.table.pagination.enabled);
    }

    #[test]
    fn workspace_resources_state_reset_all_for_disconnect_clears_every_view_state() {
        let mut resources = WorkspaceResourcesState::default();

        let _ = resources.overview.begin_load_for_cluster("ctx-a");
        assert!(
            resources
                .overview
                .apply_loaded("ctx-a", 1, ClusterOverviewSummary::default())
        );

        let _ = resources.deployments.begin_load_for_cluster("ctx-a");
        assert!(resources.deployments.apply_loaded(
            "ctx-a",
            1,
            vec![sample_deployment("api", "default")]
        ));

        let _ = resources.pods.begin_load_for_cluster("ctx-a");
        assert!(
            resources
                .pods
                .apply_loaded("ctx-a", 1, vec![sample_pod("api-0", "default")])
        );

        let _ = resources.nodes.begin_load_for_cluster("ctx-a");
        assert!(resources.nodes.apply_loaded(
            "ctx-a",
            1,
            vec![sample_node("node-1", "Ready", false)]
        ));

        let _ = resources.events.begin_load_for_cluster("ctx-a");
        assert!(resources.events.apply_loaded(
            "ctx-a",
            1,
            vec![sample_event("ev-1", "Normal", "Started", "default")]
        ));

        let _ = resources.list_views.hpas.begin_load_for_cluster("ctx-a");
        assert!(resources.list_views.hpas.apply_loaded(
            "ctx-a",
            1,
            vec![sample_hpa("api-hpa", "default")]
        ));

        resources.reset_all_for_disconnect();

        assert_eq!(resources.overview.status, OverviewLoadStatus::Idle);
        assert!(resources.overview.cluster_id.is_none());
        assert!(resources.overview.summary.is_none());

        assert_eq!(resources.deployments.status, DeploymentsLoadStatus::Idle);
        assert!(resources.deployments.cluster_id.is_none());
        assert!(resources.deployments.deployments.is_empty());

        assert_eq!(resources.pods.status, PodsLoadStatus::Idle);
        assert!(resources.pods.cluster_id.is_none());
        assert!(resources.pods.pods.is_empty());

        assert_eq!(resources.nodes.status, NodesLoadStatus::Idle);
        assert!(resources.nodes.cluster_id.is_none());
        assert!(resources.nodes.nodes.is_empty());

        assert_eq!(resources.events.status, EventsLoadStatus::Idle);
        assert!(resources.events.cluster_id.is_none());
        assert!(resources.events.events.is_empty());

        assert_eq!(resources.list_views.hpas.status, HpasLoadStatus::Idle);
        assert!(resources.list_views.hpas.cluster_id.is_none());
        assert!(resources.list_views.hpas.hpas.is_empty());
    }

    #[test]
    fn workspace_resources_state_load_command_dispatches_only_when_stale() {
        let mut resources = WorkspaceResourcesState::default();
        let switcher = ClusterSwitcherState {
            connection_state: ClusterConnectionState::Connected {
                cluster_id: "ctx-a".to_string(),
                display_name: "ctx-a".to_string(),
                k8s_version: Some("v1.30.0".to_string()),
            },
            ..ClusterSwitcherState::default()
        };

        assert!(
            resources
                .load_command_for_active_view(&switcher, None)
                .is_none()
        );

        let cmd =
            resources.load_command_for_active_view(&switcher, Some(WorkspaceView::Deployments));
        assert_eq!(
            cmd,
            Some(ClusterBridgeCommand::LoadDeployments {
                cluster_id: "ctx-a".to_string(),
                generation: 1,
            })
        );
        assert!(resources.deployments.apply_loaded("ctx-a", 1, Vec::new()));

        assert!(
            resources
                .load_command_for_active_view(&switcher, Some(WorkspaceView::Deployments))
                .is_none()
        );
    }
}
