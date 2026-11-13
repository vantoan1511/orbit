use crate::bridge::start_core_bridge;
use crate::icons::{LUCIDE_FONT_FAMILY, category_icon, icon_glyph, resource_icon};
use crate::resources::{
    ColumnDef, ResourceRow, SUPPORTED_RESOURCE_KINDS, columns_for_kind, kind_api_name,
    kind_display_name,
};
use crate::state::{
    ActivityCategory, ActiveView, ClusterStatus, OrbitGpuiState, SelectedResource, SubNavItem,
    sub_nav_items,
};
use crate::theme;
use gpui::prelude::FluentBuilder;
use gpui::{
    AnyElement, Context, Div, FocusHandle, FontWeight, InteractiveElement, IntoElement,
    KeyDownEvent, ParentElement, Render, Rgba, StatefulInteractiveElement, Styled, Window,
    WindowControlArea, div, px,
};
use lucide_icons::Icon;
use orbit_core::command::{CoreCommand, ResourceKind};
use orbit_core::kubernetes::models::ClusterInfo;
use tokio::sync::mpsc;

pub struct OrbitGpuiApp {
    state: OrbitGpuiState,
    command_tx: mpsc::Sender<CoreCommand>,
    search_focus_handle: FocusHandle,
}

impl OrbitGpuiApp {
    pub fn new(cx: &mut Context<Self>) -> Self {
        let mut bridge = start_core_bridge();
        let command_tx = bridge.command_tx;
        let search_focus_handle = cx.focus_handle();

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
            command_tx,
            search_focus_handle,
        }
    }

    fn send_command(&self, cmd: CoreCommand) {
        if let Err(err) = self.command_tx.try_send(cmd) {
            eprintln!("Failed to dispatch CoreCommand: {err}");
        }
    }

    fn lucide_icon(icon: Icon, size_px: f32, color: Rgba) -> impl IntoElement {
        div()
            .font_family(LUCIDE_FONT_FAMILY)
            .text_size(px(size_px))
            .line_height(px(size_px))
            .text_color(color)
            .child(icon_glyph(icon))
    }

    fn eyebrow_label(text: impl Into<String>) -> impl IntoElement {
        div()
            .text_size(px(10.0))
            .font_weight(FontWeight::SEMIBOLD)
            .text_color(theme::text_muted())
            .child(text.into())
    }

    fn column_cell_container(col: &ColumnDef) -> Div {
        let cell = match col.width_px {
            Some(w) => div().w(px(w)),
            None => div().flex_1(),
        };
        if col.right_aligned {
            cell.flex().justify_end()
        } else {
            cell
        }
    }

    fn category_label(category: ActivityCategory) -> &'static str {
        match category {
            ActivityCategory::Workloads => "Workloads",
            ActivityCategory::Network => "Network",
            ActivityCategory::Storage => "Storage",
            ActivityCategory::Config => "Config",
            ActivityCategory::Cluster => "Cluster",
            ActivityCategory::Settings => "Settings",
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
                div()
                    .flex()
                    .flex_row()
                    .items_center()
                    .gap_2()
                    .child(self.render_cluster_pill(dot_color, status_label, cx))
                    .child(self.render_namespace_pill(cx)),
            )
            .child(self.render_drag_title())
            .child(self.render_window_controls(cx))
    }

    fn render_namespace_pill(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let ns_label = self
            .state
            .selected_namespace
            .clone()
            .unwrap_or_else(|| "All Namespaces".to_string());

        div()
            .id("namespace-switcher-btn")
            .flex()
            .flex_row()
            .items_center()
            .gap_2()
            .px_2()
            .py_1()
            .rounded_md()
            .cursor_pointer()
            .bg(theme::bg_card())
            .border_1()
            .border_color(theme::border())
            .hover(|s| s.bg(theme::bg_hover()))
            .on_click(cx.listener(|app, _, _, cx| {
                app.state.toggle_namespace_dropdown();
                cx.notify();
            }))
            .child(Self::lucide_icon(Icon::Layers, 12.0, theme::text_muted()))
            .child(
                div()
                    .text_xs()
                    .font_weight(FontWeight::MEDIUM)
                    .text_color(theme::text_primary())
                    .child(ns_label),
            )
            .child(Self::lucide_icon(Icon::ChevronDown, 12.0, theme::text_muted()))
    }

    fn render_cluster_pill(
        &self,
        dot_color: Rgba,
        status_label: String,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        div()
            .id("cluster-switcher-btn")
            .flex()
            .flex_row()
            .items_center()
            .gap_2()
            .px_2()
            .py_1()
            .rounded_md()
            .cursor_pointer()
            .bg(theme::bg_card())
            .border_1()
            .border_color(theme::border())
            .hover(|s| s.bg(theme::bg_hover()))
            .on_click(cx.listener(|app, _, _, cx| {
                app.state.toggle_cluster_dropdown();
                cx.notify();
            }))
            .child(div().w_2().h_2().rounded_full().bg(dot_color))
            .child(
                div()
                    .text_xs()
                    .font_weight(FontWeight::SEMIBOLD)
                    .text_color(theme::text_primary())
                    .child(status_label),
            )
            .child(Self::lucide_icon(Icon::ChevronDown, 12.0, theme::text_muted()))
    }

    fn render_drag_title(&self) -> impl IntoElement {
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
            )
    }

    fn render_window_controls(&self, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .flex()
            .flex_row()
            .items_center()
            .gap_1()
            .child(self.render_win_btn("win-min", "—", false, cx.listener(|_, _, w, _| w.minimize_window())))
            .child(self.render_win_btn("win-max", "□", false, cx.listener(|_, _, w, _| w.zoom_window())))
            .child(self.render_win_btn(
                "win-close",
                "×",
                true,
                cx.listener(|_, _, w, cx| {
                    w.remove_window();
                    cx.quit();
                }),
            ))
    }

    fn render_win_btn(
        &self,
        id: &'static str,
        glyph: &'static str,
        is_danger: bool,
        on_click: impl Fn(&gpui::ClickEvent, &mut Window, &mut gpui::App) + 'static,
    ) -> impl IntoElement {
        div()
            .id(id)
            .w_7()
            .h_7()
            .flex()
            .items_center()
            .justify_center()
            .rounded_md()
            .cursor_pointer()
            .text_xs()
            .text_color(theme::text_muted())
            .hover(move |s| {
                let bg = if is_danger {
                    theme::status_danger()
                } else {
                    theme::bg_hover()
                };
                s.bg(bg).text_color(theme::text_primary())
            })
            .on_click(on_click)
            .child(glyph)
    }

    fn render_cluster_dropdown(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let mut menu = div()
            .absolute()
            .top_10()
            .left_3()
            .w_64()
            .py_1()
            .flex()
            .flex_col()
            .bg(theme::bg_card())
            .border_1()
            .border_color(theme::border())
            .rounded_md()
            .shadow_md()
            .child(div().px_3().py_1().child(Self::eyebrow_label("SWITCH CONTEXT")));

        if self.state.clusters.is_empty() {
            return menu.child(
                div()
                    .px_3()
                    .py_2()
                    .text_xs()
                    .text_color(theme::text_muted())
                    .child("No kubeconfig contexts found"),
            );
        }

        for (idx, cluster) in self.state.clusters.iter().enumerate() {
            menu = menu.child(self.render_cluster_option(idx, cluster, cx));
        }
        menu
    }

    fn render_namespace_dropdown(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let mut ordered_namespaces = Vec::new();
        for priority in ["default", "kube-system"] {
            if self.state.namespaces.iter().any(|ns| ns == priority) {
                ordered_namespaces.push(priority.to_string());
            }
        }
        for ns in &self.state.namespaces {
            if ns != "default" && ns != "kube-system" {
                ordered_namespaces.push(ns.clone());
            }
        }

        let mut menu = div()
            .id("namespace-dropdown")
            .absolute()
            .top_10()
            .left(px(180.0))
            .w_56()
            .max_h_80()
            .overflow_y_scroll()
            .py_1()
            .flex()
            .flex_col()
            .bg(theme::bg_card())
            .border_1()
            .border_color(theme::border())
            .rounded_md()
            .shadow_md()
            .child(div().px_3().py_1().child(Self::eyebrow_label("NAMESPACE")));

        let all_active = self.state.selected_namespace.is_none();
        menu = menu.child(self.render_namespace_option("ns-opt-all", "All Namespaces", None, all_active, cx));

        for (idx, ns) in ordered_namespaces.into_iter().enumerate() {
            let is_active = self.state.selected_namespace.as_deref() == Some(ns.as_str());
            let label = ns.clone();
            menu = menu.child(self.render_namespace_option(
                ("ns-opt", idx),
                &label,
                Some(ns),
                is_active,
                cx,
            ));
        }

        menu
    }

    fn render_namespace_option(
        &self,
        id: impl Into<gpui::ElementId>,
        label: &str,
        value: Option<String>,
        is_active: bool,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        div()
            .id(id)
            .px_3()
            .py_1p5()
            .flex()
            .flex_row()
            .items_center()
            .justify_between()
            .cursor_pointer()
            .when(is_active, |s| s.bg(theme::bg_hover()))
            .hover(|s| s.bg(theme::bg_hover()))
            .on_click(cx.listener(move |app, _, _, cx| {
                app.state.select_namespace(value.clone());
                cx.notify();
            }))
            .child(
                div()
                    .text_xs()
                    .font_weight(if is_active {
                        FontWeight::SEMIBOLD
                    } else {
                        FontWeight::NORMAL
                    })
                    .text_color(theme::text_primary())
                    .child(label.to_string()),
            )
    }

    fn render_cluster_option(
        &self,
        idx: usize,
        cluster: &ClusterInfo,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let is_active = self.state.active_cluster_id.as_deref() == Some(cluster.id.as_str());
        let cluster_id = cluster.id.clone();
        let display = cluster
            .alias
            .as_deref()
            .filter(|a| !a.is_empty())
            .unwrap_or(&cluster.name)
            .to_string();

        div()
            .id(("cluster-opt", idx))
            .px_3()
            .py_2()
            .flex()
            .flex_row()
            .items_center()
            .justify_between()
            .cursor_pointer()
            .when(is_active, |s| s.bg(theme::bg_hover()))
            .hover(|s| s.bg(theme::bg_hover()))
            .on_click(cx.listener(move |app, _, _, cx| {
                let cmd = app.state.begin_cluster_switch(cluster_id.clone());
                app.send_command(cmd);
                cx.notify();
            }))
            .child(
                div()
                    .flex()
                    .flex_row()
                    .items_center()
                    .gap_2()
                    .child(div().w_2().h_2().rounded_full().bg(if is_active {
                        theme::status_success()
                    } else {
                        theme::text_muted()
                    }))
                    .child(
                        div()
                            .text_xs()
                            .font_weight(if is_active {
                                FontWeight::SEMIBOLD
                            } else {
                                FontWeight::NORMAL
                            })
                            .text_color(theme::text_primary())
                            .child(display),
                    ),
            )
            .child(
                div()
                    .text_size(px(10.0))
                    .text_color(theme::text_muted())
                    .child(cluster.status.clone()),
            )
    }

    fn render_activity_bar(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let top_categories = [
            ActivityCategory::Workloads,
            ActivityCategory::Network,
            ActivityCategory::Storage,
            ActivityCategory::Config,
            ActivityCategory::Cluster,
        ];

        let mut top_group = div().flex().flex_col().items_center().gap_2();
        for (idx, cat) in top_categories.into_iter().enumerate() {
            top_group = top_group.child(self.render_activity_button(cat, ("act-top", idx), cx));
        }

        let bottom_group = div()
            .flex()
            .flex_col()
            .items_center()
            .child(self.render_activity_button(ActivityCategory::Settings, ("act-settings", 0), cx));

        div()
            .w_12()
            .h_full()
            .flex()
            .flex_col()
            .items_center()
            .justify_between()
            .py_2()
            .bg(theme::bg_app())
            .border_r_1()
            .border_color(theme::border())
            .child(top_group)
            .child(bottom_group)
    }

    fn render_activity_button(
        &self,
        category: ActivityCategory,
        id: (&'static str, usize),
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let active = self.state.active_category == category;
        let icon_color = if active {
            theme::text_primary()
        } else {
            theme::text_muted()
        };

        div()
            .id(id)
            .w_9()
            .h_9()
            .flex()
            .items_center()
            .justify_center()
            .rounded_md()
            .cursor_pointer()
            .when(active, |s| {
                s.bg(theme::bg_card())
                    .border_1()
                    .border_color(theme::border())
            })
            .when(!active, |s| s.hover(|h| h.bg(theme::bg_hover())))
            .on_click(cx.listener(move |app, _, _, cx| {
                app.state.select_category(category);
                cx.notify();
            }))
            .child(Self::lucide_icon(category_icon(category), 16.0, icon_color))
    }

    fn render_sub_nav(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let category = self.state.active_category;
        let items = sub_nav_items(category);

        let mut list = div().flex_1().p_2().flex().flex_col().gap_1();
        if items.is_empty() {
            list = list.child(
                div()
                    .px_2()
                    .py_2()
                    .text_xs()
                    .text_color(theme::text_muted())
                    .child("Coming in Phase 2"),
            );
        } else {
            for (idx, item) in items.into_iter().enumerate() {
                list = list.child(self.render_sub_nav_item(idx, item, cx));
            }
        }

        div()
            .w_48()
            .h_full()
            .flex()
            .flex_col()
            .bg(theme::bg_sidebar())
            .border_r_1()
            .border_color(theme::border())
            .child(
                div()
                    .px_3()
                    .py_2()
                    .border_b_1()
                    .border_color(theme::border())
                    .child(
                        div()
                            .text_xs()
                            .font_weight(FontWeight::SEMIBOLD)
                            .text_color(theme::text_muted())
                            .child(Self::category_label(category).to_uppercase()),
                    ),
            )
            .child(list)
    }

    fn render_sub_nav_item(
        &self,
        idx: usize,
        item: SubNavItem,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let active = self.state.active_view == item.view;
        let icon = item
            .resource_kind
            .map(resource_icon)
            .unwrap_or(Icon::LayoutDashboard);
        let count = item
            .resource_kind
            .map(|k| self.state.filtered_resource_count(k));

        div()
            .id(("subnav", idx))
            .px_2()
            .py_1p5()
            .flex()
            .flex_row()
            .items_center()
            .justify_between()
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
                app.state.select_view(item.view);
                cx.notify();
            }))
            .child(
                div()
                    .flex()
                    .flex_row()
                    .items_center()
                    .gap_2()
                    .child(Self::lucide_icon(
                        icon,
                        14.0,
                        if active {
                            theme::accent()
                        } else {
                            theme::text_muted()
                        },
                    ))
                    .child(
                        div()
                            .text_xs()
                            .font_weight(if active {
                                FontWeight::SEMIBOLD
                            } else {
                                FontWeight::NORMAL
                            })
                            .child(item.label),
                    ),
            )
            .when_some(count, |el, c| {
                el.child(
                    div()
                        .text_size(px(10.0))
                        .text_color(theme::text_muted())
                        .child(c.to_string()),
                )
            })
    }

    fn render_overview(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let mut grid = div().flex().flex_row().flex_wrap().gap_3();
        for (idx, kind) in SUPPORTED_RESOURCE_KINDS.into_iter().enumerate() {
            grid = grid.child(self.render_overview_card(idx, kind, cx));
        }

        div()
            .flex_1()
            .w_full()
            .flex()
            .flex_col()
            .gap_4()
            .child(
                div()
                    .text_sm()
                    .font_weight(FontWeight::BOLD)
                    .text_color(theme::text_primary())
                    .child("Cluster Overview"),
            )
            .child(grid)
    }

    fn render_overview_card(
        &self,
        idx: usize,
        kind: ResourceKind,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let count = self.state.resource_count(kind);
        div()
            .id(("overview-card", idx))
            .w_48()
            .p_4()
            .flex()
            .flex_col()
            .gap_2()
            .bg(theme::bg_card())
            .border_1()
            .border_color(theme::border())
            .rounded_lg()
            .cursor_pointer()
            .hover(|s| s.bg(theme::bg_hover()))
            .on_click(cx.listener(move |app, _, _, cx| {
                app.state.select_resource_kind(kind);
                cx.notify();
            }))
            .child(
                div()
                    .flex()
                    .flex_row()
                    .items_center()
                    .justify_between()
                    .child(
                        div()
                            .text_xs()
                            .font_weight(FontWeight::SEMIBOLD)
                            .text_color(theme::text_muted())
                            .child(kind_display_name(kind)),
                    )
                    .child(Self::lucide_icon(resource_icon(kind), 14.0, theme::accent())),
            )
            .child(
                div()
                    .text_xl()
                    .font_weight(FontWeight::BOLD)
                    .text_color(theme::text_primary())
                    .child(count.to_string()),
            )
    }

    fn render_resource_table(&self, kind: ResourceKind, cx: &mut Context<Self>) -> impl IntoElement {
        let columns = columns_for_kind(kind);
        let rows_data = self.state.filtered_rows_for_active_view();

        let header_row = columns.iter().fold(
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
                .text_color(theme::text_muted()),
            |row, col| row.child(Self::column_cell_container(col).child(col.header)),
        );

        let mut body = div()
            .id("resource-table-body")
            .flex_1()
            .w_full()
            .flex()
            .flex_col()
            .overflow_y_scroll();

        if rows_data.is_empty() {
            let empty_msg = if self.state.cluster_status == ClusterStatus::Connecting {
                format!("Loading {} from cluster...", kind_display_name(kind))
            } else {
                format!("No {} found.", kind_display_name(kind))
            };
            body = body.child(
                div()
                    .p_6()
                    .text_xs()
                    .text_color(theme::text_muted())
                    .child(empty_msg),
            );
        } else {
            for (idx, row) in rows_data.iter().enumerate() {
                let is_selected = self.state.selected_resource.as_ref().is_some_and(|sel| {
                    sel.kind == row.kind && sel.name == row.name && sel.namespace == row.namespace
                });
                body = body.child(self.render_table_row(idx, row, &columns, is_selected, cx));
            }
        }

        div()
            .flex_1()
            .w_full()
            .flex()
            .flex_col()
            .bg(theme::bg_card())
            .border_1()
            .border_color(theme::border())
            .rounded_lg()
            .shadow_sm()
            .overflow_hidden()
            .child(self.render_table_toolbar(kind, rows_data.len(), cx))
            .child(header_row)
            .child(body)
    }

    fn render_table_toolbar(
        &self,
        kind: ResourceKind,
        count: usize,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        div()
            .w_full()
            .px_4()
            .py_3()
            .flex()
            .flex_row()
            .items_center()
            .justify_between()
            .gap_3()
            .border_b_1()
            .border_color(theme::border())
            .child(
                div()
                    .flex()
                    .flex_row()
                    .items_center()
                    .gap_2()
                    .child(Self::lucide_icon(resource_icon(kind), 15.0, theme::accent()))
                    .child(
                        div()
                            .text_sm()
                            .font_weight(FontWeight::BOLD)
                            .text_color(theme::text_primary())
                            .child(kind_display_name(kind)),
                    ),
            )
            .child(
                div()
                    .flex()
                    .flex_row()
                    .items_center()
                    .gap_3()
                    .child(self.render_search_input(cx))
                    .child(
                        div()
                            .text_xs()
                            .text_color(theme::text_muted())
                            .child(format!("{count} items")),
                    ),
            )
    }

    fn render_search_input(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let has_query = !self.state.search_query.is_empty();
        let search_display = if has_query {
            self.state.search_query.clone()
        } else {
            "Filter by name or label...".to_string()
        };

        div()
            .id("resource-search-input")
            .track_focus(&self.search_focus_handle)
            .w_56()
            .px_2p5()
            .py_1()
            .flex()
            .flex_row()
            .items_center()
            .justify_between()
            .gap_2()
            .rounded_md()
            .bg(theme::bg_app())
            .border_1()
            .border_color(if self.state.search_focused {
                theme::accent()
            } else {
                theme::border()
            })
            .cursor_text()
            .on_click(cx.listener(|app, _, window, cx| {
                app.state.search_focused = true;
                window.focus(&app.search_focus_handle);
                cx.notify();
            }))
            .child(
                div()
                    .flex()
                    .flex_row()
                    .items_center()
                    .gap_1p5()
                    .overflow_hidden()
                    .child(Self::lucide_icon(Icon::Search, 12.0, theme::text_muted()))
                    .child(
                        div()
                            .text_xs()
                            .text_color(if has_query {
                                theme::text_primary()
                            } else {
                                theme::text_muted()
                            })
                            .child(search_display),
                    ),
            )
            .when(has_query, |el| {
                el.child(
                    div()
                        .id("search-clear-btn")
                        .px_1()
                        .rounded_sm()
                        .cursor_pointer()
                        .text_xs()
                        .text_color(theme::text_muted())
                        .hover(|s| s.text_color(theme::text_primary()))
                        .on_click(cx.listener(|app, _, _, cx| {
                            app.state.clear_search_query();
                            cx.notify();
                        }))
                        .child("×"),
                )
            })
    }

    fn render_table_row(
        &self,
        idx: usize,
        row: &ResourceRow,
        columns: &[ColumnDef],
        is_selected: bool,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let row_clone = row.clone();
        let mut el = div()
            .id(("res-row", idx))
            .w_full()
            .px_4()
            .py_2()
            .flex()
            .flex_row()
            .items_center()
            .gap_3()
            .border_b_1()
            .border_color(theme::border())
            .cursor_pointer()
            .when(is_selected, |s| s.bg(theme::bg_hover()))
            .hover(|s| s.bg(theme::bg_hover()))
            .text_xs()
            .on_click(cx.listener(move |app, _, _, cx| {
                let cmd = app.state.select_row(&row_clone);
                app.send_command(cmd);
                cx.notify();
            }));

        for (col_idx, (col, value)) in columns.iter().zip(row.cells.iter()).enumerate() {
            el = el.child(Self::render_table_cell(col_idx, col, value));
        }
        el
    }

    fn render_table_cell(col_idx: usize, col: &ColumnDef, value: &str) -> AnyElement {
        let container = Self::column_cell_container(col);
        if col.header == "STATUS" {
            return container
                .flex()
                .flex_row()
                .items_center()
                .gap_2()
                .child(div().w_2().h_2().rounded_full().bg(theme::status_color(value)))
                .child(div().text_color(theme::text_primary()).child(value.to_string()))
                .into_any_element();
        }
        if col_idx == 0 {
            return container
                .font_weight(FontWeight::SEMIBOLD)
                .text_color(theme::text_primary())
                .child(value.to_string())
                .into_any_element();
        }
        container
            .text_color(theme::text_muted())
            .child(value.to_string())
            .into_any_element()
    }

    fn render_details_drawer(
        &self,
        sel: &SelectedResource,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let row = self.state.selected_row();
        let status = row.map(|r| r.status.as_str()).unwrap_or("Active");

        let mut content = div()
            .id("drawer-content")
            .flex_1()
            .p_4()
            .flex()
            .flex_col()
            .gap_4()
            .overflow_y_scroll();

        if let Some(row) = row {
            content = content.child(Self::render_drawer_metadata(&row.details));
            if !row.labels.is_empty() {
                content = content.child(Self::render_drawer_labels(&row.labels));
            }
        }

        content = content.child(self.render_drawer_yaml());

        div()
            .w_96()
            .h_full()
            .flex()
            .flex_col()
            .bg(theme::bg_sidebar())
            .border_l_1()
            .border_color(theme::border())
            .child(self.render_drawer_header(sel, status, cx))
            .child(content)
    }

    fn render_drawer_header(
        &self,
        sel: &SelectedResource,
        status: &str,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        div()
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
                    .flex()
                    .flex_col()
                    .gap_0p5()
                    .child(
                        div()
                            .flex()
                            .flex_row()
                            .items_center()
                            .gap_2()
                            .child(div().w_2().h_2().rounded_full().bg(theme::status_color(status)))
                            .child(Self::eyebrow_label(kind_api_name(sel.kind).to_uppercase())),
                    )
                    .child(
                        div()
                            .text_sm()
                            .font_weight(FontWeight::BOLD)
                            .text_color(theme::text_primary())
                            .child(sel.name.clone()),
                    ),
            )
            .child(
                div()
                    .id("close-drawer")
                    .w_7()
                    .h_7()
                    .flex()
                    .items_center()
                    .justify_center()
                    .rounded_md()
                    .cursor_pointer()
                    .hover(|s| s.bg(theme::bg_hover()))
                    .on_click(cx.listener(|app, _, _, cx| {
                        app.state.close_drawer();
                        cx.notify();
                    }))
                    .child(Self::lucide_icon(Icon::X, 14.0, theme::text_muted())),
            )
    }

    fn render_drawer_metadata(details: &[(String, String)]) -> impl IntoElement {
        let mut box_el = div().flex().flex_col().gap_1p5().child(Self::eyebrow_label("METADATA"));
        for (k, v) in details {
            box_el = box_el.child(
                div()
                    .flex()
                    .flex_row()
                    .items_center()
                    .justify_between()
                    .text_xs()
                    .child(div().text_color(theme::text_muted()).child(k.clone()))
                    .child(div().text_color(theme::text_primary()).child(v.clone())),
            );
        }
        box_el
    }

    fn render_drawer_labels(labels: &[(String, String)]) -> impl IntoElement {
        let mut wrap = div().flex().flex_row().flex_wrap().gap_1();
        for (k, v) in labels {
            wrap = wrap.child(
                div()
                    .px_2()
                    .py_0p5()
                    .rounded_md()
                    .bg(theme::bg_card())
                    .border_1()
                    .border_color(theme::border())
                    .text_size(px(10.0))
                    .text_color(theme::text_muted())
                    .child(format!("{k}={v}")),
            );
        }
        div()
            .flex()
            .flex_col()
            .gap_1p5()
            .child(Self::eyebrow_label("LABELS"))
            .child(wrap)
    }

    fn render_drawer_yaml(&self) -> impl IntoElement {
        let yaml_text = if self.state.raw_yaml_loading {
            "Loading YAML from cluster...".to_string()
        } else {
            self.state
                .raw_yaml
                .clone()
                .unwrap_or_else(|| "# No YAML available".to_string())
        };

        div()
            .flex_1()
            .flex()
            .flex_col()
            .gap_1p5()
            .child(Self::eyebrow_label("RAW YAML"))
            .child(
                div()
                    .id("drawer-yaml-scroll")
                    .flex_1()
                    .p_3()
                    .rounded_md()
                    .bg(theme::bg_app())
                    .border_1()
                    .border_color(theme::border())
                    .text_size(px(11.0))
                    .text_color(theme::text_primary())
                    .overflow_y_scroll()
                    .child(yaml_text),
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
    fn handle_key_down(
        &mut self,
        event: &KeyDownEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if event.keystroke.key == "escape" {
            if self.state.cluster_dropdown_open || self.state.namespace_dropdown_open {
                self.state.close_cluster_dropdown();
                self.state.close_namespace_dropdown();
                cx.notify();
                return;
            }
            if self.search_focus_handle.is_focused(window) {
                self.state.clear_search_query();
                self.state.search_focused = false;
                cx.notify();
                return;
            }
        }

        if !self.search_focus_handle.is_focused(window) {
            return;
        }

        if event.keystroke.key == "backspace" {
            let mut query = self.state.search_query.clone();
            query.pop();
            self.state.set_search_query(query);
            cx.notify();
            return;
        }

        if let Some(ch) = &event.keystroke.key_char
            && !event.keystroke.modifiers.control
            && !event.keystroke.modifiers.alt
            && !event.keystroke.modifiers.platform
            && !ch.chars().any(|c| c.is_control())
        {
            let mut query = self.state.search_query.clone();
            query.push_str(ch);
            self.state.set_search_query(query);
            cx.notify();
        }
    }
}

impl Render for OrbitGpuiApp {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        self.state.search_focused = self.search_focus_handle.is_focused(window);
        let main_content = match self.state.active_view {
            ActiveView::Overview => self.render_overview(cx).into_any_element(),
            ActiveView::Resource(kind) => self.render_resource_table(kind, cx).into_any_element(),
            ActiveView::Settings => self
                .render_placeholder(
                    "Settings",
                    "Cluster preferences and theme configuration (Phase 2).",
                )
                .into_any_element(),
        };

        let show_sub_nav =
            self.state.sub_nav_open && self.state.active_category != ActivityCategory::Settings;

        div()
            .relative()
            .size_full()
            .flex()
            .flex_col()
            .bg(theme::bg_app())
            .text_color(theme::text_primary())
            .on_key_down(cx.listener(Self::handle_key_down))
            .child(self.render_header(cx))
            .child(
                div()
                    .flex_1()
                    .w_full()
                    .flex()
                    .flex_row()
                    .overflow_hidden()
                    .child(self.render_activity_bar(cx))
                    .when(show_sub_nav, |el| el.child(self.render_sub_nav(cx)))
                    .child(
                        div()
                            .flex_1()
                            .h_full()
                            .p_4()
                            .flex()
                            .flex_col()
                            .bg(theme::bg_app())
                            .overflow_hidden()
                            .child(main_content),
                    )
                    .when_some(self.state.selected_resource.clone(), |el, sel| {
                        el.child(self.render_details_drawer(&sel, cx))
                    }),
            )
            .when(
                self.state.cluster_dropdown_open || self.state.namespace_dropdown_open,
                |el| {
                    el.child(
                        div()
                            .id("dropdown-backdrop")
                            .absolute()
                            .inset_0()
                            .on_click(cx.listener(|app, _, _, cx| {
                                app.state.close_cluster_dropdown();
                                app.state.close_namespace_dropdown();
                                cx.notify();
                            })),
                    )
                },
            )
            .when(self.state.cluster_dropdown_open, |el| {
                el.child(self.render_cluster_dropdown(cx))
            })
            .when(self.state.namespace_dropdown_open, |el| {
                el.child(self.render_namespace_dropdown(cx))
            })
    }
}
