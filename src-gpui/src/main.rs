// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod bridge;
pub mod components;
pub mod icons;
pub mod navigation;
mod state;
pub mod theme;
pub mod views;

use gpui::{
    AnyElement, App, Application, Bounds, ClipboardItem, Context, CursorStyle, FocusHandle,
    FontWeight, KeyDownEvent, MouseButton, MouseDownEvent, MouseMoveEvent, Pixels, Point, Rgba,
    SharedString, Window, WindowAppearance, WindowBounds, WindowDecorations, WindowOptions, div,
    prelude::*, px, size,
};
use lucide_icons::Icon;
use orbit_core::config::OrbitConfig;
use orbit_core::kubernetes::models::ClusterInfo;
use tokio::sync::mpsc;

use crate::bridge::{ClusterBridge, start_cluster_bridge};
use crate::components::{
    ContextMenuEntry, ContextMenuKeyAction, ContextMenuState, ResourceDrawerProps,
    ResourceDrawerState, ResourceDrawerTabDef, ResourceDrawerTabId, ResourceDrawerTarget,
    TabBarCallbacks, TabContextMenuCallbacks, YamlLoadStatus, YamlViewerProps,
    build_tab_context_menu_entries, render_context_menu_overlay, render_resource_drawer,
    render_tab_resource_picker_overlay, render_workspace_tab_bar, render_yaml_viewer,
};
use crate::icons::render_icon;
use crate::navigation::{ACTIVITY_BAR_WIDTH, SidebarCategory, SidebarLayoutState, WorkspaceView};
use crate::state::{
    ClusterBridgeCommand, ClusterBridgeEvent, ClusterConnectionState, ClusterOverviewState,
    ClusterSwitcherState, ConfigMapsState, CronJobsState, DaemonSetsState, DeploymentsState,
    EventTypeFilter,
    EventsLoadStatus, EventsState, IngressesState, JobsState, NamespaceFilterState,
    NamespaceLoadStatus, NamespacesViewState, NetworkPoliciesState, NodesState,
    PersistentVolumeClaimsState, PersistentVolumesState, PickerKeyAction, PodsState,
    ReplicaSetsState, SecretsState, ServicesState, StatefulSetsState,
    TabDropPosition, WorkloadsOverviewState, WorkspaceTabsState, calculate_tab_drop_index,
};
use crate::theme::{
    AccentColor, OrbitTheme, ThemeMode, UiStyle, apply_text_step, rgb_hex,
    sync_native_titlebar_appearance,
};

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

fn parse_hex_color(hex: Option<&str>) -> Option<Rgba> {
    let trimmed = hex?.trim();
    let raw = trimmed.strip_prefix('#').unwrap_or(trimmed);
    if raw.len() == 6 {
        u32::from_str_radix(raw, 16).ok().map(rgb_hex)
    } else {
        None
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
    parse_hex_color(cluster.color.as_deref()).unwrap_or_else(|| {
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
    pub configmaps: bool,
    pub secrets: bool,
    pub persistent_volumes: bool,
    pub persistent_volume_claims: bool,
    pub network_policies: bool,
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
        WorkspaceView::ConfigMaps => loading.configmaps,
        WorkspaceView::Secrets => loading.secrets,
        WorkspaceView::PersistentVolumes => loading.persistent_volumes,
        WorkspaceView::PersistentVolumeClaims => loading.persistent_volume_claims,
        WorkspaceView::NetworkPolicies => loading.network_policies,
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
    if view == WorkspaceView::NetworkPolicies
        && network_policies.needs_load_for_cluster(cluster_id)
    {
        return Some(network_policies.begin_load_for_cluster(cluster_id));
    }
    if view == WorkspaceView::Nodes && nodes.needs_load_for_cluster(cluster_id) {
        return Some(nodes.begin_load_for_cluster(cluster_id));
    }
    if view == WorkspaceView::Events && events.needs_load_for_cluster(cluster_id) {
        return Some(events.begin_load_for_cluster(cluster_id));
    }
    None
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
    nodes: &mut NodesState,
    events: &mut EventsState,
    tabs: &mut WorkspaceTabsState,
    event: ClusterBridgeEvent,
) -> Vec<ClusterBridgeCommand> {
    let is_connected = matches!(&event, ClusterBridgeEvent::ClusterConnected { .. });
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
                nodes,
                events,
                restored,
            ) {
                if !commands.contains(&load_cmd) {
                    commands.push(load_cmd);
                }
            }
        }
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

    let active = tabs.active_tab();
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
    nodes: &mut NodesState,
    events: &mut EventsState,
    event: ClusterBridgeEvent,
) -> Vec<ClusterBridgeCommand> {
    let mut commands = Vec::new();
    match event {
        ClusterBridgeEvent::ClusterConnected { .. } => {
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
            }
        }
        ClusterBridgeEvent::ClusterUnreachable { .. }
        | ClusterBridgeEvent::ConnectionError { .. } => {
            state.apply_event(event);
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
            nodes.reset_for_disconnected();
            events.reset_for_disconnected();
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
        ClusterBridgeEvent::ResourceYamlLoaded { .. }
        | ClusterBridgeEvent::ResourceYamlLoadFailed { .. }
        | ClusterBridgeEvent::SettingsSaveFailed { .. } => {}
    }
    commands
}

pub fn apply_drawer_lifecycle_for_event(
    drawer: &mut ResourceDrawerState,
    event: &ClusterBridgeEvent,
) {
    match event {
        ClusterBridgeEvent::ClusterConnected { .. }
        | ClusterBridgeEvent::ClusterUnreachable { .. }
        | ClusterBridgeEvent::ConnectionError { .. } => {
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
    configmaps: ConfigMapsState,
    secrets: SecretsState,
    persistent_volumes: PersistentVolumesState,
    persistent_volume_claims: PersistentVolumeClaimsState,
    network_policies: NetworkPoliciesState,
    nodes: NodesState,
    events: EventsState,
    drawer: ResourceDrawerState,
    tabs: WorkspaceTabsState,
    focus_handle: FocusHandle,
    namespace_search_focus: FocusHandle,
    picker_search_focus: FocusHandle,
    command_tx: mpsc::UnboundedSender<ClusterBridgeCommand>,
    enable_experience_features: bool,
    resolved_system_is_dark: bool,
    settings_error: Option<String>,
    context_menu: ContextMenuState,
}

impl OrbitShell {
    pub const WINDOW_TITLE: &'static str = "Orbit";
    pub const DEFAULT_WIDTH: f32 = 1200.0;
    pub const DEFAULT_HEIGHT: f32 = 800.0;
    pub const HEADER_HEIGHT: f32 = 48.0;
    pub const FOOTER_HEIGHT: f32 = 26.0;

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
                        apply_drawer_lifecycle_for_event(&mut shell.drawer, &event);
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
                            &mut shell.nodes,
                            &mut shell.events,
                            &mut shell.tabs,
                            event,
                        ) {
                            shell.dispatch_command(cmd);
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

        Self {
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
            configmaps: ConfigMapsState::default(),
            secrets: SecretsState::default(),
            persistent_volumes: PersistentVolumesState::default(),
            persistent_volume_claims: PersistentVolumeClaimsState::default(),
            network_policies: NetworkPoliciesState::default(),
            nodes: NodesState::default(),
            events: EventsState::default(),
            drawer: ResourceDrawerState::default(),
            tabs: WorkspaceTabsState::from_config(config),
            focus_handle,
            namespace_search_focus: cx.focus_handle(),
            picker_search_focus: cx.focus_handle(),
            command_tx,
            enable_experience_features: config.enable_experience_features(),
            resolved_system_is_dark,
            settings_error: None,
            context_menu: ContextMenuState::new(),
        }
    }

    fn open_context_menu(
        &mut self,
        anchor: Point<Pixels>,
        entries: Vec<ContextMenuEntry>,
        cx: &mut Context<Self>,
    ) {
        self.state.close_dropdown();
        self.namespaces.close_dropdown();
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
        cx.notify();
    }

    fn dispatch_command(&self, cmd: ClusterBridgeCommand) {
        if let Err(err) = self.command_tx.send(cmd) {
            tracing::warn!(error = %err, "Failed to send command to cluster bridge");
        }
    }

    fn select_workspace_view(&mut self, view: WorkspaceView, cx: &mut Context<Self>) {
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
            &mut self.nodes,
            &mut self.events,
            &mut self.tabs,
            view,
        ) {
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
        let active = self.tabs.active_tab();
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
            &mut self.nodes,
            &mut self.events,
            active,
        ) {
            self.dispatch_command(cmd);
        }
        self.persist_tabs();
    }

    fn activate_tab(&mut self, view: WorkspaceView, cx: &mut Context<Self>) {
        let prev_view = self.sidebar.active_view;
        self.tabs.set_active_tab(view);
        self.sync_active_tab_view(prev_view);
        cx.notify();
    }

    fn close_tab(&mut self, view: WorkspaceView, cx: &mut Context<Self>) {
        let prev_view = self.sidebar.active_view;
        self.tabs.close_tab(view);
        self.sync_active_tab_view(prev_view);
        cx.notify();
    }

    fn close_other_tabs(&mut self, view: WorkspaceView, cx: &mut Context<Self>) {
        let prev_view = self.sidebar.active_view;
        self.tabs.close_other_tabs(view);
        self.sync_active_tab_view(prev_view);
        cx.notify();
    }

    fn close_tabs_to_right(&mut self, view: WorkspaceView, cx: &mut Context<Self>) {
        let prev_view = self.sidebar.active_view;
        self.tabs.close_tabs_to_right(view);
        self.sync_active_tab_view(prev_view);
        cx.notify();
    }

    fn close_all_tabs(&mut self, cx: &mut Context<Self>) {
        let prev_view = self.sidebar.active_view;
        self.tabs.close_all_tabs();
        self.sync_active_tab_view(prev_view);
        cx.notify();
    }

    fn move_tab_left(&mut self, view: WorkspaceView, cx: &mut Context<Self>) {
        self.tabs.move_tab_left(view);
        self.persist_tabs();
        cx.notify();
    }

    fn move_tab_right(&mut self, view: WorkspaceView, cx: &mut Context<Self>) {
        self.tabs.move_tab_right(view);
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
        let to_idx = calculate_tab_drop_index(from_idx, hover_idx, pos);
        self.tabs.reorder_tab(from_idx, to_idx);
        self.tabs.drop_target = None;
        self.tabs.dragging_index = None;
        self.persist_tabs();
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
                let matched_events =
                    views::persistent_volumes::filter_events_for_persistent_volume(
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
                configmaps: self.configmaps.status == crate::state::ConfigMapsLoadStatus::Loading,
                secrets: self.secrets.status == crate::state::SecretsLoadStatus::Loading,
                persistent_volumes: self.persistent_volumes.status
                    == crate::state::PersistentVolumesLoadStatus::Loading,
                persistent_volume_claims: self.persistent_volume_claims.status
                    == crate::state::PersistentVolumeClaimsLoadStatus::Loading,
                network_policies: self.network_policies.status
                    == crate::state::NetworkPoliciesLoadStatus::Loading,
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

    fn select_theme_mode(&mut self, mode: ThemeMode, window: &mut Window, cx: &mut Context<Self>) {
        let updated = OrbitTheme::get(cx).with_mode(mode, window.appearance());
        apply_theme(updated, window, cx);
        self.settings_error = None;
        self.dispatch_command(ClusterBridgeCommand::SaveThemeMode {
            mode: mode.id().to_string(),
        });
        cx.notify();
    }

    fn select_ui_style(&mut self, style: UiStyle, window: &mut Window, cx: &mut Context<Self>) {
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
                appears_transparent: false,
                traffic_light_position: None,
            }),
            window_decorations: Some(WindowDecorations::Server),
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
                    }),
            )
            .into_any_element()
    }

    fn render_clusters_sidebar_content(&self, cx: &mut Context<Self>) -> AnyElement {
        let theme = OrbitTheme::get(cx).clone();
        let nav = theme.nav_selection();
        let hover_bg = theme.surface.bg_hover;
        let mono_font = theme.typography.mono_font(FontWeight::NORMAL);

        if self.state.clusters.is_empty() {
            return apply_text_step(div(), theme.typography.size_xs)
                .p_3()
                .text_color(theme.text.muted)
                .child("No kubeconfig contexts found.")
                .into_any_element();
        }

        let mut list = div().flex().flex_col().gap_1();
        for (idx, cluster) in self.state.clusters.iter().enumerate() {
            let cluster_id = cluster.id.clone();
            let is_active = self.state.active_cluster_id.as_deref() == Some(cluster_id.as_str());
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
                        this.select_cluster(&cluster_id, cx);
                    }))
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
                        .top(px(Self::HEADER_HEIGHT + 4.0))
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
                        .child(list),
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
                        .top(px(Self::HEADER_HEIGHT + 4.0))
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
        let (cat_label, _) = view.breadcrumb();
        let resource_kind = should_show_header_kind_badge(view)
            .then(|| view.resource_kind())
            .flatten();

        div()
            .flex()
            .flex_col()
            .gap_1()
            .child(
                apply_text_step(div(), theme.typography.size_xs)
                    .font_weight(FontWeight::SEMIBOLD)
                    .text_color(theme.text.muted)
                    .child(cat_label.to_ascii_uppercase()),
            )
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_2p5()
                    .child(
                        apply_text_step(div(), theme.typography.size_xl)
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
                apply_text_step(div(), theme.typography.size_sm)
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
        };

        let mode_entity = cx.entity().clone();
        let style_entity = cx.entity().clone();
        let accent_entity = cx.entity().clone();
        let exp_entity = cx.entity().clone();
        let reset_entity = cx.entity().clone();

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
        &self,
        cluster_id: &str,
        display_name: &str,
        k8s_version: Option<&str>,
        theme: &OrbitTheme,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let version_label = k8s_version.unwrap_or("Connected").to_string();

        if self.sidebar.active_view == WorkspaceView::Events {
            let sort_entity = cx.entity().clone();
            let on_sort = move |col_id: SharedString,
                                _event: &gpui::ClickEvent,
                                _window: &mut Window,
                                cx: &mut App| {
                sort_entity.update(cx, |shell, cx| {
                    shell.events.table.toggle_sort(col_id.as_ref());
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
                let entries = views::events::build_event_context_menu_entries(
                    ev,
                    copy_obj,
                    copy_msg,
                    filter_ns,
                    refresh_cb,
                    view_details,
                    view_yaml,
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
                    let entries = views::deployments::build_deployment_context_menu_entries(
                        deployment,
                        copy_name,
                        filter_ns,
                        refresh_cb,
                        view_details,
                        view_yaml,
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

            return views::deployments::render_deployments_view(
                &self.deployments,
                &self.namespaces,
                theme,
                on_sort,
                on_row_click,
                on_row_context_menu,
                on_refresh,
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
                    let entries = views::statefulsets::build_statefulset_context_menu_entries(
                        statefulset,
                        copy_name,
                        filter_ns,
                        refresh_cb,
                        view_details,
                        view_yaml,
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

            return views::statefulsets::render_statefulsets_view(
                &self.statefulsets,
                &self.namespaces,
                theme,
                on_sort,
                on_row_click,
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
                    let entries = views::replicasets::build_replicaset_context_menu_entries(
                        replicaset,
                        copy_name,
                        filter_ns,
                        refresh_cb,
                        view_details,
                        view_yaml,
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

            return views::replicasets::render_replicasets_view(
                &self.replicasets,
                &self.namespaces,
                theme,
                on_sort,
                on_row_click,
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
                    let entries = views::daemonsets::build_daemonset_context_menu_entries(
                        daemonset,
                        copy_name,
                        filter_ns,
                        refresh_cb,
                        view_details,
                        view_yaml,
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

            return views::daemonsets::render_daemonsets_view(
                &self.daemonsets,
                &self.namespaces,
                theme,
                on_sort,
                on_row_click,
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
                let entries = views::jobs::build_job_context_menu_entries(
                    job,
                    copy_name,
                    filter_ns,
                    refresh_cb,
                    view_details,
                    view_yaml,
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

            return views::jobs::render_jobs_view(
                &self.jobs,
                &self.namespaces,
                theme,
                on_sort,
                on_row_click,
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
                let entries = views::cronjobs::build_cronjob_context_menu_entries(
                    cj,
                    copy_name,
                    filter_ns,
                    refresh_cb,
                    view_details,
                    view_yaml,
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

            return views::cronjobs::render_cronjobs_view(
                &self.cronjobs,
                &self.namespaces,
                theme,
                on_sort,
                on_row_click,
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
                let entries = views::services::build_service_context_menu_entries(
                    svc,
                    copy_name,
                    filter_ns,
                    refresh_cb,
                    view_details,
                    view_yaml,
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

            return views::services::render_services_view(
                &self.services,
                &self.namespaces,
                theme,
                on_sort,
                on_row_click,
                on_row_context_menu,
                on_type_filter,
                on_refresh,
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
                let entries = views::ingresses::build_ingress_context_menu_entries(
                    ing,
                    copy_name,
                    filter_ns,
                    refresh_cb,
                    view_details,
                    view_yaml,
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

            return views::ingresses::render_ingresses_view(
                &self.ingresses,
                &self.namespaces,
                theme,
                on_sort,
                on_row_click,
                on_row_context_menu,
                on_refresh,
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
                let entries = views::configmaps::build_configmap_context_menu_entries(
                    cm,
                    copy_name,
                    filter_ns,
                    refresh_cb,
                    view_details,
                    view_yaml,
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

            return views::configmaps::render_configmaps_view(
                &self.configmaps,
                &self.namespaces,
                theme,
                on_sort,
                on_row_click,
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
                let entries = views::secrets::build_secret_context_menu_entries(
                    sec,
                    copy_name,
                    filter_ns,
                    refresh_cb,
                    view_details,
                    view_yaml,
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

            return views::secrets::render_secrets_view(
                &self.secrets,
                &self.namespaces,
                theme,
                on_sort,
                on_row_click,
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
                let entries = views::pods::build_pod_context_menu_entries(
                    pod,
                    copy_name,
                    filter_ns,
                    refresh_cb,
                    view_details,
                    view_yaml,
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

            return views::pods::render_pods_view(
                &self.pods,
                &self.namespaces,
                theme,
                on_sort,
                on_row_click,
                on_row_context_menu,
                on_refresh,
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
                let entries = views::namespaces::build_namespace_context_menu_entries(
                    ns,
                    copy_name,
                    filter_ns,
                    refresh_cb,
                    view_details,
                    view_yaml,
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

            return views::namespaces::render_namespaces_view(
                &self.namespaces,
                &self.namespaces_view,
                theme,
                on_sort,
                on_row_click,
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
                    shell
                        .persistent_volumes
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
                    shell.persistent_volumes.table.selected_row_key = Some(row_key.to_string());
                    let target = ResourceDrawerTarget::PersistentVolume {
                        name: row_key.to_string(),
                    };
                    shell.open_resource_drawer(target, ResourceDrawerTabId::Overview, cx);
                });
            };

            let ctx_entity = cx.entity().clone();
            let on_row_context_menu = move |row_key: SharedString,
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
                let entries =
                    views::persistent_volumes::build_persistent_volume_context_menu_entries(
                        pv,
                        copy_name,
                        refresh_cb,
                        view_details,
                        view_yaml,
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

            return views::persistent_volumes::render_persistent_volumes_view(
                &self.persistent_volumes,
                theme,
                on_sort,
                on_row_click,
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
            let on_row_context_menu = move |row_key: SharedString,
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
                let entries =
                    views::persistent_volume_claims::build_persistent_volume_claim_context_menu_entries(
                        pvc,
                        copy_name,
                        filter_ns,
                        refresh_cb,
                        view_details,
                        view_yaml,
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

            return views::persistent_volume_claims::render_persistent_volume_claims_view(
                &self.persistent_volume_claims,
                &self.namespaces,
                theme,
                on_sort,
                on_row_click,
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
                let entries = views::network_policies::build_network_policy_context_menu_entries(
                    policy,
                    copy_name,
                    filter_ns,
                    refresh_cb,
                    view_details,
                    view_yaml,
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

            return views::network_policies::render_network_policies_view(
                &self.network_policies,
                &self.namespaces,
                theme,
                on_sort,
                on_row_click,
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
                let entries = views::nodes::build_node_context_menu_entries(
                    node,
                    copy_name,
                    refresh_cb,
                    view_details,
                    view_yaml,
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

            return views::nodes::render_nodes_view(
                &self.nodes,
                theme,
                on_sort,
                on_row_click,
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

    fn dismiss_overlays(&mut self) {
        self.dismiss_context_menu();
        self.tabs.close_picker();
        self.state.dropdown_open = false;
        self.namespaces.close_dropdown();
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
                    if shell.tabs.drop_target != Some((idx, pos)) {
                        shell.tabs.drop_target = Some((idx, pos));
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

    fn render_workspace(&self, window_width: f32, cx: &mut Context<Self>) -> AnyElement {
        let theme = OrbitTheme::get(cx).clone();

        let body = if self.sidebar.active_view == WorkspaceView::Settings {
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
                    views::welcome::render_welcome_view(
                        &self.state.clusters,
                        self.state.kubeconfig_error.as_deref(),
                        &theme,
                        on_select,
                        on_refresh,
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
                        self.render_connected_workspace(
                            cluster_id,
                            display_name,
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
                        views::offline::render_offline_view(&model, &theme, on_retry, on_switch)
                    } else {
                        div().into_any_element()
                    }
                }
            }
        };

        let fill_viewport = matches!(
            self.sidebar.active_view,
            WorkspaceView::Namespaces
                | WorkspaceView::Deployments
                | WorkspaceView::Pods
                | WorkspaceView::Services
                | WorkspaceView::Ingresses
                | WorkspaceView::ConfigMaps
                | WorkspaceView::Secrets
                | WorkspaceView::PersistentVolumes
                | WorkspaceView::PersistentVolumeClaims
                | WorkspaceView::NetworkPolicies
                | WorkspaceView::Nodes
                | WorkspaceView::Events
        ) && matches!(
            self.state.connection_state,
            ClusterConnectionState::Connected { .. }
        );

        let container = div().id("orbit-workspace-scroll").flex_1();
        let container = if fill_viewport {
            container.min_h(px(0.0)).overflow_hidden()
        } else {
            container.overflow_y_scroll()
        };

        let workspace_main = container
            .p_6()
            .bg(theme.surface.bg_app)
            .flex()
            .flex_col()
            .gap_6()
            .child(self.render_workspace_view_header(&theme))
            .child(body);

        let drawer = self.render_active_drawer(window_width, &theme, cx);

        div()
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

        div()
            .id("orbit-shell-root")
            .track_focus(&self.focus_handle)
            .on_key_down(cx.listener(|this, event: &KeyDownEvent, window, cx| {
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
                if this.namespaces.dropdown_open
                    && this.namespaces.handle_search_key(
                        &event.keystroke.key,
                        event.keystroke.key_char.as_deref(),
                    )
                {
                    cx.notify();
                    return;
                }
                if event.keystroke.key.eq_ignore_ascii_case("escape") && this.drawer.is_open {
                    this.dismiss_resource_drawer(cx);
                    return;
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
                let toggle_pressed = (event.keystroke.modifiers.control
                    || event.keystroke.modifiers.platform)
                    && event.keystroke.key.eq_ignore_ascii_case("b");
                if toggle_pressed {
                    this.sidebar.toggle_collapse();
                    cx.notify();
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
            }))
            .on_mouse_up(
                MouseButton::Left,
                cx.listener(|this, _event, _window, cx| {
                    let mut changed = false;
                    if this.sidebar.is_resizing {
                        this.sidebar.end_resize();
                        changed = true;
                    }
                    if this.drawer.is_resizing {
                        this.drawer.stop_resize();
                        changed = true;
                    }
                    if this.tabs.dragging_index.is_some() || this.tabs.drop_target.is_some() {
                        this.tabs.dragging_index = None;
                        this.tabs.drop_target = None;
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
            .when_some(self.render_dropdown_overlay(cx), |el, overlay| {
                el.child(overlay)
            })
            .when_some(self.render_namespace_dropdown_overlay(cx), |el, overlay| {
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
                    std::rc::Rc::new(move |view, window, cx| {
                        select_entity.update(cx, |shell, cx| {
                            shell.tabs.close_picker();
                            shell.apply_focus_target(
                                focus_target_for_picker_transition(false),
                                window,
                            );
                            shell.select_workspace_view(view, cx);
                        });
                    }),
                    std::rc::Rc::new(move |window, cx| {
                        clear_entity.update(cx, |shell, cx| {
                            shell.tabs.clear_picker_query();
                            shell.apply_focus_target(
                                focus_target_for_picker_transition(true),
                                window,
                            );
                            cx.notify();
                        });
                    }),
                    std::rc::Rc::new(move |window, cx| {
                        close_entity.update(cx, |shell, cx| {
                            shell.tabs.close_picker();
                            shell.apply_focus_target(
                                focus_target_for_picker_transition(false),
                                window,
                            );
                            cx.notify();
                        });
                    }),
                ))
            })
            .when_some(context_menu_overlay, |el, overlay| el.child(overlay))
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::theme::{AccentColor, SemanticColorsPreference};
    use gpui::WindowBackgroundAppearance;

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
        assert_eq!(opts.window_decorations, Some(WindowDecorations::Server));
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
                configmaps: true,
                secrets: true,
                persistent_volumes: true,
                persistent_volume_claims: true,
                network_policies: true,
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
        );
        assert!(
            dep_entries
                .iter()
                .any(|e| matches!(e, ContextMenuEntry::Separator))
        );
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
            ContextMenuEntry::Item(item) if item.danger && item.disabled
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
            ContextMenuEntry::Item(item) if item.danger && item.disabled
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
            ContextMenuEntry::Item(item) if item.danger && item.disabled
        )));

        let ev = EventInfo {
            uid: "ev-1".to_string(),
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
            ContextMenuEntry::Item(item) if item.danger && item.disabled
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
            &mut nodes,
            &mut events,
            &mut tabs,
            WorkspaceView::Deployments,
        );

        assert_eq!(
            tabs.open_tabs(),
            &[
                WorkspaceView::Overview,
                WorkspaceView::Deployments,
                WorkspaceView::Nodes,
            ]
        );
        assert_eq!(tabs.active_tab(), Some(WorkspaceView::Deployments));
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
        assert_eq!(tabs.active_tab(), Some(WorkspaceView::Deployments));
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
        assert_eq!(tabs.open_tabs(), &[WorkspaceView::Overview]);
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
        sidebar.sync_for_view(tabs.active_tab());

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
            &mut nodes,
            &mut events,
            &mut tabs,
            "tab",
            true,
            false,
            false,
        );
        assert!(handled);
        assert_eq!(tabs.active_tab(), Some(WorkspaceView::Deployments));
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
            &mut nodes,
            &mut events,
            &mut tabs,
            "tab",
            true,
            true,
            false,
        );
        assert!(handled);
        assert_eq!(tabs.active_tab(), Some(WorkspaceView::Overview));

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
            tabs.open_tabs(),
            &[WorkspaceView::Deployments, WorkspaceView::Nodes]
        );
        assert_eq!(tabs.active_tab(), Some(WorkspaceView::Deployments));

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
        assert_eq!(tabs.active_tab(), None);
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
            &mut nodes,
            &mut events,
            &mut tabs,
            WorkspaceView::Pods,
        );
        assert_eq!(tabs.active_tab(), Some(WorkspaceView::Pods));
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
                "delete-pod",
            ]
        );
        let delete_item = items.iter().find(|i| i.id == "delete-pod").unwrap();
        assert!(delete_item.disabled);
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
            &mut nodes,
            &mut events,
            &mut tabs,
            WorkspaceView::Services,
        );
        assert_eq!(tabs.active_tab(), Some(WorkspaceView::Services));
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
        );
        let items: Vec<&crate::components::ContextMenuItem> = entries
            .iter()
            .filter_map(|e| match e {
                ContextMenuEntry::Item(item) => Some(item),
                ContextMenuEntry::Separator => None,
            })
            .collect();
        let delete_item = items.iter().find(|i| i.id == "delete-service").unwrap();
        assert!(delete_item.disabled);
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
        );
        let items: Vec<&crate::components::ContextMenuItem> = entries
            .iter()
            .filter_map(|e| match e {
                ContextMenuEntry::Item(item) => Some(item),
                ContextMenuEntry::Separator => None,
            })
            .collect();
        let delete_item = items.iter().find(|i| i.id == "delete-configmap").unwrap();
        assert!(delete_item.disabled);
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
        );
        let items: Vec<&crate::components::ContextMenuItem> = entries
            .iter()
            .filter_map(|e| match e {
                ContextMenuEntry::Item(item) => Some(item),
                ContextMenuEntry::Separator => None,
            })
            .collect();
        let delete_item = items.iter().find(|i| i.id == "delete-secret").unwrap();
        assert!(delete_item.disabled);
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
            &mut nodes,
            &mut events,
            &mut tabs,
            WorkspaceView::Pods,
        );
        assert_eq!(sidebar.active_view, WorkspaceView::Pods);
        assert_eq!(tabs.active_tab(), Some(WorkspaceView::Pods));
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
            &mut nodes,
            &mut events,
            &mut tabs,
            WorkspaceView::Deployments,
        );
        assert_eq!(sidebar.active_view, WorkspaceView::Deployments);
        assert_eq!(tabs.active_tab(), Some(WorkspaceView::Deployments));
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
            &mut nodes,
            &mut events,
            &mut tabs,
            WorkspaceView::StatefulSets,
        );
        assert_eq!(sidebar.active_view, WorkspaceView::StatefulSets);
        assert_eq!(tabs.active_tab(), Some(WorkspaceView::StatefulSets));
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
            &mut nodes,
            &mut events,
            &mut tabs,
            WorkspaceView::DaemonSets,
        );
        assert_eq!(sidebar.active_view, WorkspaceView::DaemonSets);
        assert_eq!(tabs.active_tab(), Some(WorkspaceView::DaemonSets));
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
            &mut nodes,
            &mut events,
            &mut tabs,
            WorkspaceView::Jobs,
        );
        assert_eq!(sidebar.active_view, WorkspaceView::Jobs);
        assert_eq!(tabs.active_tab(), Some(WorkspaceView::Jobs));
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
            &mut nodes,
            &mut events,
            &mut tabs,
            WorkspaceView::ReplicaSets,
        );
        assert_eq!(sidebar.active_view, WorkspaceView::ReplicaSets);
        assert_eq!(tabs.active_tab(), Some(WorkspaceView::ReplicaSets));
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
            &mut nodes,
            &mut events,
            &mut tabs,
            WorkspaceView::Ingresses,
        );
        assert_eq!(tabs.active_tab(), Some(WorkspaceView::Ingresses));
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
            &mut nodes,
            &mut events,
            &mut tabs,
            WorkspaceView::PersistentVolumes,
        );
        assert_eq!(tabs.active_tab(), Some(WorkspaceView::PersistentVolumes));
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
            &mut nodes,
            &mut events,
            &mut tabs,
            WorkspaceView::PersistentVolumeClaims,
        );
        assert_eq!(
            tabs.active_tab(),
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
            &mut nodes,
            &mut events,
            &mut tabs,
            WorkspaceView::NetworkPolicies,
        );
        assert_eq!(tabs.active_tab(), Some(WorkspaceView::NetworkPolicies));
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
}
