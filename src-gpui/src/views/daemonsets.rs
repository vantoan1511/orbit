use std::cmp::Ordering;
use std::rc::Rc;

use gpui::{
    AnyElement, App, ClickEvent, FontWeight, MouseDownEvent, SharedString, Window, div, prelude::*,
    px,
};
use lucide_icons::Icon;
use orbit_core::kubernetes::models::{DaemonSetInfo, EventInfo};

use crate::components::data_table::{
    DataTableCallbacks, SortState, TableColumn, render_data_table, sort_rows_stable,
};
use crate::components::{
    ContextMenuEntry, ContextMenuItem, StatusBadgeVariant, render_status_badge_from_str,
    resource_kind_meta,
};
use crate::icons::render_icon;
use crate::state::{DaemonSetsLoadStatus, DaemonSetsState, NamespaceFilterState};
use crate::theme::{K8sResourceKind, OrbitTheme, apply_text_step};
use crate::views::{format_images_summary, parse_age_seconds, render_action_button};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DaemonSetsPresentationMode {
    Table,
    ErrorCard,
}

pub fn daemonset_columns() -> Vec<TableColumn> {
    vec![
        TableColumn::new("name", "Name").flex_width(2.0, px(160.0)),
        TableColumn::new("namespace", "Namespace").flex_width(1.1, px(110.0)),
        TableColumn::new("status", "Status").fixed_width(px(115.0)),
        TableColumn::new("desired_current", "Desired/Current")
            .fixed_width(px(120.0))
            .right_numeric(),
        TableColumn::new("ready", "Ready")
            .fixed_width(px(76.0))
            .right_numeric(),
        TableColumn::new("up_to_date", "Up-To-Date")
            .fixed_width(px(96.0))
            .right_numeric(),
        TableColumn::new("available", "Available")
            .fixed_width(px(92.0))
            .right_numeric(),
        TableColumn::new("images", "Images")
            .flex_width(1.8, px(160.0))
            .monospace(true),
        TableColumn::new("age", "Age")
            .fixed_width(px(76.0))
            .right_numeric(),
    ]
}

fn compare_daemonsets_by_col(a: &DaemonSetInfo, b: &DaemonSetInfo, col_id: &str) -> Ordering {
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
        "name" => a
            .name
            .to_ascii_lowercase()
            .cmp(&b.name.to_ascii_lowercase())
            .then_with(|| {
                a.namespace
                    .to_ascii_lowercase()
                    .cmp(&b.namespace.to_ascii_lowercase())
            }),
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
        "desired_current" => (a.replicas.desired, a.replicas.current)
            .cmp(&(b.replicas.desired, b.replicas.current))
            .then_with(tie_break),
        "ready" => a.replicas.ready.cmp(&b.replicas.ready).then_with(tie_break),
        "up_to_date" => a
            .replicas
            .up_to_date
            .cmp(&b.replicas.up_to_date)
            .then_with(tie_break),
        "available" => a
            .replicas
            .available
            .cmp(&b.replicas.available)
            .then_with(tie_break),
        "images" => {
            let a_imgs = a.images.join(",");
            let b_imgs = b.images.join(",");
            a_imgs
                .to_ascii_lowercase()
                .cmp(&b_imgs.to_ascii_lowercase())
                .then_with(tie_break)
        }
        "age" => parse_age_seconds(&a.age)
            .cmp(&parse_age_seconds(&b.age))
            .then_with(tie_break),
        _ => tie_break(),
    }
}

pub fn filter_and_sort_daemonsets<'a>(
    daemonsets: &'a [DaemonSetInfo],
    namespaces: &NamespaceFilterState,
    sort: Option<&SortState>,
) -> Vec<&'a DaemonSetInfo> {
    let mut rows: Vec<&'a DaemonSetInfo> = daemonsets
        .iter()
        .filter(|ds| namespaces.matches_resource_namespace(Some(&ds.namespace)))
        .collect();
    sort_rows_stable(&mut rows, sort, |a, b, col_id| {
        compare_daemonsets_by_col(a, b, col_id)
    });
    rows
}

pub fn daemonsets_empty_message(state: &DaemonSetsState) -> &'static str {
    if state.daemonsets.is_empty() {
        "No daemonsets found in this cluster."
    } else {
        "No daemonsets found in the selected namespace(s)."
    }
}

pub fn daemonsets_presentation_mode(state: &DaemonSetsState) -> DaemonSetsPresentationMode {
    if state.status == DaemonSetsLoadStatus::Error && state.daemonsets.is_empty() {
        DaemonSetsPresentationMode::ErrorCard
    } else {
        DaemonSetsPresentationMode::Table
    }
}

fn render_daemonset_cell(ds: &DaemonSetInfo, col: &TableColumn, theme: &OrbitTheme) -> AnyElement {
    match col.id.as_ref() {
        "name" => {
            let meta = resource_kind_meta(K8sResourceKind::DaemonSet, theme);
            div()
                .flex()
                .items_center()
                .gap_2()
                .min_w(px(0.0))
                .child(render_icon(meta.icon, px(14.0), meta.color))
                .child(
                    apply_text_step(div(), theme.typography.size_xs)
                        .font_weight(FontWeight::MEDIUM)
                        .text_color(theme.text.primary)
                        .truncate()
                        .child(ds.name.clone()),
                )
                .into_any_element()
        }
        "namespace" => apply_text_step(div(), theme.typography.size_xs)
            .text_color(theme.text.secondary)
            .truncate()
            .child(ds.namespace.clone())
            .into_any_element(),
        "status" => render_status_badge_from_str(&ds.status, StatusBadgeVariant::Inline, theme),
        "desired_current" => div()
            .flex()
            .items_center()
            .justify_end()
            .child(
                apply_text_step(div(), theme.typography.size_xs)
                    .font(theme.typography.mono_font(FontWeight::SEMIBOLD))
                    .text_color(theme.text.primary)
                    .child(ds.replicas.desired.to_string()),
            )
            .child(
                apply_text_step(div(), theme.typography.size_xs)
                    .font(theme.typography.mono_font(FontWeight::NORMAL))
                    .text_color(theme.text.muted)
                    .child(format!("/{}", ds.replicas.current)),
            )
            .into_any_element(),
        "ready" => {
            let color = if ds.replicas.ready == ds.replicas.desired {
                theme.semantic.success.solid
            } else {
                theme.semantic.warning.solid
            };
            apply_text_step(div(), theme.typography.size_xs)
                .font(theme.typography.mono_font(FontWeight::MEDIUM))
                .text_color(color)
                .child(ds.replicas.ready.to_string())
                .into_any_element()
        }
        "up_to_date" => apply_text_step(div(), theme.typography.size_xs)
            .font(theme.typography.mono_font(FontWeight::NORMAL))
            .text_color(theme.text.primary)
            .child(ds.replicas.up_to_date.to_string())
            .into_any_element(),
        "available" => {
            let color = if ds.replicas.available == ds.replicas.desired {
                theme.semantic.success.solid
            } else {
                theme.semantic.warning.solid
            };
            apply_text_step(div(), theme.typography.size_xs)
                .font(theme.typography.mono_font(FontWeight::MEDIUM))
                .text_color(color)
                .child(ds.replicas.available.to_string())
                .into_any_element()
        }
        "images" => {
            let (primary, extra) = format_images_summary(&ds.images);
            div()
                .flex()
                .items_center()
                .gap_1p5()
                .min_w(px(0.0))
                .child(
                    apply_text_step(div(), theme.typography.size_xs)
                        .font(theme.typography.mono_font(FontWeight::NORMAL))
                        .text_color(theme.text.secondary)
                        .truncate()
                        .child(primary),
                )
                .when(extra > 0, |el| {
                    el.child(
                        apply_text_step(div(), theme.typography.size_xs)
                            .px_1p5()
                            .py_0p5()
                            .flex_shrink_0()
                            .rounded(theme.style.radius_control)
                            .bg(theme.surface.bg_hover)
                            .font(theme.typography.mono_font(FontWeight::MEDIUM))
                            .text_color(theme.text.muted)
                            .child(format!("+{extra}")),
                    )
                })
                .into_any_element()
        }
        "age" => apply_text_step(div(), theme.typography.size_xs)
            .font(theme.typography.mono_font(FontWeight::NORMAL))
            .text_color(theme.text.muted)
            .child(ds.age.clone())
            .into_any_element(),
        _ => div().into_any_element(),
    }
}

pub fn build_daemonset_context_menu_entries(
    daemonset: &DaemonSetInfo,
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
                format!("Filter by \"{}\"", daemonset.namespace),
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
        ContextMenuEntry::Separator,
        ContextMenuEntry::Item(
            ContextMenuItem::new("delete-daemonset", "Delete DaemonSet")
                .icon(Icon::Trash2)
                .danger(true)
                .disabled(true),
        ),
    ]
}

#[allow(clippy::too_many_arguments)]
pub fn render_daemonsets_view(
    state: &DaemonSetsState,
    namespaces: &NamespaceFilterState,
    theme: &OrbitTheme,
    on_sort: impl 'static + Fn(SharedString, &ClickEvent, &mut Window, &mut App),
    on_row_click: impl 'static + Fn(usize, SharedString, &ClickEvent, &mut Window, &mut App),
    on_row_context_menu: impl 'static
    + Fn(SharedString, &DaemonSetInfo, &MouseDownEvent, &mut Window, &mut App),
    on_refresh: impl 'static + Fn(&mut Window, &mut App),
) -> AnyElement {
    let columns = daemonset_columns();
    let filtered_rows: Rc<Vec<DaemonSetInfo>> = Rc::new(
        filter_and_sort_daemonsets(&state.daemonsets, namespaces, state.table.sort.as_ref())
            .into_iter()
            .cloned()
            .collect(),
    );
    let total_count = state.daemonsets.len();
    let visible_count = filtered_rows.len();
    let is_loading = state.status == DaemonSetsLoadStatus::Loading;
    type RefreshCallback = Rc<dyn Fn(&mut Window, &mut App)>;
    let on_refresh_rc: RefreshCallback = Rc::new(on_refresh);

    let count_label = if namespaces.is_all_namespaces() {
        format!("{total_count} daemonsets")
    } else {
        format!("{visible_count} of {total_count} daemonsets")
    };

    let toolbar = div()
        .flex()
        .items_center()
        .justify_between()
        .gap_3()
        .flex_shrink_0()
        .child(
            div()
                .flex()
                .items_center()
                .gap_2()
                .child(
                    apply_text_step(div(), theme.typography.size_xs)
                        .font_weight(FontWeight::MEDIUM)
                        .text_color(theme.text.secondary)
                        .child(count_label),
                )
                .when(!namespaces.is_all_namespaces(), |el| {
                    el.child(
                        apply_text_step(div(), theme.typography.size_xs)
                            .px_2()
                            .py_0p5()
                            .rounded(theme.style.radius_control)
                            .bg(theme.accent.soft)
                            .text_color(theme.accent.accent)
                            .font_weight(FontWeight::MEDIUM)
                            .child(format!("Namespace: {}", namespaces.summary_label())),
                    )
                }),
        )
        .child(render_action_button(
            "daemonsets-refresh-btn",
            Icon::RefreshCw,
            if is_loading {
                "Refreshing..."
            } else {
                "Refresh"
            },
            is_loading,
            theme,
            on_refresh_rc.clone(),
        ));

    let error_banner = state.error_message.as_ref().map(|message| {
        div()
            .flex_shrink_0()
            .px_3()
            .py_2()
            .rounded(theme.style.radius_control)
            .border_1()
            .border_color(theme.semantic.danger.solid)
            .bg(theme.semantic.danger.soft)
            .flex()
            .items_center()
            .justify_between()
            .gap_3()
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_2()
                    .min_w(px(0.0))
                    .child(render_icon(
                        Icon::AlertTriangle,
                        px(14.0),
                        theme.semantic.danger.solid,
                    ))
                    .child(
                        apply_text_step(div(), theme.typography.size_xs)
                            .text_color(theme.text.primary)
                            .truncate()
                            .child(format!("Failed to load daemonsets: {message}")),
                    ),
            )
            .child(render_action_button(
                "daemonsets-retry-btn",
                Icon::RotateCcw,
                "Retry",
                is_loading,
                theme,
                on_refresh_rc.clone(),
            ))
    });

    let body = match daemonsets_presentation_mode(state) {
        DaemonSetsPresentationMode::ErrorCard => {
            let message = state
                .error_message
                .clone()
                .unwrap_or_else(|| "Unknown error".to_string());
            theme
                .style_card(div())
                .flex()
                .flex_col()
                .items_start()
                .gap_3()
                .p_6()
                .bg(theme.surface.bg_panel)
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap_2()
                        .child(render_icon(
                            Icon::AlertTriangle,
                            px(16.0),
                            theme.semantic.danger.solid,
                        ))
                        .child(
                            apply_text_step(div(), theme.typography.size_sm)
                                .font_weight(FontWeight::SEMIBOLD)
                                .text_color(theme.text.primary)
                                .child("Unable to Load DaemonSets"),
                        ),
                )
                .child(
                    apply_text_step(div(), theme.typography.size_xs)
                        .text_color(theme.text.secondary)
                        .child(message),
                )
                .child(render_action_button(
                    "daemonsets-retry-btn",
                    Icon::RotateCcw,
                    "Retry",
                    is_loading,
                    theme,
                    on_refresh_rc,
                ))
                .into_any_element()
        }
        DaemonSetsPresentationMode::Table => {
            let mut table_state = state.table.clone();
            table_state.loading = is_loading && state.daemonsets.is_empty();
            table_state.empty_message = SharedString::from(daemonsets_empty_message(state));

            let key_rows = filtered_rows.clone();
            let cell_rows = filtered_rows.clone();
            let ctx_rows = filtered_rows.clone();
            let callbacks = DataTableCallbacks::new(
                move |idx| {
                    key_rows
                        .get(idx)
                        .map(|ds| format!("{}/{}", ds.namespace, ds.name))
                        .unwrap_or_default()
                },
                move |idx, col, col_theme| {
                    cell_rows
                        .get(idx)
                        .map(|ds| render_daemonset_cell(ds, col, col_theme))
                        .unwrap_or_else(|| div().into_any_element())
                },
            )
            .on_sort(on_sort)
            .on_row_click(on_row_click)
            .on_row_context_menu(move |idx, row_key, event, window, cx| {
                if let Some(daemonset) = ctx_rows.get(idx) {
                    on_row_context_menu(row_key, daemonset, event, window, cx);
                }
            });

            render_data_table(
                "daemonsets-table",
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
        .gap_4()
        .child(toolbar)
        .when(
            daemonsets_presentation_mode(state) == DaemonSetsPresentationMode::Table,
            |el| el.when_some(error_banner, |el, banner| el.child(banner)),
        )
        .child(body)
        .into_any_element()
}

pub fn filter_events_for_daemonset<'a>(
    events: &'a [EventInfo],
    daemonset: &DaemonSetInfo,
) -> Vec<&'a EventInfo> {
    let prefix = format!("{}-", daemonset.name);
    events
        .iter()
        .filter(|ev| {
            ev.namespace == daemonset.namespace
                && ((ev.object_kind == "DaemonSet" && ev.object_name == daemonset.name)
                    || (ev.object_kind == "Pod" && ev.object_name.starts_with(&prefix))
                    || ev.message.contains(&daemonset.name))
        })
        .collect()
}

pub fn render_daemonset_drawer_overview(
    daemonset: &DaemonSetInfo,
    theme: &OrbitTheme,
) -> AnyElement {
    let summary = crate::views::render_drawer_key_values(
        vec![
            ("Name", daemonset.name.clone(), true),
            ("Namespace", daemonset.namespace.clone(), true),
            ("Status", daemonset.status.clone(), false),
            ("Desired", daemonset.replicas.desired.to_string(), true),
            ("Current", daemonset.replicas.current.to_string(), true),
            ("Ready", daemonset.replicas.ready.to_string(), true),
            (
                "Up-To-Date",
                daemonset.replicas.up_to_date.to_string(),
                true,
            ),
            ("Available", daemonset.replicas.available.to_string(), true),
            ("Age", daemonset.age.clone(), true),
        ],
        theme,
    );

    let containers_section = crate::views::render_drawer_containers_section(
        &daemonset.containers,
        &daemonset.images,
        theme,
    );

    div()
        .id("daemonset-drawer-overview")
        .flex_1()
        .min_h_0()
        .overflow_y_scroll()
        .p_4()
        .flex()
        .flex_col()
        .gap_5()
        .child(crate::views::render_drawer_section(
            "SUMMARY", summary, theme,
        ))
        .child(containers_section)
        .child(crate::views::render_drawer_kv_badges(
            "LABELS",
            &daemonset.labels,
            "No labels",
            theme,
        ))
        .child(crate::views::render_drawer_kv_badges(
            "ANNOTATIONS",
            &daemonset.annotations,
            "No annotations",
            theme,
        ))
        .into_any_element()
}

pub fn render_daemonset_drawer_events(
    events: &[EventInfo],
    loading: bool,
    daemonset: &DaemonSetInfo,
    theme: &OrbitTheme,
) -> AnyElement {
    let matched = filter_events_for_daemonset(events, daemonset);
    div()
        .id("daemonset-drawer-events")
        .flex_1()
        .min_h_0()
        .overflow_y_scroll()
        .p_4()
        .child(crate::views::render_drawer_events_list(
            &matched,
            loading,
            "No events found for this daemonset.",
            theme,
        ))
        .into_any_element()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::components::data_table::{ColumnAlign, SortDirection};
    use orbit_core::kubernetes::models::{ContainerImageInfo, DaemonSetReplicas, NamespaceInfo};

    #[allow(clippy::too_many_arguments)]
    fn make_daemonset(
        name: &str,
        namespace: &str,
        status: &str,
        desired: i32,
        current: i32,
        ready: i32,
        up_to_date: i32,
        available: i32,
        image: &str,
        age: &str,
    ) -> DaemonSetInfo {
        DaemonSetInfo {
            kind: "DaemonSet".to_string(),
            name: name.to_string(),
            namespace: namespace.to_string(),
            status: status.to_string(),
            replicas: DaemonSetReplicas {
                desired,
                current,
                ready,
                up_to_date,
                available,
            },
            age: age.to_string(),
            images: vec![image.to_string()],
            containers: vec![ContainerImageInfo {
                name: name.to_string(),
                image: image.to_string(),
            }],
            labels: Default::default(),
            annotations: Default::default(),
        }
    }

    #[test]
    fn test_daemonset_columns_schema_and_alignment() {
        let cols = daemonset_columns();
        let ids: Vec<&str> = cols.iter().map(|c| c.id.as_ref()).collect();
        let labels: Vec<&str> = cols.iter().map(|c| c.header.as_ref()).collect();
        assert_eq!(
            ids,
            vec![
                "name",
                "namespace",
                "status",
                "desired_current",
                "ready",
                "up_to_date",
                "available",
                "images",
                "age",
            ]
        );
        assert_eq!(
            labels,
            vec![
                "Name",
                "Namespace",
                "Status",
                "Desired/Current",
                "Ready",
                "Up-To-Date",
                "Available",
                "Images",
                "Age",
            ]
        );
        assert_eq!(cols[3].align, ColumnAlign::Right);
        assert_eq!(cols[4].align, ColumnAlign::Right);
        assert_eq!(cols[5].align, ColumnAlign::Right);
        assert_eq!(cols[6].align, ColumnAlign::Right);
        assert_eq!(cols[8].align, ColumnAlign::Right);
    }

    #[test]
    fn test_filter_and_sort_daemonsets_by_namespace_and_columns() {
        let items = vec![
            make_daemonset(
                "node-exporter",
                "monitoring",
                "Running",
                5,
                5,
                5,
                5,
                5,
                "prom/node-exporter:v1.8",
                "10d",
            ),
            make_daemonset(
                "fluent-bit",
                "logging",
                "Progressing",
                4,
                4,
                2,
                3,
                2,
                "fluent/fluent-bit:3.0",
                "2h",
            ),
            make_daemonset(
                "vector",
                "logging",
                "Running",
                4,
                3,
                3,
                4,
                3,
                "timberio/vector:0.40",
                "45m",
            ),
        ];

        let mut ns = NamespaceFilterState::default();
        ns.begin_load_for_cluster("ctx-a");
        ns.apply_loaded(
            "ctx-a",
            1,
            vec![
                NamespaceInfo {
                    name: "logging".to_string(),
                    status: "Active".to_string(),
                    is_system: false,
                    age: "10d".to_string(),
                    labels: Default::default(),
                    annotations: Default::default(),
                    uid: "uid-logging".to_string(),
                    created: String::new(),
                },
                NamespaceInfo {
                    name: "monitoring".to_string(),
                    status: "Active".to_string(),
                    is_system: false,
                    age: "10d".to_string(),
                    labels: Default::default(),
                    annotations: Default::default(),
                    uid: "uid-monitoring".to_string(),
                    created: String::new(),
                },
            ],
        );

        // Single namespace filter
        ns.toggle_namespace("logging");
        let by_name = SortState {
            column_id: "name".to_string(),
            direction: SortDirection::Asc,
        };
        let filtered = filter_and_sort_daemonsets(&items, &ns, Some(&by_name));
        assert_eq!(
            filtered.iter().map(|d| d.name.as_str()).collect::<Vec<_>>(),
            vec!["fluent-bit", "vector"]
        );

        // Multiple namespaces filter
        ns.toggle_namespace("monitoring");
        let multi = filter_and_sort_daemonsets(&items, &ns, Some(&by_name));
        assert_eq!(multi.len(), 3);

        let all_ns = NamespaceFilterState::default();

        // Namespace asc/desc
        let by_ns_desc = SortState {
            column_id: "namespace".to_string(),
            direction: SortDirection::Desc,
        };
        assert_eq!(
            filter_and_sort_daemonsets(&items, &all_ns, Some(&by_ns_desc))[0].name,
            "node-exporter"
        );

        // Status asc
        let by_status = SortState {
            column_id: "status".to_string(),
            direction: SortDirection::Asc,
        };
        assert_eq!(
            filter_and_sort_daemonsets(&items, &all_ns, Some(&by_status))[0].status,
            "Progressing"
        );

        // Desired/Current tuple (desired, current) asc & desc
        let by_dc_asc = SortState {
            column_id: "desired_current".to_string(),
            direction: SortDirection::Asc,
        };
        assert_eq!(
            filter_and_sort_daemonsets(&items, &all_ns, Some(&by_dc_asc))
                .iter()
                .map(|d| d.name.as_str())
                .collect::<Vec<_>>(),
            vec!["vector", "fluent-bit", "node-exporter"]
        );
        let by_dc_desc = SortState {
            column_id: "desired_current".to_string(),
            direction: SortDirection::Desc,
        };
        assert_eq!(
            filter_and_sort_daemonsets(&items, &all_ns, Some(&by_dc_desc))[0].name,
            "node-exporter"
        );

        // Ready asc/desc
        let by_ready_asc = SortState {
            column_id: "ready".to_string(),
            direction: SortDirection::Asc,
        };
        assert_eq!(
            filter_and_sort_daemonsets(&items, &all_ns, Some(&by_ready_asc))
                .iter()
                .map(|d| d.name.as_str())
                .collect::<Vec<_>>(),
            vec!["fluent-bit", "vector", "node-exporter"]
        );

        // Up-to-date desc
        let by_utd_desc = SortState {
            column_id: "up_to_date".to_string(),
            direction: SortDirection::Desc,
        };
        assert_eq!(
            filter_and_sort_daemonsets(&items, &all_ns, Some(&by_utd_desc))
                .iter()
                .map(|d| d.name.as_str())
                .collect::<Vec<_>>(),
            vec!["node-exporter", "vector", "fluent-bit"]
        );

        // Available asc
        let by_avail_asc = SortState {
            column_id: "available".to_string(),
            direction: SortDirection::Asc,
        };
        assert_eq!(
            filter_and_sort_daemonsets(&items, &all_ns, Some(&by_avail_asc))[0].name,
            "fluent-bit"
        );

        // Images asc
        let by_images = SortState {
            column_id: "images".to_string(),
            direction: SortDirection::Asc,
        };
        assert_eq!(
            filter_and_sort_daemonsets(&items, &all_ns, Some(&by_images))[0].name,
            "fluent-bit"
        );

        // Age asc & desc via parse_age_seconds
        let by_age_asc = SortState {
            column_id: "age".to_string(),
            direction: SortDirection::Asc,
        };
        assert_eq!(
            filter_and_sort_daemonsets(&items, &all_ns, Some(&by_age_asc))
                .iter()
                .map(|d| d.name.as_str())
                .collect::<Vec<_>>(),
            vec!["vector", "fluent-bit", "node-exporter"]
        );
        let by_age_desc = SortState {
            column_id: "age".to_string(),
            direction: SortDirection::Desc,
        };
        assert_eq!(
            filter_and_sort_daemonsets(&items, &all_ns, Some(&by_age_desc))[0].name,
            "node-exporter"
        );
    }

    #[test]
    fn test_daemonsets_empty_message_and_presentation_mode() {
        let mut state = DaemonSetsState::default();
        assert_eq!(
            daemonsets_empty_message(&state),
            "No daemonsets found in this cluster."
        );
        assert_eq!(
            daemonsets_presentation_mode(&state),
            DaemonSetsPresentationMode::Table
        );

        // Initial error with no cached rows -> ErrorCard
        state.status = DaemonSetsLoadStatus::Error;
        state.error_message = Some("forbidden".to_string());
        assert_eq!(
            daemonsets_presentation_mode(&state),
            DaemonSetsPresentationMode::ErrorCard
        );

        // Refresh error with cached rows -> Table (inline error banner)
        state.daemonsets.push(make_daemonset(
            "fluent-bit",
            "logging",
            "Running",
            3,
            3,
            3,
            3,
            3,
            "fluent/fluent-bit:3.0",
            "1d",
        ));
        assert_eq!(
            daemonsets_empty_message(&state),
            "No daemonsets found in the selected namespace(s)."
        );
        assert_eq!(
            daemonsets_presentation_mode(&state),
            DaemonSetsPresentationMode::Table
        );
    }

    #[test]
    fn test_filter_events_for_daemonset_matches_controller_pods_and_message() {
        let ds = make_daemonset(
            "fluent-bit",
            "logging",
            "Running",
            3,
            3,
            3,
            3,
            3,
            "fluent/fluent-bit:3.0",
            "1d",
        );
        let events = vec![
            EventInfo {
                uid: "1".to_string(),
                time: "1m".to_string(),
                r#type: "Normal".to_string(),
                reason: "SuccessfulCreate".to_string(),
                object_kind: "DaemonSet".to_string(),
                object_name: "fluent-bit".to_string(),
                namespace: "logging".to_string(),
                message: "Created pod: fluent-bit-abc12".to_string(),
                count: 1,
                first_seen: String::new(),
                last_seen: String::new(),
                source: "daemonset-controller".to_string(),
                labels: Default::default(),
                annotations: Default::default(),
            },
            EventInfo {
                uid: "2".to_string(),
                time: "1m".to_string(),
                r#type: "Normal".to_string(),
                reason: "Started".to_string(),
                object_kind: "Pod".to_string(),
                object_name: "fluent-bit-abc12".to_string(),
                namespace: "logging".to_string(),
                message: "Started container".to_string(),
                count: 1,
                first_seen: String::new(),
                last_seen: String::new(),
                source: "kubelet".to_string(),
                labels: Default::default(),
                annotations: Default::default(),
            },
            EventInfo {
                uid: "3".to_string(),
                time: "1m".to_string(),
                r#type: "Warning".to_string(),
                reason: "Failed".to_string(),
                object_kind: "Pod".to_string(),
                object_name: "fluent-bit-abc12".to_string(),
                namespace: "other".to_string(),
                message: "wrong namespace".to_string(),
                count: 1,
                first_seen: String::new(),
                last_seen: String::new(),
                source: "kubelet".to_string(),
                labels: Default::default(),
                annotations: Default::default(),
            },
            EventInfo {
                uid: "4".to_string(),
                time: "1m".to_string(),
                r#type: "Normal".to_string(),
                reason: "Updated".to_string(),
                object_kind: "ConfigMap".to_string(),
                object_name: "cm".to_string(),
                namespace: "logging".to_string(),
                message: "Reloaded config for fluent-bit".to_string(),
                count: 1,
                first_seen: String::new(),
                last_seen: String::new(),
                source: "controller".to_string(),
                labels: Default::default(),
                annotations: Default::default(),
            },
            EventInfo {
                uid: "5".to_string(),
                time: "1m".to_string(),
                r#type: "Normal".to_string(),
                reason: "Started".to_string(),
                object_kind: "Pod".to_string(),
                object_name: "other-pod-xyz".to_string(),
                namespace: "logging".to_string(),
                message: "Unrelated pod".to_string(),
                count: 1,
                first_seen: String::new(),
                last_seen: String::new(),
                source: "kubelet".to_string(),
                labels: Default::default(),
                annotations: Default::default(),
            },
        ];

        let matched = filter_events_for_daemonset(&events, &ds);
        let uids: Vec<&str> = matched.iter().map(|e| e.uid.as_str()).collect();
        assert_eq!(uids, vec!["1", "2", "4"]);
    }

    #[test]
    fn test_build_daemonset_context_menu_entries_structure() {
        let ds = make_daemonset(
            "fluent-bit",
            "logging",
            "Running",
            3,
            3,
            3,
            3,
            3,
            "fluent/fluent-bit:3.0",
            "1d",
        );
        let entries = build_daemonset_context_menu_entries(
            &ds,
            |_, _| {},
            |_, _| {},
            |_, _| {},
            |_, _| {},
            |_, _| {},
        );
        assert_eq!(entries.len(), 8);
        assert!(matches!(entries[3], ContextMenuEntry::Separator));
        assert!(matches!(entries[6], ContextMenuEntry::Separator));

        let item_meta: Vec<(&str, &str, bool, bool)> = entries
            .iter()
            .filter_map(|e| match e {
                ContextMenuEntry::Item(item) => Some((
                    item.id.as_ref(),
                    item.label.as_ref(),
                    item.danger,
                    item.disabled,
                )),
                ContextMenuEntry::Separator => None,
            })
            .collect();

        assert_eq!(
            item_meta,
            vec![
                ("copy-name", "Copy Name", false, false),
                ("filter-namespace", "Filter by \"logging\"", false, false),
                ("refresh", "Refresh", false, false),
                ("view-details", "View Details", false, false),
                ("view-yaml", "View YAML", false, false),
                ("delete-daemonset", "Delete DaemonSet", true, true),
            ]
        );
    }
}
