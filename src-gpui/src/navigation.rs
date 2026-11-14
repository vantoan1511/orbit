use crate::theme::K8sResourceKind;

pub const ACTIVITY_BAR_WIDTH: f32 = 56.0;
pub const DEFAULT_PANEL_WIDTH: f32 = 260.0;
pub const MIN_PANEL_WIDTH: f32 = 180.0;
pub const MAX_PANEL_WIDTH: f32 = 600.0;
pub const COLLAPSE_THRESHOLD: f32 = 90.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SidebarCategory {
    Clusters,
    Core,
    Workloads,
    Network,
    Storage,
    Config,
    Security,
    Logs,
}

impl SidebarCategory {
    pub const ALL: [SidebarCategory; 8] = [
        SidebarCategory::Clusters,
        SidebarCategory::Core,
        SidebarCategory::Workloads,
        SidebarCategory::Network,
        SidebarCategory::Storage,
        SidebarCategory::Config,
        SidebarCategory::Security,
        SidebarCategory::Logs,
    ];

    pub const fn id(self) -> &'static str {
        match self {
            Self::Clusters => "clusters",
            Self::Core => "core",
            Self::Workloads => "workloads",
            Self::Network => "network",
            Self::Storage => "storage",
            Self::Config => "config",
            Self::Security => "security",
            Self::Logs => "logs",
        }
    }

    pub const fn label(self) -> &'static str {
        match self {
            Self::Clusters => "Clusters",
            Self::Core => "Overview & Core",
            Self::Workloads => "Workloads",
            Self::Network => "Network",
            Self::Storage => "Storage",
            Self::Config => "Config & Secrets",
            Self::Security => "Security",
            Self::Logs => "Logs",
        }
    }

    pub const fn badge_code(self) -> &'static str {
        match self {
            Self::Clusters => "CL",
            Self::Core => "OV",
            Self::Workloads => "WL",
            Self::Network => "NW",
            Self::Storage => "ST",
            Self::Config => "CF",
            Self::Security => "SC",
            Self::Logs => "LG",
        }
    }

    pub const fn default_view(self) -> Option<WorkspaceView> {
        match self {
            Self::Clusters => None,
            Self::Core => Some(WorkspaceView::Overview),
            Self::Workloads => Some(WorkspaceView::Deployments),
            Self::Network => Some(WorkspaceView::Services),
            Self::Storage => Some(WorkspaceView::StorageOverview),
            Self::Config => Some(WorkspaceView::ConfigMaps),
            Self::Security => Some(WorkspaceView::SecurityOverview),
            Self::Logs => Some(WorkspaceView::Logs),
        }
    }

    pub const fn nav_items(self) -> &'static [WorkspaceView] {
        match self {
            Self::Clusters => &[],
            Self::Core => &[
                WorkspaceView::Overview,
                WorkspaceView::Nodes,
                WorkspaceView::Namespaces,
                WorkspaceView::Events,
            ],
            Self::Workloads => &[
                WorkspaceView::WorkloadsOverview,
                WorkspaceView::Pods,
                WorkspaceView::Deployments,
                WorkspaceView::StatefulSets,
                WorkspaceView::DaemonSets,
                WorkspaceView::ReplicaSets,
                WorkspaceView::Jobs,
                WorkspaceView::CronJobs,
                WorkspaceView::Hpas,
            ],
            Self::Network => &[
                WorkspaceView::Services,
                WorkspaceView::Ingresses,
                WorkspaceView::PortForward,
            ],
            Self::Storage => &[
                WorkspaceView::StorageOverview,
                WorkspaceView::PersistentVolumes,
                WorkspaceView::PersistentVolumeClaims,
                WorkspaceView::StorageClasses,
            ],
            Self::Config => &[WorkspaceView::ConfigMaps, WorkspaceView::Secrets],
            Self::Security => &[
                WorkspaceView::SecurityOverview,
                WorkspaceView::NetworkPolicies,
                WorkspaceView::PodSecurity,
                WorkspaceView::ResourceQuotas,
                WorkspaceView::AdmissionWebhooks,
            ],
            Self::Logs => &[WorkspaceView::Logs],
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WorkspaceView {
    Overview,
    Nodes,
    Namespaces,
    Events,
    WorkloadsOverview,
    Pods,
    Deployments,
    StatefulSets,
    DaemonSets,
    ReplicaSets,
    Jobs,
    CronJobs,
    Hpas,
    Services,
    Ingresses,
    PortForward,
    StorageOverview,
    PersistentVolumes,
    PersistentVolumeClaims,
    StorageClasses,
    ConfigMaps,
    Secrets,
    SecurityOverview,
    NetworkPolicies,
    PodSecurity,
    ResourceQuotas,
    AdmissionWebhooks,
    Logs,
    Settings,
}

impl WorkspaceView {
    pub const fn id(self) -> &'static str {
        match self {
            Self::Overview => "core-overview",
            Self::Nodes => "core-nodes",
            Self::Namespaces => "core-namespaces",
            Self::Events => "core-events",
            Self::WorkloadsOverview => "workloads-overview",
            Self::Pods => "workloads-pods",
            Self::Deployments => "workloads-deployments",
            Self::StatefulSets => "workloads-statefulsets",
            Self::DaemonSets => "workloads-daemonsets",
            Self::ReplicaSets => "workloads-replicasets",
            Self::Jobs => "workloads-jobs",
            Self::CronJobs => "workloads-cronjobs",
            Self::Hpas => "workloads-hpas",
            Self::Services => "network-services",
            Self::Ingresses => "network-ingresses",
            Self::PortForward => "network-port-forward",
            Self::StorageOverview => "storage-overview",
            Self::PersistentVolumes => "storage-pvs",
            Self::PersistentVolumeClaims => "storage-pvcs",
            Self::StorageClasses => "storage-classes",
            Self::ConfigMaps => "config-configmaps",
            Self::Secrets => "config-secrets",
            Self::SecurityOverview => "security-overview",
            Self::NetworkPolicies => "security-network",
            Self::PodSecurity => "security-pod-security",
            Self::ResourceQuotas => "security-resource",
            Self::AdmissionWebhooks => "security-admission",
            Self::Logs => "logs-main",
            Self::Settings => "settings",
        }
    }

    pub const fn label(self) -> &'static str {
        match self {
            Self::Overview | Self::WorkloadsOverview | Self::StorageOverview | Self::SecurityOverview => {
                "Overview"
            }
            Self::Nodes => "Nodes",
            Self::Namespaces => "Namespaces",
            Self::Events => "Events",
            Self::Pods => "Pods",
            Self::Deployments => "Deployments",
            Self::StatefulSets => "StatefulSets",
            Self::DaemonSets => "DaemonSets",
            Self::ReplicaSets => "ReplicaSets",
            Self::Jobs => "Jobs",
            Self::CronJobs => "CronJobs",
            Self::Hpas => "HPAs",
            Self::Services => "Services",
            Self::Ingresses => "Ingresses",
            Self::PortForward => "Port Forward",
            Self::PersistentVolumes => "PersistentVolumes",
            Self::PersistentVolumeClaims => "Volume Claims",
            Self::StorageClasses => "StorageClasses",
            Self::ConfigMaps => "ConfigMaps",
            Self::Secrets => "Secrets",
            Self::NetworkPolicies => "Network Policies",
            Self::PodSecurity => "Pod Security",
            Self::ResourceQuotas => "Resource Quotas & Limits",
            Self::AdmissionWebhooks => "Admission Webhooks",
            Self::Logs => "Logs",
            Self::Settings => "Settings",
        }
    }

    pub const fn description(self) -> &'static str {
        match self {
            Self::Overview => "Cluster summary, health status, and active context diagnostics.",
            Self::Nodes => "Cluster nodes, capacity, conditions, and kubelet status.",
            Self::Namespaces => "Logical cluster partitions and namespace status.",
            Self::Events => "Real-time Kubernetes cluster events and warning stream.",
            Self::WorkloadsOverview => "Aggregate workload health across pods and controllers.",
            Self::Pods => "Running and scheduled pod instances across namespaces.",
            Self::Deployments => "Declarative rollout controllers for stateless workloads.",
            Self::StatefulSets => "Ordered, stateful workload controllers with stable identities.",
            Self::DaemonSets => "Node-level background workload controllers.",
            Self::ReplicaSets => "Pod replica count controllers managed by deployments.",
            Self::Jobs => "One-off batch execution tasks and completion status.",
            Self::CronJobs => "Scheduled recurring batch jobs and run history.",
            Self::Hpas => "Horizontal Pod Autoscalers and scaling metrics.",
            Self::Services => "ClusterIP, NodePort, and LoadBalancer service endpoints.",
            Self::Ingresses => "HTTP/HTTPS routing rules and ingress controllers.",
            Self::PortForward => "Active local-to-cluster port forwarding tunnels.",
            Self::StorageOverview => "Storage capacity, volumes, and provisioning overview.",
            Self::PersistentVolumes => "Cluster-wide provisioned storage volumes.",
            Self::PersistentVolumeClaims => "Namespace-scoped persistent storage requests.",
            Self::StorageClasses => "Dynamic storage provisioner classes and reclaim policies.",
            Self::ConfigMaps => "Non-sensitive configuration key-value data.",
            Self::Secrets => "Encrypted credentials, TLS certificates, and tokens.",
            Self::SecurityOverview => "Cluster policy posture and security controls overview.",
            Self::NetworkPolicies => "Pod-level ingress and egress firewall rules.",
            Self::PodSecurity => "Pod security standards and admission enforcement.",
            Self::ResourceQuotas => "Namespace CPU, memory, and object count limits.",
            Self::AdmissionWebhooks => "Mutating and validating admission webhook configurations.",
            Self::Logs => "Live container log streaming and filtering.",
            Self::Settings => "Orbit preferences, appearance styles, and cluster settings.",
        }
    }

    pub const fn category(self) -> Option<SidebarCategory> {
        match self {
            Self::Overview | Self::Nodes | Self::Namespaces | Self::Events => {
                Some(SidebarCategory::Core)
            }
            Self::WorkloadsOverview
            | Self::Pods
            | Self::Deployments
            | Self::StatefulSets
            | Self::DaemonSets
            | Self::ReplicaSets
            | Self::Jobs
            | Self::CronJobs
            | Self::Hpas => Some(SidebarCategory::Workloads),
            Self::Services | Self::Ingresses | Self::PortForward => {
                Some(SidebarCategory::Network)
            }
            Self::StorageOverview
            | Self::PersistentVolumes
            | Self::PersistentVolumeClaims
            | Self::StorageClasses => Some(SidebarCategory::Storage),
            Self::ConfigMaps | Self::Secrets => Some(SidebarCategory::Config),
            Self::SecurityOverview
            | Self::NetworkPolicies
            | Self::PodSecurity
            | Self::ResourceQuotas
            | Self::AdmissionWebhooks => Some(SidebarCategory::Security),
            Self::Logs => Some(SidebarCategory::Logs),
            Self::Settings => None,
        }
    }

    pub const fn resource_kind(self) -> Option<K8sResourceKind> {
        match self {
            Self::Nodes => Some(K8sResourceKind::Node),
            Self::Pods => Some(K8sResourceKind::Pod),
            Self::Deployments => Some(K8sResourceKind::Deployment),
            Self::StatefulSets => Some(K8sResourceKind::StatefulSet),
            Self::DaemonSets => Some(K8sResourceKind::DaemonSet),
            Self::ReplicaSets => Some(K8sResourceKind::ReplicaSet),
            Self::Jobs | Self::CronJobs => Some(K8sResourceKind::Job),
            Self::Hpas => Some(K8sResourceKind::Hpa),
            Self::Services => Some(K8sResourceKind::Service),
            Self::Ingresses => Some(K8sResourceKind::Ingress),
            Self::ConfigMaps => Some(K8sResourceKind::ConfigMap),
            Self::Secrets => Some(K8sResourceKind::Secret),
            _ => None,
        }
    }

    pub const fn breadcrumb(self) -> (&'static str, &'static str) {
        match self.category() {
            Some(category) => (category.label(), self.label()),
            None => ("Orbit", self.label()),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SidebarLayoutState {
    pub active_category: SidebarCategory,
    pub active_view: WorkspaceView,
    pub is_collapsed: bool,
    pub panel_width: f32,
    pub last_valid_width: f32,
    pub is_resizing: bool,
}

impl Default for SidebarLayoutState {
    fn default() -> Self {
        Self::new()
    }
}

impl SidebarLayoutState {
    pub fn new() -> Self {
        Self {
            active_category: SidebarCategory::Clusters,
            active_view: WorkspaceView::Overview,
            is_collapsed: false,
            panel_width: DEFAULT_PANEL_WIDTH,
            last_valid_width: DEFAULT_PANEL_WIDTH,
            is_resizing: false,
        }
    }

    pub fn visible_panel_category(&self) -> Option<SidebarCategory> {
        if self.is_collapsed {
            None
        } else {
            Some(self.active_category)
        }
    }

    pub fn toggle_category(&mut self, category: SidebarCategory) {
        if self.active_category == category {
            self.is_collapsed = !self.is_collapsed;
        } else {
            self.active_category = category;
            self.is_collapsed = false;
            if let Some(default_view) = category.default_view() {
                self.active_view = default_view;
            }
        }
    }

    pub fn select_view(&mut self, view: WorkspaceView) {
        self.active_view = view;
        if let Some(category) = view.category() {
            self.active_category = category;
        }
    }

    pub fn toggle_collapse(&mut self) {
        if self.is_collapsed {
            self.is_collapsed = false;
            self.panel_width = self
                .last_valid_width
                .clamp(MIN_PANEL_WIDTH, MAX_PANEL_WIDTH);
        } else {
            self.is_collapsed = true;
        }
    }

    pub fn begin_resize(&mut self) {
        self.is_resizing = true;
    }

    pub fn update_resize_from_window_x(&mut self, window_x: f32) {
        if !self.is_resizing {
            return;
        }
        let raw_width = window_x - ACTIVITY_BAR_WIDTH;
        if raw_width < COLLAPSE_THRESHOLD {
            self.is_collapsed = true;
        } else {
            let clamped = raw_width.clamp(MIN_PANEL_WIDTH, MAX_PANEL_WIDTH);
            self.is_collapsed = false;
            self.panel_width = clamped;
            self.last_valid_width = clamped;
        }
    }

    pub fn end_resize(&mut self) {
        self.is_resizing = false;
    }

    pub fn on_cluster_connected(&mut self) {
        if !self.is_collapsed && self.active_category == SidebarCategory::Clusters {
            self.active_category = SidebarCategory::Core;
            self.active_view = WorkspaceView::Overview;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sidebar_categories_and_workspace_views_match_orbit_navigation_tree() {
        assert_eq!(
            SidebarCategory::ALL,
            [
                SidebarCategory::Clusters,
                SidebarCategory::Core,
                SidebarCategory::Workloads,
                SidebarCategory::Network,
                SidebarCategory::Storage,
                SidebarCategory::Config,
                SidebarCategory::Security,
                SidebarCategory::Logs,
            ]
        );
        assert_eq!(SidebarCategory::ALL.len(), 8);

        assert!(SidebarCategory::Clusters.nav_items().is_empty());
        assert_eq!(SidebarCategory::Clusters.default_view(), None);

        for category in SidebarCategory::ALL.iter().copied() {
            if category == SidebarCategory::Clusters {
                continue;
            }
            let items = category.nav_items();
            assert!(!items.is_empty(), "{category:?} must have nav items");
            let default_view = category
                .default_view()
                .expect("non-cluster category must have default view");
            assert!(
                items.contains(&default_view),
                "{category:?} default view must be in nav_items"
            );
            for item in items {
                assert_eq!(item.category(), Some(category));
                assert!(!item.id().is_empty());
                assert!(!item.label().is_empty());
                assert!(!item.description().is_empty());
            }
        }

        assert_eq!(
            WorkspaceView::Pods.resource_kind(),
            Some(K8sResourceKind::Pod)
        );
        assert_eq!(
            WorkspaceView::Deployments.resource_kind(),
            Some(K8sResourceKind::Deployment)
        );
        assert_eq!(
            WorkspaceView::Services.resource_kind(),
            Some(K8sResourceKind::Service)
        );
        assert_eq!(
            WorkspaceView::Ingresses.resource_kind(),
            Some(K8sResourceKind::Ingress)
        );
        assert_eq!(
            WorkspaceView::ConfigMaps.resource_kind(),
            Some(K8sResourceKind::ConfigMap)
        );
        assert_eq!(
            WorkspaceView::Secrets.resource_kind(),
            Some(K8sResourceKind::Secret)
        );
        assert_eq!(
            WorkspaceView::Nodes.resource_kind(),
            Some(K8sResourceKind::Node)
        );
        assert_eq!(
            WorkspaceView::Hpas.resource_kind(),
            Some(K8sResourceKind::Hpa)
        );
        assert_eq!(WorkspaceView::Settings.category(), None);
        assert_eq!(
            WorkspaceView::Settings.breadcrumb(),
            ("Orbit", "Settings")
        );
    }

    #[test]
    fn test_toggle_category_switches_view_and_collapses_when_clicking_active() {
        let mut sidebar = SidebarLayoutState::new();
        assert_eq!(sidebar.active_category, SidebarCategory::Clusters);
        assert_eq!(sidebar.active_view, WorkspaceView::Overview);
        assert!(!sidebar.is_collapsed);
        assert_eq!(sidebar.panel_width, DEFAULT_PANEL_WIDTH);

        sidebar.toggle_category(SidebarCategory::Workloads);
        assert!(!sidebar.is_collapsed);
        assert_eq!(sidebar.active_category, SidebarCategory::Workloads);
        assert_eq!(sidebar.active_view, WorkspaceView::Deployments);

        sidebar.select_view(WorkspaceView::Pods);
        assert_eq!(sidebar.active_view, WorkspaceView::Pods);
        assert_eq!(sidebar.active_category, SidebarCategory::Workloads);

        // Clicking active category while expanded collapses while preserving active_view
        sidebar.toggle_category(SidebarCategory::Workloads);
        assert!(sidebar.is_collapsed);
        assert_eq!(sidebar.visible_panel_category(), None);
        assert_eq!(sidebar.active_view, WorkspaceView::Pods);

        // Clicking active category again re-expands with active_view preserved
        sidebar.toggle_category(SidebarCategory::Workloads);
        assert!(!sidebar.is_collapsed);
        assert_eq!(
            sidebar.visible_panel_category(),
            Some(SidebarCategory::Workloads)
        );
        assert_eq!(sidebar.active_view, WorkspaceView::Pods);
    }

    #[test]
    fn test_drag_resize_clamps_bounds_and_auto_collapses_below_threshold() {
        let mut sidebar = SidebarLayoutState::new();

        sidebar.begin_resize();
        assert!(sidebar.is_resizing);

        sidebar.update_resize_from_window_x(ACTIVITY_BAR_WIDTH + 340.0);
        assert_eq!(sidebar.panel_width, 340.0);
        assert_eq!(sidebar.last_valid_width, 340.0);
        assert!(!sidebar.is_collapsed);

        sidebar.update_resize_from_window_x(ACTIVITY_BAR_WIDTH + 900.0);
        assert_eq!(sidebar.panel_width, MAX_PANEL_WIDTH);
        assert_eq!(sidebar.last_valid_width, MAX_PANEL_WIDTH);

        sidebar.update_resize_from_window_x(ACTIVITY_BAR_WIDTH + 120.0);
        assert_eq!(sidebar.panel_width, MIN_PANEL_WIDTH);
        assert_eq!(sidebar.last_valid_width, MIN_PANEL_WIDTH);

        sidebar.update_resize_from_window_x(ACTIVITY_BAR_WIDTH + 50.0);
        assert!(sidebar.is_collapsed);

        sidebar.end_resize();
        assert!(!sidebar.is_resizing);

        sidebar.toggle_collapse();
        assert!(!sidebar.is_collapsed);
        assert_eq!(sidebar.panel_width, MIN_PANEL_WIDTH);

        // Cluster connected transitions Clusters -> Core when expanded
        sidebar.on_cluster_connected();
        assert_eq!(sidebar.active_category, SidebarCategory::Core);
        assert_eq!(sidebar.active_view, WorkspaceView::Overview);

        // Does not auto-expand when collapsed
        let mut collapsed = SidebarLayoutState::new();
        collapsed.toggle_collapse();
        assert!(collapsed.is_collapsed);
        collapsed.on_cluster_connected();
        assert!(collapsed.is_collapsed);
        assert_eq!(collapsed.active_category, SidebarCategory::Clusters);
    }
}
