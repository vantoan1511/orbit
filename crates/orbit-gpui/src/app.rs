use crate::bridge::start_core_bridge;
use crate::state::{ActivityTab, ClusterStatus, OrbitGpuiState, PodRow};
use crate::theme;
use gpui::prelude::FluentBuilder;
use gpui::{
    Context, FontWeight, InteractiveElement, IntoElement, ParentElement, Render,
    StatefulInteractiveElement, Styled, Window, WindowControlArea, div, px,
};
use orbit_core::command::CoreCommand;
use tokio::sync::mpsc;

pub struct OrbitGpuiApp {
    state: OrbitGpuiState,
    _command_tx: mpsc::Sender<CoreCommand>,
}

impl OrbitGpuiApp {
    pub fn new(cx: &mut Context<Self>) -> Self {
        let mut bridge = start_core_bridge();
        let command_tx = bridge.command_tx;

        cx.spawn(async move |this, cx| {
            while let Some(event) = bridge.event_rx.recv().await {
                let updated = this.update(cx, |app, cx| {
                    app.state.apply_core_event(&event);
                    cx.notify();
                });
                if updated.is_err() {
                    break;
                }
            }
        })
        .detach();

        Self {
            state: OrbitGpuiState::new(),
            _command_tx: command_tx,
        }
    }

    fn render_header(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let (dot_color, status_label) = match &self.state.cluster_status {
            ClusterStatus::Connecting => (theme::status_warning(), "Connecting...".to_string()),
            ClusterStatus::Connected(name) => (theme::status_success(), name.clone()),
            ClusterStatus::MockFallback => (theme::status_warning(), "Mock Cluster".to_string()),
        };

        div()
            .h_10()
            .w_full()
            .flex()
            .flex_row()
            .items_center()
            .justify_between()
            .px_3()
            .bg(theme::bg_app())
            .border_b_1()
            .border_color(theme::border())
            .child(
                // Left: Cluster status pill
                div()
                    .flex()
                    .flex_row()
                    .items_center()
                    .gap_2()
                    .px_2()
                    .py_1()
                    .rounded_md()
                    .bg(theme::bg_card())
                    .border_1()
                    .border_color(theme::border())
                    .child(div().w_2().h_2().rounded_full().bg(dot_color))
                    .child(
                        div()
                            .text_xs()
                            .font_weight(FontWeight::SEMIBOLD)
                            .text_color(theme::text_primary())
                            .child(status_label),
                    ),
            )
            .child(
                // Center: Title + drag region
                div()
                    .flex_1()
                    .h_full()
                    .flex()
                    .items_center()
                    .justify_center()
                    .window_control_area(WindowControlArea::Drag)
                    .child(
                        div()
                            .text_sm()
                            .font_weight(FontWeight::SEMIBOLD)
                            .text_color(theme::text_primary())
                            .child("Orbit"),
                    ),
            )
            .child(
                // Right: Window controls
                div()
                    .flex()
                    .flex_row()
                    .items_center()
                    .gap_1()
                    .child(
                        div()
                            .id("win-min")
                            .w_7()
                            .h_7()
                            .flex()
                            .items_center()
                            .justify_center()
                            .rounded_md()
                            .cursor_pointer()
                            .text_xs()
                            .text_color(theme::text_muted())
                            .hover(|s| s.bg(theme::bg_hover()).text_color(theme::text_primary()))
                            .on_click(cx.listener(|_, _, window, _| {
                                window.minimize_window();
                            }))
                            .child("—"),
                    )
                    .child(
                        div()
                            .id("win-max")
                            .w_7()
                            .h_7()
                            .flex()
                            .items_center()
                            .justify_center()
                            .rounded_md()
                            .cursor_pointer()
                            .text_xs()
                            .text_color(theme::text_muted())
                            .hover(|s| s.bg(theme::bg_hover()).text_color(theme::text_primary()))
                            .on_click(cx.listener(|_, _, window, _| {
                                window.zoom_window();
                            }))
                            .child("□"),
                    )
                    .child(
                        div()
                            .id("win-close")
                            .w_7()
                            .h_7()
                            .flex()
                            .items_center()
                            .justify_center()
                            .rounded_md()
                            .cursor_pointer()
                            .text_xs()
                            .text_color(theme::text_muted())
                            .hover(|s| s.bg(theme::status_danger()).text_color(theme::text_primary()))
                            .on_click(cx.listener(|_, _, window, cx| {
                                window.remove_window();
                                cx.quit();
                            }))
                            .child("×"),
                    ),
            )
    }

    fn render_activity_bar(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let tabs = [
            (ActivityTab::Pods, "Pods", "P", "tab-pods"),
            (ActivityTab::Workloads, "Work", "W", "tab-workloads"),
            (ActivityTab::Settings, "Conf", "S", "tab-settings"),
        ];

        let mut bar = div()
            .w_12()
            .h_full()
            .flex()
            .flex_col()
            .items_center()
            .py_2()
            .gap_2()
            .bg(theme::bg_app())
            .border_r_1()
            .border_color(theme::border());

        for (tab, label, glyph, id) in tabs {
            let active = self.state.active_tab == tab;
            bar = bar.child(
                div()
                    .id(id)
                    .w_9()
                    .py_1()
                    .flex()
                    .flex_col()
                    .items_center()
                    .justify_center()
                    .rounded_md()
                    .cursor_pointer()
                    .when(active, |s| {
                        s.bg(theme::bg_card())
                            .border_1()
                            .border_color(theme::border())
                            .text_color(theme::text_primary())
                    })
                    .when(!active, |s| {
                        s.text_color(theme::text_muted())
                            .hover(|h| h.bg(theme::bg_hover()).text_color(theme::text_primary()))
                    })
                    .on_click(cx.listener(move |app, _, _, cx| {
                        app.state.select_tab(tab);
                        cx.notify();
                    }))
                    .child(div().text_xs().font_weight(FontWeight::BOLD).child(glyph))
                    .child(div().text_size(px(9.0)).child(label)),
            );
        }

        bar
    }

    fn render_pods_table(&self) -> impl IntoElement {
        let mut card = div()
            .flex_1()
            .w_full()
            .flex()
            .flex_col()
            .bg(theme::bg_card())
            .border_1()
            .border_color(theme::border())
            .rounded_lg()
            .shadow_sm()
            .overflow_hidden();

        // Card header
        card = card.child(
            div()
                .w_full()
                .px_4()
                .py_3()
                .flex()
                .flex_row()
                .items_center()
                .justify_between()
                .border_b_1()
                .border_color(theme::border())
                .child(
                    div()
                        .text_sm()
                        .font_weight(FontWeight::BOLD)
                        .text_color(theme::text_primary())
                        .child("Pods"),
                )
                .child(
                    div()
                        .text_xs()
                        .text_color(theme::text_muted())
                        .child(format!("{} items", self.state.pods.len())),
                ),
        );

        // Column headers
        card = card.child(
            div()
                .w_full()
                .px_4()
                .py_2()
                .flex()
                .flex_row()
                .items_center()
                .gap_3()
                .border_b_1()
                .border_color(theme::border())
                .text_xs()
                .font_weight(FontWeight::SEMIBOLD)
                .text_color(theme::text_muted())
                .child(div().flex_1().child("NAME"))
                .child(div().w_32().child("NAMESPACE"))
                .child(div().w_32().child("STATUS"))
                .child(div().w_16().child("RESTARTS"))
                .child(div().w_16().child("AGE"))
                .child(div().w_32().child("NODE")),
        );

        // Rows
        let mut rows = div().flex_1().w_full().flex().flex_col();
        for pod in &self.state.pods {
            rows = rows.child(Self::render_pod_row(pod));
        }

        card.child(rows)
    }

    fn render_pod_row(pod: &PodRow) -> impl IntoElement {
        div()
            .w_full()
            .px_4()
            .py_2()
            .flex()
            .flex_row()
            .items_center()
            .gap_3()
            .border_b_1()
            .border_color(theme::border())
            .hover(|s| s.bg(theme::bg_hover()))
            .text_xs()
            .child(
                div()
                    .flex_1()
                    .font_weight(FontWeight::SEMIBOLD)
                    .text_color(theme::text_primary())
                    .child(pod.name.clone()),
            )
            .child(
                div()
                    .w_32()
                    .text_color(theme::text_muted())
                    .child(pod.namespace.clone()),
            )
            .child(
                div()
                    .w_32()
                    .flex()
                    .flex_row()
                    .items_center()
                    .gap_2()
                    .child(
                        div()
                            .w_2()
                            .h_2()
                            .rounded_full()
                            .bg(theme::status_color(&pod.status)),
                    )
                    .child(
                        div()
                            .text_color(theme::text_primary())
                            .child(pod.status.clone()),
                    ),
            )
            .child(
                div()
                    .w_16()
                    .text_color(theme::text_muted())
                    .child(pod.restarts.to_string()),
            )
            .child(
                div()
                    .w_16()
                    .text_color(theme::text_muted())
                    .child(pod.age.clone()),
            )
            .child(
                div()
                    .w_32()
                    .text_color(theme::text_muted())
                    .child(pod.node.clone()),
            )
    }

    fn render_placeholder(&self, title: &str, description: &str) -> impl IntoElement {
        div()
            .flex_1()
            .w_full()
            .flex()
            .flex_col()
            .items_center()
            .justify_center()
            .gap_2()
            .bg(theme::bg_card())
            .border_1()
            .border_color(theme::border())
            .rounded_lg()
            .shadow_sm()
            .child(
                div()
                    .text_lg()
                    .font_weight(FontWeight::BOLD)
                    .text_color(theme::text_primary())
                    .child(title.to_string()),
            )
            .child(
                div()
                    .text_xs()
                    .text_color(theme::text_muted())
                    .child(description.to_string()),
            )
    }
}

impl Render for OrbitGpuiApp {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let content = match self.state.active_tab {
            ActivityTab::Pods => self.render_pods_table().into_any_element(),
            ActivityTab::Workloads => self
                .render_placeholder(
                    "Workloads",
                    "Deployments, StatefulSets, and DaemonSets view (PoC placeholder).",
                )
                .into_any_element(),
            ActivityTab::Settings => self
                .render_placeholder(
                    "Settings",
                    "Cluster preferences and theme configuration (PoC placeholder).",
                )
                .into_any_element(),
        };

        div()
            .size_full()
            .flex()
            .flex_col()
            .bg(theme::bg_app())
            .text_color(theme::text_primary())
            .child(self.render_header(cx))
            .child(
                div()
                    .flex_1()
                    .w_full()
                    .flex()
                    .flex_row()
                    .child(self.render_activity_bar(cx))
                    .child(
                        div()
                            .flex_1()
                            .h_full()
                            .p_4()
                            .flex()
                            .flex_col()
                            .bg(theme::bg_app())
                            .child(content),
                    ),
            )
    }
}
