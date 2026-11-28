// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod add_cluster_state;
mod bridge;
pub mod components;
pub mod icons;
pub mod navigation;
mod shell;
mod state;
pub mod theme;
pub mod views;
mod watch_state;

use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

use gpui::{
    Anchor, AnyElement, App, Bounds, ClipboardItem, Context, CursorStyle, FocusHandle, FontWeight,
    KeyDownEvent, MouseButton, MouseMoveEvent, PathPromptOptions, Pixels, Point, Rgba,
    SharedString, Window, WindowAppearance, WindowBounds, WindowDecorations, WindowOptions,
    anchored, deferred, div, point, prelude::*, px, size,
};
use lucide_icons::Icon;
use orbit_core::config::OrbitConfig;
use orbit_core::kubernetes::logs::{format_logs_as_csv, format_logs_as_json};
use orbit_core::kubernetes::models::ClusterInfo;
use tokio::sync::mpsc;

use crate::bridge::{ClusterBridge, start_cluster_bridge};
use crate::components::{
    ActionToastKind, ActionToastState, ActiveShellDrawer, AddClusterModalCallbacks,
    CommandPaletteAction, CommandPaletteCallbacks, CommandPaletteState, ConfirmDialogCallbacks,
    ConfirmDialogKeyResult, ConfirmDialogState, ContextMenuEntry, ContextMenuKeyAction,
    ContextMenuState, MACOS_TRAFFIC_LIGHT_Y, NotificationCenterState, NotificationDrawerCallbacks,
    NotificationSeverity, PortForwardInputs, PortForwardModalCallbacks, PortForwardModalField,
    PortForwardModalKeyResult, PortForwardModalState, PortForwardResourceOption,
    ProfileDrawerCallbacks, ResourceDrawerState, TITLEBAR_CONTROL_HEIGHT, TabBarCallbacks,
    TabContextMenuCallbacks, TabPickerCallbacks, TextTooltip, TitlebarSlots,
    WorkloadActionDialogState, WorkloadActionModalCallbacks, WorkloadActionModalKeyResult,
    WorkloadActionModalState, build_command_palette_items, build_tab_context_menu_entries,
    extract_ports_from_deployment, extract_ports_from_pod, extract_ports_from_replicaset,
    extract_ports_from_service, extract_ports_from_statefulset, notifications_tooltip,
    parse_hex_color, render_action_toast_overlay, render_add_cluster_modal,
    render_command_palette_overlay, render_confirm_dialog, render_context_menu_overlay,
    render_custom_titlebar, render_notification_drawer, render_port_forward_modal,
    render_profile_drawer, render_tab_resource_picker_overlay, render_workload_action_modal,
    render_workspace_tab_bar, resolve_window_chrome_style,
};
use crate::icons::{render_icon, theme_mode_icon, theme_mode_tooltip};
use crate::navigation::{
    ACTIVITY_BAR_WIDTH, EditTabTarget, LogsTabTarget, SidebarCategory, SidebarLayoutState,
    WorkspaceTab, WorkspaceView,
};
use crate::shell::coordinator::*;
use crate::state::{
    AddClusterKeyResult, AddClusterModalState, ClusterBridgeCommand, ClusterBridgeEvent,
    ClusterConnectionState, ClusterSwitcherState, EventsLoadStatus, NamespaceFilterState,
    NamespaceLoadStatus, PickerKeyAction, PortForwardsLoadStatus, TabDropPosition,
    WorkloadActionKind, WorkspaceResourcesState, WorkspaceTabsState,
};
use crate::theme::{OrbitTheme, ThemeMode, apply_text_step, sync_native_titlebar_appearance};

pub const MAX_EVENTS_PER_FRAME_BATCH: usize = 128;
pub const FRAME_PACING_INTERVAL: std::time::Duration = std::time::Duration::from_millis(16);

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct BatchApplySummary {
    pub should_notify: bool,
    pub pure_watch_batch: bool,
}

/// Drains `first` plus up to `max_batch - 1` immediately ready events from `event_rx`
/// so bursts are applied inside a single GPUI entity update and render pass.
pub fn drain_ready_events(
    event_rx: &mut mpsc::UnboundedReceiver<ClusterBridgeEvent>,
    first: ClusterBridgeEvent,
    max_batch: usize,
) -> Vec<ClusterBridgeEvent> {
    let cap = max_batch.max(1);
    let mut batch = Vec::with_capacity(cap.min(MAX_EVENTS_PER_FRAME_BATCH));
    batch.push(first);
    while batch.len() < cap {
        match event_rx.try_recv() {
            Ok(next) => batch.push(next),
            Err(_) => break,
        }
    }
    batch
}

/// Returns the remaining 60 FPS pacing sleep duration only for pure watch-update batches
/// that triggered a visible UI refresh. User-action events always flush immediately.
pub fn should_pace_after_batch(
    summary: BatchApplySummary,
    elapsed_since_last_flush: std::time::Duration,
) -> Option<std::time::Duration> {
    if summary.pure_watch_batch
        && summary.should_notify
        && elapsed_since_last_flush < FRAME_PACING_INTERVAL
    {
        Some(FRAME_PACING_INTERVAL.saturating_sub(elapsed_since_last_flush))
    } else {
        None
    }
}

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
    crate::theme::kit_bridge::sync_kit_theme(&updated, cx);
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
pub struct ShellChromeLayout {
    #[cfg(test)]
    pub show_activity_brand_badge: bool,
    pub show_activity_settings_btn: bool,
    #[cfg(test)]
    pub show_header_settings_btn: bool,
    #[cfg(test)]
    pub show_footer_settings_btn: bool,
    #[cfg(test)]
    pub show_footer_sidebar_width_debug: bool,
    #[cfg(test)]
    pub show_header_breadcrumb: bool,
    #[cfg(test)]
    pub show_footer_breadcrumb: bool,
    #[cfg(test)]
    pub show_sidebar_panel_collapse_btn: bool,
    #[cfg(test)]
    pub show_header_sidebar_toggle_btn: bool,
}

pub const ORBIT_SHELL_CHROME: ShellChromeLayout = ShellChromeLayout {
    #[cfg(test)]
    show_activity_brand_badge: false,
    show_activity_settings_btn: true,
    #[cfg(test)]
    show_header_settings_btn: false,
    #[cfg(test)]
    show_footer_settings_btn: false,
    #[cfg(test)]
    show_footer_sidebar_width_debug: false,
    #[cfg(test)]
    show_header_breadcrumb: false,
    #[cfg(test)]
    show_footer_breadcrumb: false,
    #[cfg(test)]
    show_sidebar_panel_collapse_btn: false,
    #[cfg(test)]
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

pub fn should_show_namespace_trigger(connection_state: &ClusterConnectionState) -> bool {
    matches!(connection_state, ClusterConnectionState::Connected { .. })
}

pub fn dismiss_cluster_dropdown_on_escape(key: &str, state: &mut ClusterSwitcherState) -> bool {
    if key.eq_ignore_ascii_case("escape") && state.dropdown_open {
        state.close_dropdown();
        return true;
    }
    false
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

pub const ORBIT_GITHUB_REPO_URL: &str = "https://github.com/vantoan1511/orbit";

pub struct OrbitShell {
    state: ClusterSwitcherState,
    sidebar: SidebarLayoutState,
    resources: WorkspaceResourcesState,
    drawer: ResourceDrawerState,
    tabs: WorkspaceTabsState,
    log_tabs: HashMap<LogsTabTarget, views::logs::LogViewerState>,
    next_log_stream_id: u64,
    edit_tabs: HashMap<EditTabTarget, views::edit::EditTabEntry>,
    next_edit_request_id: u64,
    pending_edit_close: Option<(views::edit::PendingCloseTabsAction, ConfirmDialogState)>,
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
    port_forward_inputs: Option<PortForwardInputs>,
    port_forward_subscriptions: Vec<gpui::Subscription>,
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
    pub const FOOTER_HEIGHT: f32 = 26.0;

    pub const fn header_dropdown_top_offset() -> f32 {
        Self::TITLEBAR_HEIGHT + 4.0
    }

    pub const fn tab_picker_top_offset() -> f32 {
        Self::TITLEBAR_HEIGHT + crate::components::TAB_BAR_HEIGHT + 4.0
    }

    pub fn new(config: &OrbitConfig, window: &mut Window, cx: &mut Context<Self>) -> Self {
        let ClusterBridge {
            command_tx,
            mut event_rx,
        } = start_cluster_bridge();

        cx.spawn(async move |this, cx| {
            let mut last_watch_flush = std::time::Instant::now()
                .checked_sub(FRAME_PACING_INTERVAL)
                .unwrap_or_else(std::time::Instant::now);

            while let Some(first_event) = event_rx.recv().await {
                let batch = drain_ready_events(&mut event_rx, first_event, MAX_EVENTS_PER_FRAME_BATCH);
                let mut summary = BatchApplySummary {
                    should_notify: false,
                    pure_watch_batch: true,
                };

                if this
                    .update(cx, |shell, cx| {
                        for event in batch {
                            if let ClusterBridgeEvent::ResourceWatchUpdated {
                                cluster_id,
                                generation,
                                payload,
                            } = event
                            {
                                let (outcome, follow_up) = apply_resource_watch_update(
                                    &mut shell.resources,
                                    &cluster_id,
                                    generation,
                                    payload,
                                    shell.sidebar.active_view,
                                    shell.drawer.target.as_ref(),
                                );
                                if let Some(cmd) = follow_up {
                                    shell.dispatch_command(cmd);
                                }
                                if outcome.affects_active_ui {
                                    summary.should_notify = true;
                                }
                                continue;
                            }

                            summary.pure_watch_batch = false;
                            summary.should_notify = true;

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
                                    &mut shell.drawer,
                                    &mut shell.resources,
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
                                clear_all_edit_tabs_in_state(&mut shell.tabs, &mut shell.edit_tabs);
                                shell.pending_edit_close = None;
                            }
                            apply_log_bridge_event(
                                &mut shell.log_tabs,
                                shell.state.active_cluster_id.as_deref(),
                                &event,
                            );
                            if let Some((toast_info, edit_cmds)) = apply_edit_bridge_event(
                                &mut shell.edit_tabs,
                                &mut shell.resources,
                                &mut shell.drawer,
                                &mut shell.action_toast,
                                shell.state.active_cluster_id.as_deref(),
                                &event,
                            ) {
                                for cmd in edit_cmds {
                                    shell.dispatch_command(cmd);
                                }
                                if let Some((toast_id, toast_kind)) = toast_info {
                                    shell.schedule_toast_dismiss(toast_id, toast_kind, cx);
                                }
                            }
                            if let Some((pf_cmds, toast_info)) = apply_port_forward_event(
                                &mut shell.resources.port_forwards,
                                &mut shell.port_forward_modal,
                                &mut shell.action_toast,
                                &event,
                            ) {
                                if shell.port_forward_modal.is_none() {
                                    shell.close_port_forward_modal();
                                }
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
                                &mut shell.resources,
                                &mut shell.tabs,
                                event,
                            ) {
                                shell.dispatch_command(cmd);
                            }
                            if is_cluster_connected {
                                shell.hydrate_exemplar_tables();
                            }
                        }

                        if summary.should_notify {
                            shell.reconcile_drawer_target_exists();
                            cx.notify();
                        }
                    })
                    .is_err()
                {
                    break;
                }

                if summary.should_notify {
                    if let Some(wait) =
                        should_pace_after_batch(summary, last_watch_flush.elapsed())
                    {
                        cx.background_executor().timer(wait).await;
                    }
                    last_watch_flush = std::time::Instant::now();
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
        focus_handle.focus(window, cx);

        let mut shell = Self {
            state: ClusterSwitcherState::new(),
            sidebar: SidebarLayoutState::new(),
            resources: WorkspaceResourcesState::from_config(config),
            drawer: ResourceDrawerState::default(),
            tabs: WorkspaceTabsState::from_config(config),
            log_tabs: HashMap::new(),
            next_log_stream_id: 1,
            edit_tabs: HashMap::new(),
            next_edit_request_id: 1,
            pending_edit_close: None,
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
            port_forward_inputs: None,
            port_forward_subscriptions: Vec::new(),
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
        self.resources.namespaces.close_dropdown();
        self.settings_dropdown.close();
        self.context_menu.close();
        self.add_cluster.open(&self.state.clusters);
        self.focus_handle.focus(window, cx);
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
                        shell.focus_handle.focus(window, cx);
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
        self.resources.namespaces.close_dropdown();
        self.context_menu.close();
        self.workload_action_modal.open_scale(
            cluster_name,
            kind,
            namespace,
            name,
            current_replicas,
        );
        self.focus_handle.focus(window, cx);
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
        self.resources.namespaces.close_dropdown();
        self.context_menu.close();
        self.workload_action_modal.open_confirm(
            cluster_name,
            WorkloadActionKind::Delete,
            kind,
            namespace.unwrap_or_default(),
            name,
        );
        self.focus_handle.focus(window, cx);
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
        self.resources.namespaces.close_dropdown();
        self.context_menu.close();
        self.workload_action_modal
            .open_confirm(cluster_name, action, kind, namespace, name);
        self.focus_handle.focus(window, cx);
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

    fn subscribe_port_forward_input(
        input: &gpui::Entity<gpui::component::input::InputState>,
        field: PortForwardModalField,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> gpui::Subscription {
        use gpui::component::input::InputEvent;

        cx.subscribe_in(
            input,
            window,
            move |shell, input, event: &InputEvent, _window, cx| match event {
                InputEvent::Change => {
                    if let Some(m) = shell.port_forward_modal.as_mut() {
                        let value = input.read(cx).value().to_string();
                        match field {
                            PortForwardModalField::LocalPort => m.local_port_input = value,
                            PortForwardModalField::RemotePort => m.remote_port_input = value,
                            PortForwardModalField::ResourceName => {}
                        }
                        m.error = None;
                        cx.notify();
                    }
                }
                InputEvent::Focus => {
                    if let Some(m) = shell.port_forward_modal.as_mut() {
                        m.active_field = field;
                        cx.notify();
                    }
                }
                InputEvent::PressEnter { .. } => {
                    shell.submit_port_forward_modal(cx);
                }
                InputEvent::Blur => {}
            },
        )
    }

    fn init_port_forward_inputs(
        &mut self,
        modal: &PortForwardModalState,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let inputs = PortForwardInputs::new(
            &modal.local_port_input,
            &modal.remote_port_input,
            window,
            cx,
        );
        let local_sub = Self::subscribe_port_forward_input(
            &inputs.local,
            PortForwardModalField::LocalPort,
            window,
            cx,
        );
        let remote_sub = Self::subscribe_port_forward_input(
            &inputs.remote,
            PortForwardModalField::RemotePort,
            window,
            cx,
        );
        inputs.focus_field(modal.active_field, window, cx);
        self.port_forward_inputs = Some(inputs);
        self.port_forward_subscriptions = vec![local_sub, remote_sub];
    }

    fn focus_port_forward_field(
        &self,
        field: PortForwardModalField,
        window: &mut Window,
        cx: &mut App,
    ) {
        if field == PortForwardModalField::ResourceName {
            self.focus_handle.focus(window, cx);
        } else if let Some(inputs) = &self.port_forward_inputs {
            inputs.focus_field(field, window, cx);
        }
    }

    fn close_port_forward_modal(&mut self) {
        self.port_forward_modal = None;
        self.port_forward_inputs = None;
        self.port_forward_subscriptions.clear();
    }

    fn sync_port_forward_inputs(&self, window: &mut Window, cx: &mut App) {
        if let (Some(modal), Some(inputs)) = (&self.port_forward_modal, &self.port_forward_inputs) {
            inputs.sync_from_state(modal, window, cx);
        }
    }

    fn open_port_forward_target_modal(
        &mut self,
        namespace: &str,
        kind: &str,
        name: &str,
        detected_ports: Vec<u16>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let ClusterConnectionState::Connected { cluster_id, .. } = &self.state.connection_state
        else {
            return;
        };
        let cluster_id = cluster_id.clone();
        let cluster_name = self.state.cluster_display_name(&cluster_id);
        let active_local_ports: Vec<u16> = self
            .resources
            .port_forwards
            .port_forwards
            .iter()
            .map(|pf| pf.local_port)
            .collect();
        self.state.close_dropdown();
        self.resources.namespaces.close_dropdown();
        self.context_menu.close();
        let modal = PortForwardModalState::new_targeted(
            cluster_id,
            cluster_name,
            namespace,
            kind,
            name,
            detected_ports,
            active_local_ports,
        );
        self.init_port_forward_inputs(&modal, window, cx);
        self.port_forward_modal = Some(modal);
        cx.notify();
    }

    fn open_port_forward_create_modal(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let ClusterConnectionState::Connected { cluster_id, .. } = &self.state.connection_state
        else {
            return;
        };
        let cluster_id = cluster_id.clone();
        let cluster_name = self.state.cluster_display_name(&cluster_id);
        if self
            .resources
            .namespaces
            .needs_load_for_cluster(&cluster_id)
        {
            let cmd = self
                .resources
                .namespaces
                .begin_load_for_cluster(&cluster_id);
            self.dispatch_command(cmd);
        }
        if self.resources.services.needs_load_for_cluster(&cluster_id) {
            let cmd = self.resources.services.begin_load_for_cluster(&cluster_id);
            self.dispatch_command(cmd);
        }
        if self.resources.pods.needs_load_for_cluster(&cluster_id) {
            let cmd = self.resources.pods.begin_load_for_cluster(&cluster_id);
            self.dispatch_command(cmd);
        }
        if self
            .resources
            .deployments
            .needs_load_for_cluster(&cluster_id)
        {
            let cmd = self
                .resources
                .deployments
                .begin_load_for_cluster(&cluster_id);
            self.dispatch_command(cmd);
        }
        if self
            .resources
            .statefulsets
            .needs_load_for_cluster(&cluster_id)
        {
            let cmd = self
                .resources
                .statefulsets
                .begin_load_for_cluster(&cluster_id);
            self.dispatch_command(cmd);
        }
        if self
            .resources
            .replicasets
            .needs_load_for_cluster(&cluster_id)
        {
            let cmd = self
                .resources
                .replicasets
                .begin_load_for_cluster(&cluster_id);
            self.dispatch_command(cmd);
        }

        let available_namespaces: Vec<String> = self
            .resources
            .namespaces
            .namespaces
            .iter()
            .map(|ns| ns.name.clone())
            .collect();
        let mut resources = Vec::new();
        for svc in &self.resources.services.services {
            resources.push(PortForwardResourceOption {
                namespace: svc.namespace.clone(),
                kind: "Service".to_string(),
                name: svc.name.clone(),
                ports: extract_ports_from_service(svc),
            });
        }
        for pod in &self.resources.pods.pods {
            resources.push(PortForwardResourceOption {
                namespace: pod.namespace.clone(),
                kind: "Pod".to_string(),
                name: pod.name.clone(),
                ports: extract_ports_from_pod(pod),
            });
        }
        for dep in &self.resources.deployments.deployments {
            resources.push(PortForwardResourceOption {
                namespace: dep.namespace.clone(),
                kind: "Deployment".to_string(),
                name: dep.name.clone(),
                ports: extract_ports_from_deployment(dep),
            });
        }
        for ss in &self.resources.statefulsets.statefulsets {
            resources.push(PortForwardResourceOption {
                namespace: ss.namespace.clone(),
                kind: "StatefulSet".to_string(),
                name: ss.name.clone(),
                ports: extract_ports_from_statefulset(ss),
            });
        }
        for rs in &self.resources.replicasets.replicasets {
            resources.push(PortForwardResourceOption {
                namespace: rs.namespace.clone(),
                kind: "ReplicaSet".to_string(),
                name: rs.name.clone(),
                ports: extract_ports_from_replicaset(rs),
            });
        }
        let active_local_ports: Vec<u16> = self
            .resources
            .port_forwards
            .port_forwards
            .iter()
            .map(|pf| pf.local_port)
            .collect();

        self.state.close_dropdown();
        self.resources.namespaces.close_dropdown();
        self.context_menu.close();
        let modal = PortForwardModalState::new_create(
            cluster_id,
            cluster_name,
            available_namespaces,
            resources,
            active_local_ports,
        );
        self.init_port_forward_inputs(&modal, window, cx);
        self.port_forward_modal = Some(modal);
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
        self.resources.port_forwards.mark_stopping(&id);
        self.dispatch_command(ClusterBridgeCommand::StopPortForward { cluster_id, id });
        cx.notify();
    }

    fn stop_selected_port_forwards(&mut self, cx: &mut Context<Self>) {
        let Some(cluster_id) = self.state.active_cluster_id.clone() else {
            return;
        };
        let ids: Vec<String> = self
            .resources
            .port_forwards
            .selected_ids
            .iter()
            .cloned()
            .collect();
        if ids.is_empty() {
            return;
        }
        for id in &ids {
            self.resources.port_forwards.mark_stopping(id);
        }
        self.dispatch_command(ClusterBridgeCommand::StopSelectedPortForwards { cluster_id, ids });
        cx.notify();
    }

    fn stop_all_port_forwards(&mut self, cx: &mut Context<Self>) {
        let Some(cluster_id) = self.state.active_cluster_id.clone() else {
            return;
        };
        let ids: Vec<String> = self
            .resources
            .port_forwards
            .port_forwards
            .iter()
            .map(|pf| pf.id.clone())
            .collect();
        for id in ids {
            self.resources.port_forwards.mark_stopping(&id);
        }
        self.dispatch_command(ClusterBridgeCommand::StopAllPortForwards { cluster_id });
        cx.notify();
    }

    fn render_port_forward_modal_overlay(&self, cx: &mut Context<Self>) -> Option<AnyElement> {
        let modal = self.port_forward_modal.as_ref()?;
        let inputs = self.port_forward_inputs.as_ref()?;
        let theme = OrbitTheme::get(cx).clone();
        let close_entity = cx.entity().clone();
        let submit_entity = cx.entity().clone();
        let focus_entity = cx.entity().clone();
        let port_entity = cx.entity().clone();
        let kind_entity = cx.entity().clone();
        let ns_entity = cx.entity().clone();
        let res_entity = cx.entity().clone();

        let callbacks = PortForwardModalCallbacks {
            on_close: std::rc::Rc::new(move |window, cx| {
                close_entity.update(cx, |shell, cx| {
                    if !shell
                        .port_forward_modal
                        .as_ref()
                        .is_some_and(|m| m.submitting)
                    {
                        shell.close_port_forward_modal();
                        shell.focus_handle.focus(window, cx);
                        cx.notify();
                    }
                });
            }),
            on_submit: std::rc::Rc::new(move |_, cx| {
                submit_entity.update(cx, |shell, cx| {
                    shell.submit_port_forward_modal(cx);
                });
            }),
            on_focus_field: std::rc::Rc::new(move |field, window, cx| {
                focus_entity.update(cx, |shell, cx| {
                    if let Some(m) = shell.port_forward_modal.as_mut() {
                        m.active_field = field;
                        shell.focus_port_forward_field(field, window, cx);
                        cx.notify();
                    }
                });
            }),
            on_select_detected_port: std::rc::Rc::new(move |port, window, cx| {
                port_entity.update(cx, |shell, cx| {
                    if let Some(m) = shell.port_forward_modal.as_mut() {
                        m.select_detected_port(port);
                        shell.sync_port_forward_inputs(window, cx);
                        cx.notify();
                    }
                });
            }),
            on_select_kind: std::rc::Rc::new(move |kind, window, cx| {
                kind_entity.update(cx, |shell, cx| {
                    if let Some(m) = shell.port_forward_modal.as_mut() {
                        m.set_kind(&kind);
                        shell.sync_port_forward_inputs(window, cx);
                        cx.notify();
                    }
                });
            }),
            on_select_namespace: std::rc::Rc::new(move |ns, window, cx| {
                ns_entity.update(cx, |shell, cx| {
                    if let Some(m) = shell.port_forward_modal.as_mut() {
                        m.set_namespace(&ns);
                        shell.sync_port_forward_inputs(window, cx);
                        cx.notify();
                    }
                });
            }),
            on_select_resource: std::rc::Rc::new(move |name, window, cx| {
                res_entity.update(cx, |shell, cx| {
                    if let Some(m) = shell.port_forward_modal.as_mut() {
                        m.select_resource(&name);
                        shell.sync_port_forward_inputs(window, cx);
                        cx.notify();
                    }
                });
            }),
        };

        Some(render_port_forward_modal(modal, inputs, &theme, &callbacks))
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
        self.resources.namespaces.close_dropdown();
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
        self.resources.namespaces.select_all_namespaces();
        if let Some(updated) = self.resources.namespaces.toggle_namespace(ns)
            && let Some(cluster_id) = self.resources.namespaces.cluster_id.clone()
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
        self.resources.pods.hydrate_table_preferences(
            cluster_id,
            &self.table_preferences,
            &views::pods::pod_columns(),
        );
        self.resources.deployments.hydrate_table_preferences(
            cluster_id,
            &self.table_preferences,
            &views::deployments::deployment_columns(),
        );
        self.resources.services.hydrate_table_preferences(
            cluster_id,
            &self.table_preferences,
            &views::services::service_columns(),
        );
        self.resources.events.hydrate_table_preferences(
            cluster_id,
            &self.table_preferences,
            &views::events::event_columns(),
        );
    }

    fn reset_exemplar_pages(&mut self) {
        self.resources.pods.table.reset_page();
        self.resources.deployments.table.reset_page();
        self.resources.services.table.reset_page();
        self.resources.events.table.reset_page();
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
            WorkspaceView::Pods => Some((
                "pods",
                &mut self.resources.pods.table,
                views::pods::pod_columns(),
            )),
            WorkspaceView::Deployments => Some((
                "deployments",
                &mut self.resources.deployments.table,
                views::deployments::deployment_columns(),
            )),
            WorkspaceView::Services => Some((
                "services",
                &mut self.resources.services.table,
                views::services::service_columns(),
            )),
            WorkspaceView::Events => Some((
                "events",
                &mut self.resources.events.table,
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
                            shell.focus_handle.focus(window, cx);
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
            &mut self.resources.pods.table,
            &mut self.resources.deployments.table,
            &mut self.resources.services.table,
            &mut self.resources.events.table,
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
            ("pods", &mut self.resources.pods.table),
            ("deployments", &mut self.resources.deployments.table),
            ("services", &mut self.resources.services.table),
            ("events", &mut self.resources.events.table),
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
            &mut self.resources,
            &mut self.tabs,
            view,
        ) {
            self.dispatch_command(cmd);
        }
        if view == WorkspaceView::PortForward
            && let ClusterConnectionState::Connected { cluster_id, .. } =
                &self.state.connection_state
            && self
                .resources
                .port_forwards
                .needs_load_for_cluster(cluster_id)
        {
            let cluster_id = cluster_id.clone();
            let cmd = self
                .resources
                .port_forwards
                .begin_load_for_cluster(&cluster_id);
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
        if let Some(cmd) = load_command_for_active_view(&self.state, &mut self.resources, active) {
            self.dispatch_command(cmd);
        }
        if let Some(sync_cmd) = sync_active_watchers_command(
            &self.state,
            &mut self.resources,
            &self.tabs,
            self.sidebar.active_view,
        ) {
            self.dispatch_command(sync_cmd);
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
        self.request_close_tabs_action(views::edit::PendingCloseTabsAction::CloseSingle(tab.into()), cx);
    }

    fn close_other_tabs(
        &mut self,
        tab: impl Into<navigation::WorkspaceTab>,
        cx: &mut Context<Self>,
    ) {
        self.request_close_tabs_action(views::edit::PendingCloseTabsAction::CloseOthers(tab.into()), cx);
    }

    fn close_tabs_to_right(
        &mut self,
        tab: impl Into<navigation::WorkspaceTab>,
        cx: &mut Context<Self>,
    ) {
        self.request_close_tabs_action(views::edit::PendingCloseTabsAction::CloseRight(tab.into()), cx);
    }

    fn close_all_tabs(&mut self, cx: &mut Context<Self>) {
        self.request_close_tabs_action(views::edit::PendingCloseTabsAction::CloseAll, cx);
    }

    fn request_close_tabs_action(
        &mut self,
        action: views::edit::PendingCloseTabsAction,
        cx: &mut Context<Self>,
    ) {
        let dirty = dirty_edit_tabs_for_close_action(&self.tabs, &self.edit_tabs, &action);
        if !dirty.is_empty() {
            let cluster_name = self
                .active_cluster_id()
                .map(|id| self.state.cluster_display_name(id))
                .unwrap_or_else(|| "Cluster".to_string());
            let dialog = build_unsaved_changes_confirm_dialog(&cluster_name, &dirty);
            self.pending_edit_close = Some((action, dialog));
            cx.notify();
            return;
        }
        self.execute_close_tabs_action(action, cx);
    }

    fn confirm_pending_edit_close(&mut self, cx: &mut Context<Self>) {
        if let Some((action, _)) = self.pending_edit_close.take() {
            self.execute_close_tabs_action(action, cx);
        }
    }

    fn execute_close_tabs_action(
        &mut self,
        action: views::edit::PendingCloseTabsAction,
        cx: &mut Context<Self>,
    ) {
        let prev_view = self.sidebar.active_view;
        let closed = match action {
            views::edit::PendingCloseTabsAction::CloseSingle(tab) => self.tabs.close_tab(tab),
            views::edit::PendingCloseTabsAction::CloseOthers(tab) => {
                self.tabs.close_other_tabs(tab)
            }
            views::edit::PendingCloseTabsAction::CloseRight(tab) => {
                self.tabs.close_tabs_to_right(tab)
            }
            views::edit::PendingCloseTabsAction::CloseAll => self.tabs.close_all_tabs(),
        };
        self.cleanup_closed_log_tabs(closed.logs);
        cleanup_closed_edit_tabs_in_state(&mut self.edit_tabs, closed.edits);
        self.sync_active_tab_view(prev_view);
        cx.notify();
    }

    pub(crate) fn open_edit_tab(
        &mut self,
        kind: &str,
        namespace: Option<&str>,
        name: &str,
        cx: &mut Context<Self>,
    ) {
        let cluster_id = self.active_cluster_id().map(str::to_string);
        if let Some(cmd) = open_edit_tab_in_state(
            &mut self.tabs,
            &mut self.edit_tabs,
            &mut self.next_edit_request_id,
            cluster_id.as_deref(),
            kind,
            namespace,
            name,
        ) {
            self.dispatch_command(cmd);
        }
        cx.notify();
    }

    fn reload_edit_tab(&mut self, target: &EditTabTarget, cx: &mut Context<Self>) {
        let Some(cluster_id) = self.active_cluster_id().map(str::to_string) else {
            return;
        };
        let Some(entry) = self.edit_tabs.get_mut(target) else {
            return;
        };
        let request_id = self.next_edit_request_id;
        self.next_edit_request_id = self.next_edit_request_id.saturating_add(1);
        entry.session.begin_reload(request_id);
        self.dispatch_command(ClusterBridgeCommand::LoadResourceForEdit {
            cluster_id,
            request_id,
            kind: target.kind.clone(),
            namespace: target.namespace.clone(),
            name: target.name.clone(),
        });
        cx.notify();
    }

    fn submit_edit_tab_save(&mut self, target: &EditTabTarget, cx: &mut Context<Self>) {
        let Some(cluster_id) = self.active_cluster_id().map(str::to_string) else {
            return;
        };
        let request_id = self.next_edit_request_id;
        let payload = match self.edit_tabs.get_mut(target) {
            Some(entry) => {
                if !entry.session.is_dirty() {
                    return;
                }
                match entry.session.begin_save(request_id) {
                    Ok(raw_json) => raw_json,
                    Err(_) => {
                        cx.notify();
                        return;
                    }
                }
            }
            None => return,
        };
        self.next_edit_request_id = self.next_edit_request_id.saturating_add(1);
        self.dispatch_command(ClusterBridgeCommand::ApplyResource {
            cluster_id,
            request_id,
            kind: target.kind.clone(),
            namespace: target.namespace.clone(),
            name: target.name.clone(),
            raw_json: payload,
        });
        cx.notify();
    }

    fn ensure_edit_tab_entities(
        &mut self,
        target: &EditTabTarget,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        use gpui::component::input::{InputEvent, InputState};

        let Some(entry) = self.edit_tabs.get_mut(target) else {
            return;
        };
        let yaml_snapshot = entry.session.yaml_content.clone();
        let form_revision = entry.session.form_structure_revision;
        let form_specs = entry
            .session
            .form_model
            .as_ref()
            .map(views::edit::collect_form_input_specs)
            .unwrap_or_default();

        if entry.entities.is_none() {
            let mut entities = views::edit::EditTabEntities::new(&yaml_snapshot, window, cx);
            let target_clone = target.clone();
            let suppress = std::rc::Rc::clone(&entities.suppress_events);
            let sub = cx.subscribe_in(
                &entities.yaml_editor,
                window,
                move |shell, editor, event: &InputEvent, _window, cx| {
                    if matches!(event, InputEvent::Change) && !suppress.get() {
                        let next_yaml = editor.read(cx).value().to_string();
                        if let Some(entry) = shell.edit_tabs.get_mut(&target_clone) {
                            if let Some(ent) = entry.entities.as_mut() {
                                ent.synced_editor_yaml = next_yaml.clone();
                            }
                            entry.session.update_yaml_from_editor(next_yaml);
                            cx.notify();
                        }
                    }
                },
            );
            entities.subscriptions.push(sub);
            entry.entities = Some(entities);
        }

        let Some(entities) = entry.entities.as_mut() else {
            return;
        };
        entities.sync_yaml_editor(&yaml_snapshot, window, cx);

        if entities.synced_form_revision != form_revision {
            entities.form_inputs.clear();
            // Keep index 0 subscription (yaml_editor), drop stale form input subscriptions
            entities.subscriptions.truncate(1);
            for (field_id, initial_val, placeholder) in form_specs {
                let input_state = cx.new(|cx| {
                    InputState::new(window, cx)
                        .placeholder(placeholder)
                        .default_value(initial_val)
                });
                let target_clone = target.clone();
                let key_clone = field_id.clone();
                let suppress = std::rc::Rc::clone(&entities.suppress_events);
                let sub = cx.subscribe_in(
                    &input_state,
                    window,
                    move |shell, input, event: &InputEvent, _window, cx| {
                        if matches!(event, InputEvent::Change) && !suppress.get() {
                            let val = input.read(cx).value().to_string();
                            if let Some(entry) = shell.edit_tabs.get_mut(&target_clone) {
                                entry.session.update_form_model(
                                    |model| {
                                        views::edit::apply_input_value_to_form(
                                            model, &key_clone, val,
                                        );
                                    },
                                    false,
                                );
                                cx.notify();
                            }
                        }
                    },
                );
                entities.subscriptions.push(sub);
                entities.form_inputs.insert(field_id, input_state);
            }
            entities.synced_form_revision = form_revision;
        }
    }

    pub(crate) fn render_edit_tab_content(
        &mut self,
        target: &EditTabTarget,
        theme: &OrbitTheme,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        self.ensure_edit_tab_entities(target, window, cx);
        let Some(entry) = self.edit_tabs.get(target) else {
            return div().flex_1().into_any_element();
        };

        let entity = cx.entity().clone();
        let target_back = target.clone();
        let target_cancel = target.clone();
        let target_save = target.clone();
        let target_retry = target.clone();
        let target_mode = target.clone();
        let target_subtab = target.clone();
        let target_mutate = target.clone();

        let callbacks = views::edit::EditViewCallbacks {
            on_back: {
                let e = entity.clone();
                std::rc::Rc::new(move |_, cx| {
                    let t = target_back.clone();
                    e.update(cx, |shell, cx| {
                        shell.close_tab(WorkspaceTab::Edit(t), cx);
                    });
                })
            },
            on_cancel: {
                let e = entity.clone();
                std::rc::Rc::new(move |_, cx| {
                    let t = target_cancel.clone();
                    e.update(cx, |shell, cx| {
                        shell.close_tab(WorkspaceTab::Edit(t), cx);
                    });
                })
            },
            on_save: {
                let e = entity.clone();
                std::rc::Rc::new(move |_, cx| {
                    let t = target_save.clone();
                    e.update(cx, |shell, cx| {
                        shell.submit_edit_tab_save(&t, cx);
                    });
                })
            },
            on_retry_load: {
                let e = entity.clone();
                std::rc::Rc::new(move |_, cx| {
                    let t = target_retry.clone();
                    e.update(cx, |shell, cx| {
                        shell.reload_edit_tab(&t, cx);
                    });
                })
            },
            on_toggle_yaml_mode: {
                let e = entity.clone();
                std::rc::Rc::new(move |is_yaml, _, cx| {
                    let t = target_mode.clone();
                    let next_mode = if is_yaml {
                        orbit_core::kubernetes::edit_forms::EditMode::Yaml
                    } else {
                        orbit_core::kubernetes::edit_forms::EditMode::Form
                    };
                    e.update(cx, |shell, cx| {
                        if let Some(entry) = shell.edit_tabs.get_mut(&t) {
                            let _ = entry.session.switch_mode(next_mode);
                            cx.notify();
                        }
                    });
                })
            },
            on_select_sub_tab: {
                let e = entity.clone();
                std::rc::Rc::new(move |sub_tab, _, cx| {
                    let t = target_subtab.clone();
                    e.update(cx, |shell, cx| {
                        if let Some(entry) = shell.edit_tabs.get_mut(&t) {
                            entry.session.active_sub_tab = sub_tab;
                            cx.notify();
                        }
                    });
                })
            },
            on_mutate_form: {
                let e = entity.clone();
                std::rc::Rc::new(move |mutator, structural, _, cx| {
                    let t = target_mutate.clone();
                    e.update(cx, |shell, cx| {
                        if let Some(entry) = shell.edit_tabs.get_mut(&t) {
                            entry.session.update_form_model(mutator, structural);
                            cx.notify();
                        }
                    });
                })
            },
        };

        views::edit::render_edit_view(&entry.session, entry.entities.as_ref(), theme, &callbacks)
    }

    fn render_pending_edit_close_overlay(&self, cx: &mut Context<Self>) -> Option<AnyElement> {
        let (_, dialog) = self.pending_edit_close.as_ref()?;
        let theme = OrbitTheme::get(cx).clone();
        let entity = cx.entity().clone();
        let on_cancel: crate::components::ConfirmDialogCallback = {
            let e = entity.clone();
            std::rc::Rc::new(move |_, cx| {
                e.update(cx, |shell, cx| {
                    shell.pending_edit_close = None;
                    cx.notify();
                });
            })
        };
        let on_confirm: crate::components::ConfirmDialogCallback = {
            let e = entity.clone();
            std::rc::Rc::new(move |_, cx| {
                e.update(cx, |shell, cx| {
                    shell.confirm_pending_edit_close(cx);
                });
            })
        };
        let callbacks = ConfirmDialogCallbacks {
            on_close: on_cancel.clone(),
            on_submit: on_confirm,
        };

        Some(
            div()
                .id("edit-unsaved-confirm-backdrop")
                .absolute()
                .inset_0()
                .bg(theme.surface.overlay_bg)
                .flex()
                .items_center()
                .justify_center()
                .on_mouse_down(MouseButton::Left, move |_, window, cx| {
                    window.prevent_default();
                    cx.stop_propagation();
                    on_cancel(window, cx);
                })
                .child(render_confirm_dialog(dialog, &theme, &callbacks))
                .into_any_element(),
        )
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
                shell.log_search_focus.focus(window, cx);
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

    fn is_active_view_loading(&self) -> bool {
        is_workspace_view_loading(
            self.sidebar.active_view,
            WorkspaceLoadingFlags {
                overview: self.resources.overview.status
                    == crate::state::OverviewLoadStatus::Loading,
                workloads_overview: self.resources.workloads_overview.status
                    == crate::state::WorkloadsOverviewLoadStatus::Loading,
                namespaces: self.resources.namespaces.status == NamespaceLoadStatus::Loading,
                deployments: self.resources.deployments.status
                    == crate::state::DeploymentsLoadStatus::Loading,
                statefulsets: self.resources.statefulsets.status
                    == crate::state::StatefulSetsLoadStatus::Loading,
                daemonsets: self.resources.daemonsets.status
                    == crate::state::DaemonSetsLoadStatus::Loading,
                replicasets: self.resources.replicasets.status
                    == crate::state::ReplicaSetsLoadStatus::Loading,
                jobs: self.resources.jobs.status == crate::state::JobsLoadStatus::Loading,
                cronjobs: self.resources.cronjobs.status
                    == crate::state::CronJobsLoadStatus::Loading,
                pods: self.resources.pods.status == crate::state::PodsLoadStatus::Loading,
                services: self.resources.services.status
                    == crate::state::ServicesLoadStatus::Loading,
                ingresses: self.resources.ingresses.status
                    == crate::state::IngressesLoadStatus::Loading,
                port_forwards: self.resources.port_forwards.status
                    == PortForwardsLoadStatus::Loading,
                configmaps: self.resources.configmaps.status
                    == crate::state::ConfigMapsLoadStatus::Loading,
                secrets: self.resources.secrets.status == crate::state::SecretsLoadStatus::Loading,
                persistent_volumes: self.resources.persistent_volumes.status
                    == crate::state::PersistentVolumesLoadStatus::Loading,
                persistent_volume_claims: self.resources.persistent_volume_claims.status
                    == crate::state::PersistentVolumeClaimsLoadStatus::Loading,
                storage_overview: self.resources.storage_overview.status
                    == crate::state::StorageOverviewLoadStatus::Loading,
                network_policies: self.resources.network_policies.status
                    == crate::state::NetworkPoliciesLoadStatus::Loading,
                pod_security: self.resources.pod_security.status
                    == crate::state::PodSecurityLoadStatus::Loading,
                resource_quotas: self.resources.resource_quotas.status
                    == crate::state::ResourceQuotasLoadStatus::Loading,
                hpas: self.resources.list_views.hpas.status
                    == crate::state::HpasLoadStatus::Loading,
                storage_classes: self.resources.list_views.storage_classes.status
                    == crate::state::StorageClassesLoadStatus::Loading,
                security_overview: self.resources.list_views.security_overview.status
                    == crate::state::SecurityOverviewLoadStatus::Loading,
                admission_webhooks: self.resources.list_views.admission_webhooks.status
                    == crate::state::AdmissionWebhooksLoadStatus::Loading,
                nodes: self.resources.nodes.status == crate::state::NodesLoadStatus::Loading,
                events: self.resources.events.status == EventsLoadStatus::Loading,
            },
        )
    }

    fn refresh_workspace(&mut self, cx: &mut Context<Self>) {
        self.dispatch_command(ClusterBridgeCommand::RefreshClusters);
        if let ClusterConnectionState::Connected { cluster_id, .. } = &self.state.connection_state {
            if let Some(cmd) = self.resources.overview.begin_refresh() {
                self.dispatch_command(cmd);
            }
            if self.sidebar.active_view == WorkspaceView::WorkloadsOverview
                || self.resources.workloads_overview.cluster_id.is_some()
            {
                let cmd = self
                    .resources
                    .workloads_overview
                    .begin_refresh()
                    .unwrap_or_else(|| {
                        self.resources
                            .workloads_overview
                            .begin_load_for_cluster(cluster_id)
                    });
                self.dispatch_command(cmd);
            }
            let ns_cmd = self
                .resources
                .namespaces
                .begin_refresh()
                .unwrap_or_else(|| self.resources.namespaces.begin_load_for_cluster(cluster_id));
            self.dispatch_command(ns_cmd);
            if self.sidebar.active_view == WorkspaceView::Deployments
                || self.resources.deployments.cluster_id.is_some()
            {
                let cmd = self
                    .resources
                    .deployments
                    .begin_refresh()
                    .unwrap_or_else(|| {
                        self.resources
                            .deployments
                            .begin_load_for_cluster(cluster_id)
                    });
                self.dispatch_command(cmd);
            }
            if self.sidebar.active_view == WorkspaceView::StatefulSets
                || self.resources.statefulsets.cluster_id.is_some()
            {
                let cmd = self
                    .resources
                    .statefulsets
                    .begin_refresh()
                    .unwrap_or_else(|| {
                        self.resources
                            .statefulsets
                            .begin_load_for_cluster(cluster_id)
                    });
                self.dispatch_command(cmd);
            }
            if self.sidebar.active_view == WorkspaceView::DaemonSets
                || self.resources.daemonsets.cluster_id.is_some()
            {
                let cmd = self
                    .resources
                    .daemonsets
                    .begin_refresh()
                    .unwrap_or_else(|| {
                        self.resources.daemonsets.begin_load_for_cluster(cluster_id)
                    });
                self.dispatch_command(cmd);
            }
            if self.sidebar.active_view == WorkspaceView::ReplicaSets
                || self.resources.replicasets.cluster_id.is_some()
            {
                let cmd = self
                    .resources
                    .replicasets
                    .begin_refresh()
                    .unwrap_or_else(|| {
                        self.resources
                            .replicasets
                            .begin_load_for_cluster(cluster_id)
                    });
                self.dispatch_command(cmd);
            }
            if self.sidebar.active_view == WorkspaceView::Jobs
                || self.resources.jobs.cluster_id.is_some()
            {
                let cmd = self
                    .resources
                    .jobs
                    .begin_refresh()
                    .unwrap_or_else(|| self.resources.jobs.begin_load_for_cluster(cluster_id));
                self.dispatch_command(cmd);
            }
            if self.sidebar.active_view == WorkspaceView::CronJobs
                || self.resources.cronjobs.cluster_id.is_some()
            {
                let cmd =
                    self.resources.cronjobs.begin_refresh().unwrap_or_else(|| {
                        self.resources.cronjobs.begin_load_for_cluster(cluster_id)
                    });
                self.dispatch_command(cmd);
            }
            if self.sidebar.active_view == WorkspaceView::Pods
                || self.resources.pods.cluster_id.is_some()
            {
                let cmd = self
                    .resources
                    .pods
                    .begin_refresh()
                    .unwrap_or_else(|| self.resources.pods.begin_load_for_cluster(cluster_id));
                self.dispatch_command(cmd);
            }
            if self.sidebar.active_view == WorkspaceView::Services
                || self.resources.services.cluster_id.is_some()
            {
                let cmd =
                    self.resources.services.begin_refresh().unwrap_or_else(|| {
                        self.resources.services.begin_load_for_cluster(cluster_id)
                    });
                self.dispatch_command(cmd);
            }
            if self.sidebar.active_view == WorkspaceView::Ingresses
                || self.resources.ingresses.cluster_id.is_some()
            {
                let cmd =
                    self.resources.ingresses.begin_refresh().unwrap_or_else(|| {
                        self.resources.ingresses.begin_load_for_cluster(cluster_id)
                    });
                self.dispatch_command(cmd);
            }
            if self.sidebar.active_view == WorkspaceView::PortForward
                || self.resources.port_forwards.cluster_id.is_some()
            {
                let cmd = self
                    .resources
                    .port_forwards
                    .begin_refresh()
                    .unwrap_or_else(|| {
                        self.resources
                            .port_forwards
                            .begin_load_for_cluster(cluster_id)
                    });
                self.dispatch_command(cmd);
            }
            if self.sidebar.active_view == WorkspaceView::ConfigMaps
                || self.resources.configmaps.cluster_id.is_some()
            {
                let cmd = self
                    .resources
                    .configmaps
                    .begin_refresh()
                    .unwrap_or_else(|| {
                        self.resources.configmaps.begin_load_for_cluster(cluster_id)
                    });
                self.dispatch_command(cmd);
            }
            if self.sidebar.active_view == WorkspaceView::Secrets
                || self.resources.secrets.cluster_id.is_some()
            {
                let cmd =
                    self.resources.secrets.begin_refresh().unwrap_or_else(|| {
                        self.resources.secrets.begin_load_for_cluster(cluster_id)
                    });
                self.dispatch_command(cmd);
            }
            if self.sidebar.active_view == WorkspaceView::PersistentVolumes
                || self.resources.persistent_volumes.cluster_id.is_some()
            {
                let cmd = self
                    .resources
                    .persistent_volumes
                    .begin_refresh()
                    .unwrap_or_else(|| {
                        self.resources
                            .persistent_volumes
                            .begin_load_for_cluster(cluster_id)
                    });
                self.dispatch_command(cmd);
            }
            if self.sidebar.active_view == WorkspaceView::PersistentVolumeClaims
                || self.resources.persistent_volume_claims.cluster_id.is_some()
            {
                let cmd = self
                    .resources
                    .persistent_volume_claims
                    .begin_refresh()
                    .unwrap_or_else(|| {
                        self.resources
                            .persistent_volume_claims
                            .begin_load_for_cluster(cluster_id)
                    });
                self.dispatch_command(cmd);
            }
            if self.sidebar.active_view == WorkspaceView::NetworkPolicies
                || self.resources.network_policies.cluster_id.is_some()
            {
                let cmd = self
                    .resources
                    .network_policies
                    .begin_refresh()
                    .unwrap_or_else(|| {
                        self.resources
                            .network_policies
                            .begin_load_for_cluster(cluster_id)
                    });
                self.dispatch_command(cmd);
            }
            if self.sidebar.active_view == WorkspaceView::StorageOverview
                || self.resources.storage_overview.cluster_id.is_some()
            {
                let cmd = self
                    .resources
                    .storage_overview
                    .begin_refresh()
                    .unwrap_or_else(|| {
                        self.resources
                            .storage_overview
                            .begin_load_for_cluster(cluster_id)
                    });
                self.dispatch_command(cmd);
            }
            if self.sidebar.active_view == WorkspaceView::PodSecurity
                || self.resources.pod_security.cluster_id.is_some()
            {
                let cmd = self
                    .resources
                    .pod_security
                    .begin_refresh()
                    .unwrap_or_else(|| {
                        self.resources
                            .pod_security
                            .begin_load_for_cluster(cluster_id)
                    });
                self.dispatch_command(cmd);
            }
            if self.sidebar.active_view == WorkspaceView::ResourceQuotas
                || self.resources.resource_quotas.cluster_id.is_some()
            {
                let cmd = self
                    .resources
                    .resource_quotas
                    .begin_refresh()
                    .unwrap_or_else(|| {
                        self.resources
                            .resource_quotas
                            .begin_load_for_cluster(cluster_id)
                    });
                self.dispatch_command(cmd);
            }
            for cmd in self
                .resources
                .list_views
                .refresh_commands(self.sidebar.active_view, cluster_id)
            {
                self.dispatch_command(cmd);
            }
            if self.sidebar.active_view == WorkspaceView::Nodes
                || self.resources.nodes.cluster_id.is_some()
            {
                let cmd = self
                    .resources
                    .nodes
                    .begin_refresh()
                    .unwrap_or_else(|| self.resources.nodes.begin_load_for_cluster(cluster_id));
                self.dispatch_command(cmd);
            }
            if self.sidebar.active_view == WorkspaceView::Events
                || self.resources.events.cluster_id.is_some()
            {
                let cmd =
                    self.resources.events.begin_refresh().unwrap_or_else(|| {
                        self.resources.events.begin_load_for_cluster(cluster_id)
                    });
                self.dispatch_command(cmd);
            }
        }
        cx.notify();
    }

    fn select_cluster(&mut self, cluster_id: &str, cx: &mut Context<Self>) {
        if let Some(cmd) = self.state.begin_switch(cluster_id) {
            self.dispatch_command(cmd);
        }
        self.resources.namespaces.close_dropdown();
        cx.notify();
    }

    fn reconnect_cluster(&mut self, cx: &mut Context<Self>) {
        if let Some(cmd) = self.state.begin_reconnect() {
            self.dispatch_command(cmd);
        }
        self.resources.namespaces.close_dropdown();
        cx.notify();
    }

    fn disconnect_cluster(&mut self, cx: &mut Context<Self>) {
        if let Some(cmd) = disconnect_cluster_with_state(
            &mut self.state,
            &mut self.sidebar,
            &mut self.resources,
            &mut self.tabs,
            &mut self.drawer,
        ) {
            self.dispatch_command(cmd);
        }
        self.resources.namespaces.close_dropdown();
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

    fn toggle_experience_features(&mut self, enabled: bool, cx: &mut Context<Self>) {
        self.enable_experience_features = enabled;
        self.settings_error = None;
        self.dispatch_command(ClusterBridgeCommand::SaveEnableExperienceFeatures { enabled });
        cx.notify();
    }

    fn reset_appearance_defaults(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.select_theme_mode(ThemeMode::Dark, window, cx);
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
                traffic_light_position: Some(gpui::point(px(12.0), px(MACOS_TRAFFIC_LIGHT_Y))),
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
                        .h(px(crate::components::TAB_BAR_HEIGHT))
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

    fn render_anchored_dropdown(menu: AnyElement) -> AnyElement {
        deferred(
            anchored()
                .anchor(Anchor::TopLeft)
                .offset(point(px(0.0), px(4.0)))
                .snap_to_window_with_margin(px(8.0))
                .child(menu),
        )
        .with_priority(2)
        .into_any_element()
    }

    fn render_dropdown_backdrop(
        &self,
        id: &'static str,
        on_close: impl Fn(&mut Self) + Copy + 'static,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        div()
            .id(id)
            .absolute()
            .inset_0()
            .on_mouse_down(
                MouseButton::Left,
                cx.listener(move |this, _event, _window, cx| {
                    cx.stop_propagation();
                    on_close(this);
                    cx.notify();
                }),
            )
            .on_mouse_down(
                MouseButton::Right,
                cx.listener(move |this, _event, _window, cx| {
                    cx.stop_propagation();
                    on_close(this);
                    cx.notify();
                }),
            )
            .into_any_element()
    }

    fn render_namespace_trigger_control(
        &self,
        theme: &OrbitTheme,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let control_hover = theme.surface.control_hover;
        let ns_pres = namespace_trigger_presentation(&self.resources.namespaces, theme);
        let ns_chevron = dropdown_chevron_icon(self.resources.namespaces.dropdown_open);
        let ns_trigger = theme
            .style_control(div().id("namespace-selector-btn"))
            .flex()
            .items_center()
            .gap_1p5()
            .h(px(TITLEBAR_CONTROL_HEIGHT))
            .px(theme.typography.control_padding_x_sm)
            .hover(move |style| style.bg(control_hover))
            .when(self.resources.namespaces.dropdown_open, |el| {
                el.border_color(theme.accent.accent)
            })
            .cursor_pointer()
            .on_mouse_down(
                MouseButton::Left,
                cx.listener(|this, _event, window, cx| {
                    cx.stop_propagation();
                    this.dismiss_context_menu();
                    this.state.close_dropdown();
                    this.settings_dropdown.close();
                    this.resources.namespaces.toggle_dropdown();
                    if this.resources.namespaces.dropdown_open {
                        this.namespace_search_focus.focus(window, cx);
                    }
                    cx.notify();
                }),
            )
            .child(render_icon(Icon::Layers, px(13.0), ns_pres.indicator_color))
            .child(
                apply_text_step(div(), theme.typography.size_xs)
                    .font_weight(FontWeight::MEDIUM)
                    .max_w(px(180.0))
                    .truncate()
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
            .child(render_icon(ns_chevron, px(13.0), theme.text.secondary));

        div()
            .flex()
            .flex_col()
            .child(ns_trigger)
            .when(self.resources.namespaces.dropdown_open, |el| {
                el.child(Self::render_anchored_dropdown(
                    self.render_namespace_dropdown_menu(theme, cx),
                ))
            })
            .into_any_element()
    }

    fn render_titlebar_left_controls(&self, cx: &mut Context<Self>) -> AnyElement {
        let theme = OrbitTheme::get(cx).clone();
        let presentation = format_cluster_status(&self.state, &theme);
        let cluster_chevron = dropdown_chevron_icon(self.state.dropdown_open);
        let control_hover = theme.surface.control_hover;

        let cluster_trigger = theme
            .style_control(div().id("cluster-switcher-btn"))
            .flex()
            .items_center()
            .gap_2()
            .h(px(TITLEBAR_CONTROL_HEIGHT))
            .px(theme.typography.control_padding_x_sm)
            .hover(move |style| style.bg(control_hover))
            .when(self.state.dropdown_open, |el| {
                el.border_color(theme.accent.accent)
            })
            .cursor_pointer()
            .on_mouse_down(
                MouseButton::Left,
                cx.listener(|this, _event, _window, cx| {
                    cx.stop_propagation();
                    this.dismiss_context_menu();
                    this.resources.namespaces.close_dropdown();
                    this.settings_dropdown.close();
                    this.state.toggle_dropdown();
                    cx.notify();
                }),
            )
            .child(
                div()
                    .w(px(8.0))
                    .h(px(8.0))
                    .rounded_full()
                    .flex_shrink_0()
                    .bg(presentation.color),
            )
            .child(
                apply_text_step(div(), theme.typography.size_xs)
                    .font_weight(FontWeight::MEDIUM)
                    .max_w(px(220.0))
                    .truncate()
                    .child(presentation.pill_label),
            )
            .child(
                apply_text_step(div(), theme.typography.size_xs)
                    .text_color(theme.text.muted)
                    .flex_shrink_0()
                    .child(presentation.status_label),
            )
            .child(render_icon(cluster_chevron, px(13.0), theme.text.secondary));

        let cluster_control =
            div()
                .flex()
                .flex_col()
                .child(cluster_trigger)
                .when(self.state.dropdown_open, |el| {
                    el.child(Self::render_anchored_dropdown(
                        self.render_cluster_dropdown_menu(&theme, cx),
                    ))
                });

        div()
            .flex()
            .items_center()
            .gap_1p5()
            .child(cluster_control)
            .when(
                should_show_namespace_trigger(&self.state.connection_state),
                |el| el.child(self.render_namespace_trigger_control(&theme, cx)),
            )
            .into_any_element()
    }

    fn render_titlebar_right_controls(&self, cx: &mut Context<Self>) -> AnyElement {
        let theme = OrbitTheme::get(cx).clone();
        let control_hover = theme.surface.control_hover;
        let is_refreshing = self.is_active_view_loading();

        div()
            .flex()
            .items_center()
            .gap_1p5()
            .child(
                theme
                    .style_control(div().id("command-palette-trigger-btn"))
                    .flex()
                    .items_center()
                    .gap_1p5()
                    .h(px(TITLEBAR_CONTROL_HEIGHT))
                    .px(theme.typography.control_padding_x_sm)
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
                    .on_mouse_down(MouseButton::Left, |_event, _window, cx| {
                        cx.stop_propagation();
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
                    .h(px(TITLEBAR_CONTROL_HEIGHT))
                    .px(theme.typography.control_padding_x_sm)
                    .when(is_refreshing, |el| el.opacity(0.6))
                    .when(!is_refreshing, |el| {
                        el.hover(move |style| style.bg(control_hover))
                            .cursor_pointer()
                    })
                    .tooltip(|_window, cx| {
                        cx.new(|_| TextTooltip::new("Refresh workspace (R)")).into()
                    })
                    .on_mouse_down(MouseButton::Left, |_event, _window, cx| {
                        cx.stop_propagation();
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
                        apply_text_step(div(), theme.typography.size_xs)
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
                    .h(px(TITLEBAR_CONTROL_HEIGHT))
                    .px(theme.typography.control_padding_x_sm)
                    .hover(move |style| style.bg(control_hover))
                    .cursor_pointer()
                    .on_mouse_down(MouseButton::Left, |_event, _window, cx| {
                        cx.stop_propagation();
                    })
                    .on_click(cx.listener(|_this, _event, _window, cx| {
                        cx.quit();
                    }))
                    .child(render_icon(Icon::LogOut, px(13.0), theme.text.secondary))
                    .child(
                        apply_text_step(div(), theme.typography.size_xs)
                            .font_weight(FontWeight::MEDIUM)
                            .child("Switch to Old UI"),
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

    fn render_cluster_dropdown_menu(
        &self,
        theme: &OrbitTheme,
        cx: &mut Context<Self>,
    ) -> AnyElement {
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
                let dot_color = cluster_dot_color(cluster, theme);

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

        theme
            .style_overlay(div().id("cluster-dropdown-menu"))
            .w(px(400.0))
            .max_h(px(420.0))
            .overflow_y_scroll()
            .on_mouse_down(MouseButton::Left, |_event, _window, cx| {
                cx.stop_propagation();
            })
            .on_mouse_down(MouseButton::Right, |_event, _window, cx| {
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
                            .child(render_icon(Icon::Plus, px(12.0), theme.accent.accent))
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
            )
            .into_any_element()
    }

    fn render_namespace_search_input(
        &self,
        theme: &OrbitTheme,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let control_hover = theme.surface.control_hover;
        let has_query = !self.resources.namespaces.search_query.is_empty();
        let search_text = if has_query {
            self.resources.namespaces.search_query.clone()
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
                            this.resources.namespaces.search_query.clear();
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
        let is_all = self.resources.namespaces.is_all_namespaces();

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
                if let Some(cleared) = this.resources.namespaces.select_all_namespaces()
                    && let Some(cluster_id) = this.resources.namespaces.cluster_id.clone()
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
            .resources
            .namespaces
            .active_selection()
            .iter()
            .any(|s| s == &ns_name);
        let non_active_status = (ns.status != "Active").then(|| ns.status.clone());
        let cluster_id = self.resources.namespaces.cluster_id.clone();

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
                if let Some(updated) = this.resources.namespaces.toggle_namespace(&ns_name)
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
        let filtered = self.resources.namespaces.filtered_namespaces();
        let mut list = div()
            .id("namespace-dropdown-scroll-list")
            .flex()
            .flex_col()
            .gap_0p5()
            .p_1p5()
            .max_h(px(300.0))
            .overflow_y_scroll()
            .child(self.render_namespace_all_option(theme, cx));

        if self.resources.namespaces.status == NamespaceLoadStatus::Loading
            && self.resources.namespaces.namespaces.is_empty()
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

        if self.resources.namespaces.status == NamespaceLoadStatus::Error {
            let err_msg = self
                .resources
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
                                    if let Some(cmd) = this.resources.namespaces.begin_refresh() {
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
            let empty_msg = if self.resources.namespaces.search_query.trim().is_empty() {
                "No namespaces found in this cluster.".to_string()
            } else {
                format!(
                    "No namespaces match \"{}\".",
                    self.resources.namespaces.search_query.trim()
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

    fn render_namespace_dropdown_menu(
        &self,
        theme: &OrbitTheme,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        theme
            .style_overlay(div().id("namespace-dropdown-menu"))
            .w(px(320.0))
            .on_mouse_down(MouseButton::Left, |_event, _window, cx| {
                cx.stop_propagation();
            })
            .on_mouse_down(MouseButton::Right, |_event, _window, cx| {
                cx.stop_propagation();
            })
            .child(self.render_namespace_search_input(theme, cx))
            .child(self.render_namespace_list_body(theme, cx))
            .into_any_element()
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
        let exp_entity = cx.entity().clone();
        let reset_entity = cx.entity().clone();
        let toggle_dropdown_entity = cx.entity().clone();
        let hover_dropdown_entity = cx.entity().clone();

        let callbacks = views::settings::SettingsCallbacks {
            on_select_theme_mode: std::rc::Rc::new(move |mode, window, cx| {
                mode_entity.update(cx, |shell, cx| shell.select_theme_mode(mode, window, cx));
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
                    shell.resources.namespaces.close_dropdown();
                    shell.dismiss_context_menu();
                    let current_theme = OrbitTheme::get(cx);
                    let current_index = match kind {
                        views::settings::SettingsDropdownKind::ThemeMode => {
                            views::settings::theme_mode_index(current_theme.mode)
                        }
                    };
                    shell.settings_dropdown.toggle(kind, current_index);
                    if shell.settings_dropdown.is_any_open() {
                        shell.focus_handle.focus(window, cx);
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
        self.resources.namespaces.close_dropdown();
        self.settings_dropdown.close();
        self.active_shell_drawer.close();
    }

    fn toggle_command_palette(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.command_palette.is_open {
            self.close_command_palette(window, cx);
        } else {
            self.dismiss_overlays();
            self.command_palette.open();
            self.command_palette_focus.focus(window, cx);
            cx.notify();
        }
    }

    fn close_command_palette(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.command_palette.close();
        self.focus_handle.focus(window, cx);
        cx.notify();
    }

    fn execute_command_palette_action(
        &mut self,
        action: CommandPaletteAction,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.command_palette.close();
        self.focus_handle.focus(window, cx);
        match action {
            CommandPaletteAction::NavigateView(view) => {
                self.select_workspace_view(view, cx);
            }
            CommandPaletteAction::SwitchCluster(cluster_id) => {
                self.select_cluster(&cluster_id, cx);
            }
            CommandPaletteAction::SelectAllNamespaces
            | CommandPaletteAction::ClearNamespaceFilter => {
                if let Some(cleared) = self.resources.namespaces.select_all_namespaces()
                    && let Some(cluster_id) = self.resources.namespaces.cluster_id.clone()
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
                if let Some(updated) = self.resources.namespaces.toggle_namespace(&ns_name)
                    && let Some(cluster_id) = self.resources.namespaces.cluster_id.clone()
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
            &self.resources.namespaces,
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
                        shell.command_palette_focus.focus(window, cx);
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

    fn apply_focus_target(&self, target: ShellFocusTarget, window: &mut Window, cx: &mut App) {
        match target {
            ShellFocusTarget::ShellRoot => self.focus_handle.focus(window, cx),
            ShellFocusTarget::PickerSearch => self.picker_search_focus.focus(window, cx),
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
            cx,
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
            && !self
                .tabs
                .active_tab()
                .as_ref()
                .is_some_and(WorkspaceTab::is_ephemeral)
            && self.settings_dropdown.is_any_open()
    }

    fn render_workspace(
        &mut self,
        window_width: f32,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let theme = OrbitTheme::get(cx).clone();

        let body = if self.sidebar.active_view == WorkspaceView::Settings
            && !self
                .tabs
                .active_tab()
                .as_ref()
                .is_some_and(WorkspaceTab::is_ephemeral)
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
                            window,
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
            || self
                .tabs
                .active_tab()
                .as_ref()
                .is_some_and(WorkspaceTab::is_ephemeral))
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
                        self.tabs
                            .active_tab()
                            .as_ref()
                            .is_some_and(WorkspaceTab::is_ephemeral),
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
                    && this.pending_edit_close.is_none()
                {
                    this.toggle_command_palette(window, cx);
                }
            }))
            .on_action(cx.listener(|this, _: &ToggleSidebar, _window, cx| {
                if !this.add_cluster.is_open
                    && this.port_forward_modal.is_none()
                    && !this.workload_action_modal.is_open()
                    && this.pending_edit_close.is_none()
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
                                this.close_port_forward_modal();
                                this.focus_handle.focus(window, cx);
                                cx.notify();
                            }
                            PortForwardModalKeyResult::Submit => {
                                this.submit_port_forward_modal(cx);
                            }
                            PortForwardModalKeyResult::Handled => {
                                if key == "tab"
                                    && let Some(field) =
                                        this.port_forward_modal.as_ref().map(|m| m.active_field)
                                {
                                    this.focus_port_forward_field(field, window, cx);
                                }
                                cx.notify();
                            }
                            PortForwardModalKeyResult::Ignored => {}
                        }
                    }
                    return;
                }
                if this.pending_edit_close.is_some() {
                    if !(event.keystroke.modifiers.control
                        || event.keystroke.modifiers.alt
                        || event.keystroke.modifiers.platform)
                    {
                        let key = event.keystroke.key.to_ascii_lowercase();
                        let res = this
                            .pending_edit_close
                            .as_mut()
                            .map(|(_, d)| {
                                d.handle_key_input(&key, event.keystroke.key_char.as_deref())
                            })
                            .unwrap_or(ConfirmDialogKeyResult::Ignored);
                        match res {
                            ConfirmDialogKeyResult::Cancel => {
                                this.pending_edit_close = None;
                                cx.notify();
                            }
                            ConfirmDialogKeyResult::Submit => {
                                this.confirm_pending_edit_close(cx);
                            }
                            ConfirmDialogKeyResult::Handled => {
                                cx.notify();
                            }
                            ConfirmDialogKeyResult::Ignored => {}
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
                            &this.resources.namespaces,
                        );
                        let filtered = this.command_palette.filter_items(&catalog);
                        match this.command_palette.handle_key_input(
                            &key,
                            event.keystroke.key_char.as_deref(),
                            filtered.len(),
                        ) {
                            crate::components::CommandPaletteKeyResult::Close => {
                                this.focus_handle.focus(window, cx);
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
                                cx,
                            );
                            this.select_workspace_view(view, cx);
                        }
                        PickerKeyAction::Closed => {
                            this.apply_focus_target(
                                focus_target_for_picker_transition(false),
                                window,
                                cx,
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
                    && this.resources.port_forwards.search_focused
                    && !(event.keystroke.modifiers.control
                        || event.keystroke.modifiers.alt
                        || event.keystroke.modifiers.platform)
                    && this.resources.port_forwards.handle_search_key(
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
                if this.resources.namespaces.dropdown_open
                    && this.resources.namespaces.handle_search_key(
                        &event.keystroke.key,
                        event.keystroke.key_char.as_deref(),
                    )
                {
                    cx.notify();
                    return;
                }
                if dismiss_cluster_dropdown_on_escape(&event.keystroke.key, &mut this.state) {
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
                if let Some(WorkspaceTab::Edit(target)) = this.tabs.active_tab() {
                    let key = event.keystroke.key.to_ascii_lowercase();
                    let has_ctrl_or_cmd =
                        event.keystroke.modifiers.control || event.keystroke.modifiers.platform;
                    if has_ctrl_or_cmd && key == "s" {
                        this.submit_edit_tab_save(&target, cx);
                        return;
                    }
                    if has_ctrl_or_cmd && key == "w" {
                        this.close_tab(WorkspaceTab::Edit(target), cx);
                        return;
                    }
                    if this
                        .edit_tabs
                        .get(&target)
                        .and_then(|entry| entry.entities.as_ref())
                        .is_some_and(|ent| ent.is_any_input_focused(window, cx))
                    {
                        return;
                    }
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
                        this.log_search_focus.focus(window, cx);
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
                                this.focus_handle.focus(window, cx);
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
                    &mut this.resources,
                    &mut this.tabs,
                    &event.keystroke.key,
                    KeyShortcutModifiers {
                        ctrl: event.keystroke.modifiers.control,
                        shift: event.keystroke.modifiers.shift,
                        platform: event.keystroke.modifiers.platform,
                    },
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
                TitlebarSlots {
                    left_controls: Some(self.render_titlebar_left_controls(cx)),
                    right_controls: Some(self.render_titlebar_right_controls(cx)),
                },
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
                            .when_some(self.render_error_banner(cx), |el, banner| el.child(banner))
                            .when_some(self.render_tab_bar(cx), |el, tab_bar| el.child(tab_bar))
                            .child(self.render_workspace(f32::from(viewport.width), window, cx)),
                    ),
            )
            .child(self.render_footer(cx))
            .when_some(
                self.render_shell_drawer_overlay(f32::from(viewport.width), cx),
                |el, overlay| el.child(overlay),
            )
            .when(self.state.dropdown_open, |el| {
                el.child(self.render_dropdown_backdrop(
                    "cluster-dropdown-backdrop",
                    |this| this.state.close_dropdown(),
                    cx,
                ))
            })
            .when(self.resources.namespaces.dropdown_open, |el| {
                el.child(self.render_dropdown_backdrop(
                    "namespace-dropdown-backdrop",
                    |this| this.resources.namespaces.close_dropdown(),
                    cx,
                ))
            })
            .when(self.is_settings_dropdown_visible(), |el| {
                el.child(self.render_dropdown_backdrop(
                    "settings-dropdown-backdrop",
                    |this| this.settings_dropdown.close(),
                    cx,
                ))
            })
            .when_some(self.render_add_cluster_modal_overlay(cx), |el, overlay| {
                el.child(overlay)
            })
            .when_some(
                self.render_workload_action_modal_overlay(cx),
                |el, overlay| el.child(overlay),
            )
            .when_some(self.render_pending_edit_close_overlay(cx), |el, overlay| {
                el.child(overlay)
            })
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
                                    cx,
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
                                    cx,
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
                                    cx,
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

    gpui::application().run(move |cx: &mut App| {
        gpui::init(cx);

        if let Err(err) = crate::icons::register_lucide_font(cx) {
            tracing::warn!(error = %err, "Failed to register embedded Lucide icon font");
        }

        cx.bind_keys([
            gpui::KeyBinding::new("ctrl-k", OpenCommandPalette, None),
            gpui::KeyBinding::new("cmd-k", OpenCommandPalette, None),
            gpui::KeyBinding::new("ctrl-b", ToggleSidebar, None),
            gpui::KeyBinding::new("cmd-b", ToggleSidebar, None),
        ]);

        cx.on_window_closed(|cx, _window_id| {
            if cx.windows().is_empty() {
                cx.quit();
            }
        })
        .detach();

        let initial_theme = OrbitTheme::from_config(&config, WindowAppearance::Dark);
        let bounds = OrbitShell::default_window_bounds(cx);
        let options = OrbitShell::window_options_for_theme(bounds, &initial_theme);
        let config_for_window = config.clone();

        if let Err(err) = gpui::open_window(options, cx, move |window, cx| {
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
    use crate::components::{ResourceDrawerTabId, ResourceDrawerTarget};
    use crate::state::{DeploymentsState, PortForwardsState};
    use crate::theme::{AccentColor, SemanticColorsPreference, UiStyle};
    use gpui::WindowBackgroundAppearance;

    #[test]
    fn test_apply_bridge_event_cluster_added_updates_clusters_and_handles_unreachable_outcome() {
        let mut state = ClusterSwitcherState::default();
        let mut sidebar = SidebarLayoutState::default();
        let mut resources = WorkspaceResourcesState::default();

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
            &mut resources,
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
            &mut resources,
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
        assert_eq!(
            opts.titlebar
                .as_ref()
                .and_then(|t| t.traffic_light_position),
            Some(gpui::point(
                px(12.0),
                px(crate::components::MACOS_TRAFFIC_LIGHT_Y),
            ))
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
        assert_eq!(OrbitShell::TITLEBAR_HEIGHT, 38.0);
        assert_eq!(
            OrbitShell::header_dropdown_top_offset(),
            OrbitShell::TITLEBAR_HEIGHT + 4.0
        );
        assert_eq!(OrbitShell::header_dropdown_top_offset(), 42.0);
        assert_eq!(
            OrbitShell::tab_picker_top_offset(),
            OrbitShell::TITLEBAR_HEIGHT + crate::components::TAB_BAR_HEIGHT + 4.0
        );
        assert_eq!(OrbitShell::tab_picker_top_offset(), 74.0);
    }

    #[test]
    fn test_cluster_and_namespace_dropdown_escape_dismissal() {
        let mut cluster_state = ClusterSwitcherState::new();
        cluster_state.dropdown_open = true;
        assert!(!dismiss_cluster_dropdown_on_escape("a", &mut cluster_state));
        assert!(cluster_state.dropdown_open);
        assert!(dismiss_cluster_dropdown_on_escape(
            "Escape",
            &mut cluster_state
        ));
        assert!(!cluster_state.dropdown_open);
        assert!(!dismiss_cluster_dropdown_on_escape(
            "escape",
            &mut cluster_state
        ));

        let mut ns_state = NamespaceFilterState {
            dropdown_open: true,
            search_query: "kube".to_string(),
            ..Default::default()
        };
        assert!(ns_state.handle_search_key("escape", None));
        assert!(!ns_state.dropdown_open);
        assert!(ns_state.search_query.is_empty());
    }

    #[test]
    fn test_namespace_trigger_visibility_and_indicators_preserved() {
        use orbit_core::kubernetes::models::NamespaceInfo;

        assert!(!should_show_namespace_trigger(
            &ClusterConnectionState::Disconnected
        ));
        assert!(!should_show_namespace_trigger(
            &ClusterConnectionState::Connecting {
                cluster_id: "prod".to_string(),
                display_name: "Prod".to_string(),
            }
        ));
        assert!(!should_show_namespace_trigger(
            &ClusterConnectionState::Unreachable {
                cluster_id: "prod".to_string(),
                display_name: "Prod".to_string(),
                server_url: None,
                error_message: "timeout".to_string(),
            }
        ));
        assert!(!should_show_namespace_trigger(
            &ClusterConnectionState::Error {
                cluster_id: Some("prod".to_string()),
                display_name: Some("Prod".to_string()),
                error_message: "auth failed".to_string(),
            }
        ));
        assert!(should_show_namespace_trigger(
            &ClusterConnectionState::Connected {
                cluster_id: "prod".to_string(),
                display_name: "Prod".to_string(),
                k8s_version: Some("v1.31.1".to_string()),
            }
        ));

        let theme = OrbitTheme::from_config(&OrbitConfig::default(), WindowAppearance::Dark);
        let mut ns_state = NamespaceFilterState::default();

        let idle = namespace_trigger_presentation(&ns_state, &theme);
        assert!(!idle.is_loading);
        assert!(!idle.is_error);
        assert!(!idle.is_filtered);
        assert_eq!(idle.indicator_color, theme.text.secondary);

        ns_state.begin_load_for_cluster("prod");
        let loading = namespace_trigger_presentation(&ns_state, &theme);
        assert!(loading.is_loading);
        assert!(!loading.is_error);
        assert_eq!(loading.indicator_color, theme.semantic.warning.solid);

        ns_state.apply_failed("prod", 1, "denied".to_string());
        let err = namespace_trigger_presentation(&ns_state, &theme);
        assert!(!err.is_loading);
        assert!(err.is_error);
        assert_eq!(err.indicator_color, theme.semantic.danger.solid);

        ns_state.apply_loaded(
            "prod",
            1,
            vec![NamespaceInfo {
                name: "default".to_string(),
                status: "Active".to_string(),
                is_system: false,
                age: "1d".to_string(),
                labels: Default::default(),
                annotations: Default::default(),
                uid: "uid-default".to_string(),
                created: "2026-01-01T00:00:00Z".to_string(),
            }],
        );
        let loaded_all = namespace_trigger_presentation(&ns_state, &theme);
        assert!(!loaded_all.is_loading);
        assert!(!loaded_all.is_error);
        assert!(!loaded_all.is_filtered);
        assert_eq!(loaded_all.indicator_color, theme.text.secondary);

        ns_state.toggle_namespace("default");
        let loaded_filtered = namespace_trigger_presentation(&ns_state, &theme);
        assert!(loaded_filtered.is_filtered);
        assert_eq!(loaded_filtered.indicator_color, theme.accent.accent);
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
        let mut throwaway_resources = WorkspaceResourcesState::default();
        apply_cluster_event_with_workspace(
            &mut switcher,
            &mut connect_sidebar,
            &mut throwaway_resources,
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
        let mut config = OrbitConfig::default();
        config.update_selected_namespaces_for_cluster(
            "prod-ctx",
            vec!["default".to_string(), "stale-ns".to_string()],
        );
        let mut resources = WorkspaceResourcesState::from_config(&config);

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
            &mut resources,
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
        assert_eq!(
            resources.overview.status,
            crate::state::OverviewLoadStatus::Loading
        );
        assert_eq!(resources.namespaces.status, NamespaceLoadStatus::Loading);

        let summary = ClusterOverviewSummary {
            k8s_version: Some("v1.31.1".to_string()),
            overall_health: ClusterOverallHealth::Healthy,
            ..Default::default()
        };
        let loaded_overview_cmds = apply_cluster_event_with_workspace(
            &mut switcher,
            &mut sidebar,
            &mut resources,
            ClusterBridgeEvent::OverviewLoaded {
                cluster_id: "prod-ctx".to_string(),
                generation: 1,
                summary: summary.clone(),
            },
        );
        assert!(loaded_overview_cmds.is_empty());
        assert_eq!(resources.overview.summary, Some(summary));

        let loaded_ns_cmds = apply_cluster_event_with_workspace(
            &mut switcher,
            &mut sidebar,
            &mut resources,
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
        assert_eq!(resources.namespaces.namespaces.len(), 1);

        let unreachable_cmds = apply_cluster_event_with_workspace(
            &mut switcher,
            &mut sidebar,
            &mut resources,
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
        assert!(resources.overview.summary.is_none());
        assert_eq!(
            resources.overview.status,
            crate::state::OverviewLoadStatus::Idle
        );
        assert!(resources.namespaces.namespaces.is_empty());
        assert_eq!(resources.namespaces.status, NamespaceLoadStatus::Idle);
        assert_eq!(
            resources.namespaces.selected_by_cluster.get("prod-ctx"),
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
        let mut resources = WorkspaceResourcesState::default();

        // Navigating to Pods does not trigger Deployments load.
        assert!(matches!(
            select_workspace_view_with_state(
                &switcher,
                &mut sidebar,
                &mut resources,
                WorkspaceView::Pods
            ),
            Some(ClusterBridgeCommand::LoadPods { .. })
        ));
        assert_eq!(
            resources.deployments.status,
            crate::state::DeploymentsLoadStatus::Idle
        );
        assert_eq!(sidebar.active_view, WorkspaceView::Pods);

        // Navigating to Deployments while connected & Idle triggers LoadDeployments once.
        let cmd = select_workspace_view_with_state(
            &switcher,
            &mut sidebar,
            &mut resources,
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
            resources.deployments.status,
            crate::state::DeploymentsLoadStatus::Loading
        );

        // Navigating away and back while already loading does not re-trigger.
        select_workspace_view_with_state(
            &switcher,
            &mut sidebar,
            &mut resources,
            WorkspaceView::Overview,
        );
        assert_eq!(
            select_workspace_view_with_state(
                &switcher,
                &mut sidebar,
                &mut resources,
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
        let mut resources = WorkspaceResourcesState::default();

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
            &mut resources,
            connect_event.clone(),
        );
        assert!(
            !cmds
                .iter()
                .any(|c| matches!(c, ClusterBridgeCommand::LoadDeployments { .. }))
        );
        assert_eq!(
            resources.deployments.cluster_id.as_deref(),
            Some("prod-ctx")
        );
        assert_eq!(
            resources.deployments.status,
            crate::state::DeploymentsLoadStatus::Idle
        );

        // Connect while already on Deployments: fires LoadDeployments eagerly.
        sidebar.select_view(WorkspaceView::Deployments);
        let cmds_on_deployments = apply_cluster_event_with_workspace(
            &mut switcher,
            &mut sidebar,
            &mut resources,
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
            &mut resources,
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
        assert!(resources.deployments.cluster_id.is_none());
        assert_eq!(
            resources.deployments.status,
            crate::state::DeploymentsLoadStatus::Idle
        );
    }

    #[test]
    fn test_apply_cluster_event_handles_deployments_loaded_and_failed() {
        use orbit_core::kubernetes::models::{DeploymentInfo, Replicas};

        let mut switcher = ClusterSwitcherState::new();
        let mut sidebar = SidebarLayoutState::new();
        let mut resources = WorkspaceResourcesState::default();

        resources.deployments.begin_load_for_cluster("prod-ctx");
        let current_gen = resources.deployments.generation;

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
            &mut resources,
            ClusterBridgeEvent::DeploymentsLoaded {
                cluster_id: "prod-ctx".to_string(),
                generation: current_gen + 10,
                deployments: vec![dep.clone()],
            },
        );
        assert!(resources.deployments.deployments.is_empty());

        // Matching generation updates state.
        apply_cluster_event_with_workspace(
            &mut switcher,
            &mut sidebar,
            &mut resources,
            ClusterBridgeEvent::DeploymentsLoaded {
                cluster_id: "prod-ctx".to_string(),
                generation: current_gen,
                deployments: vec![dep],
            },
        );
        assert_eq!(
            resources.deployments.status,
            crate::state::DeploymentsLoadStatus::Loaded
        );
        assert_eq!(resources.deployments.deployments.len(), 1);

        // Refresh + failure updates error state.
        resources.deployments.begin_refresh();
        let refresh_gen = resources.deployments.generation;
        apply_cluster_event_with_workspace(
            &mut switcher,
            &mut sidebar,
            &mut resources,
            ClusterBridgeEvent::DeploymentsLoadFailed {
                cluster_id: "prod-ctx".to_string(),
                generation: refresh_gen,
                message: "forbidden".to_string(),
            },
        );
        assert_eq!(
            resources.deployments.status,
            crate::state::DeploymentsLoadStatus::Error
        );
        assert_eq!(
            resources.deployments.error_message.as_deref(),
            Some("forbidden")
        );
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
        let mut resources = WorkspaceResourcesState::default();

        let cmd = select_workspace_view_with_state(
            &switcher,
            &mut sidebar,
            &mut resources,
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
        assert_eq!(
            resources.nodes.status,
            crate::state::NodesLoadStatus::Loading
        );

        // Navigating away and back while already loading/loaded does not re-trigger.
        select_workspace_view_with_state(
            &switcher,
            &mut sidebar,
            &mut resources,
            WorkspaceView::Overview,
        );
        assert_eq!(
            select_workspace_view_with_state(
                &switcher,
                &mut sidebar,
                &mut resources,
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
        let mut resources = WorkspaceResourcesState::default();

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
            &mut resources,
            connect_event.clone(),
        );
        assert!(
            !cmds
                .iter()
                .any(|c| matches!(c, ClusterBridgeCommand::LoadNodes { .. }))
        );
        assert_eq!(resources.nodes.cluster_id.as_deref(), Some("prod-ctx"));
        assert_eq!(resources.nodes.status, crate::state::NodesLoadStatus::Idle);

        // Connect while already on Nodes: fires LoadNodes eagerly.
        sidebar.select_view(WorkspaceView::Nodes);
        let cmds_on_nodes = apply_cluster_event_with_workspace(
            &mut switcher,
            &mut sidebar,
            &mut resources,
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
            &mut resources,
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
        assert!(resources.nodes.cluster_id.is_none());
        assert_eq!(resources.nodes.status, crate::state::NodesLoadStatus::Idle);
    }

    #[test]
    fn test_apply_cluster_event_handles_nodes_loaded_and_failed() {
        use orbit_core::kubernetes::models::NodeInfo;

        let mut switcher = ClusterSwitcherState::new();
        let mut sidebar = SidebarLayoutState::new();
        let mut resources = WorkspaceResourcesState::default();

        resources.nodes.begin_load_for_cluster("prod-ctx");
        let current_gen = resources.nodes.generation;

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
            &mut resources,
            ClusterBridgeEvent::NodesLoaded {
                cluster_id: "prod-ctx".to_string(),
                generation: current_gen + 10,
                nodes: vec![node.clone()],
            },
        );
        assert!(resources.nodes.nodes.is_empty());

        // Matching generation updates state.
        apply_cluster_event_with_workspace(
            &mut switcher,
            &mut sidebar,
            &mut resources,
            ClusterBridgeEvent::NodesLoaded {
                cluster_id: "prod-ctx".to_string(),
                generation: current_gen,
                nodes: vec![node],
            },
        );
        assert_eq!(
            resources.nodes.status,
            crate::state::NodesLoadStatus::Loaded
        );
        assert_eq!(resources.nodes.nodes.len(), 1);

        // Refresh + failure updates error state.
        resources.nodes.begin_refresh();
        let refresh_gen = resources.nodes.generation;
        apply_cluster_event_with_workspace(
            &mut switcher,
            &mut sidebar,
            &mut resources,
            ClusterBridgeEvent::NodesLoadFailed {
                cluster_id: "prod-ctx".to_string(),
                generation: refresh_gen,
                message: "forbidden".to_string(),
            },
        );
        assert_eq!(resources.nodes.status, crate::state::NodesLoadStatus::Error);
        assert_eq!(resources.nodes.error_message.as_deref(), Some("forbidden"));
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
        let mut resources = WorkspaceResourcesState::default();

        let cmd = select_workspace_view_with_state(
            &switcher,
            &mut sidebar,
            &mut resources,
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
            &mut resources,
            WorkspaceView::Overview,
        );
        assert!(
            select_workspace_view_with_state(
                &switcher,
                &mut sidebar,
                &mut resources,
                WorkspaceView::Namespaces,
            )
            .is_none()
        );

        // Loaded state also does not re-trigger LoadNamespaces on view switch.
        resources.namespaces.apply_loaded("prod-ctx", 1, Vec::new());
        assert!(
            select_workspace_view_with_state(
                &switcher,
                &mut sidebar,
                &mut resources,
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
        let mut resources = WorkspaceResourcesState::default();

        resources.namespaces_view.table.selected_row_key = Some("default".into());
        apply_cluster_event_with_workspace(
            &mut switcher,
            &mut sidebar,
            &mut resources,
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
        assert!(resources.namespaces_view.table.selected_row_key.is_none());

        resources.namespaces_view.table.selected_row_key = Some("kube-system".into());
        apply_cluster_event_with_workspace(
            &mut switcher,
            &mut sidebar,
            &mut resources,
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
        assert!(resources.namespaces_view.table.selected_row_key.is_none());
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
        let mut resources = WorkspaceResourcesState::default();

        let cmd = select_workspace_view_with_state(
            &switcher,
            &mut sidebar,
            &mut resources,
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
        assert_eq!(
            resources.events.status,
            crate::state::EventsLoadStatus::Loading
        );

        // Navigating away and back while Loading returns None.
        select_workspace_view_with_state(
            &switcher,
            &mut sidebar,
            &mut resources,
            WorkspaceView::Overview,
        );
        assert!(
            select_workspace_view_with_state(
                &switcher,
                &mut sidebar,
                &mut resources,
                WorkspaceView::Events,
            )
            .is_none()
        );

        // Loaded state also does not re-trigger LoadEvents on view switch.
        resources.events.apply_loaded("prod-ctx", 1, Vec::new());
        assert!(
            select_workspace_view_with_state(
                &switcher,
                &mut sidebar,
                &mut resources,
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
        let mut resources = WorkspaceResourcesState::default();

        // When active view is Overview, ClusterConnected resets EventsState to Idle without loading.
        let cmds_overview = apply_cluster_event_with_workspace(
            &mut switcher,
            &mut sidebar,
            &mut resources,
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
        assert_eq!(resources.events.cluster_id.as_deref(), Some("ctx-a"));
        assert_eq!(
            resources.events.status,
            crate::state::EventsLoadStatus::Idle
        );

        // When active view is Events, ClusterConnected emits LoadEvents.
        sidebar.select_view(WorkspaceView::Events);
        let cmds_events = apply_cluster_event_with_workspace(
            &mut switcher,
            &mut sidebar,
            &mut resources,
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
        assert_eq!(
            resources.events.status,
            crate::state::EventsLoadStatus::Loading
        );

        // ClusterUnreachable resets EventsState for disconnected state.
        apply_cluster_event_with_workspace(
            &mut switcher,
            &mut sidebar,
            &mut resources,
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
        assert!(resources.events.cluster_id.is_none());
        assert_eq!(
            resources.events.status,
            crate::state::EventsLoadStatus::Idle
        );
        assert!(resources.events.events.is_empty());
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
        let mut resources = WorkspaceResourcesState::default();
        let mut tabs = WorkspaceTabsState::new();
        tabs.activate_cluster("c1");

        let cmds1 = select_workspace_view_with_tabs(
            &state,
            &mut sidebar,
            &mut resources,
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
            &mut resources,
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
            &mut resources,
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
        let mut resources = WorkspaceResourcesState::default();
        let mut tabs = WorkspaceTabsState::from_config(&config);

        let cmds_a = apply_cluster_event_with_tabs(
            &mut state,
            &mut sidebar,
            &mut resources,
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
            &mut resources,
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
        let mut resources = WorkspaceResourcesState::default();
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
            &mut resources,
            &mut tabs,
            "tab",
            KeyShortcutModifiers {
                ctrl: true,
                shift: false,
                platform: false,
            },
        );
        assert!(handled);
        assert_eq!(tabs.active_view(), Some(WorkspaceView::Deployments));
        assert_eq!(sidebar.active_view, WorkspaceView::Deployments);

        // Ctrl+Shift+Tab -> Overview
        let (handled, _) = handle_tab_shortcut_with_state(
            &state,
            &mut sidebar,
            &mut resources,
            &mut tabs,
            "tab",
            KeyShortcutModifiers {
                ctrl: true,
                shift: true,
                platform: false,
            },
        );
        assert!(handled);
        assert_eq!(tabs.active_view(), Some(WorkspaceView::Overview));

        // Ctrl+W -> closes Overview, activates Deployments
        let (handled, _) = handle_tab_shortcut_with_state(
            &state,
            &mut sidebar,
            &mut resources,
            &mut tabs,
            "w",
            KeyShortcutModifiers {
                ctrl: true,
                shift: false,
                platform: false,
            },
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
                &mut resources,
                &mut tabs,
                "w",
                KeyShortcutModifiers {
                    ctrl: false,
                    shift: false,
                    platform: true,
                },
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
        let mut resources = WorkspaceResourcesState::default();
        let mut tabs = WorkspaceTabsState::default();
        tabs.activate_cluster("prod-ctx");

        let cmds = select_workspace_view_with_tabs(
            &state,
            &mut sidebar,
            &mut resources,
            &mut tabs,
            WorkspaceView::Pods,
        );
        assert_eq!(tabs.active_view(), Some(WorkspaceView::Pods));
        assert!(cmds.iter().any(|c| matches!(
            c,
            ClusterBridgeCommand::LoadPods { cluster_id, generation: 1 } if cluster_id == "prod-ctx"
        )));

        // Connecting while already on WorkspaceView::Pods also dispatches LoadPods.
        let mut resources_on_connect = WorkspaceResourcesState::default();
        let connect_cmds = apply_cluster_event_with_workspace(
            &mut state,
            &mut sidebar,
            &mut resources_on_connect,
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
        let mut resources = WorkspaceResourcesState::default();

        resources.pods.begin_load_for_cluster("prod-ctx");
        let gen1 = resources.pods.generation;

        apply_cluster_event_with_workspace(
            &mut state,
            &mut sidebar,
            &mut resources,
            ClusterBridgeEvent::PodsLoaded {
                cluster_id: "prod-ctx".to_string(),
                generation: gen1,
                pods: vec![make_test_pod("api-0", "prod")],
            },
        );
        assert_eq!(resources.pods.status, crate::state::PodsLoadStatus::Loaded);
        assert_eq!(resources.pods.pods.len(), 1);

        resources.pods.begin_refresh();
        let gen2 = resources.pods.generation;
        apply_cluster_event_with_workspace(
            &mut state,
            &mut sidebar,
            &mut resources,
            ClusterBridgeEvent::PodsLoadFailed {
                cluster_id: "prod-ctx".to_string(),
                generation: gen2,
                message: "forbidden".to_string(),
            },
        );
        assert_eq!(resources.pods.status, crate::state::PodsLoadStatus::Error);
        assert_eq!(resources.pods.error_message.as_deref(), Some("forbidden"));

        apply_cluster_event_with_workspace(
            &mut state,
            &mut sidebar,
            &mut resources,
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
        assert!(resources.pods.cluster_id.is_none());
        assert!(resources.pods.pods.is_empty());
        assert_eq!(resources.pods.status, crate::state::PodsLoadStatus::Idle);
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
                "edit",
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
        let mut resources = WorkspaceResourcesState::default();
        let mut tabs = WorkspaceTabsState::default();
        tabs.activate_cluster("prod-ctx");

        let cmds = select_workspace_view_with_tabs(
            &state,
            &mut sidebar,
            &mut resources,
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
        let mut resources_on_connect = WorkspaceResourcesState::default();
        let connect_cmds = apply_cluster_event_with_workspace(
            &mut state,
            &mut sidebar,
            &mut resources_on_connect,
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
        let mut resources = WorkspaceResourcesState::default();

        resources.services.begin_load_for_cluster("prod-ctx");
        let gen1 = resources.services.generation;

        // Stale generation should be ignored
        apply_cluster_event_with_workspace(
            &mut state,
            &mut sidebar,
            &mut resources,
            ClusterBridgeEvent::ServicesLoaded {
                cluster_id: "prod-ctx".to_string(),
                generation: gen1 + 99,
                services: vec![make_test_service("stale-svc", "prod")],
            },
        );
        assert!(resources.services.services.is_empty());

        // Matching generation updates state
        apply_cluster_event_with_workspace(
            &mut state,
            &mut sidebar,
            &mut resources,
            ClusterBridgeEvent::ServicesLoaded {
                cluster_id: "prod-ctx".to_string(),
                generation: gen1,
                services: vec![make_test_service("api-svc", "prod")],
            },
        );
        assert_eq!(
            resources.services.status,
            crate::state::ServicesLoadStatus::Loaded
        );
        assert_eq!(resources.services.services.len(), 1);

        resources.services.begin_refresh();
        let gen2 = resources.services.generation;
        apply_cluster_event_with_workspace(
            &mut state,
            &mut sidebar,
            &mut resources,
            ClusterBridgeEvent::ServicesLoadFailed {
                cluster_id: "prod-ctx".to_string(),
                generation: gen2,
                message: "forbidden".to_string(),
            },
        );
        assert_eq!(
            resources.services.status,
            crate::state::ServicesLoadStatus::Error
        );
        assert_eq!(
            resources.services.error_message.as_deref(),
            Some("forbidden")
        );
    }

    #[test]
    fn test_apply_cluster_event_resets_services_on_connect_and_disconnect() {
        let mut state = ClusterSwitcherState::new();
        let mut sidebar = SidebarLayoutState::new();
        let mut resources = WorkspaceResourcesState::default();

        resources.services.begin_load_for_cluster("old-ctx");
        let gen1 = resources.services.generation;
        resources.services.apply_loaded(
            "old-ctx",
            gen1,
            vec![make_test_service("old-svc", "default")],
        );
        assert_eq!(resources.services.services.len(), 1);

        // Connecting to a new cluster while on another view resets services for that cluster
        sidebar.active_view = WorkspaceView::Overview;
        apply_cluster_event_with_workspace(
            &mut state,
            &mut sidebar,
            &mut resources,
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
        assert_eq!(resources.services.cluster_id.as_deref(), Some("prod-ctx"));
        assert_eq!(
            resources.services.status,
            crate::state::ServicesLoadStatus::Idle
        );
        assert!(resources.services.services.is_empty());

        // Disconnecting clears cluster_id and services
        apply_cluster_event_with_workspace(
            &mut state,
            &mut sidebar,
            &mut resources,
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
        assert!(resources.services.cluster_id.is_none());
        assert!(resources.services.services.is_empty());
        assert_eq!(
            resources.services.status,
            crate::state::ServicesLoadStatus::Idle
        );
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
        let mut resources = WorkspaceResourcesState::default();

        let cmd = select_workspace_view_with_state(
            &switcher,
            &mut sidebar,
            &mut resources,
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
            resources.configmaps.status,
            crate::state::ConfigMapsLoadStatus::Loading
        );
    }

    #[test]
    fn test_configmaps_loaded_and_failed_events_update_state() {
        let mut switcher = ClusterSwitcherState::new();
        let mut sidebar = SidebarLayoutState::new();
        let mut resources = WorkspaceResourcesState::default();

        let _ = resources.configmaps.begin_load_for_cluster("prod-ctx");
        let cm = make_test_configmap("app-config", "prod");

        // Stale generation is ignored
        apply_cluster_event_with_workspace(
            &mut switcher,
            &mut sidebar,
            &mut resources,
            ClusterBridgeEvent::ConfigMapsLoaded {
                cluster_id: "prod-ctx".to_string(),
                generation: 99,
                configmaps: vec![cm.clone()],
            },
        );
        assert!(resources.configmaps.configmaps.is_empty());

        // Matching generation updates state
        apply_cluster_event_with_workspace(
            &mut switcher,
            &mut sidebar,
            &mut resources,
            ClusterBridgeEvent::ConfigMapsLoaded {
                cluster_id: "prod-ctx".to_string(),
                generation: 1,
                configmaps: vec![cm],
            },
        );
        assert_eq!(resources.configmaps.configmaps.len(), 1);
        assert_eq!(
            resources.configmaps.status,
            crate::state::ConfigMapsLoadStatus::Loaded
        );

        let _ = resources.configmaps.begin_refresh();
        apply_cluster_event_with_workspace(
            &mut switcher,
            &mut sidebar,
            &mut resources,
            ClusterBridgeEvent::ConfigMapsLoadFailed {
                cluster_id: "prod-ctx".to_string(),
                generation: 2,
                message: "forbidden".to_string(),
            },
        );
        assert_eq!(
            resources.configmaps.status,
            crate::state::ConfigMapsLoadStatus::Error
        );
        assert_eq!(
            resources.configmaps.error_message.as_deref(),
            Some("forbidden")
        );
    }

    #[test]
    fn test_apply_cluster_event_resets_configmaps_on_connect_and_disconnect() {
        use orbit_core::kubernetes::manager::ClusterSwitchOutcome;
        use orbit_core::kubernetes::models::UserProfileInfo;

        let mut switcher = ClusterSwitcherState::new();
        let mut sidebar = SidebarLayoutState::new();
        sidebar.select_view(WorkspaceView::ConfigMaps);
        let mut resources = WorkspaceResourcesState::default();

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
            &mut resources,
            connect_event,
        );
        assert!(cmds.contains(&ClusterBridgeCommand::LoadConfigMaps {
            cluster_id: "prod-ctx".to_string(),
            generation: 1,
        }));

        apply_cluster_event_with_workspace(
            &mut switcher,
            &mut sidebar,
            &mut resources,
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
        assert_eq!(
            resources.configmaps.status,
            crate::state::ConfigMapsLoadStatus::Idle
        );
        assert!(resources.configmaps.cluster_id.is_none());
        assert!(resources.configmaps.configmaps.is_empty());
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
        let mut resources = WorkspaceResourcesState::default();

        let cmd = select_workspace_view_with_state(
            &switcher,
            &mut sidebar,
            &mut resources,
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
        assert_eq!(
            resources.secrets.status,
            crate::state::SecretsLoadStatus::Loading
        );
    }

    #[test]
    fn test_secrets_loaded_and_failed_events_update_state() {
        let mut switcher = ClusterSwitcherState::new();
        let mut sidebar = SidebarLayoutState::new();
        let mut resources = WorkspaceResourcesState::default();

        let _ = resources.secrets.begin_load_for_cluster("prod-ctx");
        let sec = make_test_secret("db-creds", "prod");

        // Stale generation is ignored
        apply_cluster_event_with_workspace(
            &mut switcher,
            &mut sidebar,
            &mut resources,
            ClusterBridgeEvent::SecretsLoaded {
                cluster_id: "prod-ctx".to_string(),
                generation: 99,
                secrets: vec![sec.clone()],
            },
        );
        assert!(resources.secrets.secrets.is_empty());

        // Matching generation updates state
        apply_cluster_event_with_workspace(
            &mut switcher,
            &mut sidebar,
            &mut resources,
            ClusterBridgeEvent::SecretsLoaded {
                cluster_id: "prod-ctx".to_string(),
                generation: 1,
                secrets: vec![sec],
            },
        );
        assert_eq!(resources.secrets.secrets.len(), 1);
        assert_eq!(
            resources.secrets.status,
            crate::state::SecretsLoadStatus::Loaded
        );

        let _ = resources.secrets.begin_refresh();
        apply_cluster_event_with_workspace(
            &mut switcher,
            &mut sidebar,
            &mut resources,
            ClusterBridgeEvent::SecretsLoadFailed {
                cluster_id: "prod-ctx".to_string(),
                generation: 2,
                message: "forbidden".to_string(),
            },
        );
        assert_eq!(
            resources.secrets.status,
            crate::state::SecretsLoadStatus::Error
        );
        assert_eq!(
            resources.secrets.error_message.as_deref(),
            Some("forbidden")
        );
    }

    #[test]
    fn test_apply_cluster_event_resets_secrets_on_connect_and_disconnect() {
        use orbit_core::kubernetes::manager::ClusterSwitchOutcome;
        use orbit_core::kubernetes::models::UserProfileInfo;

        let mut switcher = ClusterSwitcherState::new();
        let mut sidebar = SidebarLayoutState::new();
        sidebar.select_view(WorkspaceView::Secrets);
        let mut resources = WorkspaceResourcesState::default();

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
            &mut resources,
            connect_event,
        );
        assert!(cmds.contains(&ClusterBridgeCommand::LoadSecrets {
            cluster_id: "prod-ctx".to_string(),
            generation: 1,
        }));

        apply_cluster_event_with_workspace(
            &mut switcher,
            &mut sidebar,
            &mut resources,
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
        assert_eq!(
            resources.secrets.status,
            crate::state::SecretsLoadStatus::Idle
        );
        assert!(resources.secrets.cluster_id.is_none());
        assert!(resources.secrets.secrets.is_empty());
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
        assert_eq!(restored.style.ui_style, UiStyle::Flat);
        assert_eq!(restored.accent_id, AccentColor::Blue);
        assert_eq!(reloaded.ui_style(), "paper");
        assert_eq!(reloaded.accent_color(), "emerald");
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
        let mut resources = WorkspaceResourcesState::default();

        let cmd = select_workspace_view_with_state(
            &switcher,
            &mut sidebar,
            &mut resources,
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
        let mut resources = WorkspaceResourcesState::default();

        let _ = resources.statefulsets.begin_load_for_cluster("prod-ctx");
        let ss = make_test_statefulset("redis", "cache");

        apply_cluster_event_with_workspace(
            &mut switcher,
            &mut sidebar,
            &mut resources,
            ClusterBridgeEvent::StatefulSetsLoaded {
                cluster_id: "prod-ctx".to_string(),
                generation: 1,
                statefulsets: vec![ss],
            },
        );
        assert_eq!(resources.statefulsets.statefulsets.len(), 1);
        assert_eq!(
            resources.statefulsets.status,
            crate::state::StatefulSetsLoadStatus::Loaded
        );

        let _ = resources.statefulsets.begin_refresh();
        apply_cluster_event_with_workspace(
            &mut switcher,
            &mut sidebar,
            &mut resources,
            ClusterBridgeEvent::StatefulSetsLoadFailed {
                cluster_id: "prod-ctx".to_string(),
                generation: 2,
                message: "forbidden".to_string(),
            },
        );
        assert_eq!(
            resources.statefulsets.status,
            crate::state::StatefulSetsLoadStatus::Error
        );
        assert_eq!(
            resources.statefulsets.error_message.as_deref(),
            Some("forbidden")
        );
    }

    #[test]
    fn test_connecting_while_on_statefulsets_view_dispatches_load_statefulsets() {
        use orbit_core::kubernetes::manager::ClusterSwitchOutcome;
        use orbit_core::kubernetes::models::UserProfileInfo;

        let mut switcher = ClusterSwitcherState::new();
        let mut sidebar = SidebarLayoutState::new();
        sidebar.select_view(WorkspaceView::StatefulSets);
        let mut resources = WorkspaceResourcesState::default();

        let cmds = apply_cluster_event_with_workspace(
            &mut switcher,
            &mut sidebar,
            &mut resources,
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
        let mut resources = WorkspaceResourcesState::default();

        // Selecting WorkloadsOverview while connected dispatches LoadWorkloadsOverview
        let cmd = select_workspace_view_with_state(
            &switcher,
            &mut sidebar,
            &mut resources,
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
            resources.workloads_overview.status,
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
            &mut resources,
            ClusterBridgeEvent::WorkloadsOverviewLoaded {
                cluster_id: "prod-ctx".to_string(),
                generation: 99,
                snapshot: sample.clone(),
            },
        );
        assert!(resources.workloads_overview.snapshot.is_none());

        // Matching generation applies snapshot
        apply_cluster_event_with_workspace(
            &mut switcher,
            &mut sidebar,
            &mut resources,
            ClusterBridgeEvent::WorkloadsOverviewLoaded {
                cluster_id: "prod-ctx".to_string(),
                generation: 1,
                snapshot: sample.clone(),
            },
        );
        assert_eq!(
            resources.workloads_overview.status,
            WorkloadsOverviewLoadStatus::Loaded
        );
        assert_eq!(
            resources.workloads_overview.snapshot.as_ref(),
            Some(&sample)
        );

        // Disconnect resets workloads_overview
        apply_cluster_event_with_workspace(
            &mut switcher,
            &mut sidebar,
            &mut resources,
            ClusterBridgeEvent::ConnectionError {
                cluster_id: Some("prod-ctx".to_string()),
                message: "offline".to_string(),
                clusters: Vec::new(),
            },
        );
        assert_eq!(
            resources.workloads_overview.status,
            WorkloadsOverviewLoadStatus::Idle
        );
        assert!(resources.workloads_overview.snapshot.is_none());

        // Reconnecting while WorkloadsOverview is active triggers LoadWorkloadsOverview
        sidebar.active_view = WorkspaceView::WorkloadsOverview;
        let reconnect_cmds = apply_cluster_event_with_workspace(
            &mut switcher,
            &mut sidebar,
            &mut resources,
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
        let mut resources = WorkspaceResourcesState::default();
        let mut tabs = WorkspaceTabsState::default();
        tabs.activate_cluster("prod-ctx");

        // Start on WorkloadsOverview
        let _ = select_workspace_view_with_tabs(
            &switcher,
            &mut sidebar,
            &mut resources,
            &mut tabs,
            WorkspaceView::WorkloadsOverview,
        );
        assert_eq!(sidebar.active_view, WorkspaceView::WorkloadsOverview);

        // Navigate to Pods
        let pods_cmds = select_workspace_view_with_tabs(
            &switcher,
            &mut sidebar,
            &mut resources,
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
            &mut resources,
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
            &mut resources,
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
            &mut resources,
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
            &mut resources,
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
            &mut resources,
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
        let mut resources = WorkspaceResourcesState::default();

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

        let _ = resources.replicasets.begin_load_for_cluster("prod-ctx");
        let rs = make_test_replicaset("web-7f8b", "prod");

        apply_cluster_event_with_workspace(
            &mut switcher,
            &mut sidebar,
            &mut resources,
            ClusterBridgeEvent::ReplicaSetsLoaded {
                cluster_id: "prod-ctx".to_string(),
                generation: 1,
                replicasets: vec![rs],
            },
        );
        assert_eq!(resources.replicasets.replicasets.len(), 1);
        assert_eq!(
            resources.replicasets.status,
            crate::state::ReplicaSetsLoadStatus::Loaded
        );

        let _ = resources.replicasets.begin_refresh();
        apply_cluster_event_with_workspace(
            &mut switcher,
            &mut sidebar,
            &mut resources,
            ClusterBridgeEvent::ReplicaSetsLoadFailed {
                cluster_id: "prod-ctx".to_string(),
                generation: 2,
                message: "forbidden".to_string(),
            },
        );
        assert_eq!(
            resources.replicasets.status,
            crate::state::ReplicaSetsLoadStatus::Error
        );
        assert_eq!(
            resources.replicasets.error_message.as_deref(),
            Some("forbidden")
        );
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
        let mut resources = WorkspaceResourcesState::default();

        let cmd = select_workspace_view_with_state(
            &switcher,
            &mut sidebar,
            &mut resources,
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
            &mut resources,
            ClusterBridgeEvent::DaemonSetsLoaded {
                cluster_id: "prod-ctx".to_string(),
                generation: 1,
                daemonsets: vec![ds],
            },
        );
        assert_eq!(resources.daemonsets.daemonsets.len(), 1);
        assert_eq!(
            resources.daemonsets.status,
            crate::state::DaemonSetsLoadStatus::Loaded
        );

        let _ = resources.daemonsets.begin_refresh();
        apply_cluster_event_with_workspace(
            &mut switcher,
            &mut sidebar,
            &mut resources,
            ClusterBridgeEvent::DaemonSetsLoadFailed {
                cluster_id: "prod-ctx".to_string(),
                generation: 2,
                message: "forbidden".to_string(),
            },
        );
        assert_eq!(
            resources.daemonsets.status,
            crate::state::DaemonSetsLoadStatus::Error
        );
        assert_eq!(
            resources.daemonsets.error_message.as_deref(),
            Some("forbidden")
        );
    }

    #[test]
    fn test_connecting_while_on_daemonsets_view_dispatches_load_daemonsets() {
        use orbit_core::kubernetes::manager::ClusterSwitchOutcome;
        use orbit_core::kubernetes::models::UserProfileInfo;

        let mut switcher = ClusterSwitcherState::new();
        let mut sidebar = SidebarLayoutState::new();
        sidebar.select_view(WorkspaceView::DaemonSets);
        let mut resources = WorkspaceResourcesState::default();

        let cmds = apply_cluster_event_with_workspace(
            &mut switcher,
            &mut sidebar,
            &mut resources,
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
        let mut resources = WorkspaceResourcesState::default();

        let cmd = select_workspace_view_with_state(
            &switcher,
            &mut sidebar,
            &mut resources,
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
            &mut resources,
            ClusterBridgeEvent::JobsLoaded {
                cluster_id: "prod-ctx".to_string(),
                generation: 1,
                jobs: vec![job],
            },
        );
        assert_eq!(resources.jobs.jobs.len(), 1);
        assert_eq!(resources.jobs.status, crate::state::JobsLoadStatus::Loaded);

        let _ = resources.jobs.begin_refresh();
        apply_cluster_event_with_workspace(
            &mut switcher,
            &mut sidebar,
            &mut resources,
            ClusterBridgeEvent::JobsLoadFailed {
                cluster_id: "prod-ctx".to_string(),
                generation: 2,
                message: "forbidden".to_string(),
            },
        );
        assert_eq!(resources.jobs.status, crate::state::JobsLoadStatus::Error);
        assert_eq!(resources.jobs.error_message.as_deref(), Some("forbidden"));
    }

    #[test]
    fn test_connecting_while_on_jobs_view_dispatches_load_jobs() {
        use orbit_core::kubernetes::manager::ClusterSwitchOutcome;
        use orbit_core::kubernetes::models::UserProfileInfo;

        let mut switcher = ClusterSwitcherState::new();
        let mut sidebar = SidebarLayoutState::new();
        sidebar.select_view(WorkspaceView::Jobs);
        let mut resources = WorkspaceResourcesState::default();

        let cmds = apply_cluster_event_with_workspace(
            &mut switcher,
            &mut sidebar,
            &mut resources,
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
        let mut resources = WorkspaceResourcesState::default();

        let cmd = select_workspace_view_with_state(
            &switcher,
            &mut sidebar,
            &mut resources,
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
            &mut resources,
            ClusterBridgeEvent::CronJobsLoaded {
                cluster_id: "prod-ctx".to_string(),
                generation: 1,
                cronjobs: vec![cj],
            },
        );
        assert_eq!(resources.cronjobs.cronjobs.len(), 1);
        assert_eq!(
            resources.cronjobs.status,
            crate::state::CronJobsLoadStatus::Loaded
        );

        let _ = resources.cronjobs.begin_refresh();
        apply_cluster_event_with_workspace(
            &mut switcher,
            &mut sidebar,
            &mut resources,
            ClusterBridgeEvent::CronJobsLoadFailed {
                cluster_id: "prod-ctx".to_string(),
                generation: 2,
                message: "forbidden".to_string(),
            },
        );
        assert_eq!(
            resources.cronjobs.status,
            crate::state::CronJobsLoadStatus::Error
        );
        assert_eq!(
            resources.cronjobs.error_message.as_deref(),
            Some("forbidden")
        );
    }

    #[test]
    fn test_connecting_while_on_cronjobs_view_dispatches_load_cronjobs() {
        use orbit_core::kubernetes::manager::ClusterSwitchOutcome;
        use orbit_core::kubernetes::models::UserProfileInfo;

        let mut switcher = ClusterSwitcherState::new();
        let mut sidebar = SidebarLayoutState::new();
        sidebar.select_view(WorkspaceView::CronJobs);
        let mut resources = WorkspaceResourcesState::default();

        let cmds = apply_cluster_event_with_workspace(
            &mut switcher,
            &mut sidebar,
            &mut resources,
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
        let mut resources = WorkspaceResourcesState::default();
        let mut tabs = WorkspaceTabsState::default();
        tabs.activate_cluster("prod-ctx");

        let cmds = select_workspace_view_with_tabs(
            &state,
            &mut sidebar,
            &mut resources,
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
            &mut resources,
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
        assert_eq!(
            resources.ingresses.status,
            crate::state::IngressesLoadStatus::Loaded
        );
        assert_eq!(resources.ingresses.ingresses.len(), 1);
        assert_eq!(resources.ingresses.ingresses[0].name, "web-ingress");

        // Disconnecting clears ingresses state
        apply_cluster_event_with_workspace(
            &mut state,
            &mut sidebar,
            &mut resources,
            ClusterBridgeEvent::ConnectionError {
                cluster_id: Some("prod-ctx".to_string()),
                message: "lost connection".to_string(),
                clusters: vec![],
            },
        );
        assert_eq!(
            resources.ingresses.status,
            crate::state::IngressesLoadStatus::Idle
        );
        assert!(resources.ingresses.ingresses.is_empty());

        // Connecting while already on WorkspaceView::Ingresses dispatches LoadIngresses
        let connect_cmds = apply_cluster_event_with_workspace(
            &mut state,
            &mut sidebar,
            &mut resources,
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
        let mut resources = WorkspaceResourcesState::default();
        let mut tabs = WorkspaceTabsState::default();
        tabs.activate_cluster("prod-ctx");

        let cmds = select_workspace_view_with_tabs(
            &state,
            &mut sidebar,
            &mut resources,
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
            &mut resources,
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
            resources.persistent_volumes.status,
            crate::state::PersistentVolumesLoadStatus::Loaded
        );
        assert_eq!(resources.persistent_volumes.persistent_volumes.len(), 1);
        assert_eq!(
            resources.persistent_volumes.persistent_volumes[0].name,
            "pv-data-01"
        );

        // Disconnecting clears persistent_volumes state
        apply_cluster_event_with_workspace(
            &mut state,
            &mut sidebar,
            &mut resources,
            ClusterBridgeEvent::ConnectionError {
                cluster_id: Some("prod-ctx".to_string()),
                message: "lost connection".to_string(),
                clusters: vec![],
            },
        );
        assert_eq!(
            resources.persistent_volumes.status,
            crate::state::PersistentVolumesLoadStatus::Idle
        );
        assert!(resources.persistent_volumes.persistent_volumes.is_empty());

        // Connecting while already on WorkspaceView::PersistentVolumes dispatches LoadPersistentVolumes
        let connect_cmds = apply_cluster_event_with_workspace(
            &mut state,
            &mut sidebar,
            &mut resources,
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
        let mut resources = WorkspaceResourcesState::default();
        let mut tabs = WorkspaceTabsState::default();
        tabs.activate_cluster("prod-ctx");

        let cmds = select_workspace_view_with_tabs(
            &state,
            &mut sidebar,
            &mut resources,
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
            &mut resources,
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
            resources.persistent_volume_claims.status,
            crate::state::PersistentVolumeClaimsLoadStatus::Loaded
        );
        assert_eq!(
            resources
                .persistent_volume_claims
                .persistent_volume_claims
                .len(),
            1
        );
        assert_eq!(
            resources.persistent_volume_claims.persistent_volume_claims[0].name,
            "data-pg-0"
        );

        // Disconnecting clears persistent_volume_claims state
        apply_cluster_event_with_workspace(
            &mut state,
            &mut sidebar,
            &mut resources,
            ClusterBridgeEvent::ConnectionError {
                cluster_id: Some("prod-ctx".to_string()),
                message: "lost connection".to_string(),
                clusters: vec![],
            },
        );
        assert_eq!(
            resources.persistent_volume_claims.status,
            crate::state::PersistentVolumeClaimsLoadStatus::Idle
        );
        assert!(
            resources
                .persistent_volume_claims
                .persistent_volume_claims
                .is_empty()
        );

        // Connecting while already on WorkspaceView::PersistentVolumeClaims dispatches LoadPersistentVolumeClaims
        let connect_cmds = apply_cluster_event_with_workspace(
            &mut state,
            &mut sidebar,
            &mut resources,
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
        let mut resources = WorkspaceResourcesState::default();
        let mut tabs = WorkspaceTabsState::default();
        tabs.activate_cluster("prod-ctx");

        let cmds = select_workspace_view_with_tabs(
            &state,
            &mut sidebar,
            &mut resources,
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
            &mut resources,
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
            resources.network_policies.status,
            crate::state::NetworkPoliciesLoadStatus::Loaded
        );
        assert_eq!(resources.network_policies.network_policies.len(), 1);
        assert_eq!(
            resources.network_policies.network_policies[0].name,
            "allow-api"
        );

        // Disconnecting clears network_policies state
        apply_cluster_event_with_workspace(
            &mut state,
            &mut sidebar,
            &mut resources,
            ClusterBridgeEvent::ConnectionError {
                cluster_id: Some("prod-ctx".to_string()),
                message: "lost connection".to_string(),
                clusters: vec![],
            },
        );
        assert_eq!(
            resources.network_policies.status,
            crate::state::NetworkPoliciesLoadStatus::Idle
        );
        assert!(resources.network_policies.network_policies.is_empty());

        // Connecting while already on WorkspaceView::NetworkPolicies dispatches LoadNetworkPolicies
        let connect_cmds = apply_cluster_event_with_workspace(
            &mut state,
            &mut sidebar,
            &mut resources,
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
        let mut resources = WorkspaceResourcesState::default();
        let mut tabs = WorkspaceTabsState::default();
        tabs.activate_cluster("prod-ctx");

        let cmds = select_workspace_view_with_tabs(
            &state,
            &mut sidebar,
            &mut resources,
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

        let refresh_cmd = resources.pod_security.begin_refresh();
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
            &mut resources,
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
        let mut resources = WorkspaceResourcesState::default();
        let mut tabs = WorkspaceTabsState::default();
        tabs.activate_cluster("prod-ctx");

        let cmds = select_workspace_view_with_tabs(
            &state,
            &mut sidebar,
            &mut resources,
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

        let refresh_cmd = resources.resource_quotas.begin_refresh();
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
            &mut resources,
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
        let mut resources = WorkspaceResourcesState::default();
        let mut tabs = WorkspaceTabsState::default();
        tabs.activate_cluster("prod-ctx");

        let cmds = select_workspace_view_with_tabs(
            &state,
            &mut sidebar,
            &mut resources,
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

        let refresh_cmd = resources.storage_overview.begin_refresh();
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
            &mut resources,
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
        let mut resources = WorkspaceResourcesState::default();
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
            &mut resources,
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
        let _ = resources.pods.begin_load_for_cluster("ctx-a");
        let gen_before = resources.pods.generation;
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
            &mut resources,
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
            &mut resources,
            &mut tabs,
            ClusterBridgeEvent::PodsLoaded {
                cluster_id: "ctx-a".to_string(),
                generation: gen_before,
                pods: Vec::new(),
            },
        );
        assert!(resources.pods.pods.is_empty());

        // ClusterDisconnected event keeps tabs cleared and sidebar on Clusters
        let _ = apply_cluster_event_with_tabs(
            &mut state,
            &mut sidebar,
            &mut resources,
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
        resources: WorkspaceResourcesState,
    }

    impl TestWorkloadActionTargets {
        fn with_cluster(cluster_id: &str) -> Self {
            let mut this = Self::default();
            let cid = Some(cluster_id.to_string());
            this.resources.deployments.cluster_id = cid.clone();
            this.resources.statefulsets.cluster_id = cid.clone();
            this.resources.daemonsets.cluster_id = cid.clone();
            this.resources.replicasets.cluster_id = cid.clone();
            this.resources.pods.cluster_id = cid.clone();
            this.resources.cronjobs.cluster_id = cid.clone();
            this.resources.jobs.cluster_id = cid.clone();
            this.resources.services.cluster_id = cid.clone();
            this.resources.ingresses.cluster_id = cid.clone();
            this.resources.configmaps.cluster_id = cid.clone();
            this.resources.secrets.cluster_id = cid.clone();
            this.resources.network_policies.cluster_id = cid.clone();
            this.resources.resource_quotas.cluster_id = cid.clone();
            this.resources.namespaces.cluster_id = cid.clone();
            this.resources.persistent_volumes.cluster_id = cid.clone();
            this.resources.persistent_volume_claims.cluster_id = cid.clone();
            this.resources.nodes.cluster_id = cid.clone();
            this.resources.events.cluster_id = cid;
            this
        }
    }

    #[test]
    fn test_workload_action_succeeded_closes_modal_shows_toast_and_queues_targeted_refresh() {
        let mut modal = WorkloadActionModalState::default();
        modal.open_scale("prod", "Deployment", "default", "web", 3);
        modal.set_submitting(true);
        let mut toast = ActionToastState::default();

        let mut targets = TestWorkloadActionTargets::with_cluster("prod-ctx");
        targets.resources.deployments.deployments =
            vec![orbit_core::kubernetes::models::DeploymentInfo {
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
            &mut targets.drawer,
            &mut targets.resources,
            &scale_event,
        )
        .expect("should handle success event");

        assert!(!modal.is_open());
        assert_eq!(res.1, ActionToastKind::Success);
        assert!(toast.current().is_some());
        assert_eq!(
            targets.resources.deployments.deployments.len(),
            1,
            "existing rows must not be cleared"
        );
        assert!(
            !targets.resources.deployments.table.loading,
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
            &mut targets.drawer,
            &mut targets.resources,
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
            &mut targets.drawer,
            &mut targets.resources,
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
            &mut targets.drawer,
            &mut targets.resources,
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
        targets.resources.deployments.deployments =
            vec![orbit_core::kubernetes::models::DeploymentInfo {
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
            &mut targets.drawer,
            &mut targets.resources,
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
        assert_eq!(targets.resources.deployments.deployments.len(), 1);
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
                &mut targets.drawer,
                &mut targets.resources,
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
            &mut targets.drawer,
            &mut targets.resources,
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
            &mut targets.drawer,
            &mut targets.resources,
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
            &mut targets.drawer,
            &mut targets.resources,
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
        let stop_cmds = cleanup_closed_log_tabs_in_state(&mut log_tabs, closed.logs);
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

    #[test]
    fn drain_event_batch_caps_at_max_and_coalesces_burst() {
        use orbit_core::kubernetes::watchers::WatchUpdate;

        let (tx, mut rx) = mpsc::unbounded_channel();
        for _ in 0..500 {
            let _ = tx.send(ClusterBridgeEvent::ResourceWatchUpdated {
                cluster_id: "ctx-a".to_string(),
                generation: 1,
                payload: crate::state::ResourceWatchPayload::Pods(WatchUpdate::Delta(Vec::new())),
            });
        }

        let Some(first) = rx.try_recv().ok() else {
            panic!("expected queued event");
        };
        let batch = drain_ready_events(&mut rx, first, MAX_EVENTS_PER_FRAME_BATCH);
        assert_eq!(batch.len(), MAX_EVENTS_PER_FRAME_BATCH);

        let mut remaining = 0usize;
        while rx.try_recv().is_ok() {
            remaining += 1;
        }
        assert_eq!(remaining, 500 - MAX_EVENTS_PER_FRAME_BATCH);
    }

    #[test]
    fn should_pace_after_batch_only_throttles_watch_bursts_not_user_actions() {
        let pure_watch = BatchApplySummary {
            should_notify: true,
            pure_watch_batch: true,
        };
        let wait = should_pace_after_batch(pure_watch, std::time::Duration::from_millis(4));
        assert_eq!(wait, Some(std::time::Duration::from_millis(12)));
        assert_eq!(
            should_pace_after_batch(pure_watch, FRAME_PACING_INTERVAL),
            None
        );

        let bg_watch = BatchApplySummary {
            should_notify: false,
            pure_watch_batch: true,
        };
        assert_eq!(
            should_pace_after_batch(bg_watch, std::time::Duration::from_millis(1)),
            None
        );

        let user_action = BatchApplySummary {
            should_notify: true,
            pure_watch_batch: false,
        };
        assert_eq!(
            should_pace_after_batch(user_action, std::time::Duration::from_millis(1)),
            None
        );
    }

    #[test]
    fn burst_stress_10000_pod_updates_stays_fast_and_bounded() {
        use orbit_core::kubernetes::models::PodInfo;
        use orbit_core::kubernetes::watchers::{
            TypedEmitDedup, TypedWatchBuffer, WatchAction, WatchItem, WatchUpdate,
        };

        let start = std::time::Instant::now();
        let mut dedup = TypedEmitDedup::new();
        let mut buffer = TypedWatchBuffer::<PodInfo>::new();
        let mut pods_state = crate::state::PodsState::default();
        let _ = pods_state.begin_load_for_cluster("ctx-a");
        pods_state.watch_generation = 1;
        pods_state.table.sort = Some(crate::components::data_table::SortState {
            column_id: "status".to_string(),
            direction: crate::components::data_table::SortDirection::Asc,
        });
        let ns = NamespaceFilterState::default();

        const UNIQUE_PODS: usize = 500;
        for i in 0..10_000usize {
            let pod_idx = i % UNIQUE_PODS;
            let name = format!("pod-{pod_idx:04}");
            let key = format!("prod/{name}");
            let status = if i % 3 == 0 { "Running" } else { "Pending" };
            let pod = PodInfo {
                name,
                namespace: "prod".to_string(),
                status: status.to_string(),
                age: "1m".to_string(),
                cpu: None,
                memory: None,
                cpu_cores: None,
                memory_bytes: None,
                cpu_pct: None,
                memory_pct: None,
                node: Some("node-1".to_string()),
                restarts: (i / UNIQUE_PODS) as i32,
                images: Vec::new(),
                labels: Default::default(),
                annotations: Default::default(),
                ip: None,
                node_ip: None,
                controlled_by: None,
                qos_class: None,
                containers: Vec::new(),
            };

            if dedup.should_emit(&key, WatchAction::Applied, &pod)
                && buffer.upsert(WatchItem {
                    key,
                    action: WatchAction::Applied,
                    item: pod,
                })
                && let Some(batch) = buffer.take_batch()
            {
                let _ = pods_state.apply_pod_watch_update(
                    "ctx-a",
                    1,
                    WatchUpdate::Delta(batch),
                );
            }
        }
        if let Some(batch) = buffer.take_batch() {
            let _ = pods_state.apply_pod_watch_update("ctx-a", 1, WatchUpdate::Delta(batch));
        }

        let rows_a = views::pods::cached_filtered_pods(&pods_state, &ns);
        let rows_b = views::pods::cached_filtered_pods(&pods_state, &ns);
        assert!(std::rc::Rc::ptr_eq(&rows_a, &rows_b));
        assert_eq!(pods_state.pods.len(), UNIQUE_PODS);
        assert_eq!(pods_state.key_index.len(), UNIQUE_PODS);
        assert_eq!(rows_a.len(), UNIQUE_PODS);
        assert!(
            start.elapsed() < std::time::Duration::from_millis(500),
            "10,000 pod burst took {:?}, exceeding 500ms budget",
            start.elapsed()
        );
    }
}
