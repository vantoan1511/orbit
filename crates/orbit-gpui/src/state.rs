use crate::resources::{
    ResourceRow, SUPPORTED_RESOURCE_KINDS, kind_api_name, mock_rows_for_kind,
};
use orbit_core::command::{CoreCommand, ResourceKind};
use orbit_core::events::{CoreEvent, TypedResourceBatch, WatchAction};
use orbit_core::kubernetes::models::ClusterInfo;
use std::collections::{BTreeSet, HashMap};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ActivityCategory {
    Workloads,
    Network,
    Storage,
    Config,
    Cluster,
    Settings,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActiveView {
    Overview,
    Resource(ResourceKind),
    Settings,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SelectedResource {
    pub kind: ResourceKind,
    pub name: String,
    pub namespace: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SubNavItem {
    pub label: &'static str,
    pub view: ActiveView,
    pub resource_kind: Option<ResourceKind>,
}

pub fn sub_nav_items(category: ActivityCategory) -> Vec<SubNavItem> {
    match category {
        ActivityCategory::Workloads => vec![
            SubNavItem {
                label: "Overview",
                view: ActiveView::Overview,
                resource_kind: None,
            },
            SubNavItem {
                label: "Pods",
                view: ActiveView::Resource(ResourceKind::Pod),
                resource_kind: Some(ResourceKind::Pod),
            },
            SubNavItem {
                label: "Deployments",
                view: ActiveView::Resource(ResourceKind::Deployment),
                resource_kind: Some(ResourceKind::Deployment),
            },
            SubNavItem {
                label: "DaemonSets",
                view: ActiveView::Resource(ResourceKind::DaemonSet),
                resource_kind: Some(ResourceKind::DaemonSet),
            },
            SubNavItem {
                label: "StatefulSets",
                view: ActiveView::Resource(ResourceKind::StatefulSet),
                resource_kind: Some(ResourceKind::StatefulSet),
            },
        ],
        ActivityCategory::Network => vec![
            SubNavItem {
                label: "Services",
                view: ActiveView::Resource(ResourceKind::Service),
                resource_kind: Some(ResourceKind::Service),
            },
            SubNavItem {
                label: "Ingresses",
                view: ActiveView::Resource(ResourceKind::Ingress),
                resource_kind: Some(ResourceKind::Ingress),
            },
        ],
        ActivityCategory::Storage => Vec::new(),
        ActivityCategory::Config => vec![
            SubNavItem {
                label: "ConfigMaps",
                view: ActiveView::Resource(ResourceKind::ConfigMap),
                resource_kind: Some(ResourceKind::ConfigMap),
            },
            SubNavItem {
                label: "Secrets",
                view: ActiveView::Resource(ResourceKind::Secret),
                resource_kind: Some(ResourceKind::Secret),
            },
        ],
        ActivityCategory::Cluster => vec![SubNavItem {
            label: "Nodes",
            view: ActiveView::Resource(ResourceKind::Node),
            resource_kind: Some(ResourceKind::Node),
        }],
        ActivityCategory::Settings => Vec::new(),
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ClusterStatus {
    Connecting,
    Connected(String),
    MockFallback,
}

pub struct OrbitGpuiState {
    pub active_category: ActivityCategory,
    pub sub_nav_open: bool,
    pub active_view: ActiveView,
    pub cluster_status: ClusterStatus,
    pub clusters: Vec<ClusterInfo>,
    pub active_cluster_id: Option<String>,
    pub cluster_dropdown_open: bool,
    pub namespaces: Vec<String>,
    pub selected_namespace: Option<String>,
    pub namespace_dropdown_open: bool,
    pub search_query: String,
    pub search_focused: bool,
    pub resources: HashMap<ResourceKind, Vec<ResourceRow>>,
    pub selected_resource: Option<SelectedResource>,
    pub raw_yaml: Option<String>,
    pub raw_yaml_loading: bool,
}

impl Default for OrbitGpuiState {
    fn default() -> Self {
        Self::new()
    }
}

impl OrbitGpuiState {
    pub fn new() -> Self {
        Self {
            active_category: ActivityCategory::Workloads,
            sub_nav_open: true,
            active_view: ActiveView::Resource(ResourceKind::Pod),
            cluster_status: ClusterStatus::Connecting,
            clusters: Vec::new(),
            active_cluster_id: None,
            cluster_dropdown_open: false,
            namespaces: Vec::new(),
            selected_namespace: None,
            namespace_dropdown_open: false,
            search_query: String::new(),
            search_focused: false,
            resources: HashMap::new(),
            selected_resource: None,
            raw_yaml: None,
            raw_yaml_loading: false,
        }
    }

    pub fn select_category(&mut self, category: ActivityCategory) {
        if category == ActivityCategory::Settings {
            self.active_category = ActivityCategory::Settings;
            self.sub_nav_open = false;
            self.select_view(ActiveView::Settings);
            return;
        }

        if self.active_category == category {
            self.sub_nav_open = !self.sub_nav_open;
            return;
        }

        self.active_category = category;
        self.sub_nav_open = true;

        let items = sub_nav_items(category);
        let current_belongs = items.iter().any(|item| item.view == self.active_view);
        if !current_belongs {
            if let Some(first) = items.iter().find(|item| item.resource_kind.is_some()).or(items.first()) {
                self.select_view(first.view);
            } else {
                self.select_view(ActiveView::Overview);
            }
        }
    }

    pub fn select_view(&mut self, view: ActiveView) {
        self.active_view = view;
        let keep_selection = match (&self.selected_resource, view) {
            (Some(sel), ActiveView::Resource(kind)) => sel.kind == kind,
            _ => false,
        };
        if !keep_selection {
            self.close_drawer();
        }
    }

    pub fn select_resource_kind(&mut self, kind: ResourceKind) {
        self.active_category = category_for_kind(kind);
        self.sub_nav_open = true;
        self.select_view(ActiveView::Resource(kind));
    }

    pub fn toggle_cluster_dropdown(&mut self) {
        self.cluster_dropdown_open = !self.cluster_dropdown_open;
        if self.cluster_dropdown_open {
            self.namespace_dropdown_open = false;
        }
    }

    pub fn close_cluster_dropdown(&mut self) {
        self.cluster_dropdown_open = false;
    }

    pub fn toggle_namespace_dropdown(&mut self) {
        self.namespace_dropdown_open = !self.namespace_dropdown_open;
        if self.namespace_dropdown_open {
            self.cluster_dropdown_open = false;
        }
    }

    pub fn close_namespace_dropdown(&mut self) {
        self.namespace_dropdown_open = false;
    }

    pub fn select_namespace(&mut self, ns: Option<String>) {
        self.selected_namespace = ns;
        self.namespace_dropdown_open = false;
    }

    pub fn set_search_query(&mut self, query: String) {
        self.search_query = query;
    }

    pub fn clear_search_query(&mut self) {
        self.search_query.clear();
    }

    pub fn begin_cluster_switch(&mut self, cluster_id: String) -> CoreCommand {
        self.cluster_dropdown_open = false;
        self.namespace_dropdown_open = false;
        self.resources.clear();
        self.namespaces.clear();
        self.selected_namespace = None;
        self.close_drawer();
        self.cluster_status = ClusterStatus::Connecting;
        CoreCommand::SwitchCluster {
            context_name: Some(cluster_id),
        }
    }

    pub fn select_row(&mut self, row: &ResourceRow) -> CoreCommand {
        self.selected_resource = Some(SelectedResource {
            kind: row.kind,
            name: row.name.clone(),
            namespace: row.namespace.clone(),
        });
        self.raw_yaml = None;
        self.raw_yaml_loading = true;

        CoreCommand::GetResourceRaw {
            kind: kind_api_name(row.kind).to_string(),
            name: row.name.clone(),
            namespace: row.namespace.clone(),
            request_id: None,
        }
    }

    pub fn close_drawer(&mut self) {
        self.selected_resource = None;
        self.raw_yaml = None;
        self.raw_yaml_loading = false;
    }

    #[cfg(test)]
    pub fn rows_for_active_view(&self) -> &[ResourceRow] {
        match self.active_view {
            ActiveView::Resource(kind) => self
                .resources
                .get(&kind)
                .map(|v| v.as_slice())
                .unwrap_or(&[]),
            _ => &[],
        }
    }

    pub fn selected_row(&self) -> Option<&ResourceRow> {
        let sel = self.selected_resource.as_ref()?;
        self.resources.get(&sel.kind)?.iter().find(|row| {
            row.name == sel.name && row.namespace == sel.namespace
        })
    }

    pub fn resource_count(&self, kind: ResourceKind) -> usize {
        self.resources.get(&kind).map(|v| v.len()).unwrap_or(0)
    }

    fn row_matches_filters(&self, row: &ResourceRow, search_lower: &str) -> bool {
        if let Some(selected_ns) = &self.selected_namespace
            && let Some(row_ns) = &row.namespace
            && row_ns != selected_ns
        {
            return false;
        }

        if !search_lower.is_empty() {
            let name_matches = row.name.to_lowercase().contains(search_lower);
            let label_matches = row.labels.iter().any(|(k, v)| {
                k.to_lowercase().contains(search_lower) || v.to_lowercase().contains(search_lower)
            });
            if !name_matches && !label_matches {
                return false;
            }
        }

        true
    }

    pub fn filtered_rows_for_kind(&self, kind: ResourceKind) -> Vec<&ResourceRow> {
        let Some(rows) = self.resources.get(&kind) else {
            return Vec::new();
        };
        let search_lower = self.search_query.to_lowercase();
        rows.iter()
            .filter(|row| self.row_matches_filters(row, &search_lower))
            .collect()
    }

    pub fn filtered_rows_for_active_view(&self) -> Vec<&ResourceRow> {
        match self.active_view {
            ActiveView::Resource(kind) => self.filtered_rows_for_kind(kind),
            _ => Vec::new(),
        }
    }

    pub fn filtered_resource_count(&self, kind: ResourceKind) -> usize {
        self.filtered_rows_for_kind(kind).len()
    }

    pub fn fallback_to_mock(&mut self) {
        self.cluster_status = ClusterStatus::MockFallback;
        let mut ns_set: BTreeSet<String> = ["default".to_string(), "kube-system".to_string()]
            .into_iter()
            .collect();
        for kind in SUPPORTED_RESOURCE_KINDS {
            let rows = mock_rows_for_kind(kind);
            for row in &rows {
                if let Some(ns) = &row.namespace {
                    ns_set.insert(ns.clone());
                }
            }
            self.resources.insert(kind, rows);
        }
        self.namespaces = ns_set.into_iter().collect();
    }

    pub fn apply_core_event(&mut self, event: &CoreEvent) {
        match event {
            CoreEvent::EngineReady {
                clusters,
                active_cluster_id,
                ..
            } => {
                self.clusters = clusters.clone();
                self.active_cluster_id = active_cluster_id.clone();
                match active_cluster_id {
                    Some(id) => {
                        if matches!(self.cluster_status, ClusterStatus::MockFallback) {
                            self.resources.clear();
                        }
                        self.cluster_status = ClusterStatus::Connected(id.clone());
                    }
                    None => {
                        self.fallback_to_mock();
                    }
                }
            }
            CoreEvent::ClustersUpdated { clusters } => {
                self.clusters = clusters.clone();
            }
            CoreEvent::ActiveClusterChanged { active_cluster_id } => {
                self.active_cluster_id = active_cluster_id.clone();
                self.cluster_dropdown_open = false;
                match active_cluster_id {
                    Some(id) => {
                        if self.cluster_status != ClusterStatus::Connected(id.clone()) {
                            self.resources.clear();
                            self.namespaces.clear();
                            self.selected_namespace = None;
                            self.close_drawer();
                        }
                        self.cluster_status = ClusterStatus::Connected(id.clone());
                    }
                    None => {
                        self.fallback_to_mock();
                    }
                }
            }
            CoreEvent::ResourceBatchUpdated(batch) => self.apply_resource_batch(batch),
            CoreEvent::RawResourceLoaded {
                kind,
                name,
                namespace,
                yaml,
                ..
            } => {
                if let Some(sel) = &self.selected_resource
                    && kind_api_name(sel.kind) == kind
                    && sel.name == *name
                    && sel.namespace == *namespace
                {
                    self.raw_yaml = Some(yaml.clone());
                    self.raw_yaml_loading = false;
                }
            }
            CoreEvent::ErrorOccurred { message, .. } => {
                if matches!(self.cluster_status, ClusterStatus::Connecting) {
                    self.fallback_to_mock();
                }
                if self.raw_yaml_loading {
                    self.raw_yaml_loading = false;
                    self.raw_yaml = Some(format!("# Failed to load YAML: {message}"));
                }
            }
            _ => {}
        }
    }

    fn apply_resource_batch(&mut self, batch: &TypedResourceBatch) {
        match batch {
            TypedResourceBatch::Namespaces(actions) => {
                let mut ns_set: BTreeSet<String> = self.namespaces.iter().cloned().collect();
                for action in actions {
                    match action {
                        WatchAction::Applied(ns) => {
                            ns_set.insert(ns.name.clone());
                        }
                        WatchAction::Deleted(ns) => {
                            ns_set.remove(&ns.name);
                        }
                    }
                }
                self.namespaces = ns_set.into_iter().collect();
            }
            TypedResourceBatch::Pods(actions) => {
                apply_watch_actions(&mut self.resources, ResourceKind::Pod, actions, |p| {
                    (p.name.as_str(), Some(p.namespace.as_str()))
                });
            }
            TypedResourceBatch::Deployments(actions) => {
                apply_watch_actions(&mut self.resources, ResourceKind::Deployment, actions, |d| {
                    (d.name.as_str(), Some(d.namespace.as_str()))
                });
            }
            TypedResourceBatch::DaemonSets(actions) => {
                apply_watch_actions(&mut self.resources, ResourceKind::DaemonSet, actions, |d| {
                    (d.name.as_str(), Some(d.namespace.as_str()))
                });
            }
            TypedResourceBatch::StatefulSets(actions) => {
                apply_watch_actions(&mut self.resources, ResourceKind::StatefulSet, actions, |s| {
                    (s.name.as_str(), Some(s.namespace.as_str()))
                });
            }
            TypedResourceBatch::Nodes(actions) => {
                apply_watch_actions(&mut self.resources, ResourceKind::Node, actions, |n| {
                    (n.name.as_str(), None)
                });
            }
            TypedResourceBatch::Services(actions) => {
                apply_watch_actions(&mut self.resources, ResourceKind::Service, actions, |s| {
                    (s.name.as_str(), Some(s.namespace.as_str()))
                });
            }
            TypedResourceBatch::Ingresses(actions) => {
                apply_watch_actions(&mut self.resources, ResourceKind::Ingress, actions, |i| {
                    (i.name.as_str(), Some(i.namespace.as_str()))
                });
            }
            TypedResourceBatch::ConfigMaps(actions) => {
                apply_watch_actions(&mut self.resources, ResourceKind::ConfigMap, actions, |c| {
                    (c.name.as_str(), Some(c.namespace.as_str()))
                });
            }
            TypedResourceBatch::Secrets(actions) => {
                apply_watch_actions(&mut self.resources, ResourceKind::Secret, actions, |s| {
                    (s.name.as_str(), Some(s.namespace.as_str()))
                });
            }
            _ => {}
        }
    }
}

fn category_for_kind(kind: ResourceKind) -> ActivityCategory {
    match kind {
        ResourceKind::Service | ResourceKind::Ingress => ActivityCategory::Network,
        ResourceKind::PersistentVolume
        | ResourceKind::PersistentVolumeClaim
        | ResourceKind::StorageClass => ActivityCategory::Storage,
        ResourceKind::ConfigMap | ResourceKind::Secret | ResourceKind::Policy => {
            ActivityCategory::Config
        }
        ResourceKind::Node | ResourceKind::Namespace | ResourceKind::Event => {
            ActivityCategory::Cluster
        }
        _ => ActivityCategory::Workloads,
    }
}

fn apply_watch_actions<T>(
    resources: &mut HashMap<ResourceKind, Vec<ResourceRow>>,
    kind: ResourceKind,
    actions: &[WatchAction<T>],
    key_of: impl Fn(&T) -> (&str, Option<&str>),
) where
    for<'a> &'a T: Into<ResourceRow>,
{
    let rows = resources.entry(kind).or_default();
    for action in actions {
        match action {
            WatchAction::Applied(item) => {
                let row: ResourceRow = item.into();
                if let Some(existing) = rows
                    .iter_mut()
                    .find(|r| r.name == row.name && r.namespace == row.namespace)
                {
                    *existing = row;
                } else {
                    rows.push(row);
                }
            }
            WatchAction::Deleted(item) => {
                let (name, ns) = key_of(item);
                rows.retain(|r| !(r.name == name && r.namespace.as_deref() == ns));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use orbit_core::kubernetes::models::{
        ConfigMapInfo, DeploymentInfo, IngressInfo, NamespaceInfo, NodeInfo, PodInfo, Replicas,
        SecretInfo, ServiceInfo,
    };
    use std::collections::BTreeMap;

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
    fn test_select_category_toggles_sub_nav_and_switches_category() {
        let mut state = OrbitGpuiState::new();
        assert_eq!(state.active_category, ActivityCategory::Workloads);
        assert!(state.sub_nav_open);

        // Clicking active category toggles sub_nav_open
        state.select_category(ActivityCategory::Workloads);
        assert!(!state.sub_nav_open);
        state.select_category(ActivityCategory::Workloads);
        assert!(state.sub_nav_open);

        // Switching to Network opens sub_nav and selects Service
        state.select_category(ActivityCategory::Network);
        assert_eq!(state.active_category, ActivityCategory::Network);
        assert!(state.sub_nav_open);
        assert_eq!(state.active_view, ActiveView::Resource(ResourceKind::Service));

        // Switching to Settings closes sub_nav and selects Settings view
        state.select_category(ActivityCategory::Settings);
        assert_eq!(state.active_category, ActivityCategory::Settings);
        assert!(!state.sub_nav_open);
        assert_eq!(state.active_view, ActiveView::Settings);
    }

    #[test]
    fn test_clusters_updated_and_switch_clears_resources() {
        let mut state = OrbitGpuiState::new();
        let clusters = vec![
            ClusterInfo {
                id: "dev-cluster".to_string(),
                name: "dev-cluster".to_string(),
                status: "Connected".to_string(),
                ..Default::default()
            },
            ClusterInfo {
                id: "prod-cluster".to_string(),
                name: "prod-cluster".to_string(),
                status: "Ready".to_string(),
                ..Default::default()
            },
        ];

        state.apply_core_event(&CoreEvent::EngineReady {
            status: "ready".to_string(),
            message: "ok".to_string(),
            clusters: clusters.clone(),
            active_cluster_id: Some("dev-cluster".to_string()),
            settings: HashMap::new(),
        });
        assert_eq!(state.clusters.len(), 2);
        assert_eq!(
            state.cluster_status,
            ClusterStatus::Connected("dev-cluster".to_string())
        );

        state.apply_core_event(&CoreEvent::ResourceBatchUpdated(
            TypedResourceBatch::Pods(vec![WatchAction::Applied(make_pod(
                "pod-1",
                "default",
                "Running",
                0,
            ))]),
        ));
        assert_eq!(state.rows_for_active_view().len(), 1);

        state.apply_core_event(&CoreEvent::ActiveClusterChanged {
            active_cluster_id: Some("prod-cluster".to_string()),
        });
        assert_eq!(
            state.cluster_status,
            ClusterStatus::Connected("prod-cluster".to_string())
        );
        assert!(state.rows_for_active_view().is_empty());
    }

    #[test]
    fn test_resource_batch_updates_deployments_nodes_services_configmaps_secrets() {
        let mut state = OrbitGpuiState::new();
        state.apply_core_event(&CoreEvent::ActiveClusterChanged {
            active_cluster_id: Some("prod".to_string()),
        });

        state.apply_core_event(&CoreEvent::ResourceBatchUpdated(
            TypedResourceBatch::Deployments(vec![WatchAction::Applied(DeploymentInfo {
                name: "web".to_string(),
                namespace: "default".to_string(),
                status: "Running".to_string(),
                replicas: Replicas {
                    current: 2,
                    desired: 2,
                },
                available: 2,
                up_to_date: 2,
                age: "1d".to_string(),
                ..Default::default()
            })]),
        ));
        state.apply_core_event(&CoreEvent::ResourceBatchUpdated(
            TypedResourceBatch::Nodes(vec![WatchAction::Applied(NodeInfo {
                name: "node-a".to_string(),
                status: "Ready".to_string(),
                role: "worker".to_string(),
                version: "v1.31.0".to_string(),
                cpu_used: "100m".to_string(),
                cpu_total: "4".to_string(),
                mem_used: "1Gi".to_string(),
                mem_total: "8Gi".to_string(),
                uptime: "5d".to_string(),
                ..Default::default()
            })]),
        ));
        state.apply_core_event(&CoreEvent::ResourceBatchUpdated(
            TypedResourceBatch::Services(vec![WatchAction::Applied(ServiceInfo {
                name: "web-svc".to_string(),
                namespace: "default".to_string(),
                r#type: "ClusterIP".to_string(),
                cluster_ip: "10.96.0.10".to_string(),
                external_ip: "-".to_string(),
                ports: "80/TCP".to_string(),
                endpoints: "10.0.0.1:80".to_string(),
                age: "1d".to_string(),
                session_affinity: "None".to_string(),
                internal_traffic_policy: None,
                created: "".to_string(),
                uid: "".to_string(),
                selector: BTreeMap::new(),
                labels: BTreeMap::new(),
                ports_list: vec![],
                endpoints_list: vec![],
                events: vec![],
            })]),
        ));
        state.apply_core_event(&CoreEvent::ResourceBatchUpdated(
            TypedResourceBatch::ConfigMaps(vec![WatchAction::Applied(ConfigMapInfo {
                name: "app-config".to_string(),
                namespace: "default".to_string(),
                labels: BTreeMap::new(),
                annotations: 0,
                created: "".to_string(),
                age: "1d".to_string(),
                resource_version: "1".to_string(),
                immutable: false,
                keys_count: 2,
                size: "100 B".to_string(),
                mounted_pods: 1,
                used_by: vec![],
                data: BTreeMap::new(),
            })]),
        ));
        state.apply_core_event(&CoreEvent::ResourceBatchUpdated(
            TypedResourceBatch::Secrets(vec![WatchAction::Applied(SecretInfo {
                name: "app-secret".to_string(),
                namespace: "default".to_string(),
                labels: BTreeMap::new(),
                annotations: 0,
                r#type: "Opaque".to_string(),
                created: "".to_string(),
                age: "1d".to_string(),
                resource_version: "1".to_string(),
                immutable: false,
                keys_count: 1,
                size: "64 B".to_string(),
                mounted_pods: 1,
                used_by: vec![],
                data: BTreeMap::new(),
            })]),
        ));

        assert_eq!(state.resource_count(ResourceKind::Deployment), 1);
        assert_eq!(state.resource_count(ResourceKind::Node), 1);
        assert_eq!(state.resource_count(ResourceKind::Service), 1);
        assert_eq!(state.resource_count(ResourceKind::ConfigMap), 1);
        assert_eq!(state.resource_count(ResourceKind::Secret), 1);
    }

    #[test]
    fn test_select_row_sets_selection_and_raw_yaml_loaded_populates_drawer() {
        let mut state = OrbitGpuiState::new();
        state.fallback_to_mock();
        let row = state.rows_for_active_view()[0].clone();
        let cmd = state.select_row(&row);

        assert!(state.raw_yaml_loading);
        assert_eq!(
            state.selected_resource,
            Some(SelectedResource {
                kind: ResourceKind::Pod,
                name: row.name.clone(),
                namespace: row.namespace.clone(),
            })
        );

        match cmd {
            CoreCommand::GetResourceRaw {
                kind,
                name,
                namespace,
                ..
            } => {
                assert_eq!(kind, "Pod");
                assert_eq!(name, row.name);
                assert_eq!(namespace, row.namespace);
            }
            other => panic!("expected GetResourceRaw, got {:?}", other),
        }

        state.apply_core_event(&CoreEvent::RawResourceLoaded {
            kind: "Pod".to_string(),
            name: row.name.clone(),
            namespace: row.namespace.clone(),
            yaml: "apiVersion: v1\nkind: Pod".to_string(),
            request_id: None,
        });

        assert!(!state.raw_yaml_loading);
        assert_eq!(
            state.raw_yaml.as_deref(),
            Some("apiVersion: v1\nkind: Pod")
        );

        state.close_drawer();
        assert!(state.selected_resource.is_none());
        assert!(state.raw_yaml.is_none());
    }

    #[test]
    fn test_namespace_batch_and_filtering_keeps_cluster_scoped_nodes() {
        let mut state = OrbitGpuiState::new();
        state.apply_core_event(&CoreEvent::ActiveClusterChanged {
            active_cluster_id: Some("prod".to_string()),
        });

        let make_ns = |name: &str| NamespaceInfo {
            name: name.to_string(),
            status: "Active".to_string(),
            is_system: name == "kube-system",
            age: "10d".to_string(),
            labels: BTreeMap::new(),
            annotations: BTreeMap::new(),
            uid: String::new(),
            created: String::new(),
        };

        state.apply_core_event(&CoreEvent::ResourceBatchUpdated(
            TypedResourceBatch::Namespaces(vec![
                WatchAction::Applied(make_ns("production")),
                WatchAction::Applied(make_ns("default")),
                WatchAction::Applied(make_ns("kube-system")),
                WatchAction::Applied(make_ns("default")),
            ]),
        ));
        assert_eq!(
            state.namespaces,
            vec![
                "default".to_string(),
                "kube-system".to_string(),
                "production".to_string()
            ]
        );

        state.apply_core_event(&CoreEvent::ResourceBatchUpdated(
            TypedResourceBatch::Pods(vec![
                WatchAction::Applied(make_pod("pod-default", "default", "Running", 0)),
                WatchAction::Applied(make_pod("pod-prod", "production", "Running", 0)),
            ]),
        ));
        state.apply_core_event(&CoreEvent::ResourceBatchUpdated(
            TypedResourceBatch::Nodes(vec![WatchAction::Applied(NodeInfo {
                name: "node-1".to_string(),
                status: "Ready".to_string(),
                ..Default::default()
            })]),
        ));

        state.select_namespace(Some("production".to_string()));
        assert_eq!(state.selected_namespace.as_deref(), Some("production"));
        assert!(!state.namespace_dropdown_open);

        let filtered_pods = state.filtered_rows_for_kind(ResourceKind::Pod);
        assert_eq!(filtered_pods.len(), 1);
        assert_eq!(filtered_pods[0].name, "pod-prod");

        // Cluster-scoped nodes (namespace: None) are always kept
        let filtered_nodes = state.filtered_rows_for_kind(ResourceKind::Node);
        assert_eq!(filtered_nodes.len(), 1);
        assert_eq!(filtered_nodes[0].name, "node-1");

        // Deleting a namespace removes it from namespaces
        state.apply_core_event(&CoreEvent::ResourceBatchUpdated(
            TypedResourceBatch::Namespaces(vec![WatchAction::Deleted(make_ns("production"))]),
        ));
        assert_eq!(
            state.namespaces,
            vec!["default".to_string(), "kube-system".to_string()]
        );
    }

    #[test]
    fn test_search_filter_matches_name_and_labels_case_insensitively_and_persists_across_views() {
        let mut state = OrbitGpuiState::new();
        let mut pod1 = make_pod("api-gateway-1", "production", "Running", 0);
        pod1.labels.insert("app".to_string(), "OrbitAPI".to_string());
        let mut pod2 = make_pod("worker-2", "production", "Running", 0);
        pod2.labels.insert("tier".to_string(), "backend".to_string());

        state.apply_core_event(&CoreEvent::ResourceBatchUpdated(
            TypedResourceBatch::Pods(vec![
                WatchAction::Applied(pod1),
                WatchAction::Applied(pod2),
            ]),
        ));

        // Match by label value case-insensitively
        state.set_search_query("orbitapi".to_string());
        let pods = state.filtered_rows_for_active_view();
        assert_eq!(pods.len(), 1);
        assert_eq!(pods[0].name, "api-gateway-1");

        // Switch view -> search_query persists
        state.select_resource_kind(ResourceKind::Deployment);
        assert_eq!(state.search_query, "orbitapi");

        state.clear_search_query();
        assert!(state.search_query.is_empty());
        assert_eq!(state.filtered_resource_count(ResourceKind::Pod), 2);
    }

    #[test]
    fn test_begin_cluster_switch_clears_state_and_sets_connecting() {
        let mut state = OrbitGpuiState::new();
        state.fallback_to_mock();
        state.select_namespace(Some("production".to_string()));
        let row = state.rows_for_active_view()[0].clone();
        let _ = state.select_row(&row);
        state.cluster_dropdown_open = true;

        let cmd = state.begin_cluster_switch("staging-cluster".to_string());
        assert_eq!(state.cluster_status, ClusterStatus::Connecting);
        assert!(state.resources.is_empty());
        assert!(state.namespaces.is_empty());
        assert!(state.selected_namespace.is_none());
        assert!(state.selected_resource.is_none());
        assert!(!state.cluster_dropdown_open);

        match cmd {
            CoreCommand::SwitchCluster { context_name } => {
                assert_eq!(context_name.as_deref(), Some("staging-cluster"));
            }
            other => panic!("expected SwitchCluster, got {:?}", other),
        }
    }

    #[test]
    fn test_ingresses_in_network_sub_nav_and_batch_update() {
        let network_items = sub_nav_items(ActivityCategory::Network);
        assert!(network_items
            .iter()
            .any(|item| item.label == "Ingresses" && item.resource_kind == Some(ResourceKind::Ingress)));

        let mut state = OrbitGpuiState::new();
        state.apply_core_event(&CoreEvent::ResourceBatchUpdated(
            TypedResourceBatch::Ingresses(vec![WatchAction::Applied(IngressInfo {
                name: "web-ingress".to_string(),
                namespace: "default".to_string(),
                class_name: Some("nginx".to_string()),
                hosts: "app.example.com".to_string(),
                address: "10.0.0.20".to_string(),
                ports: "80, 443".to_string(),
                age: "1d".to_string(),
                created: String::new(),
                uid: String::new(),
                labels: BTreeMap::new(),
                annotations: BTreeMap::new(),
                rules_summary: vec![],
            })]),
        ));
        assert_eq!(state.resource_count(ResourceKind::Ingress), 1);
    }
}
