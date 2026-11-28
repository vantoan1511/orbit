use std::cmp::Ordering;
use std::rc::Rc;

use gpui::{
    AnyElement, App, ClickEvent, ElementId, FontWeight, MouseDownEvent, SharedString, Window, div,
    prelude::*, px,
};
use lucide_icons::Icon;
use orbit_core::kubernetes::configmaps::{format_size, parse_size_bytes};
use orbit_core::kubernetes::models::{ConfigMapInfo, EventInfo};

use crate::components::data_table::{
    DataTableCallbacks, SortState, TableColumn, render_data_table, sort_rows_stable,
};
use crate::components::{
    ContextMenuEntry, ContextMenuItem, SemanticStatus, StatusBadgeVariant, render_status_badge,
    render_status_badge_from_str,
};
use crate::state::{ConfigMapsLoadStatus, ConfigMapsState, NamespaceFilterState};
use crate::theme::{OrbitTheme, apply_text_step};
pub use crate::views::{
    nonzero_metric_text_color, parse_age_seconds, render_action_button,
    render_resource_error_banner, render_resource_error_state, render_resource_view_header,
    render_truncated_cell,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConfigMapsPresentationMode {
    Table,
    ErrorCard,
}

pub fn configmap_columns() -> Vec<TableColumn> {
    vec![
        TableColumn::new("name", "Name").flex_width(1.8, px(160.0)),
        TableColumn::new("namespace", "Namespace").flex_width(1.0, px(105.0)),
        TableColumn::new("keys_count", "Keys")
            .fixed_width(px(80.0))
            .right_numeric(),
        TableColumn::new("size", "Size")
            .fixed_width(px(96.0))
            .right_numeric(),
        TableColumn::new("mounted_pods", "Mounted Pods")
            .fixed_width(px(116.0))
            .right_numeric(),
        TableColumn::new("immutable", "Immutable").fixed_width(px(104.0)),
        TableColumn::new("age", "Age")
            .fixed_width(px(76.0))
            .right_numeric(),
    ]
}

pub fn compare_configmaps_by_col(a: &ConfigMapInfo, b: &ConfigMapInfo, col_id: &str) -> Ordering {
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
        "keys_count" => a.keys_count.cmp(&b.keys_count).then_with(tie_break),
        "size" => parse_size_bytes(&a.size)
            .cmp(&parse_size_bytes(&b.size))
            .then_with(tie_break),
        "mounted_pods" => a.mounted_pods.cmp(&b.mounted_pods).then_with(tie_break),
        "immutable" => a.immutable.cmp(&b.immutable).then_with(tie_break),
        "age" => parse_age_seconds(&a.age)
            .cmp(&parse_age_seconds(&b.age))
            .then_with(tie_break),
        _ => tie_break(),
    }
}

pub fn filter_and_sort_configmaps<'a>(
    configmaps: &'a [ConfigMapInfo],
    namespaces: &NamespaceFilterState,
    sort: Option<&SortState>,
) -> Vec<&'a ConfigMapInfo> {
    let mut rows: Vec<&'a ConfigMapInfo> = configmaps
        .iter()
        .filter(|cm| namespaces.matches_resource_namespace(Some(&cm.namespace)))
        .collect();
    sort_rows_stable(&mut rows, sort, |a, b, col_id| {
        compare_configmaps_by_col(a, b, col_id)
    });
    rows
}

pub fn configmaps_empty_message(state: &ConfigMapsState) -> &'static str {
    if state.configmaps.is_empty() {
        "No configmaps found in this cluster."
    } else {
        "No configmaps found in the selected namespace(s)."
    }
}

pub fn configmaps_presentation_mode(state: &ConfigMapsState) -> ConfigMapsPresentationMode {
    if state.status == ConfigMapsLoadStatus::Error && state.configmaps.is_empty() {
        ConfigMapsPresentationMode::ErrorCard
    } else {
        ConfigMapsPresentationMode::Table
    }
}

fn render_configmap_cell(
    row_idx: usize,
    cm: &ConfigMapInfo,
    col: &TableColumn,
    theme: &OrbitTheme,
) -> AnyElement {
    match col.id.as_ref() {
        "name" => render_truncated_cell(
            ElementId::NamedInteger(SharedString::from("cm-name"), row_idx as u64),
            cm.name.clone(),
            Some(SharedString::from(cm.name.clone())),
            theme.text.primary,
            FontWeight::MEDIUM,
            false,
            theme,
        ),
        "namespace" => render_truncated_cell(
            ElementId::NamedInteger(SharedString::from("cm-ns"), row_idx as u64),
            cm.namespace.clone(),
            None,
            theme.text.secondary,
            FontWeight::NORMAL,
            false,
            theme,
        ),
        "keys_count" => apply_text_step(div(), theme.typography.size_xs)
            .font(theme.typography.mono_font(FontWeight::NORMAL))
            .text_color(if cm.keys_count == 0 {
                theme.text.muted
            } else {
                theme.text.primary
            })
            .child(cm.keys_count.to_string())
            .into_any_element(),
        "size" => apply_text_step(div(), theme.typography.size_xs)
            .font(theme.typography.mono_font(FontWeight::NORMAL))
            .text_color(theme.text.secondary)
            .child(cm.size.clone())
            .into_any_element(),
        "mounted_pods" => {
            let label = if cm.mounted_pods == 1 {
                "1 pod".to_string()
            } else {
                format!("{} pods", cm.mounted_pods)
            };
            apply_text_step(div(), theme.typography.size_xs)
                .font(theme.typography.mono_font(FontWeight::NORMAL))
                .text_color(nonzero_metric_text_color(cm.mounted_pods, theme))
                .child(label)
                .into_any_element()
        }
        "immutable" => {
            if cm.immutable {
                render_status_badge(
                    "Immutable",
                    SemanticStatus::Info,
                    StatusBadgeVariant::Inline,
                    theme,
                )
            } else {
                apply_text_step(div(), theme.typography.size_xs)
                    .text_color(theme.text.muted)
                    .child("Mutable")
                    .into_any_element()
            }
        }
        "age" => apply_text_step(div(), theme.typography.size_xs)
            .font(theme.typography.mono_font(FontWeight::NORMAL))
            .text_color(theme.text.muted)
            .child(cm.age.clone())
            .into_any_element(),
        _ => div().into_any_element(),
    }
}

pub fn build_configmap_context_menu_entries(
    cm: &ConfigMapInfo,
    on_copy_name: impl 'static + Fn(&mut Window, &mut App),
    on_filter_namespace: impl 'static + Fn(&mut Window, &mut App),
    on_refresh: impl 'static + Fn(&mut Window, &mut App),
    on_view_details: impl 'static + Fn(&mut Window, &mut App),
    on_view_yaml: impl 'static + Fn(&mut Window, &mut App),
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
                format!("Filter by \"{}\"", cm.namespace),
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
            ContextMenuItem::new("delete-configmap", "Delete ConfigMap")
                .icon(Icon::Trash2)
                .danger(true)
                .on_select(on_delete),
        ),
    ]
}

#[allow(clippy::too_many_arguments)]
pub fn render_configmaps_view(
    state: &ConfigMapsState,
    namespaces: &NamespaceFilterState,
    theme: &OrbitTheme,
    on_sort: impl 'static + Fn(SharedString, &ClickEvent, &mut Window, &mut App),
    on_row_click: impl 'static + Fn(usize, SharedString, &ClickEvent, &mut Window, &mut App),
    on_row_quick_yaml: impl 'static + Fn(usize, SharedString, &ClickEvent, &mut Window, &mut App),
    on_row_context_menu: impl 'static + Fn(SharedString, &ConfigMapInfo, &MouseDownEvent, &mut Window, &mut App),
    on_refresh: impl 'static + Fn(&mut Window, &mut App),
) -> AnyElement {
    let columns = configmap_columns();
    let cache_key = crate::state::RowCacheKey::from_table(
        state.data_revision,
        namespaces.selection_revision,
        &state.table,
        0,
    );
    let filtered_rows = state.row_cache.get_or_compute(cache_key, || {
        filter_and_sort_configmaps(&state.configmaps, namespaces, state.table.sort.as_ref())
            .into_iter()
            .cloned()
            .collect()
    });
    let total_count = state.configmaps.len();
    let visible_count = filtered_rows.len();
    let is_loading = state.status == ConfigMapsLoadStatus::Loading;
    let on_refresh_rc: super::ActionCallback = Rc::new(on_refresh);

    let count_label = if namespaces.is_all_namespaces() {
        format!("{total_count} configmaps")
    } else {
        format!("{visible_count} of {total_count} configmaps")
    };

    let header = render_resource_view_header(count_label, Some(namespaces), None, theme);

    let error_banner = state.error_message.as_ref().map(|message| {
        render_resource_error_banner(
            "configmaps-retry-btn",
            format!("Failed to load configmaps: {message}"),
            is_loading,
            theme,
            on_refresh_rc.clone(),
        )
    });

    let mode = configmaps_presentation_mode(state);
    let body = match mode {
        ConfigMapsPresentationMode::ErrorCard => {
            let message = state
                .error_message
                .clone()
                .unwrap_or_else(|| "Unknown error".to_string());
            render_resource_error_state(
                "configmaps-retry-btn",
                "Unable to Load ConfigMaps",
                message,
                is_loading,
                theme,
                on_refresh_rc,
            )
        }
        ConfigMapsPresentationMode::Table => {
            let mut table_state = state.table.clone();
            table_state.loading = is_loading && state.configmaps.is_empty();
            table_state.refreshing = is_loading && !state.configmaps.is_empty();
            table_state.empty_message = SharedString::from(configmaps_empty_message(state));

            let key_rows = filtered_rows.clone();
            let cell_rows = filtered_rows.clone();
            let ctx_rows = filtered_rows.clone();
            let callbacks = DataTableCallbacks::new(
                move |idx| {
                    key_rows
                        .get(idx)
                        .map(|c| format!("{}/{}", c.namespace, c.name))
                        .unwrap_or_default()
                },
                move |idx, col, col_theme| {
                    cell_rows
                        .get(idx)
                        .map(|c| render_configmap_cell(idx, c, col, col_theme))
                        .unwrap_or_else(|| div().into_any_element())
                },
            )
            .on_sort(on_sort)
            .on_row_click(on_row_click)
            .on_row_quick_yaml(on_row_quick_yaml)
            .on_row_context_menu(move |idx, row_key, event, window, cx| {
                if let Some(cm) = ctx_rows.get(idx) {
                    on_row_context_menu(row_key, cm, event, window, cx);
                }
            });

            render_data_table(
                "configmaps-table",
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
        .when(mode == ConfigMapsPresentationMode::Table, |el| {
            el.when_some(error_banner, |el, banner| el.child(banner))
        })
        .child(body)
        .into_any_element()
}

pub fn filter_events_for_configmap<'a>(
    events: &'a [EventInfo],
    cm: &ConfigMapInfo,
) -> Vec<&'a EventInfo> {
    events
        .iter()
        .filter(|ev| {
            ev.namespace == cm.namespace
                && ((ev.object_kind == "ConfigMap" && ev.object_name == cm.name)
                    || ev.message.contains(&cm.name))
        })
        .collect()
}

pub fn render_configmap_drawer_overview(cm: &ConfigMapInfo, theme: &OrbitTheme) -> AnyElement {
    let summary = crate::views::render_drawer_key_values(
        vec![
            ("Name", cm.name.clone(), true),
            ("Namespace", cm.namespace.clone(), true),
            ("Keys", cm.keys_count.to_string(), true),
            ("Size", cm.size.clone(), true),
            (
                "Immutable",
                if cm.immutable { "Yes" } else { "No" }.to_string(),
                false,
            ),
            ("Mounted Pods", cm.mounted_pods.to_string(), true),
            (
                "Resource Version",
                if cm.resource_version.is_empty() {
                    "-".to_string()
                } else {
                    cm.resource_version.clone()
                },
                true,
            ),
            ("Age", cm.age.clone(), true),
        ],
        theme,
    );

    let mono = theme.typography.mono_font(FontWeight::NORMAL);
    let used_by_section = {
        let content = if cm.used_by.is_empty() {
            apply_text_step(div(), theme.typography.size_xs)
                .text_color(theme.text.muted)
                .child("Not mounted by any pods.")
                .into_any_element()
        } else {
            div()
                .w_full()
                .flex()
                .flex_col()
                .gap_1p5()
                .children(cm.used_by.iter().map(|pod| {
                    div()
                        .w_full()
                        .px_2p5()
                        .py_1p5()
                        .rounded(theme.style.radius_control)
                        .bg(theme.surface.bg_panel)
                        .border_1()
                        .border_color(theme.border.default)
                        .flex()
                        .items_center()
                        .justify_between()
                        .gap_2()
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .gap_2()
                                .min_w(px(0.0))
                                .child(
                                    apply_text_step(div(), theme.typography.size_xs)
                                        .font(mono.clone())
                                        .font_weight(FontWeight::MEDIUM)
                                        .text_color(theme.text.primary)
                                        .truncate()
                                        .child(pod.name.clone()),
                                )
                                .child(render_status_badge_from_str(
                                    &pod.status,
                                    StatusBadgeVariant::Inline,
                                    theme,
                                )),
                        )
                        .child(
                            apply_text_step(div(), theme.typography.size_2xs)
                                .px_1p5()
                                .py_0p5()
                                .rounded(theme.style.radius_control)
                                .bg(theme.surface.control_bg)
                                .border_1()
                                .border_color(theme.border.default)
                                .font(mono.clone())
                                .text_color(theme.text.secondary)
                                .flex_shrink_0()
                                .child(pod.mount_type.clone()),
                        )
                }))
                .into_any_element()
        };
        crate::views::render_drawer_section("USED BY PODS", content, theme)
    };

    div()
        .id("configmap-drawer-overview")
        .flex_1()
        .min_h_0()
        .overflow_y_scroll()
        .p_4()
        .flex()
        .flex_col()
        .gap_4()
        .child(crate::views::render_drawer_section("SUMMARY", summary, theme))
        .child(used_by_section)
        .child(crate::views::render_drawer_kv_badges(
            "LABELS",
            &cm.labels,
            "No labels",
            theme,
        ))
        .into_any_element()
}

pub fn render_configmap_drawer_data(cm: &ConfigMapInfo, theme: &OrbitTheme) -> AnyElement {
    let mono = theme.typography.mono_font(FontWeight::NORMAL);
    let content = if cm.data.is_empty() {
        apply_text_step(div(), theme.typography.size_xs)
            .text_color(theme.text.muted)
            .child("No data entries in this ConfigMap.")
            .into_any_element()
    } else {
        div()
            .w_full()
            .flex()
            .flex_col()
            .gap_3()
            .children(cm.data.iter().map(|(key, value)| {
                div()
                    .w_full()
                    .rounded(theme.style.radius_control)
                    .bg(theme.surface.bg_panel)
                    .border_1()
                    .border_color(theme.border.default)
                    .flex()
                    .flex_col()
                    .overflow_hidden()
                    .child(
                        div()
                            .px_3()
                            .py_1p5()
                            .bg(theme.surface.control_bg)
                            .border_b_1()
                            .border_color(theme.border.default)
                            .flex()
                            .items_center()
                            .justify_between()
                            .child(
                                apply_text_step(div(), theme.typography.size_xs)
                                    .font(mono.clone())
                                    .font_weight(FontWeight::SEMIBOLD)
                                    .text_color(theme.text.primary)
                                    .truncate()
                                    .child(key.clone()),
                            )
                            .child(
                                apply_text_step(div(), theme.typography.size_2xs)
                                    .font(mono.clone())
                                    .text_color(theme.text.muted)
                                    .child(format_size(value.len())),
                            ),
                    )
                    .child(
                        div().p_3().child(
                            apply_text_step(div(), theme.typography.size_xs)
                                .font(mono.clone())
                                .text_color(theme.text.secondary)
                                .child(value.clone()),
                        ),
                    )
            }))
            .into_any_element()
    };

    div()
        .id("configmap-drawer-data")
        .flex_1()
        .min_h_0()
        .overflow_y_scroll()
        .p_4()
        .child(crate::views::render_drawer_section(
            "DATA ENTRIES",
            content,
            theme,
        ))
        .into_any_element()
}

pub fn render_configmap_drawer_events(
    events: &[EventInfo],
    loading: bool,
    cm: &ConfigMapInfo,
    theme: &OrbitTheme,
) -> AnyElement {
    let matched = filter_events_for_configmap(events, cm);
    div()
        .id("configmap-drawer-events")
        .flex_1()
        .min_h_0()
        .overflow_y_scroll()
        .p_4()
        .child(crate::views::render_drawer_events_list(
            &matched,
            loading,
            "No events found for this configmap.",
            theme,
        ))
        .into_any_element()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::components::data_table::{ColumnAlign, SortDirection};
    use std::collections::BTreeMap;

    fn make_configmap(
        name: &str,
        namespace: &str,
        keys_count: i32,
        size: &str,
        mounted_pods: i32,
        immutable: bool,
        age: &str,
    ) -> ConfigMapInfo {
        ConfigMapInfo {
            name: name.to_string(),
            namespace: namespace.to_string(),
            labels: BTreeMap::new(),
            annotations: 0,
            created: String::new(),
            age: age.to_string(),
            resource_version: "1".to_string(),
            immutable,
            keys_count,
            size: size.to_string(),
            mounted_pods,
            used_by: Vec::new(),
            data: BTreeMap::new(),
        }
    }

    #[test]
    fn test_configmap_columns_cover_required_fields_and_alignment() {
        let cols = configmap_columns();
        let ids: Vec<&str> = cols.iter().map(|c| c.id.as_ref()).collect();
        assert_eq!(
            ids,
            vec![
                "name",
                "namespace",
                "keys_count",
                "size",
                "mounted_pods",
                "immutable",
                "age",
            ]
        );

        for right_col in ["keys_count", "size", "mounted_pods", "age"] {
            let col = cols
                .iter()
                .find(|c| c.id.as_ref() == right_col)
                .unwrap_or_else(|| panic!("missing column {right_col}"));
            assert_eq!(col.align, ColumnAlign::Right, "{right_col} should be right-aligned");
            assert!(col.monospace, "{right_col} should be monospace");
        }
    }

    #[test]
    fn test_filter_and_sort_configmaps_respects_namespaces_and_numeric_columns() {
        let cms = vec![
            make_configmap("large-cm", "prod", 10, "2.0 MiB", 3, true, "2d"),
            make_configmap("small-cm", "prod", 1, "512 B", 0, false, "10m"),
            make_configmap("mid-cm", "prod", 4, "1.5 KiB", 1, false, "3h"),
            make_configmap("dev-cm", "dev", 2, "64 B", 1, false, "1h"),
        ];

        let mut ns = NamespaceFilterState::default();
        ns.begin_load_for_cluster("ctx-a");
        ns.apply_loaded(
            "ctx-a",
            1,
            vec![
                orbit_core::kubernetes::models::NamespaceInfo {
                    name: "prod".to_string(),
                    status: "Active".to_string(),
                    age: "1d".to_string(),
                    is_system: false,
                    labels: BTreeMap::new(),
                    annotations: BTreeMap::new(),
                    uid: "ns-prod".to_string(),
                    created: String::new(),
                },
                orbit_core::kubernetes::models::NamespaceInfo {
                    name: "dev".to_string(),
                    status: "Active".to_string(),
                    age: "1d".to_string(),
                    is_system: false,
                    labels: BTreeMap::new(),
                    annotations: BTreeMap::new(),
                    uid: "ns-dev".to_string(),
                    created: String::new(),
                },
            ],
        );
        ns.toggle_namespace("prod");

        let by_size = filter_and_sort_configmaps(
            &cms,
            &ns,
            Some(&SortState {
                column_id: "size".to_string(),
                direction: SortDirection::Asc,
            }),
        );
        let size_names: Vec<&str> = by_size.iter().map(|c| c.name.as_str()).collect();
        assert_eq!(size_names, vec!["small-cm", "mid-cm", "large-cm"]);

        let by_keys = filter_and_sort_configmaps(
            &cms,
            &ns,
            Some(&SortState {
                column_id: "keys_count".to_string(),
                direction: SortDirection::Desc,
            }),
        );
        let key_names: Vec<&str> = by_keys.iter().map(|c| c.name.as_str()).collect();
        assert_eq!(key_names, vec!["large-cm", "mid-cm", "small-cm"]);

        let by_pods = filter_and_sort_configmaps(
            &cms,
            &ns,
            Some(&SortState {
                column_id: "mounted_pods".to_string(),
                direction: SortDirection::Desc,
            }),
        );
        assert_eq!(by_pods[0].name, "large-cm");
        assert_eq!(by_pods[2].name, "small-cm");

        let by_immutable = filter_and_sort_configmaps(
            &cms,
            &ns,
            Some(&SortState {
                column_id: "immutable".to_string(),
                direction: SortDirection::Desc,
            }),
        );
        assert_eq!(by_immutable[0].name, "large-cm");

        let by_age = filter_and_sort_configmaps(
            &cms,
            &ns,
            Some(&SortState {
                column_id: "age".to_string(),
                direction: SortDirection::Asc,
            }),
        );
        assert_eq!(
            by_age.iter().map(|c| c.name.as_str()).collect::<Vec<_>>(),
            vec!["small-cm", "mid-cm", "large-cm"]
        );
    }

    #[test]
    fn test_configmaps_presentation_mode_and_empty_messages() {
        let mut state = ConfigMapsState::default();
        assert_eq!(
            configmaps_empty_message(&state),
            "No configmaps found in this cluster."
        );
        assert_eq!(
            configmaps_presentation_mode(&state),
            ConfigMapsPresentationMode::Table
        );

        state.status = ConfigMapsLoadStatus::Error;
        state.error_message = Some("forbidden".to_string());
        assert_eq!(
            configmaps_presentation_mode(&state),
            ConfigMapsPresentationMode::ErrorCard
        );

        state.configmaps.push(make_configmap(
            "cm-1", "default", 1, "10 B", 0, false, "1m",
        ));
        assert_eq!(
            configmaps_presentation_mode(&state),
            ConfigMapsPresentationMode::Table
        );
        assert_eq!(
            configmaps_empty_message(&state),
            "No configmaps found in the selected namespace(s)."
        );
    }

    #[test]
    fn test_build_configmap_context_menu_entries_and_event_matching() {
        let cm = make_configmap("app-config", "prod", 2, "128 B", 1, false, "1h");
        let entries = build_configmap_context_menu_entries(
            &cm,
            |_, _| {},
            |_, _| {},
            |_, _| {},
            |_, _| {},
            |_, _| {},
            |_, _| {},
        );
        let item_ids: Vec<&str> = entries
            .iter()
            .filter_map(|e| match e {
                ContextMenuEntry::Item(item) => Some(item.id.as_ref()),
                ContextMenuEntry::Separator => None,
            })
            .collect();
        assert_eq!(
            item_ids,
            vec![
                "copy-name",
                "filter-namespace",
                "refresh",
                "view-details",
                "view-yaml",
                "delete-configmap",
            ]
        );
        let delete_item = entries
            .iter()
            .find_map(|e| match e {
                ContextMenuEntry::Item(item) if item.id.as_ref() == "delete-configmap" => Some(item),
                _ => None,
            })
            .expect("delete-configmap item");
        assert!(delete_item.danger);
        assert!(!delete_item.disabled);
        assert!(delete_item.on_select.is_some());

        let events = vec![
            EventInfo {
                uid: "1".to_string(),
                name: String::new(),
                r#type: "Normal".to_string(),
                reason: "Updated".to_string(),
                object_kind: "ConfigMap".to_string(),
                object_name: "app-config".to_string(),
                namespace: "prod".to_string(),
                message: "ConfigMap updated".to_string(),
                count: 1,
                time: "1m".to_string(),
                first_seen: String::new(),
                last_seen: "1m".to_string(),
                source: String::new(),
                labels: BTreeMap::new(),
                annotations: BTreeMap::new(),
            },
            EventInfo {
                uid: "2".to_string(),
                name: String::new(),
                r#type: "Normal".to_string(),
                reason: "Updated".to_string(),
                object_kind: "ConfigMap".to_string(),
                object_name: "app-config".to_string(),
                namespace: "dev".to_string(),
                message: "Other namespace".to_string(),
                count: 1,
                time: "1m".to_string(),
                first_seen: String::new(),
                last_seen: "1m".to_string(),
                source: String::new(),
                labels: BTreeMap::new(),
                annotations: BTreeMap::new(),
            },
        ];
        let matched = filter_events_for_configmap(&events, &cm);
        assert_eq!(matched.len(), 1);
        assert_eq!(matched[0].uid, "1");
    }
}
