mod bridge;
mod state;

use gpui::{
    AnyElement, App, Application, Bounds, Context, SharedString, Window, WindowBounds,
    WindowDecorations, WindowOptions, div, prelude::*, px, rgb, size,
};
use orbit_core::kubernetes::models::ClusterInfo;
use tokio::sync::mpsc;

use crate::bridge::{ClusterBridge, start_cluster_bridge};
use crate::state::{ClusterBridgeCommand, ClusterConnectionState, ClusterSwitcherState};

mod theme {
    pub const BG_APP: u32 = 0x0b0f17;
    pub const BG_PANEL: u32 = 0x0f172a;
    pub const BG_CONTROL: u32 = 0x1e293b;
    pub const BG_CONTROL_HOVER: u32 = 0x334155;
    pub const BORDER: u32 = 0x1e293b;
    pub const BORDER_CONTROL: u32 = 0x334155;
    pub const TEXT_PRIMARY: u32 = 0xe2e8f0;
    pub const TEXT_SECONDARY: u32 = 0x94a3b8;
    pub const TEXT_MUTED: u32 = 0x64748b;

    pub const STATUS_SUCCESS: u32 = 0x22c55e;
    pub const STATUS_WARNING: u32 = 0xeab308;
    pub const STATUS_DANGER: u32 = 0xef4444;
    pub const STATUS_DANGER_SOFT: u32 = 0x2a1215;
    pub const STATUS_WARNING_SOFT: u32 = 0x2c220a;
    pub const ACCENT: u32 = 0x6aa8ff;
    pub const ACCENT_SOFT: u32 = 0x14223b;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StatusPresentation {
    pub status_label: &'static str,
    pub color: u32,
    pub pill_label: String,
}

pub fn format_cluster_status(state: &ClusterSwitcherState) -> StatusPresentation {
    match &state.connection_state {
        ClusterConnectionState::Disconnected => StatusPresentation {
            status_label: "Disconnected",
            color: theme::TEXT_MUTED,
            pill_label: "Select Cluster".to_string(),
        },
        ClusterConnectionState::Connecting { display_name, .. } => StatusPresentation {
            status_label: "Connecting",
            color: theme::STATUS_WARNING,
            pill_label: display_name.clone(),
        },
        ClusterConnectionState::Connected { display_name, .. } => StatusPresentation {
            status_label: "Connected",
            color: theme::STATUS_SUCCESS,
            pill_label: display_name.clone(),
        },
        ClusterConnectionState::Unreachable { display_name, .. } => StatusPresentation {
            status_label: "Unreachable",
            color: theme::STATUS_DANGER,
            pill_label: display_name.clone(),
        },
        ClusterConnectionState::Error { display_name, .. } => StatusPresentation {
            status_label: "Error",
            color: theme::STATUS_DANGER,
            pill_label: display_name
                .clone()
                .unwrap_or_else(|| "Select Cluster".to_string()),
        },
    }
}

fn parse_hex_color(hex: Option<&str>) -> Option<u32> {
    let raw = hex?.trim().strip_prefix('#').unwrap_or_else(|| hex.unwrap_or("").trim());
    if raw.len() == 6 {
        u32::from_str_radix(raw, 16).ok()
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

pub struct OrbitShell {
    state: ClusterSwitcherState,
    command_tx: mpsc::UnboundedSender<ClusterBridgeCommand>,
}

impl OrbitShell {
    pub const WINDOW_TITLE: &'static str = "Orbit (GPUI Preview)";
    pub const DEFAULT_WIDTH: f32 = 1200.0;
    pub const DEFAULT_HEIGHT: f32 = 800.0;

    pub fn new(cx: &mut Context<Self>) -> Self {
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

        Self {
            state: ClusterSwitcherState::new(),
            command_tx,
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
        cx.notify();
    }

    fn reconnect_cluster(&mut self, cx: &mut Context<Self>) {
        if let Some(cmd) = self.state.begin_reconnect() {
            self.dispatch_command(cmd);
        }
        cx.notify();
    }

    pub fn default_window_bounds(cx: &App) -> Bounds<gpui::Pixels> {
        Bounds::centered(
            None,
            size(px(Self::DEFAULT_WIDTH), px(Self::DEFAULT_HEIGHT)),
            cx,
        )
    }

    pub fn default_window_options(bounds: Bounds<gpui::Pixels>) -> WindowOptions {
        WindowOptions {
            window_bounds: Some(WindowBounds::Windowed(bounds)),
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
        let presentation = format_cluster_status(&self.state);
        let chevron = if self.state.dropdown_open { "^" } else { "v" };

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
                                div()
                                    .text_lg()
                                    .font_weight(gpui::FontWeight::BOLD)
                                    .child("Orbit"),
                            )
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(rgb(theme::TEXT_MUTED))
                                    .child("GPUI Preview"),
                            ),
                    )
                    .child(
                        div()
                            .id("cluster-switcher-btn")
                            .flex()
                            .items_center()
                            .gap_2p5()
                            .px_3()
                            .py_1p5()
                            .rounded_md()
                            .bg(rgb(theme::BG_CONTROL))
                            .hover(|style| style.bg(rgb(theme::BG_CONTROL_HOVER)))
                            .border_1()
                            .border_color(rgb(theme::BORDER_CONTROL))
                            .cursor_pointer()
                            .on_click(cx.listener(|this, _event, _window, cx| {
                                this.state.toggle_dropdown();
                                cx.notify();
                            }))
                            .child(
                                div()
                                    .w(px(8.0))
                                    .h(px(8.0))
                                    .rounded_full()
                                    .bg(rgb(presentation.color)),
                            )
                            .child(
                                div()
                                    .text_sm()
                                    .font_weight(gpui::FontWeight::MEDIUM)
                                    .child(presentation.pill_label),
                            )
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(rgb(theme::TEXT_MUTED))
                                    .child(presentation.status_label),
                            )
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(rgb(theme::TEXT_SECONDARY))
                                    .child(chevron),
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
                            .id("refresh-clusters-btn")
                            .px_3()
                            .py_1p5()
                            .rounded_md()
                            .bg(rgb(theme::BG_CONTROL))
                            .hover(|style| style.bg(rgb(theme::BG_CONTROL_HOVER)))
                            .border_1()
                            .border_color(rgb(theme::BORDER_CONTROL))
                            .cursor_pointer()
                            .text_sm()
                            .font_weight(gpui::FontWeight::MEDIUM)
                            .child("Refresh")
                            .on_click(cx.listener(|this, _event, _window, cx| {
                                this.dispatch_command(ClusterBridgeCommand::RefreshClusters);
                                cx.notify();
                            })),
                    )
                    .child(
                        div()
                            .id("switch-to-old-ui")
                            .px_3()
                            .py_1p5()
                            .rounded_md()
                            .bg(rgb(theme::BG_CONTROL))
                            .hover(|style| style.bg(rgb(theme::BG_CONTROL_HOVER)))
                            .border_1()
                            .border_color(rgb(theme::BORDER_CONTROL))
                            .cursor_pointer()
                            .text_sm()
                            .font_weight(gpui::FontWeight::MEDIUM)
                            .child("Switch to Old UI")
                            .on_click(cx.listener(|_this, _event, _window, cx| {
                                cx.quit();
                            })),
                    ),
            )
            .into_any_element()
    }

    fn render_error_banner(&self, cx: &mut Context<Self>) -> Option<AnyElement> {
        if self.state.error_banner_dismissed {
            return None;
        }
        let message = self.state.kubeconfig_error.as_ref()?.clone();

        Some(
            div()
                .flex()
                .items_center()
                .justify_between()
                .gap_3()
                .px_4()
                .py_2p5()
                .rounded_md()
                .bg(rgb(theme::STATUS_DANGER_SOFT))
                .border_1()
                .border_color(rgb(theme::STATUS_DANGER))
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap_2()
                        .child(
                            div()
                                .text_xs()
                                .font_weight(gpui::FontWeight::SEMIBOLD)
                                .text_color(rgb(theme::STATUS_DANGER))
                                .child("KUBECONFIG ERROR"),
                        )
                        .child(
                            div()
                                .text_xs()
                                .text_color(rgb(theme::TEXT_PRIMARY))
                                .child(message),
                        ),
                )
                .child(
                    div()
                        .id("dismiss-error-banner-btn")
                        .px_2()
                        .py_1()
                        .rounded_sm()
                        .cursor_pointer()
                        .text_xs()
                        .text_color(rgb(theme::TEXT_SECONDARY))
                        .hover(|style| style.bg(rgb(theme::BG_CONTROL_HOVER)))
                        .child("Dismiss")
                        .on_click(cx.listener(|this, _event, _window, cx| {
                            this.state.dismiss_error_banner();
                            cx.notify();
                        })),
                )
                .into_any_element(),
        )
    }

    fn render_dropdown_overlay(&self, cx: &mut Context<Self>) -> Option<AnyElement> {
        if !self.state.dropdown_open {
            return None;
        }

        let mut list = div().flex().flex_col().gap_1().p_1p5();

        if self.state.clusters.is_empty() {
            list = list.child(
                div()
                    .px_3()
                    .py_4()
                    .text_xs()
                    .text_color(rgb(theme::TEXT_MUTED))
                    .child("No kubeconfig contexts found in ~/.kube/config or custom kubeconfig paths."),
            );
        } else {
            for (idx, cluster) in self.state.clusters.iter().enumerate() {
                let cluster_id = cluster.id.clone();
                let is_active = self.state.active_cluster_id.as_deref() == Some(cluster_id.as_str());
                let display = cluster_display_label(cluster);
                let show_raw = display != cluster.name;
                let dot_color = parse_hex_color(cluster.color.as_deref()).unwrap_or_else(|| {
                    if cluster.status == "healthy" {
                        theme::STATUS_SUCCESS
                    } else {
                        theme::TEXT_MUTED
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
                        .rounded_md()
                        .cursor_pointer()
                        .when(is_active, |el| el.bg(rgb(theme::ACCENT_SOFT)))
                        .hover(|style| style.bg(rgb(theme::BG_CONTROL_HOVER)))
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
                                        .bg(rgb(dot_color)),
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
                                                    div()
                                                        .text_sm()
                                                        .font_weight(gpui::FontWeight::MEDIUM)
                                                        .child(display),
                                                )
                                                .when(show_raw, |el| {
                                                    el.child(
                                                        div()
                                                            .text_xs()
                                                            .text_color(rgb(theme::TEXT_MUTED))
                                                            .child(cluster.name.clone()),
                                                    )
                                                }),
                                        )
                                        .when_some(cluster.server_url.clone(), |el, url| {
                                            el.child(
                                                div()
                                                    .text_xs()
                                                    .text_color(rgb(theme::TEXT_SECONDARY))
                                                    .child(url),
                                            )
                                        }),
                                ),
                        )
                        .when(is_active, |el| {
                            el.child(
                                div()
                                    .text_xs()
                                    .font_weight(gpui::FontWeight::SEMIBOLD)
                                    .text_color(rgb(theme::ACCENT))
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
                    div()
                        .id("cluster-dropdown-menu")
                        .absolute()
                        .top(px(76.0))
                        .left(px(108.0))
                        .w(px(400.0))
                        .max_h(px(420.0))
                        .overflow_y_scroll()
                        .rounded_lg()
                        .bg(rgb(theme::BG_PANEL))
                        .border_1()
                        .border_color(rgb(theme::BORDER_CONTROL))
                        .on_click(|_event, _window, cx| {
                            cx.stop_propagation();
                        })
                        .child(
                            div()
                                .px_3()
                                .py_2()
                                .border_b_1()
                                .border_color(rgb(theme::BORDER))
                                .text_xs()
                                .font_weight(gpui::FontWeight::SEMIBOLD)
                                .text_color(rgb(theme::TEXT_MUTED))
                                .child(format!("KUBECONFIG CONTEXTS ({})", self.state.clusters.len())),
                        )
                        .child(list),
                )
                .into_any_element(),
        )
    }

    fn render_detail_row(label: &'static str, value: String) -> AnyElement {
        div()
            .flex()
            .items_center()
            .justify_between()
            .gap_4()
            .py_1p5()
            .border_b_1()
            .border_color(rgb(theme::BORDER))
            .child(
                div()
                    .text_xs()
                    .text_color(rgb(theme::TEXT_MUTED))
                    .child(label),
            )
            .child(
                div()
                    .text_xs()
                    .font_weight(gpui::FontWeight::MEDIUM)
                    .text_color(rgb(theme::TEXT_PRIMARY))
                    .child(value),
            )
            .into_any_element()
    }

    fn render_workspace(&self, cx: &mut Context<Self>) -> AnyElement {
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
                        div()
                            .text_base()
                            .font_weight(gpui::FontWeight::SEMIBOLD)
                            .child("Select a Kubernetes Cluster"),
                    )
                    .child(
                        div()
                            .text_sm()
                            .text_color(rgb(theme::TEXT_SECONDARY))
                            .child(
                                "Contexts are loaded in a disconnected state. Choose a cluster from the header switcher or below to connect.",
                            ),
                    );

                if self.state.clusters.is_empty() {
                    container = container.child(
                        div()
                            .p_4()
                            .rounded_md()
                            .bg(rgb(theme::BG_APP))
                            .border_1()
                            .border_color(rgb(theme::BORDER))
                            .text_xs()
                            .text_color(rgb(theme::TEXT_MUTED))
                            .child("No kubeconfig contexts found. Add a kubeconfig in ~/.kube/config or via Orbit settings."),
                    );
                } else {
                    let mut list = div().flex().flex_col().gap_2();
                    for (idx, cluster) in self.state.clusters.iter().enumerate() {
                        let cluster_id = cluster.id.clone();
                        let display = cluster_display_label(cluster);
                        let dot_color =
                            parse_hex_color(cluster.color.as_deref()).unwrap_or(theme::TEXT_MUTED);

                        list = list.child(
                            div()
                                .id(("workspace-cluster-row", idx))
                                .flex()
                                .items_center()
                                .justify_between()
                                .px_4()
                                .py_2p5()
                                .rounded_md()
                                .bg(rgb(theme::BG_APP))
                                .hover(|style| style.bg(rgb(theme::BG_CONTROL)))
                                .border_1()
                                .border_color(rgb(theme::BORDER))
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
                                                .bg(rgb(dot_color)),
                                        )
                                        .child(
                                            div()
                                                .text_sm()
                                                .font_weight(gpui::FontWeight::MEDIUM)
                                                .child(display),
                                        ),
                                )
                                .when_some(cluster.server_url.clone(), |el, url| {
                                    el.child(
                                        div()
                                            .text_xs()
                                            .text_color(rgb(theme::TEXT_MUTED))
                                            .child(url),
                                    )
                                }),
                        );
                    }
                    container = container.child(list);
                }

                div()
                    .flex_1()
                    .flex()
                    .items_center()
                    .justify_center()
                    .rounded_lg()
                    .border_1()
                    .border_color(rgb(theme::BORDER))
                    .bg(rgb(theme::BG_PANEL))
                    .child(container)
                    .into_any_element()
            }
            ClusterConnectionState::Connecting {
                cluster_id,
                display_name,
            } => div()
                .flex_1()
                .flex()
                .items_center()
                .justify_center()
                .rounded_lg()
                .border_1()
                .border_color(rgb(theme::BORDER))
                .bg(rgb(theme::BG_PANEL))
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .items_center()
                        .gap_3()
                        .p_6()
                        .child(
                            div()
                                .px_3()
                                .py_1()
                                .rounded_md()
                                .bg(rgb(theme::STATUS_WARNING_SOFT))
                                .text_xs()
                                .font_weight(gpui::FontWeight::SEMIBOLD)
                                .text_color(rgb(theme::STATUS_WARNING))
                                .child("CONNECTING"),
                        )
                        .child(
                            div()
                                .text_base()
                                .font_weight(gpui::FontWeight::SEMIBOLD)
                                .child(format!("Connecting to {display_name}...")),
                        )
                        .child(
                            div()
                                .text_xs()
                                .text_color(rgb(theme::TEXT_MUTED))
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
                                        div()
                                            .text_lg()
                                            .font_weight(gpui::FontWeight::BOLD)
                                            .child(display_name.clone()),
                                    )
                                    .when_some(description, |el, desc| {
                                        el.child(
                                            div()
                                                .text_xs()
                                                .text_color(rgb(theme::TEXT_SECONDARY))
                                                .child(desc),
                                        )
                                    }),
                            )
                            .child(
                                div()
                                    .px_2p5()
                                    .py_1()
                                    .rounded_md()
                                    .bg(rgb(theme::BG_CONTROL))
                                    .text_xs()
                                    .font_weight(gpui::FontWeight::SEMIBOLD)
                                    .text_color(rgb(theme::STATUS_SUCCESS))
                                    .child(format!("Connected • {version_label}")),
                            ),
                    )
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .child(Self::render_detail_row("Context", cluster_id.clone()))
                            .child(Self::render_detail_row("Kubernetes Version", version_label))
                            .child(Self::render_detail_row("API Server", server_url))
                            .child(Self::render_detail_row("User", user_name))
                            .child(Self::render_detail_row("Auth Type", auth_type))
                            .child(Self::render_detail_row("Kubeconfig Paths", kubeconfigs)),
                    );

                if let Some(tag_str) = tags {
                    card = card.child(Self::render_detail_row("Tags", tag_str));
                }

                div()
                    .flex_1()
                    .flex()
                    .items_center()
                    .justify_center()
                    .rounded_lg()
                    .border_1()
                    .border_color(rgb(theme::BORDER))
                    .bg(rgb(theme::BG_PANEL))
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
        div()
            .flex_1()
            .flex()
            .items_center()
            .justify_center()
            .rounded_lg()
            .border_1()
            .border_color(rgb(theme::BORDER))
            .bg(rgb(theme::BG_PANEL))
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
                                div()
                                    .px_2p5()
                                    .py_1()
                                    .rounded_md()
                                    .bg(rgb(theme::STATUS_DANGER_SOFT))
                                    .text_xs()
                                    .font_weight(gpui::FontWeight::SEMIBOLD)
                                    .text_color(rgb(theme::STATUS_DANGER))
                                    .child(badge),
                            ),
                    )
                    .child(
                        div()
                            .text_lg()
                            .font_weight(gpui::FontWeight::BOLD)
                            .child(title),
                    )
                    .when_some(cluster_id, |el, id| {
                        el.child(
                            div()
                                .text_xs()
                                .text_color(rgb(theme::TEXT_MUTED))
                                .child(format!("Context: {id}")),
                        )
                    })
                    .when_some(server_url, |el, url| {
                        el.child(
                            div()
                                .text_xs()
                                .text_color(rgb(theme::TEXT_SECONDARY))
                                .child(format!("Server: {url}")),
                        )
                    })
                    .child(
                        div()
                            .p_3()
                            .rounded_md()
                            .bg(rgb(theme::STATUS_DANGER_SOFT))
                            .border_1()
                            .border_color(rgb(theme::STATUS_DANGER))
                            .text_xs()
                            .text_color(rgb(theme::TEXT_PRIMARY))
                            .child(error_message),
                    )
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_3()
                            .when(can_retry, |el| {
                                el.child(
                                    div()
                                        .id("retry-cluster-connection-btn")
                                        .px_3()
                                        .py_1p5()
                                        .rounded_md()
                                        .bg(rgb(theme::BG_CONTROL))
                                        .hover(|style| style.bg(rgb(theme::BG_CONTROL_HOVER)))
                                        .border_1()
                                        .border_color(rgb(theme::BORDER_CONTROL))
                                        .cursor_pointer()
                                        .text_sm()
                                        .font_weight(gpui::FontWeight::MEDIUM)
                                        .child("Retry / Reconnect")
                                        .on_click(cx.listener(|this, _event, _window, cx| {
                                            this.reconnect_cluster(cx);
                                        })),
                                )
                            })
                            .child(
                                div()
                                    .id("open-cluster-switcher-btn")
                                    .px_3()
                                    .py_1p5()
                                    .rounded_md()
                                    .bg(rgb(theme::BG_CONTROL))
                                    .hover(|style| style.bg(rgb(theme::BG_CONTROL_HOVER)))
                                    .border_1()
                                    .border_color(rgb(theme::BORDER_CONTROL))
                                    .cursor_pointer()
                                    .text_sm()
                                    .font_weight(gpui::FontWeight::MEDIUM)
                                    .child("Choose Another Cluster")
                                    .on_click(cx.listener(|this, _event, _window, cx| {
                                        this.state.dropdown_open = true;
                                        cx.notify();
                                    })),
                            ),
                    ),
            )
            .into_any_element()
    }
}

impl Render for OrbitShell {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .relative()
            .flex()
            .flex_col()
            .size_full()
            .bg(rgb(theme::BG_APP))
            .text_color(rgb(theme::TEXT_PRIMARY))
            .p_8()
            .gap_6()
            .child(self.render_header(cx))
            .when_some(self.render_error_banner(cx), |el, banner| el.child(banner))
            .child(self.render_workspace(cx))
            .when_some(self.render_dropdown_overlay(cx), |el, overlay| {
                el.child(overlay)
            })
    }
}

fn main() {
    Application::new().run(|cx: &mut App| {
        cx.on_window_closed(|cx| {
            if cx.windows().is_empty() {
                cx.quit();
            }
        })
        .detach();

        let bounds = OrbitShell::default_window_bounds(cx);
        let options = OrbitShell::default_window_options(bounds);
        if let Err(err) = cx.open_window(options, |_, cx| cx.new(OrbitShell::new)) {
            eprintln!("Failed to open Orbit GPUI window: {err}");
            cx.quit();
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_window_options_and_shell_defaults() {
        assert_eq!(OrbitShell::WINDOW_TITLE, "Orbit (GPUI Preview)");
        const {
            assert!(OrbitShell::DEFAULT_WIDTH >= 800.0);
            assert!(OrbitShell::DEFAULT_HEIGHT >= 600.0);
        }

        let bounds = Bounds::new(
            gpui::point(px(0.0), px(0.0)),
            size(px(OrbitShell::DEFAULT_WIDTH), px(OrbitShell::DEFAULT_HEIGHT)),
        );
        let opts = OrbitShell::default_window_options(bounds);
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
    fn test_status_indicator_and_header_label_formatting() {
        let mut state = ClusterSwitcherState::new();
        assert_eq!(
            format_cluster_status(&state),
            StatusPresentation {
                status_label: "Disconnected",
                color: theme::TEXT_MUTED,
                pill_label: "Select Cluster".to_string(),
            }
        );

        state.connection_state = ClusterConnectionState::Connecting {
            cluster_id: "prod-ctx".to_string(),
            display_name: "Prod US".to_string(),
        };
        assert_eq!(
            format_cluster_status(&state),
            StatusPresentation {
                status_label: "Connecting",
                color: theme::STATUS_WARNING,
                pill_label: "Prod US".to_string(),
            }
        );

        state.connection_state = ClusterConnectionState::Connected {
            cluster_id: "prod-ctx".to_string(),
            display_name: "Prod US".to_string(),
            k8s_version: Some("v1.31.2".to_string()),
        };
        assert_eq!(
            format_cluster_status(&state),
            StatusPresentation {
                status_label: "Connected",
                color: theme::STATUS_SUCCESS,
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
            format_cluster_status(&state),
            StatusPresentation {
                status_label: "Unreachable",
                color: theme::STATUS_DANGER,
                pill_label: "Prod US".to_string(),
            }
        );

        state.connection_state = ClusterConnectionState::Error {
            cluster_id: None,
            display_name: None,
            error_message: "No kubeconfig loaded".to_string(),
        };
        assert_eq!(
            format_cluster_status(&state),
            StatusPresentation {
                status_label: "Error",
                color: theme::STATUS_DANGER,
                pill_label: "Select Cluster".to_string(),
            }
        );
    }
}
