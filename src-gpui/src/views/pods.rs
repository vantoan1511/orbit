use std::cmp::Ordering;
use std::rc::Rc;

use gpui::{
    AnyElement, App, ClickEvent, ElementId, FontWeight, MouseDownEvent, Rgba, SharedString, Window,
    div, prelude::*, px,
};
use lucide_icons::Icon;
use orbit_core::kubernetes::models::{EventInfo, PodInfo};

use crate::components::data_table::{
    DataTableCallbacks, DataTableEnhancementCallbacks, SortState, TableColumn,
    TableFilterChipGroup, TableFilterChipOption, TableFilterState, render_enhanced_data_table,
    sort_rows_stable,
};
use crate::components::{
    ContextMenuEntry, ContextMenuItem, StatusBadgeVariant, render_status_badge_from_str,
};
use crate::navigation::WorkspaceView;
use crate::state::{NamespaceFilterState, PodsLoadStatus, PodsState};
use crate::theme::{K8sResourceKind, OrbitTheme, apply_text_step};
pub use crate::views::{
    nonzero_metric_text_color, numeric_metric_text_color, parse_age_seconds, render_action_button,
    render_ratio_metric_cell, render_resource_error_banner, render_resource_error_state,
    render_resource_view_header, render_truncated_cell, table_numeric_font,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PodsPresentationMode {
    Table,
    ErrorCard,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct PodSummary {
    pub total: usize,
    pub running: usize,
    pub pending: usize,
    pub failed: usize,
    pub succeeded: usize,
    pub other: usize,
}

#[derive(Debug, Clone)]
pub struct PodMetricCard {
    pub title: &'static str,
    pub primary_value: String,
    pub subtitle: String,
    pub accent_color: Rgba,
}

pub fn pod_columns() -> Vec<TableColumn> {
    vec![
        TableColumn::new("name", "Name")
            .flex_width(2.0, px(160.0))
            .locked(true),
        TableColumn::new("namespace", "Namespace").flex_width(1.0, px(105.0)),
        TableColumn::new("status", "Status").fixed_width(px(125.0)),
        TableColumn::new("ready", "Ready")
            .fixed_width(px(76.0))
            .right_numeric(),
        TableColumn::new("restarts", "Restarts")
            .fixed_width(px(84.0))
            .right_numeric(),
        TableColumn::new("node", "Node")
            .flex_width(1.2, px(120.0))
            .monospace(true),
        TableColumn::new("ip", "IP")
            .fixed_width(px(116.0))
            .monospace(true),
        TableColumn::new("cpu", "CPU")
            .fixed_width(px(86.0))
            .right_numeric(),
        TableColumn::new("memory", "Memory")
            .fixed_width(px(90.0))
            .right_numeric(),
        TableColumn::new("age", "Age")
            .fixed_width(px(76.0))
            .right_numeric(),
    ]
}

pub fn pod_ready_counts(pod: &PodInfo) -> (usize, usize) {
    let total = pod.containers.len();
    let ready = pod
        .containers
        .iter()
        .filter(|c| c.ready.eq_ignore_ascii_case("true"))
        .count();
    (ready, total)
}

pub fn format_pod_ready(pod: &PodInfo) -> String {
    let (ready, total) = pod_ready_counts(pod);
    if total == 0 {
        "-".to_string()
    } else {
        format!("{ready}/{total}")
    }
}

fn compare_pods_by_col(a: &PodInfo, b: &PodInfo, col_id: &str) -> Ordering {
    let tie_break = || {
        a.name
            .to_ascii_lowercase()
            .cmp(&b.name.to_ascii_lowercase())
            .then_with(|| {
                a.namespace
                    .to_ascii_lowercase()
                    .cmp(&b.namespace.to_ascii_lowercase())
            })
    };

    match col_id {
        "name" => tie_break(),
        "namespace" => a
            .namespace
            .to_ascii_lowercase()
            .cmp(&b.namespace.to_ascii_lowercase())
            .then_with(|| {
                a.name
                    .to_ascii_lowercase()
                    .cmp(&b.name.to_ascii_lowercase())
            }),
        "status" => a
            .status
            .to_ascii_lowercase()
            .cmp(&b.status.to_ascii_lowercase())
            .then_with(tie_break),
        "ready" => pod_ready_counts(a)
            .cmp(&pod_ready_counts(b))
            .then_with(tie_break),
        "restarts" => a.restarts.cmp(&b.restarts).then_with(tie_break),
        "node" => a
            .node
            .as_deref()
            .unwrap_or("-")
            .to_ascii_lowercase()
            .cmp(&b.node.as_deref().unwrap_or("-").to_ascii_lowercase())
            .then_with(tie_break),
        "ip" => {
            a.ip.as_deref()
                .unwrap_or("-")
                .to_ascii_lowercase()
                .cmp(&b.ip.as_deref().unwrap_or("-").to_ascii_lowercase())
                .then_with(tie_break)
        }
        "cpu" => a
            .cpu_cores
            .unwrap_or(-1.0)
            .partial_cmp(&b.cpu_cores.unwrap_or(-1.0))
            .unwrap_or(Ordering::Equal)
            .then_with(tie_break),
        "memory" => a
            .memory_bytes
            .unwrap_or(-1.0)
            .partial_cmp(&b.memory_bytes.unwrap_or(-1.0))
            .unwrap_or(Ordering::Equal)
            .then_with(tie_break),
        "age" => parse_age_seconds(&a.age)
            .cmp(&parse_age_seconds(&b.age))
            .then_with(tie_break),
        _ => tie_break(),
    }
}

pub fn filter_and_sort_pods<'a>(
    pods: &'a [PodInfo],
    namespaces: &NamespaceFilterState,
    sort: Option<&SortState>,
) -> Vec<&'a PodInfo> {
    filter_and_sort_pods_with_filters(pods, namespaces, &TableFilterState::default(), sort)
}

pub fn filter_and_sort_pods_with_filters<'a>(
    pods: &'a [PodInfo],
    namespaces: &NamespaceFilterState,
    filters: &TableFilterState,
    sort: Option<&SortState>,
) -> Vec<&'a PodInfo> {
    let query = filters.search_query.trim().to_ascii_lowercase();
    let status_filter = filters
        .extra_filters
        .get("status")
        .map(|s| s.trim().to_ascii_lowercase())
        .filter(|s| !s.is_empty() && s != "all");

    let mut rows: Vec<&'a PodInfo> = pods
        .iter()
        .filter(|pod| namespaces.matches_resource_namespace(Some(&pod.namespace)))
        .filter(|pod| {
            if let Some(status) = status_filter.as_deref() {
                let pod_status = pod.status.to_ascii_lowercase();
                match status {
                    "running" => pod_status == "running",
                    "pending" => pod_status == "pending",
                    "failed" => matches!(
                        pod_status.as_str(),
                        "failed" | "error" | "crashloopbackoff" | "imagepullbackoff" | "errimagepull"
                    ),
                    "succeeded" => matches!(pod_status.as_str(), "succeeded" | "completed"),
                    other => pod_status == other,
                }
            } else {
                true
            }
        })
        .filter(|pod| {
            if query.is_empty() {
                return true;
            }
            pod.name.to_ascii_lowercase().contains(&query)
                || pod.namespace.to_ascii_lowercase().contains(&query)
                || pod.status.to_ascii_lowercase().contains(&query)
                || pod
                    .node
                    .as_deref()
                    .is_some_and(|n| n.to_ascii_lowercase().contains(&query))
                || pod
                    .ip
                    .as_deref()
                    .is_some_and(|ip| ip.to_ascii_lowercase().contains(&query))
        })
        .collect();
    sort_rows_stable(&mut rows, sort, |a, b, col_id| {
        compare_pods_by_col(a, b, col_id)
    });
    rows
}

pub fn compute_pod_summary(pods: &[&PodInfo]) -> PodSummary {
    let mut summary = PodSummary {
        total: pods.len(),
        ..Default::default()
    };

    for pod in pods {
        match pod.status.to_ascii_lowercase().as_str() {
            "running" => summary.running += 1,
            "pending" => summary.pending += 1,
            "failed" | "error" | "crashloopbackoff" | "imagepullbackoff" | "errimagepull" => {
                summary.failed += 1
            }
            "succeeded" | "completed" => summary.succeeded += 1,
            _ => summary.other += 1,
        }
    }

    summary
}

pub fn pod_summary_cards(
    pods: &[&PodInfo],
    namespaces: &NamespaceFilterState,
    theme: &OrbitTheme,
) -> Vec<PodMetricCard> {
    let summary = compute_pod_summary(pods);
    let pct = |count: usize, total: usize| -> f64 {
        if total == 0 {
            0.0
        } else {
            (count as f64 / total as f64) * 100.0
        }
    };

    let total_subtitle = if !namespaces.is_all_namespaces() {
        format!("Scope: {}", namespaces.summary_label())
    } else if summary.succeeded > 0 || summary.other > 0 {
        format!("{} succeeded · {} other", summary.succeeded, summary.other)
    } else {
        "All namespaces".to_string()
    };

    let failed_subtitle = if summary.failed > 0 {
        "Requires attention".to_string()
    } else {
        format!("{:.0}% of pods", pct(summary.failed, summary.total))
    };

    vec![
        PodMetricCard {
            title: "Total Pods",
            primary_value: summary.total.to_string(),
            subtitle: total_subtitle,
            accent_color: theme.k8s_color(K8sResourceKind::Pod),
        },
        PodMetricCard {
            title: "Running",
            primary_value: summary.running.to_string(),
            subtitle: format!("{:.0}% of pods", pct(summary.running, summary.total)),
            accent_color: theme.semantic.success.solid,
        },
        PodMetricCard {
            title: "Pending",
            primary_value: summary.pending.to_string(),
            subtitle: format!("{:.0}% of pods", pct(summary.pending, summary.total)),
            accent_color: theme.semantic.warning.solid,
        },
        PodMetricCard {
            title: "Failed",
            primary_value: summary.failed.to_string(),
            subtitle: failed_subtitle,
            accent_color: theme.semantic.danger.solid,
        },
    ]
}

pub fn pods_empty_message(state: &PodsState) -> &'static str {
    if state.pods.is_empty() {
        "No pods found in this cluster."
    } else {
        "No pods found in the selected namespace(s)."
    }
}

pub fn pods_presentation_mode(state: &PodsState) -> PodsPresentationMode {
    if state.status == PodsLoadStatus::Error && state.pods.is_empty() {
        PodsPresentationMode::ErrorCard
    } else {
        PodsPresentationMode::Table
    }
}

fn render_pod_cell(
    row_idx: usize,
    pod: &PodInfo,
    col: &TableColumn,
    theme: &OrbitTheme,
) -> AnyElement {
    match col.id.as_ref() {
        "name" => render_truncated_cell(
            ElementId::NamedInteger(SharedString::from("pod-name"), row_idx as u64),
            pod.name.clone(),
            Some(SharedString::from(pod.name.clone())),
            theme.text.primary,
            FontWeight::MEDIUM,
            false,
            theme,
        ),
        "namespace" => render_truncated_cell(
            ElementId::NamedInteger(SharedString::from("pod-ns"), row_idx as u64),
            pod.namespace.clone(),
            None,
            theme.text.secondary,
            FontWeight::NORMAL,
            false,
            theme,
        ),
        "status" => render_status_badge_from_str(&pod.status, StatusBadgeVariant::Inline, theme),
        "ready" => {
            let (ready, total) = pod_ready_counts(pod);
            if total == 0 {
                apply_text_step(div(), theme.typography.size_xs)
                    .font(table_numeric_font(theme))
                    .text_color(theme.text.muted)
                    .child("-")
                    .into_any_element()
            } else {
                render_ratio_metric_cell(ready.to_string(), total.to_string(), theme)
            }
        }
        "restarts" => apply_text_step(div(), theme.typography.size_xs)
            .font(table_numeric_font(theme))
            .text_color(nonzero_metric_text_color(pod.restarts, theme))
            .child(pod.restarts.to_string())
            .into_any_element(),
        "node" => render_truncated_cell(
            ElementId::NamedInteger(SharedString::from("pod-node"), row_idx as u64),
            pod.node.as_deref().unwrap_or("-").to_string(),
            pod.node.clone().map(SharedString::from),
            if pod.node.is_some() {
                theme.text.secondary
            } else {
                theme.text.muted
            },
            FontWeight::NORMAL,
            true,
            theme,
        ),
        "ip" => render_truncated_cell(
            ElementId::NamedInteger(SharedString::from("pod-ip"), row_idx as u64),
            pod.ip.as_deref().unwrap_or("-").to_string(),
            None,
            if pod.ip.is_some() {
                theme.text.secondary
            } else {
                theme.text.muted
            },
            FontWeight::NORMAL,
            true,
            theme,
        ),
        "cpu" => apply_text_step(div(), theme.typography.size_xs)
            .font(theme.typography.mono_font(FontWeight::NORMAL))
            .text_color(if pod.cpu.is_some() {
                theme.text.primary
            } else {
                theme.text.muted
            })
            .child(pod.cpu.as_deref().unwrap_or("-").to_string())
            .into_any_element(),
        "memory" => apply_text_step(div(), theme.typography.size_xs)
            .font(theme.typography.mono_font(FontWeight::NORMAL))
            .text_color(if pod.memory.is_some() {
                theme.text.primary
            } else {
                theme.text.muted
            })
            .child(pod.memory.as_deref().unwrap_or("-").to_string())
            .into_any_element(),
        "age" => apply_text_step(div(), theme.typography.size_xs)
            .font(theme.typography.mono_font(FontWeight::NORMAL))
            .text_color(theme.text.muted)
            .child(pod.age.clone())
            .into_any_element(),
        _ => div().into_any_element(),
    }
}

fn render_summary_cards_row(
    pods: &[&PodInfo],
    namespaces: &NamespaceFilterState,
    theme: &OrbitTheme,
) -> AnyElement {
    let cards = pod_summary_cards(pods, namespaces, theme);
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
                        .child(
                            div()
                                .w(px(6.0))
                                .h(px(6.0))
                                .rounded_full()
                                .bg(card.accent_color),
                        ),
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

#[allow(clippy::too_many_arguments)]
pub fn build_pod_context_menu_entries(
    pod: &PodInfo,
    on_copy_name: impl 'static + Fn(&mut Window, &mut App),
    on_filter_namespace: impl 'static + Fn(&mut Window, &mut App),
    on_refresh: impl 'static + Fn(&mut Window, &mut App),
    on_view_details: impl 'static + Fn(&mut Window, &mut App),
    on_view_yaml: impl 'static + Fn(&mut Window, &mut App),
    on_view_logs: impl 'static + Fn(&mut Window, &mut App),
    on_port_forward: impl 'static + Fn(&mut Window, &mut App),
    on_restart_pod: impl 'static + Fn(&mut Window, &mut App),
    on_delete: impl 'static + Fn(&mut Window, &mut App),
) -> Vec<ContextMenuEntry> {
    vec![
        ContextMenuEntry::Item(
            ContextMenuItem::new("copy-name", "Copy Name")
                .icon(Icon::Copy)
                .on_select(on_copy_name),
        ),
        ContextMenuEntry::Item(
            ContextMenuItem::new(
                "filter-namespace",
                format!("Filter by \"{}\"", pod.namespace),
            )
            .icon(Icon::Filter)
            .on_select(on_filter_namespace),
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
        ContextMenuEntry::Item(
            ContextMenuItem::new("view-logs", "View Logs")
                .icon(Icon::FileText)
                .on_select(on_view_logs),
        ),
        ContextMenuEntry::Item(
            ContextMenuItem::new("port-forward", "Port Forward...")
                .icon(Icon::Cable)
                .on_select(on_port_forward),
        ),
        ContextMenuEntry::Separator,
        ContextMenuEntry::Item(
            ContextMenuItem::new("restart-pod", "Restart Pod")
                .icon(Icon::RotateCw)
                .on_select(on_restart_pod),
        ),
        ContextMenuEntry::Separator,
        ContextMenuEntry::Item(
            ContextMenuItem::new("delete-pod", "Delete Pod")
                .icon(Icon::Trash2)
                .danger(true)
                .on_select(on_delete),
        ),
    ]
}

#[allow(clippy::too_many_arguments)]
pub fn render_pods_view(
    state: &PodsState,
    namespaces: &NamespaceFilterState,
    theme: &OrbitTheme,
    on_sort: impl 'static + Fn(SharedString, &ClickEvent, &mut Window, &mut App),
    on_row_click: impl 'static + Fn(usize, SharedString, &ClickEvent, &mut Window, &mut App),
    on_row_quick_yaml: impl 'static + Fn(usize, SharedString, &ClickEvent, &mut Window, &mut App),
    on_row_context_menu: impl 'static
    + Fn(SharedString, &PodInfo, &MouseDownEvent, &mut Window, &mut App),
    on_refresh: impl 'static + Fn(&mut Window, &mut App),
    enhancements: Option<DataTableEnhancementCallbacks>,
) -> AnyElement {
    let columns = pod_columns();
    let filtered_refs = filter_and_sort_pods_with_filters(
        &state.pods,
        namespaces,
        &state.table.filters,
        state.table.sort.as_ref(),
    );
    let summary_row = div()
        .w_full()
        .px_4()
        .pb_3()
        .flex_shrink_0()
        .child(render_summary_cards_row(&filtered_refs, namespaces, theme));
    let filtered_rows: Rc<Vec<PodInfo>> = Rc::new(filtered_refs.into_iter().cloned().collect());
    let total_count = state.pods.len();
    let visible_count = filtered_rows.len();
    let is_loading = state.status == PodsLoadStatus::Loading;
    let on_refresh_rc: super::ActionCallback = Rc::new(on_refresh);

    let count_label = if namespaces.is_all_namespaces() && visible_count == total_count {
        format!("{total_count} pods")
    } else {
        format!("{visible_count} of {total_count} pods")
    };

    let header = render_resource_view_header(
        WorkspaceView::Pods.label(),
        WorkspaceView::Pods.description(),
        count_label,
        Some(namespaces),
        None,
        theme,
    );

    let error_banner = state.error_message.as_ref().map(|message| {
        render_resource_error_banner(
            "pods-retry-btn",
            format!("Failed to load pods: {message}"),
            is_loading,
            theme,
            on_refresh_rc.clone(),
        )
    });

    let filter_groups = [TableFilterChipGroup {
        key: "status",
        default_value: "All",
        options: vec![
            TableFilterChipOption { value: "All", label: "All" },
            TableFilterChipOption { value: "Running", label: "Running" },
            TableFilterChipOption { value: "Pending", label: "Pending" },
            TableFilterChipOption { value: "Failed", label: "Failed" },
            TableFilterChipOption { value: "Succeeded", label: "Succeeded" },
        ],
    }];

    let mode = pods_presentation_mode(state);
    let body = match mode {
        PodsPresentationMode::ErrorCard => {
            let message = state
                .error_message
                .clone()
                .unwrap_or_else(|| "Unknown error".to_string());
            render_resource_error_state(
                "pods-retry-btn",
                "Unable to Load Pods",
                message,
                is_loading,
                theme,
                on_refresh_rc,
            )
        }
        PodsPresentationMode::Table => {
            let mut table_state = state.table.clone();
            table_state.loading = is_loading && state.pods.is_empty();
            table_state.refreshing = is_loading && !state.pods.is_empty();
            table_state.empty_message = SharedString::from(pods_empty_message(state));

            let key_rows = filtered_rows.clone();
            let cell_rows = filtered_rows.clone();
            let ctx_rows = filtered_rows.clone();
            let has_table_filters = table_state.has_active_filters(&[("status", "All")]);
            let empty_action = crate::views::build_empty_state_action(
                has_table_filters,
                enhancements
                    .as_ref()
                    .map(|e| e.toolbar.on_clear_filters.clone()),
                on_refresh_rc,
            );
            let callbacks = DataTableCallbacks::new(
                move |idx| {
                    key_rows
                        .get(idx)
                        .map(|p| format!("{}/{}", p.namespace, p.name))
                        .unwrap_or_default()
                },
                move |idx, col, col_theme| {
                    cell_rows
                        .get(idx)
                        .map(|p| render_pod_cell(idx, p, col, col_theme))
                        .unwrap_or_else(|| div().into_any_element())
                },
            )
            .on_sort(on_sort)
            .on_row_click(on_row_click)
            .on_row_quick_yaml(on_row_quick_yaml)
            .on_empty_action(empty_action)
            .on_row_context_menu(move |idx, row_key, event, window, cx| {
                if let Some(pod) = ctx_rows.get(idx) {
                    on_row_context_menu(row_key, pod, event, window, cx);
                }
            });

            render_enhanced_data_table(
                "pods-table",
                "Filter pods...",
                &filter_groups,
                &columns,
                visible_count,
                &table_state,
                theme,
                callbacks,
                enhancements,
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
        .when(mode == PodsPresentationMode::Table, |el| {
            el.when_some(error_banner, |el, banner| el.child(banner))
        })
        .child(body)
        .into_any_element()
}

pub fn filter_events_for_pod<'a>(events: &'a [EventInfo], pod: &PodInfo) -> Vec<&'a EventInfo> {
    events
        .iter()
        .filter(|ev| {
            ev.namespace == pod.namespace
                && ((ev.object_kind == "Pod" && ev.object_name == pod.name)
                    || ev.message.contains(&pod.name))
        })
        .collect()
}

pub fn render_pod_drawer_overview(pod: &PodInfo, theme: &OrbitTheme) -> AnyElement {
    let summary = crate::views::render_drawer_key_values(
        vec![
            ("Name", pod.name.clone(), true),
            ("Namespace", pod.namespace.clone(), true),
            ("Status", pod.status.clone(), false),
            ("Ready", format_pod_ready(pod), true),
            ("Restarts", pod.restarts.to_string(), true),
            (
                "Node",
                pod.node.clone().unwrap_or_else(|| "-".to_string()),
                true,
            ),
            (
                "Pod IP",
                pod.ip.clone().unwrap_or_else(|| "-".to_string()),
                true,
            ),
            (
                "Node IP",
                pod.node_ip.clone().unwrap_or_else(|| "-".to_string()),
                true,
            ),
            (
                "Controlled By",
                pod.controlled_by.clone().unwrap_or_else(|| "-".to_string()),
                true,
            ),
            (
                "QoS Class",
                pod.qos_class.clone().unwrap_or_else(|| "-".to_string()),
                false,
            ),
            (
                "CPU",
                pod.cpu.clone().unwrap_or_else(|| "-".to_string()),
                true,
            ),
            (
                "Memory",
                pod.memory.clone().unwrap_or_else(|| "-".to_string()),
                true,
            ),
            ("Age", pod.age.clone(), true),
        ],
        theme,
    );

    let mono = theme.typography.mono_font(FontWeight::NORMAL);
    let containers_section = {
        let content = if pod.containers.is_empty() {
            apply_text_step(div(), theme.typography.size_xs)
                .text_color(theme.text.muted)
                .child("No containers reported.")
                .into_any_element()
        } else {
            div()
                .w_full()
                .flex()
                .flex_col()
                .gap_2()
                .children(pod.containers.iter().map(|c| {
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
                                .gap_2()
                                .child(
                                    apply_text_step(div(), theme.typography.size_xs)
                                        .font_weight(FontWeight::SEMIBOLD)
                                        .text_color(theme.text.primary)
                                        .child(c.name.clone()),
                                )
                                .child(render_status_badge_from_str(
                                    &c.status,
                                    StatusBadgeVariant::Inline,
                                    theme,
                                )),
                        )
                        .child(
                            apply_text_step(div(), theme.typography.size_2xs)
                                .font(mono.clone())
                                .text_color(theme.text.secondary)
                                .child(c.image.clone()),
                        )
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .gap_3()
                                .child(
                                    apply_text_step(div(), theme.typography.size_2xs)
                                        .text_color(theme.text.muted)
                                        .child(format!("Ready: {}", c.ready)),
                                )
                                .child(
                                    apply_text_step(div(), theme.typography.size_2xs)
                                        .text_color(theme.text.muted)
                                        .child(format!("Restarts: {}", c.restarts)),
                                )
                                .when_some(c.ports.clone(), |el, ports| {
                                    el.child(
                                        apply_text_step(div(), theme.typography.size_2xs)
                                            .font(mono.clone())
                                            .text_color(theme.text.muted)
                                            .child(format!("Ports: {ports}")),
                                    )
                                }),
                        )
                }))
                .into_any_element()
        };
        crate::views::render_drawer_section("CONTAINERS", content, theme)
    };

    div()
        .id("pod-drawer-overview")
        .flex_1()
        .min_h_0()
        .overflow_y_scroll()
        .p_4()
        .flex()
        .flex_col()
        .gap_4()
        .child(crate::views::render_drawer_section(
            "SUMMARY", summary, theme,
        ))
        .child(containers_section)
        .child(crate::views::render_drawer_kv_badges(
            "LABELS",
            &pod.labels,
            "No labels",
            theme,
        ))
        .child(crate::views::render_drawer_kv_badges(
            "ANNOTATIONS",
            &pod.annotations,
            "No annotations",
            theme,
        ))
        .into_any_element()
}

pub fn render_pod_drawer_events(
    events: &[EventInfo],
    loading: bool,
    pod: &PodInfo,
    theme: &OrbitTheme,
) -> AnyElement {
    let matched = filter_events_for_pod(events, pod);
    div()
        .id("pod-drawer-events")
        .flex_1()
        .min_h_0()
        .overflow_y_scroll()
        .p_4()
        .child(crate::views::render_drawer_events_list(
            &matched,
            loading,
            "No events found for this pod.",
            theme,
        ))
        .into_any_element()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::components::data_table::{ColumnAlign, SortDirection};
    use gpui::WindowAppearance;
    use orbit_core::config::OrbitConfig;
    use orbit_core::kubernetes::models::{NamespaceInfo, PodContainer};

    fn make_ns(name: &str) -> NamespaceInfo {
        NamespaceInfo {
            name: name.to_string(),
            status: "Active".to_string(),
            is_system: name == "kube-system" || name == "default",
            age: "10d".to_string(),
            labels: Default::default(),
            annotations: Default::default(),
            uid: format!("uid-{name}"),
            created: String::new(),
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn make_pod(
        name: &str,
        namespace: &str,
        status: &str,
        ready_count: usize,
        total_count: usize,
        restarts: i32,
        cpu_cores: Option<f64>,
        memory_bytes: Option<f64>,
        age: &str,
    ) -> PodInfo {
        let containers = (0..total_count)
            .map(|idx| PodContainer {
                name: format!("c-{idx}"),
                image: "nginx:1.25".to_string(),
                status: status.to_string(),
                ready: if idx < ready_count { "true" } else { "false" }.to_string(),
                restarts: if idx == 0 { restarts } else { 0 },
                ports: Some("80/TCP".to_string()),
            })
            .collect();

        PodInfo {
            name: name.to_string(),
            namespace: namespace.to_string(),
            status: status.to_string(),
            age: age.to_string(),
            cpu: cpu_cores.map(|c| format!("{c} cores")),
            memory: memory_bytes.map(|m| format!("{:.0}Mi", m / (1024.0 * 1024.0))),
            cpu_cores,
            memory_bytes,
            cpu_pct: None,
            memory_pct: None,
            node: Some("node-1".to_string()),
            restarts,
            images: vec!["nginx:1.25".to_string()],
            labels: Default::default(),
            annotations: Default::default(),
            ip: Some("10.0.0.10".to_string()),
            node_ip: Some("192.168.1.10".to_string()),
            controlled_by: Some("ReplicaSet/web".to_string()),
            qos_class: Some("Burstable".to_string()),
            containers,
        }
    }

    #[test]
    fn test_pod_columns_cover_required_fields_and_numeric_alignment() {
        let cols = pod_columns();
        let ids: Vec<&str> = cols.iter().map(|c| c.id.as_ref()).collect();
        assert_eq!(
            ids,
            vec![
                "name",
                "namespace",
                "status",
                "ready",
                "restarts",
                "node",
                "ip",
                "cpu",
                "memory",
                "age",
            ]
        );
        for id in ["ready", "restarts", "cpu", "memory", "age"] {
            let col = cols.iter().find(|c| c.id == id).unwrap();
            assert_eq!(
                col.align,
                ColumnAlign::Right,
                "column {id} should be right-aligned"
            );
            assert!(col.monospace, "column {id} should be monospace");
        }
        for id in ["name", "namespace", "status", "node", "ip"] {
            let col = cols.iter().find(|c| c.id == id).unwrap();
            assert_eq!(
                col.align,
                ColumnAlign::Left,
                "column {id} should be left-aligned"
            );
        }
        for id in ["node", "ip"] {
            let col = cols.iter().find(|c| c.id == id).unwrap();
            assert!(col.monospace, "column {id} should be monospace");
        }
    }

    #[test]
    fn test_pod_ready_counts_and_formatting() {
        let full = make_pod("a", "default", "Running", 2, 2, 0, None, None, "1m");
        assert_eq!(pod_ready_counts(&full), (2, 2));
        assert_eq!(format_pod_ready(&full), "2/2");

        let partial = make_pod("b", "default", "Pending", 1, 2, 0, None, None, "1m");
        assert_eq!(pod_ready_counts(&partial), (1, 2));
        assert_eq!(format_pod_ready(&partial), "1/2");

        let empty = make_pod("c", "default", "Pending", 0, 0, 0, None, None, "1m");
        assert_eq!(pod_ready_counts(&empty), (0, 0));
        assert_eq!(format_pod_ready(&empty), "-");
    }

    #[test]
    fn test_filter_and_sort_pods_by_namespace_and_columns() {
        let pods = vec![
            make_pod(
                "web-b",
                "prod",
                "Running",
                2,
                2,
                1,
                Some(0.5),
                Some(200.0),
                "2h",
            ),
            make_pod(
                "api-a",
                "prod",
                "Pending",
                1,
                2,
                4,
                Some(1.2),
                Some(500.0),
                "10m",
            ),
            make_pod(
                "db-0",
                "staging",
                "Failed",
                0,
                1,
                2,
                Some(0.1),
                Some(100.0),
                "3d",
            ),
        ];

        let mut ns_filter = NamespaceFilterState::default();
        ns_filter.begin_load_for_cluster("ctx-a");
        ns_filter.apply_loaded("ctx-a", 1, vec![make_ns("prod"), make_ns("staging")]);
        ns_filter.toggle_namespace("prod");

        let by_name = filter_and_sort_pods(
            &pods,
            &ns_filter,
            Some(&SortState {
                column_id: "name".to_string(),
                direction: SortDirection::Asc,
            }),
        );
        assert_eq!(
            by_name.iter().map(|p| p.name.as_str()).collect::<Vec<_>>(),
            vec!["api-a", "web-b"]
        );

        let by_restarts_desc = filter_and_sort_pods(
            &pods,
            &ns_filter,
            Some(&SortState {
                column_id: "restarts".to_string(),
                direction: SortDirection::Desc,
            }),
        );
        assert_eq!(by_restarts_desc[0].name, "api-a");

        let by_ready_asc = filter_and_sort_pods(
            &pods,
            &ns_filter,
            Some(&SortState {
                column_id: "ready".to_string(),
                direction: SortDirection::Asc,
            }),
        );
        assert_eq!(by_ready_asc[0].name, "api-a");

        let by_cpu_desc = filter_and_sort_pods(
            &pods,
            &ns_filter,
            Some(&SortState {
                column_id: "cpu".to_string(),
                direction: SortDirection::Desc,
            }),
        );
        assert_eq!(by_cpu_desc[0].name, "api-a");

        let by_mem_asc = filter_and_sort_pods(
            &pods,
            &ns_filter,
            Some(&SortState {
                column_id: "memory".to_string(),
                direction: SortDirection::Asc,
            }),
        );
        assert_eq!(by_mem_asc[0].name, "web-b");

        let by_age_asc = filter_and_sort_pods(
            &pods,
            &ns_filter,
            Some(&SortState {
                column_id: "age".to_string(),
                direction: SortDirection::Asc,
            }),
        );
        assert_eq!(by_age_asc[0].name, "api-a");

        let by_status_asc = filter_and_sort_pods(
            &pods,
            &ns_filter,
            Some(&SortState {
                column_id: "status".to_string(),
                direction: SortDirection::Asc,
            }),
        );
        assert_eq!(by_status_asc[0].name, "api-a");
    }

    #[test]
    fn test_pod_summary_cards_scoped_by_namespace() {
        let pods = vec![
            make_pod("p1", "prod", "Running", 1, 1, 0, None, None, "5m"),
            make_pod("p2", "prod", "Pending", 0, 1, 0, None, None, "2m"),
            make_pod("p3", "staging", "Failed", 0, 1, 3, None, None, "1h"),
        ];

        let mut ns_filter = NamespaceFilterState::default();
        ns_filter.begin_load_for_cluster("ctx-a");
        ns_filter.apply_loaded("ctx-a", 1, vec![make_ns("prod"), make_ns("staging")]);
        ns_filter.toggle_namespace("prod");

        let filtered = filter_and_sort_pods(&pods, &ns_filter, None);
        let theme = OrbitTheme::from_config(&OrbitConfig::default(), WindowAppearance::Dark);
        let cards = pod_summary_cards(&filtered, &ns_filter, &theme);

        assert_eq!(cards.len(), 4);
        assert_eq!(cards[0].title, "Total Pods");
        assert_eq!(cards[0].primary_value, "2");
        assert_eq!(cards[0].accent_color, theme.k8s_color(K8sResourceKind::Pod));
        assert_eq!(cards[1].title, "Running");
        assert_eq!(cards[1].primary_value, "1");
        assert_eq!(cards[1].subtitle, "50% of pods");
        assert_eq!(cards[1].accent_color, theme.semantic.success.solid);
        assert_eq!(cards[2].title, "Pending");
        assert_eq!(cards[2].primary_value, "1");
        assert_eq!(cards[2].accent_color, theme.semantic.warning.solid);
        assert_eq!(cards[3].title, "Failed");
        assert_eq!(cards[3].primary_value, "0");
        assert_eq!(cards[3].accent_color, theme.semantic.danger.solid);
    }

    #[test]
    fn test_pods_empty_message_and_presentation_mode() {
        let mut state = PodsState::default();
        assert_eq!(pods_empty_message(&state), "No pods found in this cluster.");
        assert_eq!(pods_presentation_mode(&state), PodsPresentationMode::Table);

        state.status = PodsLoadStatus::Error;
        state.error_message = Some("connection refused".to_string());
        assert_eq!(
            pods_presentation_mode(&state),
            PodsPresentationMode::ErrorCard
        );

        state.pods.push(make_pod(
            "api-0", "prod", "Running", 1, 1, 0, None, None, "1m",
        ));
        assert_eq!(
            pods_empty_message(&state),
            "No pods found in the selected namespace(s)."
        );
        assert_eq!(pods_presentation_mode(&state), PodsPresentationMode::Table);
    }

    #[test]
    fn test_filter_events_for_pod() {
        let pod = make_pod("api-0", "prod", "Running", 1, 1, 0, None, None, "5m");
        let events = vec![
            EventInfo {
                uid: "1".to_string(),
                name: String::new(),
                r#type: "Normal".to_string(),
                reason: "Scheduled".to_string(),
                message: "Successfully assigned prod/api-0 to node-1".to_string(),
                object_kind: "Pod".to_string(),
                object_name: "api-0".to_string(),
                namespace: "prod".to_string(),
                count: 1,
                time: "1m".to_string(),
                source: "default-scheduler".to_string(),
                first_seen: String::new(),
                last_seen: String::new(),
                labels: Default::default(),
                annotations: Default::default(),
            },
            EventInfo {
                uid: "2".to_string(),
                name: String::new(),
                r#type: "Normal".to_string(),
                reason: "Created".to_string(),
                message: "Created pod api-0".to_string(),
                object_kind: "ReplicaSet".to_string(),
                object_name: "api".to_string(),
                namespace: "prod".to_string(),
                count: 1,
                time: "1m".to_string(),
                source: "replicaset-controller".to_string(),
                first_seen: String::new(),
                last_seen: String::new(),
                labels: Default::default(),
                annotations: Default::default(),
            },
            EventInfo {
                uid: "3".to_string(),
                name: String::new(),
                r#type: "Warning".to_string(),
                reason: "Failed".to_string(),
                message: "Error in other namespace".to_string(),
                object_kind: "Pod".to_string(),
                object_name: "api-0".to_string(),
                namespace: "staging".to_string(),
                count: 1,
                time: "1m".to_string(),
                source: "kubelet".to_string(),
                first_seen: String::new(),
                last_seen: String::new(),
                labels: Default::default(),
                annotations: Default::default(),
            },
        ];

        let matched = filter_events_for_pod(&events, &pod);
        assert_eq!(matched.len(), 2);
        assert_eq!(matched[0].uid, "1");
        assert_eq!(matched[1].uid, "2");
    }

    #[test]
    fn test_filter_and_sort_pods_applies_search_query_and_status_extra_filter() {
        let cols = pod_columns();
        let name_col = cols.iter().find(|c| c.id.as_ref() == "name").expect("name col");
        assert!(name_col.locked);

        let pods = vec![
            make_pod("api-0", "default", "Running", 1, 1, 0, None, None, "1m"),
            make_pod("worker-0", "default", "Pending", 0, 1, 0, None, None, "2m"),
            make_pod("api-1", "default", "Failed", 0, 1, 3, None, None, "3m"),
        ];
        let ns = NamespaceFilterState::default();
        let mut filters = crate::components::TableFilterState {
            search_query: "api".to_string(),
            ..Default::default()
        };
        filters
            .extra_filters
            .insert("status".to_string(), "Failed".to_string());

        let matched = filter_and_sort_pods_with_filters(&pods, &ns, &filters, None);
        assert_eq!(matched.len(), 1);
        assert_eq!(matched[0].name, "api-1");
    }
}
