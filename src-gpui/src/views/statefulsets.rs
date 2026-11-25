use std::cmp::Ordering;
use std::rc::Rc;

use gpui::{
    AnyElement, App, ClickEvent, ElementId, FontWeight, MouseDownEvent, SharedString, Window, div,
    prelude::*, px,
};
use lucide_icons::Icon;
use orbit_core::kubernetes::models::{EventInfo, StatefulSetInfo};

use crate::components::data_table::{
    DataTableCallbacks, SortState, TableColumn, render_data_table, sort_rows_stable,
};
use crate::components::{
    ContextMenuEntry, ContextMenuItem, StatusBadgeVariant, render_status_badge_from_str,
};
use crate::navigation::WorkspaceView;
use crate::state::{NamespaceFilterState, StatefulSetsLoadStatus, StatefulSetsState};
use crate::theme::{OrbitTheme, apply_text_step};
use crate::views::{
    parse_age_seconds, render_images_cell, render_ratio_metric_cell, render_resource_error_banner,
    render_resource_error_state, render_resource_view_header, render_truncated_cell,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StatefulSetsPresentationMode {
    Table,
    ErrorCard,
}

pub fn statefulset_columns() -> Vec<TableColumn> {
    vec![
        TableColumn::new("name", "Name").flex_width(2.0, px(160.0)),
        TableColumn::new("namespace", "Namespace").flex_width(1.1, px(110.0)),
        TableColumn::new("status", "Status").fixed_width(px(115.0)),
        TableColumn::new("replicas", "Replicas")
            .fixed_width(px(95.0))
            .right_numeric(),
        TableColumn::new("images", "Images")
            .flex_width(1.8, px(160.0))
            .monospace(true),
        TableColumn::new("age", "Age")
            .fixed_width(px(76.0))
            .right_numeric(),
    ]
}

fn compare_statefulsets_by_col(a: &StatefulSetInfo, b: &StatefulSetInfo, col_id: &str) -> Ordering {
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
        "replicas" => (a.replicas.desired, a.replicas.current)
            .cmp(&(b.replicas.desired, b.replicas.current))
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

pub fn filter_and_sort_statefulsets<'a>(
    statefulsets: &'a [StatefulSetInfo],
    namespaces: &NamespaceFilterState,
    sort: Option<&SortState>,
) -> Vec<&'a StatefulSetInfo> {
    let mut rows: Vec<&'a StatefulSetInfo> = statefulsets
        .iter()
        .filter(|ss| namespaces.matches_resource_namespace(Some(&ss.namespace)))
        .collect();
    sort_rows_stable(&mut rows, sort, |a, b, col_id| {
        compare_statefulsets_by_col(a, b, col_id)
    });
    rows
}

pub fn statefulsets_empty_message(state: &StatefulSetsState) -> &'static str {
    if state.statefulsets.is_empty() {
        "No statefulsets found in this cluster."
    } else {
        "No statefulsets found in the selected namespace(s)."
    }
}

pub fn statefulsets_presentation_mode(state: &StatefulSetsState) -> StatefulSetsPresentationMode {
    if state.status == StatefulSetsLoadStatus::Error && state.statefulsets.is_empty() {
        StatefulSetsPresentationMode::ErrorCard
    } else {
        StatefulSetsPresentationMode::Table
    }
}

fn render_statefulset_cell(
    row_idx: usize,
    ss: &StatefulSetInfo,
    col: &TableColumn,
    theme: &OrbitTheme,
) -> AnyElement {
    match col.id.as_ref() {
        "name" => render_truncated_cell(
            ElementId::NamedInteger(SharedString::from("ss-name"), row_idx as u64),
            ss.name.clone(),
            Some(SharedString::from(ss.name.clone())),
            theme.text.primary,
            FontWeight::MEDIUM,
            false,
            theme,
        ),
        "namespace" => render_truncated_cell(
            ElementId::NamedInteger(SharedString::from("ss-ns"), row_idx as u64),
            ss.namespace.clone(),
            None,
            theme.text.secondary,
            FontWeight::NORMAL,
            false,
            theme,
        ),
        "status" => render_status_badge_from_str(&ss.status, StatusBadgeVariant::Inline, theme),
        "replicas" => render_ratio_metric_cell(
            ss.replicas.current.to_string(),
            ss.replicas.desired.to_string(),
            theme,
        ),
        "images" => render_images_cell(
            ElementId::NamedInteger(SharedString::from("ss-img"), row_idx as u64),
            &ss.images,
            theme,
        ),
        "age" => apply_text_step(div(), theme.typography.size_xs)
            .font(theme.typography.mono_font(FontWeight::NORMAL))
            .text_color(theme.text.muted)
            .child(ss.age.clone())
            .into_any_element(),
        _ => div().into_any_element(),
    }
}

#[allow(clippy::too_many_arguments)]
pub fn build_statefulset_context_menu_entries(
    statefulset: &StatefulSetInfo,
    on_copy_name: impl 'static + Fn(&mut Window, &mut App),
    on_filter_namespace: impl 'static + Fn(&mut Window, &mut App),
    on_refresh: impl 'static + Fn(&mut Window, &mut App),
    on_view_details: impl 'static + Fn(&mut Window, &mut App),
    on_view_yaml: impl 'static + Fn(&mut Window, &mut App),
    on_view_logs: impl 'static + Fn(&mut Window, &mut App),
    on_scale: impl 'static + Fn(&mut Window, &mut App),
    on_restart: impl 'static + Fn(&mut Window, &mut App),
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
                format!("Filter by \"{}\"", statefulset.namespace),
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
        ContextMenuEntry::Separator,
        ContextMenuEntry::Item(
            ContextMenuItem::new("scale", "Scale...")
                .icon(Icon::SlidersHorizontal)
                .on_select(on_scale),
        ),
        ContextMenuEntry::Item(
            ContextMenuItem::new("restart", "Restart")
                .icon(Icon::RotateCw)
                .on_select(on_restart),
        ),
        ContextMenuEntry::Separator,
        ContextMenuEntry::Item(
            ContextMenuItem::new("delete-statefulset", "Delete StatefulSet")
                .icon(Icon::Trash2)
                .danger(true)
                .on_select(on_delete),
        ),
    ]
}

#[allow(clippy::too_many_arguments)]
pub fn render_statefulsets_view(
    state: &StatefulSetsState,
    namespaces: &NamespaceFilterState,
    theme: &OrbitTheme,
    on_sort: impl 'static + Fn(SharedString, &ClickEvent, &mut Window, &mut App),
    on_row_click: impl 'static + Fn(usize, SharedString, &ClickEvent, &mut Window, &mut App),
    on_row_quick_yaml: impl 'static + Fn(usize, SharedString, &ClickEvent, &mut Window, &mut App),
    on_row_context_menu: impl 'static
    + Fn(SharedString, &StatefulSetInfo, &MouseDownEvent, &mut Window, &mut App),
    on_refresh: impl 'static + Fn(&mut Window, &mut App),
) -> AnyElement {
    let columns = statefulset_columns();
    let filtered_rows: Rc<Vec<StatefulSetInfo>> = Rc::new(
        filter_and_sort_statefulsets(&state.statefulsets, namespaces, state.table.sort.as_ref())
            .into_iter()
            .cloned()
            .collect(),
    );
    let total_count = state.statefulsets.len();
    let visible_count = filtered_rows.len();
    let is_loading = state.status == StatefulSetsLoadStatus::Loading;
    let on_refresh_rc: super::ActionCallback = Rc::new(on_refresh);

    let count_label = if namespaces.is_all_namespaces() {
        format!("{total_count} statefulsets")
    } else {
        format!("{visible_count} of {total_count} statefulsets")
    };

    let header = render_resource_view_header(
        WorkspaceView::StatefulSets.label(),
        WorkspaceView::StatefulSets.description(),
        count_label,
        Some(namespaces),
        None,
        theme,
    );

    let error_banner = state.error_message.as_ref().map(|message| {
        render_resource_error_banner(
            "statefulsets-retry-btn",
            format!("Failed to load statefulsets: {message}"),
            is_loading,
            theme,
            on_refresh_rc.clone(),
        )
    });

    let body = match statefulsets_presentation_mode(state) {
        StatefulSetsPresentationMode::ErrorCard => {
            let message = state
                .error_message
                .clone()
                .unwrap_or_else(|| "Unknown error".to_string());
            render_resource_error_state(
                "statefulsets-retry-btn",
                "Unable to Load StatefulSets",
                message,
                is_loading,
                theme,
                on_refresh_rc,
            )
        }
        StatefulSetsPresentationMode::Table => {
            let mut table_state = state.table.clone();
            table_state.loading = is_loading && state.statefulsets.is_empty();
            table_state.empty_message = SharedString::from(statefulsets_empty_message(state));

            let key_rows = filtered_rows.clone();
            let cell_rows = filtered_rows.clone();
            let ctx_rows = filtered_rows.clone();
            let callbacks = DataTableCallbacks::new(
                move |idx| {
                    key_rows
                        .get(idx)
                        .map(|ss| format!("{}/{}", ss.namespace, ss.name))
                        .unwrap_or_default()
                },
                move |idx, col, col_theme| {
                    cell_rows
                        .get(idx)
                        .map(|ss| render_statefulset_cell(idx, ss, col, col_theme))
                        .unwrap_or_else(|| div().into_any_element())
                },
            )
            .on_sort(on_sort)
            .on_row_click(on_row_click)
            .on_row_quick_yaml(on_row_quick_yaml)
            .on_row_context_menu(move |idx, row_key, event, window, cx| {
                if let Some(statefulset) = ctx_rows.get(idx) {
                    on_row_context_menu(row_key, statefulset, event, window, cx);
                }
            });

            render_data_table(
                "statefulsets-table",
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
        .when(
            statefulsets_presentation_mode(state) == StatefulSetsPresentationMode::Table,
            |el| el.when_some(error_banner, |el, banner| el.child(banner)),
        )
        .child(body)
        .into_any_element()
}

pub fn filter_events_for_statefulset<'a>(
    events: &'a [EventInfo],
    statefulset: &StatefulSetInfo,
) -> Vec<&'a EventInfo> {
    let prefix = format!("{}-", statefulset.name);
    events
        .iter()
        .filter(|ev| {
            ev.namespace == statefulset.namespace
                && ((ev.object_kind == "StatefulSet" && ev.object_name == statefulset.name)
                    || (ev.object_kind == "Pod" && ev.object_name.starts_with(&prefix))
                    || ev.message.contains(&statefulset.name))
        })
        .collect()
}

pub fn render_statefulset_drawer_overview(
    statefulset: &StatefulSetInfo,
    theme: &OrbitTheme,
) -> AnyElement {
    let summary = crate::views::render_drawer_key_values(
        vec![
            ("Name", statefulset.name.clone(), true),
            ("Namespace", statefulset.namespace.clone(), true),
            ("Status", statefulset.status.clone(), false),
            (
                "Ready Replicas",
                format!(
                    "{}/{}",
                    statefulset.replicas.current, statefulset.replicas.desired
                ),
                true,
            ),
            ("Age", statefulset.age.clone(), true),
        ],
        theme,
    );

    let containers_section = crate::views::render_drawer_containers_section(
        &statefulset.containers,
        &statefulset.images,
        theme,
    );

    div()
        .id("statefulset-drawer-overview")
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
            &statefulset.labels,
            "No labels",
            theme,
        ))
        .child(crate::views::render_drawer_kv_badges(
            "ANNOTATIONS",
            &statefulset.annotations,
            "No annotations",
            theme,
        ))
        .into_any_element()
}

pub fn render_statefulset_drawer_events(
    events: &[EventInfo],
    loading: bool,
    statefulset: &StatefulSetInfo,
    theme: &OrbitTheme,
) -> AnyElement {
    let matched = filter_events_for_statefulset(events, statefulset);
    div()
        .id("statefulset-drawer-events")
        .flex_1()
        .min_h_0()
        .overflow_y_scroll()
        .p_4()
        .child(crate::views::render_drawer_events_list(
            &matched,
            loading,
            "No events found for this statefulset.",
            theme,
        ))
        .into_any_element()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::components::data_table::{ColumnAlign, SortDirection};
    use orbit_core::kubernetes::models::{ContainerImageInfo, NamespaceInfo, Replicas};

    fn make_statefulset(
        name: &str,
        namespace: &str,
        status: &str,
        current: i32,
        desired: i32,
        image: &str,
        age: &str,
    ) -> StatefulSetInfo {
        StatefulSetInfo {
            kind: "StatefulSet".to_string(),
            name: name.to_string(),
            namespace: namespace.to_string(),
            status: status.to_string(),
            replicas: Replicas { current, desired },
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
    fn test_statefulset_columns_schema_and_alignment() {
        let cols = statefulset_columns();
        let ids: Vec<&str> = cols.iter().map(|c| c.id.as_ref()).collect();
        assert_eq!(
            ids,
            vec!["name", "namespace", "status", "replicas", "images", "age"]
        );
        for idx in [3, 5] {
            assert_eq!(cols[idx].align, ColumnAlign::Right);
            assert!(cols[idx].monospace);
        }
        for idx in [0, 1, 2, 4] {
            assert_eq!(cols[idx].align, ColumnAlign::Left);
        }
    }

    #[test]
    fn test_filter_and_sort_statefulsets_by_namespace_and_columns() {
        let items = vec![
            make_statefulset("zk", "infra", "Running", 3, 3, "zookeeper:3.9", "5d"),
            make_statefulset("redis", "cache", "Progressing", 1, 3, "redis:7.2", "2h"),
            make_statefulset("pg", "cache", "Running", 2, 2, "postgres:16", "45m"),
        ];

        let mut ns = NamespaceFilterState::default();
        ns.begin_load_for_cluster("ctx-a");
        ns.apply_loaded(
            "ctx-a",
            1,
            vec![
                NamespaceInfo {
                    name: "cache".to_string(),
                    status: "Active".to_string(),
                    is_system: false,
                    age: "10d".to_string(),
                    labels: Default::default(),
                    annotations: Default::default(),
                    uid: "uid-cache".to_string(),
                    created: String::new(),
                },
                NamespaceInfo {
                    name: "infra".to_string(),
                    status: "Active".to_string(),
                    is_system: false,
                    age: "10d".to_string(),
                    labels: Default::default(),
                    annotations: Default::default(),
                    uid: "uid-infra".to_string(),
                    created: String::new(),
                },
            ],
        );
        ns.toggle_namespace("cache");

        let by_name = SortState {
            column_id: "name".to_string(),
            direction: SortDirection::Asc,
        };
        let filtered = filter_and_sort_statefulsets(&items, &ns, Some(&by_name));
        assert_eq!(
            filtered.iter().map(|s| s.name.as_str()).collect::<Vec<_>>(),
            vec!["pg", "redis"]
        );

        let all_ns = NamespaceFilterState::default();
        let by_status = SortState {
            column_id: "status".to_string(),
            direction: SortDirection::Asc,
        };
        let sorted_status = filter_and_sort_statefulsets(&items, &all_ns, Some(&by_status));
        assert_eq!(sorted_status[0].status, "Progressing");

        let by_replicas = SortState {
            column_id: "replicas".to_string(),
            direction: SortDirection::Desc,
        };
        let sorted_replicas = filter_and_sort_statefulsets(&items, &all_ns, Some(&by_replicas));
        assert_eq!(sorted_replicas[0].name, "zk");

        let by_images = SortState {
            column_id: "images".to_string(),
            direction: SortDirection::Asc,
        };
        let sorted_images = filter_and_sort_statefulsets(&items, &all_ns, Some(&by_images));
        assert_eq!(sorted_images[0].name, "pg");

        let by_age = SortState {
            column_id: "age".to_string(),
            direction: SortDirection::Asc,
        };
        let sorted_age = filter_and_sort_statefulsets(&items, &all_ns, Some(&by_age));
        assert_eq!(
            sorted_age
                .iter()
                .map(|s| s.name.as_str())
                .collect::<Vec<_>>(),
            vec!["pg", "redis", "zk"]
        );
    }

    #[test]
    fn test_statefulsets_view_empty_and_error_presentation_modes() {
        let mut state = StatefulSetsState::default();
        assert_eq!(
            statefulsets_empty_message(&state),
            "No statefulsets found in this cluster."
        );
        assert_eq!(
            statefulsets_presentation_mode(&state),
            StatefulSetsPresentationMode::Table
        );

        state.status = StatefulSetsLoadStatus::Error;
        state.error_message = Some("forbidden".to_string());
        assert_eq!(
            statefulsets_presentation_mode(&state),
            StatefulSetsPresentationMode::ErrorCard
        );

        state.statefulsets.push(make_statefulset(
            "redis",
            "cache",
            "Running",
            3,
            3,
            "redis:7.2",
            "1d",
        ));
        assert_eq!(
            statefulsets_empty_message(&state),
            "No statefulsets found in the selected namespace(s)."
        );
        assert_eq!(
            statefulsets_presentation_mode(&state),
            StatefulSetsPresentationMode::Table
        );
    }

    #[test]
    fn test_filter_events_for_statefulset_matches_namespace_and_pod_prefix() {
        let ss = make_statefulset("redis", "cache", "Running", 3, 3, "redis:7.2", "1d");
        let events = vec![
            EventInfo {
                uid: "1".to_string(),
                name: String::new(),
                time: "1m".to_string(),
                r#type: "Normal".to_string(),
                reason: "SuccessfulCreate".to_string(),
                object_kind: "StatefulSet".to_string(),
                object_name: "redis".to_string(),
                namespace: "cache".to_string(),
                message: "create Pod redis-0 in StatefulSet redis successful".to_string(),
                count: 1,
                first_seen: String::new(),
                last_seen: String::new(),
                source: "statefulset-controller".to_string(),
                labels: Default::default(),
                annotations: Default::default(),
            },
            EventInfo {
                uid: "2".to_string(),
                name: String::new(),
                time: "1m".to_string(),
                r#type: "Normal".to_string(),
                reason: "Started".to_string(),
                object_kind: "Pod".to_string(),
                object_name: "redis-0".to_string(),
                namespace: "cache".to_string(),
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
                name: String::new(),
                time: "1m".to_string(),
                r#type: "Warning".to_string(),
                reason: "Failed".to_string(),
                object_kind: "Pod".to_string(),
                object_name: "redis-0".to_string(),
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
                name: String::new(),
                time: "1m".to_string(),
                r#type: "Normal".to_string(),
                reason: "Scaling".to_string(),
                object_kind: "ConfigMap".to_string(),
                object_name: "cm".to_string(),
                namespace: "cache".to_string(),
                message: "Updated for redis".to_string(),
                count: 1,
                first_seen: String::new(),
                last_seen: String::new(),
                source: "controller".to_string(),
                labels: Default::default(),
                annotations: Default::default(),
            },
        ];

        let matched = filter_events_for_statefulset(&events, &ss);
        let uids: Vec<&str> = matched.iter().map(|e| e.uid.as_str()).collect();
        assert_eq!(uids, vec!["1", "2", "4"]);
    }

    #[test]
    fn test_build_statefulset_context_menu_entries_structure() {
        let ss = make_statefulset("db", "prod", "Running", 3, 3, "postgres:16", "1d");
        let entries = build_statefulset_context_menu_entries(
            &ss,
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
        let ids: Vec<&str> = entries
            .iter()
            .filter_map(|e| match e {
                ContextMenuEntry::Item(item) => Some(item.id.as_ref()),
                ContextMenuEntry::Separator => None,
            })
            .collect();
        assert_eq!(
            ids,
            vec![
                "copy-name",
                "filter-namespace",
                "refresh",
                "view-details",
                "view-yaml",
                "view-logs",
                "scale",
                "restart",
                "delete-statefulset",
            ]
        );
        let delete_item = entries
            .iter()
            .find_map(|e| match e {
                ContextMenuEntry::Item(item) if item.id.as_ref() == "delete-statefulset" => Some(item),
                _ => None,
            })
            .expect("delete-statefulset item");
        assert!(delete_item.danger);
        assert!(!delete_item.disabled);
        assert!(delete_item.on_select.is_some());
    }
}
