use std::cmp::Ordering;
use std::rc::Rc;

use gpui::{
    AnyElement, App, ClickEvent, ElementId, FontWeight, MouseDownEvent, Rgba, SharedString, Window,
    div, prelude::*, px,
};
use lucide_icons::Icon;
use orbit_core::kubernetes::models::NodeInfo;
use orbit_core::kubernetes::nodes::format_decimal;

use crate::components::data_table::{
    DataTableCallbacks, SortState, TableColumn, render_data_table, sort_rows_stable,
};
use crate::components::{
    ContextMenuEntry, ContextMenuItem, StatusBadgeVariant, render_status_badge,
    render_status_badge_from_str,
};
use crate::navigation::WorkspaceView;
use crate::state::{NodesLoadStatus, NodesState};
use crate::theme::{K8sResourceKind, OrbitTheme, apply_text_step};
use crate::views::{
    parse_age_seconds, render_resource_error_banner, render_resource_error_state,
    render_resource_view_header, render_truncated_cell,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NodesPresentationMode {
    Table,
    ErrorCard,
}

#[derive(Debug, Clone, PartialEq)]
pub struct NodeSummary {
    pub total: usize,
    pub ready: usize,
    pub not_ready: usize,
    pub cordoned: usize,
    pub control_plane: usize,
    pub workers: usize,
    pub cpu_used_cores: f64,
    pub cpu_total_cores: f64,
    pub cpu_pct: f64,
    pub mem_used_gib: f64,
    pub mem_total_gib: f64,
    pub mem_pct: f64,
}

#[derive(Debug, Clone)]
pub struct NodeMetricCard {
    pub title: &'static str,
    pub primary_value: String,
    pub subtitle: String,
    pub accent_color: Rgba,
}

pub fn node_columns() -> Vec<TableColumn> {
    vec![
        TableColumn::new("name", "Name").flex_width(1.8, px(150.0)),
        TableColumn::new("status", "Status").fixed_width(px(175.0)),
        TableColumn::new("role", "Role").fixed_width(px(120.0)),
        TableColumn::new("version", "Version")
            .fixed_width(px(125.0))
            .monospace(true),
        TableColumn::new("cpu", "CPU")
            .fixed_width(px(135.0))
            .right_numeric(),
        TableColumn::new("memory", "Memory")
            .fixed_width(px(140.0))
            .right_numeric(),
        TableColumn::new("pods", "Pods")
            .fixed_width(px(90.0))
            .right_numeric(),
        TableColumn::new("age", "Age")
            .fixed_width(px(76.0))
            .right_numeric(),
    ]
}

fn compare_nodes_by_col(a: &NodeInfo, b: &NodeInfo, col_id: &str) -> Ordering {
    let tie_break = || a.name.to_ascii_lowercase().cmp(&b.name.to_ascii_lowercase());

    match col_id {
        "name" => tie_break(),
        "status" => a
            .status
            .to_ascii_lowercase()
            .cmp(&b.status.to_ascii_lowercase())
            .then_with(|| a.is_cordoned.cmp(&b.is_cordoned))
            .then_with(tie_break),
        "role" => a
            .role
            .to_ascii_lowercase()
            .cmp(&b.role.to_ascii_lowercase())
            .then_with(tie_break),
        "version" => a
            .version
            .to_ascii_lowercase()
            .cmp(&b.version.to_ascii_lowercase())
            .then_with(tie_break),
        "cpu" => a
            .cpu_pct
            .total_cmp(&b.cpu_pct)
            .then_with(tie_break),
        "memory" => a
            .mem_pct
            .total_cmp(&b.mem_pct)
            .then_with(tie_break),
        "pods" => a
            .pods_count
            .cmp(&b.pods_count)
            .then_with(|| a.pods_limit.cmp(&b.pods_limit))
            .then_with(tie_break),
        "age" => parse_age_seconds(&a.uptime)
            .cmp(&parse_age_seconds(&b.uptime))
            .then_with(tie_break),
        _ => tie_break(),
    }
}

pub fn sort_nodes<'a>(nodes: &'a [NodeInfo], sort: Option<&SortState>) -> Vec<&'a NodeInfo> {
    let mut rows: Vec<&'a NodeInfo> = nodes.iter().collect();
    if sort.is_some() {
        sort_rows_stable(&mut rows, sort, |a, b, col_id| {
            compare_nodes_by_col(a, b, col_id)
        });
    } else {
        rows.sort_by_key(|a| a.name.to_ascii_lowercase());
    }
    rows
}

pub fn compute_node_summary(nodes: &[NodeInfo]) -> NodeSummary {
    let total = nodes.len();
    let mut ready = 0_usize;
    let mut not_ready = 0_usize;
    let mut cordoned = 0_usize;
    let mut control_plane = 0_usize;
    let mut workers = 0_usize;
    let mut cpu_used_cores = 0.0_f64;
    let mut cpu_total_cores = 0.0_f64;
    let mut mem_used_gib = 0.0_f64;
    let mut mem_total_gib = 0.0_f64;

    for node in nodes {
        if node.status.eq_ignore_ascii_case("ready") {
            ready += 1;
        } else {
            not_ready += 1;
        }
        if node.is_cordoned {
            cordoned += 1;
        }
        if node.role.eq_ignore_ascii_case("control-plane") {
            control_plane += 1;
        } else {
            workers += 1;
        }

        cpu_used_cores += node.cpu_used.trim().parse::<f64>().unwrap_or(0.0);
        cpu_total_cores += node.cpu_total.trim().parse::<f64>().unwrap_or(0.0);
        mem_used_gib += node.mem_used.trim().parse::<f64>().unwrap_or(0.0);
        mem_total_gib += node.mem_total.trim().parse::<f64>().unwrap_or(0.0);
    }

    let cpu_pct = if cpu_total_cores > 0.0 {
        ((cpu_used_cores / cpu_total_cores) * 1000.0).round() / 10.0
    } else {
        0.0
    };
    let mem_pct = if mem_total_gib > 0.0 {
        ((mem_used_gib / mem_total_gib) * 1000.0).round() / 10.0
    } else {
        0.0
    };

    NodeSummary {
        total,
        ready,
        not_ready,
        cordoned,
        control_plane,
        workers,
        cpu_used_cores,
        cpu_total_cores,
        cpu_pct,
        mem_used_gib,
        mem_total_gib,
        mem_pct,
    }
}

pub fn node_summary_cards(nodes: &[NodeInfo], theme: &OrbitTheme) -> [NodeMetricCard; 4] {
    let summary = compute_node_summary(nodes);
    let ready_pct = (summary.ready.min(summary.total) * 100)
        .checked_div(summary.total)
        .unwrap_or(0);

    let attention_count = summary.cordoned + summary.not_ready;
    let attention_color = if summary.not_ready > 0 {
        theme.semantic.danger.solid
    } else if summary.cordoned > 0 {
        theme.semantic.warning.solid
    } else {
        theme.text.muted
    };

    [
        NodeMetricCard {
            title: "Total Nodes",
            primary_value: summary.total.to_string(),
            subtitle: format!(
                "{} control-plane • {} worker",
                summary.control_plane, summary.workers
            ),
            accent_color: theme.k8s_color(K8sResourceKind::Node),
        },
        NodeMetricCard {
            title: "Ready Nodes",
            primary_value: summary.ready.to_string(),
            subtitle: format!("{ready_pct}% online"),
            accent_color: theme.semantic.success.solid,
        },
        NodeMetricCard {
            title: "Cordoned / NotReady",
            primary_value: attention_count.to_string(),
            subtitle: format!(
                "{} cordoned • {} not ready",
                summary.cordoned, summary.not_ready
            ),
            accent_color: attention_color,
        },
        NodeMetricCard {
            title: "Resource Allocation",
            primary_value: format!("{:.1}% / {:.1}%", summary.cpu_pct, summary.mem_pct),
            subtitle: format!(
                "{} / {}C • {} / {}G",
                format_decimal(summary.cpu_used_cores, 2),
                format_decimal(summary.cpu_total_cores, 0),
                format_decimal(summary.mem_used_gib, 2),
                format_decimal(summary.mem_total_gib, 0),
            ),
            accent_color: theme.accent.accent,
        },
    ]
}

pub fn nodes_empty_message(_state: &NodesState) -> &'static str {
    "No nodes found in this cluster."
}

pub fn nodes_presentation_mode(state: &NodesState) -> NodesPresentationMode {
    if state.status == NodesLoadStatus::Error && state.nodes.is_empty() {
        NodesPresentationMode::ErrorCard
    } else {
        NodesPresentationMode::Table
    }
}

fn render_node_cell(
    row_idx: usize,
    node: &NodeInfo,
    col: &TableColumn,
    theme: &OrbitTheme,
) -> AnyElement {
    match col.id.as_ref() {
        "name" => render_truncated_cell(
            ElementId::NamedInteger(SharedString::from("node-name"), row_idx as u64),
            node.name.clone(),
            Some(SharedString::from(node.name.clone())),
            theme.text.primary,
            FontWeight::MEDIUM,
            false,
            theme,
        ),
        "status" => div()
            .flex()
            .items_center()
            .gap_1p5()
            .min_w(px(0.0))
            .child(render_status_badge_from_str(
                &node.status,
                StatusBadgeVariant::Inline,
                theme,
            ))
            .when(node.is_cordoned, |el| {
                el.child(render_status_badge(
                    "Cordoned",
                    crate::components::SemanticStatus::Warning,
                    StatusBadgeVariant::Pill,
                    theme,
                ))
            })
            .into_any_element(),
        "role" => render_truncated_cell(
            ElementId::NamedInteger(SharedString::from("node-role"), row_idx as u64),
            node.role.clone(),
            None,
            theme.text.secondary,
            FontWeight::NORMAL,
            false,
            theme,
        ),
        "version" => render_truncated_cell(
            ElementId::NamedInteger(SharedString::from("node-ver"), row_idx as u64),
            node.version.clone(),
            None,
            theme.text.secondary,
            FontWeight::NORMAL,
            true,
            theme,
        ),
        "cpu" => apply_text_step(div(), theme.typography.size_xs)
            .font(theme.typography.mono_font(FontWeight::NORMAL))
            .text_color(theme.text.primary)
            .truncate()
            .child(format!(
                "{:.1}% ({}/{}C)",
                node.cpu_pct, node.cpu_used, node.cpu_total
            ))
            .into_any_element(),
        "memory" => apply_text_step(div(), theme.typography.size_xs)
            .font(theme.typography.mono_font(FontWeight::NORMAL))
            .text_color(theme.text.primary)
            .truncate()
            .child(format!(
                "{:.1}% ({}/{}G)",
                node.mem_pct, node.mem_used, node.mem_total
            ))
            .into_any_element(),
        "pods" => apply_text_step(div(), theme.typography.size_xs)
            .font(theme.typography.mono_font(FontWeight::NORMAL))
            .text_color(theme.text.primary)
            .child(format!("{}/{}", node.pods_count, node.pods_limit))
            .into_any_element(),
        "age" => apply_text_step(div(), theme.typography.size_xs)
            .font(theme.typography.mono_font(FontWeight::NORMAL))
            .text_color(theme.text.muted)
            .child(node.uptime.clone())
            .into_any_element(),
        _ => div().into_any_element(),
    }
}

fn render_summary_cards_row(nodes: &[NodeInfo], theme: &OrbitTheme) -> AnyElement {
    let cards = node_summary_cards(nodes, theme);
    div()
        .grid()
        .grid_cols(4)
        .gap_3()
        .flex_shrink_0()
        .children(cards.into_iter().map(|card| {
            theme
                .style_card(div())
                .p_3()
                .flex()
                .flex_col()
                .gap_1()
                .child(
                    div()
                        .flex()
                        .items_center()
                        .justify_between()
                        .child(
                            apply_text_step(div(), theme.typography.size_xs)
                                .font_weight(FontWeight::MEDIUM)
                                .text_color(theme.text.muted)
                                .child(card.title),
                        )
                        .child(div().w(px(6.0)).h(px(6.0)).rounded_full().bg(card.accent_color)),
                )
                .child(
                    apply_text_step(div(), theme.typography.size_lg)
                        .font(theme.typography.tabular_ui_font(FontWeight::SEMIBOLD))
                        .text_color(theme.text.primary)
                        .child(card.primary_value),
                )
                .child(
                    apply_text_step(div(), theme.typography.size_xs)
                        .text_color(theme.text.secondary)
                        .truncate()
                        .child(card.subtitle),
                )
        }))
        .into_any_element()
}

pub fn build_node_context_menu_entries(
    _node: &NodeInfo,
    on_copy_name: impl 'static + Fn(&mut Window, &mut App),
    on_refresh: impl 'static + Fn(&mut Window, &mut App),
    on_view_details: impl 'static + Fn(&mut Window, &mut App),
    on_view_yaml: impl 'static + Fn(&mut Window, &mut App),
) -> Vec<ContextMenuEntry> {
    vec![
        ContextMenuEntry::Item(
            ContextMenuItem::new("copy-name", "Copy Name")
                .icon(Icon::Copy)
                .on_select(on_copy_name),
        ),
        ContextMenuEntry::Item(
            ContextMenuItem::new("refresh", "Refresh")
                .icon(Icon::RefreshCw)
                .on_select(on_refresh),
        ),
        ContextMenuEntry::Separator,
        ContextMenuEntry::Item(
            ContextMenuItem::new("view-details", "View Details")
                .icon(Icon::Info)
                .on_select(on_view_details),
        ),
        ContextMenuEntry::Item(
            ContextMenuItem::new("view-yaml", "View YAML")
                .icon(Icon::FileCode)
                .on_select(on_view_yaml),
        ),
        ContextMenuEntry::Separator,
        ContextMenuEntry::Item(
            ContextMenuItem::new("delete-node", "Delete Node")
                .icon(Icon::Trash2)
                .danger(true)
                .disabled(true),
        ),
    ]
}

#[allow(clippy::too_many_arguments)]
pub fn render_nodes_view(
    state: &NodesState,
    theme: &OrbitTheme,
    on_sort: impl 'static + Fn(SharedString, &ClickEvent, &mut Window, &mut App),
    on_row_click: impl 'static + Fn(usize, SharedString, &ClickEvent, &mut Window, &mut App),
    on_row_quick_yaml: impl 'static + Fn(usize, SharedString, &ClickEvent, &mut Window, &mut App),
    on_row_context_menu: impl 'static
        + Fn(SharedString, &NodeInfo, &MouseDownEvent, &mut Window, &mut App),
    on_refresh: impl 'static + Fn(&mut Window, &mut App),
) -> AnyElement {
    let columns = node_columns();
    let sorted_rows: Rc<Vec<NodeInfo>> = Rc::new(
        sort_nodes(&state.nodes, state.table.sort.as_ref())
            .into_iter()
            .cloned()
            .collect(),
    );
    let total_count = sorted_rows.len();
    let is_loading = state.status == NodesLoadStatus::Loading;
    let on_refresh_rc: super::ActionCallback = Rc::new(on_refresh);

    let header = render_resource_view_header(
        WorkspaceView::Nodes.label(),
        WorkspaceView::Nodes.description(),
        format!("{total_count} nodes"),
        None,
        None,
        theme,
    );

    let summary_row = div()
        .w_full()
        .px_4()
        .pb_3()
        .flex_shrink_0()
        .child(render_summary_cards_row(&state.nodes, theme));

    let error_banner = state.error_message.as_ref().map(|message| {
        render_resource_error_banner(
            "nodes-retry-btn",
            format!("Failed to load nodes: {message}"),
            is_loading,
            theme,
            on_refresh_rc.clone(),
        )
    });

    let body = match nodes_presentation_mode(state) {
        NodesPresentationMode::ErrorCard => {
            let message = state
                .error_message
                .clone()
                .unwrap_or_else(|| "Unknown error".to_string());
            render_resource_error_state(
                "nodes-retry-btn",
                "Unable to Load Nodes",
                message,
                is_loading,
                theme,
                on_refresh_rc,
            )
        }
        NodesPresentationMode::Table => {
            let mut table_state = state.table.clone();
            table_state.loading = is_loading && state.nodes.is_empty();
            table_state.empty_message = SharedString::from(nodes_empty_message(state));

            let key_rows = sorted_rows.clone();
            let cell_rows = sorted_rows.clone();
            let ctx_rows = sorted_rows.clone();
            let callbacks = DataTableCallbacks::new(
                move |idx| {
                    key_rows
                        .get(idx)
                        .map(|node| node.name.clone())
                        .unwrap_or_default()
                },
                move |idx, col, col_theme| {
                    cell_rows
                        .get(idx)
                        .map(|node| render_node_cell(idx, node, col, col_theme))
                        .unwrap_or_else(|| div().into_any_element())
                },
            )
            .on_sort(on_sort)
            .on_row_click(on_row_click)
            .on_row_quick_yaml(on_row_quick_yaml)
            .on_row_context_menu(move |idx, row_key, event, window, cx| {
                if let Some(node) = ctx_rows.get(idx) {
                    on_row_context_menu(row_key, node, event, window, cx);
                }
            });

            render_data_table(
                "nodes-table",
                &columns,
                total_count,
                &table_state,
                theme,
                callbacks,
            )
        }
    };

    div()
        .flex()
        .flex_col()
        .flex_1()
        .min_h(px(0.0))
        .gap_0()
        .child(header)
        .child(summary_row)
        .when(
            nodes_presentation_mode(state) == NodesPresentationMode::Table,
            |el| el.when_some(error_banner, |inner, banner| inner.child(banner)),
        )
        .child(body)
        .into_any_element()
}

pub fn filter_events_for_node<'a>(
    events: &'a [orbit_core::kubernetes::models::EventInfo],
    node: &NodeInfo,
) -> Vec<&'a orbit_core::kubernetes::models::EventInfo> {
    events
        .iter()
        .filter(|ev| {
            (ev.object_kind == "Node" && ev.object_name == node.name)
                || ev.message.contains(&node.name)
        })
        .collect()
}

fn render_node_resource_bar(
    label: &'static str,
    pct: f64,
    used: &str,
    total: &str,
    theme: &OrbitTheme,
) -> AnyElement {
    let clamped = pct.clamp(0.0, 100.0) as f32;
    let bar_color = if clamped >= 85.0 {
        theme.semantic.danger.solid
    } else if clamped >= 70.0 {
        theme.semantic.warning.solid
    } else {
        theme.accent.accent
    };

    div()
        .w_full()
        .p_2p5()
        .rounded(theme.style.radius_control)
        .bg(theme.surface.bg_panel)
        .border_1()
        .border_color(theme.border.default)
        .flex()
        .flex_col()
        .gap_1p5()
        .child(
            div()
                .w_full()
                .flex()
                .items_center()
                .justify_between()
                .child(
                    apply_text_step(div(), theme.typography.size_xs)
                        .font_weight(FontWeight::MEDIUM)
                        .text_color(theme.text.secondary)
                        .child(label),
                )
                .child(
                    apply_text_step(div(), theme.typography.size_xs)
                        .font(theme.typography.mono_font(FontWeight::MEDIUM))
                        .text_color(theme.text.primary)
                        .child(format!("{used} / {total} ({pct:.0}%)")),
                ),
        )
        .child(
            div()
                .w_full()
                .h(px(6.0))
                .rounded_full()
                .bg(theme.surface.control_bg)
                .overflow_hidden()
                .child(
                    div()
                        .h_full()
                        .w(gpui::relative(clamped / 100.0))
                        .bg(bar_color),
                ),
        )
        .into_any_element()
}

pub fn render_node_drawer_overview(node: &NodeInfo, theme: &OrbitTheme) -> AnyElement {
    let pods_pct = if node.pods_limit > 0 {
        (node.pods_count as f64 / node.pods_limit as f64) * 100.0
    } else {
        0.0
    };

    let utilization = div()
        .w_full()
        .flex()
        .flex_col()
        .gap_2()
        .child(render_node_resource_bar(
            "CPU",
            node.cpu_pct,
            &node.cpu_used,
            &node.cpu_total,
            theme,
        ))
        .child(render_node_resource_bar(
            "Memory",
            node.mem_pct,
            &node.mem_used,
            &node.mem_total,
            theme,
        ))
        .child(render_node_resource_bar(
            "Pods",
            pods_pct,
            &node.pods_count.to_string(),
            &node.pods_limit.to_string(),
            theme,
        ))
        .into_any_element();

    let mut summary_rows = vec![
        ("Name", node.name.clone(), true),
        ("Status", node.status.clone(), false),
        ("Role", node.role.clone(), false),
        ("Kubelet Version", node.version.clone(), true),
        ("Cordoned", if node.is_cordoned { "Yes" } else { "No" }.to_string(), false),
        ("Uptime", node.uptime.clone(), true),
        ("Images Cached", node.images_count.to_string(), true),
    ];
    if let Some(created) = node.created_at.as_ref().filter(|s| !s.is_empty()) {
        summary_rows.push(("Created", created.clone(), true));
    }

    let system_section = node.node_info.as_ref().map(|sys| {
        crate::views::render_drawer_section(
            "SYSTEM INFO",
            crate::views::render_drawer_key_values(
                vec![
                    ("OS Image", sys.os_image.clone(), false),
                    ("Kernel", sys.kernel_version.clone(), true),
                    ("Runtime", sys.container_runtime_version.clone(), true),
                    ("Architecture", sys.architecture.clone(), true),
                ],
                theme,
            ),
            theme,
        )
    });

    let addresses_section = if node.addresses.is_empty() {
        None
    } else {
        Some(crate::views::render_drawer_section(
            "ADDRESSES",
            crate::views::render_drawer_key_values(
                node.addresses
                    .iter()
                    .map(|addr| {
                        let label: &'static str = match addr.type_.as_str() {
                            "InternalIP" => "InternalIP",
                            "ExternalIP" => "ExternalIP",
                            "Hostname" => "Hostname",
                            _ => "Address",
                        };
                        (label, addr.address.clone(), true)
                    })
                    .collect(),
                theme,
            ),
            theme,
        ))
    };

    let conditions_section = if node.conditions.is_empty() {
        None
    } else {
        Some(crate::views::render_drawer_section(
            "CONDITIONS",
            div()
                .w_full()
                .flex()
                .flex_col()
                .gap_2()
                .children(node.conditions.iter().map(|cond| {
                    div()
                        .w_full()
                        .p_2p5()
                        .rounded(theme.style.radius_control)
                        .bg(theme.surface.bg_panel)
                        .border_1()
                        .border_color(theme.border.default)
                        .flex()
                        .flex_col()
                        .gap_1()
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .justify_between()
                                .child(
                                    apply_text_step(div(), theme.typography.size_xs)
                                        .font_weight(FontWeight::SEMIBOLD)
                                        .text_color(theme.text.primary)
                                        .child(cond.type_.clone()),
                                )
                                .child(
                                    apply_text_step(div(), theme.typography.size_2xs)
                                        .text_color(theme.text.secondary)
                                        .child(cond.status.clone()),
                                ),
                        )
                        .when_some(cond.message.as_ref(), |el, msg| {
                            el.child(
                                apply_text_step(div(), theme.typography.size_2xs)
                                    .text_color(theme.text.muted)
                                    .child(msg.clone()),
                            )
                        })
                }))
                .into_any_element(),
            theme,
        ))
    };

    let taints_section = if node.taints.is_empty() {
        None
    } else {
        let mono = theme.typography.mono_font(FontWeight::NORMAL);
        Some(crate::views::render_drawer_section(
            "TAINTS",
            div()
                .flex()
                .flex_wrap()
                .gap_1p5()
                .children(node.taints.iter().map(|t| {
                    let val = t.value.as_deref().unwrap_or("");
                    let label = if val.is_empty() {
                        format!("{}:{}", t.key, t.effect)
                    } else {
                        format!("{}={}:{}", t.key, val, t.effect)
                    };
                    apply_text_step(div(), theme.typography.size_2xs)
                        .px_2()
                        .py_0p5()
                        .rounded(theme.style.radius_control)
                        .bg(theme.surface.control_bg)
                        .border_1()
                        .border_color(theme.border.default)
                        .font(mono.clone())
                        .text_color(theme.text.primary)
                        .child(label)
                }))
                .into_any_element(),
            theme,
        ))
    };

    div()
        .id("node-drawer-overview")
        .flex_1()
        .min_h_0()
        .overflow_y_scroll()
        .p_4()
        .flex()
        .flex_col()
        .gap_4()
        .child(crate::views::render_drawer_section(
            "RESOURCE UTILIZATION",
            utilization,
            theme,
        ))
        .child(crate::views::render_drawer_section(
            "SUMMARY",
            crate::views::render_drawer_key_values(summary_rows, theme),
            theme,
        ))
        .when_some(addresses_section, |el, sec| el.child(sec))
        .when_some(system_section, |el, sec| el.child(sec))
        .when_some(conditions_section, |el, sec| el.child(sec))
        .when_some(taints_section, |el, sec| el.child(sec))
        .child(crate::views::render_drawer_kv_badges(
            "LABELS",
            &node.labels_map,
            "No labels",
            theme,
        ))
        .child(crate::views::render_drawer_kv_badges(
            "ANNOTATIONS",
            &node.annotations,
            "No annotations",
            theme,
        ))
        .into_any_element()
}

pub fn render_node_drawer_events(
    events: &[orbit_core::kubernetes::models::EventInfo],
    loading: bool,
    node: &NodeInfo,
    theme: &OrbitTheme,
) -> AnyElement {
    let matched = filter_events_for_node(events, node);
    div()
        .id("node-drawer-events")
        .flex_1()
        .min_h_0()
        .overflow_y_scroll()
        .p_4()
        .child(crate::views::render_drawer_events_list(
            &matched,
            loading,
            "No events found for this node.",
            theme,
        ))
        .into_any_element()
}

#[cfg(test)]
mod tests {
    use super::*;
    use gpui::WindowAppearance;

    use crate::components::data_table::{ColumnAlign, SortDirection};
    use crate::theme::{AccentColor, SemanticColorsPreference, ThemeMode, UiStyle};

    #[allow(clippy::too_many_arguments)]
    fn make_node(
        name: &str,
        status: &str,
        role: &str,
        version: &str,
        is_cordoned: bool,
        cpu_pct: f64,
        cpu_used: &str,
        cpu_total: &str,
        mem_pct: f64,
        mem_used: &str,
        mem_total: &str,
        pods_count: i32,
        pods_limit: i32,
        uptime: &str,
    ) -> NodeInfo {
        NodeInfo {
            name: name.to_string(),
            status: status.to_string(),
            role: role.to_string(),
            version: version.to_string(),
            cpu_pct,
            cpu_used: cpu_used.to_string(),
            cpu_total: cpu_total.to_string(),
            mem_pct,
            mem_used: mem_used.to_string(),
            mem_total: mem_total.to_string(),
            pods_count,
            pods_limit,
            uptime: uptime.to_string(),
            created_at: None,
            labels: Vec::new(),
            labels_map: Default::default(),
            annotations: Default::default(),
            is_cordoned,
            node_info: None,
            addresses: Vec::new(),
            conditions: Vec::new(),
            taints: Vec::new(),
            capacity: None,
            allocatable: None,
            images_count: 0,
        }
    }

    #[test]
    fn test_node_columns_cover_required_fields_and_numeric_alignment() {
        let cols = node_columns();
        let ids: Vec<&str> = cols.iter().map(|c| c.id.as_ref()).collect();
        assert_eq!(
            ids,
            vec!["name", "status", "role", "version", "cpu", "memory", "pods", "age"]
        );

        for numeric_id in ["cpu", "memory", "pods", "age"] {
            let col = cols
                .iter()
                .find(|c| c.id.as_ref() == numeric_id)
                .expect("numeric column");
            assert_eq!(col.align, ColumnAlign::Right, "{numeric_id} right aligned");
            assert!(col.monospace, "{numeric_id} monospace");
        }
    }

    #[test]
    fn test_sort_nodes_handles_status_role_cpu_memory_pods_and_age() {
        let cp = make_node(
            "node-a",
            "Ready",
            "control-plane",
            "v1.30.2",
            false,
            15.0,
            "1.2",
            "8",
            40.0,
            "12.8",
            "32",
            18,
            110,
            "5d",
        );
        let worker_cordoned = make_node(
            "node-b",
            "Ready",
            "worker",
            "v1.29.8",
            true,
            72.5,
            "5.8",
            "8",
            80.0,
            "25.6",
            "32",
            45,
            110,
            "45s",
        );
        let worker_not_ready = make_node(
            "node-c",
            "NotReady",
            "worker",
            "v1.31.0",
            false,
            5.0,
            "0.4",
            "8",
            10.0,
            "3.2",
            "32",
            2,
            110,
            "Unknown",
        );
        let nodes = vec![worker_cordoned.clone(), worker_not_ready.clone(), cp.clone()];

        let by_cpu_desc = sort_nodes(
            &nodes,
            Some(&SortState {
                column_id: "cpu".to_string(),
                direction: SortDirection::Desc,
            }),
        );
        assert_eq!(
            by_cpu_desc.iter().map(|n| n.name.as_str()).collect::<Vec<_>>(),
            vec!["node-b", "node-a", "node-c"]
        );

        let by_mem_asc = sort_nodes(
            &nodes,
            Some(&SortState {
                column_id: "memory".to_string(),
                direction: SortDirection::Asc,
            }),
        );
        assert_eq!(
            by_mem_asc.iter().map(|n| n.name.as_str()).collect::<Vec<_>>(),
            vec!["node-c", "node-a", "node-b"]
        );

        let by_pods_desc = sort_nodes(
            &nodes,
            Some(&SortState {
                column_id: "pods".to_string(),
                direction: SortDirection::Desc,
            }),
        );
        assert_eq!(
            by_pods_desc.iter().map(|n| n.name.as_str()).collect::<Vec<_>>(),
            vec!["node-b", "node-a", "node-c"]
        );

        let by_age_asc = sort_nodes(
            &nodes,
            Some(&SortState {
                column_id: "age".to_string(),
                direction: SortDirection::Asc,
            }),
        );
        assert_eq!(
            by_age_asc.iter().map(|n| n.name.as_str()).collect::<Vec<_>>(),
            vec!["node-b", "node-a", "node-c"]
        );

        let by_status_asc = sort_nodes(
            &nodes,
            Some(&SortState {
                column_id: "status".to_string(),
                direction: SortDirection::Asc,
            }),
        );
        assert_eq!(
            by_status_asc.iter().map(|n| n.name.as_str()).collect::<Vec<_>>(),
            vec!["node-c", "node-a", "node-b"]
        );

        let by_role_asc = sort_nodes(
            &nodes,
            Some(&SortState {
                column_id: "role".to_string(),
                direction: SortDirection::Asc,
            }),
        );
        assert_eq!(
            by_role_asc.iter().map(|n| n.name.as_str()).collect::<Vec<_>>(),
            vec!["node-a", "node-b", "node-c"]
        );
    }

    #[test]
    fn test_compute_node_summary_and_cards_cover_total_ready_cordoned_and_not_ready() {
        let nodes = vec![
            make_node(
                "node-1",
                "Ready",
                "control-plane",
                "v1.30.2",
                false,
                25.0,
                "2",
                "8",
                50.0,
                "16",
                "32",
                20,
                110,
                "5d",
            ),
            make_node(
                "node-2",
                "Ready",
                "worker",
                "v1.30.2",
                true,
                50.0,
                "4",
                "8",
                25.0,
                "8",
                "32",
                30,
                110,
                "3d",
            ),
            make_node(
                "node-3",
                "NotReady",
                "worker",
                "v1.30.2",
                false,
                0.0,
                "0",
                "4",
                0.0,
                "0",
                "16",
                0,
                110,
                "1d",
            ),
        ];

        let summary = compute_node_summary(&nodes);
        assert_eq!(summary.total, 3);
        assert_eq!(summary.ready, 2);
        assert_eq!(summary.not_ready, 1);
        assert_eq!(summary.cordoned, 1);
        assert_eq!(summary.control_plane, 1);
        assert_eq!(summary.workers, 2);
        assert_eq!(summary.cpu_used_cores, 6.0);
        assert_eq!(summary.cpu_total_cores, 20.0);
        assert_eq!(summary.cpu_pct, 30.0);
        assert_eq!(summary.mem_used_gib, 24.0);
        assert_eq!(summary.mem_total_gib, 80.0);
        assert_eq!(summary.mem_pct, 30.0);

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
            let cards = node_summary_cards(&nodes, &theme);
            assert_eq!(cards[0].title, "Total Nodes");
            assert_eq!(cards[0].primary_value, "3");
            assert_eq!(cards[0].subtitle, "1 control-plane • 2 worker");
            assert_eq!(cards[0].accent_color, theme.k8s_color(K8sResourceKind::Node));

            assert_eq!(cards[1].title, "Ready Nodes");
            assert_eq!(cards[1].primary_value, "2");
            assert_eq!(cards[1].subtitle, "66% online");
            assert_eq!(cards[1].accent_color, theme.semantic.success.solid);

            assert_eq!(cards[2].title, "Cordoned / NotReady");
            assert_eq!(cards[2].primary_value, "2");
            assert_eq!(cards[2].subtitle, "1 cordoned • 1 not ready");
            assert_eq!(cards[2].accent_color, theme.semantic.danger.solid);

            assert_eq!(cards[3].title, "Resource Allocation");
            assert_eq!(cards[3].primary_value, "30.0% / 30.0%");
            assert_eq!(cards[3].subtitle, "6 / 20C • 24 / 80G");
            assert_eq!(cards[3].accent_color, theme.accent.accent);
        }
    }

    #[test]
    fn test_nodes_view_empty_and_error_presentation_modes() {
        let mut state = NodesState::default();
        assert_eq!(nodes_empty_message(&state), "No nodes found in this cluster.");
        assert_eq!(nodes_presentation_mode(&state), NodesPresentationMode::Table);

        state.status = NodesLoadStatus::Error;
        state.error_message = Some("forbidden".to_string());
        assert_eq!(
            nodes_presentation_mode(&state),
            NodesPresentationMode::ErrorCard
        );

        state.nodes.push(make_node(
            "node-1",
            "Ready",
            "worker",
            "v1.30.2",
            false,
            10.0,
            "1",
            "8",
            20.0,
            "4",
            "16",
            5,
            110,
            "2d",
        ));
        assert_eq!(nodes_presentation_mode(&state), NodesPresentationMode::Table);
    }

    #[test]
    fn test_filter_events_for_node_matches_object_name_or_message() {
        let node = make_node(
            "node-1",
            "Ready",
            "worker",
            "v1.30.2",
            false,
            10.0,
            "1",
            "8",
            20.0,
            "4",
            "16",
            5,
            110,
            "2d",
        );
        let events = vec![
            orbit_core::kubernetes::models::EventInfo {
                uid: "1".to_string(),
                time: "1m".to_string(),
                r#type: "Normal".to_string(),
                reason: "NodeReady".to_string(),
                object_name: "node-1".to_string(),
                object_kind: "Node".to_string(),
                message: "Node node-1 status is now: NodeReady".to_string(),
                namespace: String::new(),
                source: "kubelet".to_string(),
                first_seen: String::new(),
                last_seen: String::new(),
                count: 1,
                labels: Default::default(),
                annotations: Default::default(),
            },
            orbit_core::kubernetes::models::EventInfo {
                uid: "2".to_string(),
                time: "1m".to_string(),
                r#type: "Normal".to_string(),
                reason: "Scheduled".to_string(),
                object_name: "web-1".to_string(),
                object_kind: "Pod".to_string(),
                message: "Successfully assigned default/web-1 to node-1".to_string(),
                namespace: "default".to_string(),
                source: "scheduler".to_string(),
                first_seen: String::new(),
                last_seen: String::new(),
                count: 1,
                labels: Default::default(),
                annotations: Default::default(),
            },
            orbit_core::kubernetes::models::EventInfo {
                uid: "3".to_string(),
                time: "1m".to_string(),
                r#type: "Normal".to_string(),
                reason: "Scheduled".to_string(),
                object_name: "web-2".to_string(),
                object_kind: "Pod".to_string(),
                message: "Successfully assigned default/web-2 to node-2".to_string(),
                namespace: "default".to_string(),
                source: "scheduler".to_string(),
                first_seen: String::new(),
                last_seen: String::new(),
                count: 1,
                labels: Default::default(),
                annotations: Default::default(),
            },
        ];

        let filtered = filter_events_for_node(&events, &node);
        assert_eq!(filtered.len(), 2);
        assert_eq!(filtered[0].uid, "1");
        assert_eq!(filtered[1].uid, "2");
    }
}
