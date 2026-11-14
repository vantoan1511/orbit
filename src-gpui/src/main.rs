mod bridge;
mod state;
pub mod theme;

use gpui::{
    AnyElement, App, Application, Bounds, Context, FontWeight, Rgba, SharedString, Window,
    WindowAppearance, WindowBounds, WindowDecorations, WindowOptions, div, prelude::*, px, size,
};
use orbit_core::config::OrbitConfig;
use orbit_core::kubernetes::models::ClusterInfo;
use tokio::sync::mpsc;

use crate::bridge::{ClusterBridge, start_cluster_bridge};
use crate::state::{ClusterBridgeCommand, ClusterConnectionState, ClusterSwitcherState};
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

fn apply_theme(updated: OrbitTheme, window: &mut Window, cx: &mut App) {
    let bg = updated.style.window_background;
    cx.set_global(updated);
    window.set_background_appearance(bg);
}

pub struct OrbitShell {
    state: ClusterSwitcherState,
    command_tx: mpsc::UnboundedSender<ClusterBridgeCommand>,
    style_dropdown_open: bool,
}

impl OrbitShell {
    pub const WINDOW_TITLE: &'static str = "Orbit (GPUI Preview)";
    pub const DEFAULT_WIDTH: f32 = 1200.0;
    pub const DEFAULT_HEIGHT: f32 = 800.0;

    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let ClusterBridge {
            command_tx,
            mut event_rx,
        } = start_cluster_bridge();

        cx.spawn(async move |this, cx| {
            while let Some(event) = event_rx.recv().await {
                if this
                    .update(cx, |shell, cx| {
                        shell.state.apply_event(event);
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
            command_tx,
            style_dropdown_open: false,
        }
    }

    fn dispatch_command(&self, cmd: ClusterBridgeCommand) {
        if let Err(err) = self.command_tx.send(cmd) {
            tracing::warn!(error = %err, "Failed to send command to cluster bridge");
        }
    }

    fn select_cluster(&mut self, cluster_id: &str, cx: &mut Context<Self>) {
        if let Some(cmd) = self.state.begin_switch(cluster_id) {
            self.dispatch_command(cmd);
        }
        self.style_dropdown_open = false;
        cx.notify();
    }

    fn reconnect_cluster(&mut self, cx: &mut Context<Self>) {
        if let Some(cmd) = self.state.begin_reconnect() {
            self.dispatch_command(cmd);
        }
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

    fn render_header(&self, cx: &mut Context<Self>) -> AnyElement {
        let theme = OrbitTheme::get(cx).clone();
        let presentation = format_cluster_status(&self.state, &theme);
        let cluster_chevron = if self.state.dropdown_open { "^" } else { "v" };
        let style_chevron = if self.style_dropdown_open { "^" } else { "v" };
        let control_hover = theme.surface.control_hover;

        div()
            .flex()
            .items_center()
            .justify_between()
            .gap_4()
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_4()
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .child(
                                apply_text_step(div(), theme.typography.size_lg)
                                    .font_weight(FontWeight::BOLD)
                                    .child("Orbit"),
                            )
                            .child(
                                apply_text_step(div(), theme.typography.size_xs)
                                    .text_color(theme.text.muted)
                                    .child("GPUI Preview"),
                            ),
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
                            .child(
                                apply_text_step(div(), theme.typography.size_xs)
                                    .text_color(theme.text.secondary)
                                    .child(cluster_chevron),
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
                            .child(
                                apply_text_step(div(), theme.typography.size_xs)
                                    .text_color(theme.text.secondary)
                                    .child(style_chevron),
                            ),
                    )
                    .child(
                        theme
                            .style_control(div().id("refresh-clusters-btn"))
                            .flex()
                            .items_center()
                            .h(theme.typography.control_height_sm)
                            .px(theme.typography.control_padding_x_md)
                            .hover(move |style| style.bg(control_hover))
                            .cursor_pointer()
                            .on_click(cx.listener(|this, _event, _window, cx| {
                                this.dispatch_command(ClusterBridgeCommand::RefreshClusters);
                                cx.notify();
                            }))
                            .child(
                                apply_text_step(div(), theme.typography.size_sm)
                                    .font_weight(FontWeight::MEDIUM)
                                    .child("Refresh"),
                            ),
                    )
                    .child(
                        theme
                            .style_control(div().id("switch-to-old-ui"))
                            .flex()
                            .items_center()
                            .h(theme.typography.control_height_sm)
                            .px(theme.typography.control_padding_x_md)
                            .hover(move |style| style.bg(control_hover))
                            .cursor_pointer()
                            .on_click(cx.listener(|_this, _event, _window, cx| {
                                cx.quit();
                            }))
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
                            apply_text_step(div(), theme.typography.size_xs)
                                .font_weight(FontWeight::SEMIBOLD)
                                .text_color(theme.semantic.danger.solid)
                                .child("KUBECONFIG ERROR"),
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
                let dot_color = parse_hex_color(cluster.color.as_deref()).unwrap_or_else(|| {
                    if cluster.status == "healthy" {
                        theme.semantic.success.solid
                    } else {
                        theme.text.muted
                    }
                });

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
                                apply_text_step(div(), theme.typography.size_xs)
                                    .font_weight(FontWeight::SEMIBOLD)
                                    .text_color(theme.accent.accent)
                                    .child("Active"),
                            )
                        }),
                );
            }
        }

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
                        .top(px(76.0))
                        .left(px(108.0))
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
                    .child(
                        apply_text_step(div(), theme.typography.size_2xs)
                            .text_color(if is_active {
                                theme.accent.accent
                            } else {
                                theme.text.muted
                            })
                            .font_weight(if is_active {
                                FontWeight::SEMIBOLD
                            } else {
                                FontWeight::NORMAL
                            })
                            .child(if is_active { "Active" } else { badge }),
                    ),
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
                        .top(px(76.0))
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

    fn render_detail_row(
        label: &'static str,
        value: String,
        mono: bool,
        theme: &OrbitTheme,
    ) -> AnyElement {
        let mut value_el = apply_text_step(div(), theme.typography.size_xs)
            .font_weight(FontWeight::MEDIUM)
            .text_color(theme.text.primary);
        if mono {
            value_el = value_el.font(theme.typography.mono_font(FontWeight::MEDIUM));
        }

        div()
            .flex()
            .items_center()
            .justify_between()
            .gap_4()
            .py_1p5()
            .border_b_1()
            .border_color(theme.border.default)
            .child(
                apply_text_step(div(), theme.typography.size_xs)
                    .text_color(theme.text.muted)
                    .child(label),
            )
            .child(value_el.child(value))
            .into_any_element()
    }

    fn render_workspace(&self, cx: &mut Context<Self>) -> AnyElement {
        let theme = OrbitTheme::get(cx).clone();
        let control_hover = theme.surface.control_hover;
        let mono_font = theme.typography.mono_font(FontWeight::NORMAL);

        match &self.state.connection_state {
            ClusterConnectionState::Disconnected => {
                let mut container = div()
                    .flex()
                    .flex_col()
                    .gap_4()
                    .p_6()
                    .w_full()
                    .max_w(px(720.0));

                container = container
                    .child(
                        apply_text_step(div(), theme.typography.size_base)
                            .font_weight(FontWeight::SEMIBOLD)
                            .child("Select a Kubernetes Cluster"),
                    )
                    .child(
                        apply_text_step(div(), theme.typography.size_sm)
                            .text_color(theme.text.secondary)
                            .child(
                                "Contexts are loaded in a disconnected state. Choose a cluster from the header switcher or below to connect.",
                            ),
                    );

                if self.state.clusters.is_empty() {
                    container = container.child(
                        apply_text_step(div(), theme.typography.size_xs)
                            .p_4()
                            .rounded(theme.style.radius_control)
                            .bg(theme.surface.bg_app)
                            .border_1()
                            .border_color(theme.border.default)
                            .text_color(theme.text.muted)
                            .child("No kubeconfig contexts found. Add a kubeconfig in ~/.kube/config or via Orbit settings."),
                    );
                } else {
                    let mut list = div().flex().flex_col().gap_2();
                    for (idx, cluster) in self.state.clusters.iter().enumerate() {
                        let cluster_id = cluster.id.clone();
                        let display = cluster_display_label(cluster);
                        let dot_color = parse_hex_color(cluster.color.as_deref())
                            .unwrap_or(theme.text.muted);

                        list = list.child(
                            div()
                                .id(("workspace-cluster-row", idx))
                                .flex()
                                .items_center()
                                .justify_between()
                                .px_4()
                                .py_2p5()
                                .rounded(theme.style.radius_control)
                                .bg(theme.surface.bg_app)
                                .hover(move |style| style.bg(control_hover))
                                .border_1()
                                .border_color(theme.border.default)
                                .cursor_pointer()
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
                                            apply_text_step(div(), theme.typography.size_sm)
                                                .font_weight(FontWeight::MEDIUM)
                                                .child(display),
                                        ),
                                )
                                .when_some(cluster.server_url.clone(), |el, url| {
                                    el.child(
                                        apply_text_step(div(), theme.typography.size_xs)
                                            .font(mono_font.clone())
                                            .text_color(theme.text.muted)
                                            .child(url),
                                    )
                                }),
                        );
                    }
                    container = container.child(list);
                }

                theme
                    .style_card(div())
                    .flex_1()
                    .flex()
                    .items_center()
                    .justify_center()
                    .bg(theme.surface.bg_panel)
                    .child(container)
                    .into_any_element()
            }
            ClusterConnectionState::Connecting {
                cluster_id,
                display_name,
            } => theme
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
                .into_any_element(),
            ClusterConnectionState::Connected {
                cluster_id,
                display_name,
                k8s_version,
            } => {
                let profile = self.state.active_profile.as_ref();
                let active_cluster = self.state.active_cluster();
                let server_url = profile
                    .and_then(|p| p.server_url.clone())
                    .or_else(|| active_cluster.and_then(|c| c.server_url.clone()))
                    .unwrap_or_else(|| "Unknown".to_string());
                let user_name = profile
                    .and_then(|p| p.user_name.clone())
                    .unwrap_or_else(|| "None".to_string());
                let auth_type = profile
                    .map(|p| p.auth_type.clone())
                    .unwrap_or_else(|| "None".to_string());
                let kubeconfigs = profile
                    .map(|p| p.kubeconfig_paths.join(", "))
                    .filter(|s| !s.is_empty())
                    .unwrap_or_else(|| "None".to_string());
                let version_label = k8s_version
                    .clone()
                    .unwrap_or_else(|| "Connected".to_string());
                let description = active_cluster.and_then(|c| c.description.clone());
                let tags = active_cluster
                    .map(|c| c.tags.join(", "))
                    .filter(|s| !s.is_empty());

                let mut card = div()
                    .flex()
                    .flex_col()
                    .gap_4()
                    .p_6()
                    .w_full()
                    .max_w(px(720.0))
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .justify_between()
                            .child(
                                div()
                                    .flex()
                                    .flex_col()
                                    .gap_1()
                                    .child(
                                        apply_text_step(div(), theme.typography.size_lg)
                                            .font_weight(FontWeight::BOLD)
                                            .child(display_name.clone()),
                                    )
                                    .when_some(description, |el, desc| {
                                        el.child(
                                            apply_text_step(div(), theme.typography.size_xs)
                                                .text_color(theme.text.secondary)
                                                .child(desc),
                                        )
                                    }),
                            )
                            .child(
                                apply_text_step(div(), theme.typography.size_xs)
                                    .px_2p5()
                                    .py_1()
                                    .rounded(theme.style.radius_control)
                                    .bg(theme.semantic.success.soft)
                                    .font_weight(FontWeight::SEMIBOLD)
                                    .text_color(theme.semantic.success.solid)
                                    .child(format!("Connected • {version_label}")),
                            ),
                    )
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .child(Self::render_detail_row(
                                "Context",
                                cluster_id.clone(),
                                true,
                                &theme,
                            ))
                            .child(Self::render_detail_row(
                                "Kubernetes Version",
                                version_label,
                                true,
                                &theme,
                            ))
                            .child(Self::render_detail_row(
                                "API Server",
                                server_url,
                                true,
                                &theme,
                            ))
                            .child(Self::render_detail_row(
                                "User",
                                user_name,
                                false,
                                &theme,
                            ))
                            .child(Self::render_detail_row(
                                "Auth Type",
                                auth_type,
                                false,
                                &theme,
                            ))
                            .child(Self::render_detail_row(
                                "Kubeconfig Paths",
                                kubeconfigs,
                                true,
                                &theme,
                            )),
                    );

                if let Some(tag_str) = tags {
                    card = card.child(Self::render_detail_row("Tags", tag_str, false, &theme));
                }

                theme
                    .style_card(div())
                    .flex_1()
                    .flex()
                    .items_center()
                    .justify_center()
                    .bg(theme.surface.bg_panel)
                    .child(card)
                    .into_any_element()
            }
            ClusterConnectionState::Unreachable {
                cluster_id,
                display_name,
                server_url,
                error_message,
            } => self.render_error_card(
                "CLUSTER UNREACHABLE",
                display_name.clone(),
                Some(cluster_id.clone()),
                server_url.clone(),
                error_message.clone(),
                true,
                cx,
            ),
            ClusterConnectionState::Error {
                cluster_id,
                display_name,
                error_message,
            } => {
                let title = display_name
                    .clone()
                    .or_else(|| cluster_id.clone())
                    .unwrap_or_else(|| "Cluster Connection Failed".to_string());
                self.render_error_card(
                    "CONNECTION ERROR",
                    title,
                    cluster_id.clone(),
                    None,
                    error_message.clone(),
                    cluster_id.is_some(),
                    cx,
                )
            }
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn render_error_card(
        &self,
        badge: &'static str,
        title: String,
        cluster_id: Option<String>,
        server_url: Option<String>,
        error_message: String,
        can_retry: bool,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let theme = OrbitTheme::get(cx).clone();
        let control_hover = theme.surface.control_hover;
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
                    .gap_4()
                    .p_6()
                    .w_full()
                    .max_w(px(640.0))
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_2()
                            .child(
                                apply_text_step(div(), theme.typography.size_xs)
                                    .px_2p5()
                                    .py_1()
                                    .rounded(theme.style.radius_control)
                                    .bg(theme.semantic.danger.soft)
                                    .font_weight(FontWeight::SEMIBOLD)
                                    .text_color(theme.semantic.danger.solid)
                                    .child(badge),
                            ),
                    )
                    .child(
                        apply_text_step(div(), theme.typography.size_lg)
                            .font_weight(FontWeight::BOLD)
                            .child(title),
                    )
                    .when_some(cluster_id, |el, id| {
                        el.child(
                            apply_text_step(div(), theme.typography.size_xs)
                                .font(mono_font.clone())
                                .text_color(theme.text.muted)
                                .child(format!("Context: {id}")),
                        )
                    })
                    .when_some(server_url, |el, url| {
                        el.child(
                            apply_text_step(div(), theme.typography.size_xs)
                                .font(mono_font.clone())
                                .text_color(theme.text.secondary)
                                .child(format!("Server: {url}")),
                        )
                    })
                    .child(
                        apply_text_step(div(), theme.typography.size_xs)
                            .p_3()
                            .rounded(theme.style.radius_control)
                            .bg(theme.semantic.danger.soft)
                            .border_1()
                            .border_color(theme.semantic.danger.solid)
                            .text_color(theme.text.primary)
                            .child(error_message),
                    )
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_3()
                            .when(can_retry, |el| {
                                el.child(
                                    theme
                                        .style_control(div().id("retry-cluster-connection-btn"))
                                        .flex()
                                        .items_center()
                                        .h(theme.typography.control_height_sm)
                                        .px(theme.typography.control_padding_x_md)
                                        .hover(move |style| style.bg(control_hover))
                                        .cursor_pointer()
                                        .on_click(cx.listener(|this, _event, _window, cx| {
                                            this.reconnect_cluster(cx);
                                        }))
                                        .child(
                                            apply_text_step(div(), theme.typography.size_sm)
                                                .font_weight(FontWeight::MEDIUM)
                                                .child("Retry / Reconnect"),
                                        ),
                                )
                            })
                            .child(
                                theme
                                    .style_control(div().id("open-cluster-switcher-btn"))
                                    .flex()
                                    .items_center()
                                    .h(theme.typography.control_height_sm)
                                    .px(theme.typography.control_padding_x_md)
                                    .hover(move |style| style.bg(control_hover))
                                    .cursor_pointer()
                                    .on_click(cx.listener(|this, _event, _window, cx| {
                                        this.style_dropdown_open = false;
                                        this.state.dropdown_open = true;
                                        cx.notify();
                                    }))
                                    .child(
                                        apply_text_step(div(), theme.typography.size_sm)
                                            .font_weight(FontWeight::MEDIUM)
                                            .child("Choose Another Cluster"),
                                    ),
                            ),
                    ),
            )
            .into_any_element()
    }
}

impl Render for OrbitShell {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = OrbitTheme::get(cx).clone();

        div()
            .relative()
            .flex()
            .flex_col()
            .size_full()
            .font(theme.typography.ui_font(FontWeight::NORMAL))
            .bg(theme.surface.bg_app)
            .text_color(theme.text.primary)
            .p_8()
            .gap_6()
            .child(self.render_header(cx))
            .when_some(self.render_error_banner(cx), |el, banner| el.child(banner))
            .child(self.render_workspace(cx))
            .when_some(self.render_dropdown_overlay(cx), |el, overlay| {
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
            cx.new(|cx| OrbitShell::new(window, cx))
        }) {
            eprintln!("Failed to open Orbit GPUI window: {err}");
            cx.quit();
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::theme::{AccentColor, SemanticColorsPreference};
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
}
