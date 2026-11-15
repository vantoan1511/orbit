mod bridge;
pub mod components;
pub mod icons;
pub mod navigation;
mod state;
pub mod theme;
pub mod views;

use gpui::{
    AnyElement, App, Application, Bounds, Context, CursorStyle, FocusHandle, FontWeight,
    KeyDownEvent, MouseButton, MouseMoveEvent, Rgba, SharedString, Window,
    WindowAppearance, WindowBounds, WindowDecorations, WindowOptions, div, prelude::*, px, size,
};
use lucide_icons::Icon;
use orbit_core::config::OrbitConfig;
use orbit_core::kubernetes::models::ClusterInfo;
use tokio::sync::mpsc;

use crate::bridge::{ClusterBridge, start_cluster_bridge};
use crate::icons::{render_icon, theme_mode_icon};
use crate::navigation::{
    ACTIVITY_BAR_WIDTH, SidebarCategory, SidebarLayoutState, WorkspaceView,
};
use crate::state::{
    ClusterBridgeCommand, ClusterBridgeEvent, ClusterConnectionState, ClusterOverviewState,
    ClusterSwitcherState, NamespaceFilterState, NamespaceLoadStatus,
};
use crate::theme::{OrbitTheme, ThemeMode, UiStyle, apply_text_step, rgb_hex};

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
    cx.set_global(updated);
    window.set_background_appearance(bg);
}

pub fn nav_item_icon_color(view: WorkspaceView, is_active: bool, theme: &OrbitTheme) -> Rgba {
    view.resource_kind()
        .map(|kind| theme.k8s_color(kind))
        .unwrap_or(if is_active {
            theme.accent.accent
        } else {
            theme.text.muted
        })
}

pub const fn sidebar_toggle_icon(is_collapsed: bool) -> Icon {
    if is_collapsed {
        Icon::PanelLeftOpen
    } else {
        Icon::PanelLeftClose
    }
}

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

pub fn apply_cluster_event_with_workspace(
    state: &mut ClusterSwitcherState,
    sidebar: &mut SidebarLayoutState,
    overview: &mut ClusterOverviewState,
    namespaces: &mut NamespaceFilterState,
    event: ClusterBridgeEvent,
) -> Vec<ClusterBridgeCommand> {
    let mut commands = Vec::new();
    match event {
        ClusterBridgeEvent::ClusterConnected { .. } => {
            state.apply_event(event);
            sidebar.on_cluster_connected();
            if let Some(id) = state.active_cluster_id.as_deref() {
                commands.push(overview.begin_load_for_cluster(id));
                commands.push(namespaces.begin_load_for_cluster(id));
            }
        }
        ClusterBridgeEvent::ClusterUnreachable { .. }
        | ClusterBridgeEvent::ConnectionError { .. } => {
            state.apply_event(event);
            overview.reset_for_disconnected();
            namespaces.reset_for_disconnected();
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
        other @ ClusterBridgeEvent::ContextsLoaded { .. } => {
            state.apply_event(other);
        }
    }
    commands
}

pub struct OrbitShell {
    state: ClusterSwitcherState,
    sidebar: SidebarLayoutState,
    overview: ClusterOverviewState,
    namespaces: NamespaceFilterState,
    focus_handle: FocusHandle,
    namespace_search_focus: FocusHandle,
    command_tx: mpsc::UnboundedSender<ClusterBridgeCommand>,
    style_dropdown_open: bool,
}

impl OrbitShell {
    pub const WINDOW_TITLE: &'static str = "Orbit (GPUI Preview)";
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
                        for cmd in apply_cluster_event_with_workspace(
                            &mut shell.state,
                            &mut shell.sidebar,
                            &mut shell.overview,
                            &mut shell.namespaces,
                            event,
                        ) {
                            shell.dispatch_command(cmd);
                        }
                        cx.notify();
                    })
                    .is_err()
                {
                    break;
                }
            }
        })
        .detach();

        window
            .observe_window_appearance(|window, cx| {
                let current = OrbitTheme::get(cx);
                if current.mode == ThemeMode::System {
                    let updated = current.with_window_appearance(window.appearance());
                    apply_theme(updated, window, cx);
                    window.refresh();
                }
            })
            .detach();

        Self {
            state: ClusterSwitcherState::new(),
            sidebar: SidebarLayoutState::new(),
            overview: ClusterOverviewState::default(),
            namespaces: NamespaceFilterState::from_config(config),
            focus_handle: cx.focus_handle(),
            namespace_search_focus: cx.focus_handle(),
            command_tx,
            style_dropdown_open: false,
        }
    }

    fn dispatch_command(&self, cmd: ClusterBridgeCommand) {
        if let Err(err) = self.command_tx.send(cmd) {
            tracing::warn!(error = %err, "Failed to send command to cluster bridge");
        }
    }

    fn refresh_workspace(&mut self, cx: &mut Context<Self>) {
        self.dispatch_command(ClusterBridgeCommand::RefreshClusters);
        if matches!(
            self.state.connection_state,
            ClusterConnectionState::Connected { .. }
        ) {
            if let Some(cmd) = self.overview.begin_refresh() {
                self.dispatch_command(cmd);
            }
            if let Some(cmd) = self.namespaces.begin_refresh() {
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
        self.style_dropdown_open = false;
        cx.notify();
    }

    fn reconnect_cluster(&mut self, cx: &mut Context<Self>) {
        if let Some(cmd) = self.state.begin_reconnect() {
            self.dispatch_command(cmd);
        }
        self.namespaces.close_dropdown();
        cx.notify();
    }

    fn cycle_theme_mode(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let current = OrbitTheme::get(cx);
        let next_mode = current.mode.next();
        let updated = current.with_mode(next_mode, window.appearance());
        apply_theme(updated, window, cx);
        cx.notify();
    }

    fn select_ui_style(&mut self, style: UiStyle, window: &mut Window, cx: &mut Context<Self>) {
        let updated = OrbitTheme::get(cx).with_ui_style(style, window.appearance());
        apply_theme(updated, window, cx);
        self.style_dropdown_open = false;
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
        let hover_bg = theme.surface.bg_hover;
        let is_settings = self.sidebar.active_view == WorkspaceView::Settings;

        let mut categories_col = div().flex().flex_col().items_center().gap_1p5().w_full();
        for (idx, category) in SidebarCategory::ALL.iter().copied().enumerate() {
            let is_active =
                !is_settings && self.sidebar.active_category == category && !self.sidebar.is_collapsed;
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
                    .when(is_active, |el| el.bg(theme.accent.soft))
                    .hover(move |style| style.bg(hover_bg))
                    .on_click(cx.listener(move |this, _event, _window, cx| {
                        this.sidebar.toggle_category(category);
                        cx.notify();
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
                                .bg(theme.accent.accent),
                        )
                    })
                    .child(render_icon(
                        category.icon(),
                        px(18.0),
                        if is_selected_category {
                            theme.accent.accent
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
            .child(
                div()
                    .w(px(32.0))
                    .h(px(32.0))
                    .rounded(theme.style.radius_control)
                    .bg(theme.accent.soft)
                    .flex()
                    .items_center()
                    .justify_center()
                    .child(
                        apply_text_step(div(), theme.typography.size_sm)
                            .font_weight(FontWeight::BOLD)
                            .text_color(theme.accent.accent)
                            .child("O"),
                    ),
            )
            .child(categories_col)
            .child(
                div()
                    .mt_auto()
                    .flex()
                    .flex_col()
                    .items_center()
                    .gap_1p5()
                    .child(
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
                            .when(is_settings, |el| el.bg(theme.accent.soft))
                            .hover(move |style| style.bg(hover_bg))
                            .on_click(cx.listener(|this, _event, _window, cx| {
                                this.sidebar.select_view(WorkspaceView::Settings);
                                cx.notify();
                            }))
                            .child(render_icon(
                                Icon::Settings,
                                px(18.0),
                                if is_settings {
                                    theme.accent.accent
                                } else {
                                    theme.text.secondary
                                },
                            )),
                    )
                    .child(
                        div()
                            .id("activity-theme-btn")
                            .flex()
                            .items_center()
                            .justify_center()
                            .w(px(40.0))
                            .h(px(34.0))
                            .rounded(theme.style.radius_control)
                            .cursor_pointer()
                            .hover(move |style| style.bg(hover_bg))
                            .on_click(cx.listener(|this, _event, window, cx| {
                                this.cycle_theme_mode(window, cx);
                            }))
                            .child(render_icon(
                                theme_mode_icon(theme.mode),
                                px(18.0),
                                theme.text.muted,
                            )),
                    ),
            )
            .into_any_element()
    }

    fn render_clusters_sidebar_content(&self, cx: &mut Context<Self>) -> AnyElement {
        let theme = OrbitTheme::get(cx).clone();
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
                    .when(is_active, |el| el.bg(theme.accent.soft))
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
                                                theme.accent.accent
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
                    .when(is_active, |el| el.bg(theme.accent.soft))
                    .hover(move |style| style.bg(hover_bg))
                    .on_click(cx.listener(move |this, _event, _window, cx| {
                        this.sidebar.select_view(item);
                        cx.notify();
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
                                        theme.accent.accent
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
        let hover_bg = theme.surface.bg_hover;

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
                        .justify_between()
                        .child(
                            apply_text_step(div(), theme.typography.size_xs)
                                .font_weight(FontWeight::SEMIBOLD)
                                .text_color(theme.text.muted)
                                .child(category.label().to_ascii_uppercase()),
                        )
                        .child(
                            div()
                                .id("sidebar-collapse-btn")
                                .px_1p5()
                                .py_0p5()
                                .rounded(theme.style.radius_control)
                                .cursor_pointer()
                                .hover(move |style| style.bg(hover_bg))
                                .on_click(cx.listener(|this, _event, _window, cx| {
                                    this.sidebar.toggle_collapse();
                                    cx.notify();
                                }))
                                .child(render_icon(
                                    Icon::PanelLeftClose,
                                    px(14.0),
                                    theme.text.secondary,
                                )),
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
        let style_chevron = dropdown_chevron_icon(self.style_dropdown_open);
        let control_hover = theme.surface.control_hover;
        let (cat_label, view_label) = self.sidebar.active_view.breadcrumb();
        let is_settings = self.sidebar.active_view == WorkspaceView::Settings;

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
                            .style_control(div().id("sidebar-toggle-btn"))
                            .flex()
                            .items_center()
                            .h(theme.typography.control_height_sm)
                            .px(theme.typography.control_padding_x_sm)
                            .hover(move |style| style.bg(control_hover))
                            .cursor_pointer()
                            .on_click(cx.listener(|this, _event, _window, cx| {
                                this.sidebar.toggle_collapse();
                                cx.notify();
                            }))
                            .child(render_icon(
                                sidebar_toggle_icon(self.sidebar.is_collapsed),
                                px(14.0),
                                theme.text.secondary,
                            )),
                    )
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
                                this.style_dropdown_open = false;
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
                            .child(render_icon(
                                cluster_chevron,
                                px(13.0),
                                theme.text.secondary,
                            )),
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
                                        this.state.close_dropdown();
                                        this.style_dropdown_open = false;
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
                                    .child(render_icon(
                                        ns_chevron,
                                        px(13.0),
                                        theme.text.secondary,
                                    )),
                            )
                        },
                    )
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_1p5()
                            .child(
                                apply_text_step(div(), theme.typography.size_xs)
                                    .text_color(theme.text.muted)
                                    .child(cat_label),
                            )
                            .child(render_icon(
                                Icon::ChevronRight,
                                px(12.0),
                                theme.text.muted,
                            ))
                            .child(
                                apply_text_step(div(), theme.typography.size_xs)
                                    .font_weight(FontWeight::SEMIBOLD)
                                    .text_color(theme.text.primary)
                                    .child(view_label),
                            ),
                    ),
            )
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_2()
                    .child(
                        theme
                            .style_control(div().id("theme-mode-toggle-btn"))
                            .flex()
                            .items_center()
                            .gap_1p5()
                            .h(theme.typography.control_height_sm)
                            .px(theme.typography.control_padding_x_sm)
                            .hover(move |style| style.bg(control_hover))
                            .cursor_pointer()
                            .on_click(cx.listener(|this, _event, window, cx| {
                                this.cycle_theme_mode(window, cx);
                            }))
                            .child(render_icon(
                                theme_mode_icon(theme.mode),
                                px(13.0),
                                theme.text.secondary,
                            ))
                            .child(
                                apply_text_step(div(), theme.typography.size_xs)
                                    .text_color(theme.text.muted)
                                    .child("Mode:"),
                            )
                            .child(
                                apply_text_step(div(), theme.typography.size_xs)
                                    .font_weight(FontWeight::MEDIUM)
                                    .text_color(theme.text.primary)
                                    .child(theme.mode.label()),
                            ),
                    )
                    .child(
                        theme
                            .style_control(div().id("ui-style-selector-btn"))
                            .flex()
                            .items_center()
                            .gap_2()
                            .h(theme.typography.control_height_sm)
                            .px(theme.typography.control_padding_x_sm)
                            .hover(move |style| style.bg(control_hover))
                            .cursor_pointer()
                            .on_click(cx.listener(|this, _event, _window, cx| {
                                this.state.close_dropdown();
                                this.namespaces.close_dropdown();
                                this.style_dropdown_open = !this.style_dropdown_open;
                                cx.notify();
                            }))
                            .child(
                                apply_text_step(div(), theme.typography.size_xs)
                                    .text_color(theme.text.muted)
                                    .child("Style:"),
                            )
                            .child(
                                apply_text_step(div(), theme.typography.size_xs)
                                    .font_weight(FontWeight::MEDIUM)
                                    .text_color(theme.text.primary)
                                    .child(theme.style.ui_style.label()),
                            )
                            .child(render_icon(
                                style_chevron,
                                px(13.0),
                                theme.text.secondary,
                            )),
                    )
                    .child(
                        theme
                            .style_control(div().id("refresh-clusters-btn"))
                            .flex()
                            .items_center()
                            .gap_1p5()
                            .h(theme.typography.control_height_sm)
                            .px(theme.typography.control_padding_x_md)
                            .hover(move |style| style.bg(control_hover))
                            .cursor_pointer()
                            .on_click(cx.listener(|this, _event, _window, cx| {
                                this.refresh_workspace(cx);
                            }))
                            .child(render_icon(
                                Icon::RefreshCw,
                                px(13.0),
                                theme.text.secondary,
                            ))
                            .child(
                                apply_text_step(div(), theme.typography.size_sm)
                                    .font_weight(FontWeight::MEDIUM)
                                    .child("Refresh"),
                            ),
                    )
                    .child(
                        theme
                            .style_control(div().id("header-settings-btn"))
                            .flex()
                            .items_center()
                            .justify_center()
                            .w(theme.typography.control_height_sm)
                            .h(theme.typography.control_height_sm)
                            .when(is_settings, |el| el.bg(theme.accent.soft))
                            .hover(move |style| style.bg(control_hover))
                            .cursor_pointer()
                            .on_click(cx.listener(|this, _event, _window, cx| {
                                this.sidebar.select_view(WorkspaceView::Settings);
                                cx.notify();
                            }))
                            .child(render_icon(
                                Icon::Settings,
                                px(14.0),
                                if is_settings {
                                    theme.accent.accent
                                } else {
                                    theme.text.secondary
                                },
                            )),
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
                            .child(render_icon(
                                Icon::LogOut,
                                px(13.0),
                                theme.text.secondary,
                            ))
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
                        .child(
                            apply_text_step(div(), theme.typography.size_xs).child("Dismiss"),
                        ),
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
                                .child(
                                    div()
                                        .w(px(8.0))
                                        .h(px(8.0))
                                        .rounded_full()
                                        .bg(dot_color),
                                )
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
                                                    apply_text_step(div(), theme.typography.size_sm)
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
                                    .child(render_icon(
                                        Icon::Check,
                                        px(12.0),
                                        theme.accent.accent,
                                    ))
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

    fn render_namespace_list_body(
        &self,
        theme: &OrbitTheme,
        cx: &mut Context<Self>,
    ) -> AnyElement {
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

    fn render_style_dropdown_overlay(&self, cx: &mut Context<Self>) -> Option<AnyElement> {
        if !self.style_dropdown_open {
            return None;
        }

        let theme = OrbitTheme::get(cx).clone();
        let control_hover = theme.surface.control_hover;
        let mut list = div().flex().flex_col().gap_1().p_1p5();

        for (idx, style) in UiStyle::ALL.iter().copied().enumerate() {
            let is_active = theme.style.ui_style == style;
            let badge = if style.is_translucent() {
                "Blurred"
            } else {
                "Opaque"
            };
            list = list.child(
                div()
                    .id(("ui-style-option", idx))
                    .flex()
                    .items_center()
                    .justify_between()
                    .gap_3()
                    .px_3()
                    .py_2()
                    .rounded(theme.style.radius_control)
                    .cursor_pointer()
                    .when(is_active, |el| el.bg(theme.accent.soft))
                    .hover(move |el| el.bg(control_hover))
                    .on_click(cx.listener(move |this, _event, window, cx| {
                        this.select_ui_style(style, window, cx);
                    }))
                    .child(
                        apply_text_step(div(), theme.typography.size_sm)
                            .font_weight(FontWeight::MEDIUM)
                            .text_color(theme.text.primary)
                            .child(style.label()),
                    )
                    .child(if is_active {
                        div()
                            .flex()
                            .items_center()
                            .gap_1()
                            .child(render_icon(Icon::Check, px(12.0), theme.accent.accent))
                            .child(
                                apply_text_step(div(), theme.typography.size_2xs)
                                    .text_color(theme.accent.accent)
                                    .font_weight(FontWeight::SEMIBOLD)
                                    .child("Active"),
                            )
                            .into_any_element()
                    } else {
                        apply_text_step(div(), theme.typography.size_2xs)
                            .text_color(theme.text.muted)
                            .font_weight(FontWeight::NORMAL)
                            .child(badge)
                            .into_any_element()
                    }),
            );
        }

        Some(
            div()
                .absolute()
                .inset_0()
                .child(
                    div()
                        .id("style-dropdown-backdrop")
                        .absolute()
                        .inset_0()
                        .on_click(cx.listener(|this, _event, _window, cx| {
                            this.style_dropdown_open = false;
                            cx.notify();
                        })),
                )
                .child(
                    theme
                        .style_overlay(div().id("ui-style-dropdown-menu"))
                        .absolute()
                        .top(px(Self::HEADER_HEIGHT + 4.0))
                        .right(px(230.0))
                        .w(px(240.0))
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
                                .child("UI STYLE PREVIEW"),
                        )
                        .child(list),
                )
                .into_any_element(),
        )
    }

    fn render_workspace_view_header(&self, theme: &OrbitTheme) -> AnyElement {
        let view = self.sidebar.active_view;
        let (cat_label, _) = view.breadcrumb();
        let resource_kind = view.resource_kind();

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

    fn render_settings_workspace(&self, theme: &OrbitTheme) -> AnyElement {
        let sidebar_state = if self.sidebar.is_collapsed {
            "Collapsed".to_string()
        } else {
            format!("Expanded ({:.0}px)", self.sidebar.panel_width)
        };

        theme
            .style_card(div())
            .flex()
            .flex_col()
            .gap_3()
            .p_6()
            .w_full()
            .max_w(px(720.0))
            .bg(theme.surface.bg_panel)
            .child(
                apply_text_step(div(), theme.typography.size_base)
                    .font_weight(FontWeight::SEMIBOLD)
                    .child("Appearance & Shell Preferences"),
            )
            .child(views::render_detail_row(
                "Theme Mode",
                theme.mode.label().to_string(),
                false,
                theme,
            ))
            .child(views::render_detail_row(
                "UI Style",
                theme.style.ui_style.label().to_string(),
                false,
                theme,
            ))
            .child(views::render_detail_row(
                "Accent Color",
                format!("{:?}", theme.accent_id),
                false,
                theme,
            ))
            .child(views::render_detail_row(
                "Sidebar Layout",
                sidebar_state,
                true,
                theme,
            ))
            .into_any_element()
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
        let on_navigate =
            std::rc::Rc::new(move |view: WorkspaceView, _window: &mut Window, cx: &mut App| {
                nav_entity.update(cx, |shell, cx| {
                    shell.sidebar.select_view(view);
                    cx.notify();
                });
            });
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

    fn render_workspace(&self, cx: &mut Context<Self>) -> AnyElement {
        let theme = OrbitTheme::get(cx).clone();

        let body = if self.sidebar.active_view == WorkspaceView::Settings {
            self.render_settings_workspace(&theme)
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
                } => self.render_connected_workspace(
                    cluster_id,
                    display_name,
                    k8s_version.as_deref(),
                    &theme,
                    cx,
                ),
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
                                shell.style_dropdown_open = false;
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

        div()
            .id("orbit-workspace-scroll")
            .flex_1()
            .overflow_y_scroll()
            .p_6()
            .bg(theme.surface.bg_app)
            .flex()
            .flex_col()
            .gap_6()
            .child(self.render_workspace_view_header(&theme))
            .child(body)
            .into_any_element()
    }

    fn render_footer(&self, cx: &mut Context<Self>) -> AnyElement {
        let theme = OrbitTheme::get(cx).clone();
        let presentation = format_cluster_status(&self.state, &theme);
        let (cat_label, view_label) = self.sidebar.active_view.breadcrumb();
        let sidebar_status = if self.sidebar.is_collapsed {
            "Sidebar: Collapsed".to_string()
        } else {
            format!("Sidebar: {:.0}px", self.sidebar.panel_width)
        };
        let k8s_version = match &self.state.connection_state {
            ClusterConnectionState::Connected { k8s_version, .. } => k8s_version.clone(),
            _ => None,
        };
        let control_hover = theme.surface.control_hover;
        let is_settings = self.sidebar.active_view == WorkspaceView::Settings;

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
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_3()
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_1()
                            .child(
                                apply_text_step(div(), theme.typography.size_2xs)
                                    .font_weight(FontWeight::MEDIUM)
                                    .text_color(theme.text.secondary)
                                    .child(cat_label),
                            )
                            .child(render_icon(
                                Icon::ChevronRight,
                                px(11.0),
                                theme.text.muted,
                            ))
                            .child(
                                apply_text_step(div(), theme.typography.size_2xs)
                                    .font_weight(FontWeight::MEDIUM)
                                    .text_color(theme.text.secondary)
                                    .child(view_label),
                            ),
                    )
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_1()
                            .child(render_icon(Icon::PanelLeft, px(11.0), theme.text.muted))
                            .child(
                                apply_text_step(div(), theme.typography.size_2xs)
                                    .text_color(theme.text.muted)
                                    .child(sidebar_status),
                            ),
                    ),
            )
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
                    })
                    .child(
                        div()
                            .id("footer-refresh-btn")
                            .flex()
                            .items_center()
                            .justify_center()
                            .w(px(20.0))
                            .h(px(18.0))
                            .rounded(theme.style.radius_control)
                            .cursor_pointer()
                            .hover(move |style| style.bg(control_hover))
                            .on_click(cx.listener(|this, _event, _window, cx| {
                                this.refresh_workspace(cx);
                            }))
                            .child(render_icon(
                                Icon::RefreshCw,
                                px(12.0),
                                theme.text.secondary,
                            )),
                    )
                    .child(
                        div()
                            .id("footer-settings-btn")
                            .flex()
                            .items_center()
                            .justify_center()
                            .w(px(20.0))
                            .h(px(18.0))
                            .rounded(theme.style.radius_control)
                            .cursor_pointer()
                            .when(is_settings, |el| el.bg(theme.accent.soft))
                            .hover(move |style| style.bg(control_hover))
                            .on_click(cx.listener(|this, _event, _window, cx| {
                                this.sidebar.select_view(WorkspaceView::Settings);
                                cx.notify();
                            }))
                            .child(render_icon(
                                Icon::Settings,
                                px(12.0),
                                if is_settings {
                                    theme.accent.accent
                                } else {
                                    theme.text.secondary
                                },
                            )),
                    ),
            )
            .into_any_element()
    }
}

impl Render for OrbitShell {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = OrbitTheme::get(cx).clone();

        div()
            .id("orbit-shell-root")
            .track_focus(&self.focus_handle)
            .on_key_down(cx.listener(|this, event: &KeyDownEvent, _window, cx| {
                if this.namespaces.dropdown_open
                    && this.namespaces.handle_search_key(
                        &event.keystroke.key,
                        event.keystroke.key_char.as_deref(),
                    )
                {
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
            .on_mouse_move(cx.listener(|this, event: &MouseMoveEvent, _window, cx| {
                if this.sidebar.is_resizing {
                    this.sidebar
                        .update_resize_from_window_x(f32::from(event.position.x));
                    cx.notify();
                }
            }))
            .on_mouse_up(
                MouseButton::Left,
                cx.listener(|this, _event, _window, cx| {
                    if this.sidebar.is_resizing {
                        this.sidebar.end_resize();
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
                            .child(self.render_workspace(cx)),
                    ),
            )
            .child(self.render_footer(cx))
            .when_some(self.render_dropdown_overlay(cx), |el, overlay| {
                el.child(overlay)
            })
            .when_some(self.render_namespace_dropdown_overlay(cx), |el, overlay| {
                el.child(overlay)
            })
            .when_some(self.render_style_dropdown_overlay(cx), |el, overlay| {
                el.child(overlay)
            })
    }
}

fn main() {
    let config = OrbitConfig::load();

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
            cx.quit();
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::theme::{AccentColor, K8sResourceKind, SemanticColorsPreference};
    use gpui::WindowBackgroundAppearance;

    #[test]
    fn test_window_options_and_shell_defaults() {
        assert_eq!(OrbitShell::WINDOW_TITLE, "Orbit (GPUI Preview)");
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
            size(px(OrbitShell::DEFAULT_WIDTH), px(OrbitShell::DEFAULT_HEIGHT)),
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
        assert_eq!(title, Some("Orbit (GPUI Preview)"));
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
            size(px(OrbitShell::DEFAULT_WIDTH), px(OrbitShell::DEFAULT_HEIGHT)),
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
        apply_cluster_event_with_workspace(
            &mut switcher,
            &mut connect_sidebar,
            &mut throwaway_overview,
            &mut throwaway_namespaces,
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
            let theme = OrbitTheme::resolve(
                mode,
                UiStyle::Flat,
                AccentColor::Blue,
                SemanticColorsPreference::default(),
                appearance,
            );
            assert_eq!(
                nav_item_icon_color(WorkspaceView::Deployments, false, &theme),
                theme.k8s_color(K8sResourceKind::Deployment)
            );
            assert_eq!(
                nav_item_icon_color(WorkspaceView::Pods, true, &theme),
                theme.k8s_color(K8sResourceKind::Pod)
            );
            assert_eq!(
                nav_item_icon_color(WorkspaceView::Services, false, &theme),
                theme.k8s_color(K8sResourceKind::Service)
            );
            assert_eq!(
                nav_item_icon_color(WorkspaceView::Overview, true, &theme),
                theme.accent.accent
            );
            assert_eq!(
                nav_item_icon_color(WorkspaceView::Overview, false, &theme),
                theme.text.muted
            );
            assert_eq!(
                nav_item_icon_color(WorkspaceView::Logs, false, &theme),
                theme.text.muted
            );
        }
    }

    #[test]
    fn test_header_and_status_icon_mapping() {
        assert_eq!(
            char::from(sidebar_toggle_icon(true)),
            char::from(Icon::PanelLeftOpen)
        );
        assert_eq!(
            char::from(sidebar_toggle_icon(false)),
            char::from(Icon::PanelLeftClose)
        );
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
    fn test_apply_cluster_event_dispatches_overview_and_namespaces_on_connect_and_resets_on_unreachable() {
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
            &mut namespaces,
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
            &mut namespaces,
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
            &mut namespaces,
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
            &mut namespaces,
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
}
