// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod add_cluster_state;
mod bridge;
pub mod components;
pub mod icons;
pub mod navigation;
mod state;
pub mod theme;
pub mod views;

use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

use gpui::{
    AnyElement, App, Application, Bounds, ClipboardItem, Context, CursorStyle, FocusHandle,
    FontWeight, KeyDownEvent, MouseButton, MouseDownEvent, MouseMoveEvent, PathPromptOptions,
    Pixels, Point, Rgba, SharedString, Window, WindowAppearance, WindowBounds, WindowDecorations,
    WindowOptions, div, prelude::*, px, size,
};
use lucide_icons::Icon;
use orbit_core::config::OrbitConfig;
use orbit_core::kubernetes::logs::{format_logs_as_csv, format_logs_as_json};
use orbit_core::kubernetes::models::ClusterInfo;
use tokio::sync::mpsc;

use crate::bridge::{ClusterBridge, start_cluster_bridge};
use crate::components::{
    ActionToastKind, ActionToastState, ActiveShellDrawer, AddClusterModalCallbacks,
    CommandPaletteAction, CommandPaletteCallbacks, CommandPaletteState, ContextMenuEntry,
    ContextMenuKeyAction, ContextMenuState, NotificationCenterState, NotificationDrawerCallbacks,
    NotificationSeverity, PortForwardModalCallbacks, PortForwardModalKeyResult,
    PortForwardModalState, PortForwardResourceOption, ProfileDrawerCallbacks, ResourceDrawerProps,
    ResourceDrawerState, ResourceDrawerTabDef, ResourceDrawerTabId, ResourceDrawerTarget,
    TabBarCallbacks, TabContextMenuCallbacks, TabPickerCallbacks, TextTooltip,
    WorkloadActionDialogState, WorkloadActionModalCallbacks, WorkloadActionModalKeyResult,
    WorkloadActionModalState, YamlLoadStatus, YamlViewerProps, build_command_palette_items,
    build_tab_context_menu_entries, extract_ports_from_deployment, extract_ports_from_pod,
    extract_ports_from_replicaset, extract_ports_from_service, extract_ports_from_statefulset,
    notifications_tooltip, parse_hex_color, render_action_toast_overlay, render_add_cluster_modal,
    render_command_palette_overlay, render_context_menu_overlay, render_custom_titlebar,
    render_notification_drawer, render_port_forward_modal, render_profile_drawer,
    render_resource_drawer, render_tab_resource_picker_overlay, render_workload_action_modal,
    render_workspace_tab_bar, render_yaml_viewer, resolve_window_chrome_style,
};
use crate::icons::{render_icon, theme_mode_icon, theme_mode_tooltip};
use crate::navigation::{
    ACTIVITY_BAR_WIDTH, LogsTabTarget, SidebarCategory, SidebarLayoutState, WorkspaceTab,
    WorkspaceView,
};
use crate::state::{
    AddClusterKeyResult, AddClusterModalState, ClusterBridgeCommand, ClusterBridgeEvent,
    ClusterConnectionState, ClusterListViews, ClusterOverviewState, ClusterSwitcherState,
    ConfigMapsState, CronJobsState, DaemonSetsState, DeploymentsState, EventTypeFilter,
    EventsLoadStatus, EventsState, IngressesState, JobsState, NamespaceFilterState,
    NamespaceLoadStatus, NamespacesViewState, NetworkPoliciesState, NodesState,
    PersistentVolumeClaimsState, PersistentVolumesState, PickerKeyAction, PodSecurityState,
    PodsState, PortForwardKindFilter, PortForwardsLoadStatus, PortForwardsState, ReplicaSetsState,
    ResourceQuotaKindFilter, ResourceQuotasState, SecretsState, ServicesState, StatefulSetsState,
    StorageOverviewState, TabDropPosition, WorkloadActionKind, WorkloadsOverviewState,
    WorkspaceTabsState,
};
use crate::theme::{
    AccentColor, OrbitTheme, ThemeMode, UiStyle, apply_text_step, sync_native_titlebar_appearance,
};

gpui::actions!(orbit_shell, [OpenCommandPalette, ToggleSidebar]);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GlobalWorkspaceShortcut {
    ToggleCommandPalette,
    ToggleSidebar,
    FocusSearch,
    RefreshWorkspace,
}

pub fn primary_shortcut_label_for_os(key: &str, is_macos: bool) -> String {
    if is_macos {
        format!("⌘{key}")
    } else {
        format!("Ctrl+{key}")
    }
}

pub fn primary_shortcut_label(key: &str) -> String {
    primary_shortcut_label_for_os(key, cfg!(target_os = "macos"))
}

pub fn resolve_global_workspace_shortcut(
    key: &str,
    key_char: Option<&str>,
    ctrl: bool,
    platform: bool,
    alt: bool,
    text_input_focused: bool,
) -> Option<GlobalWorkspaceShortcut> {
    let has_primary_mod = (ctrl || platform) && !alt;
    if has_primary_mod && key.eq_ignore_ascii_case("k") {
        return Some(GlobalWorkspaceShortcut::ToggleCommandPalette);
    }
    if has_primary_mod && key.eq_ignore_ascii_case("b") {
        return Some(GlobalWorkspaceShortcut::ToggleSidebar);
    }
    if text_input_focused || ctrl || platform || alt {
        return None;
    }
    if key == "/" || key_char == Some("/") {
        return Some(GlobalWorkspaceShortcut::FocusSearch);
    }
    if key.eq_ignore_ascii_case("r") || key_char.is_some_and(|c| c.eq_ignore_ascii_case("r")) {
        return Some(GlobalWorkspaceShortcut::RefreshWorkspace);
    }
    None
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShellFocusTarget {
    ShellRoot,
    PickerSearch,
}

pub const fn focus_target_for_picker_transition(picker_open: bool) -> ShellFocusTarget {
    if picker_open {
        ShellFocusTarget::PickerSearch
    } else {
        ShellFocusTarget::ShellRoot
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StatusPresentation {
    pub status_label: &'static str,
    pub color: Rgba,
    pub pill_label: String,
}

type DrawerTargetContent = (
    SharedString,
    SharedString,
    Option<SharedString>,
    Option<SharedString>,
    Option<SharedString>,
    Vec<ResourceDrawerTabDef>,
    AnyElement,
);

pub fn format_cluster_status(
    state: &ClusterSwitcherState,
    theme: &OrbitTheme,
) -> StatusPresentation {
    match &state.connection_state {
        ClusterConnectionState::Disconnected => StatusPresentation {
            status_label: "Disconnected",
            color: theme.text.muted,
            pill_label: "Select Cluster".to_string(),
        },
        ClusterConnectionState::Connecting { display_name, .. } => StatusPresentation {
            status_label: "Connecting",
            color: theme.semantic.warning.solid,
            pill_label: display_name.clone(),
        },
        ClusterConnectionState::Connected { display_name, .. } => StatusPresentation {
            status_label: "Connected",
            color: theme.semantic.success.solid,
            pill_label: display_name.clone(),
        },
        ClusterConnectionState::Unreachable { display_name, .. } => StatusPresentation {
            status_label: "Unreachable",
            color: theme.semantic.danger.solid,
            pill_label: display_name.clone(),
        },
        ClusterConnectionState::Error { display_name, .. } => StatusPresentation {
            status_label: "Error",
            color: theme.semantic.danger.solid,
            pill_label: display_name
                .clone()
                .unwrap_or_else(|| "Select Cluster".to_string()),
        },
    }
}

fn cluster_display_label(cluster: &ClusterInfo) -> String {
    cluster
        .alias
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(ToString::to_string)
        .unwrap_or_else(|| cluster.name.clone())
}

fn cluster_dot_color(cluster: &ClusterInfo, theme: &OrbitTheme) -> Rgba {
    cluster
        .color
        .as_deref()
        .map(str::trim)
        .and_then(parse_hex_color)
        .unwrap_or_else(|| {
            if cluster.status == "healthy" {
                theme.semantic.success.solid
            } else {
                theme.text.muted
            }
        })
}

fn apply_theme(updated: OrbitTheme, window: &mut Window, cx: &mut App) {
    let bg = updated.style.window_background;
    sync_native_titlebar_appearance(window, &updated);
    cx.set_global(updated);
    window.set_background_appearance(bg);
}

pub fn nav_item_icon_color(_view: WorkspaceView, is_active: bool, theme: &OrbitTheme) -> Rgba {
    let nav = theme.nav_selection();
    if is_active {
        nav.active_fg
    } else {
        nav.inactive_fg
    }
}

pub const fn should_show_header_kind_badge(_view: WorkspaceView) -> bool {
    false
}

pub const fn should_render_workspace_view_header(
    view: WorkspaceView,
    has_active_logs_tab: bool,
) -> bool {
    !matches!(view, WorkspaceView::Settings) || has_active_logs_tab
}

pub const fn is_full_bleed_table_view(view: WorkspaceView) -> bool {
    matches!(
        view,
        WorkspaceView::Namespaces
            | WorkspaceView::Deployments
            | WorkspaceView::StatefulSets
            | WorkspaceView::DaemonSets
            | WorkspaceView::ReplicaSets
            | WorkspaceView::Jobs
            | WorkspaceView::CronJobs
            | WorkspaceView::Pods
            | WorkspaceView::Services
            | WorkspaceView::Ingresses
            | WorkspaceView::ConfigMaps
            | WorkspaceView::Secrets
            | WorkspaceView::PersistentVolumes
            | WorkspaceView::PersistentVolumeClaims
            | WorkspaceView::NetworkPolicies
            | WorkspaceView::PodSecurity
            | WorkspaceView::ResourceQuotas
            | WorkspaceView::Hpas
            | WorkspaceView::StorageClasses
            | WorkspaceView::SecurityOverview
            | WorkspaceView::AdmissionWebhooks
            | WorkspaceView::Nodes
            | WorkspaceView::Events
    )
}

#[derive(Debug, Clone, Copy, Default)]
pub struct WorkspaceLoadingFlags {
    pub overview: bool,
    pub workloads_overview: bool,
    pub namespaces: bool,
    pub deployments: bool,
    pub statefulsets: bool,
    pub daemonsets: bool,
    pub replicasets: bool,
    pub jobs: bool,
    pub cronjobs: bool,
    pub pods: bool,
    pub services: bool,
    pub ingresses: bool,
    pub port_forwards: bool,
    pub configmaps: bool,
    pub secrets: bool,
    pub persistent_volumes: bool,
    pub persistent_volume_claims: bool,
    pub storage_overview: bool,
    pub network_policies: bool,
    pub pod_security: bool,
    pub resource_quotas: bool,
    pub hpas: bool,
    pub storage_classes: bool,
    pub security_overview: bool,
    pub admission_webhooks: bool,
    pub nodes: bool,
    pub events: bool,
}

pub const fn is_workspace_view_loading(
    active_view: WorkspaceView,
    loading: WorkspaceLoadingFlags,
) -> bool {
    match active_view {
        WorkspaceView::Overview => loading.overview,
        WorkspaceView::WorkloadsOverview => loading.workloads_overview,
        WorkspaceView::Namespaces => loading.namespaces,
        WorkspaceView::Deployments => loading.deployments,
        WorkspaceView::StatefulSets => loading.statefulsets,
        WorkspaceView::DaemonSets => loading.daemonsets,
        WorkspaceView::ReplicaSets => loading.replicasets,
        WorkspaceView::Jobs => loading.jobs,
        WorkspaceView::CronJobs => loading.cronjobs,
        WorkspaceView::Pods => loading.pods,
        WorkspaceView::Services => loading.services,
        WorkspaceView::Ingresses => loading.ingresses,
        WorkspaceView::PortForward => loading.port_forwards,
        WorkspaceView::ConfigMaps => loading.configmaps,
        WorkspaceView::Secrets => loading.secrets,
        WorkspaceView::PersistentVolumes => loading.persistent_volumes,
        WorkspaceView::PersistentVolumeClaims => loading.persistent_volume_claims,
        WorkspaceView::StorageOverview => loading.storage_overview,
        WorkspaceView::NetworkPolicies => loading.network_policies,
        WorkspaceView::PodSecurity => loading.pod_security,
        WorkspaceView::ResourceQuotas => loading.resource_quotas,
        WorkspaceView::Hpas => loading.hpas,
        WorkspaceView::StorageClasses => loading.storage_classes,
        WorkspaceView::SecurityOverview => loading.security_overview,
        WorkspaceView::AdmissionWebhooks => loading.admission_webhooks,
        WorkspaceView::Nodes => loading.nodes,
        WorkspaceView::Events => loading.events,
        _ => false,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
pub struct ShellChromeLayout {
    pub show_activity_brand_badge: bool,
    pub show_activity_settings_btn: bool,
    pub show_header_settings_btn: bool,
    pub show_footer_settings_btn: bool,
    pub show_footer_sidebar_width_debug: bool,
    pub show_header_breadcrumb: bool,
    pub show_footer_breadcrumb: bool,
    pub show_sidebar_panel_collapse_btn: bool,
    pub show_header_sidebar_toggle_btn: bool,
}

pub const ORBIT_SHELL_CHROME: ShellChromeLayout = ShellChromeLayout {
    show_activity_brand_badge: false,
    show_activity_settings_btn: true,
    show_header_settings_btn: false,
    show_footer_settings_btn: false,
    show_footer_sidebar_width_debug: false,
    show_header_breadcrumb: false,
    show_footer_breadcrumb: false,
    show_sidebar_panel_collapse_btn: false,
    show_header_sidebar_toggle_btn: false,
};

pub const fn dropdown_chevron_icon(is_open: bool) -> Icon {
    if is_open {
        Icon::ChevronUp
    } else {
        Icon::ChevronDown
    }
}

pub struct NamespaceTriggerPresentation {
    pub label: String,
    pub is_loading: bool,
    pub is_error: bool,
    pub is_filtered: bool,
    pub indicator_color: Rgba,
}

pub fn namespace_trigger_presentation(
    namespaces: &NamespaceFilterState,
    theme: &OrbitTheme,
) -> NamespaceTriggerPresentation {
    let is_loading = namespaces.status == NamespaceLoadStatus::Loading;
    let is_error = namespaces.status == NamespaceLoadStatus::Error;
    let is_filtered = !namespaces.is_all_namespaces();
    let indicator_color = if is_error {
        theme.semantic.danger.solid
    } else if is_loading {
        theme.semantic.warning.solid
    } else if is_filtered {
        theme.accent.accent
    } else {
        theme.text.secondary
    };

    NamespaceTriggerPresentation {
        label: namespaces.summary_label(),
        is_loading,
        is_error,
        is_filtered,
        indicator_color,
    }
}

#[allow(clippy::too_many_arguments)]
pub fn select_workspace_view_with_state(
    state: &ClusterSwitcherState,
    sidebar: &mut SidebarLayoutState,
    workloads_overview: &mut WorkloadsOverviewState,
    namespaces: &mut NamespaceFilterState,
    deployments: &mut DeploymentsState,
    statefulsets: &mut StatefulSetsState,
    daemonsets: &mut DaemonSetsState,
    replicasets: &mut ReplicaSetsState,
    jobs: &mut JobsState,
    cronjobs: &mut CronJobsState,
    pods: &mut PodsState,
    services: &mut ServicesState,
    ingresses: &mut IngressesState,
    configmaps: &mut ConfigMapsState,
    secrets: &mut SecretsState,
    persistent_volumes: &mut PersistentVolumesState,
    persistent_volume_claims: &mut PersistentVolumeClaimsState,
    network_policies: &mut NetworkPoliciesState,
    storage_overview: &mut StorageOverviewState,
    pod_security: &mut PodSecurityState,
    resource_quotas: &mut ResourceQuotasState,
    list_views: &mut ClusterListViews,
    nodes: &mut NodesState,
    events: &mut EventsState,
    view: WorkspaceView,
) -> Option<ClusterBridgeCommand> {
    sidebar.select_view(view);
    load_command_for_active_view(
        state,
        workloads_overview,
        namespaces,
        deployments,
        statefulsets,
        daemonsets,
        replicasets,
        jobs,
        cronjobs,
        pods,
        services,
        ingresses,
        configmaps,
        secrets,
        persistent_volumes,
        persistent_volume_claims,
        network_policies,
        storage_overview,
        pod_security,
        resource_quotas,
        list_views,
        nodes,
        events,
        Some(view),
    )
}

#[allow(clippy::too_many_arguments)]
pub fn select_workspace_view_with_tabs(
    state: &ClusterSwitcherState,
    sidebar: &mut SidebarLayoutState,
    workloads_overview: &mut WorkloadsOverviewState,
    namespaces: &mut NamespaceFilterState,
    deployments: &mut DeploymentsState,
    statefulsets: &mut StatefulSetsState,
    daemonsets: &mut DaemonSetsState,
    replicasets: &mut ReplicaSetsState,
    jobs: &mut JobsState,
    cronjobs: &mut CronJobsState,
    pods: &mut PodsState,
    services: &mut ServicesState,
    ingresses: &mut IngressesState,
    configmaps: &mut ConfigMapsState,
    secrets: &mut SecretsState,
    persistent_volumes: &mut PersistentVolumesState,
    persistent_volume_claims: &mut PersistentVolumeClaimsState,
    network_policies: &mut NetworkPoliciesState,
    storage_overview: &mut StorageOverviewState,
    pod_security: &mut PodSecurityState,
    resource_quotas: &mut ResourceQuotasState,
    list_views: &mut ClusterListViews,
    nodes: &mut NodesState,
    events: &mut EventsState,
    tabs: &mut WorkspaceTabsState,
    view: WorkspaceView,
) -> Vec<ClusterBridgeCommand> {
    let mut commands = Vec::new();
    tabs.open_tab(view);
    if let Some(cmd) = select_workspace_view_with_state(
        state,
        sidebar,
        workloads_overview,
        namespaces,
        deployments,
        statefulsets,
        daemonsets,
        replicasets,
        jobs,
        cronjobs,
        pods,
        services,
        ingresses,
        configmaps,
        secrets,
        persistent_volumes,
        persistent_volume_claims,
        network_policies,
        storage_overview,
        pod_security,
        resource_quotas,
        list_views,
        nodes,
        events,
        view,
    ) {
        commands.push(cmd);
    }
    if let Some(save_cmd) = tabs.persistence_command() {
        commands.push(save_cmd);
    }
    commands
}

#[allow(clippy::too_many_arguments)]
fn load_command_for_active_view(
    state: &ClusterSwitcherState,
    workloads_overview: &mut WorkloadsOverviewState,
    namespaces: &mut NamespaceFilterState,
    deployments: &mut DeploymentsState,
    statefulsets: &mut StatefulSetsState,
    daemonsets: &mut DaemonSetsState,
    replicasets: &mut ReplicaSetsState,
    jobs: &mut JobsState,
    cronjobs: &mut CronJobsState,
    pods: &mut PodsState,
    services: &mut ServicesState,
    ingresses: &mut IngressesState,
    configmaps: &mut ConfigMapsState,
    secrets: &mut SecretsState,
    persistent_volumes: &mut PersistentVolumesState,
    persistent_volume_claims: &mut PersistentVolumeClaimsState,
    network_policies: &mut NetworkPoliciesState,
    storage_overview: &mut StorageOverviewState,
    pod_security: &mut PodSecurityState,
    resource_quotas: &mut ResourceQuotasState,
    list_views: &mut ClusterListViews,
    nodes: &mut NodesState,
    events: &mut EventsState,
    view: Option<WorkspaceView>,
) -> Option<ClusterBridgeCommand> {
    let view = view?;
    let ClusterConnectionState::Connected { cluster_id, .. } = &state.connection_state else {
        return None;
    };
    if view == WorkspaceView::WorkloadsOverview
        && workloads_overview.needs_load_for_cluster(cluster_id)
    {
        return Some(workloads_overview.begin_load_for_cluster(cluster_id));
    }
    if view == WorkspaceView::Namespaces && namespaces.needs_load_for_cluster(cluster_id) {
        return Some(namespaces.begin_load_for_cluster(cluster_id));
    }
    if view == WorkspaceView::Deployments && deployments.needs_load_for_cluster(cluster_id) {
        return Some(deployments.begin_load_for_cluster(cluster_id));
    }
    if view == WorkspaceView::StatefulSets && statefulsets.needs_load_for_cluster(cluster_id) {
        return Some(statefulsets.begin_load_for_cluster(cluster_id));
    }
    if view == WorkspaceView::DaemonSets && daemonsets.needs_load_for_cluster(cluster_id) {
        return Some(daemonsets.begin_load_for_cluster(cluster_id));
    }
    if view == WorkspaceView::ReplicaSets && replicasets.needs_load_for_cluster(cluster_id) {
        return Some(replicasets.begin_load_for_cluster(cluster_id));
    }
    if view == WorkspaceView::Jobs && jobs.needs_load_for_cluster(cluster_id) {
        return Some(jobs.begin_load_for_cluster(cluster_id));
    }
    if view == WorkspaceView::CronJobs && cronjobs.needs_load_for_cluster(cluster_id) {
        return Some(cronjobs.begin_load_for_cluster(cluster_id));
    }
    if view == WorkspaceView::Pods && pods.needs_load_for_cluster(cluster_id) {
        return Some(pods.begin_load_for_cluster(cluster_id));
    }
    if view == WorkspaceView::Services && services.needs_load_for_cluster(cluster_id) {
        return Some(services.begin_load_for_cluster(cluster_id));
    }
    if view == WorkspaceView::Ingresses && ingresses.needs_load_for_cluster(cluster_id) {
        return Some(ingresses.begin_load_for_cluster(cluster_id));
    }
    if view == WorkspaceView::ConfigMaps && configmaps.needs_load_for_cluster(cluster_id) {
        return Some(configmaps.begin_load_for_cluster(cluster_id));
    }
    if view == WorkspaceView::Secrets && secrets.needs_load_for_cluster(cluster_id) {
        return Some(secrets.begin_load_for_cluster(cluster_id));
    }
    if view == WorkspaceView::PersistentVolumes
        && persistent_volumes.needs_load_for_cluster(cluster_id)
    {
        return Some(persistent_volumes.begin_load_for_cluster(cluster_id));
    }
    if view == WorkspaceView::PersistentVolumeClaims
        && persistent_volume_claims.needs_load_for_cluster(cluster_id)
    {
        return Some(persistent_volume_claims.begin_load_for_cluster(cluster_id));
    }
    if view == WorkspaceView::NetworkPolicies && network_policies.needs_load_for_cluster(cluster_id)
    {
        return Some(network_policies.begin_load_for_cluster(cluster_id));
    }
    if view == WorkspaceView::StorageOverview && storage_overview.needs_load_for_cluster(cluster_id)
    {
        return Some(storage_overview.begin_load_for_cluster(cluster_id));
    }
    if view == WorkspaceView::PodSecurity && pod_security.needs_load_for_cluster(cluster_id) {
        return Some(pod_security.begin_load_for_cluster(cluster_id));
    }
    if view == WorkspaceView::ResourceQuotas && resource_quotas.needs_load_for_cluster(cluster_id) {
        return Some(resource_quotas.begin_load_for_cluster(cluster_id));
    }
    if view == WorkspaceView::Nodes && nodes.needs_load_for_cluster(cluster_id) {
        return Some(nodes.begin_load_for_cluster(cluster_id));
    }
    if view == WorkspaceView::Events && events.needs_load_for_cluster(cluster_id) {
        return Some(events.begin_load_for_cluster(cluster_id));
    }
    list_views.load_command(view, cluster_id)
}

pub fn dropdown_can_disconnect(state: &ClusterSwitcherState) -> bool {
    state.active_cluster_id.is_some()
        || matches!(
            state.connection_state,
            ClusterConnectionState::Connecting { .. }
                | ClusterConnectionState::Unreachable { .. }
                | ClusterConnectionState::Error {
                    cluster_id: Some(_),
                    ..
                }
        )
}

pub fn build_cluster_sidebar_context_menu_entries(
    is_active: bool,
    can_reconnect: bool,
    on_connect: impl 'static + Fn(&mut Window, &mut App),
    on_reconnect: impl 'static + Fn(&mut Window, &mut App),
    on_disconnect: impl 'static + Fn(&mut Window, &mut App),
) -> Vec<ContextMenuEntry> {
    if is_active {
        vec![
            ContextMenuEntry::Item(
                crate::components::ContextMenuItem::new("cluster-reconnect", "Reconnect")
                    .icon(Icon::RefreshCw)
                    .disabled(!can_reconnect)
                    .on_select(on_reconnect),
            ),
            ContextMenuEntry::Separator,
            ContextMenuEntry::Item(
                crate::components::ContextMenuItem::new("cluster-disconnect", "Disconnect")
                    .icon(Icon::Unplug)
                    .danger(true)
                    .on_select(on_disconnect),
            ),
        ]
    } else {
        vec![ContextMenuEntry::Item(
            crate::components::ContextMenuItem::new("cluster-connect", "Connect")
                .icon(Icon::Server)
                .on_select(on_connect),
        )]
    }
}

#[allow(clippy::too_many_arguments)]
pub fn reset_all_workspace_states_for_disconnect(
    overview: &mut ClusterOverviewState,
    workloads_overview: &mut WorkloadsOverviewState,
    namespaces: &mut NamespaceFilterState,
    namespaces_view: &mut NamespacesViewState,
    deployments: &mut DeploymentsState,
    statefulsets: &mut StatefulSetsState,
    daemonsets: &mut DaemonSetsState,
    replicasets: &mut ReplicaSetsState,
    jobs: &mut JobsState,
    cronjobs: &mut CronJobsState,
    pods: &mut PodsState,
    services: &mut ServicesState,
    ingresses: &mut IngressesState,
    configmaps: &mut ConfigMapsState,
    secrets: &mut SecretsState,
    persistent_volumes: &mut PersistentVolumesState,
    persistent_volume_claims: &mut PersistentVolumeClaimsState,
    network_policies: &mut NetworkPoliciesState,
    storage_overview: &mut StorageOverviewState,
    pod_security: &mut PodSecurityState,
    resource_quotas: &mut ResourceQuotasState,
    nodes: &mut NodesState,
    events: &mut EventsState,
    list_views: &mut ClusterListViews,
) {
    overview.reset_for_disconnected();
    workloads_overview.reset_for_disconnected();
    namespaces.reset_for_disconnected();
    namespaces_view.reset_for_disconnected();
    deployments.reset_for_disconnected();
    statefulsets.reset_for_disconnected();
    daemonsets.reset_for_disconnected();
    replicasets.reset_for_disconnected();
    jobs.reset_for_disconnected();
    cronjobs.reset_for_disconnected();
    pods.reset_for_disconnected();
    services.reset_for_disconnected();
    ingresses.reset_for_disconnected();
    configmaps.reset_for_disconnected();
    secrets.reset_for_disconnected();
    persistent_volumes.reset_for_disconnected();
    persistent_volume_claims.reset_for_disconnected();
    network_policies.reset_for_disconnected();
    storage_overview.reset_for_disconnected();
    pod_security.reset_for_disconnected();
    resource_quotas.reset_for_disconnected();
    nodes.reset_for_disconnected();
    events.reset_for_disconnected();
    list_views.reset_for_disconnected();
}

#[allow(clippy::too_many_arguments)]
pub fn disconnect_cluster_with_state(
    state: &mut ClusterSwitcherState,
    sidebar: &mut SidebarLayoutState,
    overview: &mut ClusterOverviewState,
    workloads_overview: &mut WorkloadsOverviewState,
    namespaces: &mut NamespaceFilterState,
    namespaces_view: &mut NamespacesViewState,
    deployments: &mut DeploymentsState,
    statefulsets: &mut StatefulSetsState,
    daemonsets: &mut DaemonSetsState,
    replicasets: &mut ReplicaSetsState,
    jobs: &mut JobsState,
    cronjobs: &mut CronJobsState,
    pods: &mut PodsState,
    services: &mut ServicesState,
    ingresses: &mut IngressesState,
    configmaps: &mut ConfigMapsState,
    secrets: &mut SecretsState,
    persistent_volumes: &mut PersistentVolumesState,
    persistent_volume_claims: &mut PersistentVolumeClaimsState,
    network_policies: &mut NetworkPoliciesState,
    storage_overview: &mut StorageOverviewState,
    pod_security: &mut PodSecurityState,
    resource_quotas: &mut ResourceQuotasState,
    list_views: &mut ClusterListViews,
    nodes: &mut NodesState,
    events: &mut EventsState,
    tabs: &mut WorkspaceTabsState,
    drawer: &mut ResourceDrawerState,
) -> Option<ClusterBridgeCommand> {
    let cmd = state.begin_disconnect()?;
    reset_all_workspace_states_for_disconnect(
        overview,
        workloads_overview,
        namespaces,
        namespaces_view,
        deployments,
        statefulsets,
        daemonsets,
        replicasets,
        jobs,
        cronjobs,
        pods,
        services,
        ingresses,
        configmaps,
        secrets,
        persistent_volumes,
        persistent_volume_claims,
        network_policies,
        storage_overview,
        pod_security,
        resource_quotas,
        nodes,
        events,
        list_views,
    );
    tabs.clear_active_cluster();
    drawer.reset_for_context_change();
    sidebar.on_cluster_disconnected();
    Some(cmd)
}

#[allow(clippy::too_many_arguments)]
pub fn apply_cluster_event_with_tabs(
    state: &mut ClusterSwitcherState,
    sidebar: &mut SidebarLayoutState,
    overview: &mut ClusterOverviewState,
    workloads_overview: &mut WorkloadsOverviewState,
    namespaces: &mut NamespaceFilterState,
    namespaces_view: &mut NamespacesViewState,
    deployments: &mut DeploymentsState,
    statefulsets: &mut StatefulSetsState,
    daemonsets: &mut DaemonSetsState,
    replicasets: &mut ReplicaSetsState,
    jobs: &mut JobsState,
    cronjobs: &mut CronJobsState,
    pods: &mut PodsState,
    services: &mut ServicesState,
    ingresses: &mut IngressesState,
    configmaps: &mut ConfigMapsState,
    secrets: &mut SecretsState,
    persistent_volumes: &mut PersistentVolumesState,
    persistent_volume_claims: &mut PersistentVolumeClaimsState,
    network_policies: &mut NetworkPoliciesState,
    storage_overview: &mut StorageOverviewState,
    pod_security: &mut PodSecurityState,
    resource_quotas: &mut ResourceQuotasState,
    list_views: &mut ClusterListViews,
    nodes: &mut NodesState,
    events: &mut EventsState,
    tabs: &mut WorkspaceTabsState,
    event: ClusterBridgeEvent,
) -> Vec<ClusterBridgeCommand> {
    let is_connected = matches!(
        &event,
        ClusterBridgeEvent::ClusterConnected { .. }
            | ClusterBridgeEvent::ClusterAdded {
                switch_outcome: Some(orbit_core::kubernetes::manager::ClusterSwitchOutcome {
                    healthy: true,
                    ..
                }),
                ..
            }
    );
    let is_disconnected = matches!(&event, ClusterBridgeEvent::ClusterDisconnected { .. });
    let mut commands = apply_cluster_event_with_workspace(
        state,
        sidebar,
        overview,
        workloads_overview,
        namespaces,
        namespaces_view,
        deployments,
        statefulsets,
        daemonsets,
        replicasets,
        jobs,
        cronjobs,
        pods,
        services,
        ingresses,
        configmaps,
        secrets,
        persistent_volumes,
        persistent_volume_claims,
        network_policies,
        storage_overview,
        pod_security,
        resource_quotas,
        list_views,
        nodes,
        events,
        event,
    );

    if is_connected {
        if let Some(id) = state.active_cluster_id.clone() {
            let restored = tabs.activate_cluster(&id);
            sidebar.sync_for_view(restored);
            if let Some(load_cmd) = load_command_for_active_view(
                state,
                workloads_overview,
                namespaces,
                deployments,
                statefulsets,
                daemonsets,
                replicasets,
                jobs,
                cronjobs,
                pods,
                services,
                ingresses,
                configmaps,
                secrets,
                persistent_volumes,
                persistent_volume_claims,
                network_policies,
                storage_overview,
                pod_security,
                resource_quotas,
                list_views,
                nodes,
                events,
                restored,
            ) {
                if !commands.contains(&load_cmd) {
                    commands.push(load_cmd);
                }
            }
        }
    } else if is_disconnected {
        tabs.clear_active_cluster();
        sidebar.on_cluster_disconnected();
    }

    commands
}

#[allow(clippy::too_many_arguments)]
pub fn handle_tab_shortcut_with_state(
    state: &ClusterSwitcherState,
    sidebar: &mut SidebarLayoutState,
    workloads_overview: &mut WorkloadsOverviewState,
    namespaces: &mut NamespaceFilterState,
    deployments: &mut DeploymentsState,
    statefulsets: &mut StatefulSetsState,
    daemonsets: &mut DaemonSetsState,
    replicasets: &mut ReplicaSetsState,
    jobs: &mut JobsState,
    cronjobs: &mut CronJobsState,
    pods: &mut PodsState,
    services: &mut ServicesState,
    ingresses: &mut IngressesState,
    configmaps: &mut ConfigMapsState,
    secrets: &mut SecretsState,
    persistent_volumes: &mut PersistentVolumesState,
    persistent_volume_claims: &mut PersistentVolumeClaimsState,
    network_policies: &mut NetworkPoliciesState,
    storage_overview: &mut StorageOverviewState,
    pod_security: &mut PodSecurityState,
    resource_quotas: &mut ResourceQuotasState,
    list_views: &mut ClusterListViews,
    nodes: &mut NodesState,
    events: &mut EventsState,
    tabs: &mut WorkspaceTabsState,
    key: &str,
    ctrl: bool,
    shift: bool,
    platform: bool,
) -> (bool, Vec<ClusterBridgeCommand>) {
    if !(ctrl || platform) {
        return (false, Vec::new());
    }

    let handled = if key.eq_ignore_ascii_case("tab") {
        if shift {
            tabs.activate_prev_tab();
        } else {
            tabs.activate_next_tab();
        }
        true
    } else if key.eq_ignore_ascii_case("w") {
        tabs.close_active_tab();
        true
    } else {
        false
    };

    if !handled {
        return (false, Vec::new());
    }

    let active = tabs.active_view();
    sidebar.sync_for_view(active);

    let mut commands = Vec::new();
    if let Some(cmd) = load_command_for_active_view(
        state,
        workloads_overview,
        namespaces,
        deployments,
        statefulsets,
        daemonsets,
        replicasets,
        jobs,
        cronjobs,
        pods,
        services,
        ingresses,
        configmaps,
        secrets,
        persistent_volumes,
        persistent_volume_claims,
        network_policies,
        storage_overview,
        pod_security,
        resource_quotas,
        list_views,
        nodes,
        events,
        active,
    ) {
        commands.push(cmd);
    }
    if let Some(save_cmd) = tabs.persistence_command() {
        commands.push(save_cmd);
    }

    (true, commands)
}

#[allow(clippy::too_many_arguments)]
pub fn apply_cluster_event_with_workspace(
    state: &mut ClusterSwitcherState,
    sidebar: &mut SidebarLayoutState,
    overview: &mut ClusterOverviewState,
    workloads_overview: &mut WorkloadsOverviewState,
    namespaces: &mut NamespaceFilterState,
    namespaces_view: &mut NamespacesViewState,
    deployments: &mut DeploymentsState,
    statefulsets: &mut StatefulSetsState,
    daemonsets: &mut DaemonSetsState,
    replicasets: &mut ReplicaSetsState,
    jobs: &mut JobsState,
    cronjobs: &mut CronJobsState,
    pods: &mut PodsState,
    services: &mut ServicesState,
    ingresses: &mut IngressesState,
    configmaps: &mut ConfigMapsState,
    secrets: &mut SecretsState,
    persistent_volumes: &mut PersistentVolumesState,
    persistent_volume_claims: &mut PersistentVolumeClaimsState,
    network_policies: &mut NetworkPoliciesState,
    storage_overview: &mut StorageOverviewState,
    pod_security: &mut PodSecurityState,
    resource_quotas: &mut ResourceQuotasState,
    list_views: &mut ClusterListViews,
    nodes: &mut NodesState,
    events: &mut EventsState,
    event: ClusterBridgeEvent,
) -> Vec<ClusterBridgeCommand> {
    let mut commands = Vec::new();
    match event {
        ClusterBridgeEvent::ClusterConnected { .. }
        | ClusterBridgeEvent::ClusterAdded {
            switch_outcome:
                Some(orbit_core::kubernetes::manager::ClusterSwitchOutcome { healthy: true, .. }),
            ..
        } => {
            state.apply_event(event);
            sidebar.on_cluster_connected();
            namespaces_view.reset_for_cluster();
            if let Some(id) = state.active_cluster_id.as_deref() {
                commands.push(overview.begin_load_for_cluster(id));
                commands.push(namespaces.begin_load_for_cluster(id));
                if sidebar.active_view == WorkspaceView::WorkloadsOverview {
                    commands.push(workloads_overview.begin_load_for_cluster(id));
                } else {
                    workloads_overview.reset_for_cluster(id);
                }
                if sidebar.active_view == WorkspaceView::Deployments {
                    commands.push(deployments.begin_load_for_cluster(id));
                } else {
                    deployments.reset_for_cluster(id);
                }
                if sidebar.active_view == WorkspaceView::StatefulSets {
                    commands.push(statefulsets.begin_load_for_cluster(id));
                } else {
                    statefulsets.reset_for_cluster(id);
                }
                if sidebar.active_view == WorkspaceView::DaemonSets {
                    commands.push(daemonsets.begin_load_for_cluster(id));
                } else {
                    daemonsets.reset_for_cluster(id);
                }
                if sidebar.active_view == WorkspaceView::ReplicaSets {
                    commands.push(replicasets.begin_load_for_cluster(id));
                } else {
                    replicasets.reset_for_cluster(id);
                }
                if sidebar.active_view == WorkspaceView::Jobs {
                    commands.push(jobs.begin_load_for_cluster(id));
                } else {
                    jobs.reset_for_cluster(id);
                }
                if sidebar.active_view == WorkspaceView::CronJobs {
                    commands.push(cronjobs.begin_load_for_cluster(id));
                } else {
                    cronjobs.reset_for_cluster(id);
                }
                if sidebar.active_view == WorkspaceView::Pods {
                    commands.push(pods.begin_load_for_cluster(id));
                } else {
                    pods.reset_for_cluster(id);
                }
                if sidebar.active_view == WorkspaceView::Services {
                    commands.push(services.begin_load_for_cluster(id));
                } else {
                    services.reset_for_cluster(id);
                }
                if sidebar.active_view == WorkspaceView::Ingresses {
                    commands.push(ingresses.begin_load_for_cluster(id));
                } else {
                    ingresses.reset_for_cluster(id);
                }
                if sidebar.active_view == WorkspaceView::ConfigMaps {
                    commands.push(configmaps.begin_load_for_cluster(id));
                } else {
                    configmaps.reset_for_cluster(id);
                }
                if sidebar.active_view == WorkspaceView::Secrets {
                    commands.push(secrets.begin_load_for_cluster(id));
                } else {
                    secrets.reset_for_cluster(id);
                }
                if sidebar.active_view == WorkspaceView::PersistentVolumes {
                    commands.push(persistent_volumes.begin_load_for_cluster(id));
                } else {
                    persistent_volumes.reset_for_cluster(id);
                }
                if sidebar.active_view == WorkspaceView::PersistentVolumeClaims {
                    commands.push(persistent_volume_claims.begin_load_for_cluster(id));
                } else {
                    persistent_volume_claims.reset_for_cluster(id);
                }
                if sidebar.active_view == WorkspaceView::NetworkPolicies {
                    commands.push(network_policies.begin_load_for_cluster(id));
                } else {
                    network_policies.reset_for_cluster(id);
                }
                if sidebar.active_view == WorkspaceView::StorageOverview {
                    commands.push(storage_overview.begin_load_for_cluster(id));
                } else {
                    storage_overview.reset_for_cluster(id);
                }
                if sidebar.active_view == WorkspaceView::PodSecurity {
                    commands.push(pod_security.begin_load_for_cluster(id));
                } else {
                    pod_security.reset_for_cluster(id);
                }
                if sidebar.active_view == WorkspaceView::ResourceQuotas {
                    commands.push(resource_quotas.begin_load_for_cluster(id));
                } else {
                    resource_quotas.reset_for_cluster(id);
                }
                if sidebar.active_view == WorkspaceView::Nodes {
                    commands.push(nodes.begin_load_for_cluster(id));
                } else {
                    nodes.reset_for_cluster(id);
                }
                if sidebar.active_view == WorkspaceView::Events {
                    commands.push(events.begin_load_for_cluster(id));
                } else {
                    events.reset_for_cluster(id);
                }
                list_views.on_cluster_connected(sidebar.active_view, id, &mut commands);
            }
        }
        ClusterBridgeEvent::ClusterUnreachable { .. }
        | ClusterBridgeEvent::ConnectionError { .. }
        | ClusterBridgeEvent::ClusterDisconnected { .. }
        | ClusterBridgeEvent::ClusterAdded {
            switch_outcome:
                Some(orbit_core::kubernetes::manager::ClusterSwitchOutcome { healthy: false, .. }),
            ..
        } => {
            state.apply_event(event);
            reset_all_workspace_states_for_disconnect(
                overview,
                workloads_overview,
                namespaces,
                namespaces_view,
                deployments,
                statefulsets,
                daemonsets,
                replicasets,
                jobs,
                cronjobs,
                pods,
                services,
                ingresses,
                configmaps,
                secrets,
                persistent_volumes,
                persistent_volume_claims,
                network_policies,
                storage_overview,
                pod_security,
                resource_quotas,
                nodes,
                events,
                list_views,
            );
        }
        ClusterBridgeEvent::OverviewLoaded {
            cluster_id,
            generation,
            summary,
        } => {
            overview.apply_loaded(&cluster_id, generation, summary);
        }
        ClusterBridgeEvent::OverviewLoadFailed {
            cluster_id,
            generation,
            message,
        } => {
            overview.apply_failed(&cluster_id, generation, message);
        }
        ClusterBridgeEvent::NamespacesLoaded {
            cluster_id,
            generation,
            namespaces: loaded_namespaces,
        } => {
            let (_applied, pruned) =
                namespaces.apply_loaded(&cluster_id, generation, loaded_namespaces);
            if let Some(pruned_selection) = pruned {
                commands.push(ClusterBridgeCommand::SaveNamespaceSelection {
                    cluster_id,
                    namespaces: pruned_selection,
                });
            }
        }
        ClusterBridgeEvent::NamespacesLoadFailed {
            cluster_id,
            generation,
            message,
        } => {
            namespaces.apply_failed(&cluster_id, generation, message);
        }
        ClusterBridgeEvent::DeploymentsLoaded {
            cluster_id,
            generation,
            deployments: loaded_deployments,
        } => {
            deployments.apply_loaded(&cluster_id, generation, loaded_deployments);
        }
        ClusterBridgeEvent::DeploymentsLoadFailed {
            cluster_id,
            generation,
            message,
        } => {
            deployments.apply_failed(&cluster_id, generation, message);
        }
        ClusterBridgeEvent::StatefulSetsLoaded {
            cluster_id,
            generation,
            statefulsets: loaded_statefulsets,
        } => {
            statefulsets.apply_loaded(&cluster_id, generation, loaded_statefulsets);
        }
        ClusterBridgeEvent::StatefulSetsLoadFailed {
            cluster_id,
            generation,
            message,
        } => {
            statefulsets.apply_failed(&cluster_id, generation, message);
        }
        ClusterBridgeEvent::DaemonSetsLoaded {
            cluster_id,
            generation,
            daemonsets: loaded_daemonsets,
        } => {
            daemonsets.apply_loaded(&cluster_id, generation, loaded_daemonsets);
        }
        ClusterBridgeEvent::DaemonSetsLoadFailed {
            cluster_id,
            generation,
            message,
        } => {
            daemonsets.apply_failed(&cluster_id, generation, message);
        }
        ClusterBridgeEvent::ReplicaSetsLoaded {
            cluster_id,
            generation,
            replicasets: loaded_replicasets,
        } => {
            replicasets.apply_loaded(&cluster_id, generation, loaded_replicasets);
        }
        ClusterBridgeEvent::ReplicaSetsLoadFailed {
            cluster_id,
            generation,
            message,
        } => {
            replicasets.apply_failed(&cluster_id, generation, message);
        }
        ClusterBridgeEvent::JobsLoaded {
            cluster_id,
            generation,
            jobs: loaded_jobs,
        } => {
            jobs.apply_loaded(&cluster_id, generation, loaded_jobs);
        }
        ClusterBridgeEvent::JobsLoadFailed {
            cluster_id,
            generation,
            message,
        } => {
            jobs.apply_failed(&cluster_id, generation, message);
        }
        ClusterBridgeEvent::CronJobsLoaded {
            cluster_id,
            generation,
            cronjobs: loaded_cronjobs,
        } => {
            cronjobs.apply_loaded(&cluster_id, generation, loaded_cronjobs);
        }
        ClusterBridgeEvent::CronJobsLoadFailed {
            cluster_id,
            generation,
            message,
        } => {
            cronjobs.apply_failed(&cluster_id, generation, message);
        }
        ClusterBridgeEvent::PodsLoaded {
            cluster_id,
            generation,
            pods: loaded_pods,
        } => {
            pods.apply_loaded(&cluster_id, generation, loaded_pods);
        }
        ClusterBridgeEvent::PodsLoadFailed {
            cluster_id,
            generation,
            message,
        } => {
            pods.apply_failed(&cluster_id, generation, message);
        }
        other @ ClusterBridgeEvent::ContextsLoaded { .. } => {
            state.apply_event(other);
        }
        ClusterBridgeEvent::NodesLoaded {
            cluster_id,
            generation,
            nodes: loaded_nodes,
        } => {
            nodes.apply_loaded(&cluster_id, generation, loaded_nodes);
        }
        ClusterBridgeEvent::NodesLoadFailed {
            cluster_id,
            generation,
            message,
        } => {
            nodes.apply_failed(&cluster_id, generation, message);
        }
        ClusterBridgeEvent::EventsLoaded {
            cluster_id,
            generation,
            events: loaded_events,
        } => {
            events.apply_loaded(&cluster_id, generation, loaded_events);
        }
        ClusterBridgeEvent::EventsLoadFailed {
            cluster_id,
            generation,
            message,
        } => {
            events.apply_failed(&cluster_id, generation, message);
        }
        ClusterBridgeEvent::ServicesLoaded {
            cluster_id,
            generation,
            services: loaded_services,
        } => {
            services.apply_loaded(&cluster_id, generation, loaded_services);
        }
        ClusterBridgeEvent::ServicesLoadFailed {
            cluster_id,
            generation,
            message,
        } => {
            services.apply_failed(&cluster_id, generation, message);
        }
        ClusterBridgeEvent::IngressesLoaded {
            cluster_id,
            generation,
            ingresses: loaded_ingresses,
        } => {
            ingresses.apply_loaded(&cluster_id, generation, loaded_ingresses);
        }
        ClusterBridgeEvent::IngressesLoadFailed {
            cluster_id,
            generation,
            message,
        } => {
            ingresses.apply_failed(&cluster_id, generation, message);
        }
        ClusterBridgeEvent::ConfigMapsLoaded {
            cluster_id,
            generation,
            configmaps: loaded_configmaps,
        } => {
            configmaps.apply_loaded(&cluster_id, generation, loaded_configmaps);
        }
        ClusterBridgeEvent::ConfigMapsLoadFailed {
            cluster_id,
            generation,
            message,
        } => {
            configmaps.apply_failed(&cluster_id, generation, message);
        }
        ClusterBridgeEvent::SecretsLoaded {
            cluster_id,
            generation,
            secrets: loaded_secrets,
        } => {
            secrets.apply_loaded(&cluster_id, generation, loaded_secrets);
        }
        ClusterBridgeEvent::SecretsLoadFailed {
            cluster_id,
            generation,
            message,
        } => {
            secrets.apply_failed(&cluster_id, generation, message);
        }
        ClusterBridgeEvent::WorkloadsOverviewLoaded {
            cluster_id,
            generation,
            snapshot,
        } => {
            workloads_overview.apply_loaded(&cluster_id, generation, snapshot);
        }
        ClusterBridgeEvent::WorkloadsOverviewLoadFailed {
            cluster_id,
            generation,
            message,
        } => {
            workloads_overview.apply_failed(&cluster_id, generation, message);
        }
        ClusterBridgeEvent::PersistentVolumesLoaded {
            cluster_id,
            generation,
            persistent_volumes: loaded_pvs,
        } => {
            persistent_volumes.apply_loaded(&cluster_id, generation, loaded_pvs);
        }
        ClusterBridgeEvent::PersistentVolumesLoadFailed {
            cluster_id,
            generation,
            message,
        } => {
            persistent_volumes.apply_failed(&cluster_id, generation, message);
        }
        ClusterBridgeEvent::PersistentVolumeClaimsLoaded {
            cluster_id,
            generation,
            persistent_volume_claims: loaded_pvcs,
        } => {
            persistent_volume_claims.apply_loaded(&cluster_id, generation, loaded_pvcs);
        }
        ClusterBridgeEvent::PersistentVolumeClaimsLoadFailed {
            cluster_id,
            generation,
            message,
        } => {
            persistent_volume_claims.apply_failed(&cluster_id, generation, message);
        }
        ClusterBridgeEvent::NetworkPoliciesLoaded {
            cluster_id,
            generation,
            network_policies: loaded_network_policies,
        } => {
            network_policies.apply_loaded(&cluster_id, generation, loaded_network_policies);
        }
        ClusterBridgeEvent::NetworkPoliciesLoadFailed {
            cluster_id,
            generation,
            message,
        } => {
            network_policies.apply_failed(&cluster_id, generation, message);
        }
        ClusterBridgeEvent::StorageOverviewLoaded {
            cluster_id,
            generation,
            snapshot,
        } => {
            storage_overview.apply_loaded(&cluster_id, generation, snapshot);
        }
        ClusterBridgeEvent::StorageOverviewLoadFailed {
            cluster_id,
            generation,
            message,
        } => {
            storage_overview.apply_failed(&cluster_id, generation, message);
        }
        ClusterBridgeEvent::PodSecurityLoaded {
            cluster_id,
            generation,
            policies,
        } => {
            pod_security.apply_loaded(&cluster_id, generation, policies);
        }
        ClusterBridgeEvent::PodSecurityLoadFailed {
            cluster_id,
            generation,
            message,
        } => {
            pod_security.apply_failed(&cluster_id, generation, message);
        }
        ClusterBridgeEvent::ResourceQuotasLoaded {
            cluster_id,
            generation,
            policies,
        } => {
            resource_quotas.apply_loaded(&cluster_id, generation, policies);
        }
        ClusterBridgeEvent::ResourceQuotasLoadFailed {
            cluster_id,
            generation,
            message,
        } => {
            resource_quotas.apply_failed(&cluster_id, generation, message);
        }
        other @ (ClusterBridgeEvent::HpasLoaded { .. }
        | ClusterBridgeEvent::HpasLoadFailed { .. }
        | ClusterBridgeEvent::StorageClassesLoaded { .. }
        | ClusterBridgeEvent::StorageClassesLoadFailed { .. }
        | ClusterBridgeEvent::SecurityOverviewLoaded { .. }
        | ClusterBridgeEvent::SecurityOverviewLoadFailed { .. }
        | ClusterBridgeEvent::AdmissionWebhooksLoaded { .. }
        | ClusterBridgeEvent::AdmissionWebhooksLoadFailed { .. }) => {
            list_views.apply_event(&other);
        }
        ClusterBridgeEvent::ResourceYamlLoaded { .. }
        | ClusterBridgeEvent::ResourceYamlLoadFailed { .. }
        | ClusterBridgeEvent::AddClusterFailed { .. }
        | ClusterBridgeEvent::SettingsSaveFailed { .. }
        | ClusterBridgeEvent::WorkloadActionSucceeded { .. }
        | ClusterBridgeEvent::WorkloadActionFailed { .. }
        | ClusterBridgeEvent::LogTargetsResolved { .. }
        | ClusterBridgeEvent::LogLinesReceived { .. }
        | ClusterBridgeEvent::LogStreamStatus { .. }
        | ClusterBridgeEvent::LogStreamEnded { .. }
        | ClusterBridgeEvent::PortForwardsLoaded { .. }
        | ClusterBridgeEvent::PortForwardStarted { .. }
        | ClusterBridgeEvent::PortForwardStopped { .. }
        | ClusterBridgeEvent::PortForwardsCleared { .. }
        | ClusterBridgeEvent::PortForwardFailed { .. } => {}
        ClusterBridgeEvent::ClusterAdded { .. } => {
            state.apply_event(event);
        }
    }
    commands
}

pub type PortForwardEventOutcome = (Vec<ClusterBridgeCommand>, Option<(u64, ActionToastKind)>);

pub fn apply_port_forward_event(
    port_forwards: &mut PortForwardsState,
    modal: &mut Option<PortForwardModalState>,
    toast: &mut ActionToastState,
    event: &ClusterBridgeEvent,
) -> Option<PortForwardEventOutcome> {
    match event {
        ClusterBridgeEvent::ClusterConnected { outcome, .. } => {
            if outcome.healthy {
                let cmd = port_forwards.begin_load_for_cluster(&outcome.context_name);
                Some((vec![cmd], None))
            } else {
                port_forwards.reset_for_disconnected();
                *modal = None;
                Some((Vec::new(), None))
            }
        }
        ClusterBridgeEvent::ClusterDisconnected { .. }
        | ClusterBridgeEvent::ClusterUnreachable { .. }
        | ClusterBridgeEvent::ConnectionError { .. } => {
            port_forwards.reset_for_disconnected();
            *modal = None;
            Some((Vec::new(), None))
        }
        ClusterBridgeEvent::PortForwardsLoaded {
            cluster_id,
            generation,
            port_forwards: loaded,
        } => {
            port_forwards.apply_loaded(cluster_id, *generation, loaded.clone());
            Some((Vec::new(), None))
        }
        ClusterBridgeEvent::PortForwardStarted {
            cluster_id,
            port_forward,
        } => {
            port_forwards.apply_started(cluster_id, port_forward.clone());
            if let Some(m) = modal.as_ref()
                && m.cluster_id == *cluster_id
                && m.namespace == port_forward.namespace
                && m.kind.eq_ignore_ascii_case(&port_forward.kind)
                && m.name == port_forward.name
                && m.parsed_local_port() == Some(port_forward.local_port)
            {
                *modal = None;
            }
            let toast_id = toast.show_success(
                "Port Forward Active",
                format!(
                    "Forwarding 127.0.0.1:{} -> {}/{}:{}",
                    port_forward.local_port,
                    port_forward.kind,
                    port_forward.name,
                    port_forward.remote_port
                ),
            );
            Some((Vec::new(), Some((toast_id, ActionToastKind::Success))))
        }
        ClusterBridgeEvent::PortForwardStopped { cluster_id, id } => {
            port_forwards.apply_stopped(cluster_id, id);
            Some((Vec::new(), None))
        }
        ClusterBridgeEvent::PortForwardsCleared { cluster_id } => {
            port_forwards.apply_cleared(cluster_id);
            let toast_id =
                toast.show_success("Port Forwards Stopped", "Stopped all active port forwards");
            Some((Vec::new(), Some((toast_id, ActionToastKind::Success))))
        }
        ClusterBridgeEvent::PortForwardFailed {
            cluster_id,
            id,
            message,
        } => {
            port_forwards.apply_failed(cluster_id, id.as_deref(), message.clone());
            if let Some(m) = modal.as_mut()
                && m.cluster_id == *cluster_id
            {
                m.submitting = false;
                m.error = Some(message.clone());
            }
            let toast_id = toast.show_error("Port Forward Failed", message.clone());
            Some((Vec::new(), Some((toast_id, ActionToastKind::Error))))
        }
        _ => None,
    }
}

pub fn open_logs_tab_in_state(
    tabs: &mut WorkspaceTabsState,
    log_tabs: &mut HashMap<LogsTabTarget, views::logs::LogViewerState>,
    next_stream_id: &mut u64,
    cluster_id: Option<&str>,
    kind: &str,
    namespace: &str,
    name: &str,
) -> Option<ClusterBridgeCommand> {
    let target = LogsTabTarget {
        kind: kind.to_string(),
        namespace: namespace.to_string(),
        name: name.to_string(),
    };
    let is_new = tabs.open_logs_tab(target.clone());
    if !is_new && log_tabs.contains_key(&target) {
        return None;
    }
    let stream_id = *next_stream_id;
    *next_stream_id = next_stream_id.saturating_add(1);
    let viewer = views::logs::LogViewerState::new(target.clone(), stream_id);
    let tail_lines = viewer.tail_lines;
    let include_unavailable_pods = viewer.include_unavailable_pods;
    log_tabs.insert(target.clone(), viewer);

    let cluster_id = cluster_id?.to_string();
    Some(ClusterBridgeCommand::StartLogStream {
        cluster_id,
        stream_id,
        namespace: target.namespace,
        workload_kind: target.kind,
        workload_name: target.name,
        pod_filter: None,
        container_filter: None,
        tail_lines,
        include_unavailable_pods,
    })
}

pub fn cleanup_closed_log_tabs_in_state(
    log_tabs: &mut HashMap<LogsTabTarget, views::logs::LogViewerState>,
    closed: Vec<LogsTabTarget>,
) -> Vec<ClusterBridgeCommand> {
    closed
        .into_iter()
        .filter_map(|target| {
            log_tabs
                .remove(&target)
                .map(|state| ClusterBridgeCommand::StopLogStream {
                    stream_id: state.stream_id,
                })
        })
        .collect()
}

pub fn clear_all_log_tabs_in_state(
    tabs: &mut WorkspaceTabsState,
    log_tabs: &mut HashMap<LogsTabTarget, views::logs::LogViewerState>,
) -> Vec<ClusterBridgeCommand> {
    tabs.strip_logs_tabs();
    log_tabs
        .drain()
        .map(|(_, state)| ClusterBridgeCommand::StopLogStream {
            stream_id: state.stream_id,
        })
        .collect()
}

pub fn apply_log_bridge_event(
    log_tabs: &mut HashMap<LogsTabTarget, views::logs::LogViewerState>,
    active_cluster_id: Option<&str>,
    event: &ClusterBridgeEvent,
) {
    match event {
        ClusterBridgeEvent::LogTargetsResolved {
            cluster_id,
            stream_id,
            pods,
            excluded_unavailable_count,
            ..
        } => {
            if active_cluster_id != Some(cluster_id.as_str()) {
                return;
            }
            if let Some(state) = log_tabs.values_mut().find(|s| s.stream_id == *stream_id) {
                state.pods = pods.clone();
                state.excluded_unavailable_count = *excluded_unavailable_count;
            }
        }
        ClusterBridgeEvent::LogLinesReceived {
            cluster_id,
            stream_id,
            pod,
            container,
            lines,
        } => {
            if active_cluster_id != Some(cluster_id.as_str()) {
                return;
            }
            if let Some(state) = log_tabs.values_mut().find(|s| s.stream_id == *stream_id) {
                state.push_lines(pod, container, lines.clone());
            }
        }
        ClusterBridgeEvent::LogStreamStatus {
            cluster_id,
            stream_id,
            message,
            is_error,
        } => {
            if active_cluster_id != Some(cluster_id.as_str()) {
                return;
            }
            if let Some(state) = log_tabs.values_mut().find(|s| s.stream_id == *stream_id) {
                state.status_message = Some(message.clone());
                state.status_is_error = *is_error;
            }
        }
        ClusterBridgeEvent::LogStreamEnded {
            cluster_id,
            stream_id,
        } => {
            if active_cluster_id != Some(cluster_id.as_str()) {
                return;
            }
            if let Some(state) = log_tabs.values_mut().find(|s| s.stream_id == *stream_id) {
                state.is_streaming = false;
            }
        }
        _ => {}
    }
}

pub fn apply_add_cluster_modal_event(modal: &mut AddClusterModalState, event: &ClusterBridgeEvent) {
    match event {
        ClusterBridgeEvent::ClusterAdded { .. } => {
            modal.close();
        }
        ClusterBridgeEvent::AddClusterFailed { message } => {
            modal.is_submitting = false;
            modal.submit_error = Some(message.clone());
        }
        _ => {}
    }
}

pub struct WorkloadActionRefreshTargets<'a> {
    pub drawer: &'a mut ResourceDrawerState,
    pub deployments: &'a mut DeploymentsState,
    pub statefulsets: &'a mut StatefulSetsState,
    pub daemonsets: &'a mut DaemonSetsState,
    pub replicasets: &'a mut ReplicaSetsState,
    pub pods: &'a mut PodsState,
    pub cronjobs: &'a mut CronJobsState,
    pub jobs: &'a mut JobsState,
    pub services: &'a mut ServicesState,
    pub ingresses: &'a mut IngressesState,
    pub configmaps: &'a mut ConfigMapsState,
    pub secrets: &'a mut SecretsState,
    pub network_policies: &'a mut NetworkPoliciesState,
    pub resource_quotas: &'a mut ResourceQuotasState,
    pub namespaces: &'a mut NamespaceFilterState,
    pub persistent_volumes: &'a mut PersistentVolumesState,
    pub persistent_volume_claims: &'a mut PersistentVolumeClaimsState,
    pub nodes: &'a mut NodesState,
    pub events: &'a mut EventsState,
}

pub fn apply_workload_action_event(
    active_cluster_id: Option<&str>,
    modal: &mut WorkloadActionModalState,
    toast: &mut ActionToastState,
    targets: WorkloadActionRefreshTargets<'_>,
    event: &ClusterBridgeEvent,
) -> Option<(u64, ActionToastKind, Vec<ClusterBridgeCommand>)> {
    match event {
        ClusterBridgeEvent::ClusterConnected { .. }
        | ClusterBridgeEvent::ClusterUnreachable { .. }
        | ClusterBridgeEvent::ConnectionError { .. }
        | ClusterBridgeEvent::ClusterDisconnected { .. } => {
            modal.reset_for_disconnect();
            toast.reset_for_disconnect();
            None
        }
        ClusterBridgeEvent::WorkloadActionSucceeded {
            cluster_id,
            action,
            kind,
            namespace,
            name,
            message,
        } => {
            if active_cluster_id != Some(cluster_id.as_str()) {
                return None;
            }
            modal.close();
            if *action == WorkloadActionKind::Delete {
                targets.drawer.close_if_deleted(kind, namespace, name);
            }
            let title = match action {
                WorkloadActionKind::Scale => format!("Scaled {kind}"),
                WorkloadActionKind::Restart => format!("Restarted {kind}"),
                WorkloadActionKind::RestartPod => "Restarted Pod".to_string(),
                WorkloadActionKind::TriggerCronJob => "Triggered CronJob".to_string(),
                WorkloadActionKind::Delete => format!("Deleted {kind}"),
            };
            let toast_id = toast.show_success(title, message.clone());
            let mut refresh_commands = Vec::new();
            match kind.as_str() {
                "Deployment" => {
                    if let Some(cmd) = targets.deployments.begin_refresh() {
                        refresh_commands.push(cmd);
                    }
                }
                "StatefulSet" => {
                    if let Some(cmd) = targets.statefulsets.begin_refresh() {
                        refresh_commands.push(cmd);
                    }
                }
                "DaemonSet" => {
                    if let Some(cmd) = targets.daemonsets.begin_refresh() {
                        refresh_commands.push(cmd);
                    }
                }
                "ReplicaSet" => {
                    if let Some(cmd) = targets.replicasets.begin_refresh() {
                        refresh_commands.push(cmd);
                    }
                }
                "Pod" => {
                    if let Some(cmd) = targets.pods.begin_refresh() {
                        refresh_commands.push(cmd);
                    }
                }
                "CronJob" => {
                    if let Some(cmd) = targets.cronjobs.begin_refresh() {
                        refresh_commands.push(cmd);
                    }
                    if !targets.jobs.jobs.is_empty()
                        && let Some(cmd) = targets.jobs.begin_refresh()
                    {
                        refresh_commands.push(cmd);
                    }
                }
                "Job" => {
                    if let Some(cmd) = targets.jobs.begin_refresh() {
                        refresh_commands.push(cmd);
                    }
                }
                "Service" => {
                    if let Some(cmd) = targets.services.begin_refresh() {
                        refresh_commands.push(cmd);
                    }
                }
                "Ingress" => {
                    if let Some(cmd) = targets.ingresses.begin_refresh() {
                        refresh_commands.push(cmd);
                    }
                }
                "ConfigMap" => {
                    if let Some(cmd) = targets.configmaps.begin_refresh() {
                        refresh_commands.push(cmd);
                    }
                }
                "Secret" => {
                    if let Some(cmd) = targets.secrets.begin_refresh() {
                        refresh_commands.push(cmd);
                    }
                }
                "NetworkPolicy" => {
                    if let Some(cmd) = targets.network_policies.begin_refresh() {
                        refresh_commands.push(cmd);
                    }
                }
                "ResourceQuota" | "LimitRange" => {
                    if let Some(cmd) = targets.resource_quotas.begin_refresh() {
                        refresh_commands.push(cmd);
                    }
                }
                "Namespace" => {
                    if let Some(cmd) = targets.namespaces.begin_refresh() {
                        refresh_commands.push(cmd);
                    }
                }
                "PersistentVolume" => {
                    if let Some(cmd) = targets.persistent_volumes.begin_refresh() {
                        refresh_commands.push(cmd);
                    }
                }
                "PersistentVolumeClaim" => {
                    if let Some(cmd) = targets.persistent_volume_claims.begin_refresh() {
                        refresh_commands.push(cmd);
                    }
                }
                "Node" => {
                    if let Some(cmd) = targets.nodes.begin_refresh() {
                        refresh_commands.push(cmd);
                    }
                }
                "Event" => {
                    if let Some(cmd) = targets.events.begin_refresh() {
                        refresh_commands.push(cmd);
                    }
                }
                _ => {}
            }
            Some((toast_id, ActionToastKind::Success, refresh_commands))
        }
        ClusterBridgeEvent::WorkloadActionFailed {
            cluster_id,
            action,
            kind,
            message,
            ..
        } => {
            if active_cluster_id != Some(cluster_id.as_str()) {
                return None;
            }
            if modal.is_open() {
                modal.set_error(message.clone());
            }
            let title = match action {
                WorkloadActionKind::Scale => format!("Scale {kind} Failed"),
                WorkloadActionKind::Restart => format!("Restart {kind} Failed"),
                WorkloadActionKind::RestartPod => "Restart Pod Failed".to_string(),
                WorkloadActionKind::TriggerCronJob => "Trigger CronJob Failed".to_string(),
                WorkloadActionKind::Delete => format!("Delete {kind} Failed"),
            };
            let toast_id = toast.show_error(title, message.clone());
            Some((toast_id, ActionToastKind::Error, Vec::new()))
        }
        _ => None,
    }
}

pub fn apply_drawer_lifecycle_for_event(
    drawer: &mut ResourceDrawerState,
    event: &ClusterBridgeEvent,
) {
    match event {
        ClusterBridgeEvent::ClusterConnected { .. }
        | ClusterBridgeEvent::ClusterUnreachable { .. }
        | ClusterBridgeEvent::ConnectionError { .. }
        | ClusterBridgeEvent::ClusterDisconnected { .. }
        | ClusterBridgeEvent::ClusterAdded {
            switch_outcome: Some(_),
            ..
        } => {
            drawer.reset_for_context_change();
        }
        ClusterBridgeEvent::ResourceYamlLoaded {
            cluster_id,
            generation,
            kind,
            namespace,
            name,
            yaml,
        } => {
            drawer.apply_yaml_loaded(
                cluster_id,
                *generation,
                kind,
                namespace.as_deref(),
                name,
                yaml.clone(),
            );
        }
        ClusterBridgeEvent::ResourceYamlLoadFailed {
            cluster_id,
            generation,
            kind,
            namespace,
            name,
            message,
        } => {
            drawer.apply_yaml_failed(
                cluster_id,
                *generation,
                kind,
                namespace.as_deref(),
                name,
                message.clone(),
            );
        }
        _ => {}
    }
}

pub const ORBIT_GITHUB_REPO_URL: &str = "https://github.com/vantoan1511/orbit";

fn notification_from_bridge_event(
    event: &ClusterBridgeEvent,
) -> Option<(NotificationSeverity, String, String)> {
    match event {
        ClusterBridgeEvent::WorkloadActionSucceeded {
            kind,
            name,
            message,
            ..
        } => Some((
            NotificationSeverity::Success,
            format!("{kind} {name}"),
            message.clone(),
        )),
        ClusterBridgeEvent::WorkloadActionFailed {
            kind,
            name,
            message,
            ..
        } => Some((
            NotificationSeverity::Error,
            format!("Action failed: {kind} {name}"),
            message.clone(),
        )),
        ClusterBridgeEvent::PortForwardFailed { message, .. } => Some((
            NotificationSeverity::Error,
            "Port Forward Failed".to_string(),
            message.clone(),
        )),
        ClusterBridgeEvent::SettingsSaveFailed { message } => Some((
            NotificationSeverity::Error,
            "Settings Save Failed".to_string(),
            message.clone(),
        )),
        ClusterBridgeEvent::ConnectionError {
            cluster_id,
            message,
            ..
        } => Some((
            NotificationSeverity::Error,
            cluster_id
                .as_deref()
                .map(|id| format!("Connection Error ({id})"))
                .unwrap_or_else(|| "Connection Error".to_string()),
            message.clone(),
        )),
        ClusterBridgeEvent::ClusterUnreachable { outcome, .. } => Some((
            NotificationSeverity::Warning,
            format!("Cluster Unreachable ({})", outcome.context_name),
            outcome
                .error_message
                .clone()
                .unwrap_or_else(|| "Cluster did not respond.".to_string()),
        )),
        ClusterBridgeEvent::AddClusterFailed { message } => Some((
            NotificationSeverity::Error,
            "Add Cluster Failed".to_string(),
            message.clone(),
        )),
        _ => None,
    }
}

pub struct OrbitShell {
    state: ClusterSwitcherState,
    sidebar: SidebarLayoutState,
    overview: ClusterOverviewState,
    workloads_overview: WorkloadsOverviewState,
    namespaces: NamespaceFilterState,
    namespaces_view: NamespacesViewState,
    deployments: DeploymentsState,
    statefulsets: StatefulSetsState,
    daemonsets: DaemonSetsState,
    replicasets: ReplicaSetsState,
    jobs: JobsState,
    cronjobs: CronJobsState,
    pods: PodsState,
    services: ServicesState,
    ingresses: IngressesState,
    port_forwards: PortForwardsState,
    configmaps: ConfigMapsState,
    secrets: SecretsState,
    persistent_volumes: PersistentVolumesState,
    persistent_volume_claims: PersistentVolumeClaimsState,
    network_policies: NetworkPoliciesState,
    storage_overview: StorageOverviewState,
    pod_security: PodSecurityState,
    resource_quotas: ResourceQuotasState,
    list_views: ClusterListViews,
    nodes: NodesState,
    events: EventsState,
    drawer: ResourceDrawerState,
    tabs: WorkspaceTabsState,
    log_tabs: HashMap<LogsTabTarget, views::logs::LogViewerState>,
    next_log_stream_id: u64,
    focus_handle: FocusHandle,
    namespace_search_focus: FocusHandle,
    picker_search_focus: FocusHandle,
    log_search_focus: FocusHandle,
    command_palette: CommandPaletteState,
    command_palette_focus: FocusHandle,
    command_tx: mpsc::UnboundedSender<ClusterBridgeCommand>,
    enable_experience_features: bool,
    resolved_system_is_dark: bool,
    settings_error: Option<String>,
    settings_dropdown: views::settings::SettingsDropdownState,
    context_menu: ContextMenuState,
    add_cluster: AddClusterModalState,
    workload_action_modal: WorkloadActionModalState,
    port_forward_modal: Option<PortForwardModalState>,
    action_toast: ActionToastState,
    active_shell_drawer: ActiveShellDrawer,
    notifications: NotificationCenterState,
    table_preferences: orbit_core::config::PersistedTablePreferences,
}

impl OrbitShell {
    pub const WINDOW_TITLE: &'static str = "Orbit";
    pub const DEFAULT_WIDTH: f32 = 1200.0;
    pub const DEFAULT_HEIGHT: f32 = 800.0;
    pub const TITLEBAR_HEIGHT: f32 = crate::components::TITLEBAR_HEIGHT;
    pub const HEADER_HEIGHT: f32 = 48.0;
    pub const FOOTER_HEIGHT: f32 = 26.0;

    pub const fn header_dropdown_top_offset() -> f32 {
        Self::TITLEBAR_HEIGHT + Self::HEADER_HEIGHT + 4.0
    }

    pub const fn tab_picker_top_offset() -> f32 {
        Self::TITLEBAR_HEIGHT + Self::HEADER_HEIGHT + crate::components::TAB_BAR_HEIGHT + 4.0
    }

    pub fn new(config: &OrbitConfig, window: &mut Window, cx: &mut Context<Self>) -> Self {
        let ClusterBridge {
            command_tx,
            mut event_rx,
        } = start_cluster_bridge();

        cx.spawn(async move |this, cx| {
            while let Some(event) = event_rx.recv().await {
                if this
                    .update(cx, |shell, cx| {
                        if let ClusterBridgeEvent::SettingsSaveFailed { ref message } = event {
                            shell.settings_error = Some(message.clone());
                        }
                        if let Some((severity, title, message)) =
                            notification_from_bridge_event(&event)
                        {
                            shell.record_notification(severity, title, message);
                        }
                        apply_add_cluster_modal_event(&mut shell.add_cluster, &event);
                        apply_drawer_lifecycle_for_event(&mut shell.drawer, &event);
                        if let Some((toast_id, toast_kind, refresh_cmds)) =
                            apply_workload_action_event(
                                shell.state.active_cluster_id.as_deref(),
                                &mut shell.workload_action_modal,
                                &mut shell.action_toast,
                                WorkloadActionRefreshTargets {
                                    drawer: &mut shell.drawer,
                                    deployments: &mut shell.deployments,
                                    statefulsets: &mut shell.statefulsets,
                                    daemonsets: &mut shell.daemonsets,
                                    replicasets: &mut shell.replicasets,
                                    pods: &mut shell.pods,
                                    cronjobs: &mut shell.cronjobs,
                                    jobs: &mut shell.jobs,
                                    services: &mut shell.services,
                                    ingresses: &mut shell.ingresses,
                                    configmaps: &mut shell.configmaps,
                                    secrets: &mut shell.secrets,
                                    network_policies: &mut shell.network_policies,
                                    resource_quotas: &mut shell.resource_quotas,
                                    namespaces: &mut shell.namespaces,
                                    persistent_volumes: &mut shell.persistent_volumes,
                                    persistent_volume_claims: &mut shell.persistent_volume_claims,
                                    nodes: &mut shell.nodes,
                                    events: &mut shell.events,
                                },
                                &event,
                            )
                        {
                            for cmd in refresh_cmds {
                                shell.dispatch_command(cmd);
                            }
                            shell.schedule_toast_dismiss(toast_id, toast_kind, cx);
                        }
                        if matches!(
                            event,
                            ClusterBridgeEvent::ClusterConnected { .. }
                                | ClusterBridgeEvent::ClusterDisconnected { .. }
                                | ClusterBridgeEvent::ClusterUnreachable { .. }
                        ) {
                            let stop_cmds =
                                clear_all_log_tabs_in_state(&mut shell.tabs, &mut shell.log_tabs);
                            for cmd in stop_cmds {
                                shell.dispatch_command(cmd);
                            }
                        }
                        apply_log_bridge_event(
                            &mut shell.log_tabs,
                            shell.state.active_cluster_id.as_deref(),
                            &event,
                        );
                        if let Some((pf_cmds, toast_info)) = apply_port_forward_event(
                            &mut shell.port_forwards,
                            &mut shell.port_forward_modal,
                            &mut shell.action_toast,
                            &event,
                        ) {
                            for cmd in pf_cmds {
                                shell.dispatch_command(cmd);
                            }
                            if let Some((toast_id, toast_kind)) = toast_info {
                                shell.schedule_toast_dismiss(toast_id, toast_kind, cx);
                            }
                        }
                        let is_cluster_connected =
                            matches!(event, ClusterBridgeEvent::ClusterConnected { .. });
                        for cmd in apply_cluster_event_with_tabs(
                            &mut shell.state,
                            &mut shell.sidebar,
                            &mut shell.overview,
                            &mut shell.workloads_overview,
                            &mut shell.namespaces,
                            &mut shell.namespaces_view,
                            &mut shell.deployments,
                            &mut shell.statefulsets,
                            &mut shell.daemonsets,
                            &mut shell.replicasets,
                            &mut shell.jobs,
                            &mut shell.cronjobs,
                            &mut shell.pods,
                            &mut shell.services,
                            &mut shell.ingresses,
                            &mut shell.configmaps,
                            &mut shell.secrets,
                            &mut shell.persistent_volumes,
                            &mut shell.persistent_volume_claims,
                            &mut shell.network_policies,
                            &mut shell.storage_overview,
                            &mut shell.pod_security,
                            &mut shell.resource_quotas,
                            &mut shell.list_views,
                            &mut shell.nodes,
                            &mut shell.events,
                            &mut shell.tabs,
                            event,
                        ) {
                            shell.dispatch_command(cmd);
                        }
                        if is_cluster_connected {
                            shell.hydrate_exemplar_tables();
                        }
                        shell.reconcile_drawer_target_exists();
                        cx.notify();
                    })
                    .is_err()
                {
                    break;
                }
            }
        })
        .detach();

        let appearance_entity = cx.entity().clone();
        window
            .observe_window_appearance(move |window, cx| {
                let appearance = window.appearance();
                let is_system_dark = ThemeMode::System.resolve_is_dark(appearance);
                appearance_entity.update(cx, |shell, _cx| {
                    shell.resolved_system_is_dark = is_system_dark;
                });
                let current = OrbitTheme::get(cx);
                if current.mode == ThemeMode::System {
                    let updated = current.with_window_appearance(appearance);
                    apply_theme(updated, window, cx);
                    window.refresh();
                }
                window.defer(cx, |window, cx| {
                    let theme = OrbitTheme::get(cx).clone();
                    sync_native_titlebar_appearance(window, &theme);
                });
            })
            .detach();

        let resolved_system_is_dark = ThemeMode::System.resolve_is_dark(window.appearance());

        let focus_handle = cx.focus_handle();
        focus_handle.focus(window);

        let mut shell = Self {
            state: ClusterSwitcherState::new(),
            sidebar: SidebarLayoutState::new(),
            overview: ClusterOverviewState::default(),
            workloads_overview: WorkloadsOverviewState::default(),
            namespaces: NamespaceFilterState::from_config(config),
            namespaces_view: NamespacesViewState::default(),
            deployments: DeploymentsState::default(),
            statefulsets: StatefulSetsState::default(),
            daemonsets: DaemonSetsState::default(),
            replicasets: ReplicaSetsState::default(),
            jobs: JobsState::default(),
            cronjobs: CronJobsState::default(),
            pods: PodsState::default(),
            services: ServicesState::default(),
            ingresses: IngressesState::default(),
            port_forwards: PortForwardsState::default(),
            configmaps: ConfigMapsState::default(),
            secrets: SecretsState::default(),
            persistent_volumes: PersistentVolumesState::default(),
            persistent_volume_claims: PersistentVolumeClaimsState::default(),
            network_policies: NetworkPoliciesState::default(),
            storage_overview: StorageOverviewState::default(),
            pod_security: PodSecurityState::default(),
            resource_quotas: ResourceQuotasState::default(),
            list_views: ClusterListViews::default(),
            nodes: NodesState::default(),
            events: EventsState::default(),
            drawer: ResourceDrawerState::default(),
            tabs: WorkspaceTabsState::from_config(config),
            log_tabs: HashMap::new(),
            next_log_stream_id: 1,
            focus_handle,
            namespace_search_focus: cx.focus_handle(),
            picker_search_focus: cx.focus_handle(),
            log_search_focus: cx.focus_handle(),
            command_palette: CommandPaletteState::default(),
            command_palette_focus: cx.focus_handle(),
            command_tx,
            enable_experience_features: config.enable_experience_features(),
            resolved_system_is_dark,
            settings_error: None,
            settings_dropdown: views::settings::SettingsDropdownState::default(),
            context_menu: ContextMenuState::new(),
            add_cluster: AddClusterModalState::default(),
            workload_action_modal: WorkloadActionModalState::default(),
            port_forward_modal: None,
            action_toast: ActionToastState::default(),
            active_shell_drawer: ActiveShellDrawer::default(),
            notifications: NotificationCenterState::default(),
            table_preferences: config.table_preferences(),
        };
        shell.hydrate_exemplar_tables();
        shell
    }

    fn open_add_cluster_modal(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.state.close_dropdown();
        self.namespaces.close_dropdown();
        self.settings_dropdown.close();
        self.context_menu.close();
        self.add_cluster.open(&self.state.clusters);
        self.focus_handle.focus(window);
        cx.notify();
    }

    fn browse_kubeconfig_file(&mut self, _window: &mut Window, cx: &mut Context<Self>) {
        let paths_rx = cx.prompt_for_paths(PathPromptOptions {
            files: true,
            directories: false,
            multiple: false,
            prompt: Some("Select Kubeconfig File".into()),
        });
        cx.spawn(async move |this, cx| {
            let Ok(Ok(Some(paths))) = paths_rx.await else {
                return;
            };
            let Some(path) = paths.into_iter().next() else {
                return;
            };
            let path_str = path.to_string_lossy().to_string();
            let read_result = cx
                .background_executor()
                .spawn(async move {
                    std::fs::read_to_string(&path)
                        .map_err(|e| format!("Failed to read kubeconfig file: {e}"))
                })
                .await;
            let _ = this.update(cx, |shell, cx| {
                shell.add_cluster.set_file_loaded(path_str, read_result);
                cx.notify();
            });
        })
        .detach();
    }

    fn paste_into_add_cluster(&mut self, cx: &mut Context<Self>, target_yaml_tab: bool) {
        let Some(text) = cx.read_from_clipboard().and_then(|item| item.text()) else {
            return;
        };
        if target_yaml_tab {
            self.add_cluster.set_yaml_content(text);
        } else {
            self.add_cluster.append_text_to_focused_field(&text);
        }
        cx.notify();
    }

    fn submit_add_cluster(&mut self, cx: &mut Context<Self>) {
        if self.add_cluster.is_submitting {
            return;
        }
        match self.add_cluster.build_request() {
            Ok(req) => {
                self.add_cluster.is_submitting = true;
                self.add_cluster.submit_error = None;
                self.dispatch_command(ClusterBridgeCommand::AddCluster(req));
                cx.notify();
            }
            Err(err) => {
                self.add_cluster.submit_error = Some(err);
                cx.notify();
            }
        }
    }

    fn render_add_cluster_modal_overlay(&self, cx: &mut Context<Self>) -> Option<AnyElement> {
        if !self.add_cluster.is_open {
            return None;
        }
        let theme = OrbitTheme::get(cx).clone();
        let entity = cx.entity().clone();

        let callbacks = AddClusterModalCallbacks {
            on_close: {
                let e = entity.clone();
                std::rc::Rc::new(move |_, cx| {
                    e.update(cx, |shell, cx| {
                        shell.add_cluster.close();
                        cx.notify();
                    });
                })
            },
            on_select_tab: {
                let e = entity.clone();
                std::rc::Rc::new(move |tab, _, cx| {
                    e.update(cx, |shell, cx| {
                        shell.add_cluster.set_tab(tab);
                        cx.notify();
                    });
                })
            },
            on_browse_file: {
                let e = entity.clone();
                std::rc::Rc::new(move |window, cx| {
                    e.update(cx, |shell, cx| {
                        shell.browse_kubeconfig_file(window, cx);
                    });
                })
            },
            on_paste_yaml: {
                let e = entity.clone();
                std::rc::Rc::new(move |_, cx| {
                    e.update(cx, |shell, cx| {
                        shell.paste_into_add_cluster(cx, true);
                    });
                })
            },
            on_clear_yaml: {
                let e = entity.clone();
                std::rc::Rc::new(move |_, cx| {
                    e.update(cx, |shell, cx| {
                        shell.add_cluster.set_yaml_content(String::new());
                        cx.notify();
                    });
                })
            },
            on_focus_field: {
                let e = entity.clone();
                std::rc::Rc::new(move |field, window, cx| {
                    e.update(cx, |shell, cx| {
                        shell.add_cluster.focused_field = Some(field);
                        shell.focus_handle.focus(window);
                        cx.notify();
                    });
                })
            },
            on_select_auth_type: {
                let e = entity.clone();
                std::rc::Rc::new(move |auth, _, cx| {
                    e.update(cx, |shell, cx| {
                        shell.add_cluster.set_manual_auth_type(auth);
                        cx.notify();
                    });
                })
            },
            on_toggle_insecure_tls: {
                let e = entity.clone();
                std::rc::Rc::new(move |_, cx| {
                    e.update(cx, |shell, cx| {
                        shell.add_cluster.toggle_insecure_skip_tls();
                        cx.notify();
                    });
                })
            },
            on_toggle_context: {
                let e = entity.clone();
                std::rc::Rc::new(move |name, _, cx| {
                    e.update(cx, |shell, cx| {
                        shell.add_cluster.toggle_context(&name);
                        cx.notify();
                    });
                })
            },
            on_select_all_contexts: {
                let e = entity.clone();
                std::rc::Rc::new(move |_, cx| {
                    e.update(cx, |shell, cx| {
                        shell.add_cluster.select_all_contexts();
                        cx.notify();
                    });
                })
            },
            on_deselect_all_contexts: {
                let e = entity.clone();
                std::rc::Rc::new(move |_, cx| {
                    e.update(cx, |shell, cx| {
                        shell.add_cluster.deselect_all_contexts();
                        cx.notify();
                    });
                })
            },
            on_toggle_customization: {
                let e = entity.clone();
                std::rc::Rc::new(move |_, cx| {
                    e.update(cx, |shell, cx| {
                        shell.add_cluster.customization_expanded =
                            !shell.add_cluster.customization_expanded;
                        cx.notify();
                    });
                })
            },
            on_select_color: {
                let e = entity.clone();
                std::rc::Rc::new(move |color, _, cx| {
                    e.update(cx, |shell, cx| {
                        shell.add_cluster.color = color;
                        cx.notify();
                    });
                })
            },
            on_add_tag: {
                let e = entity.clone();
                std::rc::Rc::new(move |_, cx| {
                    e.update(cx, |shell, cx| {
                        shell.add_cluster.commit_tag_input();
                        cx.notify();
                    });
                })
            },
            on_remove_tag: {
                let e = entity.clone();
                std::rc::Rc::new(move |tag, _, cx| {
                    e.update(cx, |shell, cx| {
                        shell.add_cluster.remove_tag(&tag);
                        cx.notify();
                    });
                })
            },
            on_toggle_switch_context: {
                let e = entity.clone();
                std::rc::Rc::new(move |_, cx| {
                    e.update(cx, |shell, cx| {
                        shell.add_cluster.switch_context = !shell.add_cluster.switch_context;
                        cx.notify();
                    });
                })
            },
            on_submit: {
                let e = entity.clone();
                std::rc::Rc::new(move |_, cx| {
                    e.update(cx, |shell, cx| {
                        shell.submit_add_cluster(cx);
                    });
                })
            },
        };

        Some(render_add_cluster_modal(
            &self.add_cluster,
            &theme,
            &callbacks,
        ))
    }

    fn active_cluster_context_and_name(&self) -> Option<(String, String)> {
        let cluster_id = self.state.active_cluster_id.clone()?;
        let display_name = self.state.cluster_display_name(&cluster_id);
        Some((cluster_id, display_name))
    }

    fn open_scale_modal(
        &mut self,
        kind: &str,
        namespace: &str,
        name: &str,
        current_replicas: i32,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let Some((_, cluster_name)) = self.active_cluster_context_and_name() else {
            return;
        };
        self.state.close_dropdown();
        self.namespaces.close_dropdown();
        self.context_menu.close();
        self.workload_action_modal.open_scale(
            cluster_name,
            kind,
            namespace,
            name,
            current_replicas,
        );
        self.focus_handle.focus(window);
        cx.notify();
    }

    fn open_delete_confirm_modal(
        &mut self,
        kind: &str,
        namespace: Option<&str>,
        name: &str,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let Some((_, cluster_name)) = self.active_cluster_context_and_name() else {
            return;
        };
        self.state.close_dropdown();
        self.namespaces.close_dropdown();
        self.context_menu.close();
        self.workload_action_modal.open_confirm(
            cluster_name,
            WorkloadActionKind::Delete,
            kind,
            namespace.unwrap_or_default(),
            name,
        );
        self.focus_handle.focus(window);
        cx.notify();
    }

    fn open_restart_confirm_modal(
        &mut self,
        action: WorkloadActionKind,
        kind: &str,
        namespace: &str,
        name: &str,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let Some((_, cluster_name)) = self.active_cluster_context_and_name() else {
            return;
        };
        self.state.close_dropdown();
        self.namespaces.close_dropdown();
        self.context_menu.close();
        self.workload_action_modal
            .open_confirm(cluster_name, action, kind, namespace, name);
        self.focus_handle.focus(window);
        cx.notify();
    }

    fn trigger_cronjob_now(&mut self, namespace: &str, name: &str, cx: &mut Context<Self>) {
        let Some(cluster_id) = self.state.active_cluster_id.clone() else {
            return;
        };
        self.dispatch_command(ClusterBridgeCommand::TriggerCronJob {
            cluster_id,
            namespace: namespace.to_string(),
            name: name.to_string(),
        });
        cx.notify();
    }

    fn submit_workload_action_modal(&mut self, cx: &mut Context<Self>) {
        let Some(cluster_id) = self.state.active_cluster_id.clone() else {
            return;
        };
        let cmd = match &self.workload_action_modal.dialog {
            WorkloadActionDialogState::Closed => return,
            WorkloadActionDialogState::Scale(scale) => {
                if !scale.can_submit() {
                    return;
                }
                let Some(replicas) = scale.parsed_replicas() else {
                    return;
                };
                ClusterBridgeCommand::ScaleWorkload {
                    cluster_id,
                    namespace: scale.namespace.clone(),
                    kind: scale.kind.clone(),
                    name: scale.name.clone(),
                    replicas,
                }
            }
            WorkloadActionDialogState::Confirm(confirm) => {
                if !confirm.can_submit() {
                    return;
                }
                let spec = &confirm.dialog.spec;
                match confirm.action {
                    WorkloadActionKind::Restart => ClusterBridgeCommand::RestartWorkload {
                        cluster_id,
                        namespace: spec.namespace.clone(),
                        kind: spec.kind.clone(),
                        name: spec.name.clone(),
                    },
                    WorkloadActionKind::RestartPod => ClusterBridgeCommand::RestartPod {
                        cluster_id,
                        namespace: spec.namespace.clone(),
                        name: spec.name.clone(),
                    },
                    WorkloadActionKind::Delete => ClusterBridgeCommand::DeleteResource {
                        cluster_id,
                        namespace: (!spec.namespace.trim().is_empty())
                            .then(|| spec.namespace.clone()),
                        kind: spec.kind.clone(),
                        name: spec.name.clone(),
                    },
                    _ => return,
                }
            }
        };
        self.workload_action_modal.set_submitting(true);
        self.dispatch_command(cmd);
        cx.notify();
    }

    fn schedule_toast_dismiss(&self, toast_id: u64, kind: ActionToastKind, cx: &mut Context<Self>) {
        let secs = match kind {
            ActionToastKind::Success => 4,
            ActionToastKind::Error => 7,
        };
        cx.spawn(async move |this, cx| {
            cx.background_executor()
                .timer(std::time::Duration::from_secs(secs))
                .await;
            let _ = this.update(cx, |shell, cx| {
                if shell.action_toast.dismiss_if_id(toast_id) {
                    cx.notify();
                }
            });
        })
        .detach();
    }

    fn render_workload_action_modal_overlay(&self, cx: &mut Context<Self>) -> Option<AnyElement> {
        if !self.workload_action_modal.is_open() {
            return None;
        }
        let theme = OrbitTheme::get(cx).clone();
        let entity = cx.entity().clone();
        let callbacks = WorkloadActionModalCallbacks {
            on_close: {
                let e = entity.clone();
                std::rc::Rc::new(move |_, cx| {
                    e.update(cx, |shell, cx| {
                        let is_submitting = match &shell.workload_action_modal.dialog {
                            WorkloadActionDialogState::Scale(s) => s.submitting,
                            WorkloadActionDialogState::Confirm(c) => c.dialog.submitting,
                            WorkloadActionDialogState::Closed => false,
                        };
                        if !is_submitting {
                            shell.workload_action_modal.close();
                            cx.notify();
                        }
                    });
                })
            },
            on_submit: {
                let e = entity.clone();
                std::rc::Rc::new(move |_, cx| {
                    e.update(cx, |shell, cx| {
                        shell.submit_workload_action_modal(cx);
                    });
                })
            },
            on_decrement: {
                let e = entity.clone();
                std::rc::Rc::new(move |_, cx| {
                    e.update(cx, |shell, cx| {
                        if let WorkloadActionDialogState::Scale(scale) =
                            &mut shell.workload_action_modal.dialog
                        {
                            scale.decrement();
                            cx.notify();
                        }
                    });
                })
            },
            on_increment: {
                let e = entity.clone();
                std::rc::Rc::new(move |_, cx| {
                    e.update(cx, |shell, cx| {
                        if let WorkloadActionDialogState::Scale(scale) =
                            &mut shell.workload_action_modal.dialog
                        {
                            scale.increment();
                            cx.notify();
                        }
                    });
                })
            },
        };
        render_workload_action_modal(&self.workload_action_modal, &theme, &callbacks)
    }

    fn open_port_forward_target_modal(
        &mut self,
        namespace: &str,
        kind: &str,
        name: &str,
        detected_ports: Vec<u16>,
        cx: &mut Context<Self>,
    ) {
        let ClusterConnectionState::Connected { cluster_id, .. } = &self.state.connection_state
        else {
            return;
        };
        let cluster_id = cluster_id.clone();
        let cluster_name = self.state.cluster_display_name(&cluster_id);
        let active_local_ports: Vec<u16> = self
            .port_forwards
            .port_forwards
            .iter()
            .map(|pf| pf.local_port)
            .collect();
        self.state.close_dropdown();
        self.namespaces.close_dropdown();
        self.context_menu.close();
        self.port_forward_modal = Some(PortForwardModalState::new_targeted(
            cluster_id,
            cluster_name,
            namespace,
            kind,
            name,
            detected_ports,
            active_local_ports,
        ));
        cx.notify();
    }

    fn open_port_forward_create_modal(&mut self, cx: &mut Context<Self>) {
        let ClusterConnectionState::Connected { cluster_id, .. } = &self.state.connection_state
        else {
            return;
        };
        let cluster_id = cluster_id.clone();
        let cluster_name = self.state.cluster_display_name(&cluster_id);
        if self.namespaces.needs_load_for_cluster(&cluster_id) {
            let cmd = self.namespaces.begin_load_for_cluster(&cluster_id);
            self.dispatch_command(cmd);
        }
        if self.services.needs_load_for_cluster(&cluster_id) {
            let cmd = self.services.begin_load_for_cluster(&cluster_id);
            self.dispatch_command(cmd);
        }
        if self.pods.needs_load_for_cluster(&cluster_id) {
            let cmd = self.pods.begin_load_for_cluster(&cluster_id);
            self.dispatch_command(cmd);
        }
        if self.deployments.needs_load_for_cluster(&cluster_id) {
            let cmd = self.deployments.begin_load_for_cluster(&cluster_id);
            self.dispatch_command(cmd);
        }
        if self.statefulsets.needs_load_for_cluster(&cluster_id) {
            let cmd = self.statefulsets.begin_load_for_cluster(&cluster_id);
            self.dispatch_command(cmd);
        }
        if self.replicasets.needs_load_for_cluster(&cluster_id) {
            let cmd = self.replicasets.begin_load_for_cluster(&cluster_id);
            self.dispatch_command(cmd);
        }

        let available_namespaces: Vec<String> = self
            .namespaces
            .namespaces
            .iter()
            .map(|ns| ns.name.clone())
            .collect();
        let mut resources = Vec::new();
        for svc in &self.services.services {
            resources.push(PortForwardResourceOption {
                namespace: svc.namespace.clone(),
                kind: "Service".to_string(),
                name: svc.name.clone(),
                ports: extract_ports_from_service(svc),
            });
        }
        for pod in &self.pods.pods {
            resources.push(PortForwardResourceOption {
                namespace: pod.namespace.clone(),
                kind: "Pod".to_string(),
                name: pod.name.clone(),
                ports: extract_ports_from_pod(pod),
            });
        }
        for dep in &self.deployments.deployments {
            resources.push(PortForwardResourceOption {
                namespace: dep.namespace.clone(),
                kind: "Deployment".to_string(),
                name: dep.name.clone(),
                ports: extract_ports_from_deployment(dep),
            });
        }
        for ss in &self.statefulsets.statefulsets {
            resources.push(PortForwardResourceOption {
                namespace: ss.namespace.clone(),
                kind: "StatefulSet".to_string(),
                name: ss.name.clone(),
                ports: extract_ports_from_statefulset(ss),
            });
        }
        for rs in &self.replicasets.replicasets {
            resources.push(PortForwardResourceOption {
                namespace: rs.namespace.clone(),
                kind: "ReplicaSet".to_string(),
                name: rs.name.clone(),
                ports: extract_ports_from_replicaset(rs),
            });
        }
        let active_local_ports: Vec<u16> = self
            .port_forwards
            .port_forwards
            .iter()
            .map(|pf| pf.local_port)
            .collect();

        self.state.close_dropdown();
        self.namespaces.close_dropdown();
        self.context_menu.close();
        self.port_forward_modal = Some(PortForwardModalState::new_create(
            cluster_id,
            cluster_name,
            available_namespaces,
            resources,
            active_local_ports,
        ));
        cx.notify();
    }

    fn submit_port_forward_modal(&mut self, cx: &mut Context<Self>) {
        let Some(modal) = self.port_forward_modal.as_mut() else {
            return;
        };
        if !modal.can_submit() {
            return;
        }
        let Some(local_port) = modal.parsed_local_port() else {
            return;
        };
        let Some(remote_port) = modal.parsed_remote_port() else {
            return;
        };
        modal.submitting = true;
        modal.error = None;
        let cmd = ClusterBridgeCommand::StartPortForward {
            cluster_id: modal.cluster_id.clone(),
            namespace: modal.namespace.trim().to_string(),
            kind: modal.kind.clone(),
            name: modal.name.trim().to_string(),
            local_port,
            remote_port,
        };
        self.dispatch_command(cmd);
        cx.notify();
    }

    fn open_port_forward_in_browser(&mut self, local_port: u16, cx: &mut App) {
        let url = views::port_forward::format_port_forward_url(local_port);
        cx.open_url(&url);
    }

    fn copy_port_forward_url(&mut self, local_port: u16, cx: &mut Context<Self>) {
        let url = views::port_forward::format_port_forward_url(local_port);
        cx.write_to_clipboard(ClipboardItem::new_string(url.clone()));
        let toast_id = self
            .action_toast
            .show_success("Copied URL", format!("Copied {url} to clipboard"));
        self.schedule_toast_dismiss(toast_id, ActionToastKind::Success, cx);
        cx.notify();
    }

    fn stop_port_forward(&mut self, id: String, cx: &mut Context<Self>) {
        let Some(cluster_id) = self.state.active_cluster_id.clone() else {
            return;
        };
        self.port_forwards.mark_stopping(&id);
        self.dispatch_command(ClusterBridgeCommand::StopPortForward { cluster_id, id });
        cx.notify();
    }

    fn stop_selected_port_forwards(&mut self, cx: &mut Context<Self>) {
        let Some(cluster_id) = self.state.active_cluster_id.clone() else {
            return;
        };
        let ids: Vec<String> = self.port_forwards.selected_ids.iter().cloned().collect();
        if ids.is_empty() {
            return;
        }
        for id in &ids {
            self.port_forwards.mark_stopping(id);
        }
        self.dispatch_command(ClusterBridgeCommand::StopSelectedPortForwards { cluster_id, ids });
        cx.notify();
    }

    fn stop_all_port_forwards(&mut self, cx: &mut Context<Self>) {
        let Some(cluster_id) = self.state.active_cluster_id.clone() else {
            return;
        };
        let ids: Vec<String> = self
            .port_forwards
            .port_forwards
            .iter()
            .map(|pf| pf.id.clone())
            .collect();
        for id in ids {
            self.port_forwards.mark_stopping(&id);
        }
        self.dispatch_command(ClusterBridgeCommand::StopAllPortForwards { cluster_id });
        cx.notify();
    }

    fn render_port_forward_modal_overlay(&self, cx: &mut Context<Self>) -> Option<AnyElement> {
        let modal = self.port_forward_modal.as_ref()?;
        let theme = OrbitTheme::get(cx).clone();
        let close_entity = cx.entity().clone();
        let submit_entity = cx.entity().clone();
        let focus_entity = cx.entity().clone();
        let port_entity = cx.entity().clone();
        let kind_entity = cx.entity().clone();
        let ns_entity = cx.entity().clone();
        let res_entity = cx.entity().clone();

        let callbacks = PortForwardModalCallbacks {
            on_close: std::rc::Rc::new(move |_, cx| {
                close_entity.update(cx, |shell, cx| {
                    if !shell
                        .port_forward_modal
                        .as_ref()
                        .is_some_and(|m| m.submitting)
                    {
                        shell.port_forward_modal = None;
                        cx.notify();
                    }
                });
            }),
            on_submit: std::rc::Rc::new(move |_, cx| {
                submit_entity.update(cx, |shell, cx| {
                    shell.submit_port_forward_modal(cx);
                });
            }),
            on_focus_field: std::rc::Rc::new(move |field, _, cx| {
                focus_entity.update(cx, |shell, cx| {
                    if let Some(m) = shell.port_forward_modal.as_mut() {
                        m.active_field = field;
                        cx.notify();
                    }
                });
            }),
            on_select_detected_port: std::rc::Rc::new(move |port, _, cx| {
                port_entity.update(cx, |shell, cx| {
                    if let Some(m) = shell.port_forward_modal.as_mut() {
                        m.select_detected_port(port);
                        cx.notify();
                    }
                });
            }),
            on_select_kind: std::rc::Rc::new(move |kind, _, cx| {
                kind_entity.update(cx, |shell, cx| {
                    if let Some(m) = shell.port_forward_modal.as_mut() {
                        m.set_kind(&kind);
                        cx.notify();
                    }
                });
            }),
            on_select_namespace: std::rc::Rc::new(move |ns, _, cx| {
                ns_entity.update(cx, |shell, cx| {
                    if let Some(m) = shell.port_forward_modal.as_mut() {
                        m.set_namespace(&ns);
                        cx.notify();
                    }
                });
            }),
            on_select_resource: std::rc::Rc::new(move |name, _, cx| {
                res_entity.update(cx, |shell, cx| {
                    if let Some(m) = shell.port_forward_modal.as_mut() {
                        m.select_resource(&name);
                        cx.notify();
                    }
                });
            }),
        };

        Some(render_port_forward_modal(modal, &theme, &callbacks))
    }

    fn render_action_toast_overlay_view(&self, cx: &mut Context<Self>) -> Option<AnyElement> {
        self.action_toast.current()?;
        let theme = OrbitTheme::get(cx).clone();
        let entity = cx.entity().clone();
        render_action_toast_overlay(
            &self.action_toast,
            &theme,
            std::rc::Rc::new(move |_, cx| {
                entity.update(cx, |shell, cx| {
                    shell.action_toast.dismiss();
                    cx.notify();
                });
            }),
        )
    }

    fn open_context_menu(
        &mut self,
        anchor: Point<Pixels>,
        entries: Vec<ContextMenuEntry>,
        cx: &mut Context<Self>,
    ) {
        self.state.close_dropdown();
        self.namespaces.close_dropdown();
        self.settings_dropdown.close();
        self.context_menu.open(anchor, entries);
        cx.notify();
    }

    fn dismiss_context_menu(&mut self) {
        self.context_menu.close();
    }

    fn filter_by_namespace(&mut self, ns: &str, cx: &mut Context<Self>) {
        if ns.is_empty() {
            return;
        }
        self.namespaces.select_all_namespaces();
        if let Some(updated) = self.namespaces.toggle_namespace(ns)
            && let Some(cluster_id) = self.namespaces.cluster_id.clone()
        {
            self.dispatch_command(ClusterBridgeCommand::SaveNamespaceSelection {
                cluster_id,
                namespaces: updated,
            });
        }
        self.reset_exemplar_pages();
        cx.notify();
    }

    fn hydrate_exemplar_tables(&mut self) {
        let cluster_id = self.state.active_cluster_id.as_deref();
        self.pods.hydrate_table_preferences(
            cluster_id,
            &self.table_preferences,
            &views::pods::pod_columns(),
        );
        self.deployments.hydrate_table_preferences(
            cluster_id,
            &self.table_preferences,
            &views::deployments::deployment_columns(),
        );
        self.services.hydrate_table_preferences(
            cluster_id,
            &self.table_preferences,
            &views::services::service_columns(),
        );
        self.events.hydrate_table_preferences(
            cluster_id,
            &self.table_preferences,
            &views::events::event_columns(),
        );
    }

    fn reset_exemplar_pages(&mut self) {
        self.pods.table.reset_page();
        self.deployments.table.reset_page();
        self.services.table.reset_page();
        self.events.table.reset_page();
    }

    fn exemplar_table_mut(
        &mut self,
        view: WorkspaceView,
    ) -> Option<(
        &'static str,
        &mut crate::components::DataTableState,
        Vec<crate::components::TableColumn>,
    )> {
        match view {
            WorkspaceView::Pods => Some(("pods", &mut self.pods.table, views::pods::pod_columns())),
            WorkspaceView::Deployments => Some((
                "deployments",
                &mut self.deployments.table,
                views::deployments::deployment_columns(),
            )),
            WorkspaceView::Services => Some((
                "services",
                &mut self.services.table,
                views::services::service_columns(),
            )),
            WorkspaceView::Events => Some((
                "events",
                &mut self.events.table,
                views::events::event_columns(),
            )),
            _ => None,
        }
    }

    fn persist_table_config(
        &mut self,
        table_id: &str,
        config: orbit_core::config::PersistedTableConfig,
    ) {
        self.table_preferences
            .tables
            .insert(table_id.to_string(), config.clone());
        self.dispatch_command(ClusterBridgeCommand::SaveTableConfig {
            table_id: table_id.to_string(),
            config,
        });
    }

    fn persist_cluster_table_filters(
        &mut self,
        table_id: &str,
        filters: orbit_core::config::PersistedClusterTableFilters,
    ) {
        let Some(cluster_id) = self.state.active_cluster_id.clone() else {
            return;
        };
        self.table_preferences
            .clusters
            .entry(cluster_id.clone())
            .or_default()
            .insert(table_id.to_string(), filters.clone());
        self.dispatch_command(ClusterBridgeCommand::SaveClusterTableFilters {
            cluster_id,
            table_id: table_id.to_string(),
            filters,
        });
    }

    fn handle_exemplar_table_key(
        &mut self,
        event: &gpui::KeyDownEvent,
        cx: &mut Context<Self>,
    ) -> bool {
        if event.keystroke.modifiers.control
            || event.keystroke.modifiers.alt
            || event.keystroke.modifiers.platform
        {
            return false;
        }
        let active_view = self.sidebar.active_view;
        let key_lower = event.keystroke.key.to_ascii_lowercase();
        let Some((table_id, table, _)) = self.exemplar_table_mut(active_view) else {
            return false;
        };

        if key_lower == "escape" && table.config_popover_open {
            table.config_popover_open = false;
            cx.notify();
            return true;
        }
        if !table.filters.search_focused {
            return false;
        }

        let mut changed = false;
        if key_lower == "escape" {
            table.filters.search_focused = false;
            cx.notify();
            return true;
        } else if key_lower == "backspace" {
            if table.filters.search_query.pop().is_some() {
                table.reset_page();
                changed = true;
            }
        } else if let Some(ch) = event.keystroke.key_char.as_deref()
            && !ch.is_empty()
            && !ch.chars().any(char::is_control)
        {
            table.filters.search_query.push_str(ch);
            table.reset_page();
            changed = true;
        } else if key_lower == "space" {
            table.filters.search_query.push(' ');
            table.reset_page();
            changed = true;
        }

        if changed {
            let filters = table.to_persisted_filters();
            self.persist_cluster_table_filters(table_id, filters);
        }
        cx.notify();
        true
    }

    fn table_action_cb(
        &self,
        view: WorkspaceView,
        cx: &mut Context<Self>,
        f: impl 'static
        + Fn(
            &mut Self,
            &'static str,
            &mut crate::components::DataTableState,
            &[crate::components::TableColumn],
        ),
    ) -> crate::components::TableActionCallback {
        let entity = cx.entity().clone();
        std::rc::Rc::new(move |_window, cx| {
            entity.update(cx, |shell, cx| {
                if let Some((table_id, mut table, cols)) = shell
                    .exemplar_table_mut(view)
                    .map(|(id, t, c)| (id, t.clone(), c))
                {
                    f(shell, table_id, &mut table, &cols);
                    if let Some((_, dst, _)) = shell.exemplar_table_mut(view) {
                        *dst = table;
                    }
                    cx.notify();
                }
            });
        })
    }

    fn table_enhancement_callbacks(
        &self,
        view: WorkspaceView,
        cx: &mut Context<Self>,
    ) -> crate::components::DataTableEnhancementCallbacks {
        use std::rc::Rc;

        let page_entity = cx.entity().clone();
        let on_page_change: crate::components::PageChangeCallback =
            Rc::new(move |first, page_size, _window, cx| {
                page_entity.update(cx, |shell, cx| {
                    if let Some((table_id, table, _)) = shell.exemplar_table_mut(view) {
                        let size_changed = table.pagination.page_size != page_size;
                        table.pagination.first = first;
                        table.pagination.page_size = page_size;
                        if size_changed {
                            let cfg = table.to_persisted_config();
                            shell.persist_table_config(table_id, cfg);
                        }
                        cx.notify();
                    }
                });
            });

        let focus_entity = cx.entity().clone();
        let on_focus_search: crate::components::TableBoolCallback =
            Rc::new(move |focused, window, cx| {
                focus_entity.update(cx, |shell, cx| {
                    if let Some((_, table, _)) = shell.exemplar_table_mut(view) {
                        table.filters.search_focused = focused;
                        table.config_popover_open = false;
                        if focused {
                            shell.focus_handle.focus(window);
                        }
                        cx.notify();
                    }
                });
            });

        let extra_entity = cx.entity().clone();
        let on_extra_filter: crate::components::TableExtraFilterCallback =
            Rc::new(move |key, value, _window, cx| {
                extra_entity.update(cx, |shell, cx| {
                    if let Some((table_id, table, _)) = shell.exemplar_table_mut(view) {
                        table.set_extra_filter(key, value);
                        let filters = table.to_persisted_filters();
                        shell.persist_cluster_table_filters(table_id, filters);
                        cx.notify();
                    }
                });
            });

        let toggle_col_entity = cx.entity().clone();
        let on_toggle_column: crate::components::TableColumnToggleCallback =
            Rc::new(move |col_id, locked, _window, cx| {
                toggle_col_entity.update(cx, |shell, cx| {
                    if let Some((table_id, table, _)) = shell.exemplar_table_mut(view)
                        && table.toggle_column(&col_id, locked)
                    {
                        let cfg = table.to_persisted_config();
                        shell.persist_table_config(table_id, cfg);
                        cx.notify();
                    }
                });
            });

        let toggle_all_entity = cx.entity().clone();
        let on_toggle_all: crate::components::TableBoolCallback =
            Rc::new(move |visible, _window, cx| {
                toggle_all_entity.update(cx, |shell, cx| {
                    if let Some((table_id, table, cols)) = shell.exemplar_table_mut(view) {
                        table.set_all_columns_visible(&cols, visible);
                        let cfg = table.to_persisted_config();
                        shell.persist_table_config(table_id, cfg);
                        cx.notify();
                    }
                });
            });

        let resize_entity = cx.entity().clone();
        let on_column_resize_start: crate::components::ColumnResizeStartCallback =
            Rc::new(move |col_id, pointer_x, window, cx| {
                resize_entity.update(cx, |shell, cx| {
                    let container_width = table_container_width(
                        f32::from(window.viewport_size().width),
                        &shell.sidebar,
                    );
                    if let Some((_, table, cols)) = shell.exemplar_table_mut(view) {
                        let visible_cols = table.resolved_visible_columns(&cols);
                        if table.start_column_resize(
                            col_id.as_ref(),
                            pointer_x,
                            &visible_cols,
                            container_width,
                        ) {
                            table.config_popover_open = false;
                            cx.notify();
                        }
                    }
                });
            });

        crate::components::DataTableEnhancementCallbacks {
            on_page_change,
            on_column_resize_start,
            toolbar: crate::components::TableToolbarCallbacks {
                on_focus_search,
                on_clear_search: self.table_action_cb(view, cx, |shell, table_id, table, _| {
                    table.set_search_query("");
                    shell.persist_cluster_table_filters(table_id, table.to_persisted_filters());
                }),
                on_extra_filter,
                on_clear_filters: self.table_action_cb(view, cx, |shell, table_id, table, _| {
                    table.clear_filters();
                    shell.persist_cluster_table_filters(table_id, table.to_persisted_filters());
                }),
                on_toggle_columns_popover: self.table_action_cb(view, cx, |_, _, table, _| {
                    table.config_popover_open = !table.config_popover_open;
                    table.filters.search_focused = false;
                }),
            },
            column_config: crate::components::ColumnConfigCallbacks {
                on_close: self.table_action_cb(view, cx, |_, _, table, _| {
                    table.config_popover_open = false;
                }),
                on_toggle_column,
                on_toggle_all,
                on_reset_defaults: self.table_action_cb(
                    view,
                    cx,
                    |shell, table_id, table, cols| {
                        table.reset_columns(cols);
                        shell.persist_table_config(table_id, table.to_persisted_config());
                    },
                ),
            },
        }
    }

    fn update_active_table_column_resize(&mut self, pointer_x: f32) -> bool {
        let mut updated = false;
        for table in [
            &mut self.pods.table,
            &mut self.deployments.table,
            &mut self.services.table,
            &mut self.events.table,
        ] {
            if table.update_column_resize(pointer_x) {
                updated = true;
            }
        }
        updated
    }

    fn finish_active_table_column_resize(&mut self) -> bool {
        let mut finished = Vec::new();
        for (table_id, table) in [
            ("pods", &mut self.pods.table),
            ("deployments", &mut self.deployments.table),
            ("services", &mut self.services.table),
            ("events", &mut self.events.table),
        ] {
            if let Some(item) = finish_table_column_resize(table_id, table) {
                finished.push(item);
            }
        }
        let any_finished = !finished.is_empty();
        for (table_id, cfg) in finished {
            self.persist_table_config(table_id, cfg);
        }
        any_finished
    }

    fn dispatch_command(&self, cmd: ClusterBridgeCommand) {
        if let Err(err) = self.command_tx.send(cmd) {
            tracing::warn!(error = %err, "Failed to send command to cluster bridge");
        }
    }

    fn select_workspace_view(&mut self, view: WorkspaceView, cx: &mut Context<Self>) {
        self.settings_dropdown.close();
        if view != self.sidebar.active_view {
            self.drawer.reset_for_context_change();
        }
        for cmd in select_workspace_view_with_tabs(
            &self.state,
            &mut self.sidebar,
            &mut self.workloads_overview,
            &mut self.namespaces,
            &mut self.deployments,
            &mut self.statefulsets,
            &mut self.daemonsets,
            &mut self.replicasets,
            &mut self.jobs,
            &mut self.cronjobs,
            &mut self.pods,
            &mut self.services,
            &mut self.ingresses,
            &mut self.configmaps,
            &mut self.secrets,
            &mut self.persistent_volumes,
            &mut self.persistent_volume_claims,
            &mut self.network_policies,
            &mut self.storage_overview,
            &mut self.pod_security,
            &mut self.resource_quotas,
            &mut self.list_views,
            &mut self.nodes,
            &mut self.events,
            &mut self.tabs,
            view,
        ) {
            self.dispatch_command(cmd);
        }
        if view == WorkspaceView::PortForward
            && let ClusterConnectionState::Connected { cluster_id, .. } =
                &self.state.connection_state
            && self.port_forwards.needs_load_for_cluster(cluster_id)
        {
            let cluster_id = cluster_id.clone();
            let cmd = self.port_forwards.begin_load_for_cluster(&cluster_id);
            self.dispatch_command(cmd);
        }
        cx.notify();
    }

    fn persist_tabs(&self) {
        if let Some(cmd) = self.tabs.persistence_command() {
            self.dispatch_command(cmd);
        }
    }

    fn sync_active_tab_view(&mut self, prev_view: WorkspaceView) {
        self.settings_dropdown.close();
        let active = self.tabs.active_view();
        self.sidebar.sync_for_view(active);
        if self.sidebar.active_view != prev_view {
            self.drawer.reset_for_context_change();
        }
        if let Some(cmd) = load_command_for_active_view(
            &self.state,
            &mut self.workloads_overview,
            &mut self.namespaces,
            &mut self.deployments,
            &mut self.statefulsets,
            &mut self.daemonsets,
            &mut self.replicasets,
            &mut self.jobs,
            &mut self.cronjobs,
            &mut self.pods,
            &mut self.services,
            &mut self.ingresses,
            &mut self.configmaps,
            &mut self.secrets,
            &mut self.persistent_volumes,
            &mut self.persistent_volume_claims,
            &mut self.network_policies,
            &mut self.storage_overview,
            &mut self.pod_security,
            &mut self.resource_quotas,
            &mut self.list_views,
            &mut self.nodes,
            &mut self.events,
            active,
        ) {
            self.dispatch_command(cmd);
        }
        self.persist_tabs();
    }

    fn activate_tab(&mut self, tab: impl Into<navigation::WorkspaceTab>, cx: &mut Context<Self>) {
        let prev_view = self.sidebar.active_view;
        self.tabs.set_active_tab(tab);
        self.sync_active_tab_view(prev_view);
        cx.notify();
    }

    fn close_tab(&mut self, tab: impl Into<navigation::WorkspaceTab>, cx: &mut Context<Self>) {
        let prev_view = self.sidebar.active_view;
        let closed = self.tabs.close_tab(tab);
        self.cleanup_closed_log_tabs(closed);
        self.sync_active_tab_view(prev_view);
        cx.notify();
    }

    fn close_other_tabs(
        &mut self,
        tab: impl Into<navigation::WorkspaceTab>,
        cx: &mut Context<Self>,
    ) {
        let prev_view = self.sidebar.active_view;
        let closed = self.tabs.close_other_tabs(tab);
        self.cleanup_closed_log_tabs(closed);
        self.sync_active_tab_view(prev_view);
        cx.notify();
    }

    fn close_tabs_to_right(
        &mut self,
        tab: impl Into<navigation::WorkspaceTab>,
        cx: &mut Context<Self>,
    ) {
        let prev_view = self.sidebar.active_view;
        let closed = self.tabs.close_tabs_to_right(tab);
        self.cleanup_closed_log_tabs(closed);
        self.sync_active_tab_view(prev_view);
        cx.notify();
    }

    fn close_all_tabs(&mut self, cx: &mut Context<Self>) {
        let prev_view = self.sidebar.active_view;
        let closed = self.tabs.close_all_tabs();
        self.cleanup_closed_log_tabs(closed);
        self.sync_active_tab_view(prev_view);
        cx.notify();
    }

    fn open_logs_tab(&mut self, kind: &str, namespace: &str, name: &str, cx: &mut Context<Self>) {
        let cluster_id = self.active_cluster_id().map(str::to_string);
        if let Some(cmd) = open_logs_tab_in_state(
            &mut self.tabs,
            &mut self.log_tabs,
            &mut self.next_log_stream_id,
            cluster_id.as_deref(),
            kind,
            namespace,
            name,
        ) {
            self.dispatch_command(cmd);
        }
        cx.notify();
    }

    fn restart_log_stream_for_tab(&mut self, target: &LogsTabTarget, cx: &mut Context<Self>) {
        let Some(cluster_id) = self.active_cluster_id().map(str::to_string) else {
            return;
        };
        let Some(log_state) = self.log_tabs.get_mut(target) else {
            return;
        };
        let prev_stream_id = log_state.stream_id;
        let new_stream_id = self.next_log_stream_id;
        self.next_log_stream_id = self.next_log_stream_id.saturating_add(1);
        log_state.reset_for_restart(new_stream_id);
        let cmd = ClusterBridgeCommand::StartLogStream {
            cluster_id,
            stream_id: new_stream_id,
            namespace: target.namespace.clone(),
            workload_kind: target.kind.clone(),
            workload_name: target.name.clone(),
            pod_filter: log_state.selected_pod.clone(),
            container_filter: log_state.selected_container.clone(),
            tail_lines: log_state.tail_lines,
            include_unavailable_pods: log_state.include_unavailable_pods,
        };
        self.dispatch_command(ClusterBridgeCommand::StopLogStream {
            stream_id: prev_stream_id,
        });
        self.dispatch_command(cmd);
        cx.notify();
    }

    fn cleanup_closed_log_tabs(&mut self, closed: Vec<LogsTabTarget>) {
        for cmd in cleanup_closed_log_tabs_in_state(&mut self.log_tabs, closed) {
            self.dispatch_command(cmd);
        }
    }

    fn export_logs(
        &mut self,
        target: LogsTabTarget,
        format: views::logs::LogExportFormat,
        cx: &mut Context<Self>,
    ) {
        let Some(log_state) = self.log_tabs.get(&target) else {
            return;
        };
        let rows = log_state.build_export_rows();
        let (ext, payload_res) = match format {
            views::logs::LogExportFormat::Json => (
                "json",
                format_logs_as_json(&rows).map_err(|e| format!("Failed to serialize JSON: {e}")),
            ),
            views::logs::LogExportFormat::Csv => ("csv", Ok(format_logs_as_csv(&rows))),
        };
        let payload = match payload_res {
            Ok(p) => p,
            Err(err) => {
                self.record_notification(
                    NotificationSeverity::Error,
                    "Export Logs Failed",
                    err.clone(),
                );
                let id = self.action_toast.show_error("Export Logs Failed", err);
                self.schedule_toast_dismiss(id, ActionToastKind::Error, cx);
                cx.notify();
                return;
            }
        };

        let suggested_name = format!("{}-logs.{ext}", target.name);
        let directory = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));
        let rx = cx.prompt_for_new_path(&directory, Some(&suggested_name));
        cx.spawn(async move |this, cx| {
            let Ok(Ok(Some(path))) = rx.await else {
                return;
            };
            let write_res = cx
                .background_executor()
                .spawn(async move { std::fs::write(&path, payload).map(|()| path) })
                .await;
            let _ = this.update(cx, |shell, cx| {
                match write_res {
                    Ok(saved_path) => {
                        let msg = format!("Saved to {}", saved_path.display());
                        shell.record_notification(
                            NotificationSeverity::Success,
                            "Logs Exported",
                            msg.clone(),
                        );
                        let id = shell.action_toast.show_success("Logs Exported", msg);
                        shell.schedule_toast_dismiss(id, ActionToastKind::Success, cx);
                    }
                    Err(err) => {
                        let msg = format!("Failed to write file: {err}");
                        shell.record_notification(
                            NotificationSeverity::Error,
                            "Export Logs Failed",
                            msg.clone(),
                        );
                        let id = shell.action_toast.show_error("Export Logs Failed", msg);
                        shell.schedule_toast_dismiss(id, ActionToastKind::Error, cx);
                    }
                }
                cx.notify();
            });
        })
        .detach();
    }

    fn copy_logs_for_tab(&mut self, target: &LogsTabTarget, cx: &mut Context<Self>) {
        if let Some(s) = self.log_tabs.get(target) {
            let text = s.copy_text();
            if !text.is_empty() {
                cx.write_to_clipboard(ClipboardItem::new_string(text));
                let id = self
                    .action_toast
                    .show_success("Copied Logs", "Copied log lines to clipboard");
                self.schedule_toast_dismiss(id, ActionToastKind::Success, cx);
                cx.notify();
            }
        }
    }

    fn render_logs_tab_content(
        &mut self,
        target: &LogsTabTarget,
        theme: &OrbitTheme,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let search_focus = self.log_search_focus.clone();
        let make_cb = |f: fn(&mut Self, &LogsTabTarget, &mut Window, &mut Context<Self>)| {
            let entity = cx.entity().clone();
            let target = target.clone();
            std::rc::Rc::new(move |window: &mut Window, cx: &mut App| {
                entity.update(cx, |shell, cx| f(shell, &target, window, cx));
            })
        };

        let callbacks = views::logs::LogViewerCallbacks {
            on_cycle_pod: make_cb(|shell, target, _w, cx| {
                if let Some(s) = shell.log_tabs.get_mut(target) {
                    s.cycle_pod();
                }
                shell.restart_log_stream_for_tab(target, cx);
            }),
            on_cycle_container: make_cb(|shell, target, _w, cx| {
                if let Some(s) = shell.log_tabs.get_mut(target) {
                    s.cycle_container();
                }
                shell.restart_log_stream_for_tab(target, cx);
            }),
            on_toggle_include_unavailable: make_cb(|shell, target, _w, cx| {
                if let Some(s) = shell.log_tabs.get_mut(target) {
                    s.include_unavailable_pods = !s.include_unavailable_pods;
                }
                shell.restart_log_stream_for_tab(target, cx);
            }),
            on_cycle_tail_lines: make_cb(|shell, target, _w, cx| {
                if let Some(s) = shell.log_tabs.get_mut(target) {
                    s.cycle_tail_lines();
                }
                shell.restart_log_stream_for_tab(target, cx);
            }),
            on_focus_search: make_cb(|shell, target, window, cx| {
                if let Some(s) = shell.log_tabs.get_mut(target) {
                    s.search_focused = true;
                }
                shell.log_search_focus.focus(window);
                cx.notify();
            }),
            on_clear_search: make_cb(|shell, target, _w, cx| {
                if let Some(s) = shell.log_tabs.get_mut(target) {
                    s.set_search_query(String::new());
                }
                cx.notify();
            }),
            on_prev_match: make_cb(|shell, target, _w, cx| {
                if let Some(s) = shell.log_tabs.get_mut(target) {
                    s.prev_match();
                }
                cx.notify();
            }),
            on_next_match: make_cb(|shell, target, _w, cx| {
                if let Some(s) = shell.log_tabs.get_mut(target) {
                    s.next_match();
                }
                cx.notify();
            }),
            on_toggle_pause: make_cb(|shell, target, _w, cx| {
                if let Some(s) = shell.log_tabs.get_mut(target) {
                    s.toggle_pause();
                }
                cx.notify();
            }),
            on_toggle_follow: make_cb(|shell, target, _w, cx| {
                if let Some(s) = shell.log_tabs.get_mut(target) {
                    s.toggle_follow();
                }
                cx.notify();
            }),
            on_toggle_wrap: make_cb(|shell, target, _w, cx| {
                if let Some(s) = shell.log_tabs.get_mut(target) {
                    s.toggle_wrap();
                }
                cx.notify();
            }),
            on_copy: make_cb(|shell, target, _w, cx| {
                shell.copy_logs_for_tab(target, cx);
            }),
            on_export: {
                let entity = cx.entity().clone();
                let target = target.clone();
                std::rc::Rc::new(move |format, _w, cx| {
                    entity.update(cx, |shell, cx| {
                        shell.export_logs(target.clone(), format, cx);
                    });
                })
            },
            on_line_mouse_down: {
                let entity = cx.entity().clone();
                let target = target.clone();
                std::rc::Rc::new(move |visible_idx, shift, _w, cx| {
                    entity.update(cx, |shell, cx| {
                        if let Some(s) = shell.log_tabs.get_mut(&target) {
                            s.search_focused = false;
                            s.select_line(visible_idx, shift);
                            s.is_dragging_selection = true;
                        }
                        cx.notify();
                    });
                })
            },
            on_line_mouse_move: {
                let entity = cx.entity().clone();
                let target = target.clone();
                std::rc::Rc::new(move |visible_idx, _w, cx| {
                    entity.update(cx, |shell, cx| {
                        if let Some(s) = shell.log_tabs.get_mut(&target)
                            && s.is_dragging_selection
                        {
                            s.extend_drag_selection(visible_idx);
                            cx.notify();
                        }
                    });
                })
            },
            on_scroll_wheel_up: make_cb(|shell, target, _w, cx| {
                if let Some(s) = shell.log_tabs.get_mut(target)
                    && s.is_following
                {
                    s.is_following = false;
                    cx.notify();
                }
            }),
        };

        if let Some(log_state) = self.log_tabs.get_mut(target) {
            views::logs::render_logs_view(log_state, theme, &search_focus, callbacks)
        } else {
            div().into_any_element()
        }
    }

    fn move_tab_left(&mut self, tab: impl Into<navigation::WorkspaceTab>, cx: &mut Context<Self>) {
        self.tabs.move_tab_left(tab);
        self.persist_tabs();
        cx.notify();
    }

    fn move_tab_right(&mut self, tab: impl Into<navigation::WorkspaceTab>, cx: &mut Context<Self>) {
        self.tabs.move_tab_right(tab);
        self.persist_tabs();
        cx.notify();
    }

    fn reorder_tab_drop(
        &mut self,
        from_idx: usize,
        hover_idx: usize,
        pos: TabDropPosition,
        cx: &mut Context<Self>,
    ) {
        if self.tabs.apply_drop(from_idx, hover_idx, pos) {
            self.persist_tabs();
        }
        cx.notify();
    }

    fn active_cluster_id(&self) -> Option<&str> {
        self.state.active_cluster_id.as_deref()
    }

    fn open_resource_drawer(
        &mut self,
        target: ResourceDrawerTarget,
        initial_tab: ResourceDrawerTabId,
        cx: &mut Context<Self>,
    ) {
        let cluster_id = self.active_cluster_id().map(str::to_string);
        if matches!(
            target,
            ResourceDrawerTarget::Deployment { .. }
                | ResourceDrawerTarget::StatefulSet { .. }
                | ResourceDrawerTarget::DaemonSet { .. }
                | ResourceDrawerTarget::ReplicaSet { .. }
                | ResourceDrawerTarget::Job { .. }
                | ResourceDrawerTarget::CronJob { .. }
                | ResourceDrawerTarget::Pod { .. }
                | ResourceDrawerTarget::Service { .. }
                | ResourceDrawerTarget::Ingress { .. }
                | ResourceDrawerTarget::ConfigMap { .. }
                | ResourceDrawerTarget::Secret { .. }
                | ResourceDrawerTarget::PersistentVolume { .. }
                | ResourceDrawerTarget::PersistentVolumeClaim { .. }
                | ResourceDrawerTarget::NetworkPolicy { .. }
                | ResourceDrawerTarget::PodSecurity { .. }
                | ResourceDrawerTarget::ResourceQuota { .. }
                | ResourceDrawerTarget::LimitRange { .. }
                | ResourceDrawerTarget::Hpa { .. }
                | ResourceDrawerTarget::StorageClass { .. }
                | ResourceDrawerTarget::AdmissionWebhook { .. }
                | ResourceDrawerTarget::SecurityPolicy { .. }
                | ResourceDrawerTarget::Node { .. }
        ) {
            if let Some(cid) = cluster_id.as_deref() {
                if self.events.needs_load_for_cluster(cid) {
                    let cmd = self.events.begin_load_for_cluster(cid);
                    self.dispatch_command(cmd);
                }
            }
        }
        if let Some(cmd) = self
            .drawer
            .open_or_switch(cluster_id.as_deref(), target, initial_tab)
        {
            self.dispatch_command(cmd);
        }
        cx.notify();
    }

    fn select_drawer_tab(&mut self, tab: ResourceDrawerTabId, cx: &mut Context<Self>) {
        let cluster_id = self.active_cluster_id().map(str::to_string);
        if let Some(cmd) = self.drawer.select_tab(cluster_id.as_deref(), tab) {
            self.dispatch_command(cmd);
        }
        cx.notify();
    }

    fn retry_drawer_yaml(&mut self, cx: &mut Context<Self>) {
        let cluster_id = self.active_cluster_id().map(str::to_string);
        if let Some(cmd) = self.drawer.retry_yaml(cluster_id.as_deref()) {
            self.dispatch_command(cmd);
        }
        cx.notify();
    }

    fn copy_drawer_yaml(&mut self, yaml: String, cx: &mut Context<Self>) {
        cx.write_to_clipboard(ClipboardItem::new_string(yaml));
        self.drawer.yaml_copied = true;
        cx.notify();
    }

    fn dismiss_resource_drawer(&mut self, cx: &mut Context<Self>) {
        if self.drawer.is_open {
            self.drawer.close();
            cx.notify();
        }
    }

    fn find_event_for_target(
        &self,
        uid: &str,
        reason: &str,
        namespace: &str,
    ) -> Option<&orbit_core::kubernetes::models::EventInfo> {
        self.events.events.iter().find(|ev| {
            if !uid.is_empty() && !ev.uid.is_empty() {
                ev.uid == uid
            } else {
                ev.namespace == namespace && ev.reason == reason
            }
        })
    }

    fn reconcile_drawer_target_exists(&mut self) {
        let Some(target) = self.drawer.target.as_ref() else {
            return;
        };
        if !self.drawer.is_open {
            return;
        }
        let exists = match target {
            ResourceDrawerTarget::Deployment { name, namespace } => self
                .deployments
                .deployments
                .iter()
                .any(|d| d.name == *name && d.namespace == *namespace),
            ResourceDrawerTarget::StatefulSet { name, namespace } => self
                .statefulsets
                .statefulsets
                .iter()
                .any(|s| s.name == *name && s.namespace == *namespace),
            ResourceDrawerTarget::DaemonSet { name, namespace } => self
                .daemonsets
                .daemonsets
                .iter()
                .any(|d| d.name == *name && d.namespace == *namespace),
            ResourceDrawerTarget::ReplicaSet { name, namespace } => self
                .replicasets
                .replicasets
                .iter()
                .any(|r| r.name == *name && r.namespace == *namespace),
            ResourceDrawerTarget::Job { name, namespace } => self
                .jobs
                .jobs
                .iter()
                .any(|j| j.name == *name && j.namespace == *namespace),
            ResourceDrawerTarget::CronJob { name, namespace } => self
                .cronjobs
                .cronjobs
                .iter()
                .any(|cj| cj.name == *name && cj.namespace == *namespace),
            ResourceDrawerTarget::PersistentVolume { name } => self
                .persistent_volumes
                .persistent_volumes
                .iter()
                .any(|pv| pv.name == *name),
            ResourceDrawerTarget::Node { name } => self.nodes.nodes.iter().any(|n| n.name == *name),
            ResourceDrawerTarget::Namespace { name } => {
                self.namespaces.namespaces.iter().any(|ns| ns.name == *name)
            }
            ResourceDrawerTarget::Event {
                uid,
                reason,
                namespace,
                ..
            } => self.find_event_for_target(uid, reason, namespace).is_some(),
            ResourceDrawerTarget::Pod { name, namespace } => self
                .pods
                .pods
                .iter()
                .any(|p| p.name == *name && p.namespace == *namespace),
            ResourceDrawerTarget::Service { name, namespace } => self
                .services
                .services
                .iter()
                .any(|s| s.name == *name && s.namespace == *namespace),
            ResourceDrawerTarget::Ingress { name, namespace } => self
                .ingresses
                .ingresses
                .iter()
                .any(|ing| ing.name == *name && ing.namespace == *namespace),
            ResourceDrawerTarget::ConfigMap { name, namespace } => self
                .configmaps
                .configmaps
                .iter()
                .any(|c| c.name == *name && c.namespace == *namespace),
            ResourceDrawerTarget::Secret { name, namespace } => self
                .secrets
                .secrets
                .iter()
                .any(|s| s.name == *name && s.namespace == *namespace),
            ResourceDrawerTarget::PersistentVolumeClaim { name, namespace } => self
                .persistent_volume_claims
                .persistent_volume_claims
                .iter()
                .any(|p| p.name == *name && p.namespace == *namespace),
            ResourceDrawerTarget::NetworkPolicy { name, namespace } => self
                .network_policies
                .network_policies
                .iter()
                .any(|p| p.name == *name && p.namespace == *namespace),
            ResourceDrawerTarget::PodSecurity { name, namespace } => self
                .pod_security
                .policies
                .iter()
                .any(|p| p.name == *name && p.namespace == *namespace),
            ResourceDrawerTarget::ResourceQuota { name, namespace } => {
                self.resource_quotas.policies.iter().any(|p| {
                    p.kind.eq_ignore_ascii_case("ResourceQuota")
                        && p.name == *name
                        && p.namespace == *namespace
                })
            }
            ResourceDrawerTarget::LimitRange { name, namespace } => {
                self.resource_quotas.policies.iter().any(|p| {
                    p.kind.eq_ignore_ascii_case("LimitRange")
                        && p.name == *name
                        && p.namespace == *namespace
                })
            }
            ResourceDrawerTarget::Hpa { name, namespace } => self
                .list_views
                .hpas
                .hpas
                .iter()
                .any(|h| h.name == *name && h.namespace == *namespace),
            ResourceDrawerTarget::StorageClass { name } => self
                .list_views
                .storage_classes
                .classes
                .iter()
                .any(|sc| sc.name == *name),
            ResourceDrawerTarget::AdmissionWebhook { is_mutating, name } => {
                let target_kind = if *is_mutating {
                    "MutatingWebhookConfiguration"
                } else {
                    "ValidatingWebhookConfiguration"
                };
                self.list_views
                    .admission_webhooks
                    .policies
                    .iter()
                    .any(|p| p.kind.eq_ignore_ascii_case(target_kind) && p.name == *name)
            }
            ResourceDrawerTarget::SecurityPolicy {
                kind,
                name,
                namespace,
            } => self.list_views.security_overview.policies.iter().any(|p| {
                p.kind.eq_ignore_ascii_case(kind) && p.name == *name && p.namespace == *namespace
            }),
        };
        if !exists {
            self.drawer.close();
        }
    }

    fn render_live_yaml_tab(
        &self,
        subtitle: impl Into<SharedString>,
        theme: &OrbitTheme,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let copy_yaml = self.drawer.yaml_content.clone();
        let copy_entity = cx.entity().clone();
        let on_copy = copy_yaml.map(|yaml| {
            std::rc::Rc::new(move |_w: &mut Window, cx: &mut App| {
                copy_entity.update(cx, |shell, cx| {
                    shell.copy_drawer_yaml(yaml.clone(), cx);
                });
            }) as std::rc::Rc<dyn Fn(&mut Window, &mut App)>
        });
        let retry_entity = cx.entity().clone();
        let on_retry = std::rc::Rc::new(move |_w: &mut Window, cx: &mut App| {
            retry_entity.update(cx, |shell, cx| {
                shell.retry_drawer_yaml(cx);
            });
        });
        render_yaml_viewer(
            YamlViewerProps {
                status: self.drawer.yaml_status,
                yaml_content: self.drawer.yaml_content.as_deref(),
                cached_document: self.drawer.yaml_document.current(),
                scroll_handle: Some(self.drawer.yaml_scroll_handle.clone()),
                error_message: self.drawer.yaml_error.as_deref(),
                copied: self.drawer.yaml_copied,
                subtitle: subtitle.into(),
                on_copy,
                on_retry: Some(on_retry),
            },
            theme,
        )
    }

    fn build_drawer_target_content(
        &self,
        target: &ResourceDrawerTarget,
        theme: &OrbitTheme,
        cx: &mut Context<Self>,
    ) -> Option<DrawerTargetContent> {
        let events_loading = self.events.status == EventsLoadStatus::Loading;

        match target {
            ResourceDrawerTarget::Deployment { name, namespace } => {
                let dep = self
                    .deployments
                    .deployments
                    .iter()
                    .find(|d| d.name == *name && d.namespace == *namespace)?;
                let matched_events =
                    views::deployments::filter_events_for_deployment(&self.events.events, dep);
                let tabs = vec![
                    ResourceDrawerTabDef {
                        id: ResourceDrawerTabId::Overview,
                        label: SharedString::from("Overview"),
                        badge: None,
                    },
                    ResourceDrawerTabDef {
                        id: ResourceDrawerTabId::Events,
                        label: SharedString::from("Events"),
                        badge: Some(SharedString::from(matched_events.len().to_string())),
                    },
                    ResourceDrawerTabDef {
                        id: ResourceDrawerTabId::Yaml,
                        label: SharedString::from("YAML"),
                        badge: None,
                    },
                ];
                let body = match self.drawer.active_tab {
                    ResourceDrawerTabId::Overview => {
                        views::deployments::render_deployment_drawer_overview(dep, theme)
                    }
                    ResourceDrawerTabId::Events => {
                        views::deployments::render_deployment_drawer_events(
                            &self.events.events,
                            events_loading,
                            dep,
                            theme,
                        )
                    }
                    ResourceDrawerTabId::Yaml => self.render_live_yaml_tab(
                        format!("{}/{}", dep.namespace, dep.name),
                        theme,
                        cx,
                    ),
                    ResourceDrawerTabId::Custom(_) => div().into_any_element(),
                };
                Some((
                    SharedString::from(dep.name.clone()),
                    SharedString::from("Deployment"),
                    Some(SharedString::from(dep.namespace.clone())),
                    Some(SharedString::from(dep.age.clone())),
                    Some(SharedString::from(dep.status.clone())),
                    tabs,
                    body,
                ))
            }
            ResourceDrawerTarget::StatefulSet { name, namespace } => {
                let ss = self
                    .statefulsets
                    .statefulsets
                    .iter()
                    .find(|s| s.name == *name && s.namespace == *namespace)?;
                let matched_events =
                    views::statefulsets::filter_events_for_statefulset(&self.events.events, ss);
                let tabs = vec![
                    ResourceDrawerTabDef {
                        id: ResourceDrawerTabId::Overview,
                        label: SharedString::from("Overview"),
                        badge: None,
                    },
                    ResourceDrawerTabDef {
                        id: ResourceDrawerTabId::Events,
                        label: SharedString::from("Events"),
                        badge: Some(SharedString::from(matched_events.len().to_string())),
                    },
                    ResourceDrawerTabDef {
                        id: ResourceDrawerTabId::Yaml,
                        label: SharedString::from("YAML"),
                        badge: None,
                    },
                ];
                let body = match self.drawer.active_tab {
                    ResourceDrawerTabId::Overview => {
                        views::statefulsets::render_statefulset_drawer_overview(ss, theme)
                    }
                    ResourceDrawerTabId::Events => {
                        views::statefulsets::render_statefulset_drawer_events(
                            &self.events.events,
                            events_loading,
                            ss,
                            theme,
                        )
                    }
                    ResourceDrawerTabId::Yaml => self.render_live_yaml_tab(
                        format!("{}/{}", ss.namespace, ss.name),
                        theme,
                        cx,
                    ),
                    ResourceDrawerTabId::Custom(_) => div().into_any_element(),
                };
                Some((
                    SharedString::from(ss.name.clone()),
                    SharedString::from("StatefulSet"),
                    Some(SharedString::from(ss.namespace.clone())),
                    Some(SharedString::from(ss.age.clone())),
                    Some(SharedString::from(ss.status.clone())),
                    tabs,
                    body,
                ))
            }
            ResourceDrawerTarget::ReplicaSet { name, namespace } => {
                let rs = self
                    .replicasets
                    .replicasets
                    .iter()
                    .find(|r| r.name == *name && r.namespace == *namespace)?;
                let matched_events =
                    views::replicasets::filter_events_for_replicaset(&self.events.events, rs);
                let tabs = vec![
                    ResourceDrawerTabDef {
                        id: ResourceDrawerTabId::Overview,
                        label: SharedString::from("Overview"),
                        badge: None,
                    },
                    ResourceDrawerTabDef {
                        id: ResourceDrawerTabId::Events,
                        label: SharedString::from("Events"),
                        badge: Some(SharedString::from(matched_events.len().to_string())),
                    },
                    ResourceDrawerTabDef {
                        id: ResourceDrawerTabId::Yaml,
                        label: SharedString::from("YAML"),
                        badge: None,
                    },
                ];
                let body = match self.drawer.active_tab {
                    ResourceDrawerTabId::Overview => {
                        views::replicasets::render_replicaset_drawer_overview(rs, theme)
                    }
                    ResourceDrawerTabId::Events => {
                        views::replicasets::render_replicaset_drawer_events(
                            &self.events.events,
                            events_loading,
                            rs,
                            theme,
                        )
                    }
                    ResourceDrawerTabId::Yaml => self.render_live_yaml_tab(
                        format!("{}/{}", rs.namespace, rs.name),
                        theme,
                        cx,
                    ),
                    ResourceDrawerTabId::Custom(_) => div().into_any_element(),
                };
                Some((
                    SharedString::from(rs.name.clone()),
                    SharedString::from("ReplicaSet"),
                    Some(SharedString::from(rs.namespace.clone())),
                    Some(SharedString::from(rs.age.clone())),
                    Some(SharedString::from(rs.status.clone())),
                    tabs,
                    body,
                ))
            }
            ResourceDrawerTarget::DaemonSet { name, namespace } => {
                let ds = self
                    .daemonsets
                    .daemonsets
                    .iter()
                    .find(|d| d.name == *name && d.namespace == *namespace)?;
                let matched_events =
                    views::daemonsets::filter_events_for_daemonset(&self.events.events, ds);
                let tabs = vec![
                    ResourceDrawerTabDef {
                        id: ResourceDrawerTabId::Overview,
                        label: SharedString::from("Overview"),
                        badge: None,
                    },
                    ResourceDrawerTabDef {
                        id: ResourceDrawerTabId::Events,
                        label: SharedString::from("Events"),
                        badge: Some(SharedString::from(matched_events.len().to_string())),
                    },
                    ResourceDrawerTabDef {
                        id: ResourceDrawerTabId::Yaml,
                        label: SharedString::from("YAML"),
                        badge: None,
                    },
                ];
                let body = match self.drawer.active_tab {
                    ResourceDrawerTabId::Overview => {
                        views::daemonsets::render_daemonset_drawer_overview(ds, theme)
                    }
                    ResourceDrawerTabId::Events => {
                        views::daemonsets::render_daemonset_drawer_events(
                            &self.events.events,
                            events_loading,
                            ds,
                            theme,
                        )
                    }
                    ResourceDrawerTabId::Yaml => self.render_live_yaml_tab(
                        format!("{}/{}", ds.namespace, ds.name),
                        theme,
                        cx,
                    ),
                    ResourceDrawerTabId::Custom(_) => div().into_any_element(),
                };
                Some((
                    SharedString::from(ds.name.clone()),
                    SharedString::from("DaemonSet"),
                    Some(SharedString::from(ds.namespace.clone())),
                    Some(SharedString::from(ds.age.clone())),
                    Some(SharedString::from(ds.status.clone())),
                    tabs,
                    body,
                ))
            }
            ResourceDrawerTarget::Job { name, namespace } => {
                let job = self
                    .jobs
                    .jobs
                    .iter()
                    .find(|j| j.name == *name && j.namespace == *namespace)?;
                let matched_events = views::jobs::filter_events_for_job(&self.events.events, job);
                let tabs = vec![
                    ResourceDrawerTabDef {
                        id: ResourceDrawerTabId::Overview,
                        label: SharedString::from("Overview"),
                        badge: None,
                    },
                    ResourceDrawerTabDef {
                        id: ResourceDrawerTabId::Events,
                        label: SharedString::from("Events"),
                        badge: Some(SharedString::from(matched_events.len().to_string())),
                    },
                    ResourceDrawerTabDef {
                        id: ResourceDrawerTabId::Yaml,
                        label: SharedString::from("YAML"),
                        badge: None,
                    },
                ];
                let body = match self.drawer.active_tab {
                    ResourceDrawerTabId::Overview => {
                        views::jobs::render_job_drawer_overview(job, theme)
                    }
                    ResourceDrawerTabId::Events => views::jobs::render_job_drawer_events(
                        &self.events.events,
                        events_loading,
                        job,
                        theme,
                    ),
                    ResourceDrawerTabId::Yaml => self.render_live_yaml_tab(
                        format!("{}/{}", job.namespace, job.name),
                        theme,
                        cx,
                    ),
                    ResourceDrawerTabId::Custom(_) => div().into_any_element(),
                };
                Some((
                    SharedString::from(job.name.clone()),
                    SharedString::from("Job"),
                    Some(SharedString::from(job.namespace.clone())),
                    Some(SharedString::from(job.age.clone())),
                    Some(SharedString::from(job.status.clone())),
                    tabs,
                    body,
                ))
            }
            ResourceDrawerTarget::CronJob { name, namespace } => {
                let cj = self
                    .cronjobs
                    .cronjobs
                    .iter()
                    .find(|c| c.name == *name && c.namespace == *namespace)?;
                let matched_events =
                    views::cronjobs::filter_events_for_cronjob(&self.events.events, cj);
                let tabs = vec![
                    ResourceDrawerTabDef {
                        id: ResourceDrawerTabId::Overview,
                        label: SharedString::from("Overview"),
                        badge: None,
                    },
                    ResourceDrawerTabDef {
                        id: ResourceDrawerTabId::Events,
                        label: SharedString::from("Events"),
                        badge: Some(SharedString::from(matched_events.len().to_string())),
                    },
                    ResourceDrawerTabDef {
                        id: ResourceDrawerTabId::Yaml,
                        label: SharedString::from("YAML"),
                        badge: None,
                    },
                ];
                let body = match self.drawer.active_tab {
                    ResourceDrawerTabId::Overview => {
                        views::cronjobs::render_cronjob_drawer_overview(cj, theme)
                    }
                    ResourceDrawerTabId::Events => views::cronjobs::render_cronjob_drawer_events(
                        &self.events.events,
                        events_loading,
                        cj,
                        theme,
                    ),
                    ResourceDrawerTabId::Yaml => self.render_live_yaml_tab(
                        format!("{}/{}", cj.namespace, cj.name),
                        theme,
                        cx,
                    ),
                    ResourceDrawerTabId::Custom(_) => div().into_any_element(),
                };
                Some((
                    SharedString::from(cj.name.clone()),
                    SharedString::from("CronJob"),
                    Some(SharedString::from(cj.namespace.clone())),
                    Some(SharedString::from(cj.age.clone())),
                    Some(SharedString::from(views::cronjobs::cronjob_suspend_status(
                        cj.suspend,
                    ))),
                    tabs,
                    body,
                ))
            }
            ResourceDrawerTarget::Pod { name, namespace } => {
                let pod = self
                    .pods
                    .pods
                    .iter()
                    .find(|p| p.name == *name && p.namespace == *namespace)?;
                let matched_events = views::pods::filter_events_for_pod(&self.events.events, pod);
                let tabs = vec![
                    ResourceDrawerTabDef {
                        id: ResourceDrawerTabId::Overview,
                        label: SharedString::from("Overview"),
                        badge: None,
                    },
                    ResourceDrawerTabDef {
                        id: ResourceDrawerTabId::Events,
                        label: SharedString::from("Events"),
                        badge: Some(SharedString::from(matched_events.len().to_string())),
                    },
                    ResourceDrawerTabDef {
                        id: ResourceDrawerTabId::Yaml,
                        label: SharedString::from("YAML"),
                        badge: None,
                    },
                ];
                let body = match self.drawer.active_tab {
                    ResourceDrawerTabId::Overview => {
                        views::pods::render_pod_drawer_overview(pod, theme)
                    }
                    ResourceDrawerTabId::Events => views::pods::render_pod_drawer_events(
                        &self.events.events,
                        events_loading,
                        pod,
                        theme,
                    ),
                    ResourceDrawerTabId::Yaml => self.render_live_yaml_tab(
                        format!("{}/{}", pod.namespace, pod.name),
                        theme,
                        cx,
                    ),
                    ResourceDrawerTabId::Custom(_) => div().into_any_element(),
                };
                Some((
                    SharedString::from(pod.name.clone()),
                    SharedString::from("Pod"),
                    Some(SharedString::from(pod.namespace.clone())),
                    Some(SharedString::from(pod.age.clone())),
                    Some(SharedString::from(pod.status.clone())),
                    tabs,
                    body,
                ))
            }
            ResourceDrawerTarget::PersistentVolume { name } => {
                let pv = self
                    .persistent_volumes
                    .persistent_volumes
                    .iter()
                    .find(|p| p.name == *name)?;
                let matched_events = views::persistent_volumes::filter_events_for_persistent_volume(
                    &self.events.events,
                    pv,
                );
                let tabs = vec![
                    ResourceDrawerTabDef {
                        id: ResourceDrawerTabId::Overview,
                        label: SharedString::from("Overview"),
                        badge: None,
                    },
                    ResourceDrawerTabDef {
                        id: ResourceDrawerTabId::Events,
                        label: SharedString::from("Events"),
                        badge: Some(SharedString::from(matched_events.len().to_string())),
                    },
                    ResourceDrawerTabDef {
                        id: ResourceDrawerTabId::Yaml,
                        label: SharedString::from("YAML"),
                        badge: None,
                    },
                ];
                let body = match self.drawer.active_tab {
                    ResourceDrawerTabId::Overview => {
                        views::persistent_volumes::render_persistent_volume_drawer_overview(
                            pv, theme,
                        )
                    }
                    ResourceDrawerTabId::Events => {
                        views::persistent_volumes::render_persistent_volume_drawer_events(
                            &matched_events,
                            events_loading,
                            theme,
                        )
                    }
                    ResourceDrawerTabId::Yaml => {
                        self.render_live_yaml_tab(pv.name.clone(), theme, cx)
                    }
                    ResourceDrawerTabId::Custom(_) => div().into_any_element(),
                };
                Some((
                    SharedString::from(pv.name.clone()),
                    SharedString::from("PersistentVolume"),
                    None,
                    Some(SharedString::from(pv.age.clone())),
                    Some(SharedString::from(pv.status.clone())),
                    tabs,
                    body,
                ))
            }
            ResourceDrawerTarget::Node { name } => {
                let node = self.nodes.nodes.iter().find(|n| n.name == *name)?;
                let matched_events =
                    views::nodes::filter_events_for_node(&self.events.events, node);
                let tabs = vec![
                    ResourceDrawerTabDef {
                        id: ResourceDrawerTabId::Overview,
                        label: SharedString::from("Overview"),
                        badge: None,
                    },
                    ResourceDrawerTabDef {
                        id: ResourceDrawerTabId::Events,
                        label: SharedString::from("Events"),
                        badge: Some(SharedString::from(matched_events.len().to_string())),
                    },
                    ResourceDrawerTabDef {
                        id: ResourceDrawerTabId::Yaml,
                        label: SharedString::from("YAML"),
                        badge: None,
                    },
                ];
                let body = match self.drawer.active_tab {
                    ResourceDrawerTabId::Overview => {
                        views::nodes::render_node_drawer_overview(node, theme)
                    }
                    ResourceDrawerTabId::Events => views::nodes::render_node_drawer_events(
                        &self.events.events,
                        events_loading,
                        node,
                        theme,
                    ),
                    ResourceDrawerTabId::Yaml => {
                        self.render_live_yaml_tab(node.name.clone(), theme, cx)
                    }
                    ResourceDrawerTabId::Custom(_) => div().into_any_element(),
                };
                Some((
                    SharedString::from(node.name.clone()),
                    SharedString::from("Node"),
                    None,
                    Some(SharedString::from(node.uptime.clone())),
                    Some(SharedString::from(node.status.clone())),
                    tabs,
                    body,
                ))
            }
            ResourceDrawerTarget::Namespace { name } => {
                let ns = self
                    .namespaces
                    .namespaces
                    .iter()
                    .find(|n| n.name == *name)?;
                let tabs = vec![
                    ResourceDrawerTabDef {
                        id: ResourceDrawerTabId::Overview,
                        label: SharedString::from("Overview"),
                        badge: None,
                    },
                    ResourceDrawerTabDef {
                        id: ResourceDrawerTabId::Yaml,
                        label: SharedString::from("YAML"),
                        badge: None,
                    },
                ];
                let body = match self.drawer.active_tab {
                    ResourceDrawerTabId::Yaml => {
                        self.render_live_yaml_tab(ns.name.clone(), theme, cx)
                    }
                    _ => views::namespaces::render_namespace_drawer_overview(ns, theme),
                };
                Some((
                    SharedString::from(ns.name.clone()),
                    SharedString::from("Namespace"),
                    None,
                    Some(SharedString::from(ns.age.clone())),
                    Some(SharedString::from(ns.status.clone())),
                    tabs,
                    body,
                ))
            }
            ResourceDrawerTarget::Event {
                uid,
                reason,
                namespace,
                ..
            } => {
                let ev = self.find_event_for_target(uid, reason, namespace)?;
                let tabs = vec![
                    ResourceDrawerTabDef {
                        id: ResourceDrawerTabId::Overview,
                        label: SharedString::from("Overview"),
                        badge: None,
                    },
                    ResourceDrawerTabDef {
                        id: ResourceDrawerTabId::Yaml,
                        label: SharedString::from("YAML"),
                        badge: None,
                    },
                ];
                let body = match self.drawer.active_tab {
                    ResourceDrawerTabId::Yaml => {
                        let formatted = orbit_core::kubernetes::raw::format_event_yaml(ev);
                        let copy_yaml = formatted.clone();
                        let copy_entity = cx.entity().clone();
                        let on_copy = std::rc::Rc::new(move |_w: &mut Window, cx: &mut App| {
                            copy_entity.update(cx, |shell, cx| {
                                shell.copy_drawer_yaml(copy_yaml.clone(), cx);
                            });
                        });
                        render_yaml_viewer(
                            YamlViewerProps {
                                status: YamlLoadStatus::Loaded,
                                yaml_content: Some(&formatted),
                                cached_document: None,
                                scroll_handle: Some(self.drawer.yaml_scroll_handle.clone()),
                                error_message: None,
                                copied: self.drawer.yaml_copied,
                                subtitle: SharedString::from(format!(
                                    "{}/{}",
                                    ev.object_kind, ev.object_name
                                )),
                                on_copy: Some(on_copy),
                                on_retry: None,
                            },
                            theme,
                        )
                    }
                    _ => views::events::render_event_drawer_overview(ev, theme),
                };
                Some((
                    SharedString::from(ev.reason.clone()),
                    SharedString::from("Event"),
                    if ev.namespace.is_empty() {
                        None
                    } else {
                        Some(SharedString::from(ev.namespace.clone()))
                    },
                    Some(SharedString::from(ev.time.clone())),
                    Some(SharedString::from(ev.r#type.clone())),
                    tabs,
                    body,
                ))
            }
            ResourceDrawerTarget::Ingress { name, namespace } => {
                let ing = self
                    .ingresses
                    .ingresses
                    .iter()
                    .find(|i| i.name == *name && i.namespace == *namespace)?;
                let matched_events =
                    views::ingresses::filter_events_for_ingress(&self.events.events, ing);
                let rules_tab_id = ResourceDrawerTabId::Custom(SharedString::from("rules"));
                let tabs = vec![
                    ResourceDrawerTabDef {
                        id: ResourceDrawerTabId::Overview,
                        label: SharedString::from("Overview"),
                        badge: None,
                    },
                    ResourceDrawerTabDef {
                        id: rules_tab_id.clone(),
                        label: SharedString::from("Rules"),
                        badge: Some(SharedString::from(ing.rules_summary.len().to_string())),
                    },
                    ResourceDrawerTabDef {
                        id: ResourceDrawerTabId::Events,
                        label: SharedString::from("Events"),
                        badge: Some(SharedString::from(matched_events.len().to_string())),
                    },
                    ResourceDrawerTabDef {
                        id: ResourceDrawerTabId::Yaml,
                        label: SharedString::from("YAML"),
                        badge: None,
                    },
                ];
                let body = match &self.drawer.active_tab {
                    ResourceDrawerTabId::Overview => {
                        views::ingresses::render_ingress_drawer_overview(ing, theme)
                    }
                    ResourceDrawerTabId::Custom(id) if id.as_ref() == "rules" => {
                        views::ingresses::render_ingress_drawer_rules(ing, theme)
                    }
                    ResourceDrawerTabId::Events => views::ingresses::render_ingress_drawer_events(
                        &self.events.events,
                        events_loading,
                        ing,
                        theme,
                    ),
                    ResourceDrawerTabId::Yaml => self.render_live_yaml_tab(
                        format!("{}/{}", ing.namespace, ing.name),
                        theme,
                        cx,
                    ),
                    ResourceDrawerTabId::Custom(_) => div().into_any_element(),
                };
                Some((
                    SharedString::from(ing.name.clone()),
                    SharedString::from("Ingress"),
                    Some(SharedString::from(ing.namespace.clone())),
                    Some(SharedString::from(ing.age.clone())),
                    ing.class_name
                        .as_deref()
                        .filter(|c| !c.trim().is_empty())
                        .map(|c| SharedString::from(c.to_string())),
                    tabs,
                    body,
                ))
            }
            ResourceDrawerTarget::Service { name, namespace } => {
                let svc = self
                    .services
                    .services
                    .iter()
                    .find(|s| s.name == *name && s.namespace == *namespace)?;
                let matched_events =
                    views::services::filter_events_for_service(&self.events.events, svc);
                let tabs = vec![
                    ResourceDrawerTabDef {
                        id: ResourceDrawerTabId::Overview,
                        label: SharedString::from("Overview"),
                        badge: None,
                    },
                    ResourceDrawerTabDef {
                        id: ResourceDrawerTabId::Events,
                        label: SharedString::from("Events"),
                        badge: Some(SharedString::from(matched_events.len().to_string())),
                    },
                    ResourceDrawerTabDef {
                        id: ResourceDrawerTabId::Yaml,
                        label: SharedString::from("YAML"),
                        badge: None,
                    },
                ];
                let body = match self.drawer.active_tab {
                    ResourceDrawerTabId::Overview => {
                        views::services::render_service_drawer_overview(svc, theme)
                    }
                    ResourceDrawerTabId::Events => views::services::render_service_drawer_events(
                        &self.events.events,
                        events_loading,
                        svc,
                        theme,
                    ),
                    ResourceDrawerTabId::Yaml => self.render_live_yaml_tab(
                        format!("{}/{}", svc.namespace, svc.name),
                        theme,
                        cx,
                    ),
                    ResourceDrawerTabId::Custom(_) => div().into_any_element(),
                };
                Some((
                    SharedString::from(svc.name.clone()),
                    SharedString::from("Service"),
                    Some(SharedString::from(svc.namespace.clone())),
                    Some(SharedString::from(svc.age.clone())),
                    Some(SharedString::from(svc.r#type.clone())),
                    tabs,
                    body,
                ))
            }
            ResourceDrawerTarget::ConfigMap { name, namespace } => {
                let cm = self
                    .configmaps
                    .configmaps
                    .iter()
                    .find(|c| c.name == *name && c.namespace == *namespace)?;
                let matched_events =
                    views::configmaps::filter_events_for_configmap(&self.events.events, cm);
                let tabs = vec![
                    ResourceDrawerTabDef {
                        id: ResourceDrawerTabId::Overview,
                        label: SharedString::from("Overview"),
                        badge: None,
                    },
                    ResourceDrawerTabDef {
                        id: ResourceDrawerTabId::Custom(SharedString::from("data")),
                        label: SharedString::from("Data"),
                        badge: Some(SharedString::from(cm.keys_count.to_string())),
                    },
                    ResourceDrawerTabDef {
                        id: ResourceDrawerTabId::Events,
                        label: SharedString::from("Events"),
                        badge: Some(SharedString::from(matched_events.len().to_string())),
                    },
                    ResourceDrawerTabDef {
                        id: ResourceDrawerTabId::Yaml,
                        label: SharedString::from("YAML"),
                        badge: None,
                    },
                ];
                let body = match &self.drawer.active_tab {
                    ResourceDrawerTabId::Overview => {
                        views::configmaps::render_configmap_drawer_overview(cm, theme)
                    }
                    ResourceDrawerTabId::Custom(id) if id.as_ref() == "data" => {
                        views::configmaps::render_configmap_drawer_data(cm, theme)
                    }
                    ResourceDrawerTabId::Events => {
                        views::configmaps::render_configmap_drawer_events(
                            &self.events.events,
                            events_loading,
                            cm,
                            theme,
                        )
                    }
                    ResourceDrawerTabId::Yaml => self.render_live_yaml_tab(
                        format!("{}/{}", cm.namespace, cm.name),
                        theme,
                        cx,
                    ),
                    ResourceDrawerTabId::Custom(_) => div().into_any_element(),
                };
                let status_label = if cm.immutable { "Immutable" } else { "Mutable" };
                Some((
                    SharedString::from(cm.name.clone()),
                    SharedString::from("ConfigMap"),
                    Some(SharedString::from(cm.namespace.clone())),
                    Some(SharedString::from(cm.age.clone())),
                    Some(SharedString::from(status_label)),
                    tabs,
                    body,
                ))
            }
            ResourceDrawerTarget::Secret { name, namespace } => {
                let sec = self
                    .secrets
                    .secrets
                    .iter()
                    .find(|s| s.name == *name && s.namespace == *namespace)?;
                let matched_events =
                    views::secrets::filter_events_for_secret(&self.events.events, sec);
                let tabs = vec![
                    ResourceDrawerTabDef {
                        id: ResourceDrawerTabId::Overview,
                        label: SharedString::from("Overview"),
                        badge: None,
                    },
                    ResourceDrawerTabDef {
                        id: ResourceDrawerTabId::Custom(SharedString::from("data")),
                        label: SharedString::from("Data"),
                        badge: Some(SharedString::from(sec.keys_count.to_string())),
                    },
                    ResourceDrawerTabDef {
                        id: ResourceDrawerTabId::Events,
                        label: SharedString::from("Events"),
                        badge: Some(SharedString::from(matched_events.len().to_string())),
                    },
                    ResourceDrawerTabDef {
                        id: ResourceDrawerTabId::Yaml,
                        label: SharedString::from("YAML"),
                        badge: None,
                    },
                ];
                let reveal_entity = cx.entity().clone();
                let copy_entity = cx.entity().clone();
                let body = match &self.drawer.active_tab {
                    ResourceDrawerTabId::Overview => {
                        views::secrets::render_secret_drawer_overview(sec, theme)
                    }
                    ResourceDrawerTabId::Custom(id) if id.as_ref() == "data" => {
                        views::secrets::render_secret_drawer_data(
                            sec,
                            &self.drawer.revealed_secret_keys,
                            theme,
                            move |key, _window, cx| {
                                reveal_entity.update(cx, |shell, cx| {
                                    shell.drawer.toggle_secret_key_revealed(&key);
                                    cx.notify();
                                });
                            },
                            move |value, _window, cx| {
                                copy_entity.update(cx, |_shell, cx| {
                                    cx.write_to_clipboard(ClipboardItem::new_string(value));
                                });
                            },
                        )
                    }
                    ResourceDrawerTabId::Events => views::secrets::render_secret_drawer_events(
                        &self.events.events,
                        events_loading,
                        sec,
                        theme,
                    ),
                    ResourceDrawerTabId::Yaml => self.render_live_yaml_tab(
                        format!("{}/{}", sec.namespace, sec.name),
                        theme,
                        cx,
                    ),
                    ResourceDrawerTabId::Custom(_) => div().into_any_element(),
                };
                let status_label = if sec.immutable {
                    "Immutable"
                } else {
                    "Mutable"
                };
                Some((
                    SharedString::from(sec.name.clone()),
                    SharedString::from("Secret"),
                    Some(SharedString::from(sec.namespace.clone())),
                    Some(SharedString::from(sec.age.clone())),
                    Some(SharedString::from(status_label)),
                    tabs,
                    body,
                ))
            }
            ResourceDrawerTarget::PersistentVolumeClaim { name, namespace } => {
                let pvc = self
                    .persistent_volume_claims
                    .persistent_volume_claims
                    .iter()
                    .find(|p| p.name == *name && p.namespace == *namespace)?;
                let matched_events =
                    views::persistent_volume_claims::filter_events_for_persistent_volume_claim(
                        &self.events.events,
                        pvc,
                    );
                let tabs = vec![
                    ResourceDrawerTabDef {
                        id: ResourceDrawerTabId::Overview,
                        label: SharedString::from("Overview"),
                        badge: None,
                    },
                    ResourceDrawerTabDef {
                        id: ResourceDrawerTabId::Events,
                        label: SharedString::from("Events"),
                        badge: Some(SharedString::from(matched_events.len().to_string())),
                    },
                    ResourceDrawerTabDef {
                        id: ResourceDrawerTabId::Yaml,
                        label: SharedString::from("YAML"),
                        badge: None,
                    },
                ];
                let body = match self.drawer.active_tab {
                    ResourceDrawerTabId::Overview => {
                        views::persistent_volume_claims::render_persistent_volume_claim_drawer_overview(
                            pvc, theme,
                        )
                    }
                    ResourceDrawerTabId::Events => {
                        views::persistent_volume_claims::render_persistent_volume_claim_drawer_events(
                            &matched_events,
                            events_loading,
                            theme,
                        )
                    }
                    ResourceDrawerTabId::Yaml => self.render_live_yaml_tab(
                        format!("{}/{}", pvc.namespace, pvc.name),
                        theme,
                        cx,
                    ),
                    ResourceDrawerTabId::Custom(_) => div().into_any_element(),
                };
                Some((
                    SharedString::from(pvc.name.clone()),
                    SharedString::from("PersistentVolumeClaim"),
                    Some(SharedString::from(pvc.namespace.clone())),
                    Some(SharedString::from(pvc.age.clone())),
                    Some(SharedString::from(pvc.status.clone())),
                    tabs,
                    body,
                ))
            }
            ResourceDrawerTarget::NetworkPolicy { name, namespace } => {
                let policy = self
                    .network_policies
                    .network_policies
                    .iter()
                    .find(|p| p.name == *name && p.namespace == *namespace)?;
                let matched_events = views::network_policies::filter_events_for_network_policy(
                    &self.events.events,
                    policy,
                );
                let tabs = vec![
                    ResourceDrawerTabDef {
                        id: ResourceDrawerTabId::Overview,
                        label: SharedString::from("Overview"),
                        badge: None,
                    },
                    ResourceDrawerTabDef {
                        id: ResourceDrawerTabId::Events,
                        label: SharedString::from("Events"),
                        badge: Some(SharedString::from(matched_events.len().to_string())),
                    },
                    ResourceDrawerTabDef {
                        id: ResourceDrawerTabId::Yaml,
                        label: SharedString::from("YAML"),
                        badge: None,
                    },
                ];
                let body = match self.drawer.active_tab {
                    ResourceDrawerTabId::Overview => {
                        views::network_policies::render_network_policy_drawer_overview(
                            policy, theme,
                        )
                    }
                    ResourceDrawerTabId::Events => {
                        views::network_policies::render_network_policy_drawer_events(
                            &matched_events,
                            events_loading,
                            theme,
                        )
                    }
                    ResourceDrawerTabId::Yaml => self.render_live_yaml_tab(
                        format!("{}/{}", policy.namespace, policy.name),
                        theme,
                        cx,
                    ),
                    ResourceDrawerTabId::Custom(_) => div().into_any_element(),
                };
                Some((
                    SharedString::from(policy.name.clone()),
                    SharedString::from("NetworkPolicy"),
                    Some(SharedString::from(policy.namespace.clone())),
                    Some(SharedString::from(policy.last_updated.clone())),
                    Some(SharedString::from(policy.status.clone())),
                    tabs,
                    body,
                ))
            }
            ResourceDrawerTarget::PodSecurity { name, namespace } => {
                let policy = self
                    .pod_security
                    .policies
                    .iter()
                    .find(|p| p.name == *name && p.namespace == *namespace)?;
                let matched_events = views::pod_security::filter_events_for_pod_security(
                    &self.events.events,
                    policy,
                );
                let tabs = vec![
                    ResourceDrawerTabDef {
                        id: ResourceDrawerTabId::Overview,
                        label: SharedString::from("Overview"),
                        badge: None,
                    },
                    ResourceDrawerTabDef {
                        id: ResourceDrawerTabId::Events,
                        label: SharedString::from("Events"),
                        badge: Some(SharedString::from(matched_events.len().to_string())),
                    },
                    ResourceDrawerTabDef {
                        id: ResourceDrawerTabId::Yaml,
                        label: SharedString::from("YAML"),
                        badge: None,
                    },
                ];
                let body = match self.drawer.active_tab {
                    ResourceDrawerTabId::Overview => {
                        views::pod_security::render_pod_security_drawer_overview(policy, theme)
                    }
                    ResourceDrawerTabId::Events => {
                        views::pod_security::render_pod_security_drawer_events(
                            &matched_events,
                            events_loading,
                            theme,
                        )
                    }
                    ResourceDrawerTabId::Yaml => self.render_live_yaml_tab(
                        format!("Namespace/{}", policy.namespace),
                        theme,
                        cx,
                    ),
                    ResourceDrawerTabId::Custom(_) => div().into_any_element(),
                };
                Some((
                    SharedString::from(policy.name.clone()),
                    SharedString::from("PodSecurity"),
                    Some(SharedString::from(policy.namespace.clone())),
                    Some(SharedString::from(policy.last_updated.clone())),
                    Some(SharedString::from(policy.status.clone())),
                    tabs,
                    body,
                ))
            }
            ResourceDrawerTarget::ResourceQuota { name, namespace }
            | ResourceDrawerTarget::LimitRange { name, namespace } => {
                let target_kind = target.kind();
                let policy = self.resource_quotas.policies.iter().find(|p| {
                    p.kind.eq_ignore_ascii_case(target_kind)
                        && p.name == *name
                        && p.namespace == *namespace
                })?;
                let matched_events = views::resource_quotas::filter_events_for_resource_policy(
                    &self.events.events,
                    policy,
                );
                let tabs = vec![
                    ResourceDrawerTabDef {
                        id: ResourceDrawerTabId::Overview,
                        label: SharedString::from("Overview"),
                        badge: None,
                    },
                    ResourceDrawerTabDef {
                        id: ResourceDrawerTabId::Events,
                        label: SharedString::from("Events"),
                        badge: Some(SharedString::from(matched_events.len().to_string())),
                    },
                    ResourceDrawerTabDef {
                        id: ResourceDrawerTabId::Yaml,
                        label: SharedString::from("YAML"),
                        badge: None,
                    },
                ];
                let body = match self.drawer.active_tab {
                    ResourceDrawerTabId::Overview => {
                        views::resource_quotas::render_resource_policy_drawer_overview(
                            policy, theme,
                        )
                    }
                    ResourceDrawerTabId::Events => {
                        views::resource_quotas::render_resource_policy_drawer_events(
                            &matched_events,
                            events_loading,
                            theme,
                        )
                    }
                    ResourceDrawerTabId::Yaml => self.render_live_yaml_tab(
                        format!("{}/{}", policy.namespace, policy.name),
                        theme,
                        cx,
                    ),
                    ResourceDrawerTabId::Custom(_) => div().into_any_element(),
                };
                Some((
                    SharedString::from(policy.name.clone()),
                    SharedString::from(policy.kind.clone()),
                    Some(SharedString::from(policy.namespace.clone())),
                    Some(SharedString::from(policy.last_updated.clone())),
                    Some(SharedString::from(policy.status.clone())),
                    tabs,
                    body,
                ))
            }
            ResourceDrawerTarget::Hpa { name, namespace } => {
                let hpa = self
                    .list_views
                    .hpas
                    .hpas
                    .iter()
                    .find(|h| h.name == *name && h.namespace == *namespace)?;
                let matched_events = views::hpas::filter_events_for_hpa(&self.events.events, hpa);
                let tabs = vec![
                    ResourceDrawerTabDef {
                        id: ResourceDrawerTabId::Overview,
                        label: SharedString::from("Overview"),
                        badge: None,
                    },
                    ResourceDrawerTabDef {
                        id: ResourceDrawerTabId::Events,
                        label: SharedString::from("Events"),
                        badge: Some(SharedString::from(matched_events.len().to_string())),
                    },
                    ResourceDrawerTabDef {
                        id: ResourceDrawerTabId::Yaml,
                        label: SharedString::from("YAML"),
                        badge: None,
                    },
                ];
                let body = match self.drawer.active_tab {
                    ResourceDrawerTabId::Overview => {
                        views::hpas::render_hpa_drawer_overview(hpa, theme)
                    }
                    ResourceDrawerTabId::Events => views::hpas::render_hpa_drawer_events(
                        &matched_events,
                        events_loading,
                        theme,
                    ),
                    ResourceDrawerTabId::Yaml => self.render_live_yaml_tab(
                        format!("{}/{}", hpa.namespace, hpa.name),
                        theme,
                        cx,
                    ),
                    ResourceDrawerTabId::Custom(_) => div().into_any_element(),
                };
                Some((
                    SharedString::from(hpa.name.clone()),
                    SharedString::from("HorizontalPodAutoscaler"),
                    Some(SharedString::from(hpa.namespace.clone())),
                    Some(SharedString::from(hpa.age.clone())),
                    Some(SharedString::from(hpa.status.clone())),
                    tabs,
                    body,
                ))
            }
            ResourceDrawerTarget::StorageClass { name } => {
                let sc = self
                    .list_views
                    .storage_classes
                    .classes
                    .iter()
                    .find(|c| c.name == *name)?;
                let matched_events = views::storage_classes::filter_events_for_storage_class(
                    &self.events.events,
                    sc,
                );
                let tabs = vec![
                    ResourceDrawerTabDef {
                        id: ResourceDrawerTabId::Overview,
                        label: SharedString::from("Overview"),
                        badge: None,
                    },
                    ResourceDrawerTabDef {
                        id: ResourceDrawerTabId::Events,
                        label: SharedString::from("Events"),
                        badge: Some(SharedString::from(matched_events.len().to_string())),
                    },
                    ResourceDrawerTabDef {
                        id: ResourceDrawerTabId::Yaml,
                        label: SharedString::from("YAML"),
                        badge: None,
                    },
                ];
                let body = match self.drawer.active_tab {
                    ResourceDrawerTabId::Overview => {
                        views::storage_classes::render_storage_class_drawer_overview(sc, theme)
                    }
                    ResourceDrawerTabId::Events => {
                        views::storage_classes::render_storage_class_drawer_events(
                            &matched_events,
                            events_loading,
                            theme,
                        )
                    }
                    ResourceDrawerTabId::Yaml => {
                        self.render_live_yaml_tab(format!("StorageClass/{}", sc.name), theme, cx)
                    }
                    ResourceDrawerTabId::Custom(_) => div().into_any_element(),
                };
                Some((
                    SharedString::from(sc.name.clone()),
                    SharedString::from("StorageClass"),
                    None,
                    Some(SharedString::from(sc.age.clone())),
                    None,
                    tabs,
                    body,
                ))
            }
            ResourceDrawerTarget::AdmissionWebhook { is_mutating, name } => {
                let target_kind = if *is_mutating {
                    "MutatingWebhookConfiguration"
                } else {
                    "ValidatingWebhookConfiguration"
                };
                let policy = self
                    .list_views
                    .admission_webhooks
                    .policies
                    .iter()
                    .find(|p| p.kind.eq_ignore_ascii_case(target_kind) && p.name == *name)?;
                let matched_events = views::admission_webhooks::filter_events_for_admission_webhook(
                    &self.events.events,
                    policy,
                );
                let tabs = vec![
                    ResourceDrawerTabDef {
                        id: ResourceDrawerTabId::Overview,
                        label: SharedString::from("Overview"),
                        badge: None,
                    },
                    ResourceDrawerTabDef {
                        id: ResourceDrawerTabId::Events,
                        label: SharedString::from("Events"),
                        badge: Some(SharedString::from(matched_events.len().to_string())),
                    },
                    ResourceDrawerTabDef {
                        id: ResourceDrawerTabId::Yaml,
                        label: SharedString::from("YAML"),
                        badge: None,
                    },
                ];
                let body = match self.drawer.active_tab {
                    ResourceDrawerTabId::Overview => {
                        views::admission_webhooks::render_admission_webhook_drawer_overview(
                            policy, theme,
                        )
                    }
                    ResourceDrawerTabId::Events => {
                        views::admission_webhooks::render_admission_webhook_drawer_events(
                            &matched_events,
                            events_loading,
                            theme,
                        )
                    }
                    ResourceDrawerTabId::Yaml => self.render_live_yaml_tab(
                        format!("{}/{}", policy.kind, policy.name),
                        theme,
                        cx,
                    ),
                    ResourceDrawerTabId::Custom(_) => div().into_any_element(),
                };
                Some((
                    SharedString::from(policy.name.clone()),
                    SharedString::from(policy.kind.clone()),
                    None,
                    Some(SharedString::from(policy.last_updated.clone())),
                    Some(SharedString::from(policy.status.clone())),
                    tabs,
                    body,
                ))
            }
            ResourceDrawerTarget::SecurityPolicy {
                kind,
                name,
                namespace,
            } => {
                let policy = self
                    .list_views
                    .security_overview
                    .policies
                    .iter()
                    .find(|p| {
                        p.kind.eq_ignore_ascii_case(kind)
                            && p.name == *name
                            && p.namespace == *namespace
                    })?;
                let matched_events = views::security_overview::filter_events_for_security_policy(
                    &self.events.events,
                    policy,
                );
                let tabs = vec![
                    ResourceDrawerTabDef {
                        id: ResourceDrawerTabId::Overview,
                        label: SharedString::from("Overview"),
                        badge: None,
                    },
                    ResourceDrawerTabDef {
                        id: ResourceDrawerTabId::Events,
                        label: SharedString::from("Events"),
                        badge: Some(SharedString::from(matched_events.len().to_string())),
                    },
                    ResourceDrawerTabDef {
                        id: ResourceDrawerTabId::Yaml,
                        label: SharedString::from("YAML"),
                        badge: None,
                    },
                ];
                let body = match self.drawer.active_tab {
                    ResourceDrawerTabId::Overview => {
                        views::security_overview::render_security_policy_drawer_overview(
                            policy, theme,
                        )
                    }
                    ResourceDrawerTabId::Events => {
                        views::security_overview::render_security_policy_drawer_events(
                            &matched_events,
                            events_loading,
                            theme,
                        )
                    }
                    ResourceDrawerTabId::Yaml => self.render_live_yaml_tab(
                        format!("{}/{}", policy.kind, policy.name),
                        theme,
                        cx,
                    ),
                    ResourceDrawerTabId::Custom(_) => div().into_any_element(),
                };
                let ns_opt = if policy.namespace.is_empty() || policy.namespace == "-" {
                    None
                } else {
                    Some(SharedString::from(policy.namespace.clone()))
                };
                Some((
                    SharedString::from(policy.name.clone()),
                    SharedString::from(policy.kind.clone()),
                    ns_opt,
                    Some(SharedString::from(policy.last_updated.clone())),
                    Some(SharedString::from(policy.status.clone())),
                    tabs,
                    body,
                ))
            }
        }
    }

    fn render_active_drawer(
        &self,
        window_width: f32,
        theme: &OrbitTheme,
        cx: &mut Context<Self>,
    ) -> Option<AnyElement> {
        if !self.drawer.is_open {
            return None;
        }
        let target = self.drawer.target.as_ref()?;
        let (title, kind, namespace, age, status, tabs, body) =
            self.build_drawer_target_content(target, theme, cx)?;

        let tab_entity = cx.entity().clone();
        let on_select_tab = std::rc::Rc::new(
            move |tab: ResourceDrawerTabId, _w: &mut Window, cx: &mut App| {
                tab_entity.update(cx, |shell, cx| {
                    shell.select_drawer_tab(tab.clone(), cx);
                });
            },
        );
        let close_entity = cx.entity().clone();
        let on_close = std::rc::Rc::new(move |_w: &mut Window, cx: &mut App| {
            close_entity.update(cx, |shell, cx| {
                shell.dismiss_resource_drawer(cx);
            });
        });
        let resize_entity = cx.entity().clone();
        let on_start_resize = std::rc::Rc::new(move |_w: &mut Window, cx: &mut App| {
            resize_entity.update(cx, |shell, cx| {
                shell.drawer.start_resize();
                cx.notify();
            });
        });

        render_resource_drawer(
            true,
            Some(ResourceDrawerProps {
                title,
                kind,
                namespace,
                age,
                status,
                tabs,
                active_tab: self.drawer.active_tab.clone(),
                width: self.drawer.width,
                window_width,
                on_select_tab,
                on_close,
                on_start_resize,
                body,
            }),
            theme,
        )
    }

    fn is_active_view_loading(&self) -> bool {
        is_workspace_view_loading(
            self.sidebar.active_view,
            WorkspaceLoadingFlags {
                overview: self.overview.status == crate::state::OverviewLoadStatus::Loading,
                workloads_overview: self.workloads_overview.status
                    == crate::state::WorkloadsOverviewLoadStatus::Loading,
                namespaces: self.namespaces.status == NamespaceLoadStatus::Loading,
                deployments: self.deployments.status
                    == crate::state::DeploymentsLoadStatus::Loading,
                statefulsets: self.statefulsets.status
                    == crate::state::StatefulSetsLoadStatus::Loading,
                daemonsets: self.daemonsets.status == crate::state::DaemonSetsLoadStatus::Loading,
                replicasets: self.replicasets.status
                    == crate::state::ReplicaSetsLoadStatus::Loading,
                jobs: self.jobs.status == crate::state::JobsLoadStatus::Loading,
                cronjobs: self.cronjobs.status == crate::state::CronJobsLoadStatus::Loading,
                pods: self.pods.status == crate::state::PodsLoadStatus::Loading,
                services: self.services.status == crate::state::ServicesLoadStatus::Loading,
                ingresses: self.ingresses.status == crate::state::IngressesLoadStatus::Loading,
                port_forwards: self.port_forwards.status == PortForwardsLoadStatus::Loading,
                configmaps: self.configmaps.status == crate::state::ConfigMapsLoadStatus::Loading,
                secrets: self.secrets.status == crate::state::SecretsLoadStatus::Loading,
                persistent_volumes: self.persistent_volumes.status
                    == crate::state::PersistentVolumesLoadStatus::Loading,
                persistent_volume_claims: self.persistent_volume_claims.status
                    == crate::state::PersistentVolumeClaimsLoadStatus::Loading,
                storage_overview: self.storage_overview.status
                    == crate::state::StorageOverviewLoadStatus::Loading,
                network_policies: self.network_policies.status
                    == crate::state::NetworkPoliciesLoadStatus::Loading,
                pod_security: self.pod_security.status
                    == crate::state::PodSecurityLoadStatus::Loading,
                resource_quotas: self.resource_quotas.status
                    == crate::state::ResourceQuotasLoadStatus::Loading,
                hpas: self.list_views.hpas.status == crate::state::HpasLoadStatus::Loading,
                storage_classes: self.list_views.storage_classes.status
                    == crate::state::StorageClassesLoadStatus::Loading,
                security_overview: self.list_views.security_overview.status
                    == crate::state::SecurityOverviewLoadStatus::Loading,
                admission_webhooks: self.list_views.admission_webhooks.status
                    == crate::state::AdmissionWebhooksLoadStatus::Loading,
                nodes: self.nodes.status == crate::state::NodesLoadStatus::Loading,
                events: self.events.status == EventsLoadStatus::Loading,
            },
        )
    }

    fn refresh_workspace(&mut self, cx: &mut Context<Self>) {
        self.dispatch_command(ClusterBridgeCommand::RefreshClusters);
        if let ClusterConnectionState::Connected { cluster_id, .. } = &self.state.connection_state {
            if let Some(cmd) = self.overview.begin_refresh() {
                self.dispatch_command(cmd);
            }
            if self.sidebar.active_view == WorkspaceView::WorkloadsOverview
                || self.workloads_overview.cluster_id.is_some()
            {
                let cmd = self
                    .workloads_overview
                    .begin_refresh()
                    .unwrap_or_else(|| self.workloads_overview.begin_load_for_cluster(cluster_id));
                self.dispatch_command(cmd);
            }
            let ns_cmd = self
                .namespaces
                .begin_refresh()
                .unwrap_or_else(|| self.namespaces.begin_load_for_cluster(cluster_id));
            self.dispatch_command(ns_cmd);
            if self.sidebar.active_view == WorkspaceView::Deployments
                || self.deployments.cluster_id.is_some()
            {
                let cmd = self
                    .deployments
                    .begin_refresh()
                    .unwrap_or_else(|| self.deployments.begin_load_for_cluster(cluster_id));
                self.dispatch_command(cmd);
            }
            if self.sidebar.active_view == WorkspaceView::StatefulSets
                || self.statefulsets.cluster_id.is_some()
            {
                let cmd = self
                    .statefulsets
                    .begin_refresh()
                    .unwrap_or_else(|| self.statefulsets.begin_load_for_cluster(cluster_id));
                self.dispatch_command(cmd);
            }
            if self.sidebar.active_view == WorkspaceView::DaemonSets
                || self.daemonsets.cluster_id.is_some()
            {
                let cmd = self
                    .daemonsets
                    .begin_refresh()
                    .unwrap_or_else(|| self.daemonsets.begin_load_for_cluster(cluster_id));
                self.dispatch_command(cmd);
            }
            if self.sidebar.active_view == WorkspaceView::ReplicaSets
                || self.replicasets.cluster_id.is_some()
            {
                let cmd = self
                    .replicasets
                    .begin_refresh()
                    .unwrap_or_else(|| self.replicasets.begin_load_for_cluster(cluster_id));
                self.dispatch_command(cmd);
            }
            if self.sidebar.active_view == WorkspaceView::Jobs || self.jobs.cluster_id.is_some() {
                let cmd = self
                    .jobs
                    .begin_refresh()
                    .unwrap_or_else(|| self.jobs.begin_load_for_cluster(cluster_id));
                self.dispatch_command(cmd);
            }
            if self.sidebar.active_view == WorkspaceView::CronJobs
                || self.cronjobs.cluster_id.is_some()
            {
                let cmd = self
                    .cronjobs
                    .begin_refresh()
                    .unwrap_or_else(|| self.cronjobs.begin_load_for_cluster(cluster_id));
                self.dispatch_command(cmd);
            }
            if self.sidebar.active_view == WorkspaceView::Pods || self.pods.cluster_id.is_some() {
                let cmd = self
                    .pods
                    .begin_refresh()
                    .unwrap_or_else(|| self.pods.begin_load_for_cluster(cluster_id));
                self.dispatch_command(cmd);
            }
            if self.sidebar.active_view == WorkspaceView::Services
                || self.services.cluster_id.is_some()
            {
                let cmd = self
                    .services
                    .begin_refresh()
                    .unwrap_or_else(|| self.services.begin_load_for_cluster(cluster_id));
                self.dispatch_command(cmd);
            }
            if self.sidebar.active_view == WorkspaceView::Ingresses
                || self.ingresses.cluster_id.is_some()
            {
                let cmd = self
                    .ingresses
                    .begin_refresh()
                    .unwrap_or_else(|| self.ingresses.begin_load_for_cluster(cluster_id));
                self.dispatch_command(cmd);
            }
            if self.sidebar.active_view == WorkspaceView::PortForward
                || self.port_forwards.cluster_id.is_some()
            {
                let cmd = self
                    .port_forwards
                    .begin_refresh()
                    .unwrap_or_else(|| self.port_forwards.begin_load_for_cluster(cluster_id));
                self.dispatch_command(cmd);
            }
            if self.sidebar.active_view == WorkspaceView::ConfigMaps
                || self.configmaps.cluster_id.is_some()
            {
                let cmd = self
                    .configmaps
                    .begin_refresh()
                    .unwrap_or_else(|| self.configmaps.begin_load_for_cluster(cluster_id));
                self.dispatch_command(cmd);
            }
            if self.sidebar.active_view == WorkspaceView::Secrets
                || self.secrets.cluster_id.is_some()
            {
                let cmd = self
                    .secrets
                    .begin_refresh()
                    .unwrap_or_else(|| self.secrets.begin_load_for_cluster(cluster_id));
                self.dispatch_command(cmd);
            }
            if self.sidebar.active_view == WorkspaceView::PersistentVolumes
                || self.persistent_volumes.cluster_id.is_some()
            {
                let cmd = self
                    .persistent_volumes
                    .begin_refresh()
                    .unwrap_or_else(|| self.persistent_volumes.begin_load_for_cluster(cluster_id));
                self.dispatch_command(cmd);
            }
            if self.sidebar.active_view == WorkspaceView::PersistentVolumeClaims
                || self.persistent_volume_claims.cluster_id.is_some()
            {
                let cmd = self
                    .persistent_volume_claims
                    .begin_refresh()
                    .unwrap_or_else(|| {
                        self.persistent_volume_claims
                            .begin_load_for_cluster(cluster_id)
                    });
                self.dispatch_command(cmd);
            }
            if self.sidebar.active_view == WorkspaceView::NetworkPolicies
                || self.network_policies.cluster_id.is_some()
            {
                let cmd = self
                    .network_policies
                    .begin_refresh()
                    .unwrap_or_else(|| self.network_policies.begin_load_for_cluster(cluster_id));
                self.dispatch_command(cmd);
            }
            if self.sidebar.active_view == WorkspaceView::StorageOverview
                || self.storage_overview.cluster_id.is_some()
            {
                let cmd = self
                    .storage_overview
                    .begin_refresh()
                    .unwrap_or_else(|| self.storage_overview.begin_load_for_cluster(cluster_id));
                self.dispatch_command(cmd);
            }
            if self.sidebar.active_view == WorkspaceView::PodSecurity
                || self.pod_security.cluster_id.is_some()
            {
                let cmd = self
                    .pod_security
                    .begin_refresh()
                    .unwrap_or_else(|| self.pod_security.begin_load_for_cluster(cluster_id));
                self.dispatch_command(cmd);
            }
            if self.sidebar.active_view == WorkspaceView::ResourceQuotas
                || self.resource_quotas.cluster_id.is_some()
            {
                let cmd = self
                    .resource_quotas
                    .begin_refresh()
                    .unwrap_or_else(|| self.resource_quotas.begin_load_for_cluster(cluster_id));
                self.dispatch_command(cmd);
            }
            for cmd in self
                .list_views
                .refresh_commands(self.sidebar.active_view, cluster_id)
            {
                self.dispatch_command(cmd);
            }
            if self.sidebar.active_view == WorkspaceView::Nodes || self.nodes.cluster_id.is_some() {
                let cmd = self
                    .nodes
                    .begin_refresh()
                    .unwrap_or_else(|| self.nodes.begin_load_for_cluster(cluster_id));
                self.dispatch_command(cmd);
            }
            if self.sidebar.active_view == WorkspaceView::Events || self.events.cluster_id.is_some()
            {
                let cmd = self
                    .events
                    .begin_refresh()
                    .unwrap_or_else(|| self.events.begin_load_for_cluster(cluster_id));
                self.dispatch_command(cmd);
            }
        }
        cx.notify();
    }

    fn select_cluster(&mut self, cluster_id: &str, cx: &mut Context<Self>) {
        if let Some(cmd) = self.state.begin_switch(cluster_id) {
            self.dispatch_command(cmd);
        }
        self.namespaces.close_dropdown();
        cx.notify();
    }

    fn reconnect_cluster(&mut self, cx: &mut Context<Self>) {
        if let Some(cmd) = self.state.begin_reconnect() {
            self.dispatch_command(cmd);
        }
        self.namespaces.close_dropdown();
        cx.notify();
    }

    fn disconnect_cluster(&mut self, cx: &mut Context<Self>) {
        if let Some(cmd) = disconnect_cluster_with_state(
            &mut self.state,
            &mut self.sidebar,
            &mut self.overview,
            &mut self.workloads_overview,
            &mut self.namespaces,
            &mut self.namespaces_view,
            &mut self.deployments,
            &mut self.statefulsets,
            &mut self.daemonsets,
            &mut self.replicasets,
            &mut self.jobs,
            &mut self.cronjobs,
            &mut self.pods,
            &mut self.services,
            &mut self.ingresses,
            &mut self.configmaps,
            &mut self.secrets,
            &mut self.persistent_volumes,
            &mut self.persistent_volume_claims,
            &mut self.network_policies,
            &mut self.storage_overview,
            &mut self.pod_security,
            &mut self.resource_quotas,
            &mut self.list_views,
            &mut self.nodes,
            &mut self.events,
            &mut self.tabs,
            &mut self.drawer,
        ) {
            self.dispatch_command(cmd);
        }
        self.namespaces.close_dropdown();
        self.context_menu.close();
        cx.notify();
    }

    fn select_theme_mode(&mut self, mode: ThemeMode, window: &mut Window, cx: &mut Context<Self>) {
        self.settings_dropdown.close();
        let updated = OrbitTheme::get(cx).with_mode(mode, window.appearance());
        apply_theme(updated, window, cx);
        self.settings_error = None;
        self.dispatch_command(ClusterBridgeCommand::SaveThemeMode {
            mode: mode.id().to_string(),
        });
        cx.notify();
    }

    fn select_ui_style(&mut self, style: UiStyle, window: &mut Window, cx: &mut Context<Self>) {
        self.settings_dropdown.close();
        let updated = OrbitTheme::get(cx).with_ui_style(style, window.appearance());
        apply_theme(updated, window, cx);
        self.settings_error = None;
        self.dispatch_command(ClusterBridgeCommand::SaveUiStyle {
            style: style.id().to_string(),
        });
        cx.notify();
    }

    fn select_accent_color(
        &mut self,
        accent: AccentColor,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let updated = OrbitTheme::get(cx).with_accent_color(accent, window.appearance());
        apply_theme(updated, window, cx);
        self.settings_error = None;
        self.dispatch_command(ClusterBridgeCommand::SaveAccentColor {
            accent: accent.id().to_string(),
        });
        cx.notify();
    }

    fn toggle_experience_features(&mut self, enabled: bool, cx: &mut Context<Self>) {
        self.enable_experience_features = enabled;
        self.settings_error = None;
        self.dispatch_command(ClusterBridgeCommand::SaveEnableExperienceFeatures { enabled });
        cx.notify();
    }

    fn reset_appearance_defaults(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.settings_dropdown.close();
        let updated = OrbitTheme::get(cx).with_appearance(
            ThemeMode::Dark,
            UiStyle::Flat,
            AccentColor::Blue,
            window.appearance(),
        );
        apply_theme(updated, window, cx);
        self.settings_error = None;
        self.dispatch_command(ClusterBridgeCommand::ResetAppearanceDefaults);
        cx.notify();
    }

    pub fn default_window_bounds(cx: &App) -> Bounds<gpui::Pixels> {
        Bounds::centered(
            None,
            size(px(Self::DEFAULT_WIDTH), px(Self::DEFAULT_HEIGHT)),
            cx,
        )
    }

    pub fn window_options_for_theme(
        bounds: Bounds<gpui::Pixels>,
        theme: &OrbitTheme,
    ) -> WindowOptions {
        WindowOptions {
            window_bounds: Some(WindowBounds::Windowed(bounds)),
            window_background: theme.style.window_background,
            titlebar: Some(gpui::TitlebarOptions {
                title: Some(SharedString::from(Self::WINDOW_TITLE)),
                appears_transparent: true,
                traffic_light_position: Some(gpui::point(px(12.0), px(10.0))),
            }),
            window_decorations: Some(WindowDecorations::Client),
            is_movable: true,
            is_resizable: true,
            ..Default::default()
        }
    }

    fn render_activity_bar(&self, cx: &mut Context<Self>) -> AnyElement {
        let theme = OrbitTheme::get(cx).clone();
        let nav = theme.nav_selection();
        let hover_bg = theme.surface.bg_hover;
        let is_settings = self.sidebar.active_view == WorkspaceView::Settings;

        let mut categories_col = div().flex().flex_col().items_center().gap_1p5().w_full();
        for (idx, category) in SidebarCategory::ALL.iter().copied().enumerate() {
            let is_active = !is_settings
                && self.sidebar.active_category == category
                && !self.sidebar.is_collapsed;
            let is_selected_category = !is_settings && self.sidebar.active_category == category;

            categories_col = categories_col.child(
                div()
                    .id(("activity-cat-btn", idx))
                    .relative()
                    .flex()
                    .flex_col()
                    .items_center()
                    .justify_center()
                    .w(px(40.0))
                    .h(px(36.0))
                    .rounded(theme.style.radius_control)
                    .cursor_pointer()
                    .when(is_active, |el| el.bg(nav.active_bg))
                    .hover(move |style| style.bg(hover_bg))
                    .on_click(cx.listener(move |this, _event, _window, cx| {
                        let prev_view = this.sidebar.active_view;
                        this.sidebar.toggle_category(category);
                        if this.sidebar.active_view != prev_view {
                            this.select_workspace_view(this.sidebar.active_view, cx);
                        } else {
                            cx.notify();
                        }
                    }))
                    .when(is_selected_category, |el| {
                        el.child(
                            div()
                                .absolute()
                                .left(px(-8.0))
                                .top(px(6.0))
                                .bottom(px(6.0))
                                .w(px(2.0))
                                .rounded_full()
                                .bg(nav.indicator),
                        )
                    })
                    .child(render_icon(
                        category.icon(),
                        px(18.0),
                        if is_selected_category {
                            nav.active_fg
                        } else {
                            theme.text.secondary
                        },
                    )),
            );
        }

        div()
            .id("orbit-activity-bar")
            .w(px(ACTIVITY_BAR_WIDTH))
            .flex_shrink_0()
            .h_full()
            .flex()
            .flex_col()
            .items_center()
            .py_3()
            .gap_3()
            .bg(theme.surface.bg_sidebar)
            .border_r_1()
            .border_color(theme.border.default)
            .child(categories_col)
            .child(
                div()
                    .mt_auto()
                    .flex()
                    .flex_col()
                    .items_center()
                    .gap_1p5()
                    .when(ORBIT_SHELL_CHROME.show_activity_settings_btn, |el| {
                        el.child(
                            div()
                                .id("activity-settings-btn")
                                .relative()
                                .flex()
                                .items_center()
                                .justify_center()
                                .w(px(40.0))
                                .h(px(34.0))
                                .rounded(theme.style.radius_control)
                                .cursor_pointer()
                                .when(is_settings, |el| el.bg(nav.active_bg))
                                .hover(move |style| style.bg(hover_bg))
                                .tooltip(|_, cx| cx.new(|_| TextTooltip::new("Settings")).into())
                                .on_click(cx.listener(|this, _event, _window, cx| {
                                    this.select_workspace_view(WorkspaceView::Settings, cx);
                                }))
                                .when(is_settings, |el| {
                                    el.child(
                                        div()
                                            .absolute()
                                            .left(px(-8.0))
                                            .top(px(6.0))
                                            .bottom(px(6.0))
                                            .w(px(2.0))
                                            .rounded_full()
                                            .bg(nav.indicator),
                                    )
                                })
                                .child(render_icon(
                                    Icon::Settings,
                                    px(18.0),
                                    if is_settings {
                                        nav.active_fg
                                    } else {
                                        theme.text.secondary
                                    },
                                )),
                        )
                    })
                    .child({
                        let tooltip_label = theme_mode_tooltip(theme.mode, theme.is_dark);
                        div()
                            .id("activity-theme-toggle-btn")
                            .relative()
                            .flex()
                            .items_center()
                            .justify_center()
                            .w(px(40.0))
                            .h(px(34.0))
                            .rounded(theme.style.radius_control)
                            .cursor_pointer()
                            .hover(move |style| style.bg(hover_bg))
                            .tooltip(move |_, cx| {
                                cx.new(|_| TextTooltip::new(tooltip_label.clone())).into()
                            })
                            .on_click(cx.listener(|this, _event, window, cx| {
                                let next_mode = OrbitTheme::get(cx).mode.next();
                                this.select_theme_mode(next_mode, window, cx);
                            }))
                            .child(render_icon(
                                theme_mode_icon(theme.mode),
                                px(18.0),
                                theme.text.secondary,
                            ))
                    })
                    .child({
                        let unread = self.notifications.unread_count();
                        let is_open = self.active_shell_drawer == ActiveShellDrawer::Notifications;
                        let tooltip_label = notifications_tooltip(unread);
                        div()
                            .id("activity-notifications-btn")
                            .relative()
                            .flex()
                            .items_center()
                            .justify_center()
                            .w(px(40.0))
                            .h(px(34.0))
                            .rounded(theme.style.radius_control)
                            .cursor_pointer()
                            .when(is_open, |el| el.bg(nav.active_bg))
                            .hover(move |style| style.bg(hover_bg))
                            .tooltip(move |_, cx| {
                                cx.new(|_| TextTooltip::new(tooltip_label.clone())).into()
                            })
                            .on_click(cx.listener(|this, _event, _window, cx| {
                                this.active_shell_drawer
                                    .toggle(ActiveShellDrawer::Notifications);
                                cx.notify();
                            }))
                            .when(is_open, |el| {
                                el.child(
                                    div()
                                        .absolute()
                                        .left(px(-8.0))
                                        .top(px(6.0))
                                        .bottom(px(6.0))
                                        .w(px(2.0))
                                        .rounded_full()
                                        .bg(nav.indicator),
                                )
                            })
                            .child(render_icon(
                                Icon::Bell,
                                px(18.0),
                                if is_open {
                                    nav.active_fg
                                } else {
                                    theme.text.secondary
                                },
                            ))
                            .when(unread > 0, |el| {
                                el.child(
                                    div()
                                        .absolute()
                                        .top(px(6.0))
                                        .right(px(8.0))
                                        .w(px(7.0))
                                        .h(px(7.0))
                                        .rounded_full()
                                        .bg(theme.semantic.danger.solid),
                                )
                            })
                    })
                    .child({
                        let is_open = self.active_shell_drawer == ActiveShellDrawer::UserProfile;
                        div()
                            .id("activity-profile-btn")
                            .relative()
                            .flex()
                            .items_center()
                            .justify_center()
                            .w(px(40.0))
                            .h(px(34.0))
                            .rounded(theme.style.radius_control)
                            .cursor_pointer()
                            .when(is_open, |el| el.bg(nav.active_bg))
                            .hover(move |style| style.bg(hover_bg))
                            .tooltip(|_, cx| cx.new(|_| TextTooltip::new("User Profile")).into())
                            .on_click(cx.listener(|this, _event, _window, cx| {
                                this.active_shell_drawer
                                    .toggle(ActiveShellDrawer::UserProfile);
                                cx.notify();
                            }))
                            .when(is_open, |el| {
                                el.child(
                                    div()
                                        .absolute()
                                        .left(px(-8.0))
                                        .top(px(6.0))
                                        .bottom(px(6.0))
                                        .w(px(2.0))
                                        .rounded_full()
                                        .bg(nav.indicator),
                                )
                            })
                            .child(render_icon(
                                Icon::User,
                                px(18.0),
                                if is_open {
                                    nav.active_fg
                                } else {
                                    theme.text.secondary
                                },
                            ))
                    })
                    .child(
                        div()
                            .id("activity-github-btn")
                            .relative()
                            .flex()
                            .items_center()
                            .justify_center()
                            .w(px(40.0))
                            .h(px(34.0))
                            .rounded(theme.style.radius_control)
                            .cursor_pointer()
                            .hover(move |style| style.bg(hover_bg))
                            .tooltip(|_, cx| {
                                cx.new(|_| TextTooltip::new("GitHub Repository")).into()
                            })
                            .on_click(|_event, _window, cx| {
                                cx.open_url(ORBIT_GITHUB_REPO_URL);
                            })
                            .child(render_icon(Icon::GitBranch, px(18.0), theme.text.secondary)),
                    ),
            )
            .into_any_element()
    }

    fn render_clusters_sidebar_content(&self, cx: &mut Context<Self>) -> AnyElement {
        let theme = OrbitTheme::get(cx).clone();
        let nav = theme.nav_selection();
        let hover_bg = theme.surface.bg_hover;
        let mono_font = theme.typography.mono_font(FontWeight::NORMAL);

        if self.state.clusters.is_empty() {
            return div()
                .flex()
                .flex_col()
                .gap_2()
                .p_2()
                .child(
                    apply_text_step(div(), theme.typography.size_xs)
                        .px_1()
                        .text_color(theme.text.muted)
                        .child("No kubeconfig contexts found."),
                )
                .child(
                    theme
                        .style_control(div())
                        .id("sidebar-empty-add-cluster-btn")
                        .px_2p5()
                        .py_1p5()
                        .bg(theme.accent.accent)
                        .text_color(crate::theme::rgb_hex(0xffffff))
                        .cursor_pointer()
                        .hover(|s| s.bg(theme.accent.hover))
                        .flex()
                        .items_center()
                        .justify_center()
                        .gap_1p5()
                        .text_xs()
                        .font_weight(FontWeight::MEDIUM)
                        .on_click(cx.listener(|this, _event, window, cx| {
                            this.open_add_cluster_modal(window, cx);
                        }))
                        .child(render_icon(
                            Icon::Plus,
                            px(12.0),
                            crate::theme::rgb_hex(0xffffff),
                        ))
                        .child("Add Cluster"),
                )
                .into_any_element();
        }

        let mut list = div().flex().flex_col().gap_1();
        for (idx, cluster) in self.state.clusters.iter().enumerate() {
            let cluster_id = cluster.id.clone();
            let click_cluster_id = cluster_id.clone();
            let menu_cluster_id = cluster_id.clone();
            let is_active = self.state.active_cluster_id.as_deref() == Some(cluster_id.as_str());
            let can_reconnect = is_active
                && !matches!(
                    self.state.connection_state,
                    ClusterConnectionState::Connecting { .. }
                );
            let display = cluster_display_label(cluster);
            let dot_color = cluster_dot_color(cluster, &theme);

            list = list.child(
                div()
                    .id(("sidebar-cluster-item", idx))
                    .flex()
                    .items_center()
                    .justify_between()
                    .gap_2()
                    .px_2p5()
                    .py_1p5()
                    .rounded(theme.style.radius_control)
                    .cursor_pointer()
                    .when(is_active, |el| el.bg(nav.active_bg))
                    .hover(move |style| style.bg(hover_bg))
                    .on_click(cx.listener(move |this, _event, _window, cx| {
                        this.select_cluster(&click_cluster_id, cx);
                    }))
                    .on_mouse_down(
                        MouseButton::Right,
                        cx.listener(move |this, event: &gpui::MouseDownEvent, _window, cx| {
                            cx.stop_propagation();
                            let connect_entity = cx.entity().clone();
                            let connect_id = menu_cluster_id.clone();
                            let reconnect_entity = cx.entity().clone();
                            let disconnect_entity = cx.entity().clone();
                            let entries = build_cluster_sidebar_context_menu_entries(
                                is_active,
                                can_reconnect,
                                move |_window, cx| {
                                    connect_entity.update(cx, |shell, cx| {
                                        shell.select_cluster(&connect_id, cx);
                                    });
                                },
                                move |_window, cx| {
                                    reconnect_entity.update(cx, |shell, cx| {
                                        shell.reconnect_cluster(cx);
                                    });
                                },
                                move |_window, cx| {
                                    disconnect_entity.update(cx, |shell, cx| {
                                        shell.disconnect_cluster(cx);
                                    });
                                },
                            );
                            this.open_context_menu(event.position, entries, cx);
                        }),
                    )
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_2()
                            .min_w(px(0.0))
                            .child(
                                div()
                                    .w(px(8.0))
                                    .h(px(8.0))
                                    .rounded_full()
                                    .flex_shrink_0()
                                    .bg(dot_color),
                            )
                            .child(
                                div()
                                    .flex()
                                    .flex_col()
                                    .min_w(px(0.0))
                                    .child(
                                        apply_text_step(div(), theme.typography.size_xs)
                                            .font_weight(if is_active {
                                                FontWeight::SEMIBOLD
                                            } else {
                                                FontWeight::MEDIUM
                                            })
                                            .text_color(if is_active {
                                                nav.active_fg
                                            } else {
                                                theme.text.primary
                                            })
                                            .child(display),
                                    )
                                    .when_some(cluster.server_url.clone(), |el, url| {
                                        el.child(
                                            apply_text_step(div(), theme.typography.size_3xs)
                                                .font(mono_font.clone())
                                                .text_color(theme.text.muted)
                                                .child(url),
                                        )
                                    }),
                            ),
                    ),
            );
        }

        list = list.child(
            div()
                .id("sidebar-add-cluster-btn")
                .mt_1()
                .flex()
                .items_center()
                .gap_2()
                .px_2p5()
                .py_1p5()
                .rounded(theme.style.radius_control)
                .cursor_pointer()
                .text_xs()
                .font_weight(FontWeight::MEDIUM)
                .text_color(theme.accent.accent)
                .hover(move |style| style.bg(hover_bg))
                .on_click(cx.listener(|this, _event, window, cx| {
                    this.open_add_cluster_modal(window, cx);
                }))
                .child(render_icon(Icon::Plus, px(12.0), theme.accent.accent))
                .child("Add Cluster"),
        );

        list.into_any_element()
    }

    fn render_nav_items_sidebar_content(
        &self,
        category: SidebarCategory,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let theme = OrbitTheme::get(cx).clone();
        let nav = theme.nav_selection();
        let hover_bg = theme.surface.bg_hover;
        let mut list = div().flex().flex_col().gap_1();

        for (idx, item) in category.nav_items().iter().copied().enumerate() {
            let is_active = self.sidebar.active_view == item;
            let icon_color = nav_item_icon_color(item, is_active, &theme);

            list = list.child(
                div()
                    .id(("sidebar-nav-item", idx))
                    .flex()
                    .items_center()
                    .justify_between()
                    .gap_2()
                    .px_2p5()
                    .py_1p5()
                    .rounded(theme.style.radius_control)
                    .cursor_pointer()
                    .when(is_active, |el| el.bg(nav.active_bg))
                    .hover(move |style| style.bg(hover_bg))
                    .on_click(cx.listener(move |this, _event, _window, cx| {
                        this.select_workspace_view(item, cx);
                    }))
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_2()
                            .child(render_icon(item.icon(), px(15.0), icon_color))
                            .child(
                                apply_text_step(div(), theme.typography.size_xs)
                                    .font_weight(if is_active {
                                        FontWeight::SEMIBOLD
                                    } else {
                                        FontWeight::MEDIUM
                                    })
                                    .text_color(if is_active {
                                        nav.active_fg
                                    } else {
                                        theme.text.primary
                                    })
                                    .child(item.label()),
                            ),
                    ),
            );
        }

        list.into_any_element()
    }

    fn render_sidebar_panel(&self, cx: &mut Context<Self>) -> Option<AnyElement> {
        let category = self.sidebar.visible_panel_category()?;
        let theme = OrbitTheme::get(cx).clone();

        let body = if category == SidebarCategory::Clusters {
            self.render_clusters_sidebar_content(cx)
        } else {
            self.render_nav_items_sidebar_content(category, cx)
        };

        Some(
            div()
                .id("orbit-sidebar-panel")
                .relative()
                .w(px(self.sidebar.panel_width))
                .flex_shrink_0()
                .h_full()
                .flex()
                .flex_col()
                .bg(theme.surface.bg_sidebar)
                .border_r_1()
                .border_color(theme.border.default)
                .child(
                    div()
                        .h(px(Self::HEADER_HEIGHT))
                        .flex_shrink_0()
                        .px_3()
                        .border_b_1()
                        .border_color(theme.border.default)
                        .flex()
                        .items_center()
                        .child(
                            apply_text_step(div(), theme.typography.size_xs)
                                .font_weight(FontWeight::SEMIBOLD)
                                .text_color(theme.text.muted)
                                .child(category.label().to_ascii_uppercase()),
                        ),
                )
                .child(
                    div()
                        .id("sidebar-panel-scroll")
                        .flex_1()
                        .overflow_y_scroll()
                        .p_2()
                        .child(body),
                )
                .child(
                    div()
                        .id("sidebar-resize-handle")
                        .absolute()
                        .top_0()
                        .bottom_0()
                        .right_0()
                        .w(px(6.0))
                        .cursor(CursorStyle::ResizeLeftRight)
                        .on_mouse_down(
                            MouseButton::Left,
                            cx.listener(|this, _event, _window, cx| {
                                this.sidebar.begin_resize();
                                cx.notify();
                            }),
                        ),
                )
                .into_any_element(),
        )
    }

    fn render_header(&self, cx: &mut Context<Self>) -> AnyElement {
        let theme = OrbitTheme::get(cx).clone();
        let presentation = format_cluster_status(&self.state, &theme);
        let cluster_chevron = dropdown_chevron_icon(self.state.dropdown_open);
        let control_hover = theme.surface.control_hover;
        let is_refreshing = self.is_active_view_loading();

        div()
            .id("orbit-header-toolbar")
            .h(px(Self::HEADER_HEIGHT))
            .flex_shrink_0()
            .px_4()
            .border_b_1()
            .border_color(theme.border.default)
            .bg(theme.surface.bg_panel)
            .flex()
            .items_center()
            .justify_between()
            .gap_4()
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_3()
                    .child(
                        theme
                            .style_control(div().id("cluster-switcher-btn"))
                            .flex()
                            .items_center()
                            .gap_2p5()
                            .h(theme.typography.control_height_sm)
                            .px(theme.typography.control_padding_x_md)
                            .hover(move |style| style.bg(control_hover))
                            .cursor_pointer()
                            .on_click(cx.listener(|this, _event, _window, cx| {
                                this.dismiss_context_menu();
                                this.namespaces.close_dropdown();
                                this.settings_dropdown.close();
                                this.state.toggle_dropdown();
                                cx.notify();
                            }))
                            .child(
                                div()
                                    .w(px(8.0))
                                    .h(px(8.0))
                                    .rounded_full()
                                    .bg(presentation.color),
                            )
                            .child(
                                apply_text_step(div(), theme.typography.size_sm)
                                    .font_weight(FontWeight::MEDIUM)
                                    .child(presentation.pill_label),
                            )
                            .child(
                                apply_text_step(div(), theme.typography.size_xs)
                                    .text_color(theme.text.muted)
                                    .child(presentation.status_label),
                            )
                            .child(render_icon(cluster_chevron, px(13.0), theme.text.secondary)),
                    )
                    .when(
                        matches!(
                            self.state.connection_state,
                            ClusterConnectionState::Connected { .. }
                        ),
                        |el| {
                            let ns_pres = namespace_trigger_presentation(&self.namespaces, &theme);
                            let ns_chevron = dropdown_chevron_icon(self.namespaces.dropdown_open);
                            el.child(
                                theme
                                    .style_control(div().id("namespace-selector-btn"))
                                    .flex()
                                    .items_center()
                                    .gap_2()
                                    .h(theme.typography.control_height_sm)
                                    .px(theme.typography.control_padding_x_md)
                                    .hover(move |style| style.bg(control_hover))
                                    .cursor_pointer()
                                    .on_click(cx.listener(|this, _event, window, cx| {
                                        this.dismiss_context_menu();
                                        this.state.close_dropdown();
                                        this.settings_dropdown.close();
                                        this.namespaces.toggle_dropdown();
                                        if this.namespaces.dropdown_open {
                                            this.namespace_search_focus.focus(window);
                                        }
                                        cx.notify();
                                    }))
                                    .child(render_icon(
                                        Icon::Layers,
                                        px(13.0),
                                        ns_pres.indicator_color,
                                    ))
                                    .child(
                                        apply_text_step(div(), theme.typography.size_sm)
                                            .font_weight(FontWeight::MEDIUM)
                                            .text_color(if ns_pres.is_filtered {
                                                theme.text.primary
                                            } else {
                                                theme.text.secondary
                                            })
                                            .child(ns_pres.label),
                                    )
                                    .when(ns_pres.is_loading, |el| {
                                        el.child(
                                            apply_text_step(div(), theme.typography.size_xs)
                                                .text_color(theme.text.muted)
                                                .child("…"),
                                        )
                                    })
                                    .when(ns_pres.is_error, |el| {
                                        el.child(render_icon(
                                            Icon::TriangleAlert,
                                            px(12.0),
                                            theme.semantic.danger.solid,
                                        ))
                                    })
                                    .child(render_icon(ns_chevron, px(13.0), theme.text.secondary)),
                            )
                        },
                    ),
            )
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_2()
                    .child(
                        theme
                            .style_control(div().id("command-palette-trigger-btn"))
                            .flex()
                            .items_center()
                            .gap_2()
                            .h(theme.typography.control_height_sm)
                            .px(theme.typography.control_padding_x_md)
                            .hover(move |style| style.bg(control_hover))
                            .cursor_pointer()
                            .tooltip(|_window, cx| {
                                cx.new(|_| {
                                    TextTooltip::new(format!(
                                        "Command Palette ({})",
                                        primary_shortcut_label("K")
                                    ))
                                })
                                .into()
                            })
                            .on_click(cx.listener(|this, _event, window, cx| {
                                this.toggle_command_palette(window, cx);
                            }))
                            .child(render_icon(Icon::Search, px(13.0), theme.text.muted))
                            .child(
                                apply_text_step(div(), theme.typography.size_xs)
                                    .text_color(theme.text.muted)
                                    .child("Search..."),
                            )
                            .child(
                                apply_text_step(div(), theme.typography.size_3xs)
                                    .px_1p5()
                                    .py(px(1.0))
                                    .rounded(theme.style.radius_control)
                                    .bg(theme.surface.bg_panel)
                                    .border_1()
                                    .border_color(theme.border.default)
                                    .font(theme.typography.mono_font(FontWeight::MEDIUM))
                                    .text_color(theme.text.muted)
                                    .child(primary_shortcut_label("K")),
                            ),
                    )
                    .child(
                        theme
                            .style_control(div().id("refresh-clusters-btn"))
                            .flex()
                            .items_center()
                            .gap_1p5()
                            .h(theme.typography.control_height_sm)
                            .px(theme.typography.control_padding_x_md)
                            .when(is_refreshing, |el| el.opacity(0.6))
                            .when(!is_refreshing, |el| {
                                el.hover(move |style| style.bg(control_hover))
                                    .cursor_pointer()
                            })
                            .tooltip(|_window, cx| {
                                cx.new(|_| TextTooltip::new("Refresh workspace (R)")).into()
                            })
                            .on_click(cx.listener(|this, _event, _window, cx| {
                                if !this.is_active_view_loading() {
                                    this.refresh_workspace(cx);
                                }
                            }))
                            .child(render_icon(
                                Icon::RefreshCw,
                                px(13.0),
                                if is_refreshing {
                                    theme.text.muted
                                } else {
                                    theme.text.secondary
                                },
                            ))
                            .child(
                                apply_text_step(div(), theme.typography.size_sm)
                                    .font_weight(FontWeight::MEDIUM)
                                    .text_color(if is_refreshing {
                                        theme.text.muted
                                    } else {
                                        theme.text.primary
                                    })
                                    .child(if is_refreshing {
                                        "Refreshing..."
                                    } else {
                                        "Refresh"
                                    }),
                            ),
                    )
                    .child(
                        theme
                            .style_control(div().id("switch-to-old-ui"))
                            .flex()
                            .items_center()
                            .gap_1p5()
                            .h(theme.typography.control_height_sm)
                            .px(theme.typography.control_padding_x_md)
                            .hover(move |style| style.bg(control_hover))
                            .cursor_pointer()
                            .on_click(cx.listener(|_this, _event, _window, cx| {
                                cx.quit();
                            }))
                            .child(render_icon(Icon::LogOut, px(13.0), theme.text.secondary))
                            .child(
                                apply_text_step(div(), theme.typography.size_sm)
                                    .font_weight(FontWeight::MEDIUM)
                                    .child("Switch to Old UI"),
                            ),
                    ),
            )
            .into_any_element()
    }

    fn render_error_banner(&self, cx: &mut Context<Self>) -> Option<AnyElement> {
        if self.state.error_banner_dismissed {
            return None;
        }
        let message = self.state.kubeconfig_error.as_ref()?.clone();
        let theme = OrbitTheme::get(cx).clone();
        let control_hover = theme.surface.control_hover;

        Some(
            div()
                .flex()
                .items_center()
                .justify_between()
                .gap_3()
                .px_4()
                .py_2p5()
                .rounded(theme.style.radius_control)
                .bg(theme.semantic.danger.soft)
                .border_1()
                .border_color(theme.semantic.danger.solid)
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap_2()
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .gap_1p5()
                                .child(render_icon(
                                    Icon::CircleAlert,
                                    px(13.0),
                                    theme.semantic.danger.solid,
                                ))
                                .child(
                                    apply_text_step(div(), theme.typography.size_xs)
                                        .font_weight(FontWeight::SEMIBOLD)
                                        .text_color(theme.semantic.danger.solid)
                                        .child("KUBECONFIG ERROR"),
                                ),
                        )
                        .child(
                            apply_text_step(div(), theme.typography.size_xs)
                                .text_color(theme.text.primary)
                                .child(message),
                        ),
                )
                .child(
                    div()
                        .id("dismiss-error-banner-btn")
                        .flex()
                        .items_center()
                        .gap_1()
                        .px_2()
                        .py_1()
                        .rounded(theme.style.radius_control)
                        .cursor_pointer()
                        .text_color(theme.text.secondary)
                        .hover(move |style| style.bg(control_hover))
                        .on_click(cx.listener(|this, _event, _window, cx| {
                            this.state.dismiss_error_banner();
                            cx.notify();
                        }))
                        .child(render_icon(Icon::X, px(12.0), theme.text.secondary))
                        .child(apply_text_step(div(), theme.typography.size_xs).child("Dismiss")),
                )
                .into_any_element(),
        )
    }

    fn render_dropdown_overlay(&self, cx: &mut Context<Self>) -> Option<AnyElement> {
        if !self.state.dropdown_open {
            return None;
        }

        let theme = OrbitTheme::get(cx).clone();
        let control_hover = theme.surface.control_hover;
        let mono_font = theme.typography.mono_font(FontWeight::NORMAL);
        let mut list = div().flex().flex_col().gap_1().p_1p5();

        if self.state.clusters.is_empty() {
            list = list.child(
                apply_text_step(div(), theme.typography.size_xs)
                    .px_3()
                    .py_4()
                    .text_color(theme.text.muted)
                    .child("No kubeconfig contexts found in ~/.kube/config or custom kubeconfig paths."),
            );
        } else {
            for (idx, cluster) in self.state.clusters.iter().enumerate() {
                let cluster_id = cluster.id.clone();
                let is_active =
                    self.state.active_cluster_id.as_deref() == Some(cluster_id.as_str());
                let display = cluster_display_label(cluster);
                let show_raw = display != cluster.name;
                let dot_color = cluster_dot_color(cluster, &theme);

                list = list.child(
                    div()
                        .id(("dropdown-cluster-row", idx))
                        .flex()
                        .items_center()
                        .justify_between()
                        .gap_3()
                        .px_3()
                        .py_2()
                        .rounded(theme.style.radius_control)
                        .cursor_pointer()
                        .when(is_active, |el| el.bg(theme.accent.soft))
                        .hover(move |style| style.bg(control_hover))
                        .on_click(cx.listener(move |this, _event, _window, cx| {
                            this.select_cluster(&cluster_id, cx);
                        }))
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .gap_2p5()
                                .child(div().w(px(8.0)).h(px(8.0)).rounded_full().bg(dot_color))
                                .child(
                                    div()
                                        .flex()
                                        .flex_col()
                                        .gap_0p5()
                                        .child(
                                            div()
                                                .flex()
                                                .items_center()
                                                .gap_2()
                                                .child(
                                                    apply_text_step(
                                                        div(),
                                                        theme.typography.size_sm,
                                                    )
                                                    .font_weight(FontWeight::MEDIUM)
                                                    .child(display),
                                                )
                                                .when(show_raw, |el| {
                                                    el.child(
                                                        apply_text_step(
                                                            div(),
                                                            theme.typography.size_xs,
                                                        )
                                                        .font(mono_font.clone())
                                                        .text_color(theme.text.muted)
                                                        .child(cluster.name.clone()),
                                                    )
                                                }),
                                        )
                                        .when_some(cluster.server_url.clone(), |el, url| {
                                            el.child(
                                                apply_text_step(div(), theme.typography.size_xs)
                                                    .font(mono_font.clone())
                                                    .text_color(theme.text.secondary)
                                                    .child(url),
                                            )
                                        }),
                                ),
                        )
                        .when(is_active, |el| {
                            el.child(
                                div()
                                    .flex()
                                    .items_center()
                                    .gap_1()
                                    .child(render_icon(Icon::Check, px(12.0), theme.accent.accent))
                                    .child(
                                        apply_text_step(div(), theme.typography.size_xs)
                                            .font_weight(FontWeight::SEMIBOLD)
                                            .text_color(theme.accent.accent)
                                            .child("Active"),
                                    ),
                            )
                        }),
                );
            }
        }

        let dropdown_left = if self.sidebar.is_collapsed {
            ACTIVITY_BAR_WIDTH + 60.0
        } else {
            ACTIVITY_BAR_WIDTH + self.sidebar.panel_width + 60.0
        };

        Some(
            div()
                .absolute()
                .inset_0()
                .child(
                    div()
                        .id("cluster-dropdown-backdrop")
                        .absolute()
                        .inset_0()
                        .on_click(cx.listener(|this, _event, _window, cx| {
                            this.state.close_dropdown();
                            cx.notify();
                        })),
                )
                .child(
                    theme
                        .style_overlay(div().id("cluster-dropdown-menu"))
                        .absolute()
                        .top(px(Self::header_dropdown_top_offset()))
                        .left(px(dropdown_left))
                        .w(px(400.0))
                        .max_h(px(420.0))
                        .overflow_y_scroll()
                        .on_click(|_event, _window, cx| {
                            cx.stop_propagation();
                        })
                        .child(
                            apply_text_step(div(), theme.typography.size_xs)
                                .px_3()
                                .py_2()
                                .border_b_1()
                                .border_color(theme.border.default)
                                .font_weight(FontWeight::SEMIBOLD)
                                .text_color(theme.text.muted)
                                .child(format!(
                                    "KUBECONFIG CONTEXTS ({})",
                                    self.state.clusters.len()
                                )),
                        )
                        .child(list)
                        .child(
                            div()
                                .border_t_1()
                                .border_color(theme.border.default)
                                .flex()
                                .items_center()
                                .justify_between()
                                .child(
                                    div()
                                        .id("dropdown-add-cluster-btn")
                                        .px_3()
                                        .py_2()
                                        .flex()
                                        .items_center()
                                        .gap_2()
                                        .cursor_pointer()
                                        .text_xs()
                                        .font_weight(FontWeight::MEDIUM)
                                        .text_color(theme.accent.accent)
                                        .hover(move |s| s.bg(control_hover))
                                        .on_click(cx.listener(|this, _event, window, cx| {
                                            this.open_add_cluster_modal(window, cx);
                                        }))
                                        .child(render_icon(
                                            Icon::Plus,
                                            px(12.0),
                                            theme.accent.accent,
                                        ))
                                        .child("Add Cluster"),
                                )
                                .when(dropdown_can_disconnect(&self.state), |el| {
                                    el.child(
                                        div()
                                            .id("dropdown-disconnect-cluster-btn")
                                            .px_3()
                                            .py_2()
                                            .flex()
                                            .items_center()
                                            .gap_1p5()
                                            .cursor_pointer()
                                            .text_xs()
                                            .font_weight(FontWeight::MEDIUM)
                                            .text_color(theme.semantic.danger.solid)
                                            .hover(move |s| s.bg(control_hover))
                                            .on_click(cx.listener(|this, _event, _window, cx| {
                                                this.disconnect_cluster(cx);
                                            }))
                                            .child(render_icon(
                                                Icon::Unplug,
                                                px(12.0),
                                                theme.semantic.danger.solid,
                                            ))
                                            .child("Disconnect"),
                                    )
                                }),
                        ),
                )
                .into_any_element(),
        )
    }

    fn render_namespace_search_input(
        &self,
        theme: &OrbitTheme,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let control_hover = theme.surface.control_hover;
        let has_query = !self.namespaces.search_query.is_empty();
        let search_text = if has_query {
            self.namespaces.search_query.clone()
        } else {
            "Filter namespaces...".to_string()
        };

        div()
            .id("namespace-search-input")
            .track_focus(&self.namespace_search_focus)
            .flex()
            .items_center()
            .justify_between()
            .gap_2()
            .px_3()
            .py_2()
            .border_b_1()
            .border_color(theme.border.default)
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_2()
                    .flex_1()
                    .min_w(px(0.0))
                    .child(render_icon(Icon::Search, px(13.0), theme.text.muted))
                    .child(
                        apply_text_step(div(), theme.typography.size_xs)
                            .text_color(if has_query {
                                theme.text.primary
                            } else {
                                theme.text.muted
                            })
                            .child(search_text),
                    ),
            )
            .when(has_query, |el| {
                el.child(
                    div()
                        .id("namespace-search-clear-btn")
                        .cursor_pointer()
                        .p_0p5()
                        .rounded(theme.style.radius_control)
                        .hover(move |style| style.bg(control_hover))
                        .on_click(cx.listener(|this, _event, _window, cx| {
                            this.namespaces.search_query.clear();
                            cx.notify();
                        }))
                        .child(render_icon(Icon::X, px(12.0), theme.text.muted)),
                )
            })
            .into_any_element()
    }

    fn render_namespace_all_option(
        &self,
        theme: &OrbitTheme,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let control_hover = theme.surface.control_hover;
        let is_all = self.namespaces.is_all_namespaces();

        div()
            .id("namespace-option-all")
            .flex()
            .items_center()
            .justify_between()
            .gap_2()
            .px_2p5()
            .py_1p5()
            .rounded(theme.style.radius_control)
            .cursor_pointer()
            .when(is_all, |el| el.bg(theme.accent.soft))
            .hover(move |style| style.bg(control_hover))
            .on_click(cx.listener(|this, _event, _window, cx| {
                if let Some(cleared) = this.namespaces.select_all_namespaces()
                    && let Some(cluster_id) = this.namespaces.cluster_id.clone()
                {
                    this.dispatch_command(ClusterBridgeCommand::SaveNamespaceSelection {
                        cluster_id,
                        namespaces: cleared,
                    });
                }
                this.reset_exemplar_pages();
                cx.notify();
            }))
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_2()
                    .child(render_icon(
                        Icon::Layers,
                        px(13.0),
                        if is_all {
                            theme.accent.accent
                        } else {
                            theme.text.secondary
                        },
                    ))
                    .child(
                        apply_text_step(div(), theme.typography.size_sm)
                            .font_weight(FontWeight::MEDIUM)
                            .text_color(theme.text.primary)
                            .child("All Namespaces"),
                    ),
            )
            .when(is_all, |el| {
                el.child(render_icon(Icon::Check, px(13.0), theme.accent.accent))
            })
            .into_any_element()
    }

    fn render_namespace_option_row(
        &self,
        idx: usize,
        ns: &orbit_core::kubernetes::models::NamespaceInfo,
        theme: &OrbitTheme,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let control_hover = theme.surface.control_hover;
        let ns_name = ns.name.clone();
        let is_selected = self
            .namespaces
            .active_selection()
            .iter()
            .any(|s| s == &ns_name);
        let non_active_status = (ns.status != "Active").then(|| ns.status.clone());
        let cluster_id = self.namespaces.cluster_id.clone();

        div()
            .id(("namespace-option-row", idx))
            .flex()
            .items_center()
            .justify_between()
            .gap_2()
            .px_2p5()
            .py_1p5()
            .rounded(theme.style.radius_control)
            .cursor_pointer()
            .when(is_selected, |el| el.bg(theme.accent.soft))
            .hover(move |style| style.bg(control_hover))
            .on_click(cx.listener(move |this, _event, _window, cx| {
                if let Some(updated) = this.namespaces.toggle_namespace(&ns_name)
                    && let Some(cid) = cluster_id.clone()
                {
                    this.dispatch_command(ClusterBridgeCommand::SaveNamespaceSelection {
                        cluster_id: cid,
                        namespaces: updated,
                    });
                }
                this.reset_exemplar_pages();
                cx.notify();
            }))
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_2()
                    .min_w(px(0.0))
                    .child(
                        div()
                            .w(px(14.0))
                            .h(px(14.0))
                            .flex()
                            .items_center()
                            .justify_center()
                            .rounded(theme.style.radius_control)
                            .border_1()
                            .border_color(if is_selected {
                                theme.accent.accent
                            } else {
                                theme.border.strong
                            })
                            .when(is_selected, |el| {
                                el.bg(theme.accent.accent).child(render_icon(
                                    Icon::Check,
                                    px(10.0),
                                    theme.text.on_accent,
                                ))
                            }),
                    )
                    .child(
                        apply_text_step(div(), theme.typography.size_sm)
                            .font_weight(if is_selected {
                                FontWeight::MEDIUM
                            } else {
                                FontWeight::NORMAL
                            })
                            .text_color(theme.text.primary)
                            .child(ns.name.clone()),
                    ),
            )
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_1p5()
                    .when_some(non_active_status, |el, phase| {
                        el.child(
                            apply_text_step(div(), theme.typography.size_xs)
                                .text_color(theme.semantic.warning.solid)
                                .child(phase),
                        )
                    })
                    .when(ns.is_system, |el| {
                        el.child(
                            apply_text_step(div(), theme.typography.size_xs)
                                .px_1p5()
                                .py_0p5()
                                .rounded(theme.style.radius_control)
                                .bg(theme.surface.bg_hover)
                                .text_color(theme.text.muted)
                                .child("system"),
                        )
                    }),
            )
            .into_any_element()
    }

    fn render_namespace_list_body(&self, theme: &OrbitTheme, cx: &mut Context<Self>) -> AnyElement {
        let control_hover = theme.surface.control_hover;
        let filtered = self.namespaces.filtered_namespaces();
        let mut list = div()
            .id("namespace-dropdown-scroll-list")
            .flex()
            .flex_col()
            .gap_0p5()
            .p_1p5()
            .max_h(px(300.0))
            .overflow_y_scroll()
            .child(self.render_namespace_all_option(theme, cx));

        if self.namespaces.status == NamespaceLoadStatus::Loading
            && self.namespaces.namespaces.is_empty()
        {
            return list
                .child(
                    apply_text_step(div(), theme.typography.size_xs)
                        .px_3()
                        .py_3()
                        .text_color(theme.text.muted)
                        .child("Loading namespaces..."),
                )
                .into_any_element();
        }

        if self.namespaces.status == NamespaceLoadStatus::Error {
            let err_msg = self
                .namespaces
                .error_message
                .clone()
                .unwrap_or_else(|| "Failed to load namespaces.".to_string());
            return list
                .child(
                    div()
                        .flex()
                        .items_center()
                        .justify_between()
                        .gap_2()
                        .px_2p5()
                        .py_2()
                        .rounded(theme.style.radius_control)
                        .bg(theme.semantic.danger.soft)
                        .child(
                            apply_text_step(div(), theme.typography.size_xs)
                                .flex_1()
                                .text_color(theme.semantic.danger.solid)
                                .child(err_msg),
                        )
                        .child(
                            theme
                                .style_control(div().id("namespace-retry-btn"))
                                .px_2()
                                .py_1()
                                .cursor_pointer()
                                .hover(move |style| style.bg(control_hover))
                                .on_click(cx.listener(|this, _event, _window, cx| {
                                    if let Some(cmd) = this.namespaces.begin_refresh() {
                                        this.dispatch_command(cmd);
                                    }
                                    cx.notify();
                                }))
                                .child(
                                    apply_text_step(div(), theme.typography.size_xs)
                                        .font_weight(FontWeight::SEMIBOLD)
                                        .text_color(theme.text.primary)
                                        .child("Retry"),
                                ),
                        ),
                )
                .into_any_element();
        }

        if filtered.is_empty() {
            let empty_msg = if self.namespaces.search_query.trim().is_empty() {
                "No namespaces found in this cluster.".to_string()
            } else {
                format!(
                    "No namespaces match \"{}\".",
                    self.namespaces.search_query.trim()
                )
            };
            return list
                .child(
                    apply_text_step(div(), theme.typography.size_xs)
                        .px_3()
                        .py_3()
                        .text_color(theme.text.muted)
                        .child(empty_msg),
                )
                .into_any_element();
        }

        for (idx, ns) in filtered.into_iter().enumerate() {
            list = list.child(self.render_namespace_option_row(idx, ns, theme, cx));
        }

        list.into_any_element()
    }

    fn render_namespace_dropdown_overlay(&self, cx: &mut Context<Self>) -> Option<AnyElement> {
        if !self.namespaces.dropdown_open {
            return None;
        }

        let theme = OrbitTheme::get(cx).clone();
        let dropdown_left = if self.sidebar.is_collapsed {
            ACTIVITY_BAR_WIDTH + 240.0
        } else {
            ACTIVITY_BAR_WIDTH + self.sidebar.panel_width + 240.0
        };

        Some(
            div()
                .absolute()
                .inset_0()
                .child(
                    div()
                        .id("namespace-dropdown-backdrop")
                        .absolute()
                        .inset_0()
                        .on_click(cx.listener(|this, _event, _window, cx| {
                            this.namespaces.close_dropdown();
                            cx.notify();
                        })),
                )
                .child(
                    theme
                        .style_overlay(div().id("namespace-dropdown-menu"))
                        .absolute()
                        .top(px(Self::header_dropdown_top_offset()))
                        .left(px(dropdown_left))
                        .w(px(320.0))
                        .on_click(|_event, _window, cx| {
                            cx.stop_propagation();
                        })
                        .child(self.render_namespace_search_input(&theme, cx))
                        .child(self.render_namespace_list_body(&theme, cx)),
                )
                .into_any_element(),
        )
    }

    fn render_workspace_view_header(&self, theme: &OrbitTheme) -> AnyElement {
        let view = self.sidebar.active_view;
        let resource_kind = should_show_header_kind_badge(view)
            .then(|| view.resource_kind())
            .flatten();

        div()
            .flex()
            .flex_col()
            .gap_1()
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_2()
                    .child(
                        apply_text_step(div(), theme.typography.size_lg)
                            .font_weight(FontWeight::BOLD)
                            .text_color(theme.text.primary)
                            .child(view.label()),
                    )
                    .when_some(resource_kind, |el, kind| {
                        let color = theme.k8s_color(kind);
                        el.child(
                            div()
                                .flex()
                                .items_center()
                                .gap_1p5()
                                .px_2()
                                .py_0p5()
                                .rounded(theme.style.radius_control)
                                .bg(theme.surface.bg_panel)
                                .border_1()
                                .border_color(theme.border.default)
                                .child(render_icon(view.icon(), px(12.0), color))
                                .child(
                                    apply_text_step(div(), theme.typography.size_2xs)
                                        .font_weight(FontWeight::SEMIBOLD)
                                        .text_color(theme.text.secondary)
                                        .child(format!("{kind:?}")),
                                ),
                        )
                    }),
            )
            .child(
                apply_text_step(div(), theme.typography.size_xs)
                    .text_color(theme.text.secondary)
                    .child(view.description()),
            )
            .into_any_element()
    }

    fn render_settings_workspace(&self, theme: &OrbitTheme, cx: &mut Context<Self>) -> AnyElement {
        let sidebar_status = if self.sidebar.is_collapsed {
            "Collapsed".to_string()
        } else {
            format!("Expanded ({:.0}px)", self.sidebar.panel_width)
        };
        let config_file_path = OrbitConfig::file_path().map(|p| p.display().to_string());
        let model = views::settings::SettingsViewModel {
            theme,
            resolved_system_is_dark: self.resolved_system_is_dark,
            enable_experience_features: self.enable_experience_features,
            config_file_path,
            sidebar_status,
            save_error: self.settings_error.as_deref(),
            dropdown: &self.settings_dropdown,
        };

        let mode_entity = cx.entity().clone();
        let style_entity = cx.entity().clone();
        let accent_entity = cx.entity().clone();
        let exp_entity = cx.entity().clone();
        let reset_entity = cx.entity().clone();
        let toggle_dropdown_entity = cx.entity().clone();
        let hover_dropdown_entity = cx.entity().clone();

        let callbacks = views::settings::SettingsCallbacks {
            on_select_theme_mode: std::rc::Rc::new(move |mode, window, cx| {
                mode_entity.update(cx, |shell, cx| shell.select_theme_mode(mode, window, cx));
            }),
            on_select_ui_style: std::rc::Rc::new(move |style, window, cx| {
                style_entity.update(cx, |shell, cx| shell.select_ui_style(style, window, cx));
            }),
            on_select_accent_color: std::rc::Rc::new(move |accent, window, cx| {
                accent_entity.update(cx, |shell, cx| {
                    shell.select_accent_color(accent, window, cx);
                });
            }),
            on_toggle_experience_features: std::rc::Rc::new(move |enabled, _window, cx| {
                exp_entity.update(cx, |shell, cx| {
                    shell.toggle_experience_features(enabled, cx)
                });
            }),
            on_reset_appearance: std::rc::Rc::new(move |window, cx| {
                reset_entity.update(cx, |shell, cx| shell.reset_appearance_defaults(window, cx));
            }),
            on_toggle_dropdown: std::rc::Rc::new(move |kind, window, cx| {
                toggle_dropdown_entity.update(cx, |shell, cx| {
                    shell.state.close_dropdown();
                    shell.namespaces.close_dropdown();
                    shell.dismiss_context_menu();
                    let current_theme = OrbitTheme::get(cx);
                    let current_index = match kind {
                        views::settings::SettingsDropdownKind::ThemeMode => {
                            views::settings::theme_mode_index(current_theme.mode)
                        }
                        views::settings::SettingsDropdownKind::UiStyle => {
                            views::settings::ui_style_index(current_theme.style.ui_style)
                        }
                    };
                    shell.settings_dropdown.toggle(kind, current_index);
                    if shell.settings_dropdown.is_any_open() {
                        shell.focus_handle.focus(window);
                    }
                    cx.notify();
                });
            }),
            on_hover_dropdown_option: std::rc::Rc::new(move |idx, _window, cx| {
                hover_dropdown_entity.update(cx, |shell, cx| {
                    if shell.settings_dropdown.highlighted_index != idx {
                        shell.settings_dropdown.set_highlight(idx);
                        cx.notify();
                    }
                });
            }),
        };

        views::settings::render_settings_view(&model, &callbacks)
    }

    fn render_connecting_workspace(
        cluster_id: &str,
        display_name: &str,
        theme: &OrbitTheme,
    ) -> AnyElement {
        let mono_font = theme.typography.mono_font(FontWeight::NORMAL);
        theme
            .style_card(div())
            .flex_1()
            .flex()
            .items_center()
            .justify_center()
            .bg(theme.surface.bg_panel)
            .child(
                div()
                    .flex()
                    .flex_col()
                    .items_center()
                    .gap_3()
                    .p_6()
                    .child(
                        apply_text_step(div(), theme.typography.size_xs)
                            .px_3()
                            .py_1()
                            .rounded(theme.style.radius_control)
                            .bg(theme.semantic.warning.soft)
                            .font_weight(FontWeight::SEMIBOLD)
                            .text_color(theme.semantic.warning.solid)
                            .child("CONNECTING"),
                    )
                    .child(
                        apply_text_step(div(), theme.typography.size_base)
                            .font_weight(FontWeight::SEMIBOLD)
                            .child(format!("Connecting to {display_name}...")),
                    )
                    .child(
                        apply_text_step(div(), theme.typography.size_xs)
                            .font(mono_font)
                            .text_color(theme.text.muted)
                            .child(format!(
                                "Context: {cluster_id} • Probing Kubernetes API server (10s timeout)"
                            )),
                    ),
            )
            .into_any_element()
    }

    fn render_connected_workspace(
        &mut self,
        cluster_id: &str,
        display_name: &str,
        k8s_version: Option<&str>,
        theme: &OrbitTheme,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        if let Some(WorkspaceTab::Logs(target)) = self.tabs.active_tab() {
            return self.render_logs_tab_content(&target, theme, cx);
        }
        let version_label = k8s_version.unwrap_or("Connected").to_string();

        if self.sidebar.active_view == WorkspaceView::Events {
            let sort_entity = cx.entity().clone();
            let on_sort = move |col_id: SharedString,
                                _event: &gpui::ClickEvent,
                                _window: &mut Window,
                                cx: &mut App| {
                sort_entity.update(cx, |shell, cx| {
                    shell.events.table.toggle_sort(col_id.as_ref());
                    let cfg = shell.events.table.to_persisted_config();
                    shell.persist_table_config("events", cfg);
                    cx.notify();
                });
            };

            let row_entity = cx.entity().clone();
            let on_row_click = move |_row_idx: usize,
                                     row_key: SharedString,
                                     _event: &gpui::ClickEvent,
                                     _window: &mut Window,
                                     cx: &mut App| {
                row_entity.update(cx, |shell, cx| {
                    shell.events.table.selected_row_key = Some(row_key.to_string());
                    if let Some(ev) = shell.events.events.iter().find(|e| {
                        if !e.uid.is_empty() {
                            e.uid == row_key.as_ref()
                        } else {
                            format!("{}/{}/{}", e.namespace, e.reason, e.object_name)
                                == row_key.as_ref()
                        }
                    }) {
                        let target = ResourceDrawerTarget::Event {
                            uid: ev.uid.clone(),
                            name: ev.name.clone(),
                            reason: ev.reason.clone(),
                            namespace: ev.namespace.clone(),
                        };
                        shell.open_resource_drawer(target, ResourceDrawerTabId::Overview, cx);
                    } else {
                        cx.notify();
                    }
                });
            };

            let filter_entity = cx.entity().clone();
            let on_type_filter =
                move |filter: EventTypeFilter, _window: &mut Window, cx: &mut App| {
                    filter_entity.update(cx, |shell, cx| {
                        shell.events.set_type_filter(filter);
                        cx.notify();
                    });
                };

            let ctx_entity = cx.entity().clone();
            let on_row_context_menu = move |row_key: SharedString,
                                            ev: &orbit_core::kubernetes::models::EventInfo,
                                            event: &MouseDownEvent,
                                            _window: &mut Window,
                                            cx: &mut App| {
                let object_name = ev.object_name.clone();
                let message = ev.message.clone();
                let namespace = ev.namespace.clone();
                let drawer_target = ResourceDrawerTarget::Event {
                    uid: ev.uid.clone(),
                    name: ev.name.clone(),
                    reason: ev.reason.clone(),
                    namespace: ev.namespace.clone(),
                };
                let anchor = event.position;
                let copy_obj = move |_w: &mut Window, cx: &mut App| {
                    cx.write_to_clipboard(ClipboardItem::new_string(object_name.clone()));
                };
                let copy_msg = move |_w: &mut Window, cx: &mut App| {
                    cx.write_to_clipboard(ClipboardItem::new_string(message.clone()));
                };
                let filter_entity = ctx_entity.clone();
                let filter_ns = move |_w: &mut Window, cx: &mut App| {
                    filter_entity.update(cx, |shell, cx| {
                        shell.filter_by_namespace(&namespace, cx);
                    });
                };
                let refresh_ctx_entity = ctx_entity.clone();
                let refresh_cb = move |_w: &mut Window, cx: &mut App| {
                    refresh_ctx_entity.update(cx, |shell, cx| {
                        shell.refresh_workspace(cx);
                    });
                };
                let details_entity = ctx_entity.clone();
                let details_target = drawer_target.clone();
                let view_details = move |_w: &mut Window, cx: &mut App| {
                    details_entity.update(cx, |shell, cx| {
                        shell.open_resource_drawer(
                            details_target.clone(),
                            ResourceDrawerTabId::Overview,
                            cx,
                        );
                    });
                };
                let yaml_entity = ctx_entity.clone();
                let view_yaml = move |_w: &mut Window, cx: &mut App| {
                    yaml_entity.update(cx, |shell, cx| {
                        shell.open_resource_drawer(
                            drawer_target.clone(),
                            ResourceDrawerTabId::Yaml,
                            cx,
                        );
                    });
                };
                let delete_action = {
                    let e = ctx_entity.clone();
                    let ns = ev.namespace.clone();
                    let name = ev.name.clone();
                    move |window: &mut Window, cx: &mut App| {
                        e.update(cx, |shell, cx| {
                            shell.open_delete_confirm_modal("Event", Some(&ns), &name, window, cx);
                        });
                    }
                };
                let entries = views::events::build_event_context_menu_entries(
                    ev,
                    copy_obj,
                    copy_msg,
                    filter_ns,
                    refresh_cb,
                    view_details,
                    view_yaml,
                    delete_action,
                );
                ctx_entity.update(cx, |shell, cx| {
                    shell.events.table.selected_row_key = Some(row_key.to_string());
                    shell.open_context_menu(anchor, entries, cx);
                });
            };

            let refresh_entity = cx.entity().clone();
            let on_refresh = move |_window: &mut Window, cx: &mut App| {
                refresh_entity.update(cx, |shell, cx| {
                    shell.refresh_workspace(cx);
                });
            };

            return views::events::render_events_view(
                &self.events,
                &self.namespaces,
                theme,
                on_sort,
                on_row_click,
                on_row_context_menu,
                on_type_filter,
                on_refresh,
                Some(self.table_enhancement_callbacks(WorkspaceView::Events, cx)),
            );
        }

        if self.sidebar.active_view == WorkspaceView::Deployments {
            let sort_entity = cx.entity().clone();
            let on_sort = move |col_id: SharedString,
                                _event: &gpui::ClickEvent,
                                _window: &mut Window,
                                cx: &mut App| {
                sort_entity.update(cx, |shell, cx| {
                    shell.deployments.table.toggle_sort(col_id.as_ref());
                    let cfg = shell.deployments.table.to_persisted_config();
                    shell.persist_table_config("deployments", cfg);
                    cx.notify();
                });
            };

            let row_entity = cx.entity().clone();
            let on_row_click = move |_row_idx: usize,
                                     row_key: SharedString,
                                     _event: &gpui::ClickEvent,
                                     _window: &mut Window,
                                     cx: &mut App| {
                row_entity.update(cx, |shell, cx| {
                    shell.deployments.table.selected_row_key = Some(row_key.to_string());
                    if let Some((ns, name)) = row_key.as_ref().split_once('/') {
                        let target = ResourceDrawerTarget::Deployment {
                            name: name.to_string(),
                            namespace: ns.to_string(),
                        };
                        shell.open_resource_drawer(target, ResourceDrawerTabId::Overview, cx);
                    } else {
                        cx.notify();
                    }
                });
            };

            let ctx_entity = cx.entity().clone();
            let on_row_context_menu =
                move |row_key: SharedString,
                      deployment: &orbit_core::kubernetes::models::DeploymentInfo,
                      event: &MouseDownEvent,
                      _window: &mut Window,
                      cx: &mut App| {
                    let name = deployment.name.clone();
                    let namespace = deployment.namespace.clone();
                    let drawer_target = ResourceDrawerTarget::Deployment {
                        name: deployment.name.clone(),
                        namespace: deployment.namespace.clone(),
                    };
                    let anchor = event.position;
                    let copy_name = move |_w: &mut Window, cx: &mut App| {
                        cx.write_to_clipboard(ClipboardItem::new_string(name.clone()));
                    };
                    let filter_entity = ctx_entity.clone();
                    let filter_ns = move |_w: &mut Window, cx: &mut App| {
                        filter_entity.update(cx, |shell, cx| {
                            shell.filter_by_namespace(&namespace, cx);
                        });
                    };
                    let refresh_ctx_entity = ctx_entity.clone();
                    let refresh_cb = move |_w: &mut Window, cx: &mut App| {
                        refresh_ctx_entity.update(cx, |shell, cx| {
                            shell.refresh_workspace(cx);
                        });
                    };
                    let details_entity = ctx_entity.clone();
                    let details_target = drawer_target.clone();
                    let view_details = move |_w: &mut Window, cx: &mut App| {
                        details_entity.update(cx, |shell, cx| {
                            shell.open_resource_drawer(
                                details_target.clone(),
                                ResourceDrawerTabId::Overview,
                                cx,
                            );
                        });
                    };
                    let yaml_entity = ctx_entity.clone();
                    let view_yaml = move |_w: &mut Window, cx: &mut App| {
                        yaml_entity.update(cx, |shell, cx| {
                            shell.open_resource_drawer(
                                drawer_target.clone(),
                                ResourceDrawerTabId::Yaml,
                                cx,
                            );
                        });
                    };
                    let scale_action = {
                        let e = ctx_entity.clone();
                        let ns = deployment.namespace.clone();
                        let name = deployment.name.clone();
                        let replicas = deployment.replicas.desired;
                        std::rc::Rc::new(move |window: &mut Window, cx: &mut App| {
                            e.update(cx, |shell, cx| {
                                shell.open_scale_modal(
                                    "Deployment",
                                    &ns,
                                    &name,
                                    replicas,
                                    window,
                                    cx,
                                );
                            });
                        })
                    };
                    let restart_action = {
                        let e = ctx_entity.clone();
                        let ns = deployment.namespace.clone();
                        let name = deployment.name.clone();
                        std::rc::Rc::new(move |window: &mut Window, cx: &mut App| {
                            e.update(cx, |shell, cx| {
                                shell.open_restart_confirm_modal(
                                    WorkloadActionKind::Restart,
                                    "Deployment",
                                    &ns,
                                    &name,
                                    window,
                                    cx,
                                );
                            });
                        })
                    };
                    let delete_action = {
                        let e = ctx_entity.clone();
                        let ns = deployment.namespace.clone();
                        let name = deployment.name.clone();
                        move |window: &mut Window, cx: &mut App| {
                            e.update(cx, |shell, cx| {
                                shell.open_delete_confirm_modal(
                                    "Deployment",
                                    Some(&ns),
                                    &name,
                                    window,
                                    cx,
                                );
                            });
                        }
                    };
                    let view_logs = {
                        let entity = ctx_entity.clone();
                        let name = deployment.name.clone();
                        let ns = deployment.namespace.clone();
                        move |_window: &mut Window, cx: &mut App| {
                            entity.update(cx, |shell, cx| {
                                shell.open_logs_tab("Deployment", &ns, &name, cx);
                            });
                        }
                    };
                    let dep_ns = deployment.namespace.clone();
                    let dep_name = deployment.name.clone();
                    let dep_ports = extract_ports_from_deployment(deployment);
                    let pf_entity = ctx_entity.clone();
                    let port_forward_action = move |_w: &mut Window, cx: &mut App| {
                        let ns = dep_ns.clone();
                        let name = dep_name.clone();
                        let ports = dep_ports.clone();
                        pf_entity.update(cx, |shell, cx| {
                            shell.open_port_forward_target_modal(
                                &ns,
                                "Deployment",
                                &name,
                                ports,
                                cx,
                            );
                        });
                    };
                    let entries = views::deployments::build_deployment_context_menu_entries(
                        deployment,
                        copy_name,
                        filter_ns,
                        refresh_cb,
                        view_details,
                        view_yaml,
                        view_logs,
                        port_forward_action,
                        move |w, cx| scale_action(w, cx),
                        move |w, cx| restart_action(w, cx),
                        delete_action,
                    );
                    ctx_entity.update(cx, |shell, cx| {
                        shell.deployments.table.selected_row_key = Some(row_key.to_string());
                        shell.open_context_menu(anchor, entries, cx);
                    });
                };

            let refresh_entity = cx.entity().clone();
            let on_refresh = move |_window: &mut Window, cx: &mut App| {
                refresh_entity.update(cx, |shell, cx| {
                    shell.refresh_workspace(cx);
                });
            };

            let yaml_entity = cx.entity().clone();
            let on_row_quick_yaml = move |_row_idx: usize,
                                          row_key: SharedString,
                                          _event: &gpui::ClickEvent,
                                          _window: &mut Window,
                                          cx: &mut App| {
                yaml_entity.update(cx, |shell, cx| {
                    shell.deployments.table.selected_row_key = Some(row_key.to_string());
                    if let Some((ns, name)) = row_key.as_ref().split_once('/') {
                        let target = ResourceDrawerTarget::Deployment {
                            name: name.to_string(),
                            namespace: ns.to_string(),
                        };
                        shell.open_resource_drawer(target, ResourceDrawerTabId::Yaml, cx);
                    } else {
                        cx.notify();
                    }
                });
            };

            return views::deployments::render_deployments_view(
                &self.deployments,
                &self.namespaces,
                theme,
                on_sort,
                on_row_click,
                on_row_quick_yaml,
                on_row_context_menu,
                on_refresh,
                Some(self.table_enhancement_callbacks(WorkspaceView::Deployments, cx)),
            );
        }

        if self.sidebar.active_view == WorkspaceView::StatefulSets {
            let sort_entity = cx.entity().clone();
            let on_sort = move |col_id: SharedString,
                                _event: &gpui::ClickEvent,
                                _window: &mut Window,
                                cx: &mut App| {
                sort_entity.update(cx, |shell, cx| {
                    shell.statefulsets.table.toggle_sort(col_id.as_ref());
                    cx.notify();
                });
            };

            let row_entity = cx.entity().clone();
            let on_row_click = move |_row_idx: usize,
                                     row_key: SharedString,
                                     _event: &gpui::ClickEvent,
                                     _window: &mut Window,
                                     cx: &mut App| {
                row_entity.update(cx, |shell, cx| {
                    shell.statefulsets.table.selected_row_key = Some(row_key.to_string());
                    if let Some((ns, name)) = row_key.as_ref().split_once('/') {
                        let target = ResourceDrawerTarget::StatefulSet {
                            name: name.to_string(),
                            namespace: ns.to_string(),
                        };
                        shell.open_resource_drawer(target, ResourceDrawerTabId::Overview, cx);
                    } else {
                        cx.notify();
                    }
                });
            };

            let ctx_entity = cx.entity().clone();
            let on_row_context_menu =
                move |row_key: SharedString,
                      statefulset: &orbit_core::kubernetes::models::StatefulSetInfo,
                      event: &MouseDownEvent,
                      _window: &mut Window,
                      cx: &mut App| {
                    let name = statefulset.name.clone();
                    let namespace = statefulset.namespace.clone();
                    let drawer_target = ResourceDrawerTarget::StatefulSet {
                        name: statefulset.name.clone(),
                        namespace: statefulset.namespace.clone(),
                    };
                    let anchor = event.position;
                    let copy_name = move |_w: &mut Window, cx: &mut App| {
                        cx.write_to_clipboard(ClipboardItem::new_string(name.clone()));
                    };
                    let filter_entity = ctx_entity.clone();
                    let filter_ns = move |_w: &mut Window, cx: &mut App| {
                        filter_entity.update(cx, |shell, cx| {
                            shell.filter_by_namespace(&namespace, cx);
                        });
                    };
                    let refresh_ctx_entity = ctx_entity.clone();
                    let refresh_cb = move |_w: &mut Window, cx: &mut App| {
                        refresh_ctx_entity.update(cx, |shell, cx| {
                            shell.refresh_workspace(cx);
                        });
                    };
                    let details_entity = ctx_entity.clone();
                    let details_target = drawer_target.clone();
                    let view_details = move |_w: &mut Window, cx: &mut App| {
                        details_entity.update(cx, |shell, cx| {
                            shell.open_resource_drawer(
                                details_target.clone(),
                                ResourceDrawerTabId::Overview,
                                cx,
                            );
                        });
                    };
                    let yaml_entity = ctx_entity.clone();
                    let view_yaml = move |_w: &mut Window, cx: &mut App| {
                        yaml_entity.update(cx, |shell, cx| {
                            shell.open_resource_drawer(
                                drawer_target.clone(),
                                ResourceDrawerTabId::Yaml,
                                cx,
                            );
                        });
                    };
                    let scale_action = {
                        let e = ctx_entity.clone();
                        let ns = statefulset.namespace.clone();
                        let name = statefulset.name.clone();
                        let replicas = statefulset.replicas.desired;
                        std::rc::Rc::new(move |window: &mut Window, cx: &mut App| {
                            e.update(cx, |shell, cx| {
                                shell.open_scale_modal(
                                    "StatefulSet",
                                    &ns,
                                    &name,
                                    replicas,
                                    window,
                                    cx,
                                );
                            });
                        })
                    };
                    let restart_action = {
                        let e = ctx_entity.clone();
                        let ns = statefulset.namespace.clone();
                        let name = statefulset.name.clone();
                        std::rc::Rc::new(move |window: &mut Window, cx: &mut App| {
                            e.update(cx, |shell, cx| {
                                shell.open_restart_confirm_modal(
                                    WorkloadActionKind::Restart,
                                    "StatefulSet",
                                    &ns,
                                    &name,
                                    window,
                                    cx,
                                );
                            });
                        })
                    };
                    let delete_action = {
                        let e = ctx_entity.clone();
                        let ns = statefulset.namespace.clone();
                        let name = statefulset.name.clone();
                        move |window: &mut Window, cx: &mut App| {
                            e.update(cx, |shell, cx| {
                                shell.open_delete_confirm_modal(
                                    "StatefulSet",
                                    Some(&ns),
                                    &name,
                                    window,
                                    cx,
                                );
                            });
                        }
                    };
                    let view_logs = {
                        let entity = ctx_entity.clone();
                        let name = statefulset.name.clone();
                        let ns = statefulset.namespace.clone();
                        move |_window: &mut Window, cx: &mut App| {
                            entity.update(cx, |shell, cx| {
                                shell.open_logs_tab("StatefulSet", &ns, &name, cx);
                            });
                        }
                    };
                    let ss_ns = statefulset.namespace.clone();
                    let ss_name = statefulset.name.clone();
                    let ss_ports = extract_ports_from_statefulset(statefulset);
                    let pf_entity = ctx_entity.clone();
                    let port_forward_action = move |_w: &mut Window, cx: &mut App| {
                        let ns = ss_ns.clone();
                        let name = ss_name.clone();
                        let ports = ss_ports.clone();
                        pf_entity.update(cx, |shell, cx| {
                            shell.open_port_forward_target_modal(
                                &ns,
                                "StatefulSet",
                                &name,
                                ports,
                                cx,
                            );
                        });
                    };
                    let entries = views::statefulsets::build_statefulset_context_menu_entries(
                        statefulset,
                        copy_name,
                        filter_ns,
                        refresh_cb,
                        view_details,
                        view_yaml,
                        view_logs,
                        port_forward_action,
                        move |w, cx| scale_action(w, cx),
                        move |w, cx| restart_action(w, cx),
                        delete_action,
                    );
                    ctx_entity.update(cx, |shell, cx| {
                        shell.statefulsets.table.selected_row_key = Some(row_key.to_string());
                        shell.open_context_menu(anchor, entries, cx);
                    });
                };

            let refresh_entity = cx.entity().clone();
            let on_refresh = move |_window: &mut Window, cx: &mut App| {
                refresh_entity.update(cx, |shell, cx| {
                    shell.refresh_workspace(cx);
                });
            };

            let yaml_entity = cx.entity().clone();
            let on_row_quick_yaml = move |_row_idx: usize,
                                          row_key: SharedString,
                                          _event: &gpui::ClickEvent,
                                          _window: &mut Window,
                                          cx: &mut App| {
                yaml_entity.update(cx, |shell, cx| {
                    shell.statefulsets.table.selected_row_key = Some(row_key.to_string());
                    if let Some((ns, name)) = row_key.as_ref().split_once('/') {
                        let target = ResourceDrawerTarget::StatefulSet {
                            name: name.to_string(),
                            namespace: ns.to_string(),
                        };
                        shell.open_resource_drawer(target, ResourceDrawerTabId::Yaml, cx);
                    } else {
                        cx.notify();
                    }
                });
            };

            return views::statefulsets::render_statefulsets_view(
                &self.statefulsets,
                &self.namespaces,
                theme,
                on_sort,
                on_row_click,
                on_row_quick_yaml,
                on_row_context_menu,
                on_refresh,
            );
        }

        if self.sidebar.active_view == WorkspaceView::ReplicaSets {
            let sort_entity = cx.entity().clone();
            let on_sort = move |col_id: SharedString,
                                _event: &gpui::ClickEvent,
                                _window: &mut Window,
                                cx: &mut App| {
                sort_entity.update(cx, |shell, cx| {
                    shell.replicasets.table.toggle_sort(col_id.as_ref());
                    cx.notify();
                });
            };

            let row_entity = cx.entity().clone();
            let on_row_click = move |_row_idx: usize,
                                     row_key: SharedString,
                                     _event: &gpui::ClickEvent,
                                     _window: &mut Window,
                                     cx: &mut App| {
                row_entity.update(cx, |shell, cx| {
                    shell.replicasets.table.selected_row_key = Some(row_key.to_string());
                    if let Some((ns, name)) = row_key.as_ref().split_once('/') {
                        let target = ResourceDrawerTarget::ReplicaSet {
                            name: name.to_string(),
                            namespace: ns.to_string(),
                        };
                        shell.open_resource_drawer(target, ResourceDrawerTabId::Overview, cx);
                    } else {
                        cx.notify();
                    }
                });
            };

            let ctx_entity = cx.entity().clone();
            let on_row_context_menu =
                move |row_key: SharedString,
                      replicaset: &orbit_core::kubernetes::models::ReplicaSetInfo,
                      event: &MouseDownEvent,
                      _window: &mut Window,
                      cx: &mut App| {
                    let name = replicaset.name.clone();
                    let namespace = replicaset.namespace.clone();
                    let drawer_target = ResourceDrawerTarget::ReplicaSet {
                        name: replicaset.name.clone(),
                        namespace: replicaset.namespace.clone(),
                    };
                    let anchor = event.position;
                    let copy_name = move |_w: &mut Window, cx: &mut App| {
                        cx.write_to_clipboard(ClipboardItem::new_string(name.clone()));
                    };
                    let filter_entity = ctx_entity.clone();
                    let filter_ns = move |_w: &mut Window, cx: &mut App| {
                        filter_entity.update(cx, |shell, cx| {
                            shell.filter_by_namespace(&namespace, cx);
                        });
                    };
                    let refresh_ctx_entity = ctx_entity.clone();
                    let refresh_cb = move |_w: &mut Window, cx: &mut App| {
                        refresh_ctx_entity.update(cx, |shell, cx| {
                            shell.refresh_workspace(cx);
                        });
                    };
                    let details_entity = ctx_entity.clone();
                    let details_target = drawer_target.clone();
                    let view_details = move |_w: &mut Window, cx: &mut App| {
                        details_entity.update(cx, |shell, cx| {
                            shell.open_resource_drawer(
                                details_target.clone(),
                                ResourceDrawerTabId::Overview,
                                cx,
                            );
                        });
                    };
                    let yaml_entity = ctx_entity.clone();
                    let view_yaml = move |_w: &mut Window, cx: &mut App| {
                        yaml_entity.update(cx, |shell, cx| {
                            shell.open_resource_drawer(
                                drawer_target.clone(),
                                ResourceDrawerTabId::Yaml,
                                cx,
                            );
                        });
                    };
                    let scale_action = {
                        let e = ctx_entity.clone();
                        let ns = replicaset.namespace.clone();
                        let name = replicaset.name.clone();
                        let replicas = replicaset.replicas.desired;
                        std::rc::Rc::new(move |window: &mut Window, cx: &mut App| {
                            e.update(cx, |shell, cx| {
                                shell.open_scale_modal(
                                    "ReplicaSet",
                                    &ns,
                                    &name,
                                    replicas,
                                    window,
                                    cx,
                                );
                            });
                        })
                    };
                    let delete_action = {
                        let e = ctx_entity.clone();
                        let ns = replicaset.namespace.clone();
                        let name = replicaset.name.clone();
                        move |window: &mut Window, cx: &mut App| {
                            e.update(cx, |shell, cx| {
                                shell.open_delete_confirm_modal(
                                    "ReplicaSet",
                                    Some(&ns),
                                    &name,
                                    window,
                                    cx,
                                );
                            });
                        }
                    };
                    let view_logs = {
                        let entity = ctx_entity.clone();
                        let name = replicaset.name.clone();
                        let ns = replicaset.namespace.clone();
                        move |_window: &mut Window, cx: &mut App| {
                            entity.update(cx, |shell, cx| {
                                shell.open_logs_tab("ReplicaSet", &ns, &name, cx);
                            });
                        }
                    };
                    let rs_ns = replicaset.namespace.clone();
                    let rs_name = replicaset.name.clone();
                    let rs_ports = extract_ports_from_replicaset(replicaset);
                    let pf_entity = ctx_entity.clone();
                    let port_forward_action = move |_w: &mut Window, cx: &mut App| {
                        let ns = rs_ns.clone();
                        let name = rs_name.clone();
                        let ports = rs_ports.clone();
                        pf_entity.update(cx, |shell, cx| {
                            shell.open_port_forward_target_modal(
                                &ns,
                                "ReplicaSet",
                                &name,
                                ports,
                                cx,
                            );
                        });
                    };
                    let entries = views::replicasets::build_replicaset_context_menu_entries(
                        replicaset,
                        copy_name,
                        filter_ns,
                        refresh_cb,
                        view_details,
                        view_yaml,
                        view_logs,
                        port_forward_action,
                        move |w, cx| scale_action(w, cx),
                        delete_action,
                    );
                    ctx_entity.update(cx, |shell, cx| {
                        shell.replicasets.table.selected_row_key = Some(row_key.to_string());
                        shell.open_context_menu(anchor, entries, cx);
                    });
                };

            let refresh_entity = cx.entity().clone();
            let on_refresh = move |_window: &mut Window, cx: &mut App| {
                refresh_entity.update(cx, |shell, cx| {
                    shell.refresh_workspace(cx);
                });
            };

            let yaml_entity = cx.entity().clone();
            let on_row_quick_yaml = move |_row_idx: usize,
                                          row_key: SharedString,
                                          _event: &gpui::ClickEvent,
                                          _window: &mut Window,
                                          cx: &mut App| {
                yaml_entity.update(cx, |shell, cx| {
                    shell.replicasets.table.selected_row_key = Some(row_key.to_string());
                    if let Some((ns, name)) = row_key.as_ref().split_once('/') {
                        let target = ResourceDrawerTarget::ReplicaSet {
                            name: name.to_string(),
                            namespace: ns.to_string(),
                        };
                        shell.open_resource_drawer(target, ResourceDrawerTabId::Yaml, cx);
                    } else {
                        cx.notify();
                    }
                });
            };

            return views::replicasets::render_replicasets_view(
                &self.replicasets,
                &self.namespaces,
                theme,
                on_sort,
                on_row_click,
                on_row_quick_yaml,
                on_row_context_menu,
                on_refresh,
            );
        }

        if self.sidebar.active_view == WorkspaceView::DaemonSets {
            let sort_entity = cx.entity().clone();
            let on_sort = move |col_id: SharedString,
                                _event: &gpui::ClickEvent,
                                _window: &mut Window,
                                cx: &mut App| {
                sort_entity.update(cx, |shell, cx| {
                    shell.daemonsets.table.toggle_sort(col_id.as_ref());
                    cx.notify();
                });
            };

            let row_entity = cx.entity().clone();
            let on_row_click = move |_row_idx: usize,
                                     row_key: SharedString,
                                     _event: &gpui::ClickEvent,
                                     _window: &mut Window,
                                     cx: &mut App| {
                row_entity.update(cx, |shell, cx| {
                    shell.daemonsets.table.selected_row_key = Some(row_key.to_string());
                    if let Some((ns, name)) = row_key.as_ref().split_once('/') {
                        let target = ResourceDrawerTarget::DaemonSet {
                            name: name.to_string(),
                            namespace: ns.to_string(),
                        };
                        shell.open_resource_drawer(target, ResourceDrawerTabId::Overview, cx);
                    } else {
                        cx.notify();
                    }
                });
            };

            let ctx_entity = cx.entity().clone();
            let on_row_context_menu =
                move |row_key: SharedString,
                      daemonset: &orbit_core::kubernetes::models::DaemonSetInfo,
                      event: &MouseDownEvent,
                      _window: &mut Window,
                      cx: &mut App| {
                    let name = daemonset.name.clone();
                    let namespace = daemonset.namespace.clone();
                    let drawer_target = ResourceDrawerTarget::DaemonSet {
                        name: daemonset.name.clone(),
                        namespace: daemonset.namespace.clone(),
                    };
                    let anchor = event.position;
                    let copy_name = move |_w: &mut Window, cx: &mut App| {
                        cx.write_to_clipboard(ClipboardItem::new_string(name.clone()));
                    };
                    let filter_entity = ctx_entity.clone();
                    let filter_ns = move |_w: &mut Window, cx: &mut App| {
                        filter_entity.update(cx, |shell, cx| {
                            shell.filter_by_namespace(&namespace, cx);
                        });
                    };
                    let refresh_ctx_entity = ctx_entity.clone();
                    let refresh_cb = move |_w: &mut Window, cx: &mut App| {
                        refresh_ctx_entity.update(cx, |shell, cx| {
                            shell.refresh_workspace(cx);
                        });
                    };
                    let details_entity = ctx_entity.clone();
                    let details_target = drawer_target.clone();
                    let view_details = move |_w: &mut Window, cx: &mut App| {
                        details_entity.update(cx, |shell, cx| {
                            shell.open_resource_drawer(
                                details_target.clone(),
                                ResourceDrawerTabId::Overview,
                                cx,
                            );
                        });
                    };
                    let yaml_entity = ctx_entity.clone();
                    let view_yaml = move |_w: &mut Window, cx: &mut App| {
                        yaml_entity.update(cx, |shell, cx| {
                            shell.open_resource_drawer(
                                drawer_target.clone(),
                                ResourceDrawerTabId::Yaml,
                                cx,
                            );
                        });
                    };
                    let restart_action = {
                        let e = ctx_entity.clone();
                        let ns = daemonset.namespace.clone();
                        let name = daemonset.name.clone();
                        std::rc::Rc::new(move |window: &mut Window, cx: &mut App| {
                            e.update(cx, |shell, cx| {
                                shell.open_restart_confirm_modal(
                                    WorkloadActionKind::Restart,
                                    "DaemonSet",
                                    &ns,
                                    &name,
                                    window,
                                    cx,
                                );
                            });
                        })
                    };
                    let delete_action = {
                        let e = ctx_entity.clone();
                        let ns = daemonset.namespace.clone();
                        let name = daemonset.name.clone();
                        move |window: &mut Window, cx: &mut App| {
                            e.update(cx, |shell, cx| {
                                shell.open_delete_confirm_modal(
                                    "DaemonSet",
                                    Some(&ns),
                                    &name,
                                    window,
                                    cx,
                                );
                            });
                        }
                    };
                    let view_logs = {
                        let entity = ctx_entity.clone();
                        let name = daemonset.name.clone();
                        let ns = daemonset.namespace.clone();
                        move |_window: &mut Window, cx: &mut App| {
                            entity.update(cx, |shell, cx| {
                                shell.open_logs_tab("DaemonSet", &ns, &name, cx);
                            });
                        }
                    };
                    let entries = views::daemonsets::build_daemonset_context_menu_entries(
                        daemonset,
                        copy_name,
                        filter_ns,
                        refresh_cb,
                        view_details,
                        view_yaml,
                        view_logs,
                        move |w, cx| restart_action(w, cx),
                        delete_action,
                    );
                    ctx_entity.update(cx, |shell, cx| {
                        shell.daemonsets.table.selected_row_key = Some(row_key.to_string());
                        shell.open_context_menu(anchor, entries, cx);
                    });
                };

            let refresh_entity = cx.entity().clone();
            let on_refresh = move |_window: &mut Window, cx: &mut App| {
                refresh_entity.update(cx, |shell, cx| {
                    shell.refresh_workspace(cx);
                });
            };

            let yaml_entity = cx.entity().clone();
            let on_row_quick_yaml = move |_row_idx: usize,
                                          row_key: SharedString,
                                          _event: &gpui::ClickEvent,
                                          _window: &mut Window,
                                          cx: &mut App| {
                yaml_entity.update(cx, |shell, cx| {
                    shell.daemonsets.table.selected_row_key = Some(row_key.to_string());
                    if let Some((ns, name)) = row_key.as_ref().split_once('/') {
                        let target = ResourceDrawerTarget::DaemonSet {
                            name: name.to_string(),
                            namespace: ns.to_string(),
                        };
                        shell.open_resource_drawer(target, ResourceDrawerTabId::Yaml, cx);
                    } else {
                        cx.notify();
                    }
                });
            };

            return views::daemonsets::render_daemonsets_view(
                &self.daemonsets,
                &self.namespaces,
                theme,
                on_sort,
                on_row_click,
                on_row_quick_yaml,
                on_row_context_menu,
                on_refresh,
            );
        }

        if self.sidebar.active_view == WorkspaceView::Jobs {
            let sort_entity = cx.entity().clone();
            let on_sort = move |col_id: SharedString,
                                _event: &gpui::ClickEvent,
                                _window: &mut Window,
                                cx: &mut App| {
                sort_entity.update(cx, |shell, cx| {
                    shell.jobs.table.toggle_sort(col_id.as_ref());
                    cx.notify();
                });
            };

            let row_entity = cx.entity().clone();
            let on_row_click = move |_row_idx: usize,
                                     row_key: SharedString,
                                     _event: &gpui::ClickEvent,
                                     _window: &mut Window,
                                     cx: &mut App| {
                row_entity.update(cx, |shell, cx| {
                    shell.jobs.table.selected_row_key = Some(row_key.to_string());
                    if let Some((ns, name)) = row_key.as_ref().split_once('/') {
                        let target = ResourceDrawerTarget::Job {
                            name: name.to_string(),
                            namespace: ns.to_string(),
                        };
                        shell.open_resource_drawer(target, ResourceDrawerTabId::Overview, cx);
                    } else {
                        cx.notify();
                    }
                });
            };

            let ctx_entity = cx.entity().clone();
            let on_row_context_menu = move |row_key: SharedString,
                                            job: &orbit_core::kubernetes::models::JobInfo,
                                            event: &MouseDownEvent,
                                            _window: &mut Window,
                                            cx: &mut App| {
                let name = job.name.clone();
                let namespace = job.namespace.clone();
                let drawer_target = ResourceDrawerTarget::Job {
                    name: job.name.clone(),
                    namespace: job.namespace.clone(),
                };
                let anchor = event.position;
                let copy_name = move |_w: &mut Window, cx: &mut App| {
                    cx.write_to_clipboard(ClipboardItem::new_string(name.clone()));
                };
                let filter_entity = ctx_entity.clone();
                let filter_ns = move |_w: &mut Window, cx: &mut App| {
                    filter_entity.update(cx, |shell, cx| {
                        shell.filter_by_namespace(&namespace, cx);
                    });
                };
                let refresh_ctx_entity = ctx_entity.clone();
                let refresh_cb = move |_w: &mut Window, cx: &mut App| {
                    refresh_ctx_entity.update(cx, |shell, cx| {
                        shell.refresh_workspace(cx);
                    });
                };
                let details_entity = ctx_entity.clone();
                let details_target = drawer_target.clone();
                let view_details = move |_w: &mut Window, cx: &mut App| {
                    details_entity.update(cx, |shell, cx| {
                        shell.open_resource_drawer(
                            details_target.clone(),
                            ResourceDrawerTabId::Overview,
                            cx,
                        );
                    });
                };
                let yaml_entity = ctx_entity.clone();
                let view_yaml = move |_w: &mut Window, cx: &mut App| {
                    yaml_entity.update(cx, |shell, cx| {
                        shell.open_resource_drawer(
                            drawer_target.clone(),
                            ResourceDrawerTabId::Yaml,
                            cx,
                        );
                    });
                };
                let delete_action = {
                    let e = ctx_entity.clone();
                    let ns = job.namespace.clone();
                    let name = job.name.clone();
                    move |window: &mut Window, cx: &mut App| {
                        e.update(cx, |shell, cx| {
                            shell.open_delete_confirm_modal("Job", Some(&ns), &name, window, cx);
                        });
                    }
                };
                let view_logs = {
                    let entity = ctx_entity.clone();
                    let name = job.name.clone();
                    let ns = job.namespace.clone();
                    move |_window: &mut Window, cx: &mut App| {
                        entity.update(cx, |shell, cx| {
                            shell.open_logs_tab("Job", &ns, &name, cx);
                        });
                    }
                };
                let entries = views::jobs::build_job_context_menu_entries(
                    job,
                    copy_name,
                    filter_ns,
                    refresh_cb,
                    view_details,
                    view_yaml,
                    view_logs,
                    delete_action,
                );
                ctx_entity.update(cx, |shell, cx| {
                    shell.jobs.table.selected_row_key = Some(row_key.to_string());
                    shell.open_context_menu(anchor, entries, cx);
                });
            };

            let refresh_entity = cx.entity().clone();
            let on_refresh = move |_window: &mut Window, cx: &mut App| {
                refresh_entity.update(cx, |shell, cx| {
                    shell.refresh_workspace(cx);
                });
            };

            let yaml_entity = cx.entity().clone();
            let on_row_quick_yaml = move |_row_idx: usize,
                                          row_key: SharedString,
                                          _event: &gpui::ClickEvent,
                                          _window: &mut Window,
                                          cx: &mut App| {
                yaml_entity.update(cx, |shell, cx| {
                    shell.jobs.table.selected_row_key = Some(row_key.to_string());
                    if let Some((ns, name)) = row_key.as_ref().split_once('/') {
                        let target = ResourceDrawerTarget::Job {
                            name: name.to_string(),
                            namespace: ns.to_string(),
                        };
                        shell.open_resource_drawer(target, ResourceDrawerTabId::Yaml, cx);
                    } else {
                        cx.notify();
                    }
                });
            };

            return views::jobs::render_jobs_view(
                &self.jobs,
                &self.namespaces,
                theme,
                on_sort,
                on_row_click,
                on_row_quick_yaml,
                on_row_context_menu,
                on_refresh,
            );
        }

        if self.sidebar.active_view == WorkspaceView::CronJobs {
            let sort_entity = cx.entity().clone();
            let on_sort = move |col_id: SharedString,
                                _event: &gpui::ClickEvent,
                                _window: &mut Window,
                                cx: &mut App| {
                sort_entity.update(cx, |shell, cx| {
                    shell.cronjobs.table.toggle_sort(col_id.as_ref());
                    cx.notify();
                });
            };

            let row_entity = cx.entity().clone();
            let on_row_click = move |_row_idx: usize,
                                     row_key: SharedString,
                                     _event: &gpui::ClickEvent,
                                     _window: &mut Window,
                                     cx: &mut App| {
                row_entity.update(cx, |shell, cx| {
                    shell.cronjobs.table.selected_row_key = Some(row_key.to_string());
                    if let Some((ns, name)) = row_key.as_ref().split_once('/') {
                        let target = ResourceDrawerTarget::CronJob {
                            name: name.to_string(),
                            namespace: ns.to_string(),
                        };
                        shell.open_resource_drawer(target, ResourceDrawerTabId::Overview, cx);
                    } else {
                        cx.notify();
                    }
                });
            };

            let ctx_entity = cx.entity().clone();
            let on_row_context_menu = move |row_key: SharedString,
                                            cj: &orbit_core::kubernetes::models::CronJobInfo,
                                            event: &MouseDownEvent,
                                            _window: &mut Window,
                                            cx: &mut App| {
                let name = cj.name.clone();
                let namespace = cj.namespace.clone();
                let drawer_target = ResourceDrawerTarget::CronJob {
                    name: cj.name.clone(),
                    namespace: cj.namespace.clone(),
                };
                let anchor = event.position;
                let copy_name = move |_w: &mut Window, cx: &mut App| {
                    cx.write_to_clipboard(ClipboardItem::new_string(name.clone()));
                };
                let filter_entity = ctx_entity.clone();
                let filter_ns = move |_w: &mut Window, cx: &mut App| {
                    filter_entity.update(cx, |shell, cx| {
                        shell.filter_by_namespace(&namespace, cx);
                    });
                };
                let refresh_ctx_entity = ctx_entity.clone();
                let refresh_cb = move |_w: &mut Window, cx: &mut App| {
                    refresh_ctx_entity.update(cx, |shell, cx| {
                        shell.refresh_workspace(cx);
                    });
                };
                let details_entity = ctx_entity.clone();
                let details_target = drawer_target.clone();
                let view_details = move |_w: &mut Window, cx: &mut App| {
                    details_entity.update(cx, |shell, cx| {
                        shell.open_resource_drawer(
                            details_target.clone(),
                            ResourceDrawerTabId::Overview,
                            cx,
                        );
                    });
                };
                let yaml_entity = ctx_entity.clone();
                let view_yaml = move |_w: &mut Window, cx: &mut App| {
                    yaml_entity.update(cx, |shell, cx| {
                        shell.open_resource_drawer(
                            drawer_target.clone(),
                            ResourceDrawerTabId::Yaml,
                            cx,
                        );
                    });
                };
                let trigger_action = {
                    let e = ctx_entity.clone();
                    let ns = cj.namespace.clone();
                    let name = cj.name.clone();
                    std::rc::Rc::new(move |_window: &mut Window, cx: &mut App| {
                        e.update(cx, |shell, cx| {
                            shell.trigger_cronjob_now(&ns, &name, cx);
                        });
                    })
                };
                let delete_action = {
                    let e = ctx_entity.clone();
                    let ns = cj.namespace.clone();
                    let name = cj.name.clone();
                    move |window: &mut Window, cx: &mut App| {
                        e.update(cx, |shell, cx| {
                            shell.open_delete_confirm_modal(
                                "CronJob",
                                Some(&ns),
                                &name,
                                window,
                                cx,
                            );
                        });
                    }
                };
                let entries = views::cronjobs::build_cronjob_context_menu_entries(
                    cj,
                    copy_name,
                    filter_ns,
                    refresh_cb,
                    view_details,
                    view_yaml,
                    move |w, cx| trigger_action(w, cx),
                    delete_action,
                );
                ctx_entity.update(cx, |shell, cx| {
                    shell.cronjobs.table.selected_row_key = Some(row_key.to_string());
                    shell.open_context_menu(anchor, entries, cx);
                });
            };

            let refresh_entity = cx.entity().clone();
            let on_refresh = move |_window: &mut Window, cx: &mut App| {
                refresh_entity.update(cx, |shell, cx| {
                    shell.refresh_workspace(cx);
                });
            };

            let yaml_entity = cx.entity().clone();
            let on_row_quick_yaml = move |_row_idx: usize,
                                          row_key: SharedString,
                                          _event: &gpui::ClickEvent,
                                          _window: &mut Window,
                                          cx: &mut App| {
                yaml_entity.update(cx, |shell, cx| {
                    shell.cronjobs.table.selected_row_key = Some(row_key.to_string());
                    if let Some((ns, name)) = row_key.as_ref().split_once('/') {
                        let target = ResourceDrawerTarget::CronJob {
                            name: name.to_string(),
                            namespace: ns.to_string(),
                        };
                        shell.open_resource_drawer(target, ResourceDrawerTabId::Yaml, cx);
                    } else {
                        cx.notify();
                    }
                });
            };

            return views::cronjobs::render_cronjobs_view(
                &self.cronjobs,
                &self.namespaces,
                theme,
                on_sort,
                on_row_click,
                on_row_quick_yaml,
                on_row_context_menu,
                on_refresh,
            );
        }

        if self.sidebar.active_view == WorkspaceView::Services {
            let sort_entity = cx.entity().clone();
            let on_sort = move |col_id: SharedString,
                                _event: &gpui::ClickEvent,
                                _window: &mut Window,
                                cx: &mut App| {
                sort_entity.update(cx, |shell, cx| {
                    shell.services.table.toggle_sort(col_id.as_ref());
                    let cfg = shell.services.table.to_persisted_config();
                    shell.persist_table_config("services", cfg);
                    cx.notify();
                });
            };

            let row_entity = cx.entity().clone();
            let on_row_click = move |_row_idx: usize,
                                     row_key: SharedString,
                                     _event: &gpui::ClickEvent,
                                     _window: &mut Window,
                                     cx: &mut App| {
                row_entity.update(cx, |shell, cx| {
                    shell.services.table.selected_row_key = Some(row_key.to_string());
                    if let Some((ns, name)) = row_key.as_ref().split_once('/') {
                        let target = ResourceDrawerTarget::Service {
                            name: name.to_string(),
                            namespace: ns.to_string(),
                        };
                        shell.open_resource_drawer(target, ResourceDrawerTabId::Overview, cx);
                    } else {
                        cx.notify();
                    }
                });
            };

            let ctx_entity = cx.entity().clone();
            let on_row_context_menu = move |row_key: SharedString,
                                            svc: &orbit_core::kubernetes::models::ServiceInfo,
                                            event: &MouseDownEvent,
                                            _window: &mut Window,
                                            cx: &mut App| {
                let name = svc.name.clone();
                let namespace = svc.namespace.clone();
                let drawer_target = ResourceDrawerTarget::Service {
                    name: svc.name.clone(),
                    namespace: svc.namespace.clone(),
                };
                let anchor = event.position;
                let copy_name = move |_w: &mut Window, cx: &mut App| {
                    cx.write_to_clipboard(ClipboardItem::new_string(name.clone()));
                };
                let filter_entity = ctx_entity.clone();
                let filter_ns = move |_w: &mut Window, cx: &mut App| {
                    filter_entity.update(cx, |shell, cx| {
                        shell.filter_by_namespace(&namespace, cx);
                    });
                };
                let refresh_ctx_entity = ctx_entity.clone();
                let refresh_cb = move |_w: &mut Window, cx: &mut App| {
                    refresh_ctx_entity.update(cx, |shell, cx| {
                        shell.refresh_workspace(cx);
                    });
                };
                let details_entity = ctx_entity.clone();
                let details_target = drawer_target.clone();
                let view_details = move |_w: &mut Window, cx: &mut App| {
                    details_entity.update(cx, |shell, cx| {
                        shell.open_resource_drawer(
                            details_target.clone(),
                            ResourceDrawerTabId::Overview,
                            cx,
                        );
                    });
                };
                let yaml_entity = ctx_entity.clone();
                let view_yaml = move |_w: &mut Window, cx: &mut App| {
                    yaml_entity.update(cx, |shell, cx| {
                        shell.open_resource_drawer(
                            drawer_target.clone(),
                            ResourceDrawerTabId::Yaml,
                            cx,
                        );
                    });
                };
                let delete_action = {
                    let e = ctx_entity.clone();
                    let ns = svc.namespace.clone();
                    let name = svc.name.clone();
                    move |window: &mut Window, cx: &mut App| {
                        e.update(cx, |shell, cx| {
                            shell.open_delete_confirm_modal(
                                "Service",
                                Some(&ns),
                                &name,
                                window,
                                cx,
                            );
                        });
                    }
                };
                let svc_ns = svc.namespace.clone();
                let svc_name = svc.name.clone();
                let svc_ports = extract_ports_from_service(svc);
                let pf_entity = ctx_entity.clone();
                let port_forward_action = move |_w: &mut Window, cx: &mut App| {
                    let ns = svc_ns.clone();
                    let name = svc_name.clone();
                    let ports = svc_ports.clone();
                    pf_entity.update(cx, |shell, cx| {
                        shell.open_port_forward_target_modal(&ns, "Service", &name, ports, cx);
                    });
                };
                let entries = views::services::build_service_context_menu_entries(
                    svc,
                    copy_name,
                    filter_ns,
                    refresh_cb,
                    view_details,
                    view_yaml,
                    port_forward_action,
                    delete_action,
                );
                ctx_entity.update(cx, |shell, cx| {
                    shell.services.table.selected_row_key = Some(row_key.to_string());
                    shell.open_context_menu(anchor, entries, cx);
                });
            };

            let type_filter_entity = cx.entity().clone();
            let on_type_filter = move |filter: crate::state::ServiceTypeFilter,
                                       _window: &mut Window,
                                       cx: &mut App| {
                type_filter_entity.update(cx, |shell, cx| {
                    shell.services.type_filter = filter;
                    cx.notify();
                });
            };

            let refresh_entity = cx.entity().clone();
            let on_refresh = move |_window: &mut Window, cx: &mut App| {
                refresh_entity.update(cx, |shell, cx| {
                    if let ClusterConnectionState::Connected { cluster_id, .. } =
                        &shell.state.connection_state
                    {
                        let cmd = shell
                            .services
                            .begin_refresh()
                            .unwrap_or_else(|| shell.services.begin_load_for_cluster(cluster_id));
                        shell.dispatch_command(cmd);
                        cx.notify();
                    }
                });
            };

            let yaml_entity = cx.entity().clone();
            let on_row_quick_yaml = move |_row_idx: usize,
                                          row_key: SharedString,
                                          _event: &gpui::ClickEvent,
                                          _window: &mut Window,
                                          cx: &mut App| {
                yaml_entity.update(cx, |shell, cx| {
                    shell.services.table.selected_row_key = Some(row_key.to_string());
                    if let Some((ns, name)) = row_key.as_ref().split_once('/') {
                        let target = ResourceDrawerTarget::Service {
                            name: name.to_string(),
                            namespace: ns.to_string(),
                        };
                        shell.open_resource_drawer(target, ResourceDrawerTabId::Yaml, cx);
                    } else {
                        cx.notify();
                    }
                });
            };

            return views::services::render_services_view(
                &self.services,
                &self.namespaces,
                theme,
                on_sort,
                on_row_click,
                on_row_quick_yaml,
                on_row_context_menu,
                on_type_filter,
                on_refresh,
                Some(self.table_enhancement_callbacks(WorkspaceView::Services, cx)),
            );
        }

        if self.sidebar.active_view == WorkspaceView::Ingresses {
            let sort_entity = cx.entity().clone();
            let on_sort = move |col_id: SharedString,
                                _event: &gpui::ClickEvent,
                                _window: &mut Window,
                                cx: &mut App| {
                sort_entity.update(cx, |shell, cx| {
                    shell.ingresses.table.toggle_sort(col_id.as_ref());
                    cx.notify();
                });
            };

            let row_entity = cx.entity().clone();
            let on_row_click = move |_row_idx: usize,
                                     row_key: SharedString,
                                     _event: &gpui::ClickEvent,
                                     _window: &mut Window,
                                     cx: &mut App| {
                row_entity.update(cx, |shell, cx| {
                    shell.ingresses.table.selected_row_key = Some(row_key.to_string());
                    if let Some((ns, name)) = row_key.as_ref().split_once('/') {
                        let target = ResourceDrawerTarget::Ingress {
                            name: name.to_string(),
                            namespace: ns.to_string(),
                        };
                        shell.open_resource_drawer(target, ResourceDrawerTabId::Overview, cx);
                    } else {
                        cx.notify();
                    }
                });
            };

            let ctx_entity = cx.entity().clone();
            let on_row_context_menu = move |row_key: SharedString,
                                            ing: &orbit_core::kubernetes::models::IngressInfo,
                                            event: &MouseDownEvent,
                                            _window: &mut Window,
                                            cx: &mut App| {
                let name = ing.name.clone();
                let namespace = ing.namespace.clone();
                let drawer_target = ResourceDrawerTarget::Ingress {
                    name: ing.name.clone(),
                    namespace: ing.namespace.clone(),
                };
                let anchor = event.position;
                let copy_name = move |_w: &mut Window, cx: &mut App| {
                    cx.write_to_clipboard(ClipboardItem::new_string(name.clone()));
                };
                let filter_entity = ctx_entity.clone();
                let filter_ns = move |_w: &mut Window, cx: &mut App| {
                    filter_entity.update(cx, |shell, cx| {
                        shell.filter_by_namespace(&namespace, cx);
                    });
                };
                let refresh_ctx_entity = ctx_entity.clone();
                let refresh_cb = move |_w: &mut Window, cx: &mut App| {
                    refresh_ctx_entity.update(cx, |shell, cx| {
                        shell.refresh_workspace(cx);
                    });
                };
                let details_entity = ctx_entity.clone();
                let details_target = drawer_target.clone();
                let view_details = move |_w: &mut Window, cx: &mut App| {
                    details_entity.update(cx, |shell, cx| {
                        shell.open_resource_drawer(
                            details_target.clone(),
                            ResourceDrawerTabId::Overview,
                            cx,
                        );
                    });
                };
                let yaml_entity = ctx_entity.clone();
                let view_yaml = move |_w: &mut Window, cx: &mut App| {
                    yaml_entity.update(cx, |shell, cx| {
                        shell.open_resource_drawer(
                            drawer_target.clone(),
                            ResourceDrawerTabId::Yaml,
                            cx,
                        );
                    });
                };
                let delete_action = {
                    let e = ctx_entity.clone();
                    let ns = ing.namespace.clone();
                    let name = ing.name.clone();
                    move |window: &mut Window, cx: &mut App| {
                        e.update(cx, |shell, cx| {
                            shell.open_delete_confirm_modal(
                                "Ingress",
                                Some(&ns),
                                &name,
                                window,
                                cx,
                            );
                        });
                    }
                };
                let entries = views::ingresses::build_ingress_context_menu_entries(
                    ing,
                    copy_name,
                    filter_ns,
                    refresh_cb,
                    view_details,
                    view_yaml,
                    delete_action,
                );
                ctx_entity.update(cx, |shell, cx| {
                    shell.ingresses.table.selected_row_key = Some(row_key.to_string());
                    shell.open_context_menu(anchor, entries, cx);
                });
            };

            let refresh_entity = cx.entity().clone();
            let on_refresh = move |_window: &mut Window, cx: &mut App| {
                refresh_entity.update(cx, |shell, cx| {
                    if let ClusterConnectionState::Connected { cluster_id, .. } =
                        &shell.state.connection_state
                    {
                        let cmd = shell
                            .ingresses
                            .begin_refresh()
                            .unwrap_or_else(|| shell.ingresses.begin_load_for_cluster(cluster_id));
                        shell.dispatch_command(cmd);
                        cx.notify();
                    }
                });
            };

            let yaml_entity = cx.entity().clone();
            let on_row_quick_yaml = move |_row_idx: usize,
                                          row_key: SharedString,
                                          _event: &gpui::ClickEvent,
                                          _window: &mut Window,
                                          cx: &mut App| {
                yaml_entity.update(cx, |shell, cx| {
                    shell.ingresses.table.selected_row_key = Some(row_key.to_string());
                    if let Some((ns, name)) = row_key.as_ref().split_once('/') {
                        let target = ResourceDrawerTarget::Ingress {
                            name: name.to_string(),
                            namespace: ns.to_string(),
                        };
                        shell.open_resource_drawer(target, ResourceDrawerTabId::Yaml, cx);
                    } else {
                        cx.notify();
                    }
                });
            };

            return views::ingresses::render_ingresses_view(
                &self.ingresses,
                &self.namespaces,
                theme,
                on_sort,
                on_row_click,
                on_row_quick_yaml,
                on_row_context_menu,
                on_refresh,
            );
        }

        if self.sidebar.active_view == WorkspaceView::PortForward {
            let sort_entity = cx.entity().clone();
            let on_sort = std::rc::Rc::new(
                move |col_id: SharedString,
                      _event: &gpui::ClickEvent,
                      _window: &mut Window,
                      cx: &mut App| {
                    sort_entity.update(cx, |shell, cx| {
                        shell.port_forwards.table.toggle_sort(col_id.as_ref());
                        cx.notify();
                    });
                },
            );

            let row_entity = cx.entity().clone();
            let on_row_click = std::rc::Rc::new(
                move |_row_idx: usize,
                      row_key: SharedString,
                      _event: &gpui::ClickEvent,
                      _window: &mut Window,
                      cx: &mut App| {
                    row_entity.update(cx, |shell, cx| {
                        shell.port_forwards.table.selected_row_key = Some(row_key.to_string());
                        shell.port_forwards.search_focused = false;
                        cx.notify();
                    });
                },
            );

            let ctx_entity = cx.entity().clone();
            let on_row_context_menu = std::rc::Rc::new(
                move |row_key: SharedString,
                      pf: &orbit_core::kubernetes::port_forward::PersistedPortForward,
                      event: &MouseDownEvent,
                      _window: &mut Window,
                      cx: &mut App| {
                    let local_port = pf.local_port;
                    let pf_id = pf.id.clone();
                    let anchor = event.position;

                    let open_entity = ctx_entity.clone();
                    let on_open_browser = move |_w: &mut Window, cx: &mut App| {
                        open_entity.update(cx, |shell, cx| {
                            shell.open_port_forward_in_browser(local_port, cx);
                        });
                    };
                    let copy_entity = ctx_entity.clone();
                    let on_copy_url = move |_w: &mut Window, cx: &mut App| {
                        copy_entity.update(cx, |shell, cx| {
                            shell.copy_port_forward_url(local_port, cx);
                        });
                    };
                    let stop_entity = ctx_entity.clone();
                    let on_stop = move |_w: &mut Window, cx: &mut App| {
                        let id = pf_id.clone();
                        stop_entity.update(cx, |shell, cx| {
                            shell.stop_port_forward(id, cx);
                        });
                    };

                    let entries = views::port_forward::build_port_forward_context_menu_entries(
                        pf,
                        on_open_browser,
                        on_copy_url,
                        on_stop,
                    );
                    ctx_entity.update(cx, |shell, cx| {
                        shell.port_forwards.table.selected_row_key = Some(row_key.to_string());
                        shell.open_context_menu(anchor, entries, cx);
                    });
                },
            );

            let select_entity = cx.entity().clone();
            let on_toggle_select =
                std::rc::Rc::new(move |id: String, _window: &mut Window, cx: &mut App| {
                    select_entity.update(cx, |shell, cx| {
                        shell.port_forwards.toggle_select(&id);
                        cx.notify();
                    });
                });

            let select_all_entity = cx.entity().clone();
            let on_toggle_select_all = std::rc::Rc::new(
                move |ids: Vec<String>, _window: &mut Window, cx: &mut App| {
                    select_all_entity.update(cx, |shell, cx| {
                        shell.port_forwards.toggle_select_all_visible(&ids);
                        cx.notify();
                    });
                },
            );

            let kind_entity = cx.entity().clone();
            let on_kind_filter = std::rc::Rc::new(
                move |filter: PortForwardKindFilter, _window: &mut Window, cx: &mut App| {
                    kind_entity.update(cx, |shell, cx| {
                        shell.port_forwards.set_kind_filter(filter);
                        cx.notify();
                    });
                },
            );

            let focus_entity = cx.entity().clone();
            let on_focus_search =
                std::rc::Rc::new(move |focused: bool, _window: &mut Window, cx: &mut App| {
                    focus_entity.update(cx, |shell, cx| {
                        shell.port_forwards.search_focused = focused;
                        cx.notify();
                    });
                });

            let clear_entity = cx.entity().clone();
            let on_clear_search = std::rc::Rc::new(move |_window: &mut Window, cx: &mut App| {
                clear_entity.update(cx, |shell, cx| {
                    shell.port_forwards.search_query.clear();
                    cx.notify();
                });
            });

            let create_entity = cx.entity().clone();
            let on_open_create_modal =
                std::rc::Rc::new(move |_window: &mut Window, cx: &mut App| {
                    create_entity.update(cx, |shell, cx| {
                        shell.open_port_forward_create_modal(cx);
                    });
                });

            let browser_entity = cx.entity().clone();
            let on_open_browser =
                std::rc::Rc::new(move |port: u16, _window: &mut Window, cx: &mut App| {
                    browser_entity.update(cx, |shell, cx| {
                        shell.open_port_forward_in_browser(port, cx);
                    });
                });

            let copy_entity = cx.entity().clone();
            let on_copy_url =
                std::rc::Rc::new(move |port: u16, _window: &mut Window, cx: &mut App| {
                    copy_entity.update(cx, |shell, cx| {
                        shell.copy_port_forward_url(port, cx);
                    });
                });

            let stop_entity = cx.entity().clone();
            let on_stop_tunnel =
                std::rc::Rc::new(move |id: String, _window: &mut Window, cx: &mut App| {
                    stop_entity.update(cx, |shell, cx| {
                        shell.stop_port_forward(id, cx);
                    });
                });

            let stop_sel_entity = cx.entity().clone();
            let on_stop_selected = std::rc::Rc::new(move |_window: &mut Window, cx: &mut App| {
                stop_sel_entity.update(cx, |shell, cx| {
                    shell.stop_selected_port_forwards(cx);
                });
            });

            let stop_all_entity = cx.entity().clone();
            let on_stop_all = std::rc::Rc::new(move |_window: &mut Window, cx: &mut App| {
                stop_all_entity.update(cx, |shell, cx| {
                    shell.stop_all_port_forwards(cx);
                });
            });

            let refresh_entity = cx.entity().clone();
            let on_refresh = std::rc::Rc::new(move |_window: &mut Window, cx: &mut App| {
                refresh_entity.update(cx, |shell, cx| {
                    if let ClusterConnectionState::Connected { cluster_id, .. } =
                        &shell.state.connection_state
                    {
                        let cmd = shell.port_forwards.begin_refresh().unwrap_or_else(|| {
                            shell.port_forwards.begin_load_for_cluster(cluster_id)
                        });
                        shell.dispatch_command(cmd);
                        cx.notify();
                    }
                });
            });

            return views::port_forward::render_port_forward_view(
                &self.port_forwards,
                &self.namespaces,
                theme,
                views::port_forward::PortForwardViewCallbacks {
                    on_sort,
                    on_row_click,
                    on_row_context_menu,
                    on_toggle_select,
                    on_toggle_select_all,
                    on_kind_filter,
                    on_focus_search,
                    on_clear_search,
                    on_open_create_modal,
                    on_open_browser,
                    on_copy_url,
                    on_stop_tunnel,
                    on_stop_selected,
                    on_stop_all,
                    on_refresh,
                },
            );
        }

        if self.sidebar.active_view == WorkspaceView::ConfigMaps {
            let sort_entity = cx.entity().clone();
            let on_sort = move |col_id: SharedString,
                                _event: &gpui::ClickEvent,
                                _window: &mut Window,
                                cx: &mut App| {
                sort_entity.update(cx, |shell, cx| {
                    shell.configmaps.table.toggle_sort(col_id.as_ref());
                    cx.notify();
                });
            };

            let row_entity = cx.entity().clone();
            let on_row_click = move |_row_idx: usize,
                                     row_key: SharedString,
                                     _event: &gpui::ClickEvent,
                                     _window: &mut Window,
                                     cx: &mut App| {
                row_entity.update(cx, |shell, cx| {
                    shell.configmaps.table.selected_row_key = Some(row_key.to_string());
                    if let Some((ns, name)) = row_key.as_ref().split_once('/') {
                        let target = ResourceDrawerTarget::ConfigMap {
                            name: name.to_string(),
                            namespace: ns.to_string(),
                        };
                        shell.open_resource_drawer(target, ResourceDrawerTabId::Overview, cx);
                    } else {
                        cx.notify();
                    }
                });
            };

            let ctx_entity = cx.entity().clone();
            let on_row_context_menu = move |row_key: SharedString,
                                            cm: &orbit_core::kubernetes::models::ConfigMapInfo,
                                            event: &MouseDownEvent,
                                            _window: &mut Window,
                                            cx: &mut App| {
                let name = cm.name.clone();
                let namespace = cm.namespace.clone();
                let drawer_target = ResourceDrawerTarget::ConfigMap {
                    name: cm.name.clone(),
                    namespace: cm.namespace.clone(),
                };
                let anchor = event.position;
                let copy_name = move |_w: &mut Window, cx: &mut App| {
                    cx.write_to_clipboard(ClipboardItem::new_string(name.clone()));
                };
                let filter_entity = ctx_entity.clone();
                let filter_ns = move |_w: &mut Window, cx: &mut App| {
                    filter_entity.update(cx, |shell, cx| {
                        shell.filter_by_namespace(&namespace, cx);
                    });
                };
                let refresh_ctx_entity = ctx_entity.clone();
                let refresh_cb = move |_w: &mut Window, cx: &mut App| {
                    refresh_ctx_entity.update(cx, |shell, cx| {
                        shell.refresh_workspace(cx);
                    });
                };
                let details_entity = ctx_entity.clone();
                let details_target = drawer_target.clone();
                let view_details = move |_w: &mut Window, cx: &mut App| {
                    details_entity.update(cx, |shell, cx| {
                        shell.open_resource_drawer(
                            details_target.clone(),
                            ResourceDrawerTabId::Overview,
                            cx,
                        );
                    });
                };
                let yaml_entity = ctx_entity.clone();
                let view_yaml = move |_w: &mut Window, cx: &mut App| {
                    yaml_entity.update(cx, |shell, cx| {
                        shell.open_resource_drawer(
                            drawer_target.clone(),
                            ResourceDrawerTabId::Yaml,
                            cx,
                        );
                    });
                };
                let delete_action = {
                    let e = ctx_entity.clone();
                    let ns = cm.namespace.clone();
                    let name = cm.name.clone();
                    move |window: &mut Window, cx: &mut App| {
                        e.update(cx, |shell, cx| {
                            shell.open_delete_confirm_modal(
                                "ConfigMap",
                                Some(&ns),
                                &name,
                                window,
                                cx,
                            );
                        });
                    }
                };
                let entries = views::configmaps::build_configmap_context_menu_entries(
                    cm,
                    copy_name,
                    filter_ns,
                    refresh_cb,
                    view_details,
                    view_yaml,
                    delete_action,
                );
                ctx_entity.update(cx, |shell, cx| {
                    shell.configmaps.table.selected_row_key = Some(row_key.to_string());
                    shell.open_context_menu(anchor, entries, cx);
                });
            };

            let refresh_entity = cx.entity().clone();
            let on_refresh = move |_window: &mut Window, cx: &mut App| {
                refresh_entity.update(cx, |shell, cx| {
                    if let ClusterConnectionState::Connected { cluster_id, .. } =
                        &shell.state.connection_state
                    {
                        let cmd = shell
                            .configmaps
                            .begin_refresh()
                            .unwrap_or_else(|| shell.configmaps.begin_load_for_cluster(cluster_id));
                        shell.dispatch_command(cmd);
                        cx.notify();
                    }
                });
            };

            let yaml_entity = cx.entity().clone();
            let on_row_quick_yaml = move |_row_idx: usize,
                                          row_key: SharedString,
                                          _event: &gpui::ClickEvent,
                                          _window: &mut Window,
                                          cx: &mut App| {
                yaml_entity.update(cx, |shell, cx| {
                    shell.configmaps.table.selected_row_key = Some(row_key.to_string());
                    if let Some((ns, name)) = row_key.as_ref().split_once('/') {
                        let target = ResourceDrawerTarget::ConfigMap {
                            name: name.to_string(),
                            namespace: ns.to_string(),
                        };
                        shell.open_resource_drawer(target, ResourceDrawerTabId::Yaml, cx);
                    } else {
                        cx.notify();
                    }
                });
            };

            return views::configmaps::render_configmaps_view(
                &self.configmaps,
                &self.namespaces,
                theme,
                on_sort,
                on_row_click,
                on_row_quick_yaml,
                on_row_context_menu,
                on_refresh,
            );
        }

        if self.sidebar.active_view == WorkspaceView::Secrets {
            let sort_entity = cx.entity().clone();
            let on_sort = move |col_id: SharedString,
                                _event: &gpui::ClickEvent,
                                _window: &mut Window,
                                cx: &mut App| {
                sort_entity.update(cx, |shell, cx| {
                    shell.secrets.table.toggle_sort(col_id.as_ref());
                    cx.notify();
                });
            };

            let row_entity = cx.entity().clone();
            let on_row_click = move |_row_idx: usize,
                                     row_key: SharedString,
                                     _event: &gpui::ClickEvent,
                                     _window: &mut Window,
                                     cx: &mut App| {
                row_entity.update(cx, |shell, cx| {
                    shell.secrets.table.selected_row_key = Some(row_key.to_string());
                    if let Some((ns, name)) = row_key.as_ref().split_once('/') {
                        let target = ResourceDrawerTarget::Secret {
                            name: name.to_string(),
                            namespace: ns.to_string(),
                        };
                        shell.open_resource_drawer(target, ResourceDrawerTabId::Overview, cx);
                    } else {
                        cx.notify();
                    }
                });
            };

            let ctx_entity = cx.entity().clone();
            let on_row_context_menu = move |row_key: SharedString,
                                            sec: &orbit_core::kubernetes::models::SecretInfo,
                                            event: &MouseDownEvent,
                                            _window: &mut Window,
                                            cx: &mut App| {
                let name = sec.name.clone();
                let namespace = sec.namespace.clone();
                let drawer_target = ResourceDrawerTarget::Secret {
                    name: sec.name.clone(),
                    namespace: sec.namespace.clone(),
                };
                let anchor = event.position;
                let copy_name = move |_w: &mut Window, cx: &mut App| {
                    cx.write_to_clipboard(ClipboardItem::new_string(name.clone()));
                };
                let filter_entity = ctx_entity.clone();
                let filter_ns = move |_w: &mut Window, cx: &mut App| {
                    filter_entity.update(cx, |shell, cx| {
                        shell.filter_by_namespace(&namespace, cx);
                    });
                };
                let refresh_ctx_entity = ctx_entity.clone();
                let refresh_cb = move |_w: &mut Window, cx: &mut App| {
                    refresh_ctx_entity.update(cx, |shell, cx| {
                        shell.refresh_workspace(cx);
                    });
                };
                let details_entity = ctx_entity.clone();
                let details_target = drawer_target.clone();
                let view_details = move |_w: &mut Window, cx: &mut App| {
                    details_entity.update(cx, |shell, cx| {
                        shell.open_resource_drawer(
                            details_target.clone(),
                            ResourceDrawerTabId::Overview,
                            cx,
                        );
                    });
                };
                let yaml_entity = ctx_entity.clone();
                let view_yaml = move |_w: &mut Window, cx: &mut App| {
                    yaml_entity.update(cx, |shell, cx| {
                        shell.open_resource_drawer(
                            drawer_target.clone(),
                            ResourceDrawerTabId::Yaml,
                            cx,
                        );
                    });
                };
                let delete_action = {
                    let e = ctx_entity.clone();
                    let ns = sec.namespace.clone();
                    let name = sec.name.clone();
                    move |window: &mut Window, cx: &mut App| {
                        e.update(cx, |shell, cx| {
                            shell.open_delete_confirm_modal("Secret", Some(&ns), &name, window, cx);
                        });
                    }
                };
                let entries = views::secrets::build_secret_context_menu_entries(
                    sec,
                    copy_name,
                    filter_ns,
                    refresh_cb,
                    view_details,
                    view_yaml,
                    delete_action,
                );
                ctx_entity.update(cx, |shell, cx| {
                    shell.secrets.table.selected_row_key = Some(row_key.to_string());
                    shell.open_context_menu(anchor, entries, cx);
                });
            };

            let refresh_entity = cx.entity().clone();
            let on_refresh = move |_window: &mut Window, cx: &mut App| {
                refresh_entity.update(cx, |shell, cx| {
                    if let ClusterConnectionState::Connected { cluster_id, .. } =
                        &shell.state.connection_state
                    {
                        let cmd = shell
                            .secrets
                            .begin_refresh()
                            .unwrap_or_else(|| shell.secrets.begin_load_for_cluster(cluster_id));
                        shell.dispatch_command(cmd);
                        cx.notify();
                    }
                });
            };

            let yaml_entity = cx.entity().clone();
            let on_row_quick_yaml = move |_row_idx: usize,
                                          row_key: SharedString,
                                          _event: &gpui::ClickEvent,
                                          _window: &mut Window,
                                          cx: &mut App| {
                yaml_entity.update(cx, |shell, cx| {
                    shell.secrets.table.selected_row_key = Some(row_key.to_string());
                    if let Some((ns, name)) = row_key.as_ref().split_once('/') {
                        let target = ResourceDrawerTarget::Secret {
                            name: name.to_string(),
                            namespace: ns.to_string(),
                        };
                        shell.open_resource_drawer(target, ResourceDrawerTabId::Yaml, cx);
                    } else {
                        cx.notify();
                    }
                });
            };

            return views::secrets::render_secrets_view(
                &self.secrets,
                &self.namespaces,
                theme,
                on_sort,
                on_row_click,
                on_row_quick_yaml,
                on_row_context_menu,
                on_refresh,
            );
        }

        if self.sidebar.active_view == WorkspaceView::Pods {
            let sort_entity = cx.entity().clone();
            let on_sort = move |col_id: SharedString,
                                _event: &gpui::ClickEvent,
                                _window: &mut Window,
                                cx: &mut App| {
                sort_entity.update(cx, |shell, cx| {
                    shell.pods.table.toggle_sort(col_id.as_ref());
                    let cfg = shell.pods.table.to_persisted_config();
                    shell.persist_table_config("pods", cfg);
                    cx.notify();
                });
            };

            let row_entity = cx.entity().clone();
            let on_row_click = move |_row_idx: usize,
                                     row_key: SharedString,
                                     _event: &gpui::ClickEvent,
                                     _window: &mut Window,
                                     cx: &mut App| {
                row_entity.update(cx, |shell, cx| {
                    shell.pods.table.selected_row_key = Some(row_key.to_string());
                    if let Some((ns, name)) = row_key.as_ref().split_once('/') {
                        let target = ResourceDrawerTarget::Pod {
                            name: name.to_string(),
                            namespace: ns.to_string(),
                        };
                        shell.open_resource_drawer(target, ResourceDrawerTabId::Overview, cx);
                    } else {
                        cx.notify();
                    }
                });
            };

            let ctx_entity = cx.entity().clone();
            let on_row_context_menu = move |row_key: SharedString,
                                            pod: &orbit_core::kubernetes::models::PodInfo,
                                            event: &MouseDownEvent,
                                            _window: &mut Window,
                                            cx: &mut App| {
                let name = pod.name.clone();
                let namespace = pod.namespace.clone();
                let drawer_target = ResourceDrawerTarget::Pod {
                    name: pod.name.clone(),
                    namespace: pod.namespace.clone(),
                };
                let anchor = event.position;
                let copy_name = move |_w: &mut Window, cx: &mut App| {
                    cx.write_to_clipboard(ClipboardItem::new_string(name.clone()));
                };
                let filter_entity = ctx_entity.clone();
                let filter_ns = move |_w: &mut Window, cx: &mut App| {
                    filter_entity.update(cx, |shell, cx| {
                        shell.filter_by_namespace(&namespace, cx);
                    });
                };
                let refresh_ctx_entity = ctx_entity.clone();
                let refresh_cb = move |_w: &mut Window, cx: &mut App| {
                    refresh_ctx_entity.update(cx, |shell, cx| {
                        shell.refresh_workspace(cx);
                    });
                };
                let details_entity = ctx_entity.clone();
                let details_target = drawer_target.clone();
                let view_details = move |_w: &mut Window, cx: &mut App| {
                    details_entity.update(cx, |shell, cx| {
                        shell.open_resource_drawer(
                            details_target.clone(),
                            ResourceDrawerTabId::Overview,
                            cx,
                        );
                    });
                };
                let yaml_entity = ctx_entity.clone();
                let view_yaml = move |_w: &mut Window, cx: &mut App| {
                    yaml_entity.update(cx, |shell, cx| {
                        shell.open_resource_drawer(
                            drawer_target.clone(),
                            ResourceDrawerTabId::Yaml,
                            cx,
                        );
                    });
                };
                let restart_pod_action = {
                    let e = ctx_entity.clone();
                    let ns = pod.namespace.clone();
                    let name = pod.name.clone();
                    std::rc::Rc::new(move |window: &mut Window, cx: &mut App| {
                        e.update(cx, |shell, cx| {
                            shell.open_restart_confirm_modal(
                                WorkloadActionKind::RestartPod,
                                "Pod",
                                &ns,
                                &name,
                                window,
                                cx,
                            );
                        });
                    })
                };
                let delete_action = {
                    let e = ctx_entity.clone();
                    let ns = pod.namespace.clone();
                    let name = pod.name.clone();
                    move |window: &mut Window, cx: &mut App| {
                        e.update(cx, |shell, cx| {
                            shell.open_delete_confirm_modal("Pod", Some(&ns), &name, window, cx);
                        });
                    }
                };
                let view_logs = {
                    let entity = ctx_entity.clone();
                    let name = pod.name.clone();
                    let ns = pod.namespace.clone();
                    move |_window: &mut Window, cx: &mut App| {
                        entity.update(cx, |shell, cx| {
                            shell.open_logs_tab("Pod", &ns, &name, cx);
                        });
                    }
                };
                let pod_ns = pod.namespace.clone();
                let pod_name = pod.name.clone();
                let pod_ports = extract_ports_from_pod(pod);
                let pf_entity = ctx_entity.clone();
                let port_forward_action = move |_w: &mut Window, cx: &mut App| {
                    let ns = pod_ns.clone();
                    let name = pod_name.clone();
                    let ports = pod_ports.clone();
                    pf_entity.update(cx, |shell, cx| {
                        shell.open_port_forward_target_modal(&ns, "Pod", &name, ports, cx);
                    });
                };
                let entries = views::pods::build_pod_context_menu_entries(
                    pod,
                    copy_name,
                    filter_ns,
                    refresh_cb,
                    view_details,
                    view_yaml,
                    view_logs,
                    port_forward_action,
                    move |w, cx| restart_pod_action(w, cx),
                    delete_action,
                );
                ctx_entity.update(cx, |shell, cx| {
                    shell.pods.table.selected_row_key = Some(row_key.to_string());
                    shell.open_context_menu(anchor, entries, cx);
                });
            };

            let refresh_entity = cx.entity().clone();
            let on_refresh = move |_window: &mut Window, cx: &mut App| {
                refresh_entity.update(cx, |shell, cx| {
                    shell.refresh_workspace(cx);
                });
            };

            let yaml_entity = cx.entity().clone();
            let on_row_quick_yaml = move |_row_idx: usize,
                                          row_key: SharedString,
                                          _event: &gpui::ClickEvent,
                                          _window: &mut Window,
                                          cx: &mut App| {
                yaml_entity.update(cx, |shell, cx| {
                    shell.pods.table.selected_row_key = Some(row_key.to_string());
                    if let Some((ns, name)) = row_key.as_ref().split_once('/') {
                        let target = ResourceDrawerTarget::Pod {
                            name: name.to_string(),
                            namespace: ns.to_string(),
                        };
                        shell.open_resource_drawer(target, ResourceDrawerTabId::Yaml, cx);
                    } else {
                        cx.notify();
                    }
                });
            };

            return views::pods::render_pods_view(
                &self.pods,
                &self.namespaces,
                theme,
                on_sort,
                on_row_click,
                on_row_quick_yaml,
                on_row_context_menu,
                on_refresh,
                Some(self.table_enhancement_callbacks(WorkspaceView::Pods, cx)),
            );
        }

        if self.sidebar.active_view == WorkspaceView::Namespaces {
            let sort_entity = cx.entity().clone();
            let on_sort = move |col_id: SharedString,
                                _event: &gpui::ClickEvent,
                                _window: &mut Window,
                                cx: &mut App| {
                sort_entity.update(cx, |shell, cx| {
                    shell.namespaces_view.table.toggle_sort(col_id.as_ref());
                    cx.notify();
                });
            };

            let row_entity = cx.entity().clone();
            let on_row_click = move |_row_idx: usize,
                                     row_key: SharedString,
                                     _event: &gpui::ClickEvent,
                                     _window: &mut Window,
                                     cx: &mut App| {
                row_entity.update(cx, |shell, cx| {
                    shell.namespaces_view.table.selected_row_key = Some(row_key.to_string());
                    let target = ResourceDrawerTarget::Namespace {
                        name: row_key.to_string(),
                    };
                    shell.open_resource_drawer(target, ResourceDrawerTabId::Overview, cx);
                });
            };

            let ctx_entity = cx.entity().clone();
            let on_row_context_menu = move |row_key: SharedString,
                                            ns: &orbit_core::kubernetes::models::NamespaceInfo,
                                            event: &MouseDownEvent,
                                            _window: &mut Window,
                                            cx: &mut App| {
                let name = ns.name.clone();
                let filter_name = ns.name.clone();
                let drawer_target = ResourceDrawerTarget::Namespace {
                    name: ns.name.clone(),
                };
                let anchor = event.position;
                let copy_name = move |_w: &mut Window, cx: &mut App| {
                    cx.write_to_clipboard(ClipboardItem::new_string(name.clone()));
                };
                let filter_entity = ctx_entity.clone();
                let filter_ns = move |_w: &mut Window, cx: &mut App| {
                    filter_entity.update(cx, |shell, cx| {
                        shell.filter_by_namespace(&filter_name, cx);
                    });
                };
                let refresh_ctx_entity = ctx_entity.clone();
                let refresh_cb = move |_w: &mut Window, cx: &mut App| {
                    refresh_ctx_entity.update(cx, |shell, cx| {
                        shell.refresh_workspace(cx);
                    });
                };
                let details_entity = ctx_entity.clone();
                let details_target = drawer_target.clone();
                let view_details = move |_w: &mut Window, cx: &mut App| {
                    details_entity.update(cx, |shell, cx| {
                        shell.open_resource_drawer(
                            details_target.clone(),
                            ResourceDrawerTabId::Overview,
                            cx,
                        );
                    });
                };
                let yaml_entity = ctx_entity.clone();
                let view_yaml = move |_w: &mut Window, cx: &mut App| {
                    yaml_entity.update(cx, |shell, cx| {
                        shell.open_resource_drawer(
                            drawer_target.clone(),
                            ResourceDrawerTabId::Yaml,
                            cx,
                        );
                    });
                };
                let delete_action = {
                    let e = ctx_entity.clone();
                    let name = ns.name.clone();
                    move |window: &mut Window, cx: &mut App| {
                        e.update(cx, |shell, cx| {
                            shell.open_delete_confirm_modal("Namespace", None, &name, window, cx);
                        });
                    }
                };
                let entries = views::namespaces::build_namespace_context_menu_entries(
                    ns,
                    copy_name,
                    filter_ns,
                    refresh_cb,
                    view_details,
                    view_yaml,
                    delete_action,
                );
                ctx_entity.update(cx, |shell, cx| {
                    shell.namespaces_view.table.selected_row_key = Some(row_key.to_string());
                    shell.open_context_menu(anchor, entries, cx);
                });
            };

            let refresh_entity = cx.entity().clone();
            let on_refresh = move |_window: &mut Window, cx: &mut App| {
                refresh_entity.update(cx, |shell, cx| {
                    shell.refresh_workspace(cx);
                });
            };

            let yaml_entity = cx.entity().clone();
            let on_row_quick_yaml = move |_row_idx: usize,
                                          row_key: SharedString,
                                          _event: &gpui::ClickEvent,
                                          _window: &mut Window,
                                          cx: &mut App| {
                yaml_entity.update(cx, |shell, cx| {
                    shell.namespaces_view.table.selected_row_key = Some(row_key.to_string());
                    let target = ResourceDrawerTarget::Namespace {
                        name: row_key.to_string(),
                    };
                    shell.open_resource_drawer(target, ResourceDrawerTabId::Yaml, cx);
                });
            };

            return views::namespaces::render_namespaces_view(
                &self.namespaces,
                &self.namespaces_view,
                theme,
                on_sort,
                on_row_click,
                on_row_quick_yaml,
                on_row_context_menu,
                on_refresh,
            );
        }

        if self.sidebar.active_view == WorkspaceView::PersistentVolumes {
            let sort_entity = cx.entity().clone();
            let on_sort = move |col_id: SharedString,
                                _event: &gpui::ClickEvent,
                                _window: &mut Window,
                                cx: &mut App| {
                sort_entity.update(cx, |shell, cx| {
                    shell.persistent_volumes.table.toggle_sort(col_id.as_ref());
                    cx.notify();
                });
            };

            let row_entity = cx.entity().clone();
            let on_row_click = move |_row_idx: usize,
                                     row_key: SharedString,
                                     _event: &gpui::ClickEvent,
                                     _window: &mut Window,
                                     cx: &mut App| {
                row_entity.update(cx, |shell, cx| {
                    shell.persistent_volumes.table.selected_row_key = Some(row_key.to_string());
                    let target = ResourceDrawerTarget::PersistentVolume {
                        name: row_key.to_string(),
                    };
                    shell.open_resource_drawer(target, ResourceDrawerTabId::Overview, cx);
                });
            };

            let ctx_entity = cx.entity().clone();
            let on_row_context_menu =
                move |row_key: SharedString,
                      pv: &orbit_core::kubernetes::models::PersistentVolumeInfo,
                      event: &MouseDownEvent,
                      _window: &mut Window,
                      cx: &mut App| {
                    let name = pv.name.clone();
                    let drawer_target = ResourceDrawerTarget::PersistentVolume {
                        name: pv.name.clone(),
                    };
                    let anchor = event.position;
                    let copy_name = move |_w: &mut Window, cx: &mut App| {
                        cx.write_to_clipboard(ClipboardItem::new_string(name.clone()));
                    };
                    let refresh_ctx_entity = ctx_entity.clone();
                    let refresh_cb = move |_w: &mut Window, cx: &mut App| {
                        refresh_ctx_entity.update(cx, |shell, cx| {
                            shell.refresh_workspace(cx);
                        });
                    };
                    let details_entity = ctx_entity.clone();
                    let details_target = drawer_target.clone();
                    let view_details = move |_w: &mut Window, cx: &mut App| {
                        details_entity.update(cx, |shell, cx| {
                            shell.open_resource_drawer(
                                details_target.clone(),
                                ResourceDrawerTabId::Overview,
                                cx,
                            );
                        });
                    };
                    let yaml_entity = ctx_entity.clone();
                    let view_yaml = move |_w: &mut Window, cx: &mut App| {
                        yaml_entity.update(cx, |shell, cx| {
                            shell.open_resource_drawer(
                                drawer_target.clone(),
                                ResourceDrawerTabId::Yaml,
                                cx,
                            );
                        });
                    };
                    let delete_action = {
                        let e = ctx_entity.clone();
                        let name = pv.name.clone();
                        move |window: &mut Window, cx: &mut App| {
                            e.update(cx, |shell, cx| {
                                shell.open_delete_confirm_modal(
                                    "PersistentVolume",
                                    None,
                                    &name,
                                    window,
                                    cx,
                                );
                            });
                        }
                    };
                    let entries =
                        views::persistent_volumes::build_persistent_volume_context_menu_entries(
                            pv,
                            copy_name,
                            refresh_cb,
                            view_details,
                            view_yaml,
                            delete_action,
                        );
                    ctx_entity.update(cx, |shell, cx| {
                        shell.persistent_volumes.table.selected_row_key = Some(row_key.to_string());
                        shell.open_context_menu(anchor, entries, cx);
                    });
                };

            let refresh_entity = cx.entity().clone();
            let on_refresh = move |_window: &mut Window, cx: &mut App| {
                refresh_entity.update(cx, |shell, cx| {
                    shell.refresh_workspace(cx);
                });
            };

            let yaml_entity = cx.entity().clone();
            let on_row_quick_yaml = move |_row_idx: usize,
                                          row_key: SharedString,
                                          _event: &gpui::ClickEvent,
                                          _window: &mut Window,
                                          cx: &mut App| {
                yaml_entity.update(cx, |shell, cx| {
                    shell.persistent_volumes.table.selected_row_key = Some(row_key.to_string());
                    let target = ResourceDrawerTarget::PersistentVolume {
                        name: row_key.to_string(),
                    };
                    shell.open_resource_drawer(target, ResourceDrawerTabId::Yaml, cx);
                });
            };

            return views::persistent_volumes::render_persistent_volumes_view(
                &self.persistent_volumes,
                theme,
                on_sort,
                on_row_click,
                on_row_quick_yaml,
                on_row_context_menu,
                on_refresh,
            );
        }

        if self.sidebar.active_view == WorkspaceView::PersistentVolumeClaims {
            let sort_entity = cx.entity().clone();
            let on_sort = move |col_id: SharedString,
                                _event: &gpui::ClickEvent,
                                _window: &mut Window,
                                cx: &mut App| {
                sort_entity.update(cx, |shell, cx| {
                    shell
                        .persistent_volume_claims
                        .table
                        .toggle_sort(col_id.as_ref());
                    cx.notify();
                });
            };

            let row_entity = cx.entity().clone();
            let on_row_click = move |_row_idx: usize,
                                     row_key: SharedString,
                                     _event: &gpui::ClickEvent,
                                     _window: &mut Window,
                                     cx: &mut App| {
                row_entity.update(cx, |shell, cx| {
                    shell.persistent_volume_claims.table.selected_row_key =
                        Some(row_key.to_string());
                    if let Some((ns, name)) = row_key.as_ref().split_once('/') {
                        let target = ResourceDrawerTarget::PersistentVolumeClaim {
                            name: name.to_string(),
                            namespace: ns.to_string(),
                        };
                        shell.open_resource_drawer(target, ResourceDrawerTabId::Overview, cx);
                    } else {
                        cx.notify();
                    }
                });
            };

            let ctx_entity = cx.entity().clone();
            let on_row_context_menu =
                move |row_key: SharedString,
                      pvc: &orbit_core::kubernetes::models::PersistentVolumeClaimInfo,
                      event: &MouseDownEvent,
                      _window: &mut Window,
                      cx: &mut App| {
                    let name = pvc.name.clone();
                    let namespace = pvc.namespace.clone();
                    let drawer_target = ResourceDrawerTarget::PersistentVolumeClaim {
                        name: pvc.name.clone(),
                        namespace: pvc.namespace.clone(),
                    };
                    let anchor = event.position;
                    let copy_name = move |_w: &mut Window, cx: &mut App| {
                        cx.write_to_clipboard(ClipboardItem::new_string(name.clone()));
                    };
                    let filter_entity = ctx_entity.clone();
                    let filter_ns = move |_w: &mut Window, cx: &mut App| {
                        filter_entity.update(cx, |shell, cx| {
                            shell.filter_by_namespace(&namespace, cx);
                        });
                    };
                    let refresh_ctx_entity = ctx_entity.clone();
                    let refresh_cb = move |_w: &mut Window, cx: &mut App| {
                        refresh_ctx_entity.update(cx, |shell, cx| {
                            shell.refresh_workspace(cx);
                        });
                    };
                    let details_entity = ctx_entity.clone();
                    let details_target = drawer_target.clone();
                    let view_details = move |_w: &mut Window, cx: &mut App| {
                        details_entity.update(cx, |shell, cx| {
                            shell.open_resource_drawer(
                                details_target.clone(),
                                ResourceDrawerTabId::Overview,
                                cx,
                            );
                        });
                    };
                    let yaml_entity = ctx_entity.clone();
                    let view_yaml = move |_w: &mut Window, cx: &mut App| {
                        yaml_entity.update(cx, |shell, cx| {
                            shell.open_resource_drawer(
                                drawer_target.clone(),
                                ResourceDrawerTabId::Yaml,
                                cx,
                            );
                        });
                    };
                    let delete_action = {
                        let e = ctx_entity.clone();
                        let ns = pvc.namespace.clone();
                        let name = pvc.name.clone();
                        move |window: &mut Window, cx: &mut App| {
                            e.update(cx, |shell, cx| {
                                shell.open_delete_confirm_modal(
                                    "PersistentVolumeClaim",
                                    Some(&ns),
                                    &name,
                                    window,
                                    cx,
                                );
                            });
                        }
                    };
                    let entries =
                    views::persistent_volume_claims::build_persistent_volume_claim_context_menu_entries(
                        pvc,
                        copy_name,
                        filter_ns,
                        refresh_cb,
                        view_details,
                        view_yaml,
                        delete_action,
                    );
                    ctx_entity.update(cx, |shell, cx| {
                        shell.persistent_volume_claims.table.selected_row_key =
                            Some(row_key.to_string());
                        shell.open_context_menu(anchor, entries, cx);
                    });
                };

            let refresh_entity = cx.entity().clone();
            let on_refresh = move |_window: &mut Window, cx: &mut App| {
                refresh_entity.update(cx, |shell, cx| {
                    shell.refresh_workspace(cx);
                });
            };

            let yaml_entity = cx.entity().clone();
            let on_row_quick_yaml = move |_row_idx: usize,
                                          row_key: SharedString,
                                          _event: &gpui::ClickEvent,
                                          _window: &mut Window,
                                          cx: &mut App| {
                yaml_entity.update(cx, |shell, cx| {
                    shell.persistent_volume_claims.table.selected_row_key =
                        Some(row_key.to_string());
                    if let Some((ns, name)) = row_key.as_ref().split_once('/') {
                        let target = ResourceDrawerTarget::PersistentVolumeClaim {
                            name: name.to_string(),
                            namespace: ns.to_string(),
                        };
                        shell.open_resource_drawer(target, ResourceDrawerTabId::Yaml, cx);
                    } else {
                        cx.notify();
                    }
                });
            };

            return views::persistent_volume_claims::render_persistent_volume_claims_view(
                &self.persistent_volume_claims,
                &self.namespaces,
                theme,
                on_sort,
                on_row_click,
                on_row_quick_yaml,
                on_row_context_menu,
                on_refresh,
            );
        }

        if self.sidebar.active_view == WorkspaceView::NetworkPolicies {
            let sort_entity = cx.entity().clone();
            let on_sort = move |col_id: SharedString,
                                _event: &gpui::ClickEvent,
                                _window: &mut Window,
                                cx: &mut App| {
                sort_entity.update(cx, |shell, cx| {
                    shell.network_policies.table.toggle_sort(col_id.as_ref());
                    cx.notify();
                });
            };

            let row_entity = cx.entity().clone();
            let on_row_click = move |_row_idx: usize,
                                     row_key: SharedString,
                                     _event: &gpui::ClickEvent,
                                     _window: &mut Window,
                                     cx: &mut App| {
                row_entity.update(cx, |shell, cx| {
                    shell.network_policies.table.selected_row_key = Some(row_key.to_string());
                    if let Some((ns, name)) = row_key.as_ref().split_once('/') {
                        let target = ResourceDrawerTarget::NetworkPolicy {
                            name: name.to_string(),
                            namespace: ns.to_string(),
                        };
                        shell.open_resource_drawer(target, ResourceDrawerTabId::Overview, cx);
                    } else {
                        cx.notify();
                    }
                });
            };

            let ctx_entity = cx.entity().clone();
            let on_row_context_menu = move |row_key: SharedString,
                                            policy: &orbit_core::kubernetes::models::PolicyInfo,
                                            event: &MouseDownEvent,
                                            _window: &mut Window,
                                            cx: &mut App| {
                let name = policy.name.clone();
                let namespace = policy.namespace.clone();
                let drawer_target = ResourceDrawerTarget::NetworkPolicy {
                    name: policy.name.clone(),
                    namespace: policy.namespace.clone(),
                };
                let anchor = event.position;
                let copy_name = move |_w: &mut Window, cx: &mut App| {
                    cx.write_to_clipboard(ClipboardItem::new_string(name.clone()));
                };
                let filter_entity = ctx_entity.clone();
                let filter_ns = move |_w: &mut Window, cx: &mut App| {
                    filter_entity.update(cx, |shell, cx| {
                        shell.filter_by_namespace(&namespace, cx);
                    });
                };
                let refresh_ctx_entity = ctx_entity.clone();
                let refresh_cb = move |_w: &mut Window, cx: &mut App| {
                    refresh_ctx_entity.update(cx, |shell, cx| {
                        shell.refresh_workspace(cx);
                    });
                };
                let details_entity = ctx_entity.clone();
                let details_target = drawer_target.clone();
                let view_details = move |_w: &mut Window, cx: &mut App| {
                    details_entity.update(cx, |shell, cx| {
                        shell.open_resource_drawer(
                            details_target.clone(),
                            ResourceDrawerTabId::Overview,
                            cx,
                        );
                    });
                };
                let yaml_entity = ctx_entity.clone();
                let view_yaml = move |_w: &mut Window, cx: &mut App| {
                    yaml_entity.update(cx, |shell, cx| {
                        shell.open_resource_drawer(
                            drawer_target.clone(),
                            ResourceDrawerTabId::Yaml,
                            cx,
                        );
                    });
                };
                let delete_action = {
                    let e = ctx_entity.clone();
                    let ns = policy.namespace.clone();
                    let name = policy.name.clone();
                    move |window: &mut Window, cx: &mut App| {
                        e.update(cx, |shell, cx| {
                            shell.open_delete_confirm_modal(
                                "NetworkPolicy",
                                Some(&ns),
                                &name,
                                window,
                                cx,
                            );
                        });
                    }
                };
                let entries = views::network_policies::build_network_policy_context_menu_entries(
                    policy,
                    copy_name,
                    filter_ns,
                    refresh_cb,
                    view_details,
                    view_yaml,
                    delete_action,
                );
                ctx_entity.update(cx, |shell, cx| {
                    shell.network_policies.table.selected_row_key = Some(row_key.to_string());
                    shell.open_context_menu(anchor, entries, cx);
                });
            };

            let refresh_entity = cx.entity().clone();
            let on_refresh = move |_window: &mut Window, cx: &mut App| {
                refresh_entity.update(cx, |shell, cx| {
                    shell.refresh_workspace(cx);
                });
            };

            let yaml_entity = cx.entity().clone();
            let on_row_quick_yaml = move |_row_idx: usize,
                                          row_key: SharedString,
                                          _event: &gpui::ClickEvent,
                                          _window: &mut Window,
                                          cx: &mut App| {
                yaml_entity.update(cx, |shell, cx| {
                    shell.network_policies.table.selected_row_key = Some(row_key.to_string());
                    if let Some((ns, name)) = row_key.as_ref().split_once('/') {
                        let target = ResourceDrawerTarget::NetworkPolicy {
                            name: name.to_string(),
                            namespace: ns.to_string(),
                        };
                        shell.open_resource_drawer(target, ResourceDrawerTabId::Yaml, cx);
                    } else {
                        cx.notify();
                    }
                });
            };

            return views::network_policies::render_network_policies_view(
                &self.network_policies,
                &self.namespaces,
                theme,
                on_sort,
                on_row_click,
                on_row_quick_yaml,
                on_row_context_menu,
                on_refresh,
            );
        }

        if self.sidebar.active_view == WorkspaceView::Nodes {
            let sort_entity = cx.entity().clone();
            let on_sort = move |col_id: SharedString,
                                _event: &gpui::ClickEvent,
                                _window: &mut Window,
                                cx: &mut App| {
                sort_entity.update(cx, |shell, cx| {
                    shell.nodes.table.toggle_sort(col_id.as_ref());
                    cx.notify();
                });
            };

            let row_entity = cx.entity().clone();
            let on_row_click = move |_row_idx: usize,
                                     row_key: SharedString,
                                     _event: &gpui::ClickEvent,
                                     _window: &mut Window,
                                     cx: &mut App| {
                row_entity.update(cx, |shell, cx| {
                    shell.nodes.table.selected_row_key = Some(row_key.to_string());
                    let target = ResourceDrawerTarget::Node {
                        name: row_key.to_string(),
                    };
                    shell.open_resource_drawer(target, ResourceDrawerTabId::Overview, cx);
                });
            };

            let ctx_entity = cx.entity().clone();
            let on_row_context_menu = move |row_key: SharedString,
                                            node: &orbit_core::kubernetes::models::NodeInfo,
                                            event: &MouseDownEvent,
                                            _window: &mut Window,
                                            cx: &mut App| {
                let name = node.name.clone();
                let drawer_target = ResourceDrawerTarget::Node {
                    name: node.name.clone(),
                };
                let anchor = event.position;
                let copy_name = move |_w: &mut Window, cx: &mut App| {
                    cx.write_to_clipboard(ClipboardItem::new_string(name.clone()));
                };
                let refresh_ctx_entity = ctx_entity.clone();
                let refresh_cb = move |_w: &mut Window, cx: &mut App| {
                    refresh_ctx_entity.update(cx, |shell, cx| {
                        shell.refresh_workspace(cx);
                    });
                };
                let details_entity = ctx_entity.clone();
                let details_target = drawer_target.clone();
                let view_details = move |_w: &mut Window, cx: &mut App| {
                    details_entity.update(cx, |shell, cx| {
                        shell.open_resource_drawer(
                            details_target.clone(),
                            ResourceDrawerTabId::Overview,
                            cx,
                        );
                    });
                };
                let yaml_entity = ctx_entity.clone();
                let view_yaml = move |_w: &mut Window, cx: &mut App| {
                    yaml_entity.update(cx, |shell, cx| {
                        shell.open_resource_drawer(
                            drawer_target.clone(),
                            ResourceDrawerTabId::Yaml,
                            cx,
                        );
                    });
                };
                let delete_action = {
                    let e = ctx_entity.clone();
                    let name = node.name.clone();
                    move |window: &mut Window, cx: &mut App| {
                        e.update(cx, |shell, cx| {
                            shell.open_delete_confirm_modal("Node", None, &name, window, cx);
                        });
                    }
                };
                let entries = views::nodes::build_node_context_menu_entries(
                    node,
                    copy_name,
                    refresh_cb,
                    view_details,
                    view_yaml,
                    delete_action,
                );
                ctx_entity.update(cx, |shell, cx| {
                    shell.nodes.table.selected_row_key = Some(row_key.to_string());
                    shell.open_context_menu(anchor, entries, cx);
                });
            };

            let refresh_entity = cx.entity().clone();
            let on_refresh = move |_window: &mut Window, cx: &mut App| {
                refresh_entity.update(cx, |shell, cx| {
                    shell.refresh_workspace(cx);
                });
            };

            let yaml_entity = cx.entity().clone();
            let on_row_quick_yaml = move |_row_idx: usize,
                                          row_key: SharedString,
                                          _event: &gpui::ClickEvent,
                                          _window: &mut Window,
                                          cx: &mut App| {
                yaml_entity.update(cx, |shell, cx| {
                    shell.nodes.table.selected_row_key = Some(row_key.to_string());
                    let target = ResourceDrawerTarget::Node {
                        name: row_key.to_string(),
                    };
                    shell.open_resource_drawer(target, ResourceDrawerTabId::Yaml, cx);
                });
            };

            return views::nodes::render_nodes_view(
                &self.nodes,
                theme,
                on_sort,
                on_row_click,
                on_row_quick_yaml,
                on_row_context_menu,
                on_refresh,
            );
        }

        if self.sidebar.active_view == WorkspaceView::WorkloadsOverview {
            let nav_entity = cx.entity().clone();
            let on_navigate = std::rc::Rc::new(
                move |view: WorkspaceView, _window: &mut Window, cx: &mut App| {
                    nav_entity.update(cx, |shell, cx| {
                        shell.select_workspace_view(view, cx);
                    });
                },
            );
            let refresh_entity = cx.entity().clone();
            let on_refresh = move |_window: &mut Window, cx: &mut App| {
                refresh_entity.update(cx, |shell, cx| {
                    shell.refresh_workspace(cx);
                });
            };
            return views::workloads_overview::render_workloads_overview_view(
                &self.workloads_overview,
                &self.namespaces,
                theme,
                on_navigate,
                on_refresh,
            );
        }

        if self.sidebar.active_view == WorkspaceView::StorageOverview {
            let nav_entity = cx.entity().clone();
            let on_navigate = std::rc::Rc::new(
                move |view: WorkspaceView, _window: &mut Window, cx: &mut App| {
                    nav_entity.update(cx, |shell, cx| {
                        shell.select_workspace_view(view, cx);
                    });
                },
            );
            let refresh_entity = cx.entity().clone();
            let on_refresh = move |_window: &mut Window, cx: &mut App| {
                refresh_entity.update(cx, |shell, cx| {
                    shell.refresh_workspace(cx);
                });
            };
            return views::storage_overview::render_storage_overview_view(
                &self.storage_overview,
                &self.namespaces,
                theme,
                on_navigate,
                on_refresh,
            );
        }

        if self.sidebar.active_view == WorkspaceView::PodSecurity {
            let sort_entity = cx.entity().clone();
            let on_sort = move |col_id: SharedString,
                                _event: &gpui::ClickEvent,
                                _window: &mut Window,
                                cx: &mut App| {
                sort_entity.update(cx, |shell, cx| {
                    shell.pod_security.table.toggle_sort(col_id.as_ref());
                    cx.notify();
                });
            };

            let row_entity = cx.entity().clone();
            let on_row_click = move |_row_idx: usize,
                                     row_key: SharedString,
                                     _event: &gpui::ClickEvent,
                                     _window: &mut Window,
                                     cx: &mut App| {
                row_entity.update(cx, |shell, cx| {
                    shell.pod_security.table.selected_row_key = Some(row_key.to_string());
                    if let Some((ns, name)) = row_key.as_ref().split_once('/') {
                        let target = ResourceDrawerTarget::PodSecurity {
                            name: name.to_string(),
                            namespace: ns.to_string(),
                        };
                        shell.open_resource_drawer(target, ResourceDrawerTabId::Overview, cx);
                    } else {
                        cx.notify();
                    }
                });
            };

            let ctx_entity = cx.entity().clone();
            let on_row_context_menu = move |row_key: SharedString,
                                            policy: &orbit_core::kubernetes::models::PolicyInfo,
                                            event: &MouseDownEvent,
                                            _window: &mut Window,
                                            cx: &mut App| {
                let name = policy.name.clone();
                let namespace = policy.namespace.clone();
                let drawer_target = ResourceDrawerTarget::PodSecurity {
                    name: policy.name.clone(),
                    namespace: policy.namespace.clone(),
                };
                let anchor = event.position;
                let copy_name = move |_w: &mut Window, cx: &mut App| {
                    cx.write_to_clipboard(ClipboardItem::new_string(name.clone()));
                };
                let filter_entity = ctx_entity.clone();
                let filter_ns = move |_w: &mut Window, cx: &mut App| {
                    filter_entity.update(cx, |shell, cx| {
                        shell.filter_by_namespace(&namespace, cx);
                    });
                };
                let refresh_ctx_entity = ctx_entity.clone();
                let refresh_cb = move |_w: &mut Window, cx: &mut App| {
                    refresh_ctx_entity.update(cx, |shell, cx| {
                        shell.refresh_workspace(cx);
                    });
                };
                let details_entity = ctx_entity.clone();
                let details_target = drawer_target.clone();
                let view_details = move |_w: &mut Window, cx: &mut App| {
                    details_entity.update(cx, |shell, cx| {
                        shell.open_resource_drawer(
                            details_target.clone(),
                            ResourceDrawerTabId::Overview,
                            cx,
                        );
                    });
                };
                let yaml_entity = ctx_entity.clone();
                let view_yaml = move |_w: &mut Window, cx: &mut App| {
                    yaml_entity.update(cx, |shell, cx| {
                        shell.open_resource_drawer(
                            drawer_target.clone(),
                            ResourceDrawerTabId::Yaml,
                            cx,
                        );
                    });
                };
                let entries = views::pod_security::build_pod_security_context_menu_entries(
                    policy,
                    copy_name,
                    filter_ns,
                    refresh_cb,
                    view_details,
                    view_yaml,
                );
                ctx_entity.update(cx, |shell, cx| {
                    shell.pod_security.table.selected_row_key = Some(row_key.to_string());
                    shell.open_context_menu(anchor, entries, cx);
                });
            };

            let refresh_entity = cx.entity().clone();
            let on_refresh = move |_window: &mut Window, cx: &mut App| {
                refresh_entity.update(cx, |shell, cx| {
                    shell.refresh_workspace(cx);
                });
            };

            let yaml_entity = cx.entity().clone();
            let on_row_quick_yaml = move |_row_idx: usize,
                                          row_key: SharedString,
                                          _event: &gpui::ClickEvent,
                                          _window: &mut Window,
                                          cx: &mut App| {
                yaml_entity.update(cx, |shell, cx| {
                    shell.pod_security.table.selected_row_key = Some(row_key.to_string());
                    if let Some((ns, name)) = row_key.as_ref().split_once('/') {
                        let target = ResourceDrawerTarget::PodSecurity {
                            name: name.to_string(),
                            namespace: ns.to_string(),
                        };
                        shell.open_resource_drawer(target, ResourceDrawerTabId::Yaml, cx);
                    } else {
                        cx.notify();
                    }
                });
            };

            return views::pod_security::render_pod_security_view(
                &self.pod_security,
                &self.namespaces,
                theme,
                on_sort,
                on_row_click,
                on_row_quick_yaml,
                on_row_context_menu,
                on_refresh,
            );
        }

        if self.sidebar.active_view == WorkspaceView::ResourceQuotas {
            let kind_entity = cx.entity().clone();
            let on_kind_filter =
                move |filter: ResourceQuotaKindFilter, _window: &mut Window, cx: &mut App| {
                    kind_entity.update(cx, |shell, cx| {
                        shell.resource_quotas.set_kind_filter(filter);
                        cx.notify();
                    });
                };

            let sort_entity = cx.entity().clone();
            let on_sort = move |col_id: SharedString,
                                _event: &gpui::ClickEvent,
                                _window: &mut Window,
                                cx: &mut App| {
                sort_entity.update(cx, |shell, cx| {
                    shell.resource_quotas.table.toggle_sort(col_id.as_ref());
                    cx.notify();
                });
            };

            let row_entity = cx.entity().clone();
            let on_row_click = move |_row_idx: usize,
                                     row_key: SharedString,
                                     _event: &gpui::ClickEvent,
                                     _window: &mut Window,
                                     cx: &mut App| {
                row_entity.update(cx, |shell, cx| {
                    shell.resource_quotas.table.selected_row_key = Some(row_key.to_string());
                    let mut parts = row_key.as_ref().splitn(3, '/');
                    if let (Some(kind), Some(ns), Some(name)) =
                        (parts.next(), parts.next(), parts.next())
                    {
                        let target = if kind.eq_ignore_ascii_case("LimitRange") {
                            ResourceDrawerTarget::LimitRange {
                                name: name.to_string(),
                                namespace: ns.to_string(),
                            }
                        } else {
                            ResourceDrawerTarget::ResourceQuota {
                                name: name.to_string(),
                                namespace: ns.to_string(),
                            }
                        };
                        shell.open_resource_drawer(target, ResourceDrawerTabId::Overview, cx);
                    } else {
                        cx.notify();
                    }
                });
            };

            let ctx_entity = cx.entity().clone();
            let on_row_context_menu = move |row_key: SharedString,
                                            policy: &orbit_core::kubernetes::models::PolicyInfo,
                                            event: &MouseDownEvent,
                                            _window: &mut Window,
                                            cx: &mut App| {
                let name = policy.name.clone();
                let namespace = policy.namespace.clone();
                let drawer_target = if policy.kind.eq_ignore_ascii_case("LimitRange") {
                    ResourceDrawerTarget::LimitRange {
                        name: policy.name.clone(),
                        namespace: policy.namespace.clone(),
                    }
                } else {
                    ResourceDrawerTarget::ResourceQuota {
                        name: policy.name.clone(),
                        namespace: policy.namespace.clone(),
                    }
                };
                let anchor = event.position;
                let copy_name = move |_w: &mut Window, cx: &mut App| {
                    cx.write_to_clipboard(ClipboardItem::new_string(name.clone()));
                };
                let filter_entity = ctx_entity.clone();
                let filter_ns = move |_w: &mut Window, cx: &mut App| {
                    filter_entity.update(cx, |shell, cx| {
                        shell.filter_by_namespace(&namespace, cx);
                    });
                };
                let refresh_ctx_entity = ctx_entity.clone();
                let refresh_cb = move |_w: &mut Window, cx: &mut App| {
                    refresh_ctx_entity.update(cx, |shell, cx| {
                        shell.refresh_workspace(cx);
                    });
                };
                let details_entity = ctx_entity.clone();
                let details_target = drawer_target.clone();
                let view_details = move |_w: &mut Window, cx: &mut App| {
                    details_entity.update(cx, |shell, cx| {
                        shell.open_resource_drawer(
                            details_target.clone(),
                            ResourceDrawerTabId::Overview,
                            cx,
                        );
                    });
                };
                let yaml_entity = ctx_entity.clone();
                let view_yaml = move |_w: &mut Window, cx: &mut App| {
                    yaml_entity.update(cx, |shell, cx| {
                        shell.open_resource_drawer(
                            drawer_target.clone(),
                            ResourceDrawerTabId::Yaml,
                            cx,
                        );
                    });
                };
                let delete_action = {
                    let e = ctx_entity.clone();
                    let kind = policy.kind.clone();
                    let ns = policy.namespace.clone();
                    let name = policy.name.clone();
                    move |window: &mut Window, cx: &mut App| {
                        e.update(cx, |shell, cx| {
                            shell.open_delete_confirm_modal(&kind, Some(&ns), &name, window, cx);
                        });
                    }
                };
                let entries = views::resource_quotas::build_resource_policy_context_menu_entries(
                    policy,
                    copy_name,
                    filter_ns,
                    refresh_cb,
                    view_details,
                    view_yaml,
                    delete_action,
                );
                ctx_entity.update(cx, |shell, cx| {
                    shell.resource_quotas.table.selected_row_key = Some(row_key.to_string());
                    shell.open_context_menu(anchor, entries, cx);
                });
            };

            let refresh_entity = cx.entity().clone();
            let on_refresh = move |_window: &mut Window, cx: &mut App| {
                refresh_entity.update(cx, |shell, cx| {
                    shell.refresh_workspace(cx);
                });
            };

            let yaml_entity = cx.entity().clone();
            let on_row_quick_yaml = move |_row_idx: usize,
                                          row_key: SharedString,
                                          _event: &gpui::ClickEvent,
                                          _window: &mut Window,
                                          cx: &mut App| {
                yaml_entity.update(cx, |shell, cx| {
                    shell.resource_quotas.table.selected_row_key = Some(row_key.to_string());
                    let mut parts = row_key.as_ref().splitn(3, '/');
                    if let (Some(kind), Some(ns), Some(name)) =
                        (parts.next(), parts.next(), parts.next())
                    {
                        let target = if kind.eq_ignore_ascii_case("LimitRange") {
                            ResourceDrawerTarget::LimitRange {
                                name: name.to_string(),
                                namespace: ns.to_string(),
                            }
                        } else {
                            ResourceDrawerTarget::ResourceQuota {
                                name: name.to_string(),
                                namespace: ns.to_string(),
                            }
                        };
                        shell.open_resource_drawer(target, ResourceDrawerTabId::Yaml, cx);
                    } else {
                        cx.notify();
                    }
                });
            };

            return views::resource_quotas::render_resource_quotas_view(
                &self.resource_quotas,
                &self.namespaces,
                theme,
                on_kind_filter,
                on_sort,
                on_row_click,
                on_row_quick_yaml,
                on_row_context_menu,
                on_refresh,
            );
        }

        if self.sidebar.active_view == WorkspaceView::Hpas {
            let sort_entity = cx.entity().clone();
            let on_sort = move |col_id: SharedString,
                                _event: &gpui::ClickEvent,
                                _window: &mut Window,
                                cx: &mut App| {
                sort_entity.update(cx, |shell, cx| {
                    shell.list_views.hpas.table.toggle_sort(col_id.as_ref());
                    cx.notify();
                });
            };

            let row_entity = cx.entity().clone();
            let on_row_click = move |_row_idx: usize,
                                     row_key: SharedString,
                                     _event: &gpui::ClickEvent,
                                     _window: &mut Window,
                                     cx: &mut App| {
                row_entity.update(cx, |shell, cx| {
                    shell.list_views.hpas.table.selected_row_key = Some(row_key.to_string());
                    if let Some((ns, name)) = row_key.as_ref().split_once('/') {
                        let target = ResourceDrawerTarget::Hpa {
                            name: name.to_string(),
                            namespace: ns.to_string(),
                        };
                        shell.open_resource_drawer(target, ResourceDrawerTabId::Overview, cx);
                    } else {
                        cx.notify();
                    }
                });
            };

            let yaml_entity = cx.entity().clone();
            let on_row_quick_yaml = move |_row_idx: usize,
                                          row_key: SharedString,
                                          _event: &gpui::ClickEvent,
                                          _window: &mut Window,
                                          cx: &mut App| {
                yaml_entity.update(cx, |shell, cx| {
                    shell.list_views.hpas.table.selected_row_key = Some(row_key.to_string());
                    if let Some((ns, name)) = row_key.as_ref().split_once('/') {
                        let target = ResourceDrawerTarget::Hpa {
                            name: name.to_string(),
                            namespace: ns.to_string(),
                        };
                        shell.open_resource_drawer(target, ResourceDrawerTabId::Yaml, cx);
                    } else {
                        cx.notify();
                    }
                });
            };

            let ctx_entity = cx.entity().clone();
            let on_row_context_menu = move |row_key: SharedString,
                                            hpa: &orbit_core::kubernetes::models::HpaInfo,
                                            event: &MouseDownEvent,
                                            _window: &mut Window,
                                            cx: &mut App| {
                let name = hpa.name.clone();
                let namespace = hpa.namespace.clone();
                let drawer_target = ResourceDrawerTarget::Hpa {
                    name: hpa.name.clone(),
                    namespace: hpa.namespace.clone(),
                };
                let anchor = event.position;
                let copy_name = move |_w: &mut Window, cx: &mut App| {
                    cx.write_to_clipboard(ClipboardItem::new_string(name.clone()));
                };
                let filter_entity = ctx_entity.clone();
                let filter_ns = move |_w: &mut Window, cx: &mut App| {
                    filter_entity.update(cx, |shell, cx| {
                        shell.filter_by_namespace(&namespace, cx);
                    });
                };
                let refresh_ctx_entity = ctx_entity.clone();
                let refresh_cb = move |_w: &mut Window, cx: &mut App| {
                    refresh_ctx_entity.update(cx, |shell, cx| {
                        shell.refresh_workspace(cx);
                    });
                };
                let details_entity = ctx_entity.clone();
                let details_target = drawer_target.clone();
                let view_details = move |_w: &mut Window, cx: &mut App| {
                    details_entity.update(cx, |shell, cx| {
                        shell.open_resource_drawer(
                            details_target.clone(),
                            ResourceDrawerTabId::Overview,
                            cx,
                        );
                    });
                };
                let yaml_ctx_entity = ctx_entity.clone();
                let view_yaml = move |_w: &mut Window, cx: &mut App| {
                    yaml_ctx_entity.update(cx, |shell, cx| {
                        shell.open_resource_drawer(
                            drawer_target.clone(),
                            ResourceDrawerTabId::Yaml,
                            cx,
                        );
                    });
                };
                let entries = views::hpas::build_hpa_context_menu_entries(
                    hpa,
                    copy_name,
                    filter_ns,
                    refresh_cb,
                    view_details,
                    view_yaml,
                );
                ctx_entity.update(cx, |shell, cx| {
                    shell.list_views.hpas.table.selected_row_key = Some(row_key.to_string());
                    shell.open_context_menu(anchor, entries, cx);
                });
            };

            let refresh_entity = cx.entity().clone();
            let on_refresh = move |_window: &mut Window, cx: &mut App| {
                refresh_entity.update(cx, |shell, cx| {
                    shell.refresh_workspace(cx);
                });
            };

            return views::hpas::render_hpas_view(
                &self.list_views.hpas,
                &self.namespaces,
                theme,
                on_sort,
                on_row_click,
                on_row_quick_yaml,
                on_row_context_menu,
                on_refresh,
            );
        }

        if self.sidebar.active_view == WorkspaceView::StorageClasses {
            let sort_entity = cx.entity().clone();
            let on_sort = move |col_id: SharedString,
                                _event: &gpui::ClickEvent,
                                _window: &mut Window,
                                cx: &mut App| {
                sort_entity.update(cx, |shell, cx| {
                    shell
                        .list_views
                        .storage_classes
                        .table
                        .toggle_sort(col_id.as_ref());
                    cx.notify();
                });
            };

            let row_entity = cx.entity().clone();
            let on_row_click = move |_row_idx: usize,
                                     row_key: SharedString,
                                     _event: &gpui::ClickEvent,
                                     _window: &mut Window,
                                     cx: &mut App| {
                row_entity.update(cx, |shell, cx| {
                    shell.list_views.storage_classes.table.selected_row_key =
                        Some(row_key.to_string());
                    let target = ResourceDrawerTarget::StorageClass {
                        name: row_key.to_string(),
                    };
                    shell.open_resource_drawer(target, ResourceDrawerTabId::Overview, cx);
                });
            };

            let yaml_entity = cx.entity().clone();
            let on_row_quick_yaml = move |_row_idx: usize,
                                          row_key: SharedString,
                                          _event: &gpui::ClickEvent,
                                          _window: &mut Window,
                                          cx: &mut App| {
                yaml_entity.update(cx, |shell, cx| {
                    shell.list_views.storage_classes.table.selected_row_key =
                        Some(row_key.to_string());
                    let target = ResourceDrawerTarget::StorageClass {
                        name: row_key.to_string(),
                    };
                    shell.open_resource_drawer(target, ResourceDrawerTabId::Yaml, cx);
                });
            };

            let ctx_entity = cx.entity().clone();
            let on_row_context_menu =
                move |row_key: SharedString,
                      sc: &orbit_core::kubernetes::models::StorageClassInfo,
                      event: &MouseDownEvent,
                      _window: &mut Window,
                      cx: &mut App| {
                    let name = sc.name.clone();
                    let drawer_target = ResourceDrawerTarget::StorageClass {
                        name: sc.name.clone(),
                    };
                    let anchor = event.position;
                    let copy_name = move |_w: &mut Window, cx: &mut App| {
                        cx.write_to_clipboard(ClipboardItem::new_string(name.clone()));
                    };
                    let refresh_ctx_entity = ctx_entity.clone();
                    let refresh_cb = move |_w: &mut Window, cx: &mut App| {
                        refresh_ctx_entity.update(cx, |shell, cx| {
                            shell.refresh_workspace(cx);
                        });
                    };
                    let details_entity = ctx_entity.clone();
                    let details_target = drawer_target.clone();
                    let view_details = move |_w: &mut Window, cx: &mut App| {
                        details_entity.update(cx, |shell, cx| {
                            shell.open_resource_drawer(
                                details_target.clone(),
                                ResourceDrawerTabId::Overview,
                                cx,
                            );
                        });
                    };
                    let yaml_ctx_entity = ctx_entity.clone();
                    let view_yaml = move |_w: &mut Window, cx: &mut App| {
                        yaml_ctx_entity.update(cx, |shell, cx| {
                            shell.open_resource_drawer(
                                drawer_target.clone(),
                                ResourceDrawerTabId::Yaml,
                                cx,
                            );
                        });
                    };
                    let entries = views::storage_classes::build_storage_class_context_menu_entries(
                        copy_name,
                        refresh_cb,
                        view_details,
                        view_yaml,
                    );
                    ctx_entity.update(cx, |shell, cx| {
                        shell.list_views.storage_classes.table.selected_row_key =
                            Some(row_key.to_string());
                        shell.open_context_menu(anchor, entries, cx);
                    });
                };

            let refresh_entity = cx.entity().clone();
            let on_refresh = move |_window: &mut Window, cx: &mut App| {
                refresh_entity.update(cx, |shell, cx| {
                    shell.refresh_workspace(cx);
                });
            };

            return views::storage_classes::render_storage_classes_view(
                &self.list_views.storage_classes,
                theme,
                on_sort,
                on_row_click,
                on_row_quick_yaml,
                on_row_context_menu,
                on_refresh,
            );
        }

        if self.sidebar.active_view == WorkspaceView::AdmissionWebhooks {
            let sort_entity = cx.entity().clone();
            let on_sort = move |col_id: SharedString,
                                _event: &gpui::ClickEvent,
                                _window: &mut Window,
                                cx: &mut App| {
                sort_entity.update(cx, |shell, cx| {
                    shell
                        .list_views
                        .admission_webhooks
                        .table
                        .toggle_sort(col_id.as_ref());
                    cx.notify();
                });
            };

            let row_entity = cx.entity().clone();
            let on_row_click = move |_row_idx: usize,
                                     row_key: SharedString,
                                     _event: &gpui::ClickEvent,
                                     _window: &mut Window,
                                     cx: &mut App| {
                row_entity.update(cx, |shell, cx| {
                    shell.list_views.admission_webhooks.table.selected_row_key =
                        Some(row_key.to_string());
                    if let Some((kind, name)) = row_key.as_ref().split_once('/') {
                        let target = ResourceDrawerTarget::AdmissionWebhook {
                            is_mutating: kind.eq_ignore_ascii_case("MutatingWebhookConfiguration"),
                            name: name.to_string(),
                        };
                        shell.open_resource_drawer(target, ResourceDrawerTabId::Overview, cx);
                    } else {
                        cx.notify();
                    }
                });
            };

            let yaml_entity = cx.entity().clone();
            let on_row_quick_yaml = move |_row_idx: usize,
                                          row_key: SharedString,
                                          _event: &gpui::ClickEvent,
                                          _window: &mut Window,
                                          cx: &mut App| {
                yaml_entity.update(cx, |shell, cx| {
                    shell.list_views.admission_webhooks.table.selected_row_key =
                        Some(row_key.to_string());
                    if let Some((kind, name)) = row_key.as_ref().split_once('/') {
                        let target = ResourceDrawerTarget::AdmissionWebhook {
                            is_mutating: kind.eq_ignore_ascii_case("MutatingWebhookConfiguration"),
                            name: name.to_string(),
                        };
                        shell.open_resource_drawer(target, ResourceDrawerTabId::Yaml, cx);
                    } else {
                        cx.notify();
                    }
                });
            };

            let ctx_entity = cx.entity().clone();
            let on_row_context_menu = move |row_key: SharedString,
                                            policy: &orbit_core::kubernetes::models::PolicyInfo,
                                            event: &MouseDownEvent,
                                            _window: &mut Window,
                                            cx: &mut App| {
                let name = policy.name.clone();
                let drawer_target = ResourceDrawerTarget::AdmissionWebhook {
                    is_mutating: policy
                        .kind
                        .eq_ignore_ascii_case("MutatingWebhookConfiguration"),
                    name: policy.name.clone(),
                };
                let anchor = event.position;
                let copy_name = move |_w: &mut Window, cx: &mut App| {
                    cx.write_to_clipboard(ClipboardItem::new_string(name.clone()));
                };
                let refresh_ctx_entity = ctx_entity.clone();
                let refresh_cb = move |_w: &mut Window, cx: &mut App| {
                    refresh_ctx_entity.update(cx, |shell, cx| {
                        shell.refresh_workspace(cx);
                    });
                };
                let details_entity = ctx_entity.clone();
                let details_target = drawer_target.clone();
                let view_details = move |_w: &mut Window, cx: &mut App| {
                    details_entity.update(cx, |shell, cx| {
                        shell.open_resource_drawer(
                            details_target.clone(),
                            ResourceDrawerTabId::Overview,
                            cx,
                        );
                    });
                };
                let yaml_ctx_entity = ctx_entity.clone();
                let view_yaml = move |_w: &mut Window, cx: &mut App| {
                    yaml_ctx_entity.update(cx, |shell, cx| {
                        shell.open_resource_drawer(
                            drawer_target.clone(),
                            ResourceDrawerTabId::Yaml,
                            cx,
                        );
                    });
                };
                let entries =
                    views::admission_webhooks::build_admission_webhook_context_menu_entries(
                        copy_name,
                        refresh_cb,
                        view_details,
                        view_yaml,
                    );
                ctx_entity.update(cx, |shell, cx| {
                    shell.list_views.admission_webhooks.table.selected_row_key =
                        Some(row_key.to_string());
                    shell.open_context_menu(anchor, entries, cx);
                });
            };

            let refresh_entity = cx.entity().clone();
            let on_refresh = move |_window: &mut Window, cx: &mut App| {
                refresh_entity.update(cx, |shell, cx| {
                    shell.refresh_workspace(cx);
                });
            };

            return views::admission_webhooks::render_admission_webhooks_view(
                &self.list_views.admission_webhooks,
                theme,
                on_sort,
                on_row_click,
                on_row_quick_yaml,
                on_row_context_menu,
                on_refresh,
            );
        }

        if self.sidebar.active_view == WorkspaceView::SecurityOverview {
            let sort_entity = cx.entity().clone();
            let on_sort = move |col_id: SharedString,
                                _event: &gpui::ClickEvent,
                                _window: &mut Window,
                                cx: &mut App| {
                sort_entity.update(cx, |shell, cx| {
                    shell
                        .list_views
                        .security_overview
                        .table
                        .toggle_sort(col_id.as_ref());
                    cx.notify();
                });
            };

            let row_entity = cx.entity().clone();
            let on_row_click = move |_row_idx: usize,
                                     row_key: SharedString,
                                     _event: &gpui::ClickEvent,
                                     _window: &mut Window,
                                     cx: &mut App| {
                row_entity.update(cx, |shell, cx| {
                    shell.list_views.security_overview.table.selected_row_key =
                        Some(row_key.to_string());
                    let mut parts = row_key.as_ref().splitn(3, '/');
                    if let (Some(kind), Some(ns), Some(name)) =
                        (parts.next(), parts.next(), parts.next())
                    {
                        let target = ResourceDrawerTarget::SecurityPolicy {
                            kind: kind.to_string(),
                            name: name.to_string(),
                            namespace: ns.to_string(),
                        };
                        shell.open_resource_drawer(target, ResourceDrawerTabId::Overview, cx);
                    } else {
                        cx.notify();
                    }
                });
            };

            let yaml_entity = cx.entity().clone();
            let on_row_quick_yaml = move |_row_idx: usize,
                                          row_key: SharedString,
                                          _event: &gpui::ClickEvent,
                                          _window: &mut Window,
                                          cx: &mut App| {
                yaml_entity.update(cx, |shell, cx| {
                    shell.list_views.security_overview.table.selected_row_key =
                        Some(row_key.to_string());
                    let mut parts = row_key.as_ref().splitn(3, '/');
                    if let (Some(kind), Some(ns), Some(name)) =
                        (parts.next(), parts.next(), parts.next())
                    {
                        let target = ResourceDrawerTarget::SecurityPolicy {
                            kind: kind.to_string(),
                            name: name.to_string(),
                            namespace: ns.to_string(),
                        };
                        shell.open_resource_drawer(target, ResourceDrawerTabId::Yaml, cx);
                    } else {
                        cx.notify();
                    }
                });
            };

            let ctx_entity = cx.entity().clone();
            let on_row_context_menu = move |row_key: SharedString,
                                            policy: &orbit_core::kubernetes::models::PolicyInfo,
                                            event: &MouseDownEvent,
                                            _window: &mut Window,
                                            cx: &mut App| {
                let name = policy.name.clone();
                let namespace = policy.namespace.clone();
                let drawer_target = ResourceDrawerTarget::SecurityPolicy {
                    kind: policy.kind.clone(),
                    name: policy.name.clone(),
                    namespace: policy.namespace.clone(),
                };
                let anchor = event.position;
                let copy_name = move |_w: &mut Window, cx: &mut App| {
                    cx.write_to_clipboard(ClipboardItem::new_string(name.clone()));
                };
                let filter_entity = ctx_entity.clone();
                let filter_ns = (!namespace.is_empty() && namespace != "-").then(move || {
                    let ns_clone = namespace.clone();
                    move |_w: &mut Window, cx: &mut App| {
                        filter_entity.update(cx, |shell, cx| {
                            shell.filter_by_namespace(&ns_clone, cx);
                        });
                    }
                });
                let refresh_ctx_entity = ctx_entity.clone();
                let refresh_cb = move |_w: &mut Window, cx: &mut App| {
                    refresh_ctx_entity.update(cx, |shell, cx| {
                        shell.refresh_workspace(cx);
                    });
                };
                let details_entity = ctx_entity.clone();
                let details_target = drawer_target.clone();
                let view_details = move |_w: &mut Window, cx: &mut App| {
                    details_entity.update(cx, |shell, cx| {
                        shell.open_resource_drawer(
                            details_target.clone(),
                            ResourceDrawerTabId::Overview,
                            cx,
                        );
                    });
                };
                let yaml_ctx_entity = ctx_entity.clone();
                let view_yaml = move |_w: &mut Window, cx: &mut App| {
                    yaml_ctx_entity.update(cx, |shell, cx| {
                        shell.open_resource_drawer(
                            drawer_target.clone(),
                            ResourceDrawerTabId::Yaml,
                            cx,
                        );
                    });
                };
                let entries = views::security_overview::build_security_policy_context_menu_entries(
                    policy,
                    copy_name,
                    filter_ns,
                    refresh_cb,
                    view_details,
                    view_yaml,
                );
                ctx_entity.update(cx, |shell, cx| {
                    shell.list_views.security_overview.table.selected_row_key =
                        Some(row_key.to_string());
                    shell.open_context_menu(anchor, entries, cx);
                });
            };

            let refresh_entity = cx.entity().clone();
            let on_refresh = move |_window: &mut Window, cx: &mut App| {
                refresh_entity.update(cx, |shell, cx| {
                    shell.refresh_workspace(cx);
                });
            };

            return views::security_overview::render_security_overview_view(
                &self.list_views.security_overview,
                &self.namespaces,
                theme,
                on_sort,
                on_row_click,
                on_row_quick_yaml,
                on_row_context_menu,
                on_refresh,
            );
        }

        if self.sidebar.active_view != WorkspaceView::Overview {
            let view = self.sidebar.active_view;
            return theme
                .style_card(div())
                .flex()
                .flex_col()
                .gap_3()
                .p_6()
                .w_full()
                .max_w(px(720.0))
                .bg(theme.surface.bg_panel)
                .child(
                    div()
                        .flex()
                        .items_center()
                        .justify_between()
                        .child(
                            apply_text_step(div(), theme.typography.size_base)
                                .font_weight(FontWeight::SEMIBOLD)
                                .child(format!("{} Workspace", view.label())),
                        )
                        .child(
                            apply_text_step(div(), theme.typography.size_xs)
                                .px_2p5()
                                .py_1()
                                .rounded(theme.style.radius_control)
                                .bg(theme.semantic.success.soft)
                                .font_weight(FontWeight::SEMIBOLD)
                                .text_color(theme.semantic.success.solid)
                                .child(format!("{display_name} • {version_label}")),
                        ),
                )
                .child(views::render_detail_row(
                    "View ID",
                    view.id().to_string(),
                    true,
                    theme,
                ))
                .child(views::render_detail_row(
                    "Active Context",
                    cluster_id.to_string(),
                    true,
                    theme,
                ))
                .child(views::render_detail_row(
                    "Kubernetes Version",
                    version_label,
                    true,
                    theme,
                ))
                .into_any_element();
        }

        let nav_entity = cx.entity().clone();
        let on_navigate = std::rc::Rc::new(
            move |view: WorkspaceView, _window: &mut Window, cx: &mut App| {
                nav_entity.update(cx, |shell, cx| {
                    shell.select_workspace_view(view, cx);
                });
            },
        );
        let refresh_entity = cx.entity().clone();
        let on_refresh = move |_window: &mut Window, cx: &mut App| {
            refresh_entity.update(cx, |shell, cx| {
                shell.refresh_workspace(cx);
            });
        };

        views::overview::render_overview_view(
            cluster_id,
            display_name,
            k8s_version,
            self.state.active_profile.as_ref(),
            self.state.active_cluster(),
            &self.overview,
            theme,
            on_navigate,
            on_refresh,
        )
    }

    fn record_notification(
        &mut self,
        severity: NotificationSeverity,
        title: impl Into<String>,
        message: impl Into<String>,
    ) {
        let now_secs = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        self.notifications.push(severity, title, message, now_secs);
    }

    fn render_shell_drawer_overlay(
        &self,
        window_width: f32,
        cx: &mut Context<Self>,
    ) -> Option<AnyElement> {
        let theme = OrbitTheme::get(cx).clone();
        match self.active_shell_drawer {
            ActiveShellDrawer::None => None,
            ActiveShellDrawer::Notifications => {
                let close_e = cx.entity().clone();
                let filter_e = cx.entity().clone();
                let mark_e = cx.entity().clone();
                let mark_all_e = cx.entity().clone();
                let remove_e = cx.entity().clone();
                let clear_e = cx.entity().clone();
                Some(render_notification_drawer(
                    &self.notifications,
                    window_width,
                    &theme,
                    NotificationDrawerCallbacks {
                        on_close: std::rc::Rc::new(move |_w, cx| {
                            close_e.update(cx, |shell, cx| {
                                if shell.active_shell_drawer.close() {
                                    cx.notify();
                                }
                            });
                        }),
                        on_select_filter: std::rc::Rc::new(move |tab, _w, cx| {
                            filter_e.update(cx, |shell, cx| {
                                if shell.notifications.filter != tab {
                                    shell.notifications.filter = tab;
                                    cx.notify();
                                }
                            });
                        }),
                        on_mark_read: std::rc::Rc::new(move |id, _w, cx| {
                            mark_e.update(cx, |shell, cx| {
                                if shell.notifications.mark_read(id) {
                                    cx.notify();
                                }
                            });
                        }),
                        on_mark_all_read: std::rc::Rc::new(move |_w, cx| {
                            mark_all_e.update(cx, |shell, cx| {
                                if shell.notifications.mark_all_read() {
                                    cx.notify();
                                }
                            });
                        }),
                        on_remove: std::rc::Rc::new(move |id, _w, cx| {
                            remove_e.update(cx, |shell, cx| {
                                if shell.notifications.remove(id) {
                                    cx.notify();
                                }
                            });
                        }),
                        on_clear_all: std::rc::Rc::new(move |_w, cx| {
                            clear_e.update(cx, |shell, cx| {
                                if shell.notifications.clear_all() {
                                    cx.notify();
                                }
                            });
                        }),
                        on_copy: std::rc::Rc::new(move |text, _w, cx| {
                            cx.write_to_clipboard(ClipboardItem::new_string(text));
                        }),
                    },
                ))
            }
            ActiveShellDrawer::UserProfile => {
                let close_e = cx.entity().clone();
                let switch_e = cx.entity().clone();
                let add_e = cx.entity().clone();
                let refresh_e = cx.entity().clone();
                let settings_e = cx.entity().clone();
                Some(render_profile_drawer(
                    self.state.active_profile.as_ref(),
                    &self.state.clusters,
                    self.state.active_cluster_id.as_deref(),
                    window_width,
                    &theme,
                    ProfileDrawerCallbacks {
                        on_close: std::rc::Rc::new(move |_w, cx| {
                            close_e.update(cx, |shell, cx| {
                                if shell.active_shell_drawer.close() {
                                    cx.notify();
                                }
                            });
                        }),
                        on_switch_cluster: std::rc::Rc::new(move |cluster_id, _w, cx| {
                            switch_e.update(cx, |shell, cx| {
                                shell.active_shell_drawer.close();
                                shell.select_cluster(&cluster_id, cx);
                            });
                        }),
                        on_add_cluster: std::rc::Rc::new(move |window, cx| {
                            add_e.update(cx, |shell, cx| {
                                shell.active_shell_drawer.close();
                                shell.open_add_cluster_modal(window, cx);
                            });
                        }),
                        on_refresh: std::rc::Rc::new(move |_w, cx| {
                            refresh_e.update(cx, |shell, cx| {
                                shell.refresh_workspace(cx);
                            });
                        }),
                        on_open_settings: std::rc::Rc::new(move |_w, cx| {
                            settings_e.update(cx, |shell, cx| {
                                shell.active_shell_drawer.close();
                                shell.select_workspace_view(WorkspaceView::Settings, cx);
                            });
                        }),
                    },
                ))
            }
        }
    }

    fn dismiss_overlays(&mut self) {
        self.dismiss_context_menu();
        self.tabs.close_picker();
        self.command_palette.close();
        self.state.dropdown_open = false;
        self.namespaces.close_dropdown();
        self.settings_dropdown.close();
        self.active_shell_drawer.close();
    }

    fn toggle_command_palette(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.command_palette.is_open {
            self.close_command_palette(window, cx);
        } else {
            self.dismiss_overlays();
            self.command_palette.open();
            self.command_palette_focus.focus(window);
            cx.notify();
        }
    }

    fn close_command_palette(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.command_palette.close();
        self.focus_handle.focus(window);
        cx.notify();
    }

    fn execute_command_palette_action(
        &mut self,
        action: CommandPaletteAction,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.command_palette.close();
        self.focus_handle.focus(window);
        match action {
            CommandPaletteAction::NavigateView(view) => {
                self.select_workspace_view(view, cx);
            }
            CommandPaletteAction::SwitchCluster(cluster_id) => {
                self.select_cluster(&cluster_id, cx);
            }
            CommandPaletteAction::SelectAllNamespaces
            | CommandPaletteAction::ClearNamespaceFilter => {
                if let Some(cleared) = self.namespaces.select_all_namespaces()
                    && let Some(cluster_id) = self.namespaces.cluster_id.clone()
                {
                    self.dispatch_command(ClusterBridgeCommand::SaveNamespaceSelection {
                        cluster_id,
                        namespaces: cleared,
                    });
                }
                self.reset_exemplar_pages();
                cx.notify();
            }
            CommandPaletteAction::ToggleNamespace(ns_name) => {
                if let Some(updated) = self.namespaces.toggle_namespace(&ns_name)
                    && let Some(cluster_id) = self.namespaces.cluster_id.clone()
                {
                    self.dispatch_command(ClusterBridgeCommand::SaveNamespaceSelection {
                        cluster_id,
                        namespaces: updated,
                    });
                }
                self.reset_exemplar_pages();
                cx.notify();
            }
            CommandPaletteAction::RefreshWorkspace => {
                self.refresh_workspace(cx);
            }
            CommandPaletteAction::ToggleSidebar => {
                self.sidebar.toggle_collapse();
                cx.notify();
            }
            CommandPaletteAction::OpenSettings => {
                self.select_workspace_view(WorkspaceView::Settings, cx);
            }
            CommandPaletteAction::AddCluster => {
                self.open_add_cluster_modal(window, cx);
            }
            CommandPaletteAction::DisconnectCluster => {
                self.disconnect_cluster(cx);
            }
        }
    }

    fn is_text_input_focused(&self) -> bool {
        if self.add_cluster.is_open
            || self.port_forward_modal.is_some()
            || self.workload_action_modal.is_open()
            || self.command_palette.is_open
            || self.tabs.picker_open
            || self.namespaces.dropdown_open
        {
            return true;
        }
        if let Some(WorkspaceTab::Logs(target)) = self.tabs.active_tab()
            && self
                .log_tabs
                .get(&target)
                .is_some_and(|log_state| log_state.search_focused)
        {
            return true;
        }
        match self.sidebar.active_view {
            WorkspaceView::Pods => self.pods.table.filters.search_focused,
            WorkspaceView::Deployments => self.deployments.table.filters.search_focused,
            WorkspaceView::Services => self.services.table.filters.search_focused,
            WorkspaceView::Events => self.events.table.filters.search_focused,
            WorkspaceView::PortForward => self.port_forwards.search_focused,
            _ => false,
        }
    }

    fn focus_active_view_search(&mut self, window: &mut Window, cx: &mut Context<Self>) -> bool {
        if let Some(WorkspaceTab::Logs(target)) = self.tabs.active_tab() {
            if let Some(log_state) = self.log_tabs.get_mut(&target) {
                log_state.search_focused = true;
                self.log_search_focus.focus(window);
                cx.notify();
                return true;
            }
            return false;
        }
        match self.sidebar.active_view {
            WorkspaceView::Pods => {
                self.pods.table.filters.search_focused = true;
                cx.notify();
                true
            }
            WorkspaceView::Deployments => {
                self.deployments.table.filters.search_focused = true;
                cx.notify();
                true
            }
            WorkspaceView::Services => {
                self.services.table.filters.search_focused = true;
                cx.notify();
                true
            }
            WorkspaceView::Events => {
                self.events.table.filters.search_focused = true;
                cx.notify();
                true
            }
            WorkspaceView::PortForward => {
                self.port_forwards.search_focused = true;
                cx.notify();
                true
            }
            _ => false,
        }
    }

    fn move_active_table_selection(&mut self, delta: isize, cx: &mut Context<Self>) -> bool {
        if matches!(self.tabs.active_tab(), Some(WorkspaceTab::Logs(_))) {
            return false;
        }
        let moved = match self.sidebar.active_view {
            WorkspaceView::Pods => {
                let keys: Vec<String> = views::pods::filter_and_sort_pods_with_filters(
                    &self.pods.pods,
                    &self.namespaces,
                    &self.pods.table.filters,
                    self.pods.table.sort.as_ref(),
                )
                .into_iter()
                .map(|p| format!("{}/{}", p.namespace, p.name))
                .collect();
                self.pods.table.move_selection_by_delta(delta, &keys).is_some()
            }
            WorkspaceView::Deployments => {
                let keys: Vec<String> =
                    views::deployments::filter_and_sort_deployments_with_filters(
                        &self.deployments.deployments,
                        &self.namespaces,
                        &self.deployments.table.filters,
                        self.deployments.table.sort.as_ref(),
                    )
                    .into_iter()
                    .map(|d| format!("{}/{}", d.namespace, d.name))
                    .collect();
                self.deployments
                    .table
                    .move_selection_by_delta(delta, &keys)
                    .is_some()
            }
            WorkspaceView::Services => {
                let keys: Vec<String> = views::services::filter_and_sort_services_with_filters(
                    &self.services.services,
                    &self.namespaces,
                    self.services.type_filter,
                    &self.services.table.filters,
                    self.services.table.sort.as_ref(),
                )
                .into_iter()
                .map(|s| format!("{}/{}", s.namespace, s.name))
                .collect();
                self.services
                    .table
                    .move_selection_by_delta(delta, &keys)
                    .is_some()
            }
            WorkspaceView::Events => {
                let keys: Vec<String> = views::events::filter_and_sort_events_with_filters(
                    &self.events.events,
                    &self.namespaces,
                    self.events.type_filter,
                    &self.events.table.filters,
                    self.events.table.sort.as_ref(),
                )
                .into_iter()
                .map(|ev| {
                    if !ev.uid.is_empty() {
                        ev.uid.clone()
                    } else {
                        format!("{}/{}/{}", ev.namespace, ev.reason, ev.object_name)
                    }
                })
                .collect();
                self.events
                    .table
                    .move_selection_by_delta(delta, &keys)
                    .is_some()
            }
            WorkspaceView::Nodes => {
                let keys: Vec<String> =
                    views::nodes::sort_nodes(&self.nodes.nodes, self.nodes.table.sort.as_ref())
                        .into_iter()
                        .map(|n| n.name.clone())
                        .collect();
                self.nodes
                    .table
                    .move_selection_by_delta(delta, &keys)
                    .is_some()
            }
            WorkspaceView::Namespaces => {
                let keys: Vec<String> = views::namespaces::sort_namespaces(
                    &self.namespaces.namespaces,
                    self.namespaces_view.table.sort.as_ref(),
                )
                .into_iter()
                .map(|ns| ns.name.clone())
                .collect();
                self.namespaces_view
                    .table
                    .move_selection_by_delta(delta, &keys)
                    .is_some()
            }
            WorkspaceView::StatefulSets => {
                let keys: Vec<String> = views::statefulsets::filter_and_sort_statefulsets(
                    &self.statefulsets.statefulsets,
                    &self.namespaces,
                    self.statefulsets.table.sort.as_ref(),
                )
                .into_iter()
                .map(|s| format!("{}/{}", s.namespace, s.name))
                .collect();
                self.statefulsets
                    .table
                    .move_selection_by_delta(delta, &keys)
                    .is_some()
            }
            WorkspaceView::DaemonSets => {
                let keys: Vec<String> = views::daemonsets::filter_and_sort_daemonsets(
                    &self.daemonsets.daemonsets,
                    &self.namespaces,
                    self.daemonsets.table.sort.as_ref(),
                )
                .into_iter()
                .map(|d| format!("{}/{}", d.namespace, d.name))
                .collect();
                self.daemonsets
                    .table
                    .move_selection_by_delta(delta, &keys)
                    .is_some()
            }
            WorkspaceView::ReplicaSets => {
                let keys: Vec<String> = views::replicasets::filter_and_sort_replicasets(
                    &self.replicasets.replicasets,
                    &self.namespaces,
                    self.replicasets.table.sort.as_ref(),
                )
                .into_iter()
                .map(|r| format!("{}/{}", r.namespace, r.name))
                .collect();
                self.replicasets
                    .table
                    .move_selection_by_delta(delta, &keys)
                    .is_some()
            }
            WorkspaceView::Jobs => {
                let keys: Vec<String> = views::jobs::filter_and_sort_jobs(
                    &self.jobs.jobs,
                    &self.namespaces,
                    self.jobs.table.sort.as_ref(),
                )
                .into_iter()
                .map(|j| format!("{}/{}", j.namespace, j.name))
                .collect();
                self.jobs
                    .table
                    .move_selection_by_delta(delta, &keys)
                    .is_some()
            }
            WorkspaceView::CronJobs => {
                let keys: Vec<String> = views::cronjobs::filter_and_sort_cronjobs(
                    &self.cronjobs.cronjobs,
                    &self.namespaces,
                    self.cronjobs.table.sort.as_ref(),
                )
                .into_iter()
                .map(|c| format!("{}/{}", c.namespace, c.name))
                .collect();
                self.cronjobs
                    .table
                    .move_selection_by_delta(delta, &keys)
                    .is_some()
            }
            WorkspaceView::Hpas => {
                let keys: Vec<String> = views::hpas::filter_and_sort_hpas(
                    &self.list_views.hpas.hpas,
                    &self.namespaces,
                    "",
                    self.list_views.hpas.table.sort.as_ref(),
                )
                .into_iter()
                .map(|h| format!("{}/{}", h.namespace, h.name))
                .collect();
                self.list_views
                    .hpas
                    .table
                    .move_selection_by_delta(delta, &keys)
                    .is_some()
            }
            WorkspaceView::Ingresses => {
                let keys: Vec<String> = views::ingresses::filter_and_sort_ingresses(
                    &self.ingresses.ingresses,
                    &self.namespaces,
                    self.ingresses.table.sort.as_ref(),
                )
                .into_iter()
                .map(|i| format!("{}/{}", i.namespace, i.name))
                .collect();
                self.ingresses
                    .table
                    .move_selection_by_delta(delta, &keys)
                    .is_some()
            }
            WorkspaceView::PersistentVolumes => {
                let keys: Vec<String> = views::persistent_volumes::sort_persistent_volumes(
                    &self.persistent_volumes.persistent_volumes,
                    self.persistent_volumes.table.sort.as_ref(),
                )
                .into_iter()
                .map(|pv| pv.name.clone())
                .collect();
                self.persistent_volumes
                    .table
                    .move_selection_by_delta(delta, &keys)
                    .is_some()
            }
            WorkspaceView::PersistentVolumeClaims => {
                let keys: Vec<String> =
                    views::persistent_volume_claims::filter_and_sort_persistent_volume_claims(
                        &self.persistent_volume_claims.persistent_volume_claims,
                        &self.namespaces,
                        self.persistent_volume_claims.table.sort.as_ref(),
                    )
                    .into_iter()
                    .map(|pvc| format!("{}/{}", pvc.namespace, pvc.name))
                    .collect();
                self.persistent_volume_claims
                    .table
                    .move_selection_by_delta(delta, &keys)
                    .is_some()
            }
            WorkspaceView::StorageClasses => {
                let keys: Vec<String> = views::storage_classes::filter_and_sort_storage_classes(
                    &self.list_views.storage_classes.classes,
                    "",
                    self.list_views.storage_classes.table.sort.as_ref(),
                )
                .into_iter()
                .map(|sc| sc.name.clone())
                .collect();
                self.list_views
                    .storage_classes
                    .table
                    .move_selection_by_delta(delta, &keys)
                    .is_some()
            }
            WorkspaceView::ConfigMaps => {
                let keys: Vec<String> = views::configmaps::filter_and_sort_configmaps(
                    &self.configmaps.configmaps,
                    &self.namespaces,
                    self.configmaps.table.sort.as_ref(),
                )
                .into_iter()
                .map(|cm| format!("{}/{}", cm.namespace, cm.name))
                .collect();
                self.configmaps
                    .table
                    .move_selection_by_delta(delta, &keys)
                    .is_some()
            }
            WorkspaceView::Secrets => {
                let keys: Vec<String> = views::secrets::filter_and_sort_secrets(
                    &self.secrets.secrets,
                    &self.namespaces,
                    self.secrets.table.sort.as_ref(),
                )
                .into_iter()
                .map(|s| format!("{}/{}", s.namespace, s.name))
                .collect();
                self.secrets
                    .table
                    .move_selection_by_delta(delta, &keys)
                    .is_some()
            }
            WorkspaceView::NetworkPolicies => {
                let keys: Vec<String> = views::network_policies::filter_and_sort_network_policies(
                    &self.network_policies.network_policies,
                    &self.namespaces,
                    self.network_policies.table.sort.as_ref(),
                )
                .into_iter()
                .map(|np| format!("{}/{}", np.namespace, np.name))
                .collect();
                self.network_policies
                    .table
                    .move_selection_by_delta(delta, &keys)
                    .is_some()
            }
            WorkspaceView::PodSecurity => {
                let keys: Vec<String> = views::pod_security::filter_and_sort_pod_security_policies(
                    &self.pod_security.policies,
                    &self.namespaces,
                    self.pod_security.table.sort.as_ref(),
                )
                .into_iter()
                .map(|p| format!("{}/{}", p.namespace, p.name))
                .collect();
                self.pod_security
                    .table
                    .move_selection_by_delta(delta, &keys)
                    .is_some()
            }
            WorkspaceView::ResourceQuotas => {
                let keys: Vec<String> = views::resource_quotas::filter_and_sort_resource_policies(
                    &self.resource_quotas.policies,
                    &self.namespaces,
                    self.resource_quotas.kind_filter,
                    self.resource_quotas.table.sort.as_ref(),
                )
                .into_iter()
                .map(|p| format!("{}/{}/{}", p.kind, p.namespace, p.name))
                .collect();
                self.resource_quotas
                    .table
                    .move_selection_by_delta(delta, &keys)
                    .is_some()
            }
            WorkspaceView::AdmissionWebhooks => {
                let keys: Vec<String> =
                    views::admission_webhooks::filter_and_sort_admission_webhooks(
                        &self.list_views.admission_webhooks.policies,
                        self.list_views.admission_webhooks.table.sort.as_ref(),
                    )
                    .into_iter()
                    .map(|w| format!("{}/{}", w.kind, w.name))
                    .collect();
                self.list_views
                    .admission_webhooks
                    .table
                    .move_selection_by_delta(delta, &keys)
                    .is_some()
            }
            WorkspaceView::SecurityOverview => {
                let keys: Vec<String> = views::security_overview::filter_and_sort_security_policies(
                    &self.list_views.security_overview.policies,
                    &self.namespaces,
                    self.list_views.security_overview.table.sort.as_ref(),
                )
                .into_iter()
                .map(|p| format!("{}/{}/{}", p.kind, p.namespace, p.name))
                .collect();
                self.list_views
                    .security_overview
                    .table
                    .move_selection_by_delta(delta, &keys)
                    .is_some()
            }
            _ => false,
        };
        if moved {
            cx.notify();
        }
        moved
    }

    fn open_selected_table_row_drawer(&mut self, cx: &mut Context<Self>) -> bool {
        let Some(target) = self.selected_table_row_drawer_target() else {
            return false;
        };
        self.open_resource_drawer(target, ResourceDrawerTabId::Overview, cx);
        true
    }

    fn selected_table_row_drawer_target(&self) -> Option<ResourceDrawerTarget> {
        if matches!(self.tabs.active_tab(), Some(WorkspaceTab::Logs(_))) {
            return None;
        }
        match self.sidebar.active_view {
            WorkspaceView::Pods => {
                let key = self.pods.table.selected_row_key.as_deref()?;
                views::pods::filter_and_sort_pods_with_filters(
                    &self.pods.pods,
                    &self.namespaces,
                    &self.pods.table.filters,
                    self.pods.table.sort.as_ref(),
                )
                .into_iter()
                .find(|p| format!("{}/{}", p.namespace, p.name) == key)
                .map(|p| ResourceDrawerTarget::Pod {
                    name: p.name.clone(),
                    namespace: p.namespace.clone(),
                })
            }
            WorkspaceView::Deployments => {
                let key = self.deployments.table.selected_row_key.as_deref()?;
                views::deployments::filter_and_sort_deployments_with_filters(
                    &self.deployments.deployments,
                    &self.namespaces,
                    &self.deployments.table.filters,
                    self.deployments.table.sort.as_ref(),
                )
                .into_iter()
                .find(|d| format!("{}/{}", d.namespace, d.name) == key)
                .map(|d| ResourceDrawerTarget::Deployment {
                    name: d.name.clone(),
                    namespace: d.namespace.clone(),
                })
            }
            WorkspaceView::Services => {
                let key = self.services.table.selected_row_key.as_deref()?;
                views::services::filter_and_sort_services_with_filters(
                    &self.services.services,
                    &self.namespaces,
                    self.services.type_filter,
                    &self.services.table.filters,
                    self.services.table.sort.as_ref(),
                )
                .into_iter()
                .find(|s| format!("{}/{}", s.namespace, s.name) == key)
                .map(|s| ResourceDrawerTarget::Service {
                    name: s.name.clone(),
                    namespace: s.namespace.clone(),
                })
            }
            WorkspaceView::Events => {
                let key = self.events.table.selected_row_key.as_deref()?;
                views::events::filter_and_sort_events_with_filters(
                    &self.events.events,
                    &self.namespaces,
                    self.events.type_filter,
                    &self.events.table.filters,
                    self.events.table.sort.as_ref(),
                )
                .into_iter()
                .find(|ev| {
                    let ev_key = if !ev.uid.is_empty() {
                        ev.uid.clone()
                    } else {
                        format!("{}/{}/{}", ev.namespace, ev.reason, ev.object_name)
                    };
                    ev_key == key
                })
                .map(|ev| ResourceDrawerTarget::Event {
                    uid: ev.uid.clone(),
                    name: ev.object_name.clone(),
                    reason: ev.reason.clone(),
                    namespace: ev.namespace.clone(),
                })
            }
            WorkspaceView::Nodes => {
                let key = self.nodes.table.selected_row_key.as_deref()?;
                self.nodes
                    .nodes
                    .iter()
                    .find(|n| n.name == key)
                    .map(|n| ResourceDrawerTarget::Node {
                        name: n.name.clone(),
                    })
            }
            WorkspaceView::Namespaces => {
                let key = self.namespaces_view.table.selected_row_key.as_deref()?;
                self.namespaces
                    .namespaces
                    .iter()
                    .find(|ns| ns.name == key)
                    .map(|ns| ResourceDrawerTarget::Namespace {
                        name: ns.name.clone(),
                    })
            }
            WorkspaceView::StatefulSets => {
                let key = self.statefulsets.table.selected_row_key.as_deref()?;
                self.statefulsets
                    .statefulsets
                    .iter()
                    .find(|s| format!("{}/{}", s.namespace, s.name) == key)
                    .map(|s| ResourceDrawerTarget::StatefulSet {
                        name: s.name.clone(),
                        namespace: s.namespace.clone(),
                    })
            }
            WorkspaceView::DaemonSets => {
                let key = self.daemonsets.table.selected_row_key.as_deref()?;
                self.daemonsets
                    .daemonsets
                    .iter()
                    .find(|d| format!("{}/{}", d.namespace, d.name) == key)
                    .map(|d| ResourceDrawerTarget::DaemonSet {
                        name: d.name.clone(),
                        namespace: d.namespace.clone(),
                    })
            }
            WorkspaceView::ReplicaSets => {
                let key = self.replicasets.table.selected_row_key.as_deref()?;
                self.replicasets
                    .replicasets
                    .iter()
                    .find(|r| format!("{}/{}", r.namespace, r.name) == key)
                    .map(|r| ResourceDrawerTarget::ReplicaSet {
                        name: r.name.clone(),
                        namespace: r.namespace.clone(),
                    })
            }
            WorkspaceView::Jobs => {
                let key = self.jobs.table.selected_row_key.as_deref()?;
                self.jobs
                    .jobs
                    .iter()
                    .find(|j| format!("{}/{}", j.namespace, j.name) == key)
                    .map(|j| ResourceDrawerTarget::Job {
                        name: j.name.clone(),
                        namespace: j.namespace.clone(),
                    })
            }
            WorkspaceView::CronJobs => {
                let key = self.cronjobs.table.selected_row_key.as_deref()?;
                self.cronjobs
                    .cronjobs
                    .iter()
                    .find(|c| format!("{}/{}", c.namespace, c.name) == key)
                    .map(|c| ResourceDrawerTarget::CronJob {
                        name: c.name.clone(),
                        namespace: c.namespace.clone(),
                    })
            }
            WorkspaceView::Hpas => {
                let key = self.list_views.hpas.table.selected_row_key.as_deref()?;
                self.list_views
                    .hpas
                    .hpas
                    .iter()
                    .find(|h| format!("{}/{}", h.namespace, h.name) == key)
                    .map(|h| ResourceDrawerTarget::Hpa {
                        name: h.name.clone(),
                        namespace: h.namespace.clone(),
                    })
            }
            WorkspaceView::Ingresses => {
                let key = self.ingresses.table.selected_row_key.as_deref()?;
                self.ingresses
                    .ingresses
                    .iter()
                    .find(|i| format!("{}/{}", i.namespace, i.name) == key)
                    .map(|i| ResourceDrawerTarget::Ingress {
                        name: i.name.clone(),
                        namespace: i.namespace.clone(),
                    })
            }
            WorkspaceView::PersistentVolumes => {
                let key = self.persistent_volumes.table.selected_row_key.as_deref()?;
                self.persistent_volumes
                    .persistent_volumes
                    .iter()
                    .find(|pv| pv.name == key)
                    .map(|pv| ResourceDrawerTarget::PersistentVolume {
                        name: pv.name.clone(),
                    })
            }
            WorkspaceView::PersistentVolumeClaims => {
                let key = self
                    .persistent_volume_claims
                    .table
                    .selected_row_key
                    .as_deref()?;
                self.persistent_volume_claims
                    .persistent_volume_claims
                    .iter()
                    .find(|pvc| format!("{}/{}", pvc.namespace, pvc.name) == key)
                    .map(|pvc| ResourceDrawerTarget::PersistentVolumeClaim {
                        name: pvc.name.clone(),
                        namespace: pvc.namespace.clone(),
                    })
            }
            WorkspaceView::StorageClasses => {
                let key = self
                    .list_views
                    .storage_classes
                    .table
                    .selected_row_key
                    .as_deref()?;
                self.list_views
                    .storage_classes
                    .classes
                    .iter()
                    .find(|sc| sc.name == key)
                    .map(|sc| ResourceDrawerTarget::StorageClass {
                        name: sc.name.clone(),
                    })
            }
            WorkspaceView::ConfigMaps => {
                let key = self.configmaps.table.selected_row_key.as_deref()?;
                self.configmaps
                    .configmaps
                    .iter()
                    .find(|cm| format!("{}/{}", cm.namespace, cm.name) == key)
                    .map(|cm| ResourceDrawerTarget::ConfigMap {
                        name: cm.name.clone(),
                        namespace: cm.namespace.clone(),
                    })
            }
            WorkspaceView::Secrets => {
                let key = self.secrets.table.selected_row_key.as_deref()?;
                self.secrets
                    .secrets
                    .iter()
                    .find(|s| format!("{}/{}", s.namespace, s.name) == key)
                    .map(|s| ResourceDrawerTarget::Secret {
                        name: s.name.clone(),
                        namespace: s.namespace.clone(),
                    })
            }
            WorkspaceView::NetworkPolicies => {
                let key = self.network_policies.table.selected_row_key.as_deref()?;
                self.network_policies
                    .network_policies
                    .iter()
                    .find(|np| format!("{}/{}", np.namespace, np.name) == key)
                    .map(|np| ResourceDrawerTarget::NetworkPolicy {
                        name: np.name.clone(),
                        namespace: np.namespace.clone(),
                    })
            }
            WorkspaceView::PodSecurity => {
                let key = self.pod_security.table.selected_row_key.as_deref()?;
                self.pod_security
                    .policies
                    .iter()
                    .find(|p| format!("{}/{}", p.namespace, p.name) == key)
                    .map(|p| ResourceDrawerTarget::PodSecurity {
                        name: p.name.clone(),
                        namespace: p.namespace.clone(),
                    })
            }
            WorkspaceView::ResourceQuotas => {
                let key = self.resource_quotas.table.selected_row_key.as_deref()?;
                self.resource_quotas
                    .policies
                    .iter()
                    .find(|p| format!("{}/{}/{}", p.kind, p.namespace, p.name) == key)
                    .map(|p| {
                        if p.kind.eq_ignore_ascii_case("LimitRange") {
                            ResourceDrawerTarget::LimitRange {
                                name: p.name.clone(),
                                namespace: p.namespace.clone(),
                            }
                        } else {
                            ResourceDrawerTarget::ResourceQuota {
                                name: p.name.clone(),
                                namespace: p.namespace.clone(),
                            }
                        }
                    })
            }
            WorkspaceView::AdmissionWebhooks => {
                let key = self
                    .list_views
                    .admission_webhooks
                    .table
                    .selected_row_key
                    .as_deref()?;
                self.list_views
                    .admission_webhooks
                    .policies
                    .iter()
                    .find(|w| format!("{}/{}", w.kind, w.name) == key)
                    .map(|w| ResourceDrawerTarget::AdmissionWebhook {
                        is_mutating: w.kind.contains("Mutating"),
                        name: w.name.clone(),
                    })
            }
            WorkspaceView::SecurityOverview => {
                let key = self
                    .list_views
                    .security_overview
                    .table
                    .selected_row_key
                    .as_deref()?;
                self.list_views
                    .security_overview
                    .policies
                    .iter()
                    .find(|p| format!("{}/{}/{}", p.kind, p.namespace, p.name) == key)
                    .map(|p| ResourceDrawerTarget::SecurityPolicy {
                        kind: p.kind.clone(),
                        name: p.name.clone(),
                        namespace: p.namespace.clone(),
                    })
            }
            _ => None,
        }
    }

    fn render_command_palette_overlay_view(
        &self,
        window: &Window,
        cx: &mut Context<Self>,
    ) -> Option<AnyElement> {
        if !self.command_palette.is_open {
            return None;
        }
        let theme = OrbitTheme::get(cx).clone();
        let catalog = build_command_palette_items(
            &self.state.clusters,
            self.state.active_cluster_id.as_deref(),
            &self.namespaces,
        );
        let filtered = self.command_palette.filter_items(&catalog);
        let exec_entity = cx.entity().clone();
        let clear_entity = cx.entity().clone();
        let close_entity = cx.entity().clone();
        Some(render_command_palette_overlay(
            &self.command_palette,
            &filtered,
            &theme,
            &self.command_palette_focus,
            self.command_palette_focus.is_focused(window),
            CommandPaletteCallbacks {
                on_execute: std::rc::Rc::new(move |action, window, cx| {
                    exec_entity.update(cx, |shell, cx| {
                        shell.execute_command_palette_action(action, window, cx);
                    });
                }),
                on_clear_query: std::rc::Rc::new(move |window, cx| {
                    clear_entity.update(cx, |shell, cx| {
                        shell.command_palette.clear_query();
                        shell.command_palette_focus.focus(window);
                        cx.notify();
                    });
                }),
                on_close: std::rc::Rc::new(move |window, cx| {
                    close_entity.update(cx, |shell, cx| {
                        shell.close_command_palette(window, cx);
                    });
                }),
            },
        ))
    }

    fn apply_focus_target(&self, target: ShellFocusTarget, window: &mut Window) {
        match target {
            ShellFocusTarget::ShellRoot => self.focus_handle.focus(window),
            ShellFocusTarget::PickerSearch => self.picker_search_focus.focus(window),
        }
    }

    fn open_or_toggle_picker(&mut self, toggle: bool, window: &mut Window, cx: &mut Context<Self>) {
        let should_open = if toggle { !self.tabs.picker_open } else { true };
        self.dismiss_overlays();
        if should_open {
            self.tabs.open_picker();
        }
        self.apply_focus_target(
            focus_target_for_picker_transition(self.tabs.picker_open),
            window,
        );
        cx.notify();
    }

    fn build_tab_context_menu_callbacks(cx: &Context<Self>) -> TabContextMenuCallbacks {
        let move_left_e = cx.entity().clone();
        let move_right_e = cx.entity().clone();
        let close_e = cx.entity().clone();
        let close_others_e = cx.entity().clone();
        let close_right_e = cx.entity().clone();
        let close_all_e = cx.entity().clone();

        TabContextMenuCallbacks {
            on_move_left: std::rc::Rc::new(move |v, _w, cx| {
                move_left_e.update(cx, |s, cx| s.move_tab_left(v, cx));
            }),
            on_move_right: std::rc::Rc::new(move |v, _w, cx| {
                move_right_e.update(cx, |s, cx| s.move_tab_right(v, cx));
            }),
            on_close: std::rc::Rc::new(move |v, _w, cx| {
                close_e.update(cx, |s, cx| s.close_tab(v, cx));
            }),
            on_close_others: std::rc::Rc::new(move |v, _w, cx| {
                close_others_e.update(cx, |s, cx| s.close_other_tabs(v, cx));
            }),
            on_close_right: std::rc::Rc::new(move |v, _w, cx| {
                close_right_e.update(cx, |s, cx| s.close_tabs_to_right(v, cx));
            }),
            on_close_all: std::rc::Rc::new(move |_w, cx| {
                close_all_e.update(cx, |s, cx| s.close_all_tabs(cx));
            }),
        }
    }

    fn render_tab_bar(&mut self, cx: &mut Context<Self>) -> Option<AnyElement> {
        let has_active_cluster = self.active_cluster_id().is_some();
        if !self.tabs.is_bar_visible(has_active_cluster) {
            return None;
        }
        let theme = OrbitTheme::get(cx).clone();
        let total = self.tabs.open_tabs().len();

        let select_entity = cx.entity().clone();
        let close_entity = cx.entity().clone();
        let menu_entity = cx.entity().clone();
        let drag_start_entity = cx.entity().clone();
        let drag_move_entity = cx.entity().clone();
        let drag_leave_entity = cx.entity().clone();
        let drop_entity = cx.entity().clone();
        let picker_entity = cx.entity().clone();

        let callbacks = TabBarCallbacks {
            on_select: std::rc::Rc::new(move |view, _window, cx| {
                select_entity.update(cx, |shell, cx| {
                    shell.activate_tab(view, cx);
                });
            }),
            on_close: std::rc::Rc::new(move |view, _window, cx| {
                close_entity.update(cx, |shell, cx| {
                    shell.close_tab(view, cx);
                });
            }),
            on_context_menu: std::rc::Rc::new(move |view, index, position, _window, cx| {
                menu_entity.update(cx, |shell, cx| {
                    shell.dismiss_overlays();
                    let entries = build_tab_context_menu_entries(
                        view,
                        index,
                        total,
                        &Self::build_tab_context_menu_callbacks(cx),
                    );
                    shell.open_context_menu(position, entries, cx);
                });
            }),
            on_drag_start: std::rc::Rc::new(move |idx, _window, cx| {
                drag_start_entity.update(cx, |shell, cx| {
                    shell.tabs.dragging_index = Some(idx);
                    cx.notify();
                });
            }),
            on_drag_move: std::rc::Rc::new(move |idx, pos, _window, cx| {
                drag_move_entity.update(cx, |shell, cx| {
                    if shell.tabs.update_drop_target(idx, pos) {
                        cx.notify();
                    }
                });
            }),
            on_drag_leave: std::rc::Rc::new(move |_window, cx| {
                drag_leave_entity.update(cx, |shell, cx| {
                    if shell.tabs.clear_drop_target() {
                        cx.notify();
                    }
                });
            }),
            on_drop: std::rc::Rc::new(move |from_idx, hover_idx, pos, _window, cx| {
                drop_entity.update(cx, |shell, cx| {
                    shell.reorder_tab_drop(from_idx, hover_idx, pos, cx);
                });
            }),
            on_toggle_picker: std::rc::Rc::new(move |window, cx| {
                picker_entity.update(cx, |shell, cx| {
                    shell.open_or_toggle_picker(true, window, cx);
                });
            }),
        };

        Some(render_workspace_tab_bar(&mut self.tabs, &theme, callbacks))
    }

    fn is_settings_dropdown_visible(&self) -> bool {
        self.sidebar.active_view == WorkspaceView::Settings
            && !matches!(self.tabs.active_tab(), Some(WorkspaceTab::Logs(_)))
            && self.settings_dropdown.is_any_open()
    }

    fn render_workspace(&mut self, window_width: f32, cx: &mut Context<Self>) -> AnyElement {
        let theme = OrbitTheme::get(cx).clone();

        let body = if self.sidebar.active_view == WorkspaceView::Settings
            && !matches!(self.tabs.active_tab(), Some(WorkspaceTab::Logs(_)))
        {
            self.render_settings_workspace(&theme, cx)
        } else {
            match &self.state.connection_state {
                ClusterConnectionState::Disconnected => {
                    let select_entity = cx.entity().clone();
                    let on_select =
                        std::rc::Rc::new(move |cid: String, _window: &mut Window, cx: &mut App| {
                            select_entity.update(cx, |shell, cx| {
                                shell.select_cluster(&cid, cx);
                            });
                        });
                    let refresh_entity = cx.entity().clone();
                    let on_refresh = move |_window: &mut Window, cx: &mut App| {
                        refresh_entity.update(cx, |shell, cx| {
                            shell.refresh_workspace(cx);
                        });
                    };
                    let add_entity = cx.entity().clone();
                    let on_add_cluster = move |window: &mut Window, cx: &mut App| {
                        add_entity.update(cx, |shell, cx| {
                            shell.open_add_cluster_modal(window, cx);
                        });
                    };
                    views::welcome::render_welcome_view(
                        &self.state.clusters,
                        self.state.kubeconfig_error.as_deref(),
                        &theme,
                        on_select,
                        on_refresh,
                        on_add_cluster,
                    )
                }
                ClusterConnectionState::Connecting {
                    cluster_id,
                    display_name,
                } => Self::render_connecting_workspace(cluster_id, display_name, &theme),
                ClusterConnectionState::Connected {
                    cluster_id,
                    display_name,
                    k8s_version,
                } => {
                    if self.tabs.open_tabs().is_empty() {
                        let open_view_entity = cx.entity().clone();
                        let on_open_view = std::rc::Rc::new(
                            move |view: WorkspaceView, _window: &mut Window, cx: &mut App| {
                                open_view_entity.update(cx, |shell, cx| {
                                    shell.select_workspace_view(view, cx);
                                });
                            },
                        );
                        let open_picker_entity = cx.entity().clone();
                        let on_open_picker = move |window: &mut Window, cx: &mut App| {
                            open_picker_entity.update(cx, |shell, cx| {
                                shell.open_or_toggle_picker(false, window, cx);
                            });
                        };
                        views::welcome::render_empty_tabs_workspace(
                            display_name,
                            &theme,
                            on_open_view,
                            on_open_picker,
                        )
                    } else {
                        let cluster_id = cluster_id.clone();
                        let display_name = display_name.clone();
                        let k8s_version = k8s_version.clone();
                        self.render_connected_workspace(
                            &cluster_id,
                            &display_name,
                            k8s_version.as_deref(),
                            &theme,
                            cx,
                        )
                    }
                }
                ClusterConnectionState::Unreachable { .. }
                | ClusterConnectionState::Error { .. } => {
                    if let Some(model) =
                        views::offline::offline_view_model(&self.state.connection_state)
                    {
                        let retry_entity = cx.entity().clone();
                        let on_retry = move |_window: &mut Window, cx: &mut App| {
                            retry_entity.update(cx, |shell, cx| {
                                shell.reconnect_cluster(cx);
                            });
                        };
                        let switch_entity = cx.entity().clone();
                        let on_switch = move |_window: &mut Window, cx: &mut App| {
                            switch_entity.update(cx, |shell, cx| {
                                shell.state.dropdown_open = true;
                                cx.notify();
                            });
                        };
                        let disconnect_entity = cx.entity().clone();
                        let on_disconnect = move |_window: &mut Window, cx: &mut App| {
                            disconnect_entity.update(cx, |shell, cx| {
                                shell.disconnect_cluster(cx);
                            });
                        };
                        views::offline::render_offline_view(
                            &model,
                            &theme,
                            on_retry,
                            on_switch,
                            on_disconnect,
                        )
                    } else {
                        div().into_any_element()
                    }
                }
            }
        };

        let is_full_bleed = (is_full_bleed_table_view(self.sidebar.active_view)
            || matches!(self.tabs.active_tab(), Some(WorkspaceTab::Logs(_))))
            && matches!(
                self.state.connection_state,
                ClusterConnectionState::Connected { .. }
            )
            && !self.tabs.open_tabs().is_empty();

        let workspace_main = if is_full_bleed {
            div()
                .id("orbit-workspace-scroll")
                .flex_1()
                .min_h(px(0.0))
                .overflow_hidden()
                .bg(theme.surface.bg_app)
                .flex()
                .flex_col()
                .child(body)
        } else {
            div()
                .id("orbit-workspace-scroll")
                .flex_1()
                .overflow_y_scroll()
                .p_4()
                .bg(theme.surface.bg_app)
                .flex()
                .flex_col()
                .gap_4()
                .when(
                    should_render_workspace_view_header(
                        self.sidebar.active_view,
                        matches!(self.tabs.active_tab(), Some(WorkspaceTab::Logs(_))),
                    ),
                    |el| el.child(self.render_workspace_view_header(&theme)),
                )
                .child(body)
        };

        let drawer = self.render_active_drawer(window_width, &theme, cx);

        div()
            .relative()
            .flex_1()
            .min_h(px(0.0))
            .min_w(px(0.0))
            .flex()
            .overflow_hidden()
            .child(workspace_main)
            .when_some(drawer, |el, panel| el.child(panel))
            .into_any_element()
    }

    fn render_footer(&self, cx: &mut Context<Self>) -> AnyElement {
        let theme = OrbitTheme::get(cx).clone();
        let presentation = format_cluster_status(&self.state, &theme);
        let k8s_version = match &self.state.connection_state {
            ClusterConnectionState::Connected { k8s_version, .. } => k8s_version.clone(),
            _ => None,
        };

        div()
            .id("orbit-footer-status-bar")
            .h(px(Self::FOOTER_HEIGHT))
            .flex_shrink_0()
            .px_3()
            .border_t_1()
            .border_color(theme.border.default)
            .bg(theme.surface.bg_sidebar)
            .flex()
            .items_center()
            .justify_between()
            .child(div().flex().items_center().gap_3())
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_2()
                    .child(
                        div()
                            .w(px(7.0))
                            .h(px(7.0))
                            .rounded_full()
                            .bg(presentation.color),
                    )
                    .child(
                        apply_text_step(div(), theme.typography.size_2xs)
                            .font_weight(FontWeight::MEDIUM)
                            .text_color(theme.text.primary)
                            .child(presentation.pill_label),
                    )
                    .child(
                        apply_text_step(div(), theme.typography.size_2xs)
                            .text_color(theme.text.muted)
                            .child(presentation.status_label),
                    )
                    .when_some(k8s_version, |el, ver| {
                        el.child(
                            apply_text_step(div(), theme.typography.size_2xs)
                                .text_color(theme.text.muted)
                                .child(format!("• {ver}")),
                        )
                    }),
            )
            .into_any_element()
    }
}

impl Render for OrbitShell {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = OrbitTheme::get(cx).clone();
        let viewport = window.viewport_size();
        let dismiss_entity = cx.entity().clone();
        let hover_entity = cx.entity().clone();
        let activate_entity = cx.entity().clone();
        let context_menu_overlay = render_context_menu_overlay(
            &self.context_menu,
            viewport,
            &theme,
            move |_window, cx| {
                dismiss_entity.update(cx, |shell, cx| {
                    shell.dismiss_context_menu();
                    cx.notify();
                });
            },
            move |idx, _window, cx| {
                hover_entity.update(cx, |shell, cx| {
                    shell.context_menu.set_highlight(idx);
                    cx.notify();
                });
            },
            move |idx, window, cx| {
                let callback = activate_entity.update(cx, |shell, cx| {
                    let cb = shell.context_menu.activate_at(idx);
                    cx.notify();
                    cb
                });
                if let Some(cb) = callback {
                    cb(window, cx);
                }
            },
        );

        let is_window_active = window.is_window_active();
        let is_maximized = window.is_maximized();
        let decorations = window.window_decorations();
        let chrome = resolve_window_chrome_style(&theme, is_window_active, is_maximized);

        div()
            .id("orbit-shell-root")
            .track_focus(&self.focus_handle)
            .on_action(cx.listener(|this, _: &OpenCommandPalette, window, cx| {
                if !this.add_cluster.is_open
                    && this.port_forward_modal.is_none()
                    && !this.workload_action_modal.is_open()
                {
                    this.toggle_command_palette(window, cx);
                }
            }))
            .on_action(cx.listener(|this, _: &ToggleSidebar, _window, cx| {
                if !this.add_cluster.is_open
                    && this.port_forward_modal.is_none()
                    && !this.workload_action_modal.is_open()
                {
                    this.sidebar.toggle_collapse();
                    cx.notify();
                }
            }))
            .on_key_down(cx.listener(|this, event: &KeyDownEvent, window, cx| {
                if this.add_cluster.is_open {
                    let key = event.keystroke.key.to_ascii_lowercase();
                    let is_paste = (event.keystroke.modifiers.control
                        || event.keystroke.modifiers.platform)
                        && key == "v";
                    if is_paste {
                        this.paste_into_add_cluster(cx, false);
                        return;
                    }
                    if !(event.keystroke.modifiers.control
                        || event.keystroke.modifiers.alt
                        || event.keystroke.modifiers.platform)
                    {
                        match this
                            .add_cluster
                            .handle_key_input(&key, event.keystroke.key_char.as_deref())
                        {
                            AddClusterKeyResult::CloseModal => {
                                this.add_cluster.close();
                                cx.notify();
                                return;
                            }
                            AddClusterKeyResult::Submit => {
                                this.submit_add_cluster(cx);
                                return;
                            }
                            AddClusterKeyResult::Consumed => {
                                cx.notify();
                                return;
                            }
                            AddClusterKeyResult::Ignored => {}
                        }
                    }
                    return;
                }
                if this.port_forward_modal.is_some() {
                    if !(event.keystroke.modifiers.control
                        || event.keystroke.modifiers.alt
                        || event.keystroke.modifiers.platform)
                    {
                        let key = event.keystroke.key.to_ascii_lowercase();
                        let res = this
                            .port_forward_modal
                            .as_mut()
                            .map(|m| m.handle_key_input(&key, event.keystroke.key_char.as_deref()))
                            .unwrap_or(PortForwardModalKeyResult::Ignored);
                        match res {
                            PortForwardModalKeyResult::Cancel => {
                                this.port_forward_modal = None;
                                cx.notify();
                            }
                            PortForwardModalKeyResult::Submit => {
                                this.submit_port_forward_modal(cx);
                            }
                            PortForwardModalKeyResult::Handled => {
                                cx.notify();
                            }
                            PortForwardModalKeyResult::Ignored => {}
                        }
                    }
                    return;
                }
                if this.workload_action_modal.is_open() {
                    if !(event.keystroke.modifiers.control
                        || event.keystroke.modifiers.alt
                        || event.keystroke.modifiers.platform)
                    {
                        let key = event.keystroke.key.to_ascii_lowercase();
                        match this
                            .workload_action_modal
                            .handle_key_input(&key, event.keystroke.key_char.as_deref())
                        {
                            WorkloadActionModalKeyResult::Cancel => {
                                this.workload_action_modal.close();
                                cx.notify();
                            }
                            WorkloadActionModalKeyResult::Submit => {
                                this.submit_workload_action_modal(cx);
                            }
                            WorkloadActionModalKeyResult::Handled => {
                                cx.notify();
                            }
                            WorkloadActionModalKeyResult::Ignored => {}
                        }
                    }
                    return;
                }
                if this.command_palette.is_open {
                    if !(event.keystroke.modifiers.control
                        || event.keystroke.modifiers.alt
                        || event.keystroke.modifiers.platform)
                    {
                        let key = event.keystroke.key.to_ascii_lowercase();
                        let catalog = build_command_palette_items(
                            &this.state.clusters,
                            this.state.active_cluster_id.as_deref(),
                            &this.namespaces,
                        );
                        let filtered = this.command_palette.filter_items(&catalog);
                        match this.command_palette.handle_key_input(
                            &key,
                            event.keystroke.key_char.as_deref(),
                            filtered.len(),
                        ) {
                            crate::components::CommandPaletteKeyResult::Close => {
                                this.focus_handle.focus(window);
                                cx.notify();
                            }
                            crate::components::CommandPaletteKeyResult::Execute(idx) => {
                                if let Some(item) = filtered.get(idx) {
                                    this.execute_command_palette_action(
                                        item.action.clone(),
                                        window,
                                        cx,
                                    );
                                }
                            }
                            crate::components::CommandPaletteKeyResult::Handled => {
                                cx.notify();
                            }
                            crate::components::CommandPaletteKeyResult::Ignored => {}
                        }
                    }
                    return;
                }
                if this.context_menu.is_open {
                    let key = event.keystroke.key.to_ascii_lowercase();
                    let (action, callback) = this.context_menu.handle_key(&key);
                    if action != ContextMenuKeyAction::Ignored {
                        if let Some(cb) = callback {
                            cb(window, cx);
                        }
                        cx.notify();
                        return;
                    }
                }
                if this.is_settings_dropdown_visible() {
                    let key = event.keystroke.key.to_ascii_lowercase();
                    match this.settings_dropdown.handle_key(&key) {
                        views::settings::SettingsDropdownKeyAction::Navigated
                        | views::settings::SettingsDropdownKeyAction::Closed => {
                            cx.notify();
                            return;
                        }
                        views::settings::SettingsDropdownKeyAction::SelectThemeMode(mode) => {
                            this.select_theme_mode(mode, window, cx);
                            return;
                        }
                        views::settings::SettingsDropdownKeyAction::SelectUiStyle(style) => {
                            this.select_ui_style(style, window, cx);
                            return;
                        }
                        views::settings::SettingsDropdownKeyAction::Ignored => {}
                    }
                }
                if this.tabs.picker_open {
                    let has_shortcut_modifier = event.keystroke.modifiers.control
                        || event.keystroke.modifiers.alt
                        || event.keystroke.modifiers.platform;
                    match this.tabs.handle_picker_key(
                        &event.keystroke.key,
                        event.keystroke.key_char.as_deref(),
                        has_shortcut_modifier,
                    ) {
                        PickerKeyAction::Selected(view) => {
                            this.apply_focus_target(
                                focus_target_for_picker_transition(false),
                                window,
                            );
                            this.select_workspace_view(view, cx);
                        }
                        PickerKeyAction::Closed => {
                            this.apply_focus_target(
                                focus_target_for_picker_transition(false),
                                window,
                            );
                            cx.notify();
                        }
                        PickerKeyAction::Updated => {
                            cx.notify();
                        }
                        PickerKeyAction::Ignored => {}
                    }
                    return;
                }
                if this.sidebar.active_view == WorkspaceView::PortForward
                    && this.port_forwards.search_focused
                    && !(event.keystroke.modifiers.control
                        || event.keystroke.modifiers.alt
                        || event.keystroke.modifiers.platform)
                    && this.port_forwards.handle_search_key(
                        &event.keystroke.key.to_ascii_lowercase(),
                        event.keystroke.key_char.as_deref(),
                    )
                {
                    cx.notify();
                    return;
                }
                if this.handle_exemplar_table_key(event, cx) {
                    return;
                }
                if this.namespaces.dropdown_open
                    && this.namespaces.handle_search_key(
                        &event.keystroke.key,
                        event.keystroke.key_char.as_deref(),
                    )
                {
                    cx.notify();
                    return;
                }
                if event.keystroke.key.eq_ignore_ascii_case("escape")
                    && this.active_shell_drawer.close()
                {
                    cx.notify();
                    return;
                }
                if event.keystroke.key.eq_ignore_ascii_case("escape") && this.drawer.is_open {
                    this.dismiss_resource_drawer(cx);
                    return;
                }
                if event.keystroke.key.eq_ignore_ascii_case("escape")
                    && this.tabs.clear_drag_state()
                {
                    cx.notify();
                    return;
                }
                if let Some(WorkspaceTab::Logs(target)) = this.tabs.active_tab() {
                    let key = event.keystroke.key.to_ascii_lowercase();
                    let has_ctrl_or_cmd =
                        event.keystroke.modifiers.control || event.keystroke.modifiers.platform;
                    if has_ctrl_or_cmd && key == "w" {
                        this.close_tab(WorkspaceTab::Logs(target), cx);
                        return;
                    }
                    if has_ctrl_or_cmd && key == "f" {
                        if let Some(log_state) = this.log_tabs.get_mut(&target) {
                            log_state.search_focused = true;
                        }
                        this.log_search_focus.focus(window);
                        cx.notify();
                        return;
                    }
                    if has_ctrl_or_cmd && key == "c" {
                        this.copy_logs_for_tab(&target, cx);
                        return;
                    }
                    if let Some(log_state) = this.log_tabs.get_mut(&target)
                        && log_state.search_focused
                        && !has_ctrl_or_cmd
                        && !event.keystroke.modifiers.alt
                    {
                        match key.as_str() {
                            "escape" => {
                                log_state.search_focused = false;
                                this.focus_handle.focus(window);
                                cx.notify();
                                return;
                            }
                            "enter" => {
                                if event.keystroke.modifiers.shift {
                                    log_state.prev_match();
                                } else {
                                    log_state.next_match();
                                }
                                cx.notify();
                                return;
                            }
                            "backspace" => {
                                let mut q = log_state.search_query.clone();
                                q.pop();
                                log_state.set_search_query(q);
                                cx.notify();
                                return;
                            }
                            _ => {
                                if let Some(ch) = event.keystroke.key_char.as_deref()
                                    && !ch.chars().any(char::is_control)
                                {
                                    let mut q = log_state.search_query.clone();
                                    q.push_str(ch);
                                    log_state.set_search_query(q);
                                    cx.notify();
                                    return;
                                }
                            }
                        }
                    }
                }
                let prev_view = this.sidebar.active_view;
                let (tab_handled, tab_cmds) = handle_tab_shortcut_with_state(
                    &this.state,
                    &mut this.sidebar,
                    &mut this.workloads_overview,
                    &mut this.namespaces,
                    &mut this.deployments,
                    &mut this.statefulsets,
                    &mut this.daemonsets,
                    &mut this.replicasets,
                    &mut this.jobs,
                    &mut this.cronjobs,
                    &mut this.pods,
                    &mut this.services,
                    &mut this.ingresses,
                    &mut this.configmaps,
                    &mut this.secrets,
                    &mut this.persistent_volumes,
                    &mut this.persistent_volume_claims,
                    &mut this.network_policies,
                    &mut this.storage_overview,
                    &mut this.pod_security,
                    &mut this.resource_quotas,
                    &mut this.list_views,
                    &mut this.nodes,
                    &mut this.events,
                    &mut this.tabs,
                    &event.keystroke.key,
                    event.keystroke.modifiers.control,
                    event.keystroke.modifiers.shift,
                    event.keystroke.modifiers.platform,
                );
                if tab_handled {
                    if this.sidebar.active_view != prev_view {
                        this.drawer.reset_for_context_change();
                    }
                    for cmd in tab_cmds {
                        this.dispatch_command(cmd);
                    }
                    cx.notify();
                    return;
                }
                let text_input_focused = this.is_text_input_focused();
                if let Some(shortcut) = resolve_global_workspace_shortcut(
                    &event.keystroke.key,
                    event.keystroke.key_char.as_deref(),
                    event.keystroke.modifiers.control,
                    event.keystroke.modifiers.platform,
                    event.keystroke.modifiers.alt,
                    text_input_focused,
                ) {
                    match shortcut {
                        // Ctrl/Cmd+K and Ctrl/Cmd+B are dispatched via GPUI cx.bind_keys + .on_action
                        GlobalWorkspaceShortcut::ToggleCommandPalette
                        | GlobalWorkspaceShortcut::ToggleSidebar => {}
                        GlobalWorkspaceShortcut::FocusSearch => {
                            this.focus_active_view_search(window, cx);
                            return;
                        }
                        GlobalWorkspaceShortcut::RefreshWorkspace => {
                            if !this.is_active_view_loading() {
                                this.refresh_workspace(cx);
                            }
                            return;
                        }
                    }
                }
                if !(text_input_focused
                    || event.keystroke.modifiers.control
                    || event.keystroke.modifiers.alt
                    || event.keystroke.modifiers.platform)
                {
                    match event.keystroke.key.to_ascii_lowercase().as_str() {
                        "up" => {
                            this.move_active_table_selection(-1, cx);
                        }
                        "down" => {
                            this.move_active_table_selection(1, cx);
                        }
                        "enter" => {
                            this.open_selected_table_row_drawer(cx);
                        }
                        _ => {}
                    }
                }
            }))
            .on_mouse_down(
                MouseButton::Left,
                cx.listener(|this, _event, _window, cx| {
                    this.dismiss_resource_drawer(cx);
                }),
            )
            .on_mouse_move(cx.listener(|this, event: &MouseMoveEvent, window, cx| {
                if this.sidebar.is_resizing {
                    this.sidebar
                        .update_resize_from_window_x(f32::from(event.position.x));
                    cx.notify();
                }
                if this.drawer.is_resizing {
                    this.drawer.update_resize_from_window_x(
                        f32::from(window.viewport_size().width),
                        f32::from(event.position.x),
                    );
                    cx.notify();
                }
                if this.update_active_table_column_resize(f32::from(event.position.x)) {
                    cx.notify();
                }
            }))
            .on_mouse_up(
                MouseButton::Left,
                cx.listener(|this, _event, _window, cx| {
                    let mut changed = false;
                    for log_state in this.log_tabs.values_mut() {
                        if log_state.is_dragging_selection {
                            log_state.is_dragging_selection = false;
                            changed = true;
                        }
                    }
                    if this.sidebar.is_resizing {
                        this.sidebar.end_resize();
                        changed = true;
                    }
                    if this.drawer.is_resizing {
                        this.drawer.stop_resize();
                        changed = true;
                    }
                    if this.finish_active_table_column_resize() {
                        changed = true;
                    }
                    if this.tabs.clear_drag_state() {
                        changed = true;
                    }
                    if changed {
                        cx.notify();
                    }
                }),
            )
            .relative()
            .flex()
            .flex_col()
            .size_full()
            .font(theme.typography.ui_font(FontWeight::NORMAL))
            .bg(theme.surface.bg_app)
            .text_color(theme.text.primary)
            .when_some(chrome.outer_border_color, |el, c| {
                el.border_1().border_color(c)
            })
            .child(render_custom_titlebar(
                Self::WINDOW_TITLE,
                is_window_active,
                is_maximized,
                decorations,
                &theme,
                window,
            ))
            .child(
                div()
                    .flex()
                    .flex_1()
                    .min_h(px(0.0))
                    .overflow_hidden()
                    .child(self.render_activity_bar(cx))
                    .when_some(self.render_sidebar_panel(cx), |el, panel| el.child(panel))
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .flex_1()
                            .min_w(px(0.0))
                            .overflow_hidden()
                            .child(self.render_header(cx))
                            .when_some(self.render_error_banner(cx), |el, banner| el.child(banner))
                            .when_some(self.render_tab_bar(cx), |el, tab_bar| el.child(tab_bar))
                            .child(self.render_workspace(f32::from(viewport.width), cx)),
                    ),
            )
            .child(self.render_footer(cx))
            .when_some(
                self.render_shell_drawer_overlay(f32::from(viewport.width), cx),
                |el, overlay| el.child(overlay),
            )
            .when_some(self.render_dropdown_overlay(cx), |el, overlay| {
                el.child(overlay)
            })
            .when_some(self.render_namespace_dropdown_overlay(cx), |el, overlay| {
                el.child(overlay)
            })
            .when(self.is_settings_dropdown_visible(), |el| {
                el.child(
                    div()
                        .id("settings-dropdown-backdrop")
                        .absolute()
                        .inset_0()
                        .on_mouse_down(
                            MouseButton::Left,
                            cx.listener(|this, _event, _window, cx| {
                                cx.stop_propagation();
                                this.settings_dropdown.close();
                                cx.notify();
                            }),
                        )
                        .on_mouse_down(
                            MouseButton::Right,
                            cx.listener(|this, _event, _window, cx| {
                                cx.stop_propagation();
                                this.settings_dropdown.close();
                                cx.notify();
                            }),
                        ),
                )
            })
            .when_some(self.render_add_cluster_modal_overlay(cx), |el, overlay| {
                el.child(overlay)
            })
            .when_some(
                self.render_workload_action_modal_overlay(cx),
                |el, overlay| el.child(overlay),
            )
            .when_some(self.render_port_forward_modal_overlay(cx), |el, overlay| {
                el.child(overlay)
            })
            .when_some(self.render_action_toast_overlay_view(cx), |el, overlay| {
                el.child(overlay)
            })
            .when(self.tabs.picker_open, |el| {
                let select_entity = cx.entity().clone();
                let clear_entity = cx.entity().clone();
                let close_entity = cx.entity().clone();
                el.child(render_tab_resource_picker_overlay(
                    &self.tabs,
                    &theme,
                    &self.picker_search_focus,
                    self.picker_search_focus.is_focused(window),
                    px(Self::tab_picker_top_offset()),
                    TabPickerCallbacks {
                        on_select_view: std::rc::Rc::new(move |view, window, cx| {
                            select_entity.update(cx, |shell, cx| {
                                shell.tabs.close_picker();
                                shell.apply_focus_target(
                                    focus_target_for_picker_transition(false),
                                    window,
                                );
                                shell.select_workspace_view(view, cx);
                            });
                        }),
                        on_clear_query: std::rc::Rc::new(move |window, cx| {
                            clear_entity.update(cx, |shell, cx| {
                                shell.tabs.clear_picker_query();
                                shell.apply_focus_target(
                                    focus_target_for_picker_transition(true),
                                    window,
                                );
                                cx.notify();
                            });
                        }),
                        on_close: std::rc::Rc::new(move |window, cx| {
                            close_entity.update(cx, |shell, cx| {
                                shell.tabs.close_picker();
                                shell.apply_focus_target(
                                    focus_target_for_picker_transition(false),
                                    window,
                                );
                                cx.notify();
                            });
                        }),
                    },
                ))
            })
            .when_some(
                self.render_command_palette_overlay_view(window, cx),
                |el, overlay| el.child(overlay),
            )
            .when_some(context_menu_overlay, |el, overlay| el.child(overlay))
            .when(
                cfg!(target_os = "linux")
                    && matches!(decorations, gpui::Decorations::Client { .. })
                    && !is_maximized,
                |el| {
                    #[cfg(target_os = "linux")]
                    {
                        use gpui::ResizeEdge;
                        const EDGE: f32 = 5.0;
                        const CORNER: f32 = 10.0;
                        let resize_edge = |id: &'static str, edge: ResizeEdge| {
                            div().id(id).absolute().on_mouse_down(
                                MouseButton::Left,
                                move |_ev, window, _cx| {
                                    window.start_window_resize(edge);
                                },
                            )
                        };
                        el.child(
                            resize_edge("resize-top", ResizeEdge::Top)
                                .top_0()
                                .left(px(CORNER))
                                .right(px(CORNER))
                                .h(px(EDGE))
                                .cursor(CursorStyle::ResizeUpDown),
                        )
                        .child(
                            resize_edge("resize-bottom", ResizeEdge::Bottom)
                                .bottom_0()
                                .left(px(CORNER))
                                .right(px(CORNER))
                                .h(px(EDGE))
                                .cursor(CursorStyle::ResizeUpDown),
                        )
                        .child(
                            resize_edge("resize-left", ResizeEdge::Left)
                                .left_0()
                                .top(px(CORNER))
                                .bottom(px(CORNER))
                                .w(px(EDGE))
                                .cursor(CursorStyle::ResizeLeftRight),
                        )
                        .child(
                            resize_edge("resize-right", ResizeEdge::Right)
                                .right_0()
                                .top(px(CORNER))
                                .bottom(px(CORNER))
                                .w(px(EDGE))
                                .cursor(CursorStyle::ResizeLeftRight),
                        )
                        .child(
                            resize_edge("resize-top-left", ResizeEdge::TopLeft)
                                .top_0()
                                .left_0()
                                .w(px(CORNER))
                                .h(px(CORNER))
                                .cursor(CursorStyle::ResizeUpLeftDownRight),
                        )
                        .child(
                            resize_edge("resize-top-right", ResizeEdge::TopRight)
                                .top_0()
                                .right_0()
                                .w(px(CORNER))
                                .h(px(CORNER))
                                .cursor(CursorStyle::ResizeUpRightDownLeft),
                        )
                        .child(
                            resize_edge("resize-bottom-left", ResizeEdge::BottomLeft)
                                .bottom_0()
                                .left_0()
                                .w(px(CORNER))
                                .h(px(CORNER))
                                .cursor(CursorStyle::ResizeUpRightDownLeft),
                        )
                        .child(
                            resize_edge("resize-bottom-right", ResizeEdge::BottomRight)
                                .bottom_0()
                                .right_0()
                                .w(px(CORNER))
                                .h(px(CORNER))
                                .cursor(CursorStyle::ResizeUpLeftDownRight),
                        )
                    }
                    #[cfg(not(target_os = "linux"))]
                    {
                        el
                    }
                },
            )
    }
}

pub fn gpui_exit_code_for_window_open(open_failed: bool) -> std::process::ExitCode {
    if open_failed {
        std::process::ExitCode::FAILURE
    } else {
        std::process::ExitCode::SUCCESS
    }
}

fn main() -> std::process::ExitCode {
    let config = OrbitConfig::load();
    let open_failed = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
    let open_failed_for_run = std::sync::Arc::clone(&open_failed);

    Application::new().run(move |cx: &mut App| {
        if let Err(err) = crate::icons::register_lucide_font(cx) {
            tracing::warn!(error = %err, "Failed to register embedded Lucide icon font");
        }

        cx.bind_keys([
            gpui::KeyBinding::new("ctrl-k", OpenCommandPalette, None),
            gpui::KeyBinding::new("cmd-k", OpenCommandPalette, None),
            gpui::KeyBinding::new("ctrl-b", ToggleSidebar, None),
            gpui::KeyBinding::new("cmd-b", ToggleSidebar, None),
        ]);

        cx.on_window_closed(|cx| {
            if cx.windows().is_empty() {
                cx.quit();
            }
        })
        .detach();

        let initial_theme = OrbitTheme::from_config(&config, WindowAppearance::Dark);
        let bounds = OrbitShell::default_window_bounds(cx);
        let options = OrbitShell::window_options_for_theme(bounds, &initial_theme);
        let config_for_window = config.clone();

        if let Err(err) = cx.open_window(options, move |window, cx| {
            let resolved_theme = OrbitTheme::from_config(&config_for_window, window.appearance());
            apply_theme(resolved_theme, window, cx);
            cx.new(|cx| OrbitShell::new(&config_for_window, window, cx))
        }) {
            eprintln!("Failed to open Orbit GPUI window: {err}");
            open_failed_for_run.store(true, std::sync::atomic::Ordering::SeqCst);
            cx.quit();
        }
    });

    gpui_exit_code_for_window_open(open_failed.load(std::sync::atomic::Ordering::SeqCst))
}

pub const fn workspace_drawer_stacks_as_overlay() -> bool {
    true
}

fn table_container_width(viewport_width: f32, sidebar: &SidebarLayoutState) -> f32 {
    let sidebar_width = if sidebar.is_collapsed {
        ACTIVITY_BAR_WIDTH
    } else {
        ACTIVITY_BAR_WIDTH + sidebar.panel_width
    };
    (viewport_width - sidebar_width).max(320.0)
}

fn finish_table_column_resize(
    table_id: &'static str,
    table: &mut crate::components::DataTableState,
) -> Option<(&'static str, orbit_core::config::PersistedTableConfig)> {
    if table.end_column_resize() {
        Some((table_id, table.to_persisted_config()))
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::theme::{AccentColor, SemanticColorsPreference};
    use gpui::WindowBackgroundAppearance;

    #[test]
    fn test_apply_bridge_event_cluster_added_updates_clusters_and_handles_unreachable_outcome() {
        let mut state = ClusterSwitcherState::default();
        let mut sidebar = SidebarLayoutState::default();
        let mut overview = ClusterOverviewState::default();
        let mut workloads_overview = WorkloadsOverviewState::default();
        let mut namespaces = NamespaceFilterState::default();
        let mut namespaces_view = NamespacesViewState::default();
        let mut deployments = DeploymentsState::default();
        let mut statefulsets = StatefulSetsState::default();
        let mut daemonsets = DaemonSetsState::default();
        let mut replicasets = ReplicaSetsState::default();
        let mut jobs = JobsState::default();
        let mut cronjobs = CronJobsState::default();
        let mut pods = PodsState::default();
        let mut services = ServicesState::default();
        let mut ingresses = IngressesState::default();
        let mut configmaps = ConfigMapsState::default();
        let mut secrets = SecretsState::default();
        let mut persistent_volumes = PersistentVolumesState::default();
        let mut persistent_volume_claims = PersistentVolumeClaimsState::default();
        let mut network_policies = NetworkPoliciesState::default();
        let mut storage_overview = StorageOverviewState::default();
        let mut pod_security = PodSecurityState::default();
        let mut resource_quotas = ResourceQuotasState::default();
        let mut list_views = ClusterListViews::default();
        let mut nodes = NodesState::default();
        let mut events = EventsState::default();

        let cluster = ClusterInfo {
            id: "added-ctx".to_string(),
            name: "added-ctx".to_string(),
            status: "disconnected".to_string(),
            alias: None,
            tags: Vec::new(),
            color: None,
            description: None,
            server_url: Some("https://127.0.0.1:6443".to_string()),
        };

        let mut modal = AddClusterModalState::default();
        modal.open(&[]);
        modal.is_submitting = true;

        // 1. Healthy switch outcome
        let healthy_event = ClusterBridgeEvent::ClusterAdded {
            added_contexts: vec!["added-ctx".to_string()],
            clusters: vec![cluster.clone()],
            switch_outcome: Some(orbit_core::kubernetes::manager::ClusterSwitchOutcome {
                context_name: "added-ctx".to_string(),
                healthy: true,
                k8s_version: Some("v1.31.0".to_string()),
                error_message: None,
            }),
            profile: None,
        };
        apply_add_cluster_modal_event(&mut modal, &healthy_event);
        assert!(!modal.is_open);

        let cmds = apply_cluster_event_with_workspace(
            &mut state,
            &mut sidebar,
            &mut overview,
            &mut workloads_overview,
            &mut namespaces,
            &mut namespaces_view,
            &mut deployments,
            &mut statefulsets,
            &mut daemonsets,
            &mut replicasets,
            &mut jobs,
            &mut cronjobs,
            &mut pods,
            &mut services,
            &mut ingresses,
            &mut configmaps,
            &mut secrets,
            &mut persistent_volumes,
            &mut persistent_volume_claims,
            &mut network_policies,
            &mut storage_overview,
            &mut pod_security,
            &mut resource_quotas,
            &mut list_views,
            &mut nodes,
            &mut events,
            healthy_event,
        );
        assert_eq!(state.clusters.len(), 1);
        assert!(matches!(
            state.connection_state,
            ClusterConnectionState::Connected { .. }
        ));
        assert!(!cmds.is_empty());

        // 2. Unreachable switch outcome
        let unreachable_event = ClusterBridgeEvent::ClusterAdded {
            added_contexts: vec!["added-ctx".to_string()],
            clusters: vec![cluster],
            switch_outcome: Some(orbit_core::kubernetes::manager::ClusterSwitchOutcome {
                context_name: "added-ctx".to_string(),
                healthy: false,
                k8s_version: None,
                error_message: Some("connection refused".to_string()),
            }),
            profile: None,
        };
        apply_cluster_event_with_workspace(
            &mut state,
            &mut sidebar,
            &mut overview,
            &mut workloads_overview,
            &mut namespaces,
            &mut namespaces_view,
            &mut deployments,
            &mut statefulsets,
            &mut daemonsets,
            &mut replicasets,
            &mut jobs,
            &mut cronjobs,
            &mut pods,
            &mut services,
            &mut ingresses,
            &mut configmaps,
            &mut secrets,
            &mut persistent_volumes,
            &mut persistent_volume_claims,
            &mut network_policies,
            &mut storage_overview,
            &mut pod_security,
            &mut resource_quotas,
            &mut list_views,
            &mut nodes,
            &mut events,
            unreachable_event,
        );
        assert!(matches!(
            state.connection_state,
            ClusterConnectionState::Unreachable { .. }
        ));
    }

    #[test]
    fn test_apply_bridge_event_add_cluster_failed_preserves_modal_open_with_error() {
        let mut modal = AddClusterModalState::default();
        modal.open(&[]);
        modal.is_submitting = true;

        let failed_event = ClusterBridgeEvent::AddClusterFailed {
            message: "Context 'dup' already exists in Orbit".to_string(),
        };
        apply_add_cluster_modal_event(&mut modal, &failed_event);

        assert!(modal.is_open);
        assert!(!modal.is_submitting);
        assert_eq!(
            modal.submit_error.as_deref(),
            Some("Context 'dup' already exists in Orbit")
        );
    }

    #[test]
    fn test_gpui_exit_code_for_window_open_returns_failure_on_error() {
        assert_eq!(
            format!("{:?}", gpui_exit_code_for_window_open(false)),
            format!("{:?}", std::process::ExitCode::SUCCESS),
            "Normal window open must yield ExitCode::SUCCESS"
        );
        assert_eq!(
            format!("{:?}", gpui_exit_code_for_window_open(true)),
            format!("{:?}", std::process::ExitCode::FAILURE),
            "Window open failure must yield ExitCode::FAILURE"
        );
        assert_ne!(
            format!("{:?}", gpui_exit_code_for_window_open(false)),
            format!("{:?}", gpui_exit_code_for_window_open(true)),
            "Success and failure exit codes must differ"
        );
    }

    #[test]
    fn test_window_options_and_shell_defaults() {
        assert_eq!(OrbitShell::WINDOW_TITLE, "Orbit");
        const {
            assert!(OrbitShell::DEFAULT_WIDTH >= 800.0);
            assert!(OrbitShell::DEFAULT_HEIGHT >= 600.0);
        }

        let theme = OrbitTheme::resolve(
            ThemeMode::Dark,
            UiStyle::Flat,
            AccentColor::Blue,
            SemanticColorsPreference::default(),
            WindowAppearance::Dark,
        );
        let bounds = Bounds::new(
            gpui::point(px(0.0), px(0.0)),
            size(
                px(OrbitShell::DEFAULT_WIDTH),
                px(OrbitShell::DEFAULT_HEIGHT),
            ),
        );
        let opts = OrbitShell::window_options_for_theme(bounds, &theme);
        assert_eq!(opts.window_decorations, Some(WindowDecorations::Client));
        assert_eq!(
            opts.titlebar.as_ref().map(|t| t.appears_transparent),
            Some(true)
        );
        assert!(
            opts.titlebar
                .as_ref()
                .and_then(|t| t.traffic_light_position)
                .is_some()
        );
        assert!(opts.is_resizable);
        assert!(opts.is_movable);
        match opts.window_bounds {
            Some(WindowBounds::Windowed(b)) => {
                assert!(f32::from(b.size.width) >= 800.0);
                assert!(f32::from(b.size.height) >= 600.0);
            }
            _ => panic!("Expected WindowBounds::Windowed"),
        }
        let title = opts
            .titlebar
            .as_ref()
            .and_then(|t| t.title.as_ref())
            .map(|s| s.as_ref());
        assert_eq!(title, Some("Orbit"));
    }

    #[test]
    fn test_shell_overlay_top_offsets_account_for_titlebar() {
        assert_eq!(
            OrbitShell::header_dropdown_top_offset(),
            OrbitShell::TITLEBAR_HEIGHT + OrbitShell::HEADER_HEIGHT + 4.0
        );
        assert_eq!(OrbitShell::header_dropdown_top_offset(), 84.0);
        assert_eq!(
            OrbitShell::tab_picker_top_offset(),
            OrbitShell::TITLEBAR_HEIGHT
                + OrbitShell::HEADER_HEIGHT
                + crate::components::TAB_BAR_HEIGHT
                + 4.0
        );
        assert_eq!(OrbitShell::tab_picker_top_offset(), 116.0);
    }

    #[test]
    fn test_windows_pe_metadata_and_icon_resource_config() {
        let manifest_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
        let cargo_toml = std::fs::read_to_string(manifest_dir.join("Cargo.toml"))
            .expect("src-gpui/Cargo.toml must be readable");
        assert!(
            cargo_toml.contains("[[bin]]") && cargo_toml.contains("name = \"orbit\""),
            "Cargo.toml must declare [[bin]] name = \"orbit\""
        );
        assert!(
            cargo_toml.contains("tauri-winres"),
            "Cargo.toml must declare tauri-winres build-dependency"
        );

        let build_rs = std::fs::read_to_string(manifest_dir.join("build.rs"))
            .expect("src-gpui/build.rs must exist and be readable");
        assert!(
            build_rs.contains("../src-tauri/icons/icon.ico")
                && build_rs.contains("set_icon_with_id(")
                && build_rs.contains("\"1\""),
            "build.rs must embed icon.ico with resource ID \"1\""
        );
        assert!(
            build_rs.contains("\"FileDescription\", \"Orbit\""),
            "build.rs must set FileDescription to Orbit"
        );
        assert!(
            build_rs.contains("\"ProductName\", \"Orbit\""),
            "build.rs must set ProductName to Orbit"
        );
        assert!(
            build_rs.contains("\"OriginalFilename\", \"orbit.exe\""),
            "build.rs must set OriginalFilename to orbit.exe"
        );
        assert!(
            build_rs.contains("\"InternalName\", \"Orbit\""),
            "build.rs must set InternalName to Orbit"
        );
        assert!(
            build_rs.contains("compile_for(&[\"orbit\"])"),
            "build.rs must call compile_for(&[\"orbit\"])"
        );
    }

    #[test]
    fn test_status_formatting_uses_orbit_theme_and_window_background_tracks_style() {
        let theme_flat = OrbitTheme::resolve(
            ThemeMode::Dark,
            UiStyle::Flat,
            AccentColor::Blue,
            SemanticColorsPreference::default(),
            WindowAppearance::Dark,
        );
        let theme_glass = OrbitTheme::resolve(
            ThemeMode::Dark,
            UiStyle::Glass,
            AccentColor::Blue,
            SemanticColorsPreference::default(),
            WindowAppearance::Dark,
        );
        let theme_paper = OrbitTheme::resolve(
            ThemeMode::Light,
            UiStyle::Paper,
            AccentColor::Blue,
            SemanticColorsPreference::default(),
            WindowAppearance::Light,
        );

        let bounds = Bounds::new(
            gpui::point(px(0.0), px(0.0)),
            size(
                px(OrbitShell::DEFAULT_WIDTH),
                px(OrbitShell::DEFAULT_HEIGHT),
            ),
        );
        assert_eq!(
            OrbitShell::window_options_for_theme(bounds, &theme_flat).window_background,
            WindowBackgroundAppearance::Opaque
        );
        assert_eq!(
            OrbitShell::window_options_for_theme(bounds, &theme_glass).window_background,
            WindowBackgroundAppearance::Blurred
        );
        assert_eq!(
            OrbitShell::window_options_for_theme(bounds, &theme_paper).window_background,
            WindowBackgroundAppearance::Opaque
        );

        let mut state = ClusterSwitcherState::new();
        assert_eq!(
            format_cluster_status(&state, &theme_flat),
            StatusPresentation {
                status_label: "Disconnected",
                color: theme_flat.text.muted,
                pill_label: "Select Cluster".to_string(),
            }
        );

        state.connection_state = ClusterConnectionState::Connecting {
            cluster_id: "prod-ctx".to_string(),
            display_name: "Prod US".to_string(),
        };
        assert_eq!(
            format_cluster_status(&state, &theme_flat),
            StatusPresentation {
                status_label: "Connecting",
                color: theme_flat.semantic.warning.solid,
                pill_label: "Prod US".to_string(),
            }
        );

        state.connection_state = ClusterConnectionState::Connected {
            cluster_id: "prod-ctx".to_string(),
            display_name: "Prod US".to_string(),
            k8s_version: Some("v1.31.2".to_string()),
        };
        assert_eq!(
            format_cluster_status(&state, &theme_flat),
            StatusPresentation {
                status_label: "Connected",
                color: theme_flat.semantic.success.solid,
                pill_label: "Prod US".to_string(),
            }
        );

        state.connection_state = ClusterConnectionState::Unreachable {
            cluster_id: "prod-ctx".to_string(),
            display_name: "Prod US".to_string(),
            server_url: Some("http://127.0.0.1:1".to_string()),
            error_message: "connection refused".to_string(),
        };
        assert_eq!(
            format_cluster_status(&state, &theme_flat),
            StatusPresentation {
                status_label: "Unreachable",
                color: theme_flat.semantic.danger.solid,
                pill_label: "Prod US".to_string(),
            }
        );

        state.connection_state = ClusterConnectionState::Error {
            cluster_id: None,
            display_name: None,
            error_message: "No kubeconfig loaded".to_string(),
        };
        assert_eq!(
            format_cluster_status(&state, &theme_flat),
            StatusPresentation {
                status_label: "Error",
                color: theme_flat.semantic.danger.solid,
                pill_label: "Select Cluster".to_string(),
            }
        );
    }

    #[test]
    fn test_workspace_view_presentation_and_cluster_connect_sidebar_transition() {
        use crate::navigation::{SidebarCategory, SidebarLayoutState, WorkspaceView};
        use crate::state::ClusterBridgeEvent;
        use crate::theme::K8sResourceKind;
        use orbit_core::kubernetes::manager::ClusterSwitchOutcome;
        use orbit_core::kubernetes::models::UserProfileInfo;

        let theme = OrbitTheme::resolve(
            ThemeMode::Dark,
            UiStyle::Flat,
            AccentColor::Blue,
            SemanticColorsPreference::default(),
            WindowAppearance::Dark,
        );

        let mut sidebar = SidebarLayoutState::new();
        assert_eq!(sidebar.active_category, SidebarCategory::Clusters);

        sidebar.select_view(WorkspaceView::Overview);
        assert_eq!(
            sidebar.active_view.breadcrumb(),
            ("Overview & Core", "Overview")
        );
        assert_eq!(sidebar.active_view.resource_kind(), None);

        sidebar.toggle_category(SidebarCategory::Workloads);
        assert_eq!(sidebar.active_view, WorkspaceView::Deployments);
        assert_eq!(
            sidebar.active_view.breadcrumb(),
            ("Workloads", "Deployments")
        );
        assert_eq!(
            sidebar.active_view.resource_kind(),
            Some(K8sResourceKind::Deployment)
        );
        assert_eq!(
            sidebar
                .active_view
                .resource_kind()
                .map(|k| theme.k8s_color(k)),
            Some(theme.k8s.deployment)
        );

        sidebar.select_view(WorkspaceView::Services);
        assert_eq!(sidebar.active_view.breadcrumb(), ("Network", "Services"));
        assert_eq!(
            sidebar
                .active_view
                .resource_kind()
                .map(|k| theme.k8s_color(k)),
            Some(theme.k8s.service)
        );

        sidebar.select_view(WorkspaceView::Settings);
        assert_eq!(sidebar.active_view.breadcrumb(), ("Orbit", "Settings"));
        assert_eq!(sidebar.active_view.resource_kind(), None);

        // ClusterConnected transitions state -> Connected and sidebar Clusters -> Core
        let mut switcher = ClusterSwitcherState::new();
        let mut connect_sidebar = SidebarLayoutState::new();
        let event = ClusterBridgeEvent::ClusterConnected {
            outcome: ClusterSwitchOutcome {
                context_name: "prod-ctx".to_string(),
                healthy: true,
                k8s_version: Some("v1.31.2".to_string()),
                error_message: None,
            },
            clusters: vec![],
            profile: UserProfileInfo {
                active_context: Some("prod-ctx".to_string()),
                cluster_name: Some("Prod US".to_string()),
                server_url: Some("https://k8s.example.com".to_string()),
                user_name: Some("admin".to_string()),
                auth_type: "client-certificate".to_string(),
                kubeconfig_paths: vec![],
                k8s_version: Some("v1.31.2".to_string()),
            },
        };
        let mut throwaway_overview = ClusterOverviewState::default();
        let mut throwaway_namespaces = NamespaceFilterState::default();
        let mut throwaway_namespaces_view = NamespacesViewState::default();
        let mut throwaway_deployments = DeploymentsState::default();
        let mut throwaway_statefulsets = StatefulSetsState::default();
        let mut throwaway_daemonsets = DaemonSetsState::default();
        let mut throwaway_jobs = JobsState::default();
        let mut throwaway_cronjobs = CronJobsState::default();
        let mut throwaway_pods = PodsState::default();
        let mut throwaway_nodes = NodesState::default();
        apply_cluster_event_with_workspace(
            &mut switcher,
            &mut connect_sidebar,
            &mut throwaway_overview,
            &mut WorkloadsOverviewState::default(),
            &mut throwaway_namespaces,
            &mut throwaway_namespaces_view,
            &mut throwaway_deployments,
            &mut throwaway_statefulsets,
            &mut throwaway_daemonsets,
            &mut ReplicaSetsState::default(),
            &mut throwaway_jobs,
            &mut throwaway_cronjobs,
            &mut throwaway_pods,
            &mut ServicesState::default(),
            &mut IngressesState::default(),
            &mut ConfigMapsState::default(),
            &mut SecretsState::default(),
            &mut PersistentVolumesState::default(),
            &mut PersistentVolumeClaimsState::default(),
            &mut NetworkPoliciesState::default(),
            &mut StorageOverviewState::default(),
            &mut PodSecurityState::default(),
            &mut ResourceQuotasState::default(),
            &mut ClusterListViews::default(),
            &mut throwaway_nodes,
            &mut EventsState::default(),
            event,
        );

        assert!(matches!(
            switcher.connection_state,
            ClusterConnectionState::Connected { .. }
        ));
        assert_eq!(connect_sidebar.active_category, SidebarCategory::Core);
        assert_eq!(connect_sidebar.active_view, WorkspaceView::Overview);
    }

    #[test]
    fn test_navigation_icon_colors_resolve_resource_kinds_and_theme_foregrounds() {
        for (mode, appearance) in [
            (ThemeMode::Dark, WindowAppearance::Dark),
            (ThemeMode::Light, WindowAppearance::Light),
        ] {
            for accent in [AccentColor::Blue, AccentColor::Amber, AccentColor::Emerald] {
                let theme = OrbitTheme::resolve(
                    mode,
                    UiStyle::Flat,
                    accent,
                    SemanticColorsPreference::default(),
                    appearance,
                );
                for view in [
                    WorkspaceView::Deployments,
                    WorkspaceView::Pods,
                    WorkspaceView::Services,
                    WorkspaceView::Overview,
                    WorkspaceView::Logs,
                ] {
                    assert_eq!(nav_item_icon_color(view, false, &theme), theme.text.muted);
                    assert_eq!(nav_item_icon_color(view, true, &theme), theme.text.primary);
                }
            }
        }
    }

    #[test]
    fn test_header_omits_redundant_kind_badge_for_resource_views() {
        for view in [
            WorkspaceView::Deployments,
            WorkspaceView::Pods,
            WorkspaceView::StatefulSets,
            WorkspaceView::DaemonSets,
            WorkspaceView::Jobs,
            WorkspaceView::CronJobs,
            WorkspaceView::Services,
            WorkspaceView::Ingresses,
            WorkspaceView::ConfigMaps,
            WorkspaceView::Secrets,
            WorkspaceView::PersistentVolumes,
            WorkspaceView::PersistentVolumeClaims,
            WorkspaceView::NetworkPolicies,
            WorkspaceView::Nodes,
            WorkspaceView::Namespaces,
        ] {
            assert!(
                !should_show_header_kind_badge(view),
                "homogeneous resource view {view:?} should not show redundant header kind badge"
            );
        }
    }

    #[test]
    fn test_is_workspace_view_loading_tracks_active_view_status() {
        assert!(!is_workspace_view_loading(
            WorkspaceView::Deployments,
            WorkspaceLoadingFlags {
                overview: true,
                ..Default::default()
            },
        ));
        assert!(is_workspace_view_loading(
            WorkspaceView::Deployments,
            WorkspaceLoadingFlags {
                deployments: true,
                ..Default::default()
            },
        ));
        assert!(is_workspace_view_loading(
            WorkspaceView::Overview,
            WorkspaceLoadingFlags {
                overview: true,
                ..Default::default()
            },
        ));
        assert!(is_workspace_view_loading(
            WorkspaceView::Events,
            WorkspaceLoadingFlags {
                events: true,
                ..Default::default()
            },
        ));
        assert!(is_workspace_view_loading(
            WorkspaceView::PersistentVolumeClaims,
            WorkspaceLoadingFlags {
                persistent_volume_claims: true,
                ..Default::default()
            },
        ));
        assert!(is_workspace_view_loading(
            WorkspaceView::NetworkPolicies,
            WorkspaceLoadingFlags {
                network_policies: true,
                ..Default::default()
            },
        ));
        assert!(!is_workspace_view_loading(
            WorkspaceView::Settings,
            WorkspaceLoadingFlags {
                overview: true,
                workloads_overview: true,
                namespaces: true,
                deployments: true,
                statefulsets: true,
                daemonsets: true,
                replicasets: true,
                jobs: true,
                cronjobs: true,
                pods: true,
                services: true,
                ingresses: true,
                port_forwards: true,
                configmaps: true,
                secrets: true,
                persistent_volumes: true,
                persistent_volume_claims: true,
                storage_overview: true,
                network_policies: true,
                pod_security: true,
                resource_quotas: true,
                hpas: true,
                storage_classes: true,
                security_overview: true,
                admission_webhooks: true,
                nodes: true,
                events: true,
            },
        ));
    }

    #[test]
    fn test_header_and_status_icon_mapping() {
        assert_eq!(
            char::from(dropdown_chevron_icon(true)),
            char::from(Icon::ChevronUp)
        );
        assert_eq!(
            char::from(dropdown_chevron_icon(false)),
            char::from(Icon::ChevronDown)
        );
    }

    #[test]
    fn test_shell_chrome_removes_redundant_controls() {
        const {
            assert!(!ORBIT_SHELL_CHROME.show_activity_brand_badge);
            assert!(ORBIT_SHELL_CHROME.show_activity_settings_btn);
            assert!(!ORBIT_SHELL_CHROME.show_header_settings_btn);
            assert!(!ORBIT_SHELL_CHROME.show_footer_settings_btn);
            assert!(!ORBIT_SHELL_CHROME.show_footer_sidebar_width_debug);
            assert!(!ORBIT_SHELL_CHROME.show_header_breadcrumb);
            assert!(!ORBIT_SHELL_CHROME.show_footer_breadcrumb);
            assert!(!ORBIT_SHELL_CHROME.show_sidebar_panel_collapse_btn);
            assert!(!ORBIT_SHELL_CHROME.show_header_sidebar_toggle_btn);
        }

        let mut sidebar = SidebarLayoutState::new();
        assert!(!sidebar.is_collapsed);

        // Ctrl+B / Cmd+B path
        sidebar.toggle_collapse();
        assert!(sidebar.is_collapsed);
        sidebar.toggle_collapse();
        assert!(!sidebar.is_collapsed);

        // Activity Bar active category click path
        let active_category = sidebar.active_category;
        sidebar.toggle_category(active_category);
        assert!(sidebar.is_collapsed);
        sidebar.toggle_category(active_category);
        assert!(!sidebar.is_collapsed);
    }

    #[test]
    fn test_activity_bar_omits_brand_badge_and_keeps_settings() {
        const {
            assert!(!ORBIT_SHELL_CHROME.show_activity_brand_badge);
            assert!(ORBIT_SHELL_CHROME.show_activity_settings_btn);
        }

        assert!(!SidebarCategory::ALL.is_empty());

        let mut sidebar = SidebarLayoutState::new();
        let active_category = sidebar.active_category;
        sidebar.toggle_category(active_category);
        assert!(sidebar.is_collapsed);
        sidebar.toggle_category(active_category);
        assert!(!sidebar.is_collapsed);
    }

    #[test]
    fn test_footer_omits_sidebar_width_debug_and_preserves_cluster_status() {
        const {
            assert!(!ORBIT_SHELL_CHROME.show_footer_sidebar_width_debug);
        }

        let theme = OrbitTheme::from_config(&OrbitConfig::default(), WindowAppearance::Dark);
        let mut state = ClusterSwitcherState::new();
        let disconnected = format_cluster_status(&state, &theme);
        assert_eq!(disconnected.pill_label, "Select Cluster");
        assert_eq!(disconnected.status_label, "Disconnected");

        state.connection_state = ClusterConnectionState::Connected {
            cluster_id: "prod-ctx".to_string(),
            display_name: "Prod US".to_string(),
            k8s_version: Some("v1.31.2".to_string()),
        };
        let connected = format_cluster_status(&state, &theme);
        assert_eq!(connected.pill_label, "Prod US");
        assert_eq!(connected.status_label, "Connected");

        let mut sidebar = SidebarLayoutState::new();
        sidebar.toggle_collapse();
        assert!(sidebar.is_collapsed);
        sidebar.toggle_collapse();
        assert!(!sidebar.is_collapsed);

        sidebar.begin_resize();
        sidebar.update_resize_from_window_x(ACTIVITY_BAR_WIDTH + 300.0);
        sidebar.end_resize();
        assert!(!sidebar.is_collapsed);
        assert_eq!(sidebar.panel_width, 300.0);
    }

    #[test]
    fn test_apply_cluster_event_dispatches_overview_and_namespaces_on_connect_and_resets_on_unreachable()
     {
        use orbit_core::kubernetes::manager::ClusterSwitchOutcome;
        use orbit_core::kubernetes::models::{NamespaceInfo, UserProfileInfo};
        use orbit_core::kubernetes::overview::{ClusterOverallHealth, ClusterOverviewSummary};

        let mut switcher = ClusterSwitcherState::new();
        let mut sidebar = SidebarLayoutState::new();
        let mut overview = ClusterOverviewState::default();
        let mut config = OrbitConfig::default();
        config.update_selected_namespaces_for_cluster(
            "prod-ctx",
            vec!["default".to_string(), "stale-ns".to_string()],
        );
        let mut namespaces = NamespaceFilterState::from_config(&config);
        let mut deployments = DeploymentsState::default();
        let mut statefulsets = StatefulSetsState::default();
        let mut daemonsets = DaemonSetsState::default();
        let mut jobs = JobsState::default();
        let mut cronjobs = CronJobsState::default();
        let mut pods = PodsState::default();
        let mut nodes = NodesState::default();

        let connect_event = ClusterBridgeEvent::ClusterConnected {
            outcome: ClusterSwitchOutcome {
                context_name: "prod-ctx".to_string(),
                healthy: true,
                k8s_version: Some("v1.31.1".to_string()),
                error_message: None,
            },
            clusters: vec![],
            profile: UserProfileInfo {
                active_context: Some("prod-ctx".to_string()),
                user_name: None,
                auth_type: "None".to_string(),
                cluster_name: Some("prod-cluster".to_string()),
                server_url: Some("https://10.0.0.10:6443".to_string()),
                kubeconfig_paths: vec![],
                k8s_version: Some("v1.31.1".to_string()),
            },
        };

        let cmds = apply_cluster_event_with_workspace(
            &mut switcher,
            &mut sidebar,
            &mut overview,
            &mut WorkloadsOverviewState::default(),
            &mut namespaces,
            &mut NamespacesViewState::default(),
            &mut deployments,
            &mut statefulsets,
            &mut daemonsets,
            &mut ReplicaSetsState::default(),
            &mut jobs,
            &mut cronjobs,
            &mut pods,
            &mut ServicesState::default(),
            &mut IngressesState::default(),
            &mut ConfigMapsState::default(),
            &mut SecretsState::default(),
            &mut PersistentVolumesState::default(),
            &mut PersistentVolumeClaimsState::default(),
            &mut NetworkPoliciesState::default(),
            &mut StorageOverviewState::default(),
            &mut PodSecurityState::default(),
            &mut ResourceQuotasState::default(),
            &mut ClusterListViews::default(),
            &mut nodes,
            &mut EventsState::default(),
            connect_event,
        );
        assert_eq!(
            cmds,
            vec![
                ClusterBridgeCommand::LoadOverview {
                    cluster_id: "prod-ctx".to_string(),
                    generation: 1,
                },
                ClusterBridgeCommand::LoadNamespaces {
                    cluster_id: "prod-ctx".to_string(),
                    generation: 1,
                },
            ]
        );
        assert_eq!(overview.status, crate::state::OverviewLoadStatus::Loading);
        assert_eq!(namespaces.status, NamespaceLoadStatus::Loading);

        let summary = ClusterOverviewSummary {
            k8s_version: Some("v1.31.1".to_string()),
            overall_health: ClusterOverallHealth::Healthy,
            ..Default::default()
        };
        let loaded_overview_cmds = apply_cluster_event_with_workspace(
            &mut switcher,
            &mut sidebar,
            &mut overview,
            &mut WorkloadsOverviewState::default(),
            &mut namespaces,
            &mut NamespacesViewState::default(),
            &mut deployments,
            &mut statefulsets,
            &mut daemonsets,
            &mut ReplicaSetsState::default(),
            &mut jobs,
            &mut cronjobs,
            &mut pods,
            &mut ServicesState::default(),
            &mut IngressesState::default(),
            &mut ConfigMapsState::default(),
            &mut SecretsState::default(),
            &mut PersistentVolumesState::default(),
            &mut PersistentVolumeClaimsState::default(),
            &mut NetworkPoliciesState::default(),
            &mut StorageOverviewState::default(),
            &mut PodSecurityState::default(),
            &mut ResourceQuotasState::default(),
            &mut ClusterListViews::default(),
            &mut nodes,
            &mut EventsState::default(),
            ClusterBridgeEvent::OverviewLoaded {
                cluster_id: "prod-ctx".to_string(),
                generation: 1,
                summary: summary.clone(),
            },
        );
        assert!(loaded_overview_cmds.is_empty());
        assert_eq!(overview.summary, Some(summary));

        let loaded_ns_cmds = apply_cluster_event_with_workspace(
            &mut switcher,
            &mut sidebar,
            &mut overview,
            &mut WorkloadsOverviewState::default(),
            &mut namespaces,
            &mut NamespacesViewState::default(),
            &mut deployments,
            &mut statefulsets,
            &mut daemonsets,
            &mut ReplicaSetsState::default(),
            &mut jobs,
            &mut cronjobs,
            &mut pods,
            &mut ServicesState::default(),
            &mut IngressesState::default(),
            &mut ConfigMapsState::default(),
            &mut SecretsState::default(),
            &mut PersistentVolumesState::default(),
            &mut PersistentVolumeClaimsState::default(),
            &mut NetworkPoliciesState::default(),
            &mut StorageOverviewState::default(),
            &mut PodSecurityState::default(),
            &mut ResourceQuotasState::default(),
            &mut ClusterListViews::default(),
            &mut nodes,
            &mut EventsState::default(),
            ClusterBridgeEvent::NamespacesLoaded {
                cluster_id: "prod-ctx".to_string(),
                generation: 1,
                namespaces: vec![NamespaceInfo {
                    name: "default".to_string(),
                    status: "Active".to_string(),
                    is_system: true,
                    age: "1d".to_string(),
                    labels: Default::default(),
                    annotations: Default::default(),
                    uid: "uid-default".to_string(),
                    created: "2026-01-01T00:00:00Z".to_string(),
                }],
            },
        );
        assert_eq!(
            loaded_ns_cmds,
            vec![ClusterBridgeCommand::SaveNamespaceSelection {
                cluster_id: "prod-ctx".to_string(),
                namespaces: vec!["default".to_string()],
            }]
        );
        assert_eq!(namespaces.namespaces.len(), 1);

        let unreachable_cmds = apply_cluster_event_with_workspace(
            &mut switcher,
            &mut sidebar,
            &mut overview,
            &mut WorkloadsOverviewState::default(),
            &mut namespaces,
            &mut NamespacesViewState::default(),
            &mut deployments,
            &mut statefulsets,
            &mut daemonsets,
            &mut ReplicaSetsState::default(),
            &mut jobs,
            &mut cronjobs,
            &mut pods,
            &mut ServicesState::default(),
            &mut IngressesState::default(),
            &mut ConfigMapsState::default(),
            &mut SecretsState::default(),
            &mut PersistentVolumesState::default(),
            &mut PersistentVolumeClaimsState::default(),
            &mut NetworkPoliciesState::default(),
            &mut StorageOverviewState::default(),
            &mut PodSecurityState::default(),
            &mut ResourceQuotasState::default(),
            &mut ClusterListViews::default(),
            &mut nodes,
            &mut EventsState::default(),
            ClusterBridgeEvent::ClusterUnreachable {
                outcome: ClusterSwitchOutcome {
                    context_name: "prod-ctx".to_string(),
                    healthy: false,
                    k8s_version: None,
                    error_message: Some("timeout".to_string()),
                },
                clusters: vec![],
                profile: UserProfileInfo {
                    active_context: Some("prod-ctx".to_string()),
                    user_name: None,
                    auth_type: "None".to_string(),
                    cluster_name: Some("prod-cluster".to_string()),
                    server_url: Some("https://10.0.0.10:6443".to_string()),
                    kubeconfig_paths: vec![],
                    k8s_version: None,
                },
            },
        );
        assert!(unreachable_cmds.is_empty());
        assert!(overview.summary.is_none());
        assert_eq!(overview.status, crate::state::OverviewLoadStatus::Idle);
        assert!(namespaces.namespaces.is_empty());
        assert_eq!(namespaces.status, NamespaceLoadStatus::Idle);
        assert_eq!(
            namespaces.selected_by_cluster.get("prod-ctx"),
            Some(&vec!["default".to_string()])
        );
    }

    #[test]
    fn test_namespace_trigger_presentation_covers_loading_error_and_selection_counts() {
        use orbit_core::kubernetes::models::NamespaceInfo;

        let theme = OrbitTheme::from_config(&OrbitConfig::default(), WindowAppearance::Dark);
        let mut ns_state = NamespaceFilterState::default();

        ns_state.begin_load_for_cluster("prod-ctx");
        let loading_pres = namespace_trigger_presentation(&ns_state, &theme);
        assert!(loading_pres.is_loading);
        assert!(!loading_pres.is_error);
        assert_eq!(loading_pres.label, "All Namespaces");
        assert_eq!(loading_pres.indicator_color, theme.semantic.warning.solid);

        ns_state.apply_failed("prod-ctx", 1, "forbidden".to_string());
        let error_pres = namespace_trigger_presentation(&ns_state, &theme);
        assert!(error_pres.is_error);
        assert!(!error_pres.is_loading);
        assert_eq!(error_pres.indicator_color, theme.semantic.danger.solid);

        ns_state.apply_loaded(
            "prod-ctx",
            1,
            ["default", "payments", "monitoring"]
                .into_iter()
                .map(|name| NamespaceInfo {
                    name: name.to_string(),
                    status: "Active".to_string(),
                    is_system: name != "payments",
                    age: "1d".to_string(),
                    labels: Default::default(),
                    annotations: Default::default(),
                    uid: format!("uid-{name}"),
                    created: "2026-01-01T00:00:00Z".to_string(),
                })
                .collect(),
        );

        let all_pres = namespace_trigger_presentation(&ns_state, &theme);
        assert_eq!(all_pres.label, "All Namespaces");
        assert!(!all_pres.is_filtered);
        assert_eq!(all_pres.indicator_color, theme.text.secondary);

        ns_state.toggle_namespace("default");
        let single_pres = namespace_trigger_presentation(&ns_state, &theme);
        assert_eq!(single_pres.label, "default");
        assert!(single_pres.is_filtered);
        assert_eq!(single_pres.indicator_color, theme.accent.accent);

        ns_state.toggle_namespace("payments");
        ns_state.toggle_namespace("monitoring");
        let multi_pres = namespace_trigger_presentation(&ns_state, &theme);
        assert_eq!(multi_pres.label, "default +2");
        assert!(multi_pres.is_filtered);
        assert_eq!(multi_pres.indicator_color, theme.accent.accent);
    }

    #[test]
    fn test_select_view_deployments_triggers_load_when_connected_and_idle() {
        let mut switcher = ClusterSwitcherState::new();
        switcher.connection_state = ClusterConnectionState::Connected {
            cluster_id: "prod-ctx".to_string(),
            display_name: "Prod".to_string(),
            k8s_version: Some("v1.31.1".to_string()),
        };
        let mut sidebar = SidebarLayoutState::new();
        let mut namespaces = NamespaceFilterState::default();
        let mut deployments = DeploymentsState::default();
        let mut statefulsets = StatefulSetsState::default();
        let mut daemonsets = DaemonSetsState::default();
        let mut jobs = JobsState::default();
        let mut cronjobs = CronJobsState::default();
        let mut pods = PodsState::default();
        let mut nodes = NodesState::default();

        // Navigating to Pods does not trigger Deployments load.
        assert!(matches!(
            select_workspace_view_with_state(
                &switcher,
                &mut sidebar,
                &mut WorkloadsOverviewState::default(),
                &mut namespaces,
                &mut deployments,
                &mut statefulsets,
                &mut daemonsets,
                &mut ReplicaSetsState::default(),
                &mut jobs,
                &mut cronjobs,
                &mut pods,
                &mut ServicesState::default(),
                &mut IngressesState::default(),
                &mut ConfigMapsState::default(),
                &mut SecretsState::default(),
                &mut PersistentVolumesState::default(),
                &mut PersistentVolumeClaimsState::default(),
                &mut NetworkPoliciesState::default(),
                &mut StorageOverviewState::default(),
                &mut PodSecurityState::default(),
                &mut ResourceQuotasState::default(),
                &mut ClusterListViews::default(),
                &mut nodes,
                &mut EventsState::default(),
                WorkspaceView::Pods
            ),
            Some(ClusterBridgeCommand::LoadPods { .. })
        ));
        assert_eq!(
            deployments.status,
            crate::state::DeploymentsLoadStatus::Idle
        );
        assert_eq!(sidebar.active_view, WorkspaceView::Pods);

        // Navigating to Deployments while connected & Idle triggers LoadDeployments once.
        let cmd = select_workspace_view_with_state(
            &switcher,
            &mut sidebar,
            &mut WorkloadsOverviewState::default(),
            &mut namespaces,
            &mut deployments,
            &mut statefulsets,
            &mut daemonsets,
            &mut ReplicaSetsState::default(),
            &mut jobs,
            &mut cronjobs,
            &mut pods,
            &mut ServicesState::default(),
            &mut IngressesState::default(),
            &mut ConfigMapsState::default(),
            &mut SecretsState::default(),
            &mut PersistentVolumesState::default(),
            &mut PersistentVolumeClaimsState::default(),
            &mut NetworkPoliciesState::default(),
            &mut StorageOverviewState::default(),
            &mut PodSecurityState::default(),
            &mut ResourceQuotasState::default(),
            &mut ClusterListViews::default(),
            &mut nodes,
            &mut EventsState::default(),
            WorkspaceView::Deployments,
        );
        assert_eq!(
            cmd,
            Some(ClusterBridgeCommand::LoadDeployments {
                cluster_id: "prod-ctx".to_string(),
                generation: 1,
            })
        );
        assert_eq!(sidebar.active_view, WorkspaceView::Deployments);
        assert_eq!(
            deployments.status,
            crate::state::DeploymentsLoadStatus::Loading
        );

        // Navigating away and back while already loading does not re-trigger.
        select_workspace_view_with_state(
            &switcher,
            &mut sidebar,
            &mut WorkloadsOverviewState::default(),
            &mut namespaces,
            &mut deployments,
            &mut statefulsets,
            &mut daemonsets,
            &mut ReplicaSetsState::default(),
            &mut jobs,
            &mut cronjobs,
            &mut pods,
            &mut ServicesState::default(),
            &mut IngressesState::default(),
            &mut ConfigMapsState::default(),
            &mut SecretsState::default(),
            &mut PersistentVolumesState::default(),
            &mut PersistentVolumeClaimsState::default(),
            &mut NetworkPoliciesState::default(),
            &mut StorageOverviewState::default(),
            &mut PodSecurityState::default(),
            &mut ResourceQuotasState::default(),
            &mut ClusterListViews::default(),
            &mut nodes,
            &mut EventsState::default(),
            WorkspaceView::Overview,
        );
        assert_eq!(
            select_workspace_view_with_state(
                &switcher,
                &mut sidebar,
                &mut WorkloadsOverviewState::default(),
                &mut namespaces,
                &mut deployments,
                &mut statefulsets,
                &mut daemonsets,
                &mut ReplicaSetsState::default(),
                &mut jobs,
                &mut cronjobs,
                &mut pods,
                &mut ServicesState::default(),
                &mut IngressesState::default(),
                &mut ConfigMapsState::default(),
                &mut SecretsState::default(),
                &mut PersistentVolumesState::default(),
                &mut PersistentVolumeClaimsState::default(),
                &mut NetworkPoliciesState::default(),
                &mut StorageOverviewState::default(),
                &mut PodSecurityState::default(),
                &mut ResourceQuotasState::default(),
                &mut ClusterListViews::default(),
                &mut nodes,
                &mut EventsState::default(),
                WorkspaceView::Deployments
            ),
            None
        );
    }

    #[test]
    fn test_apply_cluster_event_resets_deployments_on_connect_and_disconnect() {
        use orbit_core::kubernetes::manager::ClusterSwitchOutcome;
        use orbit_core::kubernetes::models::UserProfileInfo;

        let mut switcher = ClusterSwitcherState::new();
        let mut sidebar = SidebarLayoutState::new();
        let mut overview = ClusterOverviewState::default();
        let mut namespaces = NamespaceFilterState::default();
        let mut deployments = DeploymentsState::default();
        let mut statefulsets = StatefulSetsState::default();
        let mut daemonsets = DaemonSetsState::default();
        let mut jobs = JobsState::default();
        let mut cronjobs = CronJobsState::default();
        let mut pods = PodsState::default();
        let mut nodes = NodesState::default();

        // Connect while on Overview: DeploymentsState resets for cluster without eager load.
        let connect_event = ClusterBridgeEvent::ClusterConnected {
            outcome: ClusterSwitchOutcome {
                context_name: "prod-ctx".to_string(),
                healthy: true,
                k8s_version: Some("v1.31.1".to_string()),
                error_message: None,
            },
            clusters: vec![],
            profile: UserProfileInfo {
                active_context: Some("prod-ctx".to_string()),
                user_name: None,
                auth_type: "None".to_string(),
                cluster_name: Some("prod-cluster".to_string()),
                server_url: Some("https://10.0.0.10:6443".to_string()),
                kubeconfig_paths: vec![],
                k8s_version: Some("v1.31.1".to_string()),
            },
        };
        let cmds = apply_cluster_event_with_workspace(
            &mut switcher,
            &mut sidebar,
            &mut overview,
            &mut WorkloadsOverviewState::default(),
            &mut namespaces,
            &mut NamespacesViewState::default(),
            &mut deployments,
            &mut statefulsets,
            &mut daemonsets,
            &mut ReplicaSetsState::default(),
            &mut jobs,
            &mut cronjobs,
            &mut pods,
            &mut ServicesState::default(),
            &mut IngressesState::default(),
            &mut ConfigMapsState::default(),
            &mut SecretsState::default(),
            &mut PersistentVolumesState::default(),
            &mut PersistentVolumeClaimsState::default(),
            &mut NetworkPoliciesState::default(),
            &mut StorageOverviewState::default(),
            &mut PodSecurityState::default(),
            &mut ResourceQuotasState::default(),
            &mut ClusterListViews::default(),
            &mut nodes,
            &mut EventsState::default(),
            connect_event.clone(),
        );
        assert!(
            !cmds
                .iter()
                .any(|c| matches!(c, ClusterBridgeCommand::LoadDeployments { .. }))
        );
        assert_eq!(deployments.cluster_id.as_deref(), Some("prod-ctx"));
        assert_eq!(
            deployments.status,
            crate::state::DeploymentsLoadStatus::Idle
        );

        // Connect while already on Deployments: fires LoadDeployments eagerly.
        sidebar.select_view(WorkspaceView::Deployments);
        let cmds_on_deployments = apply_cluster_event_with_workspace(
            &mut switcher,
            &mut sidebar,
            &mut overview,
            &mut WorkloadsOverviewState::default(),
            &mut namespaces,
            &mut NamespacesViewState::default(),
            &mut deployments,
            &mut statefulsets,
            &mut daemonsets,
            &mut ReplicaSetsState::default(),
            &mut jobs,
            &mut cronjobs,
            &mut pods,
            &mut ServicesState::default(),
            &mut IngressesState::default(),
            &mut ConfigMapsState::default(),
            &mut SecretsState::default(),
            &mut PersistentVolumesState::default(),
            &mut PersistentVolumeClaimsState::default(),
            &mut NetworkPoliciesState::default(),
            &mut StorageOverviewState::default(),
            &mut PodSecurityState::default(),
            &mut ResourceQuotasState::default(),
            &mut ClusterListViews::default(),
            &mut nodes,
            &mut EventsState::default(),
            connect_event,
        );
        assert!(cmds_on_deployments.iter().any(|c| matches!(
            c,
            ClusterBridgeCommand::LoadDeployments { cluster_id, .. } if cluster_id == "prod-ctx"
        )));

        // Disconnect resets DeploymentsState.
        apply_cluster_event_with_workspace(
            &mut switcher,
            &mut sidebar,
            &mut overview,
            &mut WorkloadsOverviewState::default(),
            &mut namespaces,
            &mut NamespacesViewState::default(),
            &mut deployments,
            &mut statefulsets,
            &mut daemonsets,
            &mut ReplicaSetsState::default(),
            &mut jobs,
            &mut cronjobs,
            &mut pods,
            &mut ServicesState::default(),
            &mut IngressesState::default(),
            &mut ConfigMapsState::default(),
            &mut SecretsState::default(),
            &mut PersistentVolumesState::default(),
            &mut PersistentVolumeClaimsState::default(),
            &mut NetworkPoliciesState::default(),
            &mut StorageOverviewState::default(),
            &mut PodSecurityState::default(),
            &mut ResourceQuotasState::default(),
            &mut ClusterListViews::default(),
            &mut nodes,
            &mut EventsState::default(),
            ClusterBridgeEvent::ClusterUnreachable {
                outcome: ClusterSwitchOutcome {
                    context_name: "prod-ctx".to_string(),
                    healthy: false,
                    k8s_version: None,
                    error_message: Some("timeout".to_string()),
                },
                clusters: vec![],
                profile: UserProfileInfo {
                    active_context: Some("prod-ctx".to_string()),
                    user_name: None,
                    auth_type: "None".to_string(),
                    cluster_name: None,
                    server_url: None,
                    kubeconfig_paths: vec![],
                    k8s_version: None,
                },
            },
        );
        assert!(deployments.cluster_id.is_none());
        assert_eq!(
            deployments.status,
            crate::state::DeploymentsLoadStatus::Idle
        );
    }

    #[test]
    fn test_apply_cluster_event_handles_deployments_loaded_and_failed() {
        use orbit_core::kubernetes::models::{DeploymentInfo, Replicas};

        let mut switcher = ClusterSwitcherState::new();
        let mut sidebar = SidebarLayoutState::new();
        let mut overview = ClusterOverviewState::default();
        let mut namespaces = NamespaceFilterState::default();
        let mut deployments = DeploymentsState::default();
        let mut statefulsets = StatefulSetsState::default();
        let mut daemonsets = DaemonSetsState::default();
        let mut jobs = JobsState::default();
        let mut cronjobs = CronJobsState::default();
        let mut pods = PodsState::default();
        let mut nodes = NodesState::default();

        deployments.begin_load_for_cluster("prod-ctx");
        let current_gen = deployments.generation;

        let dep = DeploymentInfo {
            kind: "Deployment".to_string(),
            name: "api".to_string(),
            namespace: "default".to_string(),
            status: "Running".to_string(),
            conditions: None,
            replicas: Replicas {
                current: 2,
                desired: 2,
            },
            available: 2,
            up_to_date: 2,
            age: "1h".to_string(),
            images: vec!["api:v1".to_string()],
            containers: Vec::new(),
            strategy: Some("RollingUpdate".to_string()),
            min_ready_seconds: 0,
            revision_history: Some(10),
            labels: Default::default(),
            annotations: Default::default(),
            ports: vec![],
        };

        // Stale generation ignored.
        apply_cluster_event_with_workspace(
            &mut switcher,
            &mut sidebar,
            &mut overview,
            &mut WorkloadsOverviewState::default(),
            &mut namespaces,
            &mut NamespacesViewState::default(),
            &mut deployments,
            &mut statefulsets,
            &mut daemonsets,
            &mut ReplicaSetsState::default(),
            &mut jobs,
            &mut cronjobs,
            &mut pods,
            &mut ServicesState::default(),
            &mut IngressesState::default(),
            &mut ConfigMapsState::default(),
            &mut SecretsState::default(),
            &mut PersistentVolumesState::default(),
            &mut PersistentVolumeClaimsState::default(),
            &mut NetworkPoliciesState::default(),
            &mut StorageOverviewState::default(),
            &mut PodSecurityState::default(),
            &mut ResourceQuotasState::default(),
            &mut ClusterListViews::default(),
            &mut nodes,
            &mut EventsState::default(),
            ClusterBridgeEvent::DeploymentsLoaded {
                cluster_id: "prod-ctx".to_string(),
                generation: current_gen + 10,
                deployments: vec![dep.clone()],
            },
        );
        assert!(deployments.deployments.is_empty());

        // Matching generation updates state.
        apply_cluster_event_with_workspace(
            &mut switcher,
            &mut sidebar,
            &mut overview,
            &mut WorkloadsOverviewState::default(),
            &mut namespaces,
            &mut NamespacesViewState::default(),
            &mut deployments,
            &mut statefulsets,
            &mut daemonsets,
            &mut ReplicaSetsState::default(),
            &mut jobs,
            &mut cronjobs,
            &mut pods,
            &mut ServicesState::default(),
            &mut IngressesState::default(),
            &mut ConfigMapsState::default(),
            &mut SecretsState::default(),
            &mut PersistentVolumesState::default(),
            &mut PersistentVolumeClaimsState::default(),
            &mut NetworkPoliciesState::default(),
            &mut StorageOverviewState::default(),
            &mut PodSecurityState::default(),
            &mut ResourceQuotasState::default(),
            &mut ClusterListViews::default(),
            &mut nodes,
            &mut EventsState::default(),
            ClusterBridgeEvent::DeploymentsLoaded {
                cluster_id: "prod-ctx".to_string(),
                generation: current_gen,
                deployments: vec![dep],
            },
        );
        assert_eq!(
            deployments.status,
            crate::state::DeploymentsLoadStatus::Loaded
        );
        assert_eq!(deployments.deployments.len(), 1);

        // Refresh + failure updates error state.
        deployments.begin_refresh();
        let refresh_gen = deployments.generation;
        apply_cluster_event_with_workspace(
            &mut switcher,
            &mut sidebar,
            &mut overview,
            &mut WorkloadsOverviewState::default(),
            &mut namespaces,
            &mut NamespacesViewState::default(),
            &mut deployments,
            &mut statefulsets,
            &mut daemonsets,
            &mut ReplicaSetsState::default(),
            &mut jobs,
            &mut cronjobs,
            &mut pods,
            &mut ServicesState::default(),
            &mut IngressesState::default(),
            &mut ConfigMapsState::default(),
            &mut SecretsState::default(),
            &mut PersistentVolumesState::default(),
            &mut PersistentVolumeClaimsState::default(),
            &mut NetworkPoliciesState::default(),
            &mut StorageOverviewState::default(),
            &mut PodSecurityState::default(),
            &mut ResourceQuotasState::default(),
            &mut ClusterListViews::default(),
            &mut nodes,
            &mut EventsState::default(),
            ClusterBridgeEvent::DeploymentsLoadFailed {
                cluster_id: "prod-ctx".to_string(),
                generation: refresh_gen,
                message: "forbidden".to_string(),
            },
        );
        assert_eq!(
            deployments.status,
            crate::state::DeploymentsLoadStatus::Error
        );
        assert_eq!(deployments.error_message.as_deref(), Some("forbidden"));
    }

    #[test]
    fn test_select_view_nodes_triggers_load_when_connected_and_idle() {
        let mut switcher = ClusterSwitcherState::new();
        switcher.connection_state = ClusterConnectionState::Connected {
            cluster_id: "prod-ctx".to_string(),
            display_name: "Prod".to_string(),
            k8s_version: Some("v1.31.1".to_string()),
        };
        let mut sidebar = SidebarLayoutState::new();
        let mut namespaces = NamespaceFilterState::default();
        let mut deployments = DeploymentsState::default();
        let mut statefulsets = StatefulSetsState::default();
        let mut daemonsets = DaemonSetsState::default();
        let mut jobs = JobsState::default();
        let mut cronjobs = CronJobsState::default();
        let mut pods = PodsState::default();
        let mut nodes = NodesState::default();

        let cmd = select_workspace_view_with_state(
            &switcher,
            &mut sidebar,
            &mut WorkloadsOverviewState::default(),
            &mut namespaces,
            &mut deployments,
            &mut statefulsets,
            &mut daemonsets,
            &mut ReplicaSetsState::default(),
            &mut jobs,
            &mut cronjobs,
            &mut pods,
            &mut ServicesState::default(),
            &mut IngressesState::default(),
            &mut ConfigMapsState::default(),
            &mut SecretsState::default(),
            &mut PersistentVolumesState::default(),
            &mut PersistentVolumeClaimsState::default(),
            &mut NetworkPoliciesState::default(),
            &mut StorageOverviewState::default(),
            &mut PodSecurityState::default(),
            &mut ResourceQuotasState::default(),
            &mut ClusterListViews::default(),
            &mut nodes,
            &mut EventsState::default(),
            WorkspaceView::Nodes,
        );
        assert_eq!(
            cmd,
            Some(ClusterBridgeCommand::LoadNodes {
                cluster_id: "prod-ctx".to_string(),
                generation: 1,
            })
        );
        assert_eq!(sidebar.active_view, WorkspaceView::Nodes);
        assert_eq!(nodes.status, crate::state::NodesLoadStatus::Loading);

        // Navigating away and back while already loading/loaded does not re-trigger.
        select_workspace_view_with_state(
            &switcher,
            &mut sidebar,
            &mut WorkloadsOverviewState::default(),
            &mut namespaces,
            &mut deployments,
            &mut statefulsets,
            &mut daemonsets,
            &mut ReplicaSetsState::default(),
            &mut jobs,
            &mut cronjobs,
            &mut pods,
            &mut ServicesState::default(),
            &mut IngressesState::default(),
            &mut ConfigMapsState::default(),
            &mut SecretsState::default(),
            &mut PersistentVolumesState::default(),
            &mut PersistentVolumeClaimsState::default(),
            &mut NetworkPoliciesState::default(),
            &mut StorageOverviewState::default(),
            &mut PodSecurityState::default(),
            &mut ResourceQuotasState::default(),
            &mut ClusterListViews::default(),
            &mut nodes,
            &mut EventsState::default(),
            WorkspaceView::Overview,
        );
        assert_eq!(
            select_workspace_view_with_state(
                &switcher,
                &mut sidebar,
                &mut WorkloadsOverviewState::default(),
                &mut namespaces,
                &mut deployments,
                &mut statefulsets,
                &mut daemonsets,
                &mut ReplicaSetsState::default(),
                &mut jobs,
                &mut cronjobs,
                &mut pods,
                &mut ServicesState::default(),
                &mut IngressesState::default(),
                &mut ConfigMapsState::default(),
                &mut SecretsState::default(),
                &mut PersistentVolumesState::default(),
                &mut PersistentVolumeClaimsState::default(),
                &mut NetworkPoliciesState::default(),
                &mut StorageOverviewState::default(),
                &mut PodSecurityState::default(),
                &mut ResourceQuotasState::default(),
                &mut ClusterListViews::default(),
                &mut nodes,
                &mut EventsState::default(),
                WorkspaceView::Nodes
            ),
            None
        );
    }

    #[test]
    fn test_apply_cluster_event_resets_nodes_on_connect_and_disconnect() {
        use orbit_core::kubernetes::manager::ClusterSwitchOutcome;
        use orbit_core::kubernetes::models::UserProfileInfo;

        let mut switcher = ClusterSwitcherState::new();
        let mut sidebar = SidebarLayoutState::new();
        let mut overview = ClusterOverviewState::default();
        let mut namespaces = NamespaceFilterState::default();
        let mut deployments = DeploymentsState::default();
        let mut statefulsets = StatefulSetsState::default();
        let mut daemonsets = DaemonSetsState::default();
        let mut jobs = JobsState::default();
        let mut cronjobs = CronJobsState::default();
        let mut pods = PodsState::default();
        let mut nodes = NodesState::default();

        let connect_event = ClusterBridgeEvent::ClusterConnected {
            outcome: ClusterSwitchOutcome {
                context_name: "prod-ctx".to_string(),
                healthy: true,
                k8s_version: Some("v1.31.1".to_string()),
                error_message: None,
            },
            clusters: vec![],
            profile: UserProfileInfo {
                active_context: Some("prod-ctx".to_string()),
                user_name: None,
                auth_type: "None".to_string(),
                cluster_name: Some("prod-cluster".to_string()),
                server_url: Some("https://10.0.0.10:6443".to_string()),
                kubeconfig_paths: vec![],
                k8s_version: Some("v1.31.1".to_string()),
            },
        };

        // Connect while on Overview: NodesState resets without eager LoadNodes.
        let cmds = apply_cluster_event_with_workspace(
            &mut switcher,
            &mut sidebar,
            &mut overview,
            &mut WorkloadsOverviewState::default(),
            &mut namespaces,
            &mut NamespacesViewState::default(),
            &mut deployments,
            &mut statefulsets,
            &mut daemonsets,
            &mut ReplicaSetsState::default(),
            &mut jobs,
            &mut cronjobs,
            &mut pods,
            &mut ServicesState::default(),
            &mut IngressesState::default(),
            &mut ConfigMapsState::default(),
            &mut SecretsState::default(),
            &mut PersistentVolumesState::default(),
            &mut PersistentVolumeClaimsState::default(),
            &mut NetworkPoliciesState::default(),
            &mut StorageOverviewState::default(),
            &mut PodSecurityState::default(),
            &mut ResourceQuotasState::default(),
            &mut ClusterListViews::default(),
            &mut nodes,
            &mut EventsState::default(),
            connect_event.clone(),
        );
        assert!(
            !cmds
                .iter()
                .any(|c| matches!(c, ClusterBridgeCommand::LoadNodes { .. }))
        );
        assert_eq!(nodes.cluster_id.as_deref(), Some("prod-ctx"));
        assert_eq!(nodes.status, crate::state::NodesLoadStatus::Idle);

        // Connect while already on Nodes: fires LoadNodes eagerly.
        sidebar.select_view(WorkspaceView::Nodes);
        let cmds_on_nodes = apply_cluster_event_with_workspace(
            &mut switcher,
            &mut sidebar,
            &mut overview,
            &mut WorkloadsOverviewState::default(),
            &mut namespaces,
            &mut NamespacesViewState::default(),
            &mut deployments,
            &mut statefulsets,
            &mut daemonsets,
            &mut ReplicaSetsState::default(),
            &mut jobs,
            &mut cronjobs,
            &mut pods,
            &mut ServicesState::default(),
            &mut IngressesState::default(),
            &mut ConfigMapsState::default(),
            &mut SecretsState::default(),
            &mut PersistentVolumesState::default(),
            &mut PersistentVolumeClaimsState::default(),
            &mut NetworkPoliciesState::default(),
            &mut StorageOverviewState::default(),
            &mut PodSecurityState::default(),
            &mut ResourceQuotasState::default(),
            &mut ClusterListViews::default(),
            &mut nodes,
            &mut EventsState::default(),
            connect_event,
        );
        assert!(cmds_on_nodes.iter().any(|c| matches!(
            c,
            ClusterBridgeCommand::LoadNodes { cluster_id, .. } if cluster_id == "prod-ctx"
        )));

        // Disconnect resets NodesState.
        apply_cluster_event_with_workspace(
            &mut switcher,
            &mut sidebar,
            &mut overview,
            &mut WorkloadsOverviewState::default(),
            &mut namespaces,
            &mut NamespacesViewState::default(),
            &mut deployments,
            &mut statefulsets,
            &mut daemonsets,
            &mut ReplicaSetsState::default(),
            &mut jobs,
            &mut cronjobs,
            &mut pods,
            &mut ServicesState::default(),
            &mut IngressesState::default(),
            &mut ConfigMapsState::default(),
            &mut SecretsState::default(),
            &mut PersistentVolumesState::default(),
            &mut PersistentVolumeClaimsState::default(),
            &mut NetworkPoliciesState::default(),
            &mut StorageOverviewState::default(),
            &mut PodSecurityState::default(),
            &mut ResourceQuotasState::default(),
            &mut ClusterListViews::default(),
            &mut nodes,
            &mut EventsState::default(),
            ClusterBridgeEvent::ClusterUnreachable {
                outcome: ClusterSwitchOutcome {
                    context_name: "prod-ctx".to_string(),
                    healthy: false,
                    k8s_version: None,
                    error_message: Some("timeout".to_string()),
                },
                clusters: vec![],
                profile: UserProfileInfo {
                    active_context: Some("prod-ctx".to_string()),
                    user_name: None,
                    auth_type: "None".to_string(),
                    cluster_name: None,
                    server_url: None,
                    kubeconfig_paths: vec![],
                    k8s_version: None,
                },
            },
        );
        assert!(nodes.cluster_id.is_none());
        assert_eq!(nodes.status, crate::state::NodesLoadStatus::Idle);
    }

    #[test]
    fn test_apply_cluster_event_handles_nodes_loaded_and_failed() {
        use orbit_core::kubernetes::models::NodeInfo;

        let mut switcher = ClusterSwitcherState::new();
        let mut sidebar = SidebarLayoutState::new();
        let mut overview = ClusterOverviewState::default();
        let mut namespaces = NamespaceFilterState::default();
        let mut deployments = DeploymentsState::default();
        let mut statefulsets = StatefulSetsState::default();
        let mut daemonsets = DaemonSetsState::default();
        let mut jobs = JobsState::default();
        let mut cronjobs = CronJobsState::default();
        let mut pods = PodsState::default();
        let mut nodes = NodesState::default();

        nodes.begin_load_for_cluster("prod-ctx");
        let current_gen = nodes.generation;

        let node = NodeInfo {
            name: "node-1".to_string(),
            status: "Ready".to_string(),
            role: "control-plane".to_string(),
            version: "v1.31.1".to_string(),
            cpu_pct: 12.5,
            cpu_used: "1".to_string(),
            cpu_total: "8".to_string(),
            mem_pct: 25.0,
            mem_used: "8".to_string(),
            mem_total: "32".to_string(),
            pods_count: 15,
            pods_limit: 110,
            uptime: "10d".to_string(),
            created_at: None,
            labels: Vec::new(),
            labels_map: Default::default(),
            annotations: Default::default(),
            is_cordoned: false,
            node_info: None,
            addresses: Vec::new(),
            conditions: Vec::new(),
            taints: Vec::new(),
            capacity: None,
            allocatable: None,
            images_count: 0,
        };

        // Stale generation ignored.
        apply_cluster_event_with_workspace(
            &mut switcher,
            &mut sidebar,
            &mut overview,
            &mut WorkloadsOverviewState::default(),
            &mut namespaces,
            &mut NamespacesViewState::default(),
            &mut deployments,
            &mut statefulsets,
            &mut daemonsets,
            &mut ReplicaSetsState::default(),
            &mut jobs,
            &mut cronjobs,
            &mut pods,
            &mut ServicesState::default(),
            &mut IngressesState::default(),
            &mut ConfigMapsState::default(),
            &mut SecretsState::default(),
            &mut PersistentVolumesState::default(),
            &mut PersistentVolumeClaimsState::default(),
            &mut NetworkPoliciesState::default(),
            &mut StorageOverviewState::default(),
            &mut PodSecurityState::default(),
            &mut ResourceQuotasState::default(),
            &mut ClusterListViews::default(),
            &mut nodes,
            &mut EventsState::default(),
            ClusterBridgeEvent::NodesLoaded {
                cluster_id: "prod-ctx".to_string(),
                generation: current_gen + 10,
                nodes: vec![node.clone()],
            },
        );
        assert!(nodes.nodes.is_empty());

        // Matching generation updates state.
        apply_cluster_event_with_workspace(
            &mut switcher,
            &mut sidebar,
            &mut overview,
            &mut WorkloadsOverviewState::default(),
            &mut namespaces,
            &mut NamespacesViewState::default(),
            &mut deployments,
            &mut statefulsets,
            &mut daemonsets,
            &mut ReplicaSetsState::default(),
            &mut jobs,
            &mut cronjobs,
            &mut pods,
            &mut ServicesState::default(),
            &mut IngressesState::default(),
            &mut ConfigMapsState::default(),
            &mut SecretsState::default(),
            &mut PersistentVolumesState::default(),
            &mut PersistentVolumeClaimsState::default(),
            &mut NetworkPoliciesState::default(),
            &mut StorageOverviewState::default(),
            &mut PodSecurityState::default(),
            &mut ResourceQuotasState::default(),
            &mut ClusterListViews::default(),
            &mut nodes,
            &mut EventsState::default(),
            ClusterBridgeEvent::NodesLoaded {
                cluster_id: "prod-ctx".to_string(),
                generation: current_gen,
                nodes: vec![node],
            },
        );
        assert_eq!(nodes.status, crate::state::NodesLoadStatus::Loaded);
        assert_eq!(nodes.nodes.len(), 1);

        // Refresh + failure updates error state.
        nodes.begin_refresh();
        let refresh_gen = nodes.generation;
        apply_cluster_event_with_workspace(
            &mut switcher,
            &mut sidebar,
            &mut overview,
            &mut WorkloadsOverviewState::default(),
            &mut namespaces,
            &mut NamespacesViewState::default(),
            &mut deployments,
            &mut statefulsets,
            &mut daemonsets,
            &mut ReplicaSetsState::default(),
            &mut jobs,
            &mut cronjobs,
            &mut pods,
            &mut ServicesState::default(),
            &mut IngressesState::default(),
            &mut ConfigMapsState::default(),
            &mut SecretsState::default(),
            &mut PersistentVolumesState::default(),
            &mut PersistentVolumeClaimsState::default(),
            &mut NetworkPoliciesState::default(),
            &mut StorageOverviewState::default(),
            &mut PodSecurityState::default(),
            &mut ResourceQuotasState::default(),
            &mut ClusterListViews::default(),
            &mut nodes,
            &mut EventsState::default(),
            ClusterBridgeEvent::NodesLoadFailed {
                cluster_id: "prod-ctx".to_string(),
                generation: refresh_gen,
                message: "forbidden".to_string(),
            },
        );
        assert_eq!(nodes.status, crate::state::NodesLoadStatus::Error);
        assert_eq!(nodes.error_message.as_deref(), Some("forbidden"));
    }

    #[test]
    fn test_select_view_namespaces_triggers_load_only_when_needed() {
        let mut switcher = ClusterSwitcherState::new();
        switcher.connection_state = ClusterConnectionState::Connected {
            cluster_id: "prod-ctx".to_string(),
            display_name: "prod".to_string(),
            k8s_version: Some("v1.31.1".to_string()),
        };
        let mut sidebar = SidebarLayoutState::new();
        let mut namespaces = NamespaceFilterState::default();
        let mut deployments = DeploymentsState::default();
        let mut statefulsets = StatefulSetsState::default();
        let mut daemonsets = DaemonSetsState::default();
        let mut jobs = JobsState::default();
        let mut cronjobs = CronJobsState::default();
        let mut pods = PodsState::default();
        let mut nodes = NodesState::default();

        let cmd = select_workspace_view_with_state(
            &switcher,
            &mut sidebar,
            &mut WorkloadsOverviewState::default(),
            &mut namespaces,
            &mut deployments,
            &mut statefulsets,
            &mut daemonsets,
            &mut ReplicaSetsState::default(),
            &mut jobs,
            &mut cronjobs,
            &mut pods,
            &mut ServicesState::default(),
            &mut IngressesState::default(),
            &mut ConfigMapsState::default(),
            &mut SecretsState::default(),
            &mut PersistentVolumesState::default(),
            &mut PersistentVolumeClaimsState::default(),
            &mut NetworkPoliciesState::default(),
            &mut StorageOverviewState::default(),
            &mut PodSecurityState::default(),
            &mut ResourceQuotasState::default(),
            &mut ClusterListViews::default(),
            &mut nodes,
            &mut EventsState::default(),
            WorkspaceView::Namespaces,
        );
        assert_eq!(
            cmd,
            Some(ClusterBridgeCommand::LoadNamespaces {
                cluster_id: "prod-ctx".to_string(),
                generation: 1,
            })
        );
        assert_eq!(sidebar.active_view, WorkspaceView::Namespaces);

        // Navigating away and back while Loading returns None.
        select_workspace_view_with_state(
            &switcher,
            &mut sidebar,
            &mut WorkloadsOverviewState::default(),
            &mut namespaces,
            &mut deployments,
            &mut statefulsets,
            &mut daemonsets,
            &mut ReplicaSetsState::default(),
            &mut jobs,
            &mut cronjobs,
            &mut pods,
            &mut ServicesState::default(),
            &mut IngressesState::default(),
            &mut ConfigMapsState::default(),
            &mut SecretsState::default(),
            &mut PersistentVolumesState::default(),
            &mut PersistentVolumeClaimsState::default(),
            &mut NetworkPoliciesState::default(),
            &mut StorageOverviewState::default(),
            &mut PodSecurityState::default(),
            &mut ResourceQuotasState::default(),
            &mut ClusterListViews::default(),
            &mut nodes,
            &mut EventsState::default(),
            WorkspaceView::Overview,
        );
        assert!(
            select_workspace_view_with_state(
                &switcher,
                &mut sidebar,
                &mut WorkloadsOverviewState::default(),
                &mut namespaces,
                &mut deployments,
                &mut statefulsets,
                &mut daemonsets,
                &mut ReplicaSetsState::default(),
                &mut jobs,
                &mut cronjobs,
                &mut pods,
                &mut ServicesState::default(),
                &mut IngressesState::default(),
                &mut ConfigMapsState::default(),
                &mut SecretsState::default(),
                &mut PersistentVolumesState::default(),
                &mut PersistentVolumeClaimsState::default(),
                &mut NetworkPoliciesState::default(),
                &mut StorageOverviewState::default(),
                &mut PodSecurityState::default(),
                &mut ResourceQuotasState::default(),
                &mut ClusterListViews::default(),
                &mut nodes,
                &mut EventsState::default(),
                WorkspaceView::Namespaces,
            )
            .is_none()
        );

        // Loaded state also does not re-trigger LoadNamespaces on view switch.
        namespaces.apply_loaded("prod-ctx", 1, Vec::new());
        assert!(
            select_workspace_view_with_state(
                &switcher,
                &mut sidebar,
                &mut WorkloadsOverviewState::default(),
                &mut namespaces,
                &mut deployments,
                &mut statefulsets,
                &mut daemonsets,
                &mut ReplicaSetsState::default(),
                &mut jobs,
                &mut cronjobs,
                &mut pods,
                &mut ServicesState::default(),
                &mut IngressesState::default(),
                &mut ConfigMapsState::default(),
                &mut SecretsState::default(),
                &mut PersistentVolumesState::default(),
                &mut PersistentVolumeClaimsState::default(),
                &mut NetworkPoliciesState::default(),
                &mut StorageOverviewState::default(),
                &mut PodSecurityState::default(),
                &mut ResourceQuotasState::default(),
                &mut ClusterListViews::default(),
                &mut nodes,
                &mut EventsState::default(),
                WorkspaceView::Namespaces,
            )
            .is_none()
        );
    }

    #[test]
    fn test_apply_cluster_event_resets_namespaces_view_selection_on_connect_and_disconnect() {
        use orbit_core::kubernetes::manager::ClusterSwitchOutcome;
        use orbit_core::kubernetes::models::UserProfileInfo;

        let mut switcher = ClusterSwitcherState::new();
        let mut sidebar = SidebarLayoutState::new();
        let mut overview = ClusterOverviewState::default();
        let mut namespaces = NamespaceFilterState::default();
        let mut namespaces_view = NamespacesViewState::default();
        let mut deployments = DeploymentsState::default();
        let mut statefulsets = StatefulSetsState::default();
        let mut daemonsets = DaemonSetsState::default();
        let mut jobs = JobsState::default();
        let mut cronjobs = CronJobsState::default();
        let mut pods = PodsState::default();
        let mut nodes = NodesState::default();

        namespaces_view.table.selected_row_key = Some("default".into());
        apply_cluster_event_with_workspace(
            &mut switcher,
            &mut sidebar,
            &mut overview,
            &mut WorkloadsOverviewState::default(),
            &mut namespaces,
            &mut namespaces_view,
            &mut deployments,
            &mut statefulsets,
            &mut daemonsets,
            &mut ReplicaSetsState::default(),
            &mut jobs,
            &mut cronjobs,
            &mut pods,
            &mut ServicesState::default(),
            &mut IngressesState::default(),
            &mut ConfigMapsState::default(),
            &mut SecretsState::default(),
            &mut PersistentVolumesState::default(),
            &mut PersistentVolumeClaimsState::default(),
            &mut NetworkPoliciesState::default(),
            &mut StorageOverviewState::default(),
            &mut PodSecurityState::default(),
            &mut ResourceQuotasState::default(),
            &mut ClusterListViews::default(),
            &mut nodes,
            &mut EventsState::default(),
            ClusterBridgeEvent::ClusterConnected {
                outcome: ClusterSwitchOutcome {
                    context_name: "prod-ctx".to_string(),
                    healthy: true,
                    k8s_version: Some("v1.31.1".to_string()),
                    error_message: None,
                },
                clusters: vec![],
                profile: UserProfileInfo {
                    active_context: Some("prod-ctx".to_string()),
                    user_name: None,
                    auth_type: "None".to_string(),
                    cluster_name: Some("prod-cluster".to_string()),
                    server_url: Some("https://10.0.0.10:6443".to_string()),
                    kubeconfig_paths: vec![],
                    k8s_version: Some("v1.31.1".to_string()),
                },
            },
        );
        assert!(namespaces_view.table.selected_row_key.is_none());

        namespaces_view.table.selected_row_key = Some("kube-system".into());
        apply_cluster_event_with_workspace(
            &mut switcher,
            &mut sidebar,
            &mut overview,
            &mut WorkloadsOverviewState::default(),
            &mut namespaces,
            &mut namespaces_view,
            &mut deployments,
            &mut statefulsets,
            &mut daemonsets,
            &mut ReplicaSetsState::default(),
            &mut jobs,
            &mut cronjobs,
            &mut pods,
            &mut ServicesState::default(),
            &mut IngressesState::default(),
            &mut ConfigMapsState::default(),
            &mut SecretsState::default(),
            &mut PersistentVolumesState::default(),
            &mut PersistentVolumeClaimsState::default(),
            &mut NetworkPoliciesState::default(),
            &mut StorageOverviewState::default(),
            &mut PodSecurityState::default(),
            &mut ResourceQuotasState::default(),
            &mut ClusterListViews::default(),
            &mut nodes,
            &mut EventsState::default(),
            ClusterBridgeEvent::ClusterUnreachable {
                outcome: ClusterSwitchOutcome {
                    context_name: "prod-ctx".to_string(),
                    healthy: false,
                    k8s_version: None,
                    error_message: Some("timeout".to_string()),
                },
                clusters: vec![],
                profile: UserProfileInfo {
                    active_context: Some("prod-ctx".to_string()),
                    user_name: None,
                    auth_type: "None".to_string(),
                    cluster_name: Some("prod-cluster".to_string()),
                    server_url: Some("https://10.0.0.10:6443".to_string()),
                    kubeconfig_paths: vec![],
                    k8s_version: None,
                },
            },
        );
        assert!(namespaces_view.table.selected_row_key.is_none());
    }

    #[test]
    fn test_selecting_events_view_triggers_load_once_when_idle() {
        let mut switcher = ClusterSwitcherState::new();
        switcher.connection_state = ClusterConnectionState::Connected {
            cluster_id: "prod-ctx".to_string(),
            display_name: "prod".to_string(),
            k8s_version: Some("v1.31.1".to_string()),
        };
        let mut sidebar = SidebarLayoutState::new();
        let mut namespaces = NamespaceFilterState::default();
        let mut deployments = DeploymentsState::default();
        let mut statefulsets = StatefulSetsState::default();
        let mut daemonsets = DaemonSetsState::default();
        let mut jobs = JobsState::default();
        let mut cronjobs = CronJobsState::default();
        let mut pods = PodsState::default();
        let mut nodes = NodesState::default();
        let mut events = EventsState::default();

        let cmd = select_workspace_view_with_state(
            &switcher,
            &mut sidebar,
            &mut WorkloadsOverviewState::default(),
            &mut namespaces,
            &mut deployments,
            &mut statefulsets,
            &mut daemonsets,
            &mut ReplicaSetsState::default(),
            &mut jobs,
            &mut cronjobs,
            &mut pods,
            &mut ServicesState::default(),
            &mut IngressesState::default(),
            &mut ConfigMapsState::default(),
            &mut SecretsState::default(),
            &mut PersistentVolumesState::default(),
            &mut PersistentVolumeClaimsState::default(),
            &mut NetworkPoliciesState::default(),
            &mut StorageOverviewState::default(),
            &mut PodSecurityState::default(),
            &mut ResourceQuotasState::default(),
            &mut ClusterListViews::default(),
            &mut nodes,
            &mut events,
            WorkspaceView::Events,
        );
        assert_eq!(
            cmd,
            Some(ClusterBridgeCommand::LoadEvents {
                cluster_id: "prod-ctx".to_string(),
                generation: 1,
            })
        );
        assert_eq!(sidebar.active_view, WorkspaceView::Events);
        assert_eq!(events.status, crate::state::EventsLoadStatus::Loading);

        // Navigating away and back while Loading returns None.
        select_workspace_view_with_state(
            &switcher,
            &mut sidebar,
            &mut WorkloadsOverviewState::default(),
            &mut namespaces,
            &mut deployments,
            &mut statefulsets,
            &mut daemonsets,
            &mut ReplicaSetsState::default(),
            &mut jobs,
            &mut cronjobs,
            &mut pods,
            &mut ServicesState::default(),
            &mut IngressesState::default(),
            &mut ConfigMapsState::default(),
            &mut SecretsState::default(),
            &mut PersistentVolumesState::default(),
            &mut PersistentVolumeClaimsState::default(),
            &mut NetworkPoliciesState::default(),
            &mut StorageOverviewState::default(),
            &mut PodSecurityState::default(),
            &mut ResourceQuotasState::default(),
            &mut ClusterListViews::default(),
            &mut nodes,
            &mut events,
            WorkspaceView::Overview,
        );
        assert!(
            select_workspace_view_with_state(
                &switcher,
                &mut sidebar,
                &mut WorkloadsOverviewState::default(),
                &mut namespaces,
                &mut deployments,
                &mut statefulsets,
                &mut daemonsets,
                &mut ReplicaSetsState::default(),
                &mut jobs,
                &mut cronjobs,
                &mut pods,
                &mut ServicesState::default(),
                &mut IngressesState::default(),
                &mut ConfigMapsState::default(),
                &mut SecretsState::default(),
                &mut PersistentVolumesState::default(),
                &mut PersistentVolumeClaimsState::default(),
                &mut NetworkPoliciesState::default(),
                &mut StorageOverviewState::default(),
                &mut PodSecurityState::default(),
                &mut ResourceQuotasState::default(),
                &mut ClusterListViews::default(),
                &mut nodes,
                &mut events,
                WorkspaceView::Events,
            )
            .is_none()
        );

        // Loaded state also does not re-trigger LoadEvents on view switch.
        events.apply_loaded("prod-ctx", 1, Vec::new());
        assert!(
            select_workspace_view_with_state(
                &switcher,
                &mut sidebar,
                &mut WorkloadsOverviewState::default(),
                &mut namespaces,
                &mut deployments,
                &mut statefulsets,
                &mut daemonsets,
                &mut ReplicaSetsState::default(),
                &mut jobs,
                &mut cronjobs,
                &mut pods,
                &mut ServicesState::default(),
                &mut IngressesState::default(),
                &mut ConfigMapsState::default(),
                &mut SecretsState::default(),
                &mut PersistentVolumesState::default(),
                &mut PersistentVolumeClaimsState::default(),
                &mut NetworkPoliciesState::default(),
                &mut StorageOverviewState::default(),
                &mut PodSecurityState::default(),
                &mut ResourceQuotasState::default(),
                &mut ClusterListViews::default(),
                &mut nodes,
                &mut events,
                WorkspaceView::Events,
            )
            .is_none()
        );
    }

    #[test]
    fn test_cluster_connected_on_events_view_triggers_load_events_and_disconnect_resets() {
        use orbit_core::kubernetes::manager::ClusterSwitchOutcome;
        use orbit_core::kubernetes::models::UserProfileInfo;

        let mut switcher = ClusterSwitcherState::new();
        let mut sidebar = SidebarLayoutState::new();
        let mut overview = ClusterOverviewState::default();
        let mut namespaces = NamespaceFilterState::default();
        let mut namespaces_view = NamespacesViewState::default();
        let mut deployments = DeploymentsState::default();
        let mut statefulsets = StatefulSetsState::default();
        let mut daemonsets = DaemonSetsState::default();
        let mut jobs = JobsState::default();
        let mut cronjobs = CronJobsState::default();
        let mut pods = PodsState::default();
        let mut nodes = NodesState::default();
        let mut events = EventsState::default();

        // When active view is Overview, ClusterConnected resets EventsState to Idle without loading.
        let cmds_overview = apply_cluster_event_with_workspace(
            &mut switcher,
            &mut sidebar,
            &mut overview,
            &mut WorkloadsOverviewState::default(),
            &mut namespaces,
            &mut namespaces_view,
            &mut deployments,
            &mut statefulsets,
            &mut daemonsets,
            &mut ReplicaSetsState::default(),
            &mut jobs,
            &mut cronjobs,
            &mut pods,
            &mut ServicesState::default(),
            &mut IngressesState::default(),
            &mut ConfigMapsState::default(),
            &mut SecretsState::default(),
            &mut PersistentVolumesState::default(),
            &mut PersistentVolumeClaimsState::default(),
            &mut NetworkPoliciesState::default(),
            &mut StorageOverviewState::default(),
            &mut PodSecurityState::default(),
            &mut ResourceQuotasState::default(),
            &mut ClusterListViews::default(),
            &mut nodes,
            &mut events,
            ClusterBridgeEvent::ClusterConnected {
                outcome: ClusterSwitchOutcome {
                    context_name: "ctx-a".to_string(),
                    healthy: true,
                    k8s_version: Some("v1.31.1".to_string()),
                    error_message: None,
                },
                clusters: vec![],
                profile: UserProfileInfo {
                    active_context: Some("ctx-a".to_string()),
                    user_name: None,
                    auth_type: "None".to_string(),
                    cluster_name: Some("cluster-a".to_string()),
                    server_url: Some("https://10.0.0.10:6443".to_string()),
                    kubeconfig_paths: vec![],
                    k8s_version: Some("v1.31.1".to_string()),
                },
            },
        );
        assert!(
            !cmds_overview
                .iter()
                .any(|c| matches!(c, ClusterBridgeCommand::LoadEvents { .. }))
        );
        assert_eq!(events.cluster_id.as_deref(), Some("ctx-a"));
        assert_eq!(events.status, crate::state::EventsLoadStatus::Idle);

        // When active view is Events, ClusterConnected emits LoadEvents.
        sidebar.select_view(WorkspaceView::Events);
        let cmds_events = apply_cluster_event_with_workspace(
            &mut switcher,
            &mut sidebar,
            &mut overview,
            &mut WorkloadsOverviewState::default(),
            &mut namespaces,
            &mut namespaces_view,
            &mut deployments,
            &mut statefulsets,
            &mut daemonsets,
            &mut ReplicaSetsState::default(),
            &mut jobs,
            &mut cronjobs,
            &mut pods,
            &mut ServicesState::default(),
            &mut IngressesState::default(),
            &mut ConfigMapsState::default(),
            &mut SecretsState::default(),
            &mut PersistentVolumesState::default(),
            &mut PersistentVolumeClaimsState::default(),
            &mut NetworkPoliciesState::default(),
            &mut StorageOverviewState::default(),
            &mut PodSecurityState::default(),
            &mut ResourceQuotasState::default(),
            &mut ClusterListViews::default(),
            &mut nodes,
            &mut events,
            ClusterBridgeEvent::ClusterConnected {
                outcome: ClusterSwitchOutcome {
                    context_name: "ctx-b".to_string(),
                    healthy: true,
                    k8s_version: Some("v1.31.1".to_string()),
                    error_message: None,
                },
                clusters: vec![],
                profile: UserProfileInfo {
                    active_context: Some("ctx-b".to_string()),
                    user_name: None,
                    auth_type: "None".to_string(),
                    cluster_name: Some("cluster-b".to_string()),
                    server_url: Some("https://10.0.0.11:6443".to_string()),
                    kubeconfig_paths: vec![],
                    k8s_version: Some("v1.31.1".to_string()),
                },
            },
        );
        assert!(cmds_events.iter().any(|c| matches!(
            c,
            ClusterBridgeCommand::LoadEvents { cluster_id, .. } if cluster_id == "ctx-b"
        )));
        assert_eq!(events.status, crate::state::EventsLoadStatus::Loading);

        // ClusterUnreachable resets EventsState for disconnected state.
        apply_cluster_event_with_workspace(
            &mut switcher,
            &mut sidebar,
            &mut overview,
            &mut WorkloadsOverviewState::default(),
            &mut namespaces,
            &mut namespaces_view,
            &mut deployments,
            &mut statefulsets,
            &mut daemonsets,
            &mut ReplicaSetsState::default(),
            &mut jobs,
            &mut cronjobs,
            &mut pods,
            &mut ServicesState::default(),
            &mut IngressesState::default(),
            &mut ConfigMapsState::default(),
            &mut SecretsState::default(),
            &mut PersistentVolumesState::default(),
            &mut PersistentVolumeClaimsState::default(),
            &mut NetworkPoliciesState::default(),
            &mut StorageOverviewState::default(),
            &mut PodSecurityState::default(),
            &mut ResourceQuotasState::default(),
            &mut ClusterListViews::default(),
            &mut nodes,
            &mut events,
            ClusterBridgeEvent::ClusterUnreachable {
                outcome: ClusterSwitchOutcome {
                    context_name: "ctx-b".to_string(),
                    healthy: false,
                    k8s_version: None,
                    error_message: Some("timeout".to_string()),
                },
                clusters: vec![],
                profile: UserProfileInfo {
                    active_context: Some("ctx-b".to_string()),
                    user_name: None,
                    auth_type: "None".to_string(),
                    cluster_name: Some("cluster-b".to_string()),
                    server_url: Some("https://10.0.0.11:6443".to_string()),
                    kubeconfig_paths: vec![],
                    k8s_version: None,
                },
            },
        );
        assert!(events.cluster_id.is_none());
        assert_eq!(events.status, crate::state::EventsLoadStatus::Idle);
        assert!(events.events.is_empty());
    }

    #[test]
    fn test_context_menu_key_handling_and_dropdown_mutual_exclusion() {
        use crate::components::ContextMenuItem;

        let mut cluster_state = ClusterSwitcherState::new();
        cluster_state.dropdown_open = true;
        let mut ns_state = NamespaceFilterState {
            dropdown_open: true,
            ..Default::default()
        };
        let mut context_menu = ContextMenuState::new();

        // Opening context menu closes active header dropdowns
        cluster_state.close_dropdown();
        ns_state.close_dropdown();
        context_menu.open(
            gpui::point(px(20.0), px(30.0)),
            vec![
                ContextMenuEntry::Item(ContextMenuItem::new("copy", "Copy").on_select(|_, _| {})),
                ContextMenuEntry::Separator,
                ContextMenuEntry::Item(
                    ContextMenuItem::new("refresh", "Refresh").on_select(|_, _| {}),
                ),
            ],
        );

        assert!(!cluster_state.dropdown_open);
        assert!(!ns_state.dropdown_open);
        assert!(context_menu.is_open);
        assert_eq!(context_menu.highlighted_index, Some(0));

        let (action, _) = context_menu.handle_key("arrowdown");
        assert_eq!(action, ContextMenuKeyAction::Navigated);
        assert_eq!(context_menu.highlighted_index, Some(2));

        let (action, _) = context_menu.handle_key("arrowup");
        assert_eq!(action, ContextMenuKeyAction::Navigated);
        assert_eq!(context_menu.highlighted_index, Some(0));

        let (action, cb) = context_menu.handle_key("enter");
        assert_eq!(action, ContextMenuKeyAction::Activated);
        assert!(cb.is_some());
        assert!(!context_menu.is_open);

        context_menu.open(
            gpui::point(px(20.0), px(30.0)),
            vec![ContextMenuEntry::Item(ContextMenuItem::new("copy", "Copy"))],
        );
        let (action, cb) = context_menu.handle_key("escape");
        assert_eq!(action, ContextMenuKeyAction::Closed);
        assert!(cb.is_none());
        assert!(!context_menu.is_open);
    }

    #[test]
    fn test_resource_row_context_menu_builders_include_actions_separators_and_danger_items() {
        use orbit_core::kubernetes::models::{
            DeploymentInfo, EventInfo, NamespaceInfo, NodeInfo, Replicas,
        };

        let deployment = DeploymentInfo {
            kind: "Deployment".to_string(),
            name: "api-server".to_string(),
            namespace: "prod".to_string(),
            status: "Running".to_string(),
            conditions: None,
            replicas: Replicas {
                current: 2,
                desired: 2,
            },
            available: 2,
            up_to_date: 2,
            age: "1d".to_string(),
            images: Vec::new(),
            containers: Vec::new(),
            strategy: None,
            min_ready_seconds: 0,
            revision_history: None,
            labels: Default::default(),
            annotations: Default::default(),
            ports: Vec::new(),
        };
        let dep_entries = views::deployments::build_deployment_context_menu_entries(
            &deployment,
            |_, _| {},
            |_, _| {},
            |_, _| {},
            |_, _| {},
            |_, _| {},
            |_, _| {},
            |_, _| {},
            |_, _| {},
            |_, _| {},
            |_, _| {},
        );
        assert!(
            dep_entries
                .iter()
                .any(|e| matches!(e, ContextMenuEntry::Separator))
        );
        assert!(dep_entries.iter().any(|e| matches!(
            e,
            ContextMenuEntry::Item(item) if item.id.as_ref() == "scale" && !item.disabled
        )));
        assert!(dep_entries.iter().any(|e| matches!(
            e,
            ContextMenuEntry::Item(item) if item.id.as_ref() == "restart" && !item.disabled
        )));
        assert!(dep_entries.iter().any(|e| matches!(
            e,
            ContextMenuEntry::Item(item) if item.id.as_ref() == "copy-name" && !item.disabled
        )));
        assert!(dep_entries.iter().any(|e| matches!(
            e,
            ContextMenuEntry::Item(item) if item.id.as_ref() == "filter-namespace" && !item.disabled
        )));
        assert!(dep_entries.iter().any(|e| matches!(
            e,
            ContextMenuEntry::Item(item) if item.id.as_ref() == "view-details" && !item.disabled
        )));
        assert!(dep_entries.iter().any(|e| matches!(
            e,
            ContextMenuEntry::Item(item) if item.id.as_ref() == "view-yaml" && !item.disabled
        )));
        assert!(dep_entries.iter().any(|e| matches!(
            e,
            ContextMenuEntry::Item(item) if item.id.as_ref() == "delete-deployment" && item.danger && !item.disabled
        )));

        let node = NodeInfo {
            name: "worker-01".to_string(),
            status: "Ready".to_string(),
            role: "worker".to_string(),
            version: "v1.30.0".to_string(),
            cpu_pct: 20.0,
            cpu_used: "1".to_string(),
            cpu_total: "4".to_string(),
            mem_pct: 30.0,
            mem_used: "2Gi".to_string(),
            mem_total: "8Gi".to_string(),
            pods_count: 5,
            pods_limit: 110,
            uptime: "5d".to_string(),
            created_at: None,
            labels: Vec::new(),
            labels_map: Default::default(),
            annotations: Default::default(),
            is_cordoned: false,
            node_info: None,
            addresses: Vec::new(),
            conditions: Vec::new(),
            taints: Vec::new(),
            capacity: None,
            allocatable: None,
            images_count: 0,
        };
        let node_entries = views::nodes::build_node_context_menu_entries(
            &node,
            |_, _| {},
            |_, _| {},
            |_, _| {},
            |_, _| {},
            |_, _| {},
        );
        assert!(
            node_entries
                .iter()
                .any(|e| matches!(e, ContextMenuEntry::Separator))
        );
        assert!(node_entries.iter().any(|e| matches!(
            e,
            ContextMenuEntry::Item(item) if item.id.as_ref() == "copy-name" && !item.disabled
        )));
        assert!(node_entries.iter().any(|e| matches!(
            e,
            ContextMenuEntry::Item(item) if item.id.as_ref() == "view-details" && !item.disabled
        )));
        assert!(node_entries.iter().any(|e| matches!(
            e,
            ContextMenuEntry::Item(item) if item.id.as_ref() == "view-yaml" && !item.disabled
        )));
        assert!(node_entries.iter().any(|e| matches!(
            e,
            ContextMenuEntry::Item(item) if item.id.as_ref() == "delete-node" && item.danger && !item.disabled
        )));

        let ns = NamespaceInfo {
            name: "kube-system".to_string(),
            status: "Active".to_string(),
            is_system: true,
            age: "10d".to_string(),
            labels: Default::default(),
            annotations: Default::default(),
            uid: "uid-kube-system".to_string(),
            created: String::new(),
        };
        let ns_entries = views::namespaces::build_namespace_context_menu_entries(
            &ns,
            |_, _| {},
            |_, _| {},
            |_, _| {},
            |_, _| {},
            |_, _| {},
            |_, _| {},
        );
        assert!(
            ns_entries
                .iter()
                .any(|e| matches!(e, ContextMenuEntry::Separator))
        );
        assert!(ns_entries.iter().any(|e| matches!(
            e,
            ContextMenuEntry::Item(item) if item.id.as_ref() == "filter-namespace" && !item.disabled
        )));
        assert!(ns_entries.iter().any(|e| matches!(
            e,
            ContextMenuEntry::Item(item) if item.id.as_ref() == "view-details" && !item.disabled
        )));
        assert!(ns_entries.iter().any(|e| matches!(
            e,
            ContextMenuEntry::Item(item) if item.id.as_ref() == "view-yaml" && !item.disabled
        )));
        assert!(ns_entries.iter().any(|e| matches!(
            e,
            ContextMenuEntry::Item(item) if item.id.as_ref() == "delete-namespace" && item.danger && !item.disabled
        )));

        let ev = EventInfo {
            uid: "ev-1".to_string(),
            name: "api-server-abc.12345".to_string(),
            time: "2m".to_string(),
            r#type: "Normal".to_string(),
            reason: "ScalingReplicaSet".to_string(),
            object_name: "api-server-abc".to_string(),
            object_kind: "Deployment".to_string(),
            message: "Scaled up replica set".to_string(),
            namespace: "prod".to_string(),
            source: "deployment-controller".to_string(),
            first_seen: String::new(),
            last_seen: String::new(),
            count: 1,
            labels: Default::default(),
            annotations: Default::default(),
        };
        let ev_entries = views::events::build_event_context_menu_entries(
            &ev,
            |_, _| {},
            |_, _| {},
            |_, _| {},
            |_, _| {},
            |_, _| {},
            |_, _| {},
            |_, _| {},
        );
        assert!(
            ev_entries
                .iter()
                .any(|e| matches!(e, ContextMenuEntry::Separator))
        );
        assert!(ev_entries.iter().any(|e| matches!(
            e,
            ContextMenuEntry::Item(item) if item.id.as_ref() == "copy-object" && !item.disabled
        )));
        assert!(ev_entries.iter().any(|e| matches!(
            e,
            ContextMenuEntry::Item(item) if item.id.as_ref() == "copy-message" && !item.disabled
        )));
        assert!(ev_entries.iter().any(|e| matches!(
            e,
            ContextMenuEntry::Item(item) if item.id.as_ref() == "view-details" && !item.disabled
        )));
        assert!(ev_entries.iter().any(|e| matches!(
            e,
            ContextMenuEntry::Item(item) if item.id.as_ref() == "view-yaml" && !item.disabled
        )));
        assert!(ev_entries.iter().any(|e| matches!(
            e,
            ContextMenuEntry::Item(item) if item.id.as_ref() == "delete-event" && item.danger && !item.disabled
        )));
    }

    #[test]
    fn test_resource_drawer_dismissal_preserves_table_selection_and_filters() {
        let mut deployments = DeploymentsState::default();
        deployments.table.selected_row_key = Some("default/api".to_string());
        deployments.table.toggle_sort("name");
        let mut namespaces = NamespaceFilterState::default();
        namespaces
            .selected_by_cluster
            .insert("c1".to_string(), vec!["default".to_string()]);

        let mut drawer = ResourceDrawerState::default();
        let _ = drawer.open_or_switch(
            Some("c1"),
            ResourceDrawerTarget::Deployment {
                name: "api".to_string(),
                namespace: "default".to_string(),
            },
            ResourceDrawerTabId::Overview,
        );
        assert!(drawer.is_open);

        drawer.close();
        assert!(!drawer.is_open);
        assert_eq!(
            deployments.table.selected_row_key.as_deref(),
            Some("default/api")
        );
        assert!(deployments.table.sort.is_some());
        assert_eq!(
            namespaces.selected_by_cluster.get("c1"),
            Some(&vec!["default".to_string()])
        );
    }

    #[test]
    fn test_workspace_view_change_and_cluster_disconnect_close_drawer() {
        let mut drawer = ResourceDrawerState::default();
        let _ = drawer.open_or_switch(
            Some("c1"),
            ResourceDrawerTarget::Node {
                name: "node-1".to_string(),
            },
            ResourceDrawerTabId::Yaml,
        );
        let gen_open = drawer.yaml_generation;
        assert!(drawer.is_open);

        // Simulate disconnect / unreachable event
        apply_drawer_lifecycle_for_event(
            &mut drawer,
            &ClusterBridgeEvent::ConnectionError {
                cluster_id: Some("c1".to_string()),
                message: "lost connection".to_string(),
                clusters: Vec::new(),
            },
        );
        assert!(!drawer.is_open);
        assert!(drawer.target.is_none());
        assert!(drawer.yaml_generation > gen_open);

        // Reopen and simulate view switch reset
        let _ = drawer.open_or_switch(
            Some("c1"),
            ResourceDrawerTarget::Namespace {
                name: "default".to_string(),
            },
            ResourceDrawerTabId::Overview,
        );
        let gen_reopen = drawer.yaml_generation;
        assert!(drawer.is_open);
        drawer.reset_for_context_change();
        assert!(!drawer.is_open);
        assert!(drawer.yaml_generation > gen_reopen);
    }

    #[test]
    fn test_select_workspace_view_opens_and_deduplicates_tabs_and_triggers_load() {
        let mut state = ClusterSwitcherState::new();
        state.connection_state = ClusterConnectionState::Connected {
            cluster_id: "c1".to_string(),
            display_name: "c1".to_string(),
            k8s_version: None,
        };
        state.active_cluster_id = Some("c1".to_string());
        let mut sidebar = SidebarLayoutState::new();
        let mut namespaces = NamespaceFilterState::default();
        let mut deployments = DeploymentsState::default();
        let mut statefulsets = StatefulSetsState::default();
        let mut daemonsets = DaemonSetsState::default();
        let mut jobs = JobsState::default();
        let mut cronjobs = CronJobsState::default();
        let mut pods = PodsState::default();
        let mut nodes = NodesState::default();
        let mut events = EventsState::default();
        let mut tabs = WorkspaceTabsState::new();
        tabs.activate_cluster("c1");

        let cmds1 = select_workspace_view_with_tabs(
            &state,
            &mut sidebar,
            &mut WorkloadsOverviewState::default(),
            &mut namespaces,
            &mut deployments,
            &mut statefulsets,
            &mut daemonsets,
            &mut ReplicaSetsState::default(),
            &mut jobs,
            &mut cronjobs,
            &mut pods,
            &mut ServicesState::default(),
            &mut IngressesState::default(),
            &mut ConfigMapsState::default(),
            &mut SecretsState::default(),
            &mut PersistentVolumesState::default(),
            &mut PersistentVolumeClaimsState::default(),
            &mut NetworkPoliciesState::default(),
            &mut StorageOverviewState::default(),
            &mut PodSecurityState::default(),
            &mut ResourceQuotasState::default(),
            &mut ClusterListViews::default(),
            &mut nodes,
            &mut events,
            &mut tabs,
            WorkspaceView::Deployments,
        );
        assert!(cmds1.iter().any(|c| matches!(
            c,
            ClusterBridgeCommand::LoadDeployments { cluster_id, .. } if cluster_id == "c1"
        )));
        assert!(
            cmds1
                .iter()
                .any(|c| matches!(c, ClusterBridgeCommand::SaveWorkspaceTabs { .. }))
        );

        let cmds2 = select_workspace_view_with_tabs(
            &state,
            &mut sidebar,
            &mut WorkloadsOverviewState::default(),
            &mut namespaces,
            &mut deployments,
            &mut statefulsets,
            &mut daemonsets,
            &mut ReplicaSetsState::default(),
            &mut jobs,
            &mut cronjobs,
            &mut pods,
            &mut ServicesState::default(),
            &mut IngressesState::default(),
            &mut ConfigMapsState::default(),
            &mut SecretsState::default(),
            &mut PersistentVolumesState::default(),
            &mut PersistentVolumeClaimsState::default(),
            &mut NetworkPoliciesState::default(),
            &mut StorageOverviewState::default(),
            &mut PodSecurityState::default(),
            &mut ResourceQuotasState::default(),
            &mut ClusterListViews::default(),
            &mut nodes,
            &mut events,
            &mut tabs,
            WorkspaceView::Nodes,
        );
        assert!(cmds2.iter().any(|c| matches!(
            c,
            ClusterBridgeCommand::LoadNodes { cluster_id, .. } if cluster_id == "c1"
        )));

        let _ = select_workspace_view_with_tabs(
            &state,
            &mut sidebar,
            &mut WorkloadsOverviewState::default(),
            &mut namespaces,
            &mut deployments,
            &mut statefulsets,
            &mut daemonsets,
            &mut ReplicaSetsState::default(),
            &mut jobs,
            &mut cronjobs,
            &mut pods,
            &mut ServicesState::default(),
            &mut IngressesState::default(),
            &mut ConfigMapsState::default(),
            &mut SecretsState::default(),
            &mut PersistentVolumesState::default(),
            &mut PersistentVolumeClaimsState::default(),
            &mut NetworkPoliciesState::default(),
            &mut StorageOverviewState::default(),
            &mut PodSecurityState::default(),
            &mut ResourceQuotasState::default(),
            &mut ClusterListViews::default(),
            &mut nodes,
            &mut events,
            &mut tabs,
            WorkspaceView::Deployments,
        );

        assert_eq!(
            tabs.open_views(),
            vec![
                WorkspaceView::Overview,
                WorkspaceView::Deployments,
                WorkspaceView::Nodes,
            ]
        );
        assert_eq!(tabs.active_view(), Some(WorkspaceView::Deployments));
        assert_eq!(sidebar.active_view, WorkspaceView::Deployments);
    }

    #[test]
    fn test_cluster_connected_restores_per_cluster_tabs_and_loads_active_view() {
        let mut config = OrbitConfig::default();
        config.update_workspace_tabs_for_cluster(
            "c1",
            vec![
                WorkspaceView::Overview.id().to_string(),
                WorkspaceView::Deployments.id().to_string(),
            ],
            Some(WorkspaceView::Deployments.id().to_string()),
        );

        let mut state = ClusterSwitcherState::new();
        let mut sidebar = SidebarLayoutState::new();
        let mut overview = ClusterOverviewState::default();
        let mut namespaces = NamespaceFilterState::default();
        let mut namespaces_view = NamespacesViewState::default();
        let mut deployments = DeploymentsState::default();
        let mut statefulsets = StatefulSetsState::default();
        let mut daemonsets = DaemonSetsState::default();
        let mut jobs = JobsState::default();
        let mut cronjobs = CronJobsState::default();
        let mut pods = PodsState::default();
        let mut nodes = NodesState::default();
        let mut events = EventsState::default();
        let mut tabs = WorkspaceTabsState::from_config(&config);

        let cmds_a = apply_cluster_event_with_tabs(
            &mut state,
            &mut sidebar,
            &mut overview,
            &mut WorkloadsOverviewState::default(),
            &mut namespaces,
            &mut namespaces_view,
            &mut deployments,
            &mut statefulsets,
            &mut daemonsets,
            &mut ReplicaSetsState::default(),
            &mut jobs,
            &mut cronjobs,
            &mut pods,
            &mut ServicesState::default(),
            &mut IngressesState::default(),
            &mut ConfigMapsState::default(),
            &mut SecretsState::default(),
            &mut PersistentVolumesState::default(),
            &mut PersistentVolumeClaimsState::default(),
            &mut NetworkPoliciesState::default(),
            &mut StorageOverviewState::default(),
            &mut PodSecurityState::default(),
            &mut ResourceQuotasState::default(),
            &mut ClusterListViews::default(),
            &mut nodes,
            &mut events,
            &mut tabs,
            ClusterBridgeEvent::ClusterConnected {
                outcome: orbit_core::kubernetes::manager::ClusterSwitchOutcome {
                    context_name: "c1".to_string(),
                    healthy: true,
                    error_message: None,
                    k8s_version: Some("v1.30.0".to_string()),
                },
                clusters: Vec::new(),
                profile: orbit_core::kubernetes::models::UserProfileInfo {
                    active_context: Some("c1".to_string()),
                    user_name: None,
                    auth_type: "None".to_string(),
                    cluster_name: Some("c1".to_string()),
                    server_url: None,
                    kubeconfig_paths: vec![],
                    k8s_version: Some("v1.30.0".to_string()),
                },
            },
        );

        assert_eq!(sidebar.active_view, WorkspaceView::Deployments);
        assert_eq!(tabs.active_view(), Some(WorkspaceView::Deployments));
        assert!(cmds_a.iter().any(|c| matches!(
            c,
            ClusterBridgeCommand::LoadDeployments { cluster_id, .. } if cluster_id == "c1"
        )));

        // Switch to cluster c2 (never visited -> defaults to Overview)
        let _ = apply_cluster_event_with_tabs(
            &mut state,
            &mut sidebar,
            &mut overview,
            &mut WorkloadsOverviewState::default(),
            &mut namespaces,
            &mut namespaces_view,
            &mut deployments,
            &mut statefulsets,
            &mut daemonsets,
            &mut ReplicaSetsState::default(),
            &mut jobs,
            &mut cronjobs,
            &mut pods,
            &mut ServicesState::default(),
            &mut IngressesState::default(),
            &mut ConfigMapsState::default(),
            &mut SecretsState::default(),
            &mut PersistentVolumesState::default(),
            &mut PersistentVolumeClaimsState::default(),
            &mut NetworkPoliciesState::default(),
            &mut StorageOverviewState::default(),
            &mut PodSecurityState::default(),
            &mut ResourceQuotasState::default(),
            &mut ClusterListViews::default(),
            &mut nodes,
            &mut events,
            &mut tabs,
            ClusterBridgeEvent::ClusterConnected {
                outcome: orbit_core::kubernetes::manager::ClusterSwitchOutcome {
                    context_name: "c2".to_string(),
                    healthy: true,
                    error_message: None,
                    k8s_version: Some("v1.30.0".to_string()),
                },
                clusters: Vec::new(),
                profile: orbit_core::kubernetes::models::UserProfileInfo {
                    active_context: Some("c2".to_string()),
                    user_name: None,
                    auth_type: "None".to_string(),
                    cluster_name: Some("c2".to_string()),
                    server_url: None,
                    kubeconfig_paths: vec![],
                    k8s_version: Some("v1.30.0".to_string()),
                },
            },
        );
        assert_eq!(sidebar.active_view, WorkspaceView::Overview);
        assert_eq!(tabs.open_views(), vec![WorkspaceView::Overview]);
    }

    #[test]
    fn test_tab_keyboard_shortcuts_cycle_and_close_tabs() {
        let mut state = ClusterSwitcherState::new();
        state.connection_state = ClusterConnectionState::Connected {
            cluster_id: "c1".to_string(),
            display_name: "c1".to_string(),
            k8s_version: None,
        };
        state.active_cluster_id = Some("c1".to_string());
        let mut sidebar = SidebarLayoutState::new();
        let mut namespaces = NamespaceFilterState::default();
        let mut deployments = DeploymentsState::default();
        let mut statefulsets = StatefulSetsState::default();
        let mut daemonsets = DaemonSetsState::default();
        let mut jobs = JobsState::default();
        let mut cronjobs = CronJobsState::default();
        let mut pods = PodsState::default();
        let mut nodes = NodesState::default();
        let mut events = EventsState::default();
        let mut tabs = WorkspaceTabsState::new();
        tabs.activate_cluster("c1");
        tabs.open_tab(WorkspaceView::Deployments);
        tabs.open_tab(WorkspaceView::Nodes);
        tabs.set_active_tab(WorkspaceView::Overview);
        sidebar.sync_for_view(tabs.active_view());

        // Ctrl+Tab -> Deployments
        let (handled, _) = handle_tab_shortcut_with_state(
            &state,
            &mut sidebar,
            &mut WorkloadsOverviewState::default(),
            &mut namespaces,
            &mut deployments,
            &mut statefulsets,
            &mut daemonsets,
            &mut ReplicaSetsState::default(),
            &mut jobs,
            &mut cronjobs,
            &mut pods,
            &mut ServicesState::default(),
            &mut IngressesState::default(),
            &mut ConfigMapsState::default(),
            &mut SecretsState::default(),
            &mut PersistentVolumesState::default(),
            &mut PersistentVolumeClaimsState::default(),
            &mut NetworkPoliciesState::default(),
            &mut StorageOverviewState::default(),
            &mut PodSecurityState::default(),
            &mut ResourceQuotasState::default(),
            &mut ClusterListViews::default(),
            &mut nodes,
            &mut events,
            &mut tabs,
            "tab",
            true,
            false,
            false,
        );
        assert!(handled);
        assert_eq!(tabs.active_view(), Some(WorkspaceView::Deployments));
        assert_eq!(sidebar.active_view, WorkspaceView::Deployments);

        // Ctrl+Shift+Tab -> Overview
        let (handled, _) = handle_tab_shortcut_with_state(
            &state,
            &mut sidebar,
            &mut WorkloadsOverviewState::default(),
            &mut namespaces,
            &mut deployments,
            &mut statefulsets,
            &mut daemonsets,
            &mut ReplicaSetsState::default(),
            &mut jobs,
            &mut cronjobs,
            &mut pods,
            &mut ServicesState::default(),
            &mut IngressesState::default(),
            &mut ConfigMapsState::default(),
            &mut SecretsState::default(),
            &mut PersistentVolumesState::default(),
            &mut PersistentVolumeClaimsState::default(),
            &mut NetworkPoliciesState::default(),
            &mut StorageOverviewState::default(),
            &mut PodSecurityState::default(),
            &mut ResourceQuotasState::default(),
            &mut ClusterListViews::default(),
            &mut nodes,
            &mut events,
            &mut tabs,
            "tab",
            true,
            true,
            false,
        );
        assert!(handled);
        assert_eq!(tabs.active_view(), Some(WorkspaceView::Overview));

        // Ctrl+W -> closes Overview, activates Deployments
        let (handled, _) = handle_tab_shortcut_with_state(
            &state,
            &mut sidebar,
            &mut WorkloadsOverviewState::default(),
            &mut namespaces,
            &mut deployments,
            &mut statefulsets,
            &mut daemonsets,
            &mut ReplicaSetsState::default(),
            &mut jobs,
            &mut cronjobs,
            &mut pods,
            &mut ServicesState::default(),
            &mut IngressesState::default(),
            &mut ConfigMapsState::default(),
            &mut SecretsState::default(),
            &mut PersistentVolumesState::default(),
            &mut PersistentVolumeClaimsState::default(),
            &mut NetworkPoliciesState::default(),
            &mut StorageOverviewState::default(),
            &mut PodSecurityState::default(),
            &mut ResourceQuotasState::default(),
            &mut ClusterListViews::default(),
            &mut nodes,
            &mut events,
            &mut tabs,
            "w",
            true,
            false,
            false,
        );
        assert!(handled);
        assert_eq!(
            tabs.open_views(),
            vec![WorkspaceView::Deployments, WorkspaceView::Nodes]
        );
        assert_eq!(tabs.active_view(), Some(WorkspaceView::Deployments));

        // Close remaining two tabs
        for _ in 0..2 {
            let _ = handle_tab_shortcut_with_state(
                &state,
                &mut sidebar,
                &mut WorkloadsOverviewState::default(),
                &mut namespaces,
                &mut deployments,
                &mut statefulsets,
                &mut daemonsets,
                &mut ReplicaSetsState::default(),
                &mut jobs,
                &mut cronjobs,
                &mut pods,
                &mut ServicesState::default(),
                &mut IngressesState::default(),
                &mut ConfigMapsState::default(),
                &mut SecretsState::default(),
                &mut PersistentVolumesState::default(),
                &mut PersistentVolumeClaimsState::default(),
                &mut NetworkPoliciesState::default(),
                &mut StorageOverviewState::default(),
                &mut PodSecurityState::default(),
                &mut ResourceQuotasState::default(),
                &mut ClusterListViews::default(),
                &mut nodes,
                &mut events,
                &mut tabs,
                "w",
                false,
                false,
                true,
            );
        }
        assert!(tabs.open_tabs().is_empty());
        assert_eq!(tabs.active_view(), None);
        assert!(!tabs.is_bar_visible(true));
    }

    fn make_test_pod(name: &str, namespace: &str) -> orbit_core::kubernetes::models::PodInfo {
        orbit_core::kubernetes::models::PodInfo {
            name: name.to_string(),
            namespace: namespace.to_string(),
            status: "Running".to_string(),
            age: "5m".to_string(),
            cpu: Some("50m".to_string()),
            memory: Some("64Mi".to_string()),
            cpu_cores: Some(0.05),
            memory_bytes: Some(67_108_864.0),
            cpu_pct: None,
            memory_pct: None,
            node: Some("node-1".to_string()),
            restarts: 0,
            images: vec!["nginx:latest".to_string()],
            labels: Default::default(),
            annotations: Default::default(),
            ip: Some("10.0.0.5".to_string()),
            node_ip: Some("192.168.1.5".to_string()),
            controlled_by: None,
            qos_class: Some("Burstable".to_string()),
            containers: vec![],
        }
    }

    #[test]
    fn test_selecting_pods_view_triggers_load_pods_command() {
        let mut state = ClusterSwitcherState::new();
        state.connection_state = ClusterConnectionState::Connected {
            cluster_id: "prod-ctx".to_string(),
            display_name: "Prod".to_string(),
            k8s_version: None,
        };
        let mut sidebar = SidebarLayoutState::new();
        let mut overview = ClusterOverviewState::default();
        let mut namespaces = NamespaceFilterState::default();
        let mut namespaces_view = NamespacesViewState::default();
        let mut deployments = DeploymentsState::default();
        let mut statefulsets = StatefulSetsState::default();
        let mut daemonsets = DaemonSetsState::default();
        let mut jobs = JobsState::default();
        let mut cronjobs = CronJobsState::default();
        let mut pods = PodsState::default();
        let mut nodes = NodesState::default();
        let mut events = EventsState::default();
        let mut tabs = WorkspaceTabsState::default();
        tabs.activate_cluster("prod-ctx");

        let cmds = select_workspace_view_with_tabs(
            &state,
            &mut sidebar,
            &mut WorkloadsOverviewState::default(),
            &mut namespaces,
            &mut deployments,
            &mut statefulsets,
            &mut daemonsets,
            &mut ReplicaSetsState::default(),
            &mut jobs,
            &mut cronjobs,
            &mut pods,
            &mut ServicesState::default(),
            &mut IngressesState::default(),
            &mut ConfigMapsState::default(),
            &mut SecretsState::default(),
            &mut PersistentVolumesState::default(),
            &mut PersistentVolumeClaimsState::default(),
            &mut NetworkPoliciesState::default(),
            &mut StorageOverviewState::default(),
            &mut PodSecurityState::default(),
            &mut ResourceQuotasState::default(),
            &mut ClusterListViews::default(),
            &mut nodes,
            &mut events,
            &mut tabs,
            WorkspaceView::Pods,
        );
        assert_eq!(tabs.active_view(), Some(WorkspaceView::Pods));
        assert!(cmds.iter().any(|c| matches!(
            c,
            ClusterBridgeCommand::LoadPods { cluster_id, generation: 1 } if cluster_id == "prod-ctx"
        )));

        // Connecting while already on WorkspaceView::Pods also dispatches LoadPods.
        let mut pods_on_connect = PodsState::default();
        let connect_cmds = apply_cluster_event_with_workspace(
            &mut state,
            &mut sidebar,
            &mut overview,
            &mut WorkloadsOverviewState::default(),
            &mut namespaces,
            &mut namespaces_view,
            &mut deployments,
            &mut statefulsets,
            &mut daemonsets,
            &mut ReplicaSetsState::default(),
            &mut jobs,
            &mut cronjobs,
            &mut pods_on_connect,
            &mut ServicesState::default(),
            &mut IngressesState::default(),
            &mut ConfigMapsState::default(),
            &mut SecretsState::default(),
            &mut PersistentVolumesState::default(),
            &mut PersistentVolumeClaimsState::default(),
            &mut NetworkPoliciesState::default(),
            &mut StorageOverviewState::default(),
            &mut PodSecurityState::default(),
            &mut ResourceQuotasState::default(),
            &mut ClusterListViews::default(),
            &mut nodes,
            &mut events,
            ClusterBridgeEvent::ClusterConnected {
                outcome: orbit_core::kubernetes::manager::ClusterSwitchOutcome {
                    context_name: "prod-ctx".to_string(),
                    healthy: true,
                    k8s_version: Some("v1.31.2".to_string()),
                    error_message: None,
                },
                clusters: vec![],
                profile: orbit_core::kubernetes::models::UserProfileInfo {
                    active_context: Some("prod-ctx".to_string()),
                    cluster_name: Some("Prod".to_string()),
                    server_url: None,
                    user_name: None,
                    auth_type: "token".to_string(),
                    kubeconfig_paths: vec![],
                    k8s_version: Some("v1.31.2".to_string()),
                },
            },
        );
        assert!(connect_cmds.iter().any(|c| matches!(
            c,
            ClusterBridgeCommand::LoadPods { cluster_id, generation: 1 } if cluster_id == "prod-ctx"
        )));
    }

    #[test]
    fn test_pods_loaded_and_failed_events_update_state() {
        let mut state = ClusterSwitcherState::new();
        let mut sidebar = SidebarLayoutState::new();
        let mut overview = ClusterOverviewState::default();
        let mut namespaces = NamespaceFilterState::default();
        let mut namespaces_view = NamespacesViewState::default();
        let mut deployments = DeploymentsState::default();
        let mut statefulsets = StatefulSetsState::default();
        let mut daemonsets = DaemonSetsState::default();
        let mut jobs = JobsState::default();
        let mut cronjobs = CronJobsState::default();
        let mut pods = PodsState::default();
        let mut nodes = NodesState::default();
        let mut events = EventsState::default();

        pods.begin_load_for_cluster("prod-ctx");
        let gen1 = pods.generation;

        apply_cluster_event_with_workspace(
            &mut state,
            &mut sidebar,
            &mut overview,
            &mut WorkloadsOverviewState::default(),
            &mut namespaces,
            &mut namespaces_view,
            &mut deployments,
            &mut statefulsets,
            &mut daemonsets,
            &mut ReplicaSetsState::default(),
            &mut jobs,
            &mut cronjobs,
            &mut pods,
            &mut ServicesState::default(),
            &mut IngressesState::default(),
            &mut ConfigMapsState::default(),
            &mut SecretsState::default(),
            &mut PersistentVolumesState::default(),
            &mut PersistentVolumeClaimsState::default(),
            &mut NetworkPoliciesState::default(),
            &mut StorageOverviewState::default(),
            &mut PodSecurityState::default(),
            &mut ResourceQuotasState::default(),
            &mut ClusterListViews::default(),
            &mut nodes,
            &mut events,
            ClusterBridgeEvent::PodsLoaded {
                cluster_id: "prod-ctx".to_string(),
                generation: gen1,
                pods: vec![make_test_pod("api-0", "prod")],
            },
        );
        assert_eq!(pods.status, crate::state::PodsLoadStatus::Loaded);
        assert_eq!(pods.pods.len(), 1);

        pods.begin_refresh();
        let gen2 = pods.generation;
        apply_cluster_event_with_workspace(
            &mut state,
            &mut sidebar,
            &mut overview,
            &mut WorkloadsOverviewState::default(),
            &mut namespaces,
            &mut namespaces_view,
            &mut deployments,
            &mut statefulsets,
            &mut daemonsets,
            &mut ReplicaSetsState::default(),
            &mut jobs,
            &mut cronjobs,
            &mut pods,
            &mut ServicesState::default(),
            &mut IngressesState::default(),
            &mut ConfigMapsState::default(),
            &mut SecretsState::default(),
            &mut PersistentVolumesState::default(),
            &mut PersistentVolumeClaimsState::default(),
            &mut NetworkPoliciesState::default(),
            &mut StorageOverviewState::default(),
            &mut PodSecurityState::default(),
            &mut ResourceQuotasState::default(),
            &mut ClusterListViews::default(),
            &mut nodes,
            &mut events,
            ClusterBridgeEvent::PodsLoadFailed {
                cluster_id: "prod-ctx".to_string(),
                generation: gen2,
                message: "forbidden".to_string(),
            },
        );
        assert_eq!(pods.status, crate::state::PodsLoadStatus::Error);
        assert_eq!(pods.error_message.as_deref(), Some("forbidden"));

        apply_cluster_event_with_workspace(
            &mut state,
            &mut sidebar,
            &mut overview,
            &mut WorkloadsOverviewState::default(),
            &mut namespaces,
            &mut namespaces_view,
            &mut deployments,
            &mut statefulsets,
            &mut daemonsets,
            &mut ReplicaSetsState::default(),
            &mut jobs,
            &mut cronjobs,
            &mut pods,
            &mut ServicesState::default(),
            &mut IngressesState::default(),
            &mut ConfigMapsState::default(),
            &mut SecretsState::default(),
            &mut PersistentVolumesState::default(),
            &mut PersistentVolumeClaimsState::default(),
            &mut NetworkPoliciesState::default(),
            &mut StorageOverviewState::default(),
            &mut PodSecurityState::default(),
            &mut ResourceQuotasState::default(),
            &mut ClusterListViews::default(),
            &mut nodes,
            &mut events,
            ClusterBridgeEvent::ClusterUnreachable {
                outcome: orbit_core::kubernetes::manager::ClusterSwitchOutcome {
                    context_name: "prod-ctx".to_string(),
                    healthy: false,
                    k8s_version: None,
                    error_message: Some("timeout".to_string()),
                },
                clusters: vec![],
                profile: orbit_core::kubernetes::models::UserProfileInfo {
                    active_context: Some("prod-ctx".to_string()),
                    cluster_name: Some("Prod".to_string()),
                    server_url: None,
                    user_name: None,
                    auth_type: "token".to_string(),
                    kubeconfig_paths: vec![],
                    k8s_version: None,
                },
            },
        );
        assert!(pods.cluster_id.is_none());
        assert!(pods.pods.is_empty());
        assert_eq!(pods.status, crate::state::PodsLoadStatus::Idle);
    }

    #[test]
    fn test_build_pod_context_menu_entries_contains_expected_actions() {
        let pod = make_test_pod("api-0", "prod");
        let entries = views::pods::build_pod_context_menu_entries(
            &pod,
            |_, _| {},
            |_, _| {},
            |_, _| {},
            |_, _| {},
            |_, _| {},
            |_, _| {},
            |_, _| {},
            |_, _| {},
            |_, _| {},
        );
        let items: Vec<&crate::components::ContextMenuItem> = entries
            .iter()
            .filter_map(|e| match e {
                ContextMenuEntry::Item(item) => Some(item),
                ContextMenuEntry::Separator => None,
            })
            .collect();
        let ids: Vec<&str> = items.iter().map(|i| i.id.as_ref()).collect();
        assert_eq!(
            ids,
            vec![
                "copy-name",
                "filter-namespace",
                "refresh",
                "view-details",
                "view-yaml",
                "view-logs",
                "port-forward",
                "restart-pod",
                "delete-pod",
            ]
        );
        let delete_item = items.iter().find(|i| i.id == "delete-pod").unwrap();
        assert!(!delete_item.disabled);
        assert!(delete_item.danger);
    }

    fn make_test_service(
        name: &str,
        namespace: &str,
    ) -> orbit_core::kubernetes::models::ServiceInfo {
        orbit_core::kubernetes::models::ServiceInfo {
            name: name.to_string(),
            namespace: namespace.to_string(),
            r#type: "ClusterIP".to_string(),
            cluster_ip: "10.96.0.10".to_string(),
            external_ip: "-".to_string(),
            ports: "80/TCP".to_string(),
            endpoints: "2 ready".to_string(),
            age: "1h".to_string(),
            session_affinity: "None".to_string(),
            internal_traffic_policy: None,
            created: String::new(),
            uid: format!("uid-{name}"),
            selector: Default::default(),
            labels: Default::default(),
            ports_list: Vec::new(),
            endpoints_list: Vec::new(),
            events: Vec::new(),
        }
    }

    #[test]
    fn test_selecting_services_view_triggers_load_services_command() {
        let mut state = ClusterSwitcherState::new();
        state.connection_state = ClusterConnectionState::Connected {
            cluster_id: "prod-ctx".to_string(),
            display_name: "Prod".to_string(),
            k8s_version: Some("v1.31.2".to_string()),
        };
        let mut sidebar = SidebarLayoutState::new();
        let mut overview = ClusterOverviewState::default();
        let mut namespaces = NamespaceFilterState::default();
        let mut namespaces_view = NamespacesViewState::default();
        let mut deployments = DeploymentsState::default();
        let mut statefulsets = StatefulSetsState::default();
        let mut daemonsets = DaemonSetsState::default();
        let mut jobs = JobsState::default();
        let mut cronjobs = CronJobsState::default();
        let mut pods = PodsState::default();
        let mut services = ServicesState::default();
        let mut nodes = NodesState::default();
        let mut events = EventsState::default();
        let mut tabs = WorkspaceTabsState::default();
        tabs.activate_cluster("prod-ctx");

        let cmds = select_workspace_view_with_tabs(
            &state,
            &mut sidebar,
            &mut WorkloadsOverviewState::default(),
            &mut namespaces,
            &mut deployments,
            &mut statefulsets,
            &mut daemonsets,
            &mut ReplicaSetsState::default(),
            &mut jobs,
            &mut cronjobs,
            &mut pods,
            &mut services,
            &mut IngressesState::default(),
            &mut ConfigMapsState::default(),
            &mut SecretsState::default(),
            &mut PersistentVolumesState::default(),
            &mut PersistentVolumeClaimsState::default(),
            &mut NetworkPoliciesState::default(),
            &mut StorageOverviewState::default(),
            &mut PodSecurityState::default(),
            &mut ResourceQuotasState::default(),
            &mut ClusterListViews::default(),
            &mut nodes,
            &mut events,
            &mut tabs,
            WorkspaceView::Services,
        );
        assert_eq!(tabs.active_view(), Some(WorkspaceView::Services));
        assert_eq!(sidebar.active_view, WorkspaceView::Services);
        assert!(cmds.iter().any(|c| matches!(
            c,
            ClusterBridgeCommand::LoadServices { cluster_id, generation: 1 } if cluster_id == "prod-ctx"
        )));

        // Connecting while already on WorkspaceView::Services also dispatches LoadServices.
        let mut services_on_connect = ServicesState::default();
        let connect_cmds = apply_cluster_event_with_workspace(
            &mut state,
            &mut sidebar,
            &mut overview,
            &mut WorkloadsOverviewState::default(),
            &mut namespaces,
            &mut namespaces_view,
            &mut deployments,
            &mut statefulsets,
            &mut daemonsets,
            &mut ReplicaSetsState::default(),
            &mut jobs,
            &mut cronjobs,
            &mut pods,
            &mut services_on_connect,
            &mut IngressesState::default(),
            &mut ConfigMapsState::default(),
            &mut SecretsState::default(),
            &mut PersistentVolumesState::default(),
            &mut PersistentVolumeClaimsState::default(),
            &mut NetworkPoliciesState::default(),
            &mut StorageOverviewState::default(),
            &mut PodSecurityState::default(),
            &mut ResourceQuotasState::default(),
            &mut ClusterListViews::default(),
            &mut nodes,
            &mut events,
            ClusterBridgeEvent::ClusterConnected {
                outcome: orbit_core::kubernetes::manager::ClusterSwitchOutcome {
                    context_name: "prod-ctx".to_string(),
                    healthy: true,
                    k8s_version: Some("v1.31.2".to_string()),
                    error_message: None,
                },
                clusters: vec![],
                profile: orbit_core::kubernetes::models::UserProfileInfo {
                    active_context: Some("prod-ctx".to_string()),
                    cluster_name: Some("Prod".to_string()),
                    server_url: None,
                    user_name: None,
                    auth_type: "token".to_string(),
                    kubeconfig_paths: vec![],
                    k8s_version: Some("v1.31.2".to_string()),
                },
            },
        );
        assert!(connect_cmds.iter().any(|c| matches!(
            c,
            ClusterBridgeCommand::LoadServices { cluster_id, generation: 1 } if cluster_id == "prod-ctx"
        )));
    }

    #[test]
    fn test_services_loaded_and_failed_events_update_state() {
        let mut state = ClusterSwitcherState::new();
        let mut sidebar = SidebarLayoutState::new();
        let mut overview = ClusterOverviewState::default();
        let mut namespaces = NamespaceFilterState::default();
        let mut namespaces_view = NamespacesViewState::default();
        let mut deployments = DeploymentsState::default();
        let mut statefulsets = StatefulSetsState::default();
        let mut daemonsets = DaemonSetsState::default();
        let mut jobs = JobsState::default();
        let mut cronjobs = CronJobsState::default();
        let mut pods = PodsState::default();
        let mut services = ServicesState::default();
        let mut nodes = NodesState::default();
        let mut events = EventsState::default();

        services.begin_load_for_cluster("prod-ctx");
        let gen1 = services.generation;

        // Stale generation should be ignored
        apply_cluster_event_with_workspace(
            &mut state,
            &mut sidebar,
            &mut overview,
            &mut WorkloadsOverviewState::default(),
            &mut namespaces,
            &mut namespaces_view,
            &mut deployments,
            &mut statefulsets,
            &mut daemonsets,
            &mut ReplicaSetsState::default(),
            &mut jobs,
            &mut cronjobs,
            &mut pods,
            &mut services,
            &mut IngressesState::default(),
            &mut ConfigMapsState::default(),
            &mut SecretsState::default(),
            &mut PersistentVolumesState::default(),
            &mut PersistentVolumeClaimsState::default(),
            &mut NetworkPoliciesState::default(),
            &mut StorageOverviewState::default(),
            &mut PodSecurityState::default(),
            &mut ResourceQuotasState::default(),
            &mut ClusterListViews::default(),
            &mut nodes,
            &mut events,
            ClusterBridgeEvent::ServicesLoaded {
                cluster_id: "prod-ctx".to_string(),
                generation: gen1 + 99,
                services: vec![make_test_service("stale-svc", "prod")],
            },
        );
        assert!(services.services.is_empty());

        // Matching generation updates state
        apply_cluster_event_with_workspace(
            &mut state,
            &mut sidebar,
            &mut overview,
            &mut WorkloadsOverviewState::default(),
            &mut namespaces,
            &mut namespaces_view,
            &mut deployments,
            &mut statefulsets,
            &mut daemonsets,
            &mut ReplicaSetsState::default(),
            &mut jobs,
            &mut cronjobs,
            &mut pods,
            &mut services,
            &mut IngressesState::default(),
            &mut ConfigMapsState::default(),
            &mut SecretsState::default(),
            &mut PersistentVolumesState::default(),
            &mut PersistentVolumeClaimsState::default(),
            &mut NetworkPoliciesState::default(),
            &mut StorageOverviewState::default(),
            &mut PodSecurityState::default(),
            &mut ResourceQuotasState::default(),
            &mut ClusterListViews::default(),
            &mut nodes,
            &mut events,
            ClusterBridgeEvent::ServicesLoaded {
                cluster_id: "prod-ctx".to_string(),
                generation: gen1,
                services: vec![make_test_service("api-svc", "prod")],
            },
        );
        assert_eq!(services.status, crate::state::ServicesLoadStatus::Loaded);
        assert_eq!(services.services.len(), 1);

        services.begin_refresh();
        let gen2 = services.generation;
        apply_cluster_event_with_workspace(
            &mut state,
            &mut sidebar,
            &mut overview,
            &mut WorkloadsOverviewState::default(),
            &mut namespaces,
            &mut namespaces_view,
            &mut deployments,
            &mut statefulsets,
            &mut daemonsets,
            &mut ReplicaSetsState::default(),
            &mut jobs,
            &mut cronjobs,
            &mut pods,
            &mut services,
            &mut IngressesState::default(),
            &mut ConfigMapsState::default(),
            &mut SecretsState::default(),
            &mut PersistentVolumesState::default(),
            &mut PersistentVolumeClaimsState::default(),
            &mut NetworkPoliciesState::default(),
            &mut StorageOverviewState::default(),
            &mut PodSecurityState::default(),
            &mut ResourceQuotasState::default(),
            &mut ClusterListViews::default(),
            &mut nodes,
            &mut events,
            ClusterBridgeEvent::ServicesLoadFailed {
                cluster_id: "prod-ctx".to_string(),
                generation: gen2,
                message: "forbidden".to_string(),
            },
        );
        assert_eq!(services.status, crate::state::ServicesLoadStatus::Error);
        assert_eq!(services.error_message.as_deref(), Some("forbidden"));
    }

    #[test]
    fn test_apply_cluster_event_resets_services_on_connect_and_disconnect() {
        let mut state = ClusterSwitcherState::new();
        let mut sidebar = SidebarLayoutState::new();
        let mut overview = ClusterOverviewState::default();
        let mut namespaces = NamespaceFilterState::default();
        let mut namespaces_view = NamespacesViewState::default();
        let mut deployments = DeploymentsState::default();
        let mut statefulsets = StatefulSetsState::default();
        let mut daemonsets = DaemonSetsState::default();
        let mut jobs = JobsState::default();
        let mut cronjobs = CronJobsState::default();
        let mut pods = PodsState::default();
        let mut services = ServicesState::default();
        let mut nodes = NodesState::default();
        let mut events = EventsState::default();

        services.begin_load_for_cluster("old-ctx");
        let gen1 = services.generation;
        services.apply_loaded(
            "old-ctx",
            gen1,
            vec![make_test_service("old-svc", "default")],
        );
        assert_eq!(services.services.len(), 1);

        // Connecting to a new cluster while on another view resets services for that cluster
        sidebar.active_view = WorkspaceView::Overview;
        apply_cluster_event_with_workspace(
            &mut state,
            &mut sidebar,
            &mut overview,
            &mut WorkloadsOverviewState::default(),
            &mut namespaces,
            &mut namespaces_view,
            &mut deployments,
            &mut statefulsets,
            &mut daemonsets,
            &mut ReplicaSetsState::default(),
            &mut jobs,
            &mut cronjobs,
            &mut pods,
            &mut services,
            &mut IngressesState::default(),
            &mut ConfigMapsState::default(),
            &mut SecretsState::default(),
            &mut PersistentVolumesState::default(),
            &mut PersistentVolumeClaimsState::default(),
            &mut NetworkPoliciesState::default(),
            &mut StorageOverviewState::default(),
            &mut PodSecurityState::default(),
            &mut ResourceQuotasState::default(),
            &mut ClusterListViews::default(),
            &mut nodes,
            &mut events,
            ClusterBridgeEvent::ClusterConnected {
                outcome: orbit_core::kubernetes::manager::ClusterSwitchOutcome {
                    context_name: "prod-ctx".to_string(),
                    healthy: true,
                    k8s_version: Some("v1.31.2".to_string()),
                    error_message: None,
                },
                clusters: vec![],
                profile: orbit_core::kubernetes::models::UserProfileInfo {
                    active_context: Some("prod-ctx".to_string()),
                    cluster_name: Some("Prod".to_string()),
                    server_url: None,
                    user_name: None,
                    auth_type: "token".to_string(),
                    kubeconfig_paths: vec![],
                    k8s_version: Some("v1.31.2".to_string()),
                },
            },
        );
        assert_eq!(services.cluster_id.as_deref(), Some("prod-ctx"));
        assert_eq!(services.status, crate::state::ServicesLoadStatus::Idle);
        assert!(services.services.is_empty());

        // Disconnecting clears cluster_id and services
        apply_cluster_event_with_workspace(
            &mut state,
            &mut sidebar,
            &mut overview,
            &mut WorkloadsOverviewState::default(),
            &mut namespaces,
            &mut namespaces_view,
            &mut deployments,
            &mut statefulsets,
            &mut daemonsets,
            &mut ReplicaSetsState::default(),
            &mut jobs,
            &mut cronjobs,
            &mut pods,
            &mut services,
            &mut IngressesState::default(),
            &mut ConfigMapsState::default(),
            &mut SecretsState::default(),
            &mut PersistentVolumesState::default(),
            &mut PersistentVolumeClaimsState::default(),
            &mut NetworkPoliciesState::default(),
            &mut StorageOverviewState::default(),
            &mut PodSecurityState::default(),
            &mut ResourceQuotasState::default(),
            &mut ClusterListViews::default(),
            &mut nodes,
            &mut events,
            ClusterBridgeEvent::ClusterUnreachable {
                outcome: orbit_core::kubernetes::manager::ClusterSwitchOutcome {
                    context_name: "prod-ctx".to_string(),
                    healthy: false,
                    k8s_version: None,
                    error_message: Some("timeout".to_string()),
                },
                clusters: vec![],
                profile: orbit_core::kubernetes::models::UserProfileInfo {
                    active_context: Some("prod-ctx".to_string()),
                    cluster_name: Some("Prod".to_string()),
                    server_url: None,
                    user_name: None,
                    auth_type: "token".to_string(),
                    kubeconfig_paths: vec![],
                    k8s_version: None,
                },
            },
        );
        assert!(services.cluster_id.is_none());
        assert!(services.services.is_empty());
        assert_eq!(services.status, crate::state::ServicesLoadStatus::Idle);
    }

    #[test]
    fn test_service_drawer_and_context_menu_wiring() {
        let mut drawer = ResourceDrawerState::default();
        let target = ResourceDrawerTarget::Service {
            name: "api-svc".to_string(),
            namespace: "prod".to_string(),
        };
        let yaml_cmd =
            drawer.open_or_switch(Some("prod-ctx"), target.clone(), ResourceDrawerTabId::Yaml);
        assert!(yaml_cmd.is_some());
        assert!(drawer.is_open);
        assert_eq!(drawer.target, Some(target));

        let svc = make_test_service("api-svc", "prod");
        let entries = views::services::build_service_context_menu_entries(
            &svc,
            |_, _| {},
            |_, _| {},
            |_, _| {},
            |_, _| {},
            |_, _| {},
            |_, _| {},
            |_, _| {},
        );
        let items: Vec<&crate::components::ContextMenuItem> = entries
            .iter()
            .filter_map(|e| match e {
                ContextMenuEntry::Item(item) => Some(item),
                ContextMenuEntry::Separator => None,
            })
            .collect();
        let delete_item = items.iter().find(|i| i.id == "delete-service").unwrap();
        assert!(!delete_item.disabled);
        assert!(delete_item.danger);
    }

    fn make_test_configmap(
        name: &str,
        namespace: &str,
    ) -> orbit_core::kubernetes::models::ConfigMapInfo {
        orbit_core::kubernetes::models::ConfigMapInfo {
            name: name.to_string(),
            namespace: namespace.to_string(),
            labels: Default::default(),
            annotations: 0,
            created: String::new(),
            age: "1h".to_string(),
            resource_version: "1".to_string(),
            immutable: false,
            keys_count: 1,
            size: "32 B".to_string(),
            mounted_pods: 1,
            used_by: Vec::new(),
            data: Default::default(),
        }
    }

    #[test]
    fn test_selecting_configmaps_view_triggers_load_configmaps_command() {
        let mut switcher = ClusterSwitcherState::new();
        switcher.connection_state = ClusterConnectionState::Connected {
            cluster_id: "prod-ctx".to_string(),
            display_name: "prod".to_string(),
            k8s_version: None,
        };
        let mut sidebar = SidebarLayoutState::new();
        let mut namespaces = NamespaceFilterState::default();
        let mut deployments = DeploymentsState::default();
        let mut statefulsets = StatefulSetsState::default();
        let mut daemonsets = DaemonSetsState::default();
        let mut jobs = JobsState::default();
        let mut cronjobs = CronJobsState::default();
        let mut pods = PodsState::default();
        let mut services = ServicesState::default();
        let mut configmaps = ConfigMapsState::default();
        let mut nodes = NodesState::default();
        let mut events = EventsState::default();

        let cmd = select_workspace_view_with_state(
            &switcher,
            &mut sidebar,
            &mut WorkloadsOverviewState::default(),
            &mut namespaces,
            &mut deployments,
            &mut statefulsets,
            &mut daemonsets,
            &mut ReplicaSetsState::default(),
            &mut jobs,
            &mut cronjobs,
            &mut pods,
            &mut services,
            &mut IngressesState::default(),
            &mut configmaps,
            &mut SecretsState::default(),
            &mut PersistentVolumesState::default(),
            &mut PersistentVolumeClaimsState::default(),
            &mut NetworkPoliciesState::default(),
            &mut StorageOverviewState::default(),
            &mut PodSecurityState::default(),
            &mut ResourceQuotasState::default(),
            &mut ClusterListViews::default(),
            &mut nodes,
            &mut events,
            WorkspaceView::ConfigMaps,
        );
        assert_eq!(
            cmd,
            Some(ClusterBridgeCommand::LoadConfigMaps {
                cluster_id: "prod-ctx".to_string(),
                generation: 1,
            })
        );
        assert_eq!(sidebar.active_view, WorkspaceView::ConfigMaps);
        assert_eq!(
            configmaps.status,
            crate::state::ConfigMapsLoadStatus::Loading
        );
    }

    #[test]
    fn test_configmaps_loaded_and_failed_events_update_state() {
        let mut switcher = ClusterSwitcherState::new();
        let mut sidebar = SidebarLayoutState::new();
        let mut overview = ClusterOverviewState::default();
        let mut namespaces = NamespaceFilterState::default();
        let mut namespaces_view = NamespacesViewState::default();
        let mut deployments = DeploymentsState::default();
        let mut statefulsets = StatefulSetsState::default();
        let mut daemonsets = DaemonSetsState::default();
        let mut jobs = JobsState::default();
        let mut cronjobs = CronJobsState::default();
        let mut pods = PodsState::default();
        let mut services = ServicesState::default();
        let mut configmaps = ConfigMapsState::default();
        let mut nodes = NodesState::default();
        let mut events = EventsState::default();

        let _ = configmaps.begin_load_for_cluster("prod-ctx");
        let cm = make_test_configmap("app-config", "prod");

        // Stale generation is ignored
        apply_cluster_event_with_workspace(
            &mut switcher,
            &mut sidebar,
            &mut overview,
            &mut WorkloadsOverviewState::default(),
            &mut namespaces,
            &mut namespaces_view,
            &mut deployments,
            &mut statefulsets,
            &mut daemonsets,
            &mut ReplicaSetsState::default(),
            &mut jobs,
            &mut cronjobs,
            &mut pods,
            &mut services,
            &mut IngressesState::default(),
            &mut configmaps,
            &mut SecretsState::default(),
            &mut PersistentVolumesState::default(),
            &mut PersistentVolumeClaimsState::default(),
            &mut NetworkPoliciesState::default(),
            &mut StorageOverviewState::default(),
            &mut PodSecurityState::default(),
            &mut ResourceQuotasState::default(),
            &mut ClusterListViews::default(),
            &mut nodes,
            &mut events,
            ClusterBridgeEvent::ConfigMapsLoaded {
                cluster_id: "prod-ctx".to_string(),
                generation: 99,
                configmaps: vec![cm.clone()],
            },
        );
        assert!(configmaps.configmaps.is_empty());

        // Matching generation updates state
        apply_cluster_event_with_workspace(
            &mut switcher,
            &mut sidebar,
            &mut overview,
            &mut WorkloadsOverviewState::default(),
            &mut namespaces,
            &mut namespaces_view,
            &mut deployments,
            &mut statefulsets,
            &mut daemonsets,
            &mut ReplicaSetsState::default(),
            &mut jobs,
            &mut cronjobs,
            &mut pods,
            &mut services,
            &mut IngressesState::default(),
            &mut configmaps,
            &mut SecretsState::default(),
            &mut PersistentVolumesState::default(),
            &mut PersistentVolumeClaimsState::default(),
            &mut NetworkPoliciesState::default(),
            &mut StorageOverviewState::default(),
            &mut PodSecurityState::default(),
            &mut ResourceQuotasState::default(),
            &mut ClusterListViews::default(),
            &mut nodes,
            &mut events,
            ClusterBridgeEvent::ConfigMapsLoaded {
                cluster_id: "prod-ctx".to_string(),
                generation: 1,
                configmaps: vec![cm],
            },
        );
        assert_eq!(configmaps.configmaps.len(), 1);
        assert_eq!(
            configmaps.status,
            crate::state::ConfigMapsLoadStatus::Loaded
        );

        let _ = configmaps.begin_refresh();
        apply_cluster_event_with_workspace(
            &mut switcher,
            &mut sidebar,
            &mut overview,
            &mut WorkloadsOverviewState::default(),
            &mut namespaces,
            &mut namespaces_view,
            &mut deployments,
            &mut statefulsets,
            &mut daemonsets,
            &mut ReplicaSetsState::default(),
            &mut jobs,
            &mut cronjobs,
            &mut pods,
            &mut services,
            &mut IngressesState::default(),
            &mut configmaps,
            &mut SecretsState::default(),
            &mut PersistentVolumesState::default(),
            &mut PersistentVolumeClaimsState::default(),
            &mut NetworkPoliciesState::default(),
            &mut StorageOverviewState::default(),
            &mut PodSecurityState::default(),
            &mut ResourceQuotasState::default(),
            &mut ClusterListViews::default(),
            &mut nodes,
            &mut events,
            ClusterBridgeEvent::ConfigMapsLoadFailed {
                cluster_id: "prod-ctx".to_string(),
                generation: 2,
                message: "forbidden".to_string(),
            },
        );
        assert_eq!(configmaps.status, crate::state::ConfigMapsLoadStatus::Error);
        assert_eq!(configmaps.error_message.as_deref(), Some("forbidden"));
    }

    #[test]
    fn test_apply_cluster_event_resets_configmaps_on_connect_and_disconnect() {
        use orbit_core::kubernetes::manager::ClusterSwitchOutcome;
        use orbit_core::kubernetes::models::UserProfileInfo;

        let mut switcher = ClusterSwitcherState::new();
        let mut sidebar = SidebarLayoutState::new();
        sidebar.select_view(WorkspaceView::ConfigMaps);
        let mut overview = ClusterOverviewState::default();
        let mut namespaces = NamespaceFilterState::default();
        let mut namespaces_view = NamespacesViewState::default();
        let mut deployments = DeploymentsState::default();
        let mut statefulsets = StatefulSetsState::default();
        let mut daemonsets = DaemonSetsState::default();
        let mut jobs = JobsState::default();
        let mut cronjobs = CronJobsState::default();
        let mut pods = PodsState::default();
        let mut services = ServicesState::default();
        let mut configmaps = ConfigMapsState::default();
        let mut nodes = NodesState::default();
        let mut events = EventsState::default();

        let connect_event = ClusterBridgeEvent::ClusterConnected {
            outcome: ClusterSwitchOutcome {
                context_name: "prod-ctx".to_string(),
                healthy: true,
                k8s_version: Some("v1.31.1".to_string()),
                error_message: None,
            },
            clusters: vec![],
            profile: UserProfileInfo {
                active_context: Some("prod-ctx".to_string()),
                user_name: None,
                auth_type: "None".to_string(),
                cluster_name: Some("prod".to_string()),
                server_url: Some("https://10.0.0.10:6443".to_string()),
                kubeconfig_paths: vec![],
                k8s_version: Some("v1.31.1".to_string()),
            },
        };

        let cmds = apply_cluster_event_with_workspace(
            &mut switcher,
            &mut sidebar,
            &mut overview,
            &mut WorkloadsOverviewState::default(),
            &mut namespaces,
            &mut namespaces_view,
            &mut deployments,
            &mut statefulsets,
            &mut daemonsets,
            &mut ReplicaSetsState::default(),
            &mut jobs,
            &mut cronjobs,
            &mut pods,
            &mut services,
            &mut IngressesState::default(),
            &mut configmaps,
            &mut SecretsState::default(),
            &mut PersistentVolumesState::default(),
            &mut PersistentVolumeClaimsState::default(),
            &mut NetworkPoliciesState::default(),
            &mut StorageOverviewState::default(),
            &mut PodSecurityState::default(),
            &mut ResourceQuotasState::default(),
            &mut ClusterListViews::default(),
            &mut nodes,
            &mut events,
            connect_event,
        );
        assert!(cmds.contains(&ClusterBridgeCommand::LoadConfigMaps {
            cluster_id: "prod-ctx".to_string(),
            generation: 1,
        }));

        apply_cluster_event_with_workspace(
            &mut switcher,
            &mut sidebar,
            &mut overview,
            &mut WorkloadsOverviewState::default(),
            &mut namespaces,
            &mut namespaces_view,
            &mut deployments,
            &mut statefulsets,
            &mut daemonsets,
            &mut ReplicaSetsState::default(),
            &mut jobs,
            &mut cronjobs,
            &mut pods,
            &mut services,
            &mut IngressesState::default(),
            &mut configmaps,
            &mut SecretsState::default(),
            &mut PersistentVolumesState::default(),
            &mut PersistentVolumeClaimsState::default(),
            &mut NetworkPoliciesState::default(),
            &mut StorageOverviewState::default(),
            &mut PodSecurityState::default(),
            &mut ResourceQuotasState::default(),
            &mut ClusterListViews::default(),
            &mut nodes,
            &mut events,
            ClusterBridgeEvent::ClusterUnreachable {
                outcome: ClusterSwitchOutcome {
                    context_name: "prod-ctx".to_string(),
                    healthy: false,
                    k8s_version: None,
                    error_message: Some("timeout".to_string()),
                },
                clusters: vec![],
                profile: UserProfileInfo {
                    active_context: Some("prod-ctx".to_string()),
                    user_name: None,
                    auth_type: "None".to_string(),
                    cluster_name: None,
                    server_url: None,
                    kubeconfig_paths: vec![],
                    k8s_version: None,
                },
            },
        );
        assert_eq!(configmaps.status, crate::state::ConfigMapsLoadStatus::Idle);
        assert!(configmaps.cluster_id.is_none());
        assert!(configmaps.configmaps.is_empty());
    }

    #[test]
    fn test_configmap_drawer_target_and_context_menu_integration() {
        let mut drawer = ResourceDrawerState::default();
        let target = ResourceDrawerTarget::ConfigMap {
            name: "app-config".to_string(),
            namespace: "prod".to_string(),
        };
        let yaml_cmd =
            drawer.open_or_switch(Some("prod-ctx"), target.clone(), ResourceDrawerTabId::Yaml);
        assert!(yaml_cmd.is_some());
        assert!(drawer.is_open);
        assert_eq!(drawer.target, Some(target));
        let _ = drawer.select_tab(
            Some("prod-ctx"),
            ResourceDrawerTabId::Custom(SharedString::from("data")),
        );
        assert_eq!(
            drawer.active_tab,
            ResourceDrawerTabId::Custom(SharedString::from("data"))
        );

        let cm = make_test_configmap("app-config", "prod");
        let entries = views::configmaps::build_configmap_context_menu_entries(
            &cm,
            |_, _| {},
            |_, _| {},
            |_, _| {},
            |_, _| {},
            |_, _| {},
            |_, _| {},
        );
        let items: Vec<&crate::components::ContextMenuItem> = entries
            .iter()
            .filter_map(|e| match e {
                ContextMenuEntry::Item(item) => Some(item),
                ContextMenuEntry::Separator => None,
            })
            .collect();
        let delete_item = items.iter().find(|i| i.id == "delete-configmap").unwrap();
        assert!(!delete_item.disabled);
        assert!(delete_item.danger);
    }

    fn make_test_secret(name: &str, namespace: &str) -> orbit_core::kubernetes::models::SecretInfo {
        let mut data = std::collections::BTreeMap::new();
        data.insert("password".to_string(), "c2VjcmV0".to_string());
        orbit_core::kubernetes::models::SecretInfo {
            name: name.to_string(),
            namespace: namespace.to_string(),
            labels: Default::default(),
            annotations: 0,
            created: String::new(),
            age: "1h".to_string(),
            resource_version: "1".to_string(),
            r#type: "Opaque".to_string(),
            immutable: false,
            keys_count: 1,
            size: "14 B".to_string(),
            mounted_pods: 1,
            used_by: Vec::new(),
            data,
        }
    }

    #[test]
    fn test_selecting_secrets_view_triggers_load_secrets_command() {
        let mut switcher = ClusterSwitcherState::new();
        switcher.connection_state = ClusterConnectionState::Connected {
            cluster_id: "prod-ctx".to_string(),
            display_name: "prod".to_string(),
            k8s_version: None,
        };
        let mut sidebar = SidebarLayoutState::new();
        let mut namespaces = NamespaceFilterState::default();
        let mut deployments = DeploymentsState::default();
        let mut statefulsets = StatefulSetsState::default();
        let mut daemonsets = DaemonSetsState::default();
        let mut jobs = JobsState::default();
        let mut cronjobs = CronJobsState::default();
        let mut pods = PodsState::default();
        let mut services = ServicesState::default();
        let mut configmaps = ConfigMapsState::default();
        let mut secrets = SecretsState::default();
        let mut persistent_volumes = PersistentVolumesState::default();
        let mut nodes = NodesState::default();
        let mut events = EventsState::default();

        let cmd = select_workspace_view_with_state(
            &switcher,
            &mut sidebar,
            &mut WorkloadsOverviewState::default(),
            &mut namespaces,
            &mut deployments,
            &mut statefulsets,
            &mut daemonsets,
            &mut ReplicaSetsState::default(),
            &mut jobs,
            &mut cronjobs,
            &mut pods,
            &mut services,
            &mut IngressesState::default(),
            &mut configmaps,
            &mut secrets,
            &mut persistent_volumes,
            &mut PersistentVolumeClaimsState::default(),
            &mut NetworkPoliciesState::default(),
            &mut StorageOverviewState::default(),
            &mut PodSecurityState::default(),
            &mut ResourceQuotasState::default(),
            &mut ClusterListViews::default(),
            &mut nodes,
            &mut events,
            WorkspaceView::Secrets,
        );
        assert_eq!(
            cmd,
            Some(ClusterBridgeCommand::LoadSecrets {
                cluster_id: "prod-ctx".to_string(),
                generation: 1,
            })
        );
        assert_eq!(sidebar.active_view, WorkspaceView::Secrets);
        assert_eq!(secrets.status, crate::state::SecretsLoadStatus::Loading);
    }

    #[test]
    fn test_secrets_loaded_and_failed_events_update_state() {
        let mut switcher = ClusterSwitcherState::new();
        let mut sidebar = SidebarLayoutState::new();
        let mut overview = ClusterOverviewState::default();
        let mut namespaces = NamespaceFilterState::default();
        let mut namespaces_view = NamespacesViewState::default();
        let mut deployments = DeploymentsState::default();
        let mut statefulsets = StatefulSetsState::default();
        let mut daemonsets = DaemonSetsState::default();
        let mut jobs = JobsState::default();
        let mut cronjobs = CronJobsState::default();
        let mut pods = PodsState::default();
        let mut services = ServicesState::default();
        let mut configmaps = ConfigMapsState::default();
        let mut secrets = SecretsState::default();
        let mut persistent_volumes = PersistentVolumesState::default();
        let mut nodes = NodesState::default();
        let mut events = EventsState::default();

        let _ = secrets.begin_load_for_cluster("prod-ctx");
        let sec = make_test_secret("db-creds", "prod");

        // Stale generation is ignored
        apply_cluster_event_with_workspace(
            &mut switcher,
            &mut sidebar,
            &mut overview,
            &mut WorkloadsOverviewState::default(),
            &mut namespaces,
            &mut namespaces_view,
            &mut deployments,
            &mut statefulsets,
            &mut daemonsets,
            &mut ReplicaSetsState::default(),
            &mut jobs,
            &mut cronjobs,
            &mut pods,
            &mut services,
            &mut IngressesState::default(),
            &mut configmaps,
            &mut secrets,
            &mut persistent_volumes,
            &mut PersistentVolumeClaimsState::default(),
            &mut NetworkPoliciesState::default(),
            &mut StorageOverviewState::default(),
            &mut PodSecurityState::default(),
            &mut ResourceQuotasState::default(),
            &mut ClusterListViews::default(),
            &mut nodes,
            &mut events,
            ClusterBridgeEvent::SecretsLoaded {
                cluster_id: "prod-ctx".to_string(),
                generation: 99,
                secrets: vec![sec.clone()],
            },
        );
        assert!(secrets.secrets.is_empty());

        // Matching generation updates state
        apply_cluster_event_with_workspace(
            &mut switcher,
            &mut sidebar,
            &mut overview,
            &mut WorkloadsOverviewState::default(),
            &mut namespaces,
            &mut namespaces_view,
            &mut deployments,
            &mut statefulsets,
            &mut daemonsets,
            &mut ReplicaSetsState::default(),
            &mut jobs,
            &mut cronjobs,
            &mut pods,
            &mut services,
            &mut IngressesState::default(),
            &mut configmaps,
            &mut secrets,
            &mut persistent_volumes,
            &mut PersistentVolumeClaimsState::default(),
            &mut NetworkPoliciesState::default(),
            &mut StorageOverviewState::default(),
            &mut PodSecurityState::default(),
            &mut ResourceQuotasState::default(),
            &mut ClusterListViews::default(),
            &mut nodes,
            &mut events,
            ClusterBridgeEvent::SecretsLoaded {
                cluster_id: "prod-ctx".to_string(),
                generation: 1,
                secrets: vec![sec],
            },
        );
        assert_eq!(secrets.secrets.len(), 1);
        assert_eq!(secrets.status, crate::state::SecretsLoadStatus::Loaded);

        let _ = secrets.begin_refresh();
        apply_cluster_event_with_workspace(
            &mut switcher,
            &mut sidebar,
            &mut overview,
            &mut WorkloadsOverviewState::default(),
            &mut namespaces,
            &mut namespaces_view,
            &mut deployments,
            &mut statefulsets,
            &mut daemonsets,
            &mut ReplicaSetsState::default(),
            &mut jobs,
            &mut cronjobs,
            &mut pods,
            &mut services,
            &mut IngressesState::default(),
            &mut configmaps,
            &mut secrets,
            &mut persistent_volumes,
            &mut PersistentVolumeClaimsState::default(),
            &mut NetworkPoliciesState::default(),
            &mut StorageOverviewState::default(),
            &mut PodSecurityState::default(),
            &mut ResourceQuotasState::default(),
            &mut ClusterListViews::default(),
            &mut nodes,
            &mut events,
            ClusterBridgeEvent::SecretsLoadFailed {
                cluster_id: "prod-ctx".to_string(),
                generation: 2,
                message: "forbidden".to_string(),
            },
        );
        assert_eq!(secrets.status, crate::state::SecretsLoadStatus::Error);
        assert_eq!(secrets.error_message.as_deref(), Some("forbidden"));
    }

    #[test]
    fn test_apply_cluster_event_resets_secrets_on_connect_and_disconnect() {
        use orbit_core::kubernetes::manager::ClusterSwitchOutcome;
        use orbit_core::kubernetes::models::UserProfileInfo;

        let mut switcher = ClusterSwitcherState::new();
        let mut sidebar = SidebarLayoutState::new();
        sidebar.select_view(WorkspaceView::Secrets);
        let mut overview = ClusterOverviewState::default();
        let mut namespaces = NamespaceFilterState::default();
        let mut namespaces_view = NamespacesViewState::default();
        let mut deployments = DeploymentsState::default();
        let mut statefulsets = StatefulSetsState::default();
        let mut daemonsets = DaemonSetsState::default();
        let mut jobs = JobsState::default();
        let mut cronjobs = CronJobsState::default();
        let mut pods = PodsState::default();
        let mut services = ServicesState::default();
        let mut configmaps = ConfigMapsState::default();
        let mut secrets = SecretsState::default();
        let mut persistent_volumes = PersistentVolumesState::default();
        let mut nodes = NodesState::default();
        let mut events = EventsState::default();

        let connect_event = ClusterBridgeEvent::ClusterConnected {
            outcome: ClusterSwitchOutcome {
                context_name: "prod-ctx".to_string(),
                healthy: true,
                k8s_version: Some("v1.31.1".to_string()),
                error_message: None,
            },
            clusters: vec![],
            profile: UserProfileInfo {
                active_context: Some("prod-ctx".to_string()),
                user_name: None,
                auth_type: "None".to_string(),
                cluster_name: Some("prod".to_string()),
                server_url: Some("https://10.0.0.10:6443".to_string()),
                kubeconfig_paths: vec![],
                k8s_version: Some("v1.31.1".to_string()),
            },
        };

        let cmds = apply_cluster_event_with_workspace(
            &mut switcher,
            &mut sidebar,
            &mut overview,
            &mut WorkloadsOverviewState::default(),
            &mut namespaces,
            &mut namespaces_view,
            &mut deployments,
            &mut statefulsets,
            &mut daemonsets,
            &mut ReplicaSetsState::default(),
            &mut jobs,
            &mut cronjobs,
            &mut pods,
            &mut services,
            &mut IngressesState::default(),
            &mut configmaps,
            &mut secrets,
            &mut persistent_volumes,
            &mut PersistentVolumeClaimsState::default(),
            &mut NetworkPoliciesState::default(),
            &mut StorageOverviewState::default(),
            &mut PodSecurityState::default(),
            &mut ResourceQuotasState::default(),
            &mut ClusterListViews::default(),
            &mut nodes,
            &mut events,
            connect_event,
        );
        assert!(cmds.contains(&ClusterBridgeCommand::LoadSecrets {
            cluster_id: "prod-ctx".to_string(),
            generation: 1,
        }));

        apply_cluster_event_with_workspace(
            &mut switcher,
            &mut sidebar,
            &mut overview,
            &mut WorkloadsOverviewState::default(),
            &mut namespaces,
            &mut namespaces_view,
            &mut deployments,
            &mut statefulsets,
            &mut daemonsets,
            &mut ReplicaSetsState::default(),
            &mut jobs,
            &mut cronjobs,
            &mut pods,
            &mut services,
            &mut IngressesState::default(),
            &mut configmaps,
            &mut secrets,
            &mut persistent_volumes,
            &mut PersistentVolumeClaimsState::default(),
            &mut NetworkPoliciesState::default(),
            &mut StorageOverviewState::default(),
            &mut PodSecurityState::default(),
            &mut ResourceQuotasState::default(),
            &mut ClusterListViews::default(),
            &mut nodes,
            &mut events,
            ClusterBridgeEvent::ClusterUnreachable {
                outcome: ClusterSwitchOutcome {
                    context_name: "prod-ctx".to_string(),
                    healthy: false,
                    k8s_version: None,
                    error_message: Some("timeout".to_string()),
                },
                clusters: vec![],
                profile: UserProfileInfo {
                    active_context: Some("prod-ctx".to_string()),
                    user_name: None,
                    auth_type: "None".to_string(),
                    cluster_name: None,
                    server_url: None,
                    kubeconfig_paths: vec![],
                    k8s_version: None,
                },
            },
        );
        assert_eq!(secrets.status, crate::state::SecretsLoadStatus::Idle);
        assert!(secrets.cluster_id.is_none());
        assert!(secrets.secrets.is_empty());
    }

    #[test]
    fn test_secret_drawer_target_reveal_toggle_and_context_menu_integration() {
        let mut drawer = ResourceDrawerState::default();
        let target = ResourceDrawerTarget::Secret {
            name: "db-creds".to_string(),
            namespace: "prod".to_string(),
        };
        let yaml_cmd =
            drawer.open_or_switch(Some("prod-ctx"), target.clone(), ResourceDrawerTabId::Yaml);
        assert!(yaml_cmd.is_some());
        assert!(drawer.is_open);
        assert_eq!(drawer.target, Some(target));

        let _ = drawer.select_tab(
            Some("prod-ctx"),
            ResourceDrawerTabId::Custom(SharedString::from("data")),
        );
        assert_eq!(
            drawer.active_tab,
            ResourceDrawerTabId::Custom(SharedString::from("data"))
        );

        assert!(!drawer.is_secret_key_revealed("password"));
        drawer.toggle_secret_key_revealed("password");
        assert!(drawer.is_secret_key_revealed("password"));
        drawer.close();
        assert!(!drawer.is_secret_key_revealed("password"));

        let sec = make_test_secret("db-creds", "prod");
        let entries = views::secrets::build_secret_context_menu_entries(
            &sec,
            |_, _| {},
            |_, _| {},
            |_, _| {},
            |_, _| {},
            |_, _| {},
            |_, _| {},
        );
        let items: Vec<&crate::components::ContextMenuItem> = entries
            .iter()
            .filter_map(|e| match e {
                ContextMenuEntry::Item(item) => Some(item),
                ContextMenuEntry::Separator => None,
            })
            .collect();
        let delete_item = items.iter().find(|i| i.id == "delete-secret").unwrap();
        assert!(!delete_item.disabled);
        assert!(delete_item.danger);
    }

    #[test]
    fn test_settings_persisted_config_restores_theme_and_experience_on_launch() {
        let _guard = orbit_core::config::TestConfigDirGuard::new("gpui_settings_relaunch");
        let mut config = OrbitConfig::default();
        config.update_theme_mode("system");
        config.update_ui_style("paper");
        config.update_accent_color("emerald");
        config.update_enable_experience_features(true);
        config.save().expect("save should succeed");

        let reloaded = OrbitConfig::load();
        let restored = OrbitTheme::from_config(&reloaded, WindowAppearance::Light);

        assert_eq!(restored.mode, ThemeMode::System);
        assert!(!restored.is_dark);
        assert_eq!(restored.style.ui_style, UiStyle::Paper);
        assert_eq!(restored.accent_id, AccentColor::Emerald);
        assert!(reloaded.enable_experience_features());
    }

    fn make_test_statefulset(
        name: &str,
        namespace: &str,
    ) -> orbit_core::kubernetes::models::StatefulSetInfo {
        orbit_core::kubernetes::models::StatefulSetInfo {
            kind: "StatefulSet".to_string(),
            name: name.to_string(),
            namespace: namespace.to_string(),
            status: "Running".to_string(),
            replicas: orbit_core::kubernetes::models::Replicas {
                current: 3,
                desired: 3,
            },
            age: "1d".to_string(),
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
    fn test_selecting_statefulsets_view_triggers_load_when_connected() {
        let mut switcher = ClusterSwitcherState::new();
        switcher.connection_state = ClusterConnectionState::Connected {
            cluster_id: "prod-ctx".to_string(),
            display_name: "prod".to_string(),
            k8s_version: Some("v1.30.0".to_string()),
        };
        let mut sidebar = SidebarLayoutState::new();
        let mut namespaces = NamespaceFilterState::default();
        let mut deployments = DeploymentsState::default();
        let mut statefulsets = StatefulSetsState::default();
        let mut daemonsets = DaemonSetsState::default();
        let mut jobs = JobsState::default();
        let mut cronjobs = CronJobsState::default();
        let mut pods = PodsState::default();
        let mut services = ServicesState::default();
        let mut configmaps = ConfigMapsState::default();
        let mut secrets = SecretsState::default();
        let mut persistent_volumes = PersistentVolumesState::default();
        let mut nodes = NodesState::default();
        let mut events = EventsState::default();

        let cmd = select_workspace_view_with_state(
            &switcher,
            &mut sidebar,
            &mut WorkloadsOverviewState::default(),
            &mut namespaces,
            &mut deployments,
            &mut statefulsets,
            &mut daemonsets,
            &mut ReplicaSetsState::default(),
            &mut jobs,
            &mut cronjobs,
            &mut pods,
            &mut services,
            &mut IngressesState::default(),
            &mut configmaps,
            &mut secrets,
            &mut persistent_volumes,
            &mut PersistentVolumeClaimsState::default(),
            &mut NetworkPoliciesState::default(),
            &mut StorageOverviewState::default(),
            &mut PodSecurityState::default(),
            &mut ResourceQuotasState::default(),
            &mut ClusterListViews::default(),
            &mut nodes,
            &mut events,
            WorkspaceView::StatefulSets,
        );

        assert_eq!(sidebar.active_view, WorkspaceView::StatefulSets);
        assert_eq!(
            cmd,
            Some(ClusterBridgeCommand::LoadStatefulSets {
                cluster_id: "prod-ctx".to_string(),
                generation: 1,
            })
        );
    }

    #[test]
    fn test_statefulsets_loaded_and_failed_events_update_state() {
        let mut switcher = ClusterSwitcherState::new();
        let mut sidebar = SidebarLayoutState::new();
        let mut overview = ClusterOverviewState::default();
        let mut namespaces = NamespaceFilterState::default();
        let mut namespaces_view = NamespacesViewState::default();
        let mut deployments = DeploymentsState::default();
        let mut statefulsets = StatefulSetsState::default();
        let mut daemonsets = DaemonSetsState::default();
        let mut jobs = JobsState::default();
        let mut cronjobs = CronJobsState::default();
        let mut pods = PodsState::default();
        let mut services = ServicesState::default();
        let mut configmaps = ConfigMapsState::default();
        let mut secrets = SecretsState::default();
        let mut persistent_volumes = PersistentVolumesState::default();
        let mut nodes = NodesState::default();
        let mut events = EventsState::default();

        let _ = statefulsets.begin_load_for_cluster("prod-ctx");
        let ss = make_test_statefulset("redis", "cache");

        apply_cluster_event_with_workspace(
            &mut switcher,
            &mut sidebar,
            &mut overview,
            &mut WorkloadsOverviewState::default(),
            &mut namespaces,
            &mut namespaces_view,
            &mut deployments,
            &mut statefulsets,
            &mut daemonsets,
            &mut ReplicaSetsState::default(),
            &mut jobs,
            &mut cronjobs,
            &mut pods,
            &mut services,
            &mut IngressesState::default(),
            &mut configmaps,
            &mut secrets,
            &mut persistent_volumes,
            &mut PersistentVolumeClaimsState::default(),
            &mut NetworkPoliciesState::default(),
            &mut StorageOverviewState::default(),
            &mut PodSecurityState::default(),
            &mut ResourceQuotasState::default(),
            &mut ClusterListViews::default(),
            &mut nodes,
            &mut events,
            ClusterBridgeEvent::StatefulSetsLoaded {
                cluster_id: "prod-ctx".to_string(),
                generation: 1,
                statefulsets: vec![ss],
            },
        );
        assert_eq!(statefulsets.statefulsets.len(), 1);
        assert_eq!(
            statefulsets.status,
            crate::state::StatefulSetsLoadStatus::Loaded
        );

        let _ = statefulsets.begin_refresh();
        apply_cluster_event_with_workspace(
            &mut switcher,
            &mut sidebar,
            &mut overview,
            &mut WorkloadsOverviewState::default(),
            &mut namespaces,
            &mut namespaces_view,
            &mut deployments,
            &mut statefulsets,
            &mut daemonsets,
            &mut ReplicaSetsState::default(),
            &mut jobs,
            &mut cronjobs,
            &mut pods,
            &mut services,
            &mut IngressesState::default(),
            &mut configmaps,
            &mut secrets,
            &mut persistent_volumes,
            &mut PersistentVolumeClaimsState::default(),
            &mut NetworkPoliciesState::default(),
            &mut StorageOverviewState::default(),
            &mut PodSecurityState::default(),
            &mut ResourceQuotasState::default(),
            &mut ClusterListViews::default(),
            &mut nodes,
            &mut events,
            ClusterBridgeEvent::StatefulSetsLoadFailed {
                cluster_id: "prod-ctx".to_string(),
                generation: 2,
                message: "forbidden".to_string(),
            },
        );
        assert_eq!(
            statefulsets.status,
            crate::state::StatefulSetsLoadStatus::Error
        );
        assert_eq!(statefulsets.error_message.as_deref(), Some("forbidden"));
    }

    #[test]
    fn test_connecting_while_on_statefulsets_view_dispatches_load_statefulsets() {
        use orbit_core::kubernetes::manager::ClusterSwitchOutcome;
        use orbit_core::kubernetes::models::UserProfileInfo;

        let mut switcher = ClusterSwitcherState::new();
        let mut sidebar = SidebarLayoutState::new();
        sidebar.select_view(WorkspaceView::StatefulSets);
        let mut overview = ClusterOverviewState::default();
        let mut namespaces = NamespaceFilterState::default();
        let mut namespaces_view = NamespacesViewState::default();
        let mut deployments = DeploymentsState::default();
        let mut statefulsets = StatefulSetsState::default();
        let mut daemonsets = DaemonSetsState::default();
        let mut jobs = JobsState::default();
        let mut cronjobs = CronJobsState::default();
        let mut pods = PodsState::default();
        let mut services = ServicesState::default();
        let mut configmaps = ConfigMapsState::default();
        let mut secrets = SecretsState::default();
        let mut persistent_volumes = PersistentVolumesState::default();
        let mut nodes = NodesState::default();
        let mut events = EventsState::default();

        let cmds = apply_cluster_event_with_workspace(
            &mut switcher,
            &mut sidebar,
            &mut overview,
            &mut WorkloadsOverviewState::default(),
            &mut namespaces,
            &mut namespaces_view,
            &mut deployments,
            &mut statefulsets,
            &mut daemonsets,
            &mut ReplicaSetsState::default(),
            &mut jobs,
            &mut cronjobs,
            &mut pods,
            &mut services,
            &mut IngressesState::default(),
            &mut configmaps,
            &mut secrets,
            &mut persistent_volumes,
            &mut PersistentVolumeClaimsState::default(),
            &mut NetworkPoliciesState::default(),
            &mut StorageOverviewState::default(),
            &mut PodSecurityState::default(),
            &mut ResourceQuotasState::default(),
            &mut ClusterListViews::default(),
            &mut nodes,
            &mut events,
            ClusterBridgeEvent::ClusterConnected {
                outcome: ClusterSwitchOutcome {
                    context_name: "prod-ctx".to_string(),
                    healthy: true,
                    k8s_version: Some("v1.30.0".to_string()),
                    error_message: None,
                },
                clusters: Vec::new(),
                profile: UserProfileInfo {
                    active_context: Some("prod-ctx".to_string()),
                    user_name: None,
                    auth_type: "None".to_string(),
                    cluster_name: Some("prod".to_string()),
                    server_url: Some("https://10.0.0.10:6443".to_string()),
                    kubeconfig_paths: vec![],
                    k8s_version: Some("v1.30.0".to_string()),
                },
            },
        );

        assert!(cmds.iter().any(|c| matches!(
            c,
            ClusterBridgeCommand::LoadStatefulSets { cluster_id, .. } if cluster_id == "prod-ctx"
        )));
    }

    #[test]
    fn test_selecting_workloads_overview_dispatches_load_and_applies_snapshot_lifecycle() {
        use crate::state::WorkloadsOverviewLoadStatus;
        use orbit_core::kubernetes::manager::ClusterSwitchOutcome;
        use orbit_core::kubernetes::models::UserProfileInfo;
        use orbit_core::kubernetes::workloads_overview::{
            WorkloadHealthItem, WorkloadHealthStatus, WorkloadKindSnapshot,
            WorkloadsOverviewSnapshot,
        };

        let mut switcher = ClusterSwitcherState::new();
        switcher.connection_state = ClusterConnectionState::Connected {
            cluster_id: "prod-ctx".to_string(),
            display_name: "prod".to_string(),
            k8s_version: Some("v1.31.0".to_string()),
        };
        switcher.active_cluster_id = Some("prod-ctx".to_string());

        let mut sidebar = SidebarLayoutState::new();
        let mut overview = ClusterOverviewState::default();
        let mut workloads_overview = WorkloadsOverviewState::default();
        let mut namespaces = NamespaceFilterState::default();
        let mut namespaces_view = NamespacesViewState::default();
        let mut deployments = DeploymentsState::default();
        let mut statefulsets = StatefulSetsState::default();
        let mut daemonsets = DaemonSetsState::default();
        let mut jobs = JobsState::default();
        let mut cronjobs = CronJobsState::default();
        let mut pods = PodsState::default();
        let mut services = ServicesState::default();
        let mut configmaps = ConfigMapsState::default();
        let mut secrets = SecretsState::default();
        let mut persistent_volumes = PersistentVolumesState::default();
        let mut nodes = NodesState::default();
        let mut events = EventsState::default();

        // Selecting WorkloadsOverview while connected dispatches LoadWorkloadsOverview
        let cmd = select_workspace_view_with_state(
            &switcher,
            &mut sidebar,
            &mut workloads_overview,
            &mut namespaces,
            &mut deployments,
            &mut statefulsets,
            &mut daemonsets,
            &mut ReplicaSetsState::default(),
            &mut jobs,
            &mut cronjobs,
            &mut pods,
            &mut services,
            &mut IngressesState::default(),
            &mut configmaps,
            &mut secrets,
            &mut persistent_volumes,
            &mut PersistentVolumeClaimsState::default(),
            &mut NetworkPoliciesState::default(),
            &mut StorageOverviewState::default(),
            &mut PodSecurityState::default(),
            &mut ResourceQuotasState::default(),
            &mut ClusterListViews::default(),
            &mut nodes,
            &mut events,
            WorkspaceView::WorkloadsOverview,
        );
        assert_eq!(
            cmd,
            Some(ClusterBridgeCommand::LoadWorkloadsOverview {
                cluster_id: "prod-ctx".to_string(),
                generation: 1,
            })
        );
        assert_eq!(
            workloads_overview.status,
            WorkloadsOverviewLoadStatus::Loading
        );

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

        // Stale generation is ignored
        apply_cluster_event_with_workspace(
            &mut switcher,
            &mut sidebar,
            &mut overview,
            &mut workloads_overview,
            &mut namespaces,
            &mut namespaces_view,
            &mut deployments,
            &mut statefulsets,
            &mut daemonsets,
            &mut ReplicaSetsState::default(),
            &mut jobs,
            &mut cronjobs,
            &mut pods,
            &mut services,
            &mut IngressesState::default(),
            &mut configmaps,
            &mut secrets,
            &mut persistent_volumes,
            &mut PersistentVolumeClaimsState::default(),
            &mut NetworkPoliciesState::default(),
            &mut StorageOverviewState::default(),
            &mut PodSecurityState::default(),
            &mut ResourceQuotasState::default(),
            &mut ClusterListViews::default(),
            &mut nodes,
            &mut events,
            ClusterBridgeEvent::WorkloadsOverviewLoaded {
                cluster_id: "prod-ctx".to_string(),
                generation: 99,
                snapshot: sample.clone(),
            },
        );
        assert!(workloads_overview.snapshot.is_none());

        // Matching generation applies snapshot
        apply_cluster_event_with_workspace(
            &mut switcher,
            &mut sidebar,
            &mut overview,
            &mut workloads_overview,
            &mut namespaces,
            &mut namespaces_view,
            &mut deployments,
            &mut statefulsets,
            &mut daemonsets,
            &mut ReplicaSetsState::default(),
            &mut jobs,
            &mut cronjobs,
            &mut pods,
            &mut services,
            &mut IngressesState::default(),
            &mut configmaps,
            &mut secrets,
            &mut persistent_volumes,
            &mut PersistentVolumeClaimsState::default(),
            &mut NetworkPoliciesState::default(),
            &mut StorageOverviewState::default(),
            &mut PodSecurityState::default(),
            &mut ResourceQuotasState::default(),
            &mut ClusterListViews::default(),
            &mut nodes,
            &mut events,
            ClusterBridgeEvent::WorkloadsOverviewLoaded {
                cluster_id: "prod-ctx".to_string(),
                generation: 1,
                snapshot: sample.clone(),
            },
        );
        assert_eq!(
            workloads_overview.status,
            WorkloadsOverviewLoadStatus::Loaded
        );
        assert_eq!(workloads_overview.snapshot.as_ref(), Some(&sample));

        // Disconnect resets workloads_overview
        apply_cluster_event_with_workspace(
            &mut switcher,
            &mut sidebar,
            &mut overview,
            &mut workloads_overview,
            &mut namespaces,
            &mut namespaces_view,
            &mut deployments,
            &mut statefulsets,
            &mut daemonsets,
            &mut ReplicaSetsState::default(),
            &mut jobs,
            &mut cronjobs,
            &mut pods,
            &mut services,
            &mut IngressesState::default(),
            &mut configmaps,
            &mut secrets,
            &mut persistent_volumes,
            &mut PersistentVolumeClaimsState::default(),
            &mut NetworkPoliciesState::default(),
            &mut StorageOverviewState::default(),
            &mut PodSecurityState::default(),
            &mut ResourceQuotasState::default(),
            &mut ClusterListViews::default(),
            &mut nodes,
            &mut events,
            ClusterBridgeEvent::ConnectionError {
                cluster_id: Some("prod-ctx".to_string()),
                message: "offline".to_string(),
                clusters: Vec::new(),
            },
        );
        assert_eq!(workloads_overview.status, WorkloadsOverviewLoadStatus::Idle);
        assert!(workloads_overview.snapshot.is_none());

        // Reconnecting while WorkloadsOverview is active triggers LoadWorkloadsOverview
        sidebar.active_view = WorkspaceView::WorkloadsOverview;
        let reconnect_cmds = apply_cluster_event_with_workspace(
            &mut switcher,
            &mut sidebar,
            &mut overview,
            &mut workloads_overview,
            &mut namespaces,
            &mut namespaces_view,
            &mut deployments,
            &mut statefulsets,
            &mut daemonsets,
            &mut ReplicaSetsState::default(),
            &mut jobs,
            &mut cronjobs,
            &mut pods,
            &mut services,
            &mut IngressesState::default(),
            &mut configmaps,
            &mut secrets,
            &mut persistent_volumes,
            &mut PersistentVolumeClaimsState::default(),
            &mut NetworkPoliciesState::default(),
            &mut StorageOverviewState::default(),
            &mut PodSecurityState::default(),
            &mut ResourceQuotasState::default(),
            &mut ClusterListViews::default(),
            &mut nodes,
            &mut events,
            ClusterBridgeEvent::ClusterConnected {
                outcome: ClusterSwitchOutcome {
                    context_name: "prod-ctx".to_string(),
                    healthy: true,
                    k8s_version: Some("v1.31.0".to_string()),
                    error_message: None,
                },
                clusters: Vec::new(),
                profile: UserProfileInfo {
                    active_context: Some("prod-ctx".to_string()),
                    user_name: None,
                    auth_type: "None".to_string(),
                    cluster_name: Some("prod".to_string()),
                    server_url: Some("https://10.0.0.10:6443".to_string()),
                    kubeconfig_paths: vec![],
                    k8s_version: Some("v1.31.0".to_string()),
                },
            },
        );
        assert!(reconnect_cmds.iter().any(|c| matches!(
            c,
            ClusterBridgeCommand::LoadWorkloadsOverview { cluster_id, .. } if cluster_id == "prod-ctx"
        )));
    }

    #[test]
    fn test_workloads_overview_card_navigation_opens_target_view_and_loads_implemented_workloads() {
        let mut switcher = ClusterSwitcherState::new();
        switcher.connection_state = ClusterConnectionState::Connected {
            cluster_id: "prod-ctx".to_string(),
            display_name: "prod".to_string(),
            k8s_version: Some("v1.31.0".to_string()),
        };
        switcher.active_cluster_id = Some("prod-ctx".to_string());

        let mut sidebar = SidebarLayoutState::new();
        let mut workloads_overview = WorkloadsOverviewState::default();
        let mut namespaces = NamespaceFilterState::default();
        let mut deployments = DeploymentsState::default();
        let mut statefulsets = StatefulSetsState::default();
        let mut daemonsets = DaemonSetsState::default();
        let mut jobs = JobsState::default();
        let mut cronjobs = CronJobsState::default();
        let mut pods = PodsState::default();
        let mut services = ServicesState::default();
        let mut configmaps = ConfigMapsState::default();
        let mut secrets = SecretsState::default();
        let mut persistent_volumes = PersistentVolumesState::default();
        let mut nodes = NodesState::default();
        let mut events = EventsState::default();
        let mut tabs = WorkspaceTabsState::default();
        tabs.activate_cluster("prod-ctx");

        // Start on WorkloadsOverview
        let _ = select_workspace_view_with_tabs(
            &switcher,
            &mut sidebar,
            &mut workloads_overview,
            &mut namespaces,
            &mut deployments,
            &mut statefulsets,
            &mut daemonsets,
            &mut ReplicaSetsState::default(),
            &mut jobs,
            &mut cronjobs,
            &mut pods,
            &mut services,
            &mut IngressesState::default(),
            &mut configmaps,
            &mut secrets,
            &mut persistent_volumes,
            &mut PersistentVolumeClaimsState::default(),
            &mut NetworkPoliciesState::default(),
            &mut StorageOverviewState::default(),
            &mut PodSecurityState::default(),
            &mut ResourceQuotasState::default(),
            &mut ClusterListViews::default(),
            &mut nodes,
            &mut events,
            &mut tabs,
            WorkspaceView::WorkloadsOverview,
        );
        assert_eq!(sidebar.active_view, WorkspaceView::WorkloadsOverview);

        // Navigate to Pods
        let pods_cmds = select_workspace_view_with_tabs(
            &switcher,
            &mut sidebar,
            &mut workloads_overview,
            &mut namespaces,
            &mut deployments,
            &mut statefulsets,
            &mut daemonsets,
            &mut ReplicaSetsState::default(),
            &mut jobs,
            &mut cronjobs,
            &mut pods,
            &mut services,
            &mut IngressesState::default(),
            &mut configmaps,
            &mut secrets,
            &mut persistent_volumes,
            &mut PersistentVolumeClaimsState::default(),
            &mut NetworkPoliciesState::default(),
            &mut StorageOverviewState::default(),
            &mut PodSecurityState::default(),
            &mut ResourceQuotasState::default(),
            &mut ClusterListViews::default(),
            &mut nodes,
            &mut events,
            &mut tabs,
            WorkspaceView::Pods,
        );
        assert_eq!(sidebar.active_view, WorkspaceView::Pods);
        assert_eq!(tabs.active_view(), Some(WorkspaceView::Pods));
        assert!(pods_cmds.iter().any(|c| matches!(
            c,
            ClusterBridgeCommand::LoadPods { cluster_id, .. } if cluster_id == "prod-ctx"
        )));

        // Navigate to Deployments
        let dep_cmds = select_workspace_view_with_tabs(
            &switcher,
            &mut sidebar,
            &mut workloads_overview,
            &mut namespaces,
            &mut deployments,
            &mut statefulsets,
            &mut daemonsets,
            &mut ReplicaSetsState::default(),
            &mut jobs,
            &mut cronjobs,
            &mut pods,
            &mut services,
            &mut IngressesState::default(),
            &mut configmaps,
            &mut secrets,
            &mut persistent_volumes,
            &mut PersistentVolumeClaimsState::default(),
            &mut NetworkPoliciesState::default(),
            &mut StorageOverviewState::default(),
            &mut PodSecurityState::default(),
            &mut ResourceQuotasState::default(),
            &mut ClusterListViews::default(),
            &mut nodes,
            &mut events,
            &mut tabs,
            WorkspaceView::Deployments,
        );
        assert_eq!(sidebar.active_view, WorkspaceView::Deployments);
        assert_eq!(tabs.active_view(), Some(WorkspaceView::Deployments));
        assert!(dep_cmds.iter().any(|c| matches!(
            c,
            ClusterBridgeCommand::LoadDeployments { cluster_id, .. } if cluster_id == "prod-ctx"
        )));

        // Navigate to StatefulSets
        let sts_cmds = select_workspace_view_with_tabs(
            &switcher,
            &mut sidebar,
            &mut workloads_overview,
            &mut namespaces,
            &mut deployments,
            &mut statefulsets,
            &mut daemonsets,
            &mut ReplicaSetsState::default(),
            &mut jobs,
            &mut cronjobs,
            &mut pods,
            &mut services,
            &mut IngressesState::default(),
            &mut configmaps,
            &mut secrets,
            &mut persistent_volumes,
            &mut PersistentVolumeClaimsState::default(),
            &mut NetworkPoliciesState::default(),
            &mut StorageOverviewState::default(),
            &mut PodSecurityState::default(),
            &mut ResourceQuotasState::default(),
            &mut ClusterListViews::default(),
            &mut nodes,
            &mut events,
            &mut tabs,
            WorkspaceView::StatefulSets,
        );
        assert_eq!(sidebar.active_view, WorkspaceView::StatefulSets);
        assert_eq!(tabs.active_view(), Some(WorkspaceView::StatefulSets));
        assert!(sts_cmds.iter().any(|c| matches!(
            c,
            ClusterBridgeCommand::LoadStatefulSets { cluster_id, .. } if cluster_id == "prod-ctx"
        )));

        // Navigate to DaemonSets
        let ds_cmds = select_workspace_view_with_tabs(
            &switcher,
            &mut sidebar,
            &mut workloads_overview,
            &mut namespaces,
            &mut deployments,
            &mut statefulsets,
            &mut daemonsets,
            &mut ReplicaSetsState::default(),
            &mut jobs,
            &mut cronjobs,
            &mut pods,
            &mut services,
            &mut IngressesState::default(),
            &mut configmaps,
            &mut secrets,
            &mut persistent_volumes,
            &mut PersistentVolumeClaimsState::default(),
            &mut NetworkPoliciesState::default(),
            &mut StorageOverviewState::default(),
            &mut PodSecurityState::default(),
            &mut ResourceQuotasState::default(),
            &mut ClusterListViews::default(),
            &mut nodes,
            &mut events,
            &mut tabs,
            WorkspaceView::DaemonSets,
        );
        assert_eq!(sidebar.active_view, WorkspaceView::DaemonSets);
        assert_eq!(tabs.active_view(), Some(WorkspaceView::DaemonSets));
        assert!(ds_cmds.iter().any(|c| matches!(
            c,
            ClusterBridgeCommand::LoadDaemonSets { cluster_id, .. } if cluster_id == "prod-ctx"
        )));

        // Navigate to Jobs
        let jobs_cmds = select_workspace_view_with_tabs(
            &switcher,
            &mut sidebar,
            &mut workloads_overview,
            &mut namespaces,
            &mut deployments,
            &mut statefulsets,
            &mut daemonsets,
            &mut ReplicaSetsState::default(),
            &mut jobs,
            &mut cronjobs,
            &mut pods,
            &mut services,
            &mut IngressesState::default(),
            &mut configmaps,
            &mut secrets,
            &mut persistent_volumes,
            &mut PersistentVolumeClaimsState::default(),
            &mut NetworkPoliciesState::default(),
            &mut StorageOverviewState::default(),
            &mut PodSecurityState::default(),
            &mut ResourceQuotasState::default(),
            &mut ClusterListViews::default(),
            &mut nodes,
            &mut events,
            &mut tabs,
            WorkspaceView::Jobs,
        );
        assert_eq!(sidebar.active_view, WorkspaceView::Jobs);
        assert_eq!(tabs.active_view(), Some(WorkspaceView::Jobs));
        assert!(jobs_cmds.iter().any(|c| matches!(
            c,
            ClusterBridgeCommand::LoadJobs { cluster_id, .. } if cluster_id == "prod-ctx"
        )));

        // Navigate to ReplicaSets
        let rs_cmds = select_workspace_view_with_tabs(
            &switcher,
            &mut sidebar,
            &mut workloads_overview,
            &mut namespaces,
            &mut deployments,
            &mut statefulsets,
            &mut daemonsets,
            &mut ReplicaSetsState::default(),
            &mut jobs,
            &mut cronjobs,
            &mut pods,
            &mut services,
            &mut IngressesState::default(),
            &mut configmaps,
            &mut secrets,
            &mut persistent_volumes,
            &mut PersistentVolumeClaimsState::default(),
            &mut NetworkPoliciesState::default(),
            &mut StorageOverviewState::default(),
            &mut PodSecurityState::default(),
            &mut ResourceQuotasState::default(),
            &mut ClusterListViews::default(),
            &mut nodes,
            &mut events,
            &mut tabs,
            WorkspaceView::ReplicaSets,
        );
        assert_eq!(sidebar.active_view, WorkspaceView::ReplicaSets);
        assert_eq!(tabs.active_view(), Some(WorkspaceView::ReplicaSets));
        assert!(rs_cmds.iter().any(|c| matches!(
            c,
            ClusterBridgeCommand::LoadReplicaSets { cluster_id, .. } if cluster_id == "prod-ctx"
        )));
    }

    fn make_test_replicaset(
        name: &str,
        namespace: &str,
    ) -> orbit_core::kubernetes::models::ReplicaSetInfo {
        orbit_core::kubernetes::models::ReplicaSetInfo {
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
    fn test_replicasets_loaded_and_failed_events_and_loading_flag() {
        let mut switcher = ClusterSwitcherState::new();
        let mut sidebar = SidebarLayoutState::new();
        let mut overview = ClusterOverviewState::default();
        let mut namespaces = NamespaceFilterState::default();
        let mut namespaces_view = NamespacesViewState::default();
        let mut deployments = DeploymentsState::default();
        let mut statefulsets = StatefulSetsState::default();
        let mut daemonsets = DaemonSetsState::default();
        let mut replicasets = ReplicaSetsState::default();
        let mut jobs = JobsState::default();
        let mut cronjobs = CronJobsState::default();
        let mut pods = PodsState::default();
        let mut services = ServicesState::default();
        let mut configmaps = ConfigMapsState::default();
        let mut secrets = SecretsState::default();
        let mut persistent_volumes = PersistentVolumesState::default();
        let mut nodes = NodesState::default();
        let mut events = EventsState::default();

        assert!(is_workspace_view_loading(
            WorkspaceView::ReplicaSets,
            WorkspaceLoadingFlags {
                replicasets: true,
                ..Default::default()
            },
        ));
        assert!(!is_workspace_view_loading(
            WorkspaceView::ReplicaSets,
            WorkspaceLoadingFlags {
                deployments: true,
                ..Default::default()
            },
        ));

        let _ = replicasets.begin_load_for_cluster("prod-ctx");
        let rs = make_test_replicaset("web-7f8b", "prod");

        apply_cluster_event_with_workspace(
            &mut switcher,
            &mut sidebar,
            &mut overview,
            &mut WorkloadsOverviewState::default(),
            &mut namespaces,
            &mut namespaces_view,
            &mut deployments,
            &mut statefulsets,
            &mut daemonsets,
            &mut replicasets,
            &mut jobs,
            &mut cronjobs,
            &mut pods,
            &mut services,
            &mut IngressesState::default(),
            &mut configmaps,
            &mut secrets,
            &mut persistent_volumes,
            &mut PersistentVolumeClaimsState::default(),
            &mut NetworkPoliciesState::default(),
            &mut StorageOverviewState::default(),
            &mut PodSecurityState::default(),
            &mut ResourceQuotasState::default(),
            &mut ClusterListViews::default(),
            &mut nodes,
            &mut events,
            ClusterBridgeEvent::ReplicaSetsLoaded {
                cluster_id: "prod-ctx".to_string(),
                generation: 1,
                replicasets: vec![rs],
            },
        );
        assert_eq!(replicasets.replicasets.len(), 1);
        assert_eq!(
            replicasets.status,
            crate::state::ReplicaSetsLoadStatus::Loaded
        );

        let _ = replicasets.begin_refresh();
        apply_cluster_event_with_workspace(
            &mut switcher,
            &mut sidebar,
            &mut overview,
            &mut WorkloadsOverviewState::default(),
            &mut namespaces,
            &mut namespaces_view,
            &mut deployments,
            &mut statefulsets,
            &mut daemonsets,
            &mut replicasets,
            &mut jobs,
            &mut cronjobs,
            &mut pods,
            &mut services,
            &mut IngressesState::default(),
            &mut configmaps,
            &mut secrets,
            &mut persistent_volumes,
            &mut PersistentVolumeClaimsState::default(),
            &mut NetworkPoliciesState::default(),
            &mut StorageOverviewState::default(),
            &mut PodSecurityState::default(),
            &mut ResourceQuotasState::default(),
            &mut ClusterListViews::default(),
            &mut nodes,
            &mut events,
            ClusterBridgeEvent::ReplicaSetsLoadFailed {
                cluster_id: "prod-ctx".to_string(),
                generation: 2,
                message: "forbidden".to_string(),
            },
        );
        assert_eq!(
            replicasets.status,
            crate::state::ReplicaSetsLoadStatus::Error
        );
        assert_eq!(replicasets.error_message.as_deref(), Some("forbidden"));
    }

    fn make_test_daemonset(
        name: &str,
        namespace: &str,
    ) -> orbit_core::kubernetes::models::DaemonSetInfo {
        orbit_core::kubernetes::models::DaemonSetInfo {
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
            age: "1d".to_string(),
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
    fn test_selecting_daemonsets_view_dispatches_load_and_applies_events() {
        let mut switcher = ClusterSwitcherState::new();
        switcher.connection_state = ClusterConnectionState::Connected {
            cluster_id: "prod-ctx".to_string(),
            display_name: "prod".to_string(),
            k8s_version: Some("v1.31.0".to_string()),
        };
        let mut sidebar = SidebarLayoutState::new();
        let mut overview = ClusterOverviewState::default();
        let mut workloads_overview = WorkloadsOverviewState::default();
        let mut namespaces = NamespaceFilterState::default();
        let mut namespaces_view = NamespacesViewState::default();
        let mut deployments = DeploymentsState::default();
        let mut statefulsets = StatefulSetsState::default();
        let mut daemonsets = DaemonSetsState::default();
        let mut jobs = JobsState::default();
        let mut cronjobs = CronJobsState::default();
        let mut pods = PodsState::default();
        let mut services = ServicesState::default();
        let mut configmaps = ConfigMapsState::default();
        let mut secrets = SecretsState::default();
        let mut persistent_volumes = PersistentVolumesState::default();
        let mut nodes = NodesState::default();
        let mut events = EventsState::default();

        let cmd = select_workspace_view_with_state(
            &switcher,
            &mut sidebar,
            &mut workloads_overview,
            &mut namespaces,
            &mut deployments,
            &mut statefulsets,
            &mut daemonsets,
            &mut ReplicaSetsState::default(),
            &mut jobs,
            &mut cronjobs,
            &mut pods,
            &mut services,
            &mut IngressesState::default(),
            &mut configmaps,
            &mut secrets,
            &mut persistent_volumes,
            &mut PersistentVolumeClaimsState::default(),
            &mut NetworkPoliciesState::default(),
            &mut StorageOverviewState::default(),
            &mut PodSecurityState::default(),
            &mut ResourceQuotasState::default(),
            &mut ClusterListViews::default(),
            &mut nodes,
            &mut events,
            WorkspaceView::DaemonSets,
        );
        assert_eq!(sidebar.active_view, WorkspaceView::DaemonSets);
        assert_eq!(
            cmd,
            Some(ClusterBridgeCommand::LoadDaemonSets {
                cluster_id: "prod-ctx".to_string(),
                generation: 1,
            })
        );

        let ds = make_test_daemonset("fluent-bit", "logging");
        apply_cluster_event_with_workspace(
            &mut switcher,
            &mut sidebar,
            &mut overview,
            &mut workloads_overview,
            &mut namespaces,
            &mut namespaces_view,
            &mut deployments,
            &mut statefulsets,
            &mut daemonsets,
            &mut ReplicaSetsState::default(),
            &mut jobs,
            &mut cronjobs,
            &mut pods,
            &mut services,
            &mut IngressesState::default(),
            &mut configmaps,
            &mut secrets,
            &mut persistent_volumes,
            &mut PersistentVolumeClaimsState::default(),
            &mut NetworkPoliciesState::default(),
            &mut StorageOverviewState::default(),
            &mut PodSecurityState::default(),
            &mut ResourceQuotasState::default(),
            &mut ClusterListViews::default(),
            &mut nodes,
            &mut events,
            ClusterBridgeEvent::DaemonSetsLoaded {
                cluster_id: "prod-ctx".to_string(),
                generation: 1,
                daemonsets: vec![ds],
            },
        );
        assert_eq!(daemonsets.daemonsets.len(), 1);
        assert_eq!(
            daemonsets.status,
            crate::state::DaemonSetsLoadStatus::Loaded
        );

        let _ = daemonsets.begin_refresh();
        apply_cluster_event_with_workspace(
            &mut switcher,
            &mut sidebar,
            &mut overview,
            &mut workloads_overview,
            &mut namespaces,
            &mut namespaces_view,
            &mut deployments,
            &mut statefulsets,
            &mut daemonsets,
            &mut ReplicaSetsState::default(),
            &mut jobs,
            &mut cronjobs,
            &mut pods,
            &mut services,
            &mut IngressesState::default(),
            &mut configmaps,
            &mut secrets,
            &mut persistent_volumes,
            &mut PersistentVolumeClaimsState::default(),
            &mut NetworkPoliciesState::default(),
            &mut StorageOverviewState::default(),
            &mut PodSecurityState::default(),
            &mut ResourceQuotasState::default(),
            &mut ClusterListViews::default(),
            &mut nodes,
            &mut events,
            ClusterBridgeEvent::DaemonSetsLoadFailed {
                cluster_id: "prod-ctx".to_string(),
                generation: 2,
                message: "forbidden".to_string(),
            },
        );
        assert_eq!(daemonsets.status, crate::state::DaemonSetsLoadStatus::Error);
        assert_eq!(daemonsets.error_message.as_deref(), Some("forbidden"));
    }

    #[test]
    fn test_connecting_while_on_daemonsets_view_dispatches_load_daemonsets() {
        use orbit_core::kubernetes::manager::ClusterSwitchOutcome;
        use orbit_core::kubernetes::models::UserProfileInfo;

        let mut switcher = ClusterSwitcherState::new();
        let mut sidebar = SidebarLayoutState::new();
        sidebar.select_view(WorkspaceView::DaemonSets);
        let mut overview = ClusterOverviewState::default();
        let mut workloads_overview = WorkloadsOverviewState::default();
        let mut namespaces = NamespaceFilterState::default();
        let mut namespaces_view = NamespacesViewState::default();
        let mut deployments = DeploymentsState::default();
        let mut statefulsets = StatefulSetsState::default();
        let mut daemonsets = DaemonSetsState::default();
        let mut jobs = JobsState::default();
        let mut cronjobs = CronJobsState::default();
        let mut pods = PodsState::default();
        let mut services = ServicesState::default();
        let mut configmaps = ConfigMapsState::default();
        let mut secrets = SecretsState::default();
        let mut persistent_volumes = PersistentVolumesState::default();
        let mut nodes = NodesState::default();
        let mut events = EventsState::default();

        let cmds = apply_cluster_event_with_workspace(
            &mut switcher,
            &mut sidebar,
            &mut overview,
            &mut workloads_overview,
            &mut namespaces,
            &mut namespaces_view,
            &mut deployments,
            &mut statefulsets,
            &mut daemonsets,
            &mut ReplicaSetsState::default(),
            &mut jobs,
            &mut cronjobs,
            &mut pods,
            &mut services,
            &mut IngressesState::default(),
            &mut configmaps,
            &mut secrets,
            &mut persistent_volumes,
            &mut PersistentVolumeClaimsState::default(),
            &mut NetworkPoliciesState::default(),
            &mut StorageOverviewState::default(),
            &mut PodSecurityState::default(),
            &mut ResourceQuotasState::default(),
            &mut ClusterListViews::default(),
            &mut nodes,
            &mut events,
            ClusterBridgeEvent::ClusterConnected {
                outcome: ClusterSwitchOutcome {
                    context_name: "prod-ctx".to_string(),
                    healthy: true,
                    k8s_version: Some("v1.31.0".to_string()),
                    error_message: None,
                },
                clusters: Vec::new(),
                profile: UserProfileInfo {
                    active_context: Some("prod-ctx".to_string()),
                    user_name: None,
                    auth_type: "None".to_string(),
                    cluster_name: Some("prod".to_string()),
                    server_url: Some("https://10.0.0.10:6443".to_string()),
                    kubeconfig_paths: vec![],
                    k8s_version: Some("v1.31.0".to_string()),
                },
            },
        );

        assert!(cmds.iter().any(|c| matches!(
            c,
            ClusterBridgeCommand::LoadDaemonSets { cluster_id, .. } if cluster_id == "prod-ctx"
        )));
    }

    fn make_test_job(name: &str, namespace: &str) -> orbit_core::kubernetes::models::JobInfo {
        orbit_core::kubernetes::models::JobInfo {
            kind: "Job".to_string(),
            name: name.to_string(),
            namespace: namespace.to_string(),
            status: "Succeeded".to_string(),
            completions: "1/1".to_string(),
            duration: Some("30s".to_string()),
            age: "5m".to_string(),
            images: vec!["ghcr.io/orbit/migrate:v1".to_string()],
            containers: vec![orbit_core::kubernetes::models::ContainerImageInfo {
                name: "migrate".to_string(),
                image: "ghcr.io/orbit/migrate:v1".to_string(),
            }],
            labels: Default::default(),
            annotations: Default::default(),
        }
    }

    #[test]
    fn test_selecting_jobs_view_dispatches_load_and_applies_events() {
        let mut switcher = ClusterSwitcherState::new();
        switcher.connection_state = ClusterConnectionState::Connected {
            cluster_id: "prod-ctx".to_string(),
            display_name: "prod".to_string(),
            k8s_version: Some("v1.31.0".to_string()),
        };
        let mut sidebar = SidebarLayoutState::new();
        let mut overview = ClusterOverviewState::default();
        let mut workloads_overview = WorkloadsOverviewState::default();
        let mut namespaces = NamespaceFilterState::default();
        let mut namespaces_view = NamespacesViewState::default();
        let mut deployments = DeploymentsState::default();
        let mut statefulsets = StatefulSetsState::default();
        let mut daemonsets = DaemonSetsState::default();
        let mut jobs = JobsState::default();
        let mut cronjobs = CronJobsState::default();
        let mut pods = PodsState::default();
        let mut services = ServicesState::default();
        let mut configmaps = ConfigMapsState::default();
        let mut secrets = SecretsState::default();
        let mut persistent_volumes = PersistentVolumesState::default();
        let mut nodes = NodesState::default();
        let mut events = EventsState::default();

        let cmd = select_workspace_view_with_state(
            &switcher,
            &mut sidebar,
            &mut workloads_overview,
            &mut namespaces,
            &mut deployments,
            &mut statefulsets,
            &mut daemonsets,
            &mut ReplicaSetsState::default(),
            &mut jobs,
            &mut cronjobs,
            &mut pods,
            &mut services,
            &mut IngressesState::default(),
            &mut configmaps,
            &mut secrets,
            &mut persistent_volumes,
            &mut PersistentVolumeClaimsState::default(),
            &mut NetworkPoliciesState::default(),
            &mut StorageOverviewState::default(),
            &mut PodSecurityState::default(),
            &mut ResourceQuotasState::default(),
            &mut ClusterListViews::default(),
            &mut nodes,
            &mut events,
            WorkspaceView::Jobs,
        );
        assert_eq!(sidebar.active_view, WorkspaceView::Jobs);
        assert_eq!(
            cmd,
            Some(ClusterBridgeCommand::LoadJobs {
                cluster_id: "prod-ctx".to_string(),
                generation: 1,
            })
        );

        let job = make_test_job("db-migrate", "prod");
        apply_cluster_event_with_workspace(
            &mut switcher,
            &mut sidebar,
            &mut overview,
            &mut workloads_overview,
            &mut namespaces,
            &mut namespaces_view,
            &mut deployments,
            &mut statefulsets,
            &mut daemonsets,
            &mut ReplicaSetsState::default(),
            &mut jobs,
            &mut cronjobs,
            &mut pods,
            &mut services,
            &mut IngressesState::default(),
            &mut configmaps,
            &mut secrets,
            &mut persistent_volumes,
            &mut PersistentVolumeClaimsState::default(),
            &mut NetworkPoliciesState::default(),
            &mut StorageOverviewState::default(),
            &mut PodSecurityState::default(),
            &mut ResourceQuotasState::default(),
            &mut ClusterListViews::default(),
            &mut nodes,
            &mut events,
            ClusterBridgeEvent::JobsLoaded {
                cluster_id: "prod-ctx".to_string(),
                generation: 1,
                jobs: vec![job],
            },
        );
        assert_eq!(jobs.jobs.len(), 1);
        assert_eq!(jobs.status, crate::state::JobsLoadStatus::Loaded);

        let _ = jobs.begin_refresh();
        apply_cluster_event_with_workspace(
            &mut switcher,
            &mut sidebar,
            &mut overview,
            &mut workloads_overview,
            &mut namespaces,
            &mut namespaces_view,
            &mut deployments,
            &mut statefulsets,
            &mut daemonsets,
            &mut ReplicaSetsState::default(),
            &mut jobs,
            &mut cronjobs,
            &mut pods,
            &mut services,
            &mut IngressesState::default(),
            &mut configmaps,
            &mut secrets,
            &mut persistent_volumes,
            &mut PersistentVolumeClaimsState::default(),
            &mut NetworkPoliciesState::default(),
            &mut StorageOverviewState::default(),
            &mut PodSecurityState::default(),
            &mut ResourceQuotasState::default(),
            &mut ClusterListViews::default(),
            &mut nodes,
            &mut events,
            ClusterBridgeEvent::JobsLoadFailed {
                cluster_id: "prod-ctx".to_string(),
                generation: 2,
                message: "forbidden".to_string(),
            },
        );
        assert_eq!(jobs.status, crate::state::JobsLoadStatus::Error);
        assert_eq!(jobs.error_message.as_deref(), Some("forbidden"));
    }

    #[test]
    fn test_connecting_while_on_jobs_view_dispatches_load_jobs() {
        use orbit_core::kubernetes::manager::ClusterSwitchOutcome;
        use orbit_core::kubernetes::models::UserProfileInfo;

        let mut switcher = ClusterSwitcherState::new();
        let mut sidebar = SidebarLayoutState::new();
        sidebar.select_view(WorkspaceView::Jobs);
        let mut overview = ClusterOverviewState::default();
        let mut workloads_overview = WorkloadsOverviewState::default();
        let mut namespaces = NamespaceFilterState::default();
        let mut namespaces_view = NamespacesViewState::default();
        let mut deployments = DeploymentsState::default();
        let mut statefulsets = StatefulSetsState::default();
        let mut daemonsets = DaemonSetsState::default();
        let mut jobs = JobsState::default();
        let mut cronjobs = CronJobsState::default();
        let mut pods = PodsState::default();
        let mut services = ServicesState::default();
        let mut configmaps = ConfigMapsState::default();
        let mut secrets = SecretsState::default();
        let mut persistent_volumes = PersistentVolumesState::default();
        let mut nodes = NodesState::default();
        let mut events = EventsState::default();

        let cmds = apply_cluster_event_with_workspace(
            &mut switcher,
            &mut sidebar,
            &mut overview,
            &mut workloads_overview,
            &mut namespaces,
            &mut namespaces_view,
            &mut deployments,
            &mut statefulsets,
            &mut daemonsets,
            &mut ReplicaSetsState::default(),
            &mut jobs,
            &mut cronjobs,
            &mut pods,
            &mut services,
            &mut IngressesState::default(),
            &mut configmaps,
            &mut secrets,
            &mut persistent_volumes,
            &mut PersistentVolumeClaimsState::default(),
            &mut NetworkPoliciesState::default(),
            &mut StorageOverviewState::default(),
            &mut PodSecurityState::default(),
            &mut ResourceQuotasState::default(),
            &mut ClusterListViews::default(),
            &mut nodes,
            &mut events,
            ClusterBridgeEvent::ClusterConnected {
                outcome: ClusterSwitchOutcome {
                    context_name: "prod-ctx".to_string(),
                    healthy: true,
                    k8s_version: Some("v1.31.0".to_string()),
                    error_message: None,
                },
                clusters: Vec::new(),
                profile: UserProfileInfo {
                    active_context: Some("prod-ctx".to_string()),
                    user_name: None,
                    auth_type: "None".to_string(),
                    cluster_name: Some("prod".to_string()),
                    server_url: Some("https://10.0.0.10:6443".to_string()),
                    kubeconfig_paths: vec![],
                    k8s_version: Some("v1.31.0".to_string()),
                },
            },
        );

        assert!(cmds.iter().any(|c| matches!(
            c,
            ClusterBridgeCommand::LoadJobs { cluster_id, .. } if cluster_id == "prod-ctx"
        )));
    }

    fn make_test_cronjob(
        name: &str,
        namespace: &str,
    ) -> orbit_core::kubernetes::models::CronJobInfo {
        orbit_core::kubernetes::models::CronJobInfo {
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
    fn test_selecting_cronjobs_view_dispatches_load_and_applies_events() {
        let mut switcher = ClusterSwitcherState::new();
        switcher.connection_state = ClusterConnectionState::Connected {
            cluster_id: "prod-ctx".to_string(),
            display_name: "prod".to_string(),
            k8s_version: Some("v1.31.0".to_string()),
        };
        let mut sidebar = SidebarLayoutState::new();
        let mut overview = ClusterOverviewState::default();
        let mut workloads_overview = WorkloadsOverviewState::default();
        let mut namespaces = NamespaceFilterState::default();
        let mut namespaces_view = NamespacesViewState::default();
        let mut deployments = DeploymentsState::default();
        let mut statefulsets = StatefulSetsState::default();
        let mut daemonsets = DaemonSetsState::default();
        let mut jobs = JobsState::default();
        let mut cronjobs = CronJobsState::default();
        let mut pods = PodsState::default();
        let mut services = ServicesState::default();
        let mut configmaps = ConfigMapsState::default();
        let mut secrets = SecretsState::default();
        let mut persistent_volumes = PersistentVolumesState::default();
        let mut nodes = NodesState::default();
        let mut events = EventsState::default();

        let cmd = select_workspace_view_with_state(
            &switcher,
            &mut sidebar,
            &mut workloads_overview,
            &mut namespaces,
            &mut deployments,
            &mut statefulsets,
            &mut daemonsets,
            &mut ReplicaSetsState::default(),
            &mut jobs,
            &mut cronjobs,
            &mut pods,
            &mut services,
            &mut IngressesState::default(),
            &mut configmaps,
            &mut secrets,
            &mut persistent_volumes,
            &mut PersistentVolumeClaimsState::default(),
            &mut NetworkPoliciesState::default(),
            &mut StorageOverviewState::default(),
            &mut PodSecurityState::default(),
            &mut ResourceQuotasState::default(),
            &mut ClusterListViews::default(),
            &mut nodes,
            &mut events,
            WorkspaceView::CronJobs,
        );
        assert_eq!(sidebar.active_view, WorkspaceView::CronJobs);
        assert_eq!(
            cmd,
            Some(ClusterBridgeCommand::LoadCronJobs {
                cluster_id: "prod-ctx".to_string(),
                generation: 1,
            })
        );

        let cj = make_test_cronjob("nightly-backup", "prod");
        apply_cluster_event_with_workspace(
            &mut switcher,
            &mut sidebar,
            &mut overview,
            &mut workloads_overview,
            &mut namespaces,
            &mut namespaces_view,
            &mut deployments,
            &mut statefulsets,
            &mut daemonsets,
            &mut ReplicaSetsState::default(),
            &mut jobs,
            &mut cronjobs,
            &mut pods,
            &mut services,
            &mut IngressesState::default(),
            &mut configmaps,
            &mut secrets,
            &mut persistent_volumes,
            &mut PersistentVolumeClaimsState::default(),
            &mut NetworkPoliciesState::default(),
            &mut StorageOverviewState::default(),
            &mut PodSecurityState::default(),
            &mut ResourceQuotasState::default(),
            &mut ClusterListViews::default(),
            &mut nodes,
            &mut events,
            ClusterBridgeEvent::CronJobsLoaded {
                cluster_id: "prod-ctx".to_string(),
                generation: 1,
                cronjobs: vec![cj],
            },
        );
        assert_eq!(cronjobs.cronjobs.len(), 1);
        assert_eq!(cronjobs.status, crate::state::CronJobsLoadStatus::Loaded);

        let _ = cronjobs.begin_refresh();
        apply_cluster_event_with_workspace(
            &mut switcher,
            &mut sidebar,
            &mut overview,
            &mut workloads_overview,
            &mut namespaces,
            &mut namespaces_view,
            &mut deployments,
            &mut statefulsets,
            &mut daemonsets,
            &mut ReplicaSetsState::default(),
            &mut jobs,
            &mut cronjobs,
            &mut pods,
            &mut services,
            &mut IngressesState::default(),
            &mut configmaps,
            &mut secrets,
            &mut persistent_volumes,
            &mut PersistentVolumeClaimsState::default(),
            &mut NetworkPoliciesState::default(),
            &mut StorageOverviewState::default(),
            &mut PodSecurityState::default(),
            &mut ResourceQuotasState::default(),
            &mut ClusterListViews::default(),
            &mut nodes,
            &mut events,
            ClusterBridgeEvent::CronJobsLoadFailed {
                cluster_id: "prod-ctx".to_string(),
                generation: 2,
                message: "forbidden".to_string(),
            },
        );
        assert_eq!(cronjobs.status, crate::state::CronJobsLoadStatus::Error);
        assert_eq!(cronjobs.error_message.as_deref(), Some("forbidden"));
    }

    #[test]
    fn test_connecting_while_on_cronjobs_view_dispatches_load_cronjobs() {
        use orbit_core::kubernetes::manager::ClusterSwitchOutcome;
        use orbit_core::kubernetes::models::UserProfileInfo;

        let mut switcher = ClusterSwitcherState::new();
        let mut sidebar = SidebarLayoutState::new();
        sidebar.select_view(WorkspaceView::CronJobs);
        let mut overview = ClusterOverviewState::default();
        let mut workloads_overview = WorkloadsOverviewState::default();
        let mut namespaces = NamespaceFilterState::default();
        let mut namespaces_view = NamespacesViewState::default();
        let mut deployments = DeploymentsState::default();
        let mut statefulsets = StatefulSetsState::default();
        let mut daemonsets = DaemonSetsState::default();
        let mut jobs = JobsState::default();
        let mut cronjobs = CronJobsState::default();
        let mut pods = PodsState::default();
        let mut services = ServicesState::default();
        let mut configmaps = ConfigMapsState::default();
        let mut secrets = SecretsState::default();
        let mut persistent_volumes = PersistentVolumesState::default();
        let mut nodes = NodesState::default();
        let mut events = EventsState::default();

        let cmds = apply_cluster_event_with_workspace(
            &mut switcher,
            &mut sidebar,
            &mut overview,
            &mut workloads_overview,
            &mut namespaces,
            &mut namespaces_view,
            &mut deployments,
            &mut statefulsets,
            &mut daemonsets,
            &mut ReplicaSetsState::default(),
            &mut jobs,
            &mut cronjobs,
            &mut pods,
            &mut services,
            &mut IngressesState::default(),
            &mut configmaps,
            &mut secrets,
            &mut persistent_volumes,
            &mut PersistentVolumeClaimsState::default(),
            &mut NetworkPoliciesState::default(),
            &mut StorageOverviewState::default(),
            &mut PodSecurityState::default(),
            &mut ResourceQuotasState::default(),
            &mut ClusterListViews::default(),
            &mut nodes,
            &mut events,
            ClusterBridgeEvent::ClusterConnected {
                outcome: ClusterSwitchOutcome {
                    context_name: "prod-ctx".to_string(),
                    healthy: true,
                    k8s_version: Some("v1.31.0".to_string()),
                    error_message: None,
                },
                clusters: Vec::new(),
                profile: UserProfileInfo {
                    active_context: Some("prod-ctx".to_string()),
                    user_name: None,
                    auth_type: "None".to_string(),
                    cluster_name: Some("prod".to_string()),
                    server_url: Some("https://10.0.0.10:6443".to_string()),
                    kubeconfig_paths: vec![],
                    k8s_version: Some("v1.31.0".to_string()),
                },
            },
        );

        assert!(cmds.iter().any(|c| matches!(
            c,
            ClusterBridgeCommand::LoadCronJobs { cluster_id, .. } if cluster_id == "prod-ctx"
        )));
    }

    #[test]
    fn test_is_workspace_view_loading_tracks_ingresses_status() {
        assert!(!is_workspace_view_loading(
            WorkspaceView::Ingresses,
            WorkspaceLoadingFlags {
                services: true,
                ..Default::default()
            },
        ));
        assert!(is_workspace_view_loading(
            WorkspaceView::Ingresses,
            WorkspaceLoadingFlags {
                ingresses: true,
                ..Default::default()
            },
        ));
    }

    #[test]
    fn test_selecting_ingresses_view_dispatches_load_and_handles_events() {
        let mut state = ClusterSwitcherState::new();
        state.active_cluster_id = Some("prod-ctx".to_string());
        state.connection_state = ClusterConnectionState::Connected {
            cluster_id: "prod-ctx".to_string(),
            display_name: "Prod".to_string(),
            k8s_version: Some("v1.31.2".to_string()),
        };
        let mut sidebar = SidebarLayoutState::new();
        let mut overview = ClusterOverviewState::default();
        let mut workloads_overview = WorkloadsOverviewState::default();
        let mut namespaces = NamespaceFilterState::default();
        let mut namespaces_view = NamespacesViewState::default();
        let mut deployments = DeploymentsState::default();
        let mut statefulsets = StatefulSetsState::default();
        let mut daemonsets = DaemonSetsState::default();
        let mut jobs = JobsState::default();
        let mut cronjobs = CronJobsState::default();
        let mut pods = PodsState::default();
        let mut services = ServicesState::default();
        let mut ingresses = IngressesState::default();
        let mut configmaps = ConfigMapsState::default();
        let mut secrets = SecretsState::default();
        let mut persistent_volumes = PersistentVolumesState::default();
        let mut nodes = NodesState::default();
        let mut events = EventsState::default();
        let mut tabs = WorkspaceTabsState::default();
        tabs.activate_cluster("prod-ctx");

        let cmds = select_workspace_view_with_tabs(
            &state,
            &mut sidebar,
            &mut workloads_overview,
            &mut namespaces,
            &mut deployments,
            &mut statefulsets,
            &mut daemonsets,
            &mut ReplicaSetsState::default(),
            &mut jobs,
            &mut cronjobs,
            &mut pods,
            &mut services,
            &mut ingresses,
            &mut configmaps,
            &mut secrets,
            &mut persistent_volumes,
            &mut PersistentVolumeClaimsState::default(),
            &mut NetworkPoliciesState::default(),
            &mut StorageOverviewState::default(),
            &mut PodSecurityState::default(),
            &mut ResourceQuotasState::default(),
            &mut ClusterListViews::default(),
            &mut nodes,
            &mut events,
            &mut tabs,
            WorkspaceView::Ingresses,
        );
        assert_eq!(tabs.active_view(), Some(WorkspaceView::Ingresses));
        assert_eq!(sidebar.active_view, WorkspaceView::Ingresses);
        assert!(cmds.iter().any(|c| matches!(
            c,
            ClusterBridgeCommand::LoadIngresses { cluster_id, generation: 1 } if cluster_id == "prod-ctx"
        )));

        // Receiving IngressesLoaded populates state
        apply_cluster_event_with_workspace(
            &mut state,
            &mut sidebar,
            &mut overview,
            &mut workloads_overview,
            &mut namespaces,
            &mut namespaces_view,
            &mut deployments,
            &mut statefulsets,
            &mut daemonsets,
            &mut ReplicaSetsState::default(),
            &mut jobs,
            &mut cronjobs,
            &mut pods,
            &mut services,
            &mut ingresses,
            &mut configmaps,
            &mut secrets,
            &mut persistent_volumes,
            &mut PersistentVolumeClaimsState::default(),
            &mut NetworkPoliciesState::default(),
            &mut StorageOverviewState::default(),
            &mut PodSecurityState::default(),
            &mut ResourceQuotasState::default(),
            &mut ClusterListViews::default(),
            &mut nodes,
            &mut events,
            ClusterBridgeEvent::IngressesLoaded {
                cluster_id: "prod-ctx".to_string(),
                generation: 1,
                ingresses: vec![orbit_core::kubernetes::models::IngressInfo {
                    name: "web-ingress".to_string(),
                    namespace: "prod".to_string(),
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
                }],
            },
        );
        assert_eq!(ingresses.status, crate::state::IngressesLoadStatus::Loaded);
        assert_eq!(ingresses.ingresses.len(), 1);
        assert_eq!(ingresses.ingresses[0].name, "web-ingress");

        // Disconnecting clears ingresses state
        apply_cluster_event_with_workspace(
            &mut state,
            &mut sidebar,
            &mut overview,
            &mut workloads_overview,
            &mut namespaces,
            &mut namespaces_view,
            &mut deployments,
            &mut statefulsets,
            &mut daemonsets,
            &mut ReplicaSetsState::default(),
            &mut jobs,
            &mut cronjobs,
            &mut pods,
            &mut services,
            &mut ingresses,
            &mut configmaps,
            &mut secrets,
            &mut persistent_volumes,
            &mut PersistentVolumeClaimsState::default(),
            &mut NetworkPoliciesState::default(),
            &mut StorageOverviewState::default(),
            &mut PodSecurityState::default(),
            &mut ResourceQuotasState::default(),
            &mut ClusterListViews::default(),
            &mut nodes,
            &mut events,
            ClusterBridgeEvent::ConnectionError {
                cluster_id: Some("prod-ctx".to_string()),
                message: "lost connection".to_string(),
                clusters: vec![],
            },
        );
        assert_eq!(ingresses.status, crate::state::IngressesLoadStatus::Idle);
        assert!(ingresses.ingresses.is_empty());

        // Connecting while already on WorkspaceView::Ingresses dispatches LoadIngresses
        let connect_cmds = apply_cluster_event_with_workspace(
            &mut state,
            &mut sidebar,
            &mut overview,
            &mut workloads_overview,
            &mut namespaces,
            &mut namespaces_view,
            &mut deployments,
            &mut statefulsets,
            &mut daemonsets,
            &mut ReplicaSetsState::default(),
            &mut jobs,
            &mut cronjobs,
            &mut pods,
            &mut services,
            &mut ingresses,
            &mut configmaps,
            &mut secrets,
            &mut persistent_volumes,
            &mut PersistentVolumeClaimsState::default(),
            &mut NetworkPoliciesState::default(),
            &mut StorageOverviewState::default(),
            &mut PodSecurityState::default(),
            &mut ResourceQuotasState::default(),
            &mut ClusterListViews::default(),
            &mut nodes,
            &mut events,
            ClusterBridgeEvent::ClusterConnected {
                outcome: orbit_core::kubernetes::manager::ClusterSwitchOutcome {
                    context_name: "prod-ctx".to_string(),
                    healthy: true,
                    k8s_version: Some("v1.31.2".to_string()),
                    error_message: None,
                },
                clusters: vec![],
                profile: orbit_core::kubernetes::models::UserProfileInfo {
                    active_context: Some("prod-ctx".to_string()),
                    cluster_name: Some("Prod".to_string()),
                    server_url: None,
                    user_name: None,
                    auth_type: "token".to_string(),
                    kubeconfig_paths: vec![],
                    k8s_version: Some("v1.31.2".to_string()),
                },
            },
        );
        assert!(connect_cmds.iter().any(|c| matches!(
            c,
            ClusterBridgeCommand::LoadIngresses { cluster_id, .. } if cluster_id == "prod-ctx"
        )));
    }

    #[test]
    fn test_is_workspace_view_loading_tracks_persistent_volumes_status() {
        assert!(!is_workspace_view_loading(
            WorkspaceView::PersistentVolumes,
            WorkspaceLoadingFlags {
                nodes: true,
                ..Default::default()
            },
        ));
        assert!(is_workspace_view_loading(
            WorkspaceView::PersistentVolumes,
            WorkspaceLoadingFlags {
                persistent_volumes: true,
                ..Default::default()
            },
        ));
    }

    #[test]
    fn test_selecting_persistent_volumes_view_dispatches_load_and_handles_events() {
        let mut state = ClusterSwitcherState::new();
        state.active_cluster_id = Some("prod-ctx".to_string());
        state.connection_state = ClusterConnectionState::Connected {
            cluster_id: "prod-ctx".to_string(),
            display_name: "Prod".to_string(),
            k8s_version: Some("v1.31.2".to_string()),
        };
        let mut sidebar = SidebarLayoutState::new();
        let mut overview = ClusterOverviewState::default();
        let mut workloads_overview = WorkloadsOverviewState::default();
        let mut namespaces = NamespaceFilterState::default();
        let mut namespaces_view = NamespacesViewState::default();
        let mut deployments = DeploymentsState::default();
        let mut statefulsets = StatefulSetsState::default();
        let mut daemonsets = DaemonSetsState::default();
        let mut jobs = JobsState::default();
        let mut cronjobs = CronJobsState::default();
        let mut pods = PodsState::default();
        let mut services = ServicesState::default();
        let mut ingresses = IngressesState::default();
        let mut configmaps = ConfigMapsState::default();
        let mut secrets = SecretsState::default();
        let mut persistent_volumes = PersistentVolumesState::default();
        let mut nodes = NodesState::default();
        let mut events = EventsState::default();
        let mut tabs = WorkspaceTabsState::default();
        tabs.activate_cluster("prod-ctx");

        let cmds = select_workspace_view_with_tabs(
            &state,
            &mut sidebar,
            &mut workloads_overview,
            &mut namespaces,
            &mut deployments,
            &mut statefulsets,
            &mut daemonsets,
            &mut ReplicaSetsState::default(),
            &mut jobs,
            &mut cronjobs,
            &mut pods,
            &mut services,
            &mut ingresses,
            &mut configmaps,
            &mut secrets,
            &mut persistent_volumes,
            &mut PersistentVolumeClaimsState::default(),
            &mut NetworkPoliciesState::default(),
            &mut StorageOverviewState::default(),
            &mut PodSecurityState::default(),
            &mut ResourceQuotasState::default(),
            &mut ClusterListViews::default(),
            &mut nodes,
            &mut events,
            &mut tabs,
            WorkspaceView::PersistentVolumes,
        );
        assert_eq!(tabs.active_view(), Some(WorkspaceView::PersistentVolumes));
        assert_eq!(sidebar.active_view, WorkspaceView::PersistentVolumes);
        assert!(cmds.iter().any(|c| matches!(
            c,
            ClusterBridgeCommand::LoadPersistentVolumes { cluster_id, generation: 1 } if cluster_id == "prod-ctx"
        )));

        // Receiving PersistentVolumesLoaded populates state
        apply_cluster_event_with_workspace(
            &mut state,
            &mut sidebar,
            &mut overview,
            &mut workloads_overview,
            &mut namespaces,
            &mut namespaces_view,
            &mut deployments,
            &mut statefulsets,
            &mut daemonsets,
            &mut ReplicaSetsState::default(),
            &mut jobs,
            &mut cronjobs,
            &mut pods,
            &mut services,
            &mut ingresses,
            &mut configmaps,
            &mut secrets,
            &mut persistent_volumes,
            &mut PersistentVolumeClaimsState::default(),
            &mut NetworkPoliciesState::default(),
            &mut StorageOverviewState::default(),
            &mut PodSecurityState::default(),
            &mut ResourceQuotasState::default(),
            &mut ClusterListViews::default(),
            &mut nodes,
            &mut events,
            ClusterBridgeEvent::PersistentVolumesLoaded {
                cluster_id: "prod-ctx".to_string(),
                generation: 1,
                persistent_volumes: vec![orbit_core::kubernetes::models::PersistentVolumeInfo {
                    name: "pv-data-01".to_string(),
                    capacity: "20Gi".to_string(),
                    access_mode: "ReadWriteOnce".to_string(),
                    reclaim_policy: "Retain".to_string(),
                    status: "Bound".to_string(),
                    storage_class: "gp3".to_string(),
                    age: "4d".to_string(),
                    volume_mode: "Filesystem".to_string(),
                    reason: None,
                }],
            },
        );
        assert_eq!(
            persistent_volumes.status,
            crate::state::PersistentVolumesLoadStatus::Loaded
        );
        assert_eq!(persistent_volumes.persistent_volumes.len(), 1);
        assert_eq!(persistent_volumes.persistent_volumes[0].name, "pv-data-01");

        // Disconnecting clears persistent_volumes state
        apply_cluster_event_with_workspace(
            &mut state,
            &mut sidebar,
            &mut overview,
            &mut workloads_overview,
            &mut namespaces,
            &mut namespaces_view,
            &mut deployments,
            &mut statefulsets,
            &mut daemonsets,
            &mut ReplicaSetsState::default(),
            &mut jobs,
            &mut cronjobs,
            &mut pods,
            &mut services,
            &mut ingresses,
            &mut configmaps,
            &mut secrets,
            &mut persistent_volumes,
            &mut PersistentVolumeClaimsState::default(),
            &mut NetworkPoliciesState::default(),
            &mut StorageOverviewState::default(),
            &mut PodSecurityState::default(),
            &mut ResourceQuotasState::default(),
            &mut ClusterListViews::default(),
            &mut nodes,
            &mut events,
            ClusterBridgeEvent::ConnectionError {
                cluster_id: Some("prod-ctx".to_string()),
                message: "lost connection".to_string(),
                clusters: vec![],
            },
        );
        assert_eq!(
            persistent_volumes.status,
            crate::state::PersistentVolumesLoadStatus::Idle
        );
        assert!(persistent_volumes.persistent_volumes.is_empty());

        // Connecting while already on WorkspaceView::PersistentVolumes dispatches LoadPersistentVolumes
        let connect_cmds = apply_cluster_event_with_workspace(
            &mut state,
            &mut sidebar,
            &mut overview,
            &mut workloads_overview,
            &mut namespaces,
            &mut namespaces_view,
            &mut deployments,
            &mut statefulsets,
            &mut daemonsets,
            &mut ReplicaSetsState::default(),
            &mut jobs,
            &mut cronjobs,
            &mut pods,
            &mut services,
            &mut ingresses,
            &mut configmaps,
            &mut secrets,
            &mut persistent_volumes,
            &mut PersistentVolumeClaimsState::default(),
            &mut NetworkPoliciesState::default(),
            &mut StorageOverviewState::default(),
            &mut PodSecurityState::default(),
            &mut ResourceQuotasState::default(),
            &mut ClusterListViews::default(),
            &mut nodes,
            &mut events,
            ClusterBridgeEvent::ClusterConnected {
                outcome: orbit_core::kubernetes::manager::ClusterSwitchOutcome {
                    context_name: "prod-ctx".to_string(),
                    healthy: true,
                    k8s_version: Some("v1.31.2".to_string()),
                    error_message: None,
                },
                clusters: vec![],
                profile: orbit_core::kubernetes::models::UserProfileInfo {
                    active_context: Some("prod-ctx".to_string()),
                    cluster_name: Some("Prod".to_string()),
                    server_url: None,
                    user_name: None,
                    auth_type: "token".to_string(),
                    kubeconfig_paths: vec![],
                    k8s_version: Some("v1.31.2".to_string()),
                },
            },
        );
        assert!(connect_cmds.iter().any(|c| matches!(
            c,
            ClusterBridgeCommand::LoadPersistentVolumes { cluster_id, .. } if cluster_id == "prod-ctx"
        )));
    }

    #[test]
    fn test_selecting_persistent_volume_claims_view_triggers_load_and_handles_events() {
        let mut state = ClusterSwitcherState::new();
        state.active_cluster_id = Some("prod-ctx".to_string());
        state.connection_state = ClusterConnectionState::Connected {
            cluster_id: "prod-ctx".to_string(),
            display_name: "Prod".to_string(),
            k8s_version: Some("v1.31.2".to_string()),
        };
        let mut sidebar = SidebarLayoutState::new();
        let mut overview = ClusterOverviewState::default();
        let mut workloads_overview = WorkloadsOverviewState::default();
        let mut namespaces = NamespaceFilterState::default();
        let mut namespaces_view = NamespacesViewState::default();
        let mut deployments = DeploymentsState::default();
        let mut statefulsets = StatefulSetsState::default();
        let mut daemonsets = DaemonSetsState::default();
        let mut jobs = JobsState::default();
        let mut cronjobs = CronJobsState::default();
        let mut pods = PodsState::default();
        let mut services = ServicesState::default();
        let mut ingresses = IngressesState::default();
        let mut configmaps = ConfigMapsState::default();
        let mut secrets = SecretsState::default();
        let mut persistent_volumes = PersistentVolumesState::default();
        let mut persistent_volume_claims = PersistentVolumeClaimsState::default();
        let mut nodes = NodesState::default();
        let mut events = EventsState::default();
        let mut tabs = WorkspaceTabsState::default();
        tabs.activate_cluster("prod-ctx");

        let cmds = select_workspace_view_with_tabs(
            &state,
            &mut sidebar,
            &mut workloads_overview,
            &mut namespaces,
            &mut deployments,
            &mut statefulsets,
            &mut daemonsets,
            &mut ReplicaSetsState::default(),
            &mut jobs,
            &mut cronjobs,
            &mut pods,
            &mut services,
            &mut ingresses,
            &mut configmaps,
            &mut secrets,
            &mut persistent_volumes,
            &mut persistent_volume_claims,
            &mut NetworkPoliciesState::default(),
            &mut StorageOverviewState::default(),
            &mut PodSecurityState::default(),
            &mut ResourceQuotasState::default(),
            &mut ClusterListViews::default(),
            &mut nodes,
            &mut events,
            &mut tabs,
            WorkspaceView::PersistentVolumeClaims,
        );
        assert_eq!(
            tabs.active_view(),
            Some(WorkspaceView::PersistentVolumeClaims)
        );
        assert_eq!(sidebar.active_view, WorkspaceView::PersistentVolumeClaims);
        assert!(cmds.iter().any(|c| matches!(
            c,
            ClusterBridgeCommand::LoadPersistentVolumeClaims { cluster_id, generation: 1 } if cluster_id == "prod-ctx"
        )));

        // Receiving PersistentVolumeClaimsLoaded populates state
        apply_cluster_event_with_workspace(
            &mut state,
            &mut sidebar,
            &mut overview,
            &mut workloads_overview,
            &mut namespaces,
            &mut namespaces_view,
            &mut deployments,
            &mut statefulsets,
            &mut daemonsets,
            &mut ReplicaSetsState::default(),
            &mut jobs,
            &mut cronjobs,
            &mut pods,
            &mut services,
            &mut ingresses,
            &mut configmaps,
            &mut secrets,
            &mut persistent_volumes,
            &mut persistent_volume_claims,
            &mut NetworkPoliciesState::default(),
            &mut StorageOverviewState::default(),
            &mut PodSecurityState::default(),
            &mut ResourceQuotasState::default(),
            &mut ClusterListViews::default(),
            &mut nodes,
            &mut events,
            ClusterBridgeEvent::PersistentVolumeClaimsLoaded {
                cluster_id: "prod-ctx".to_string(),
                generation: 1,
                persistent_volume_claims: vec![
                    orbit_core::kubernetes::models::PersistentVolumeClaimInfo {
                        name: "data-pg-0".to_string(),
                        namespace: "prod".to_string(),
                        status: "Bound".to_string(),
                        volume: "pv-data-01".to_string(),
                        capacity: "20Gi".to_string(),
                        access_mode: "ReadWriteOnce".to_string(),
                        storage_class: "gp3".to_string(),
                        age: "4d".to_string(),
                    },
                ],
            },
        );
        assert_eq!(
            persistent_volume_claims.status,
            crate::state::PersistentVolumeClaimsLoadStatus::Loaded
        );
        assert_eq!(persistent_volume_claims.persistent_volume_claims.len(), 1);
        assert_eq!(
            persistent_volume_claims.persistent_volume_claims[0].name,
            "data-pg-0"
        );

        // Disconnecting clears persistent_volume_claims state
        apply_cluster_event_with_workspace(
            &mut state,
            &mut sidebar,
            &mut overview,
            &mut workloads_overview,
            &mut namespaces,
            &mut namespaces_view,
            &mut deployments,
            &mut statefulsets,
            &mut daemonsets,
            &mut ReplicaSetsState::default(),
            &mut jobs,
            &mut cronjobs,
            &mut pods,
            &mut services,
            &mut ingresses,
            &mut configmaps,
            &mut secrets,
            &mut persistent_volumes,
            &mut persistent_volume_claims,
            &mut NetworkPoliciesState::default(),
            &mut StorageOverviewState::default(),
            &mut PodSecurityState::default(),
            &mut ResourceQuotasState::default(),
            &mut ClusterListViews::default(),
            &mut nodes,
            &mut events,
            ClusterBridgeEvent::ConnectionError {
                cluster_id: Some("prod-ctx".to_string()),
                message: "lost connection".to_string(),
                clusters: vec![],
            },
        );
        assert_eq!(
            persistent_volume_claims.status,
            crate::state::PersistentVolumeClaimsLoadStatus::Idle
        );
        assert!(persistent_volume_claims.persistent_volume_claims.is_empty());

        // Connecting while already on WorkspaceView::PersistentVolumeClaims dispatches LoadPersistentVolumeClaims
        let connect_cmds = apply_cluster_event_with_workspace(
            &mut state,
            &mut sidebar,
            &mut overview,
            &mut workloads_overview,
            &mut namespaces,
            &mut namespaces_view,
            &mut deployments,
            &mut statefulsets,
            &mut daemonsets,
            &mut ReplicaSetsState::default(),
            &mut jobs,
            &mut cronjobs,
            &mut pods,
            &mut services,
            &mut ingresses,
            &mut configmaps,
            &mut secrets,
            &mut persistent_volumes,
            &mut persistent_volume_claims,
            &mut NetworkPoliciesState::default(),
            &mut StorageOverviewState::default(),
            &mut PodSecurityState::default(),
            &mut ResourceQuotasState::default(),
            &mut ClusterListViews::default(),
            &mut nodes,
            &mut events,
            ClusterBridgeEvent::ClusterConnected {
                outcome: orbit_core::kubernetes::manager::ClusterSwitchOutcome {
                    context_name: "prod-ctx".to_string(),
                    healthy: true,
                    k8s_version: Some("v1.31.2".to_string()),
                    error_message: None,
                },
                clusters: vec![],
                profile: orbit_core::kubernetes::models::UserProfileInfo {
                    active_context: Some("prod-ctx".to_string()),
                    cluster_name: Some("Prod".to_string()),
                    server_url: None,
                    user_name: None,
                    auth_type: "token".to_string(),
                    kubeconfig_paths: vec![],
                    k8s_version: Some("v1.31.2".to_string()),
                },
            },
        );
        assert!(connect_cmds.iter().any(|c| matches!(
            c,
            ClusterBridgeCommand::LoadPersistentVolumeClaims { cluster_id, .. } if cluster_id == "prod-ctx"
        )));
    }

    #[test]
    fn test_network_policies_view_navigation_load_and_disconnect_lifecycle() {
        let mut state = ClusterSwitcherState::new();
        state.active_cluster_id = Some("prod-ctx".to_string());
        state.connection_state = ClusterConnectionState::Connected {
            cluster_id: "prod-ctx".to_string(),
            display_name: "Prod".to_string(),
            k8s_version: Some("v1.31.2".to_string()),
        };
        let mut sidebar = SidebarLayoutState::new();
        let mut overview = ClusterOverviewState::default();
        let mut workloads_overview = WorkloadsOverviewState::default();
        let mut namespaces = NamespaceFilterState::default();
        let mut namespaces_view = NamespacesViewState::default();
        let mut deployments = DeploymentsState::default();
        let mut statefulsets = StatefulSetsState::default();
        let mut daemonsets = DaemonSetsState::default();
        let mut jobs = JobsState::default();
        let mut cronjobs = CronJobsState::default();
        let mut pods = PodsState::default();
        let mut services = ServicesState::default();
        let mut ingresses = IngressesState::default();
        let mut configmaps = ConfigMapsState::default();
        let mut secrets = SecretsState::default();
        let mut persistent_volumes = PersistentVolumesState::default();
        let mut persistent_volume_claims = PersistentVolumeClaimsState::default();
        let mut network_policies = NetworkPoliciesState::default();
        let mut nodes = NodesState::default();
        let mut events = EventsState::default();
        let mut tabs = WorkspaceTabsState::default();
        tabs.activate_cluster("prod-ctx");

        let cmds = select_workspace_view_with_tabs(
            &state,
            &mut sidebar,
            &mut workloads_overview,
            &mut namespaces,
            &mut deployments,
            &mut statefulsets,
            &mut daemonsets,
            &mut ReplicaSetsState::default(),
            &mut jobs,
            &mut cronjobs,
            &mut pods,
            &mut services,
            &mut ingresses,
            &mut configmaps,
            &mut secrets,
            &mut persistent_volumes,
            &mut persistent_volume_claims,
            &mut network_policies,
            &mut StorageOverviewState::default(),
            &mut PodSecurityState::default(),
            &mut ResourceQuotasState::default(),
            &mut ClusterListViews::default(),
            &mut nodes,
            &mut events,
            &mut tabs,
            WorkspaceView::NetworkPolicies,
        );
        assert_eq!(tabs.active_view(), Some(WorkspaceView::NetworkPolicies));
        assert_eq!(sidebar.active_view, WorkspaceView::NetworkPolicies);
        assert!(cmds.iter().any(|c| matches!(
            c,
            ClusterBridgeCommand::LoadNetworkPolicies { cluster_id, generation: 1 } if cluster_id == "prod-ctx"
        )));

        // Receiving NetworkPoliciesLoaded populates state
        apply_cluster_event_with_workspace(
            &mut state,
            &mut sidebar,
            &mut overview,
            &mut workloads_overview,
            &mut namespaces,
            &mut namespaces_view,
            &mut deployments,
            &mut statefulsets,
            &mut daemonsets,
            &mut ReplicaSetsState::default(),
            &mut jobs,
            &mut cronjobs,
            &mut pods,
            &mut services,
            &mut ingresses,
            &mut configmaps,
            &mut secrets,
            &mut persistent_volumes,
            &mut persistent_volume_claims,
            &mut network_policies,
            &mut StorageOverviewState::default(),
            &mut PodSecurityState::default(),
            &mut ResourceQuotasState::default(),
            &mut ClusterListViews::default(),
            &mut nodes,
            &mut events,
            ClusterBridgeEvent::NetworkPoliciesLoaded {
                cluster_id: "prod-ctx".to_string(),
                generation: 1,
                network_policies: vec![orbit_core::kubernetes::models::PolicyInfo {
                    uid: "uid-np-1".to_string(),
                    name: "allow-api".to_string(),
                    r#type: "Network Policy".to_string(),
                    scope: "Namespaced".to_string(),
                    namespace: "prod".to_string(),
                    status: "Enforced".to_string(),
                    mode: "Ingress, Egress".to_string(),
                    violations: 0,
                    last_updated: "2d".to_string(),
                    description: "Pods matching: app=api".to_string(),
                    rules: "policyTypes:\n- Ingress\n".to_string(),
                    kind: "NetworkPolicy".to_string(),
                }],
            },
        );
        assert_eq!(
            network_policies.status,
            crate::state::NetworkPoliciesLoadStatus::Loaded
        );
        assert_eq!(network_policies.network_policies.len(), 1);
        assert_eq!(network_policies.network_policies[0].name, "allow-api");

        // Disconnecting clears network_policies state
        apply_cluster_event_with_workspace(
            &mut state,
            &mut sidebar,
            &mut overview,
            &mut workloads_overview,
            &mut namespaces,
            &mut namespaces_view,
            &mut deployments,
            &mut statefulsets,
            &mut daemonsets,
            &mut ReplicaSetsState::default(),
            &mut jobs,
            &mut cronjobs,
            &mut pods,
            &mut services,
            &mut ingresses,
            &mut configmaps,
            &mut secrets,
            &mut persistent_volumes,
            &mut persistent_volume_claims,
            &mut network_policies,
            &mut StorageOverviewState::default(),
            &mut PodSecurityState::default(),
            &mut ResourceQuotasState::default(),
            &mut ClusterListViews::default(),
            &mut nodes,
            &mut events,
            ClusterBridgeEvent::ConnectionError {
                cluster_id: Some("prod-ctx".to_string()),
                message: "lost connection".to_string(),
                clusters: vec![],
            },
        );
        assert_eq!(
            network_policies.status,
            crate::state::NetworkPoliciesLoadStatus::Idle
        );
        assert!(network_policies.network_policies.is_empty());

        // Connecting while already on WorkspaceView::NetworkPolicies dispatches LoadNetworkPolicies
        let connect_cmds = apply_cluster_event_with_workspace(
            &mut state,
            &mut sidebar,
            &mut overview,
            &mut workloads_overview,
            &mut namespaces,
            &mut namespaces_view,
            &mut deployments,
            &mut statefulsets,
            &mut daemonsets,
            &mut ReplicaSetsState::default(),
            &mut jobs,
            &mut cronjobs,
            &mut pods,
            &mut services,
            &mut ingresses,
            &mut configmaps,
            &mut secrets,
            &mut persistent_volumes,
            &mut persistent_volume_claims,
            &mut network_policies,
            &mut StorageOverviewState::default(),
            &mut PodSecurityState::default(),
            &mut ResourceQuotasState::default(),
            &mut ClusterListViews::default(),
            &mut nodes,
            &mut events,
            ClusterBridgeEvent::ClusterConnected {
                outcome: orbit_core::kubernetes::manager::ClusterSwitchOutcome {
                    context_name: "prod-ctx".to_string(),
                    healthy: true,
                    k8s_version: Some("v1.31.2".to_string()),
                    error_message: None,
                },
                clusters: vec![],
                profile: orbit_core::kubernetes::models::UserProfileInfo {
                    active_context: Some("prod-ctx".to_string()),
                    cluster_name: Some("Prod".to_string()),
                    server_url: None,
                    user_name: None,
                    auth_type: "token".to_string(),
                    kubeconfig_paths: vec![],
                    k8s_version: Some("v1.31.2".to_string()),
                },
            },
        );
        assert!(connect_cmds.iter().any(|c| matches!(
            c,
            ClusterBridgeCommand::LoadNetworkPolicies { cluster_id, .. } if cluster_id == "prod-ctx"
        )));
    }

    #[test]
    fn test_pod_security_view_navigation_refresh_and_connect_dispatch() {
        let mut state = ClusterSwitcherState::new();
        state.active_cluster_id = Some("prod-ctx".to_string());
        state.connection_state = ClusterConnectionState::Connected {
            cluster_id: "prod-ctx".to_string(),
            display_name: "Prod".to_string(),
            k8s_version: Some("v1.31.2".to_string()),
        };
        let mut sidebar = SidebarLayoutState::new();
        let mut overview = ClusterOverviewState::default();
        let mut workloads_overview = WorkloadsOverviewState::default();
        let mut namespaces = NamespaceFilterState::default();
        let mut namespaces_view = NamespacesViewState::default();
        let mut pod_security = PodSecurityState::default();
        let mut tabs = WorkspaceTabsState::default();
        tabs.activate_cluster("prod-ctx");

        let cmds = select_workspace_view_with_tabs(
            &state,
            &mut sidebar,
            &mut workloads_overview,
            &mut namespaces,
            &mut DeploymentsState::default(),
            &mut StatefulSetsState::default(),
            &mut DaemonSetsState::default(),
            &mut ReplicaSetsState::default(),
            &mut JobsState::default(),
            &mut CronJobsState::default(),
            &mut PodsState::default(),
            &mut ServicesState::default(),
            &mut IngressesState::default(),
            &mut ConfigMapsState::default(),
            &mut SecretsState::default(),
            &mut PersistentVolumesState::default(),
            &mut PersistentVolumeClaimsState::default(),
            &mut NetworkPoliciesState::default(),
            &mut StorageOverviewState::default(),
            &mut pod_security,
            &mut ResourceQuotasState::default(),
            &mut ClusterListViews::default(),
            &mut NodesState::default(),
            &mut EventsState::default(),
            &mut tabs,
            WorkspaceView::PodSecurity,
        );
        assert_eq!(sidebar.active_view, WorkspaceView::PodSecurity);
        assert!(cmds.iter().any(|c| matches!(
            c,
            ClusterBridgeCommand::LoadPodSecurity { cluster_id, generation: 1 } if cluster_id == "prod-ctx"
        )));
        assert!(is_workspace_view_loading(
            WorkspaceView::PodSecurity,
            WorkspaceLoadingFlags {
                pod_security: true,
                ..WorkspaceLoadingFlags::default()
            }
        ));

        let refresh_cmd = pod_security.begin_refresh();
        assert_eq!(
            refresh_cmd,
            Some(ClusterBridgeCommand::LoadPodSecurity {
                cluster_id: "prod-ctx".to_string(),
                generation: 2,
            })
        );

        let connect_cmds = apply_cluster_event_with_workspace(
            &mut state,
            &mut sidebar,
            &mut overview,
            &mut workloads_overview,
            &mut namespaces,
            &mut namespaces_view,
            &mut DeploymentsState::default(),
            &mut StatefulSetsState::default(),
            &mut DaemonSetsState::default(),
            &mut ReplicaSetsState::default(),
            &mut JobsState::default(),
            &mut CronJobsState::default(),
            &mut PodsState::default(),
            &mut ServicesState::default(),
            &mut IngressesState::default(),
            &mut ConfigMapsState::default(),
            &mut SecretsState::default(),
            &mut PersistentVolumesState::default(),
            &mut PersistentVolumeClaimsState::default(),
            &mut NetworkPoliciesState::default(),
            &mut StorageOverviewState::default(),
            &mut pod_security,
            &mut ResourceQuotasState::default(),
            &mut ClusterListViews::default(),
            &mut NodesState::default(),
            &mut EventsState::default(),
            ClusterBridgeEvent::ClusterConnected {
                outcome: orbit_core::kubernetes::manager::ClusterSwitchOutcome {
                    context_name: "prod-ctx".to_string(),
                    healthy: true,
                    k8s_version: Some("v1.31.2".to_string()),
                    error_message: None,
                },
                clusters: vec![],
                profile: orbit_core::kubernetes::models::UserProfileInfo {
                    active_context: Some("prod-ctx".to_string()),
                    cluster_name: Some("Prod".to_string()),
                    server_url: None,
                    user_name: None,
                    auth_type: "token".to_string(),
                    kubeconfig_paths: vec![],
                    k8s_version: Some("v1.31.2".to_string()),
                },
            },
        );
        assert!(connect_cmds.iter().any(|c| matches!(
            c,
            ClusterBridgeCommand::LoadPodSecurity { cluster_id, .. } if cluster_id == "prod-ctx"
        )));
    }

    #[test]
    fn test_resource_quotas_view_navigation_refresh_and_connect_dispatch() {
        let mut state = ClusterSwitcherState::new();
        state.active_cluster_id = Some("prod-ctx".to_string());
        state.connection_state = ClusterConnectionState::Connected {
            cluster_id: "prod-ctx".to_string(),
            display_name: "Prod".to_string(),
            k8s_version: Some("v1.31.2".to_string()),
        };
        let mut sidebar = SidebarLayoutState::new();
        let mut overview = ClusterOverviewState::default();
        let mut workloads_overview = WorkloadsOverviewState::default();
        let mut namespaces = NamespaceFilterState::default();
        let mut namespaces_view = NamespacesViewState::default();
        let mut resource_quotas = ResourceQuotasState::default();
        let mut tabs = WorkspaceTabsState::default();
        tabs.activate_cluster("prod-ctx");

        let cmds = select_workspace_view_with_tabs(
            &state,
            &mut sidebar,
            &mut workloads_overview,
            &mut namespaces,
            &mut DeploymentsState::default(),
            &mut StatefulSetsState::default(),
            &mut DaemonSetsState::default(),
            &mut ReplicaSetsState::default(),
            &mut JobsState::default(),
            &mut CronJobsState::default(),
            &mut PodsState::default(),
            &mut ServicesState::default(),
            &mut IngressesState::default(),
            &mut ConfigMapsState::default(),
            &mut SecretsState::default(),
            &mut PersistentVolumesState::default(),
            &mut PersistentVolumeClaimsState::default(),
            &mut NetworkPoliciesState::default(),
            &mut StorageOverviewState::default(),
            &mut PodSecurityState::default(),
            &mut resource_quotas,
            &mut ClusterListViews::default(),
            &mut NodesState::default(),
            &mut EventsState::default(),
            &mut tabs,
            WorkspaceView::ResourceQuotas,
        );
        assert_eq!(sidebar.active_view, WorkspaceView::ResourceQuotas);
        assert!(cmds.iter().any(|c| matches!(
            c,
            ClusterBridgeCommand::LoadResourceQuotas { cluster_id, generation: 1 } if cluster_id == "prod-ctx"
        )));
        assert!(is_workspace_view_loading(
            WorkspaceView::ResourceQuotas,
            WorkspaceLoadingFlags {
                resource_quotas: true,
                hpas: true,
                storage_classes: true,
                security_overview: true,
                admission_webhooks: true,
                ..WorkspaceLoadingFlags::default()
            }
        ));

        let refresh_cmd = resource_quotas.begin_refresh();
        assert_eq!(
            refresh_cmd,
            Some(ClusterBridgeCommand::LoadResourceQuotas {
                cluster_id: "prod-ctx".to_string(),
                generation: 2,
            })
        );

        let connect_cmds = apply_cluster_event_with_workspace(
            &mut state,
            &mut sidebar,
            &mut overview,
            &mut workloads_overview,
            &mut namespaces,
            &mut namespaces_view,
            &mut DeploymentsState::default(),
            &mut StatefulSetsState::default(),
            &mut DaemonSetsState::default(),
            &mut ReplicaSetsState::default(),
            &mut JobsState::default(),
            &mut CronJobsState::default(),
            &mut PodsState::default(),
            &mut ServicesState::default(),
            &mut IngressesState::default(),
            &mut ConfigMapsState::default(),
            &mut SecretsState::default(),
            &mut PersistentVolumesState::default(),
            &mut PersistentVolumeClaimsState::default(),
            &mut NetworkPoliciesState::default(),
            &mut StorageOverviewState::default(),
            &mut PodSecurityState::default(),
            &mut resource_quotas,
            &mut ClusterListViews::default(),
            &mut NodesState::default(),
            &mut EventsState::default(),
            ClusterBridgeEvent::ClusterConnected {
                outcome: orbit_core::kubernetes::manager::ClusterSwitchOutcome {
                    context_name: "prod-ctx".to_string(),
                    healthy: true,
                    k8s_version: Some("v1.31.2".to_string()),
                    error_message: None,
                },
                clusters: vec![],
                profile: orbit_core::kubernetes::models::UserProfileInfo {
                    active_context: Some("prod-ctx".to_string()),
                    cluster_name: Some("Prod".to_string()),
                    server_url: None,
                    user_name: None,
                    auth_type: "token".to_string(),
                    kubeconfig_paths: vec![],
                    k8s_version: Some("v1.31.2".to_string()),
                },
            },
        );
        assert!(connect_cmds.iter().any(|c| matches!(
            c,
            ClusterBridgeCommand::LoadResourceQuotas { cluster_id, .. } if cluster_id == "prod-ctx"
        )));
    }

    #[test]
    fn test_storage_overview_view_navigation_refresh_and_connect_dispatch() {
        let mut state = ClusterSwitcherState::new();
        state.active_cluster_id = Some("prod-ctx".to_string());
        state.connection_state = ClusterConnectionState::Connected {
            cluster_id: "prod-ctx".to_string(),
            display_name: "Prod".to_string(),
            k8s_version: Some("v1.31.2".to_string()),
        };
        let mut sidebar = SidebarLayoutState::new();
        let mut overview = ClusterOverviewState::default();
        let mut workloads_overview = WorkloadsOverviewState::default();
        let mut namespaces = NamespaceFilterState::default();
        let mut namespaces_view = NamespacesViewState::default();
        let mut storage_overview = StorageOverviewState::default();
        let mut tabs = WorkspaceTabsState::default();
        tabs.activate_cluster("prod-ctx");

        let cmds = select_workspace_view_with_tabs(
            &state,
            &mut sidebar,
            &mut workloads_overview,
            &mut namespaces,
            &mut DeploymentsState::default(),
            &mut StatefulSetsState::default(),
            &mut DaemonSetsState::default(),
            &mut ReplicaSetsState::default(),
            &mut JobsState::default(),
            &mut CronJobsState::default(),
            &mut PodsState::default(),
            &mut ServicesState::default(),
            &mut IngressesState::default(),
            &mut ConfigMapsState::default(),
            &mut SecretsState::default(),
            &mut PersistentVolumesState::default(),
            &mut PersistentVolumeClaimsState::default(),
            &mut NetworkPoliciesState::default(),
            &mut storage_overview,
            &mut PodSecurityState::default(),
            &mut ResourceQuotasState::default(),
            &mut ClusterListViews::default(),
            &mut NodesState::default(),
            &mut EventsState::default(),
            &mut tabs,
            WorkspaceView::StorageOverview,
        );
        assert_eq!(sidebar.active_view, WorkspaceView::StorageOverview);
        assert!(cmds.iter().any(|c| matches!(
            c,
            ClusterBridgeCommand::LoadStorageOverview { cluster_id, generation: 1 } if cluster_id == "prod-ctx"
        )));
        assert!(is_workspace_view_loading(
            WorkspaceView::StorageOverview,
            WorkspaceLoadingFlags {
                storage_overview: true,
                ..WorkspaceLoadingFlags::default()
            }
        ));

        let refresh_cmd = storage_overview.begin_refresh();
        assert_eq!(
            refresh_cmd,
            Some(ClusterBridgeCommand::LoadStorageOverview {
                cluster_id: "prod-ctx".to_string(),
                generation: 2,
            })
        );

        let connect_cmds = apply_cluster_event_with_workspace(
            &mut state,
            &mut sidebar,
            &mut overview,
            &mut workloads_overview,
            &mut namespaces,
            &mut namespaces_view,
            &mut DeploymentsState::default(),
            &mut StatefulSetsState::default(),
            &mut DaemonSetsState::default(),
            &mut ReplicaSetsState::default(),
            &mut JobsState::default(),
            &mut CronJobsState::default(),
            &mut PodsState::default(),
            &mut ServicesState::default(),
            &mut IngressesState::default(),
            &mut ConfigMapsState::default(),
            &mut SecretsState::default(),
            &mut PersistentVolumesState::default(),
            &mut PersistentVolumeClaimsState::default(),
            &mut NetworkPoliciesState::default(),
            &mut storage_overview,
            &mut PodSecurityState::default(),
            &mut ResourceQuotasState::default(),
            &mut ClusterListViews::default(),
            &mut NodesState::default(),
            &mut EventsState::default(),
            ClusterBridgeEvent::ClusterConnected {
                outcome: orbit_core::kubernetes::manager::ClusterSwitchOutcome {
                    context_name: "prod-ctx".to_string(),
                    healthy: true,
                    k8s_version: Some("v1.31.2".to_string()),
                    error_message: None,
                },
                clusters: vec![],
                profile: orbit_core::kubernetes::models::UserProfileInfo {
                    active_context: Some("prod-ctx".to_string()),
                    cluster_name: Some("Prod".to_string()),
                    server_url: None,
                    user_name: None,
                    auth_type: "token".to_string(),
                    kubeconfig_paths: vec![],
                    k8s_version: Some("v1.31.2".to_string()),
                },
            },
        );
        assert!(connect_cmds.iter().any(|c| matches!(
            c,
            ClusterBridgeCommand::LoadStorageOverview { cluster_id, .. } if cluster_id == "prod-ctx"
        )));
    }

    #[test]
    fn test_focus_target_for_picker_transition_switches_between_picker_and_shell_root() {
        assert_eq!(
            focus_target_for_picker_transition(true),
            ShellFocusTarget::PickerSearch
        );
        assert_eq!(
            focus_target_for_picker_transition(false),
            ShellFocusTarget::ShellRoot
        );
    }

    #[test]
    fn test_is_full_bleed_table_view_covers_all_19_resource_table_views() {
        let table_views = [
            WorkspaceView::Namespaces,
            WorkspaceView::Deployments,
            WorkspaceView::StatefulSets,
            WorkspaceView::DaemonSets,
            WorkspaceView::ReplicaSets,
            WorkspaceView::Jobs,
            WorkspaceView::CronJobs,
            WorkspaceView::Pods,
            WorkspaceView::Services,
            WorkspaceView::Ingresses,
            WorkspaceView::ConfigMaps,
            WorkspaceView::Secrets,
            WorkspaceView::PersistentVolumes,
            WorkspaceView::PersistentVolumeClaims,
            WorkspaceView::NetworkPolicies,
            WorkspaceView::PodSecurity,
            WorkspaceView::ResourceQuotas,
            WorkspaceView::Nodes,
            WorkspaceView::Events,
        ];
        for view in table_views {
            assert!(
                is_full_bleed_table_view(view),
                "{view:?} must render as a full-bleed table view"
            );
        }

        for non_table_view in [
            WorkspaceView::Overview,
            WorkspaceView::WorkloadsOverview,
            WorkspaceView::StorageOverview,
            WorkspaceView::Settings,
            WorkspaceView::Logs,
        ] {
            assert!(
                !is_full_bleed_table_view(non_table_view),
                "{non_table_view:?} must not render as a full-bleed table view"
            );
        }
    }

    #[test]
    fn test_workspace_drawer_overlay_stacking_and_escape_dismissal() {
        assert!(workspace_drawer_stacks_as_overlay());

        let main_src = include_str!("main.rs");
        let render_workspace_idx = main_src
            .find("fn render_workspace(")
            .expect("render_workspace must exist");
        let render_footer_idx = main_src
            .find("fn render_footer(")
            .expect("render_footer must exist");
        let render_workspace_body = &main_src[render_workspace_idx..render_footer_idx];
        assert!(
            render_workspace_body.contains(".relative()"),
            "render_workspace must attach .relative() to stack the drawer as an overlay"
        );
        assert!(
            render_workspace_body.contains(".child(workspace_main)"),
            "render_workspace must render workspace_main as the flex child"
        );
        assert!(
            render_workspace_body.contains(".when_some(drawer, |el, panel| el.child(panel))"),
            "render_workspace must mount drawer as an overlay sibling"
        );

        use crate::components::{ContextMenuItem, ResourceDrawerTabId};

        let mut drawer = ResourceDrawerState::default();
        let target = ResourceDrawerTarget::Deployment {
            name: "api".to_string(),
            namespace: "default".to_string(),
        };
        let _ = drawer.open_or_switch(Some("ctx-a"), target, ResourceDrawerTabId::Overview);
        assert!(drawer.is_open);

        let mut context_menu = ContextMenuState::default();
        context_menu.open(
            gpui::point(px(10.0), px(10.0)),
            vec![ContextMenuEntry::Item(ContextMenuItem::new(
                "inspect", "Inspect",
            ))],
        );
        let (ctx_action, _) = context_menu.handle_key("escape");
        assert_eq!(ctx_action, ContextMenuKeyAction::Closed);
        assert!(!context_menu.is_open);
        assert!(drawer.is_open);

        let mut tabs = WorkspaceTabsState::default();
        tabs.open_picker();
        let picker_action = tabs.handle_picker_key("escape", None, false);
        assert_eq!(picker_action, PickerKeyAction::Closed);
        assert!(!tabs.picker_open);
        assert!(drawer.is_open);

        let mut deployments = DeploymentsState::default();
        deployments.table.toggle_sort("name");
        let sort_before = deployments.table.sort.clone();
        drawer.close();
        assert!(!drawer.is_open);
        assert_eq!(deployments.table.sort, sort_before);
    }

    #[test]
    fn test_disconnect_resets_workspace_tabs_drawer_sidebar_and_resource_caches_without_stale_data()
    {
        use crate::components::ResourceDrawerTabId;

        let mut state = ClusterSwitcherState::new();
        let mut sidebar = SidebarLayoutState::new();
        let mut overview = ClusterOverviewState::default();
        let mut workloads_overview = WorkloadsOverviewState::default();
        let mut namespaces = NamespaceFilterState::default();
        let mut namespaces_view = NamespacesViewState::default();
        let mut deployments = DeploymentsState::default();
        let mut statefulsets = StatefulSetsState::default();
        let mut daemonsets = DaemonSetsState::default();
        let mut replicasets = ReplicaSetsState::default();
        let mut jobs = JobsState::default();
        let mut cronjobs = CronJobsState::default();
        let mut pods = PodsState::default();
        let mut services = ServicesState::default();
        let mut ingresses = IngressesState::default();
        let mut configmaps = ConfigMapsState::default();
        let mut secrets = SecretsState::default();
        let mut persistent_volumes = PersistentVolumesState::default();
        let mut persistent_volume_claims = PersistentVolumeClaimsState::default();
        let mut network_policies = NetworkPoliciesState::default();
        let mut storage_overview = StorageOverviewState::default();
        let mut pod_security = PodSecurityState::default();
        let mut resource_quotas = ResourceQuotasState::default();
        let mut list_views = ClusterListViews::default();
        let mut nodes = NodesState::default();
        let mut events = EventsState::default();
        let mut tabs = WorkspaceTabsState::default();
        let mut drawer = ResourceDrawerState::default();

        let clusters = vec![orbit_core::kubernetes::models::ClusterInfo {
            id: "ctx-a".to_string(),
            name: "ctx-a".to_string(),
            alias: None,
            server_url: Some("https://10.0.0.1".to_string()),
            status: "connected".to_string(),
            tags: Vec::new(),
            color: None,
            description: None,
        }];
        let profile = orbit_core::kubernetes::models::UserProfileInfo {
            user_name: Some("admin".to_string()),
            cluster_name: Some("ctx-a".to_string()),
            active_context: Some("ctx-a".to_string()),
            auth_type: "token".to_string(),
            server_url: Some("https://10.0.0.1".to_string()),
            k8s_version: Some("v1.31.2".to_string()),
            kubeconfig_paths: Vec::new(),
        };

        let _ = apply_cluster_event_with_tabs(
            &mut state,
            &mut sidebar,
            &mut overview,
            &mut workloads_overview,
            &mut namespaces,
            &mut namespaces_view,
            &mut deployments,
            &mut statefulsets,
            &mut daemonsets,
            &mut replicasets,
            &mut jobs,
            &mut cronjobs,
            &mut pods,
            &mut services,
            &mut ingresses,
            &mut configmaps,
            &mut secrets,
            &mut persistent_volumes,
            &mut persistent_volume_claims,
            &mut network_policies,
            &mut storage_overview,
            &mut pod_security,
            &mut resource_quotas,
            &mut list_views,
            &mut nodes,
            &mut events,
            &mut tabs,
            ClusterBridgeEvent::ClusterConnected {
                outcome: orbit_core::kubernetes::manager::ClusterSwitchOutcome {
                    context_name: "ctx-a".to_string(),
                    healthy: true,
                    k8s_version: Some("v1.31.2".to_string()),
                    error_message: None,
                },
                clusters: clusters.clone(),
                profile,
            },
        );
        sidebar.select_view(WorkspaceView::Pods);
        let _ = pods.begin_load_for_cluster("ctx-a");
        let gen_before = pods.generation;
        let _ = drawer.open_or_switch(
            Some("ctx-a"),
            ResourceDrawerTarget::Deployment {
                name: "web".to_string(),
                namespace: "default".to_string(),
            },
            ResourceDrawerTabId::Overview,
        );
        assert!(tabs.active_cluster_id().is_some());
        assert!(drawer.is_open);

        let cmd = disconnect_cluster_with_state(
            &mut state,
            &mut sidebar,
            &mut overview,
            &mut workloads_overview,
            &mut namespaces,
            &mut namespaces_view,
            &mut deployments,
            &mut statefulsets,
            &mut daemonsets,
            &mut replicasets,
            &mut jobs,
            &mut cronjobs,
            &mut pods,
            &mut services,
            &mut ingresses,
            &mut configmaps,
            &mut secrets,
            &mut persistent_volumes,
            &mut persistent_volume_claims,
            &mut network_policies,
            &mut storage_overview,
            &mut pod_security,
            &mut resource_quotas,
            &mut list_views,
            &mut nodes,
            &mut events,
            &mut tabs,
            &mut drawer,
        );
        assert_eq!(cmd, Some(ClusterBridgeCommand::DisconnectCluster));
        assert!(state.active_cluster_id.is_none());
        assert_eq!(state.connection_state, ClusterConnectionState::Disconnected);
        assert_eq!(sidebar.active_category, SidebarCategory::Clusters);
        assert!(tabs.active_cluster_id().is_none());
        assert!(tabs.activate_cluster("ctx-a").is_some());
        tabs.clear_active_cluster();
        assert!(!drawer.is_open);

        // Late PodsLoaded from the old generation must be ignored
        let _ = apply_cluster_event_with_tabs(
            &mut state,
            &mut sidebar,
            &mut overview,
            &mut workloads_overview,
            &mut namespaces,
            &mut namespaces_view,
            &mut deployments,
            &mut statefulsets,
            &mut daemonsets,
            &mut replicasets,
            &mut jobs,
            &mut cronjobs,
            &mut pods,
            &mut services,
            &mut ingresses,
            &mut configmaps,
            &mut secrets,
            &mut persistent_volumes,
            &mut persistent_volume_claims,
            &mut network_policies,
            &mut storage_overview,
            &mut pod_security,
            &mut resource_quotas,
            &mut list_views,
            &mut nodes,
            &mut events,
            &mut tabs,
            ClusterBridgeEvent::PodsLoaded {
                cluster_id: "ctx-a".to_string(),
                generation: gen_before,
                pods: Vec::new(),
            },
        );
        assert!(pods.pods.is_empty());

        // ClusterDisconnected event keeps tabs cleared and sidebar on Clusters
        let _ = apply_cluster_event_with_tabs(
            &mut state,
            &mut sidebar,
            &mut overview,
            &mut workloads_overview,
            &mut namespaces,
            &mut namespaces_view,
            &mut deployments,
            &mut statefulsets,
            &mut daemonsets,
            &mut replicasets,
            &mut jobs,
            &mut cronjobs,
            &mut pods,
            &mut services,
            &mut ingresses,
            &mut configmaps,
            &mut secrets,
            &mut persistent_volumes,
            &mut persistent_volume_claims,
            &mut network_policies,
            &mut storage_overview,
            &mut pod_security,
            &mut resource_quotas,
            &mut list_views,
            &mut nodes,
            &mut events,
            &mut tabs,
            ClusterBridgeEvent::ClusterDisconnected {
                previous_cluster_id: Some("ctx-a".to_string()),
                clusters,
            },
        );
        assert!(tabs.active_cluster_id().is_none());
        assert_eq!(sidebar.active_category, SidebarCategory::Clusters);
    }

    #[test]
    fn test_build_cluster_sidebar_context_menu_entries_for_active_and_inactive_clusters() {
        let inactive = build_cluster_sidebar_context_menu_entries(
            false,
            false,
            |_w, _cx| {},
            |_w, _cx| {},
            |_w, _cx| {},
        );
        assert_eq!(inactive.len(), 1);
        match &inactive[0] {
            ContextMenuEntry::Item(item) => {
                assert_eq!(item.label.as_ref(), "Connect");
                assert!(!item.danger);
            }
            ContextMenuEntry::Separator => panic!("expected Connect item"),
        }

        let active = build_cluster_sidebar_context_menu_entries(
            true,
            true,
            |_w, _cx| {},
            |_w, _cx| {},
            |_w, _cx| {},
        );
        assert_eq!(active.len(), 3);
        match &active[0] {
            ContextMenuEntry::Item(item) => {
                assert_eq!(item.label.as_ref(), "Reconnect");
                assert!(!item.disabled);
                assert!(!item.danger);
            }
            ContextMenuEntry::Separator => panic!("expected Reconnect item"),
        }
        assert!(matches!(active[1], ContextMenuEntry::Separator));
        match &active[2] {
            ContextMenuEntry::Item(item) => {
                assert_eq!(item.label.as_ref(), "Disconnect");
                assert!(item.danger);
                assert!(item.on_select.is_some());
            }
            ContextMenuEntry::Separator => panic!("expected Disconnect item"),
        }
    }

    #[test]
    fn test_header_dropdown_and_sidebar_wire_disconnect_action() {
        let mut state = ClusterSwitcherState::new();
        assert!(!dropdown_can_disconnect(&state));

        state.connection_state = ClusterConnectionState::Connecting {
            cluster_id: "ctx-1".to_string(),
            display_name: "Ctx 1".to_string(),
        };
        assert!(dropdown_can_disconnect(&state));

        state.connection_state = ClusterConnectionState::Unreachable {
            cluster_id: "ctx-1".to_string(),
            display_name: "Ctx 1".to_string(),
            server_url: None,
            error_message: "down".to_string(),
        };
        assert!(dropdown_can_disconnect(&state));

        state.connection_state = ClusterConnectionState::Error {
            cluster_id: None,
            display_name: None,
            error_message: "bridge error".to_string(),
        };
        assert!(!dropdown_can_disconnect(&state));

        state.connection_state = ClusterConnectionState::Error {
            cluster_id: Some("ctx-1".to_string()),
            display_name: Some("Ctx 1".to_string()),
            error_message: "bad cert".to_string(),
        };
        assert!(dropdown_can_disconnect(&state));

        state.active_cluster_id = Some("ctx-1".to_string());
        state.connection_state = ClusterConnectionState::Connected {
            cluster_id: "ctx-1".to_string(),
            display_name: "Ctx 1".to_string(),
            k8s_version: None,
        };
        assert!(dropdown_can_disconnect(&state));
    }

    #[derive(Default)]
    struct TestWorkloadActionTargets {
        drawer: ResourceDrawerState,
        deployments: DeploymentsState,
        statefulsets: StatefulSetsState,
        daemonsets: DaemonSetsState,
        replicasets: ReplicaSetsState,
        pods: PodsState,
        cronjobs: CronJobsState,
        jobs: JobsState,
        services: ServicesState,
        ingresses: IngressesState,
        configmaps: ConfigMapsState,
        secrets: SecretsState,
        network_policies: NetworkPoliciesState,
        resource_quotas: ResourceQuotasState,
        namespaces: NamespaceFilterState,
        persistent_volumes: PersistentVolumesState,
        persistent_volume_claims: PersistentVolumeClaimsState,
        nodes: NodesState,
        events: EventsState,
    }

    impl TestWorkloadActionTargets {
        fn with_cluster(cluster_id: &str) -> Self {
            let mut this = Self::default();
            let cid = Some(cluster_id.to_string());
            this.deployments.cluster_id = cid.clone();
            this.statefulsets.cluster_id = cid.clone();
            this.daemonsets.cluster_id = cid.clone();
            this.replicasets.cluster_id = cid.clone();
            this.pods.cluster_id = cid.clone();
            this.cronjobs.cluster_id = cid.clone();
            this.jobs.cluster_id = cid.clone();
            this.services.cluster_id = cid.clone();
            this.ingresses.cluster_id = cid.clone();
            this.configmaps.cluster_id = cid.clone();
            this.secrets.cluster_id = cid.clone();
            this.network_policies.cluster_id = cid.clone();
            this.resource_quotas.cluster_id = cid.clone();
            this.namespaces.cluster_id = cid.clone();
            this.persistent_volumes.cluster_id = cid.clone();
            this.persistent_volume_claims.cluster_id = cid.clone();
            this.nodes.cluster_id = cid.clone();
            this.events.cluster_id = cid;
            this
        }

        fn as_targets(&mut self) -> WorkloadActionRefreshTargets<'_> {
            WorkloadActionRefreshTargets {
                drawer: &mut self.drawer,
                deployments: &mut self.deployments,
                statefulsets: &mut self.statefulsets,
                daemonsets: &mut self.daemonsets,
                replicasets: &mut self.replicasets,
                pods: &mut self.pods,
                cronjobs: &mut self.cronjobs,
                jobs: &mut self.jobs,
                services: &mut self.services,
                ingresses: &mut self.ingresses,
                configmaps: &mut self.configmaps,
                secrets: &mut self.secrets,
                network_policies: &mut self.network_policies,
                resource_quotas: &mut self.resource_quotas,
                namespaces: &mut self.namespaces,
                persistent_volumes: &mut self.persistent_volumes,
                persistent_volume_claims: &mut self.persistent_volume_claims,
                nodes: &mut self.nodes,
                events: &mut self.events,
            }
        }
    }

    #[test]
    fn test_workload_action_succeeded_closes_modal_shows_toast_and_queues_targeted_refresh() {
        let mut modal = WorkloadActionModalState::default();
        modal.open_scale("prod", "Deployment", "default", "web", 3);
        modal.set_submitting(true);
        let mut toast = ActionToastState::default();

        let mut targets = TestWorkloadActionTargets::with_cluster("prod-ctx");
        targets.deployments.deployments = vec![orbit_core::kubernetes::models::DeploymentInfo {
            kind: "Deployment".to_string(),
            name: "web".to_string(),
            namespace: "default".to_string(),
            status: "Running".to_string(),
            replicas: orbit_core::kubernetes::models::Replicas {
                current: 3,
                desired: 3,
            },
            up_to_date: 3,
            available: 3,
            strategy: None,
            images: Vec::new(),
            containers: Vec::new(),
            age: "1d".to_string(),
            conditions: None,
            min_ready_seconds: 0,
            revision_history: None,
            labels: Default::default(),
            annotations: Default::default(),
            ports: Vec::new(),
        }];

        let scale_event = ClusterBridgeEvent::WorkloadActionSucceeded {
            cluster_id: "prod-ctx".to_string(),
            action: WorkloadActionKind::Scale,
            kind: "Deployment".to_string(),
            namespace: "default".to_string(),
            name: "web".to_string(),
            message: "Scaled Deployment 'default/web' to 5 replica(s).".to_string(),
        };
        let res = apply_workload_action_event(
            Some("prod-ctx"),
            &mut modal,
            &mut toast,
            targets.as_targets(),
            &scale_event,
        )
        .expect("should handle success event");

        assert!(!modal.is_open());
        assert_eq!(res.1, ActionToastKind::Success);
        assert!(toast.current().is_some());
        assert_eq!(
            targets.deployments.deployments.len(),
            1,
            "existing rows must not be cleared"
        );
        assert!(
            !targets.deployments.table.loading,
            "in-place refresh must not show skeleton"
        );
        assert!(matches!(
            res.2.as_slice(),
            [ClusterBridgeCommand::LoadDeployments { cluster_id, .. }] if cluster_id == "prod-ctx"
        ));

        let ds_event = ClusterBridgeEvent::WorkloadActionSucceeded {
            cluster_id: "prod-ctx".to_string(),
            action: WorkloadActionKind::Restart,
            kind: "DaemonSet".to_string(),
            namespace: "kube-system".to_string(),
            name: "fluentd".to_string(),
            message: "Restarted".to_string(),
        };
        let ds_res = apply_workload_action_event(
            Some("prod-ctx"),
            &mut modal,
            &mut toast,
            targets.as_targets(),
            &ds_event,
        )
        .unwrap();
        assert!(matches!(
            ds_res.2.as_slice(),
            [ClusterBridgeCommand::LoadDaemonSets { .. }]
        ));

        let pod_event = ClusterBridgeEvent::WorkloadActionSucceeded {
            cluster_id: "prod-ctx".to_string(),
            action: WorkloadActionKind::RestartPod,
            kind: "Pod".to_string(),
            namespace: "default".to_string(),
            name: "api-0".to_string(),
            message: "Restarted pod".to_string(),
        };
        let pod_res = apply_workload_action_event(
            Some("prod-ctx"),
            &mut modal,
            &mut toast,
            targets.as_targets(),
            &pod_event,
        )
        .unwrap();
        assert!(matches!(
            pod_res.2.as_slice(),
            [ClusterBridgeCommand::LoadPods { .. }]
        ));

        let cj_event = ClusterBridgeEvent::WorkloadActionSucceeded {
            cluster_id: "prod-ctx".to_string(),
            action: WorkloadActionKind::TriggerCronJob,
            kind: "CronJob".to_string(),
            namespace: "batch".to_string(),
            name: "nightly".to_string(),
            message: "Triggered".to_string(),
        };
        let cj_res = apply_workload_action_event(
            Some("prod-ctx"),
            &mut modal,
            &mut toast,
            targets.as_targets(),
            &cj_event,
        )
        .unwrap();
        assert!(matches!(
            cj_res.2.as_slice(),
            [ClusterBridgeCommand::LoadCronJobs { .. }]
        ));
    }

    #[test]
    fn test_workload_action_failed_shows_error_and_clears_submitting_without_dropping_rows() {
        let mut modal = WorkloadActionModalState::default();
        modal.open_scale("prod", "Deployment", "default", "web", 3);
        modal.set_submitting(true);
        let mut toast = ActionToastState::default();

        let mut targets = TestWorkloadActionTargets::with_cluster("prod-ctx");
        targets.deployments.deployments = vec![orbit_core::kubernetes::models::DeploymentInfo {
            kind: "Deployment".to_string(),
            name: "web".to_string(),
            namespace: "default".to_string(),
            status: "Running".to_string(),
            replicas: orbit_core::kubernetes::models::Replicas {
                current: 3,
                desired: 3,
            },
            up_to_date: 3,
            available: 3,
            strategy: None,
            images: Vec::new(),
            containers: Vec::new(),
            age: "1d".to_string(),
            conditions: None,
            min_ready_seconds: 0,
            revision_history: None,
            labels: Default::default(),
            annotations: Default::default(),
            ports: Vec::new(),
        }];

        let failed_event = ClusterBridgeEvent::WorkloadActionFailed {
            cluster_id: "prod-ctx".to_string(),
            action: WorkloadActionKind::Scale,
            kind: "Deployment".to_string(),
            namespace: "default".to_string(),
            name: "web".to_string(),
            message: "Forbidden: RBAC denied".to_string(),
        };
        let res = apply_workload_action_event(
            Some("prod-ctx"),
            &mut modal,
            &mut toast,
            targets.as_targets(),
            &failed_event,
        )
        .expect("should handle failure event");

        assert!(modal.is_open());
        match &modal.dialog {
            WorkloadActionDialogState::Scale(scale) => {
                assert!(!scale.submitting);
                assert_eq!(scale.error.as_deref(), Some("Forbidden: RBAC denied"));
            }
            other => panic!("expected Scale modal, got {other:?}"),
        }
        assert_eq!(res.1, ActionToastKind::Error);
        assert!(res.2.is_empty());
        assert_eq!(targets.deployments.deployments.len(), 1);
    }

    #[test]
    fn test_disconnect_resets_workload_action_modal_and_toast() {
        let mut modal = WorkloadActionModalState::default();
        modal.open_confirm(
            "prod",
            WorkloadActionKind::Restart,
            "Deployment",
            "default",
            "web",
        );
        let mut toast = ActionToastState::default();
        toast.show_success("Scaled", "Done");

        let mut targets = TestWorkloadActionTargets::default();

        let disconnect_event = ClusterBridgeEvent::ClusterDisconnected {
            previous_cluster_id: Some("prod-ctx".to_string()),
            clusters: Vec::new(),
        };
        assert!(
            apply_workload_action_event(
                Some("prod-ctx"),
                &mut modal,
                &mut toast,
                targets.as_targets(),
                &disconnect_event,
            )
            .is_none()
        );
        assert!(!modal.is_open());
        assert!(toast.current().is_none());
    }

    #[test]
    fn test_apply_delete_succeeded_refreshes_resource_and_closes_matching_drawer() {
        let mut modal = WorkloadActionModalState::default();
        modal.open_confirm("prod", WorkloadActionKind::Delete, "Service", "prod", "api");
        modal.set_submitting(true);
        let mut toast = ActionToastState::default();
        let mut targets = TestWorkloadActionTargets::with_cluster("prod-ctx");

        targets.drawer.open_or_switch(
            Some("prod-ctx"),
            ResourceDrawerTarget::Service {
                name: "api".to_string(),
                namespace: "prod".to_string(),
            },
            ResourceDrawerTabId::Overview,
        );
        assert!(targets.drawer.is_open);

        let delete_svc_event = ClusterBridgeEvent::WorkloadActionSucceeded {
            cluster_id: "prod-ctx".to_string(),
            action: WorkloadActionKind::Delete,
            kind: "Service".to_string(),
            namespace: "prod".to_string(),
            name: "api".to_string(),
            message: "Deleted Service 'prod/api'.".to_string(),
        };
        let res = apply_workload_action_event(
            Some("prod-ctx"),
            &mut modal,
            &mut toast,
            targets.as_targets(),
            &delete_svc_event,
        )
        .expect("should handle delete success");

        assert!(!modal.is_open());
        assert!(!targets.drawer.is_open);
        assert_eq!(res.1, ActionToastKind::Success);
        assert!(matches!(
            res.2.as_slice(),
            [ClusterBridgeCommand::LoadServices { cluster_id, .. }] if cluster_id == "prod-ctx"
        ));

        targets.drawer.open_or_switch(
            Some("prod-ctx"),
            ResourceDrawerTarget::Node {
                name: "worker-2".to_string(),
            },
            ResourceDrawerTabId::Overview,
        );
        let delete_node_event = ClusterBridgeEvent::WorkloadActionSucceeded {
            cluster_id: "prod-ctx".to_string(),
            action: WorkloadActionKind::Delete,
            kind: "Node".to_string(),
            namespace: "".to_string(),
            name: "worker-1".to_string(),
            message: "Deleted Node 'worker-1'.".to_string(),
        };
        let node_res = apply_workload_action_event(
            Some("prod-ctx"),
            &mut modal,
            &mut toast,
            targets.as_targets(),
            &delete_node_event,
        )
        .expect("should handle cluster-scoped delete success");
        assert!(
            targets.drawer.is_open,
            "non-matching drawer target must stay open"
        );
        assert!(matches!(
            node_res.2.as_slice(),
            [ClusterBridgeCommand::LoadNodes { cluster_id, .. }] if cluster_id == "prod-ctx"
        ));
    }

    #[test]
    fn test_apply_delete_failed_sets_modal_error_without_stale_loading() {
        let mut modal = WorkloadActionModalState::default();
        modal.open_confirm(
            "prod",
            WorkloadActionKind::Delete,
            "Pod",
            "default",
            "web-0",
        );
        modal.set_submitting(true);
        let mut toast = ActionToastState::default();
        let mut targets = TestWorkloadActionTargets::with_cluster("prod-ctx");

        let failed_event = ClusterBridgeEvent::WorkloadActionFailed {
            cluster_id: "prod-ctx".to_string(),
            action: WorkloadActionKind::Delete,
            kind: "Pod".to_string(),
            namespace: "default".to_string(),
            name: "web-0".to_string(),
            message: "Permission denied: pods \"web-0\" is forbidden".to_string(),
        };
        let res = apply_workload_action_event(
            Some("prod-ctx"),
            &mut modal,
            &mut toast,
            targets.as_targets(),
            &failed_event,
        )
        .expect("should handle delete failure");

        assert!(modal.is_open());
        match &modal.dialog {
            WorkloadActionDialogState::Confirm(confirm) => {
                assert!(!confirm.dialog.submitting);
                assert_eq!(
                    confirm.dialog.error.as_deref(),
                    Some("Permission denied: pods \"web-0\" is forbidden")
                );
                assert!(confirm.can_submit());
            }
            other => panic!("expected Confirm modal, got {other:?}"),
        }
        assert_eq!(res.1, ActionToastKind::Error);
        assert!(res.2.is_empty());
    }

    #[test]
    fn test_open_and_close_logs_tab_dispatches_start_and_stop_log_stream() {
        let mut tabs = WorkspaceTabsState::new();
        tabs.activate_cluster("prod");
        let mut log_tabs = HashMap::new();
        let mut next_id = 1_u64;

        let cmd1 = open_logs_tab_in_state(
            &mut tabs,
            &mut log_tabs,
            &mut next_id,
            Some("prod"),
            "Deployment",
            "default",
            "api",
        );
        assert!(matches!(
            cmd1,
            Some(ClusterBridgeCommand::StartLogStream {
                stream_id: 1,
                tail_lines: 50,
                include_unavailable_pods: false,
                ..
            })
        ));
        assert_eq!(log_tabs.len(), 1);

        // Re-opening same target focuses without starting a duplicate stream
        let cmd2 = open_logs_tab_in_state(
            &mut tabs,
            &mut log_tabs,
            &mut next_id,
            Some("prod"),
            "Deployment",
            "default",
            "api",
        );
        assert!(cmd2.is_none());
        assert_eq!(log_tabs.len(), 1);

        let target = LogsTabTarget {
            kind: "Deployment".to_string(),
            namespace: "default".to_string(),
            name: "api".to_string(),
        };
        let closed = tabs.close_tab(WorkspaceTab::Logs(target));
        let stop_cmds = cleanup_closed_log_tabs_in_state(&mut log_tabs, closed);
        assert_eq!(
            stop_cmds,
            vec![ClusterBridgeCommand::StopLogStream { stream_id: 1 }]
        );
        assert!(log_tabs.is_empty());
    }

    #[test]
    fn test_apply_log_bridge_events_updates_tab_state_and_ignores_stale_stream_id() {
        let mut tabs = WorkspaceTabsState::new();
        tabs.activate_cluster("prod");
        let mut log_tabs = HashMap::new();
        let mut next_id = 10_u64;
        let _ = open_logs_tab_in_state(
            &mut tabs,
            &mut log_tabs,
            &mut next_id,
            Some("prod"),
            "Pod",
            "default",
            "web-0",
        );

        let target = LogsTabTarget {
            kind: "Pod".to_string(),
            namespace: "default".to_string(),
            name: "web-0".to_string(),
        };

        apply_log_bridge_event(
            &mut log_tabs,
            Some("prod"),
            &ClusterBridgeEvent::LogTargetsResolved {
                cluster_id: "prod".to_string(),
                stream_id: 10,
                pods: vec![orbit_core::kubernetes::logs::WorkloadPodTarget {
                    name: "web-0".to_string(),
                    containers: vec!["app".to_string()],
                    phase: "Running".to_string(),
                    is_terminating: false,
                    is_available: true,
                }],
                active_pod_count: 1,
                excluded_unavailable_count: 2,
            },
        );
        apply_log_bridge_event(
            &mut log_tabs,
            Some("prod"),
            &ClusterBridgeEvent::LogLinesReceived {
                cluster_id: "prod".to_string(),
                stream_id: 10,
                pod: "web-0".to_string(),
                container: "app".to_string(),
                lines: vec!["2026-11-25T12:00:00Z ready".to_string()],
            },
        );
        // Stale stream_id should be ignored
        apply_log_bridge_event(
            &mut log_tabs,
            Some("prod"),
            &ClusterBridgeEvent::LogLinesReceived {
                cluster_id: "prod".to_string(),
                stream_id: 999,
                pod: "web-0".to_string(),
                container: "app".to_string(),
                lines: vec!["stale".to_string()],
            },
        );

        let state = &log_tabs[&target];
        assert_eq!(state.excluded_unavailable_count, 2);
        assert_eq!(state.entries.len(), 1);
        assert_eq!(state.entries[0].message, "ready");
    }

    #[test]
    fn test_disconnect_cluster_clears_log_tabs_and_stops_streams() {
        let mut tabs = WorkspaceTabsState::new();
        tabs.activate_cluster("prod");
        let mut log_tabs = HashMap::new();
        let mut next_id = 5_u64;
        let _ = open_logs_tab_in_state(
            &mut tabs,
            &mut log_tabs,
            &mut next_id,
            Some("prod"),
            "Job",
            "batch",
            "nightly",
        );

        let stop_cmds = clear_all_log_tabs_in_state(&mut tabs, &mut log_tabs);
        assert_eq!(
            stop_cmds,
            vec![ClusterBridgeCommand::StopLogStream { stream_id: 5 }]
        );
        assert!(log_tabs.is_empty());
        assert!(tabs.open_tabs().iter().all(|t| t.as_logs().is_none()));
    }

    #[test]
    fn test_port_forward_view_loading_flag_and_tab_selection_dispatches_load() {
        assert!(!is_workspace_view_loading(
            WorkspaceView::PortForward,
            WorkspaceLoadingFlags::default()
        ));
        assert!(is_workspace_view_loading(
            WorkspaceView::PortForward,
            WorkspaceLoadingFlags {
                port_forwards: true,
                ..WorkspaceLoadingFlags::default()
            }
        ));

        let mut tabs = WorkspaceTabsState::new();
        tabs.activate_cluster("prod");
        tabs.open_tab(WorkspaceView::PortForward);
        assert_eq!(tabs.active_view(), Some(WorkspaceView::PortForward));

        let mut pf_state = PortForwardsState::default();
        assert!(pf_state.needs_load_for_cluster("prod"));
        let cmd = pf_state.begin_load_for_cluster("prod");
        assert_eq!(
            cmd,
            ClusterBridgeCommand::LoadPortForwards {
                cluster_id: "prod".to_string(),
                generation: 1,
            }
        );
    }

    #[test]
    fn test_cluster_disconnect_resets_port_forwards_state_and_closes_modal() {
        let mut pf_state = PortForwardsState::default();
        let _ = pf_state.begin_load_for_cluster("prod");
        let pf = orbit_core::kubernetes::port_forward::PersistedPortForward {
            id: "default/Service/api:8080:80".to_string(),
            cluster_id: "prod".to_string(),
            namespace: "default".to_string(),
            kind: "Service".to_string(),
            name: "api".to_string(),
            local_port: 8080,
            remote_port: 80,
        };
        assert!(pf_state.apply_loaded("prod", 1, vec![pf]));

        let mut modal = Some(PortForwardModalState::new_targeted(
            "prod",
            "prod",
            "default",
            "Service",
            "api",
            vec![80],
            Vec::new(),
        ));
        let mut toast = ActionToastState::default();

        let _ = apply_port_forward_event(
            &mut pf_state,
            &mut modal,
            &mut toast,
            &ClusterBridgeEvent::ClusterDisconnected {
                previous_cluster_id: Some("prod".to_string()),
                clusters: Vec::new(),
            },
        );

        assert!(pf_state.cluster_id.is_none());
        assert!(pf_state.port_forwards.is_empty());
        assert!(modal.is_none());
    }

    #[test]
    fn test_port_forward_events_update_shell_state_and_toast() {
        let mut pf_state = PortForwardsState::default();
        let _ = pf_state.begin_load_for_cluster("prod");
        let mut modal = Some(PortForwardModalState::new_targeted(
            "prod",
            "prod",
            "default",
            "Service",
            "api",
            vec![8080],
            Vec::new(),
        ));
        let mut toast = ActionToastState::default();

        let pf = orbit_core::kubernetes::port_forward::PersistedPortForward {
            id: "default/Service/api:8080:8080".to_string(),
            cluster_id: "prod".to_string(),
            namespace: "default".to_string(),
            kind: "Service".to_string(),
            name: "api".to_string(),
            local_port: 8080,
            remote_port: 8080,
        };

        let started = apply_port_forward_event(
            &mut pf_state,
            &mut modal,
            &mut toast,
            &ClusterBridgeEvent::PortForwardStarted {
                cluster_id: "prod".to_string(),
                port_forward: pf.clone(),
            },
        );
        assert!(started.is_some());
        assert!(modal.is_none());
        assert_eq!(pf_state.port_forwards, vec![pf.clone()]);
        assert!(toast.current().is_some());

        let _ = apply_port_forward_event(
            &mut pf_state,
            &mut modal,
            &mut toast,
            &ClusterBridgeEvent::PortForwardStopped {
                cluster_id: "prod".to_string(),
                id: pf.id,
            },
        );
        assert!(pf_state.port_forwards.is_empty());
    }

    #[test]
    fn test_sidebar_utility_theme_cycle_persists_next_mode() {
        assert_eq!(ThemeMode::Dark.next(), ThemeMode::Light);
        assert_eq!(ThemeMode::Light.next(), ThemeMode::System);
        assert_eq!(ThemeMode::System.next(), ThemeMode::Dark);

        assert_eq!(theme_mode_tooltip(ThemeMode::Dark, true), "Theme: Dark");
        assert_eq!(theme_mode_tooltip(ThemeMode::Light, false), "Theme: Light");
        assert_eq!(
            theme_mode_tooltip(ThemeMode::System, true),
            "Theme: System (Dark)"
        );
    }

    #[test]
    fn test_orbit_github_repo_url_is_valid_https() {
        assert_eq!(
            ORBIT_GITHUB_REPO_URL,
            "https://github.com/vantoan1511/orbit"
        );
    }

    #[test]
    fn test_shell_drawer_escape_dismissal_closes_notifications_and_profile() {
        let mut drawer = ActiveShellDrawer::Notifications;
        assert!(drawer.close());
        assert_eq!(drawer, ActiveShellDrawer::None);
        assert!(!drawer.close());

        drawer.toggle(ActiveShellDrawer::UserProfile);
        assert_eq!(drawer, ActiveShellDrawer::UserProfile);
        assert!(drawer.close());
        assert_eq!(drawer, ActiveShellDrawer::None);
    }

    #[test]
    fn test_notification_recorded_from_bridge_events() {
        let ok_event = ClusterBridgeEvent::WorkloadActionSucceeded {
            cluster_id: "prod".to_string(),
            action: WorkloadActionKind::Restart,
            kind: "Deployment".to_string(),
            namespace: "default".to_string(),
            name: "api".to_string(),
            message: "Restarted api".to_string(),
        };
        let (sev, title, msg) = notification_from_bridge_event(&ok_event).expect("should map");
        assert_eq!(sev, NotificationSeverity::Success);
        assert_eq!(title, "Deployment api");
        assert_eq!(msg, "Restarted api");

        let err_event = ClusterBridgeEvent::PortForwardFailed {
            cluster_id: "prod".to_string(),
            id: None,
            message: "Port in use".to_string(),
        };
        let (sev, title, msg) = notification_from_bridge_event(&err_event).expect("should map");
        assert_eq!(sev, NotificationSeverity::Error);
        assert_eq!(title, "Port Forward Failed");
        assert_eq!(msg, "Port in use");

        let noisy_event = ClusterBridgeEvent::PodsLoaded {
            cluster_id: "prod".to_string(),
            generation: 1,
            pods: Vec::new(),
        };
        assert!(notification_from_bridge_event(&noisy_event).is_none());
    }

    #[test]
    fn test_table_container_width_accounts_for_sidebar_state() {
        let mut sidebar = SidebarLayoutState::new();
        sidebar.is_collapsed = false;
        sidebar.panel_width = 240.0;
        assert_eq!(
            table_container_width(1280.0, &sidebar),
            (1280.0 - ACTIVITY_BAR_WIDTH - 240.0).max(320.0)
        );

        sidebar.is_collapsed = true;
        assert_eq!(
            table_container_width(1280.0, &sidebar),
            (1280.0 - ACTIVITY_BAR_WIDTH).max(320.0)
        );
        assert_eq!(table_container_width(200.0, &sidebar), 320.0);

        let cols = vec![
            crate::components::TableColumn::new("name", "Name").flex_width(2.0, px(140.0)),
            crate::components::TableColumn::new("age", "Age").fixed_width(px(80.0)),
        ];
        let mut table = crate::components::DataTableState::new();
        assert!(finish_table_column_resize("pods", &mut table).is_none());

        assert!(table.start_column_resize("name", 300.0, &cols, 680.0));
        assert!(table.update_column_resize(340.0));
        let (id, cfg) = finish_table_column_resize("pods", &mut table).expect("should finish");
        assert_eq!(id, "pods");
        assert!(table.active_resize.is_none());
        assert_eq!(cfg.column_widths.get("name").copied(), Some(640));
    }

    #[test]
    fn test_global_shortcut_guard_blocks_slash_and_r_when_text_input_active() {
        assert_eq!(
            resolve_global_workspace_shortcut("/", Some("/"), false, false, false, true),
            None
        );
        assert_eq!(
            resolve_global_workspace_shortcut("r", Some("r"), false, false, false, true),
            None
        );
        assert_eq!(
            resolve_global_workspace_shortcut("/", Some("/"), false, false, false, false),
            Some(GlobalWorkspaceShortcut::FocusSearch)
        );
        assert_eq!(
            resolve_global_workspace_shortcut("r", Some("r"), false, false, false, false),
            Some(GlobalWorkspaceShortcut::RefreshWorkspace)
        );
        assert_eq!(
            resolve_global_workspace_shortcut("k", Some("k"), true, false, false, true),
            Some(GlobalWorkspaceShortcut::ToggleCommandPalette)
        );
        assert_eq!(
            resolve_global_workspace_shortcut("k", Some("k"), false, true, false, true),
            Some(GlobalWorkspaceShortcut::ToggleCommandPalette)
        );
        assert_eq!(
            resolve_global_workspace_shortcut("b", Some("b"), true, false, false, false),
            Some(GlobalWorkspaceShortcut::ToggleSidebar)
        );
    }

    #[test]
    fn test_shortcut_hint_labels_adapt_to_platform() {
        assert_eq!(primary_shortcut_label_for_os("K", true), "⌘K");
        assert_eq!(primary_shortcut_label_for_os("K", false), "Ctrl+K");
        assert_eq!(primary_shortcut_label_for_os("B", true), "⌘B");
        assert_eq!(primary_shortcut_label_for_os("B", false), "Ctrl+B");
    }

    #[test]
    fn test_settings_workspace_header_and_dropdown_dismissal_rules() {
        assert!(!should_render_workspace_view_header(
            WorkspaceView::Settings,
            false
        ));
        assert!(should_render_workspace_view_header(
            WorkspaceView::Settings,
            true
        ));
        assert!(should_render_workspace_view_header(
            WorkspaceView::Overview,
            false
        ));
    }
}
