use std::cmp::Ordering;
use std::rc::Rc;

use gpui::{
    AnyElement, App, ClickEvent, ElementId, FontWeight, MouseDownEvent, SharedString, Window, div,
    prelude::*, px,
};
use lucide_icons::Icon;
use orbit_core::kubernetes::models::{EventInfo, HpaInfo};

use crate::components::data_table::{
    DataTableCallbacks, SortState, TableColumn, render_data_table, sort_rows_stable,
};
use crate::components::{
    ContextMenuEntry, ContextMenuItem, StatusBadgeVariant, render_status_badge_from_str,
};
use crate::navigation::WorkspaceView;
use crate::state::{HpasLoadStatus, HpasState, NamespaceFilterState};
use crate::theme::{OrbitTheme, apply_text_step};
use crate::views::{
    parse_age_seconds, render_drawer_events_list, render_drawer_key_values, render_drawer_section,
    render_resource_error_banner, render_resource_error_state, render_resource_view_header,
    render_truncated_cell, table_numeric_font,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HpasPresentationMode {
    Table,
    ErrorCard,
}

pub fn hpa_columns() -> Vec<TableColumn> {
    vec![
        TableColumn::new("name", "Name").flex_width(1.3, px(130.0)),
        TableColumn::new("namespace", "Namespace").flex_width(0.9, px(100.0)),
        TableColumn::new("target", "Target")
            .flex_width(1.3, px(140.0))
            .monospace(true),
        TableColumn::new("status", "Status").fixed_width(px(110.0)),
        TableColumn::new("min", "Min")
            .fixed_width(px(56.0))
            .right_numeric(),
        TableColumn::new("max", "Max")
            .fixed_width(px(56.0))
            .right_numeric(),
        TableColumn::new("current", "Current")
            .fixed_width(px(68.0))
            .right_numeric(),
        TableColumn::new("desired", "Desired")
            .fixed_width(px(68.0))
            .right_numeric(),
        TableColumn::new("metrics", "Metrics")
            .flex_width(1.6, px(160.0))
            .monospace(true),
        TableColumn::new("age", "Age")
            .fixed_width(px(72.0))
            .right_numeric(),
    ]
}

pub fn hpa_target_label(hpa: &HpaInfo) -> String {
    format!(
        "{}/{}",
        hpa.scale_target_ref.kind, hpa.scale_target_ref.name
    )
}

pub fn hpa_metrics_label(hpa: &HpaInfo) -> String {
    if hpa.metrics_summary.is_empty() {
        "-".to_string()
    } else {
        hpa.metrics_summary.join(", ")
    }
}

pub fn compare_hpas_by_col(a: &HpaInfo, b: &HpaInfo, col_id: &str) -> Ordering {
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
            .then_with(tie_break),
        "target" => hpa_target_label(a)
            .to_ascii_lowercase()
            .cmp(&hpa_target_label(b).to_ascii_lowercase())
            .then_with(tie_break),
        "status" => a
            .status
            .to_ascii_lowercase()
            .cmp(&b.status.to_ascii_lowercase())
            .then_with(tie_break),
        "min" => a.min_replicas.cmp(&b.min_replicas).then_with(tie_break),
        "max" => a.max_replicas.cmp(&b.max_replicas).then_with(tie_break),
        "current" => a
            .current_replicas
            .cmp(&b.current_replicas)
            .then_with(tie_break),
        "desired" => a
            .desired_replicas
            .cmp(&b.desired_replicas)
            .then_with(tie_break),
        "metrics" => hpa_metrics_label(a)
            .to_ascii_lowercase()
            .cmp(&hpa_metrics_label(b).to_ascii_lowercase())
            .then_with(tie_break),
        "age" => parse_age_seconds(&a.age)
            .cmp(&parse_age_seconds(&b.age))
            .then_with(tie_break),
        _ => tie_break(),
    }
}

pub fn filter_and_sort_hpas<'a>(
    hpas: &'a [HpaInfo],
    namespaces: &NamespaceFilterState,
    search: &str,
    sort: Option<&SortState>,
) -> Vec<&'a HpaInfo> {
    let query = search.trim().to_ascii_lowercase();
    let mut rows: Vec<&'a HpaInfo> = hpas
        .iter()
        .filter(|hpa| {
            if !namespaces.matches_resource_namespace(Some(&hpa.namespace)) {
                return false;
            }
            if query.is_empty() {
                return true;
            }
            hpa.name.to_ascii_lowercase().contains(&query)
                || hpa.namespace.to_ascii_lowercase().contains(&query)
                || hpa_target_label(hpa).to_ascii_lowercase().contains(&query)
                || hpa_metrics_label(hpa).to_ascii_lowercase().contains(&query)
        })
        .collect();

    if sort.is_some() {
        sort_rows_stable(&mut rows, sort, |a, b, col_id| {
            compare_hpas_by_col(a, b, col_id)
        });
    } else {
        rows.sort_by(|a, b| compare_hpas_by_col(a, b, "name"));
    }
    rows
}

pub fn hpas_empty_message(state: &HpasState) -> &'static str {
    if state.hpas.is_empty() {
        "No HorizontalPodAutoscalers found in this cluster."
    } else {
        "No HorizontalPodAutoscalers found in the selected namespace(s)."
    }
}

pub fn hpas_presentation_mode(state: &HpasState) -> HpasPresentationMode {
    if state.status == HpasLoadStatus::Error && state.hpas.is_empty() {
        HpasPresentationMode::ErrorCard
    } else {
        HpasPresentationMode::Table
    }
}

fn render_hpa_numeric_cell(value: i32, theme: &OrbitTheme) -> AnyElement {
    apply_text_step(div(), theme.typography.size_xs)
        .font(table_numeric_font(theme))
        .text_color(theme.text.secondary)
        .child(value.to_string())
        .into_any_element()
}

fn render_hpa_cell(
    row_idx: usize,
    hpa: &HpaInfo,
    col: &TableColumn,
    theme: &OrbitTheme,
) -> AnyElement {
    match col.id.as_ref() {
        "name" => render_truncated_cell(
            ElementId::NamedInteger(SharedString::from("hpa-name"), row_idx as u64),
            hpa.name.clone(),
            Some(SharedString::from(hpa.name.clone())),
            theme.text.primary,
            FontWeight::MEDIUM,
            false,
            theme,
        ),
        "namespace" => render_truncated_cell(
            ElementId::NamedInteger(SharedString::from("hpa-ns"), row_idx as u64),
            hpa.namespace.clone(),
            None,
            theme.text.secondary,
            FontWeight::NORMAL,
            false,
            theme,
        ),
        "target" => {
            let label = hpa_target_label(hpa);
            render_truncated_cell(
                ElementId::NamedInteger(SharedString::from("hpa-target"), row_idx as u64),
                label.clone(),
                Some(SharedString::from(label)),
                theme.text.secondary,
                FontWeight::NORMAL,
                true,
                theme,
            )
        }
        "status" => render_status_badge_from_str(&hpa.status, StatusBadgeVariant::Inline, theme),
        "min" => render_hpa_numeric_cell(hpa.min_replicas, theme),
        "max" => render_hpa_numeric_cell(hpa.max_replicas, theme),
        "current" => render_hpa_numeric_cell(hpa.current_replicas, theme),
        "desired" => render_hpa_numeric_cell(hpa.desired_replicas, theme),
        "metrics" => {
            let label = hpa_metrics_label(hpa);
            let is_empty = hpa.metrics_summary.is_empty();
            render_truncated_cell(
                ElementId::NamedInteger(SharedString::from("hpa-metrics"), row_idx as u64),
                label.clone(),
                (!is_empty).then(|| SharedString::from(label)),
                if is_empty {
                    theme.text.muted
                } else {
                    theme.text.secondary
                },
                FontWeight::NORMAL,
                true,
                theme,
            )
        }
        "age" => apply_text_step(div(), theme.typography.size_xs)
            .font(table_numeric_font(theme))
            .text_color(theme.text.muted)
            .child(hpa.age.clone())
            .into_any_element(),
        _ => div().into_any_element(),
    }
}

pub fn build_hpa_context_menu_entries(
    hpa: &HpaInfo,
    on_copy_name: impl 'static + Fn(&mut Window, &mut App),
    on_filter_namespace: impl 'static + Fn(&mut Window, &mut App),
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
            ContextMenuItem::new(
                "filter-namespace",
                format!("Filter by \"{}\"", hpa.namespace),
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
    ]
}

#[allow(clippy::too_many_arguments)]
pub fn render_hpas_view(
    state: &HpasState,
    namespaces: &NamespaceFilterState,
    theme: &OrbitTheme,
    on_sort: impl 'static + Fn(SharedString, &ClickEvent, &mut Window, &mut App),
    on_row_click: impl 'static + Fn(usize, SharedString, &ClickEvent, &mut Window, &mut App),
    on_row_quick_yaml: impl 'static + Fn(usize, SharedString, &ClickEvent, &mut Window, &mut App),
    on_row_context_menu: impl 'static
    + Fn(SharedString, &HpaInfo, &MouseDownEvent, &mut Window, &mut App),
    on_refresh: impl 'static + Fn(&mut Window, &mut App),
) -> AnyElement {
    let columns = hpa_columns();
    let filtered_rows: Rc<Vec<HpaInfo>> = Rc::new(
        filter_and_sort_hpas(&state.hpas, namespaces, "", state.table.sort.as_ref())
            .into_iter()
            .cloned()
            .collect(),
    );
    let total_count = state.hpas.len();
    let visible_count = filtered_rows.len();
    let is_loading = state.status == HpasLoadStatus::Loading;
    let presentation_mode = hpas_presentation_mode(state);
    let on_refresh_rc: super::ActionCallback = Rc::new(on_refresh);

    let count_label = if namespaces.is_all_namespaces() {
        format!("{total_count} HPAs")
    } else {
        format!("{visible_count} of {total_count} HPAs")
    };

    let header = render_resource_view_header(
        WorkspaceView::Hpas.label(),
        WorkspaceView::Hpas.description(),
        count_label,
        Some(namespaces),
        None,
        theme,
    );

    let error_banner = if presentation_mode == HpasPresentationMode::Table {
        state.error_message.as_ref().map(|message| {
            render_resource_error_banner(
                "hpas-retry-btn",
                format!("Failed to load HPAs: {message}"),
                is_loading,
                theme,
                on_refresh_rc.clone(),
            )
        })
    } else {
        None
    };

    let body = match presentation_mode {
        HpasPresentationMode::ErrorCard => {
            let message = state
                .error_message
                .clone()
                .unwrap_or_else(|| "Unknown error".to_string());
            render_resource_error_state(
                "hpas-retry-btn",
                "Unable to Load HorizontalPodAutoscalers",
                message,
                is_loading,
                theme,
                on_refresh_rc,
            )
        }
        HpasPresentationMode::Table => {
            let mut table_state = state.table.clone();
            table_state.loading = is_loading && state.hpas.is_empty();
            table_state.empty_message = SharedString::from(hpas_empty_message(state));

            let key_rows = filtered_rows.clone();
            let cell_rows = filtered_rows.clone();
            let ctx_rows = filtered_rows.clone();
            let callbacks = DataTableCallbacks::new(
                move |idx| {
                    key_rows
                        .get(idx)
                        .map(|hpa| format!("{}/{}", hpa.namespace, hpa.name))
                        .unwrap_or_default()
                },
                move |idx, col, col_theme| {
                    cell_rows
                        .get(idx)
                        .map(|hpa| render_hpa_cell(idx, hpa, col, col_theme))
                        .unwrap_or_else(|| div().into_any_element())
                },
            )
            .on_sort(on_sort)
            .on_row_click(on_row_click)
            .on_row_quick_yaml(on_row_quick_yaml)
            .on_row_context_menu(move |idx, row_key, event, window, cx| {
                if let Some(hpa) = ctx_rows.get(idx) {
                    on_row_context_menu(row_key, hpa, event, window, cx);
                }
            });

            render_data_table(
                "hpas-table",
                &columns,
                visible_count,
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
        .when_some(error_banner, |el, banner| el.child(banner))
        .child(body)
        .into_any_element()
}

pub fn filter_events_for_hpa<'a>(events: &'a [EventInfo], hpa: &HpaInfo) -> Vec<&'a EventInfo> {
    events
        .iter()
        .filter(|ev| {
            ev.object_kind == "HorizontalPodAutoscaler"
                && ev.object_name == hpa.name
                && ev.namespace == hpa.namespace
        })
        .collect()
}

pub fn render_hpa_drawer_overview(hpa: &HpaInfo, theme: &OrbitTheme) -> AnyElement {
    let info_rows = vec![
        ("Name", hpa.name.clone(), true),
        ("Namespace", hpa.namespace.clone(), false),
        ("Target", hpa_target_label(hpa), true),
        ("Status", hpa.status.clone(), false),
        ("Min Replicas", hpa.min_replicas.to_string(), true),
        ("Max Replicas", hpa.max_replicas.to_string(), true),
        ("Current Replicas", hpa.current_replicas.to_string(), true),
        ("Desired Replicas", hpa.desired_replicas.to_string(), true),
        (
            "Last Scale Time",
            hpa.last_scale_time
                .clone()
                .unwrap_or_else(|| "-".to_string()),
            true,
        ),
        ("Age", hpa.age.clone(), true),
    ];

    let metrics_section = if hpa.metrics_summary.is_empty() {
        None
    } else {
        let rows: Vec<(&'static str, String, bool)> = hpa
            .metrics_summary
            .iter()
            .map(|summary| ("Metric", summary.clone(), true))
            .collect();
        Some(render_drawer_section(
            "METRICS",
            render_drawer_key_values(rows, theme),
            theme,
        ))
    };

    let conditions_section = if hpa.conditions.is_empty() {
        None
    } else {
        let mut list = div().flex().flex_col().gap_2();
        for cond in &hpa.conditions {
            let detail = cond
                .message
                .as_deref()
                .or(cond.reason.as_deref())
                .unwrap_or("-");
            list = list.child(
                div()
                    .w_full()
                    .p_2p5()
                    .rounded(theme.style.radius_control)
                    .bg(theme.surface.bg_card)
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
                                    .child(cond.r#type.clone()),
                            )
                            .child(
                                apply_text_step(div(), theme.typography.size_xs)
                                    .font(theme.typography.mono_font(FontWeight::MEDIUM))
                                    .text_color(theme.text.secondary)
                                    .child(cond.status.clone()),
                            ),
                    )
                    .child(
                        apply_text_step(div(), theme.typography.size_xs)
                            .text_color(theme.text.secondary)
                            .child(detail.to_string()),
                    ),
            );
        }
        Some(render_drawer_section(
            "CONDITIONS",
            list.into_any_element(),
            theme,
        ))
    };

    div()
        .w_full()
        .flex()
        .flex_col()
        .gap_4()
        .child(render_drawer_section(
            "HPA INFORMATION",
            render_drawer_key_values(info_rows, theme),
            theme,
        ))
        .when_some(metrics_section, |el, sec| el.child(sec))
        .when_some(conditions_section, |el, sec| el.child(sec))
        .into_any_element()
}

pub fn render_hpa_drawer_events(
    events: &[&EventInfo],
    loading: bool,
    theme: &OrbitTheme,
) -> AnyElement {
    render_drawer_events_list(events, loading, "No recent events for this HPA.", theme)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::components::data_table::{ColumnAlign, SortDirection};
    use orbit_core::kubernetes::models::HpaTargetRef;

    #[allow(clippy::too_many_arguments)]
    fn sample_hpa(
        name: &str,
        namespace: &str,
        target_kind: &str,
        target_name: &str,
        min_replicas: i32,
        max_replicas: i32,
        current_replicas: i32,
        desired_replicas: i32,
        age: &str,
    ) -> HpaInfo {
        HpaInfo {
            kind: "HorizontalPodAutoscaler".to_string(),
            name: name.to_string(),
            namespace: namespace.to_string(),
            status: "Active".to_string(),
            scale_target_ref: HpaTargetRef {
                kind: target_kind.to_string(),
                name: target_name.to_string(),
                api_version: Some("apps/v1".to_string()),
            },
            min_replicas,
            max_replicas,
            current_replicas,
            desired_replicas,
            metrics: Vec::new(),
            metrics_summary: vec!["cpu: 40% / 80%".to_string()],
            conditions: Vec::new(),
            last_scale_time: None,
            age: age.to_string(),
            created: String::new(),
            labels: Default::default(),
            annotations: Default::default(),
        }
    }

    #[test]
    fn test_hpa_columns_cover_required_fields_and_numeric_alignment() {
        let cols = hpa_columns();
        let ids: Vec<&str> = cols.iter().map(|c| c.id.as_ref()).collect();
        assert_eq!(
            ids,
            vec![
                "name",
                "namespace",
                "target",
                "status",
                "min",
                "max",
                "current",
                "desired",
                "metrics",
                "age",
            ]
        );
        for id in ["min", "max", "current", "desired", "age"] {
            let col = cols.iter().find(|c| c.id.as_ref() == id);
            assert!(
                col.is_some_and(|c| c.align == ColumnAlign::Right && c.monospace),
                "{id} must be right_numeric"
            );
        }
    }

    #[test]
    fn test_compare_hpas_sorts_replicas_numerically() {
        let small = sample_hpa("hpa-a", "default", "Deployment", "web", 2, 10, 2, 2, "1d");
        let large = sample_hpa(
            "hpa-b",
            "default",
            "Deployment",
            "api",
            10,
            20,
            10,
            12,
            "2d",
        );

        assert_eq!(compare_hpas_by_col(&small, &large, "min"), Ordering::Less);
        assert_eq!(
            compare_hpas_by_col(&small, &large, "current"),
            Ordering::Less
        );
        assert_eq!(
            compare_hpas_by_col(&small, &large, "desired"),
            Ordering::Less
        );
        assert_eq!(compare_hpas_by_col(&small, &large, "max"), Ordering::Less);
    }

    #[test]
    fn test_filter_hpas_by_namespace_and_search() {
        let hpas = vec![
            sample_hpa("web-hpa", "prod", "Deployment", "web", 2, 10, 3, 4, "2h"),
            sample_hpa(
                "worker-hpa",
                "staging",
                "StatefulSet",
                "worker",
                1,
                5,
                1,
                1,
                "5d",
            ),
            sample_hpa("api-hpa", "prod", "Deployment", "api", 2, 8, 2, 2, "30m"),
        ];

        let mut ns = NamespaceFilterState {
            cluster_id: Some("ctx-a".to_string()),
            ..Default::default()
        };

        let all = filter_and_sort_hpas(&hpas, &ns, "", None);
        let names: Vec<&str> = all.iter().map(|h| h.name.as_str()).collect();
        assert_eq!(names, vec!["api-hpa", "web-hpa", "worker-hpa"]);

        ns.toggle_namespace("prod");
        let prod_only = filter_and_sort_hpas(&hpas, &ns, "", None);
        assert_eq!(prod_only.len(), 2);

        let search_web = filter_and_sort_hpas(&hpas, &ns, "web", None);
        assert_eq!(search_web.len(), 1);
        assert_eq!(search_web[0].name, "web-hpa");

        let by_age = filter_and_sort_hpas(
            &hpas,
            &ns,
            "",
            Some(&SortState {
                column_id: "age".to_string(),
                direction: SortDirection::Asc,
            }),
        );
        assert_eq!(by_age[0].name, "api-hpa");
    }

    #[test]
    fn test_hpa_target_label_formats_kind_slash_name() {
        let hpa = sample_hpa("web-hpa", "default", "Deployment", "web", 2, 10, 3, 4, "1d");
        assert_eq!(hpa_target_label(&hpa), "Deployment/web");
    }

    #[test]
    fn test_hpas_empty_message_and_presentation_mode() {
        let mut state = HpasState::default();
        assert_eq!(
            hpas_empty_message(&state),
            "No HorizontalPodAutoscalers found in this cluster."
        );
        assert_eq!(hpas_presentation_mode(&state), HpasPresentationMode::Table);

        state.status = HpasLoadStatus::Error;
        assert_eq!(
            hpas_presentation_mode(&state),
            HpasPresentationMode::ErrorCard
        );

        state.hpas.push(sample_hpa(
            "web-hpa",
            "default",
            "Deployment",
            "web",
            1,
            5,
            2,
            2,
            "1d",
        ));
        assert_eq!(hpas_presentation_mode(&state), HpasPresentationMode::Table);
        assert_eq!(
            hpas_empty_message(&state),
            "No HorizontalPodAutoscalers found in the selected namespace(s)."
        );
    }
}
