use crate::resources::{
    ResourceRow, SUPPORTED_RESOURCE_KINDS, kind_api_name, mock_rows_for_kind,
};
use orbit_core::command::{CoreCommand, ResourceKind};
use orbit_core::events::{CoreEvent, TypedResourceBatch, WatchAction};
use orbit_core::kubernetes::models::ClusterInfo;
use std::collections::HashMap;

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
        ActivityCategory::Network => vec![SubNavItem {
            label: "Services",
            view: ActiveView::Resource(ResourceKind::Service),
            resource_kind: Some(ResourceKind::Service),
        }],
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
    }

    pub fn close_cluster_dropdown(&mut self) {
        self.cluster_dropdown_open = false;
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

    pub fn fallback_to_mock(&mut self) {
        self.cluster_status = ClusterStatus::MockFallback;
        for kind in SUPPORTED_RESOURCE_KINDS {
            self.resources.insert(kind, mock_rows_for_kind(kind));
        }
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
        ConfigMapInfo, DeploymentInfo, NodeInfo, PodInfo, Replicas, SecretInfo, ServiceInfo,
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
}
