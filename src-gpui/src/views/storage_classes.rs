use std::cmp::Ordering;
use std::rc::Rc;

use gpui::{
    AnyElement, App, ClickEvent, ElementId, FontWeight, MouseDownEvent, SharedString, Window, div,
    prelude::*, px,
};
use lucide_icons::Icon;
use orbit_core::kubernetes::models::{EventInfo, StorageClassInfo};

use crate::components::data_table::{
    DataTableCallbacks, SortState, TableColumn, render_data_table, sort_rows_stable,
};
use crate::components::{ContextMenuEntry, ContextMenuItem};
use crate::state::{StorageClassesLoadStatus, StorageClassesState};
use crate::theme::{OrbitTheme, apply_text_step};
use crate::views::{
    parse_age_seconds, render_drawer_events_list, render_drawer_key_values, render_drawer_section,
    render_resource_error_banner, render_resource_error_state, render_resource_view_header,
    render_truncated_cell, table_numeric_font,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StorageClassesPresentationMode {
    Table,
    ErrorCard,
}

pub fn storage_class_columns() -> Vec<TableColumn> {
    vec![
        TableColumn::new("name", "Name").flex_width(1.5, px(150.0)),
        TableColumn::new("provisioner", "Provisioner")
            .flex_width(1.8, px(180.0))
            .monospace(true),
        TableColumn::new("reclaim_policy", "Reclaim Policy").fixed_width(px(120.0)),
        TableColumn::new("volume_binding_mode", "Volume Binding Mode").flex_width(1.3, px(150.0)),
        TableColumn::new("expansion", "Expansion").fixed_width(px(96.0)),
        TableColumn::new("age", "Age")
            .fixed_width(px(76.0))
            .right_numeric(),
    ]
}

pub fn compare_storage_classes_by_col(
    a: &StorageClassInfo,
    b: &StorageClassInfo,
    col_id: &str,
) -> Ordering {
    let tie_break = || {
        a.name
            .to_ascii_lowercase()
            .cmp(&b.name.to_ascii_lowercase())
    };

    match col_id {
        "name" => tie_break(),
        "provisioner" => a
            .provisioner
            .to_ascii_lowercase()
            .cmp(&b.provisioner.to_ascii_lowercase())
            .then_with(tie_break),
        "reclaim_policy" => a
            .reclaim_policy
            .to_ascii_lowercase()
            .cmp(&b.reclaim_policy.to_ascii_lowercase())
            .then_with(tie_break),
        "volume_binding_mode" => a
            .volume_binding_mode
            .to_ascii_lowercase()
            .cmp(&b.volume_binding_mode.to_ascii_lowercase())
            .then_with(tie_break),
        "expansion" => a
            .allow_volume_expansion
            .cmp(&b.allow_volume_expansion)
            .then_with(tie_break),
        "age" => parse_age_seconds(&a.age)
            .cmp(&parse_age_seconds(&b.age))
            .then_with(tie_break),
        _ => tie_break(),
    }
}

pub fn filter_and_sort_storage_classes<'a>(
    classes: &'a [StorageClassInfo],
    search: &str,
    sort: Option<&SortState>,
) -> Vec<&'a StorageClassInfo> {
    let query = search.trim().to_ascii_lowercase();
    let mut rows: Vec<&'a StorageClassInfo> = classes
        .iter()
        .filter(|sc| {
            if query.is_empty() {
                return true;
            }
            sc.name.to_ascii_lowercase().contains(&query)
                || sc.provisioner.to_ascii_lowercase().contains(&query)
                || sc.reclaim_policy.to_ascii_lowercase().contains(&query)
                || sc.volume_binding_mode.to_ascii_lowercase().contains(&query)
        })
        .collect();

    if sort.is_some() {
        sort_rows_stable(&mut rows, sort, |a, b, col_id| {
            compare_storage_classes_by_col(a, b, col_id)
        });
    } else {
        rows.sort_by(|a, b| compare_storage_classes_by_col(a, b, "name"));
    }
    rows
}

pub fn storage_classes_empty_message(state: &StorageClassesState) -> &'static str {
    if state.classes.is_empty() {
        "No storage classes found in this cluster."
    } else {
        "No storage classes match the current filter."
    }
}

pub fn storage_classes_presentation_mode(
    state: &StorageClassesState,
) -> StorageClassesPresentationMode {
    if state.status == StorageClassesLoadStatus::Error && state.classes.is_empty() {
        StorageClassesPresentationMode::ErrorCard
    } else {
        StorageClassesPresentationMode::Table
    }
}

fn render_storage_class_cell(
    row_idx: usize,
    sc: &StorageClassInfo,
    col: &TableColumn,
    theme: &OrbitTheme,
) -> AnyElement {
    match col.id.as_ref() {
        "name" => render_truncated_cell(
            ElementId::NamedInteger(SharedString::from("sc-name"), row_idx as u64),
            sc.name.clone(),
            Some(SharedString::from(sc.name.clone())),
            theme.text.primary,
            FontWeight::MEDIUM,
            false,
            theme,
        ),
        "provisioner" => render_truncated_cell(
            ElementId::NamedInteger(SharedString::from("sc-prov"), row_idx as u64),
            if sc.provisioner.trim().is_empty() {
                "-".to_string()
            } else {
                sc.provisioner.clone()
            },
            (!sc.provisioner.trim().is_empty()).then(|| SharedString::from(sc.provisioner.clone())),
            theme.text.secondary,
            FontWeight::NORMAL,
            true,
            theme,
        ),
        "reclaim_policy" => render_truncated_cell(
            ElementId::NamedInteger(SharedString::from("sc-reclaim"), row_idx as u64),
            if sc.reclaim_policy.trim().is_empty() {
                "-".to_string()
            } else {
                sc.reclaim_policy.clone()
            },
            None,
            theme.text.secondary,
            FontWeight::NORMAL,
            false,
            theme,
        ),
        "volume_binding_mode" => render_truncated_cell(
            ElementId::NamedInteger(SharedString::from("sc-binding"), row_idx as u64),
            if sc.volume_binding_mode.trim().is_empty() {
                "-".to_string()
            } else {
                sc.volume_binding_mode.clone()
            },
            None,
            theme.text.secondary,
            FontWeight::NORMAL,
            false,
            theme,
        ),
        "expansion" => {
            let (label, color) = if sc.allow_volume_expansion {
                ("Yes", theme.semantic.success.solid)
            } else {
                ("No", theme.text.muted)
            };
            apply_text_step(div(), theme.typography.size_xs)
                .font_weight(FontWeight::MEDIUM)
                .text_color(color)
                .child(label)
                .into_any_element()
        }
        "age" => apply_text_step(div(), theme.typography.size_xs)
            .font(table_numeric_font(theme))
            .text_color(theme.text.muted)
            .child(sc.age.clone())
            .into_any_element(),
        _ => div().into_any_element(),
    }
}

pub fn build_storage_class_context_menu_entries(
    on_copy_name: impl 'static + Fn(&mut Window, &mut App),
    on_refresh: impl 'static + Fn(&mut Window, &mut App),
    on_view_details: impl 'static + Fn(&mut Window, &mut App),
    on_view_yaml: impl 'static + Fn(&mut Window, &mut App),
    on_edit: impl 'static + Fn(&mut Window, &mut App),
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
        ContextMenuEntry::Item(
            ContextMenuItem::new("edit", "Edit")
                .icon(Icon::Pencil)
                .on_select(on_edit),
        ),
    ]
}

#[allow(clippy::too_many_arguments)]
pub fn render_storage_classes_view(
    state: &StorageClassesState,
    theme: &OrbitTheme,
    on_sort: impl 'static + Fn(SharedString, &ClickEvent, &mut Window, &mut App),
    on_row_click: impl 'static + Fn(usize, SharedString, &ClickEvent, &mut Window, &mut App),
    on_row_quick_yaml: impl 'static + Fn(usize, SharedString, &ClickEvent, &mut Window, &mut App),
    on_row_context_menu: impl 'static
    + Fn(
        SharedString,
        &StorageClassInfo,
        &MouseDownEvent,
        &mut Window,
        &mut App,
    ),
    on_refresh: impl 'static + Fn(&mut Window, &mut App),
) -> AnyElement {
    let columns = storage_class_columns();
    let cache_key =
        crate::state::RowCacheKey::from_table(state.data_revision, 0, &state.table, 0);
    let filtered_rows = state.row_cache.get_or_compute(cache_key, || {
        filter_and_sort_storage_classes(&state.classes, "", state.table.sort.as_ref())
            .into_iter()
            .cloned()
            .collect()
    });
    let total_count = state.classes.len();
    let visible_count = filtered_rows.len();
    let is_loading = state.status == StorageClassesLoadStatus::Loading;
    let presentation_mode = storage_classes_presentation_mode(state);
    let on_refresh_rc: super::ActionCallback = Rc::new(on_refresh);

    let count_label = format!("{total_count} storage classes");
    let header = render_resource_view_header(count_label, None, None, theme);

    let error_banner = if presentation_mode == StorageClassesPresentationMode::Table {
        state.error_message.as_ref().map(|message| {
            render_resource_error_banner(
                "storage-classes-retry-btn",
                format!("Failed to load storage classes: {message}"),
                is_loading,
                theme,
                on_refresh_rc.clone(),
            )
        })
    } else {
        None
    };

    let body = match presentation_mode {
        StorageClassesPresentationMode::ErrorCard => {
            let message = state
                .error_message
                .clone()
                .unwrap_or_else(|| "Unknown error".to_string());
            render_resource_error_state(
                "storage-classes-retry-btn",
                "Unable to Load Storage Classes",
                message,
                is_loading,
                theme,
                on_refresh_rc,
            )
        }
        StorageClassesPresentationMode::Table => {
            let mut table_state = state.table.clone();
            table_state.loading = is_loading && state.classes.is_empty();
            table_state.refreshing = is_loading && !state.classes.is_empty();
            table_state.empty_message = SharedString::from(storage_classes_empty_message(state));

            let key_rows = filtered_rows.clone();
            let cell_rows = filtered_rows.clone();
            let ctx_rows = filtered_rows.clone();
            let callbacks = DataTableCallbacks::new(
                move |idx| {
                    key_rows
                        .get(idx)
                        .map(|sc| sc.name.clone())
                        .unwrap_or_default()
                },
                move |idx, col, col_theme| {
                    cell_rows
                        .get(idx)
                        .map(|sc| render_storage_class_cell(idx, sc, col, col_theme))
                        .unwrap_or_else(|| div().into_any_element())
                },
            )
            .on_sort(on_sort)
            .on_row_click(on_row_click)
            .on_row_quick_yaml(on_row_quick_yaml)
            .on_row_context_menu(move |idx, row_key, event, window, cx| {
                if let Some(sc) = ctx_rows.get(idx) {
                    on_row_context_menu(row_key, sc, event, window, cx);
                }
            });

            render_data_table(
                "storage-classes-table",
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

pub fn filter_events_for_storage_class<'a>(
    events: &'a [EventInfo],
    sc: &StorageClassInfo,
) -> Vec<&'a EventInfo> {
    events
        .iter()
        .filter(|ev| ev.object_kind == "StorageClass" && ev.object_name == sc.name)
        .collect()
}

pub fn render_storage_class_drawer_overview(
    sc: &StorageClassInfo,
    theme: &OrbitTheme,
) -> AnyElement {
    let info_rows = vec![
        ("Name", sc.name.clone(), true),
        ("Provisioner", sc.provisioner.clone(), true),
        ("Reclaim Policy", sc.reclaim_policy.clone(), false),
        ("Volume Binding Mode", sc.volume_binding_mode.clone(), false),
        (
            "Allow Volume Expansion",
            if sc.allow_volume_expansion {
                "Yes".to_string()
            } else {
                "No".to_string()
            },
            false,
        ),
        ("Age", sc.age.clone(), true),
    ];

    div()
        .w_full()
        .flex()
        .flex_col()
        .gap_4()
        .child(render_drawer_section(
            "STORAGE CLASS INFORMATION",
            render_drawer_key_values(info_rows, theme),
            theme,
        ))
        .into_any_element()
}

pub fn render_storage_class_drawer_events(
    events: &[&EventInfo],
    loading: bool,
    theme: &OrbitTheme,
) -> AnyElement {
    render_drawer_events_list(
        events,
        loading,
        "No recent events for this storage class.",
        theme,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::components::data_table::{ColumnAlign, SortDirection};

    fn sample_sc(
        name: &str,
        provisioner: &str,
        reclaim_policy: &str,
        volume_binding_mode: &str,
        allow_volume_expansion: bool,
        age: &str,
    ) -> StorageClassInfo {
        StorageClassInfo {
            name: name.to_string(),
            provisioner: provisioner.to_string(),
            reclaim_policy: reclaim_policy.to_string(),
            volume_binding_mode: volume_binding_mode.to_string(),
            allow_volume_expansion,
            age: age.to_string(),
        }
    }

    #[test]
    fn test_storage_class_columns_cover_required_fields() {
        let cols = storage_class_columns();
        let ids: Vec<&str> = cols.iter().map(|c| c.id.as_ref()).collect();
        assert_eq!(
            ids,
            vec![
                "name",
                "provisioner",
                "reclaim_policy",
                "volume_binding_mode",
                "expansion",
                "age",
            ]
        );
        let prov = cols.iter().find(|c| c.id.as_ref() == "provisioner");
        assert!(prov.is_some_and(|c| c.monospace));
        let age = cols.iter().find(|c| c.id.as_ref() == "age");
        assert!(age.is_some_and(|c| c.align == ColumnAlign::Right && c.monospace));
    }

    #[test]
    fn test_filter_and_sort_storage_classes_by_all_columns() {
        let classes = vec![
            sample_sc(
                "standard",
                "kubernetes.io/gce-pd",
                "Delete",
                "Immediate",
                false,
                "10d",
            ),
            sample_sc(
                "fast-ssd",
                "ebs.csi.aws.com",
                "Retain",
                "WaitForFirstConsumer",
                true,
                "2h",
            ),
            sample_sc(
                "archive",
                "nfs.csi.k8s.io",
                "Retain",
                "Immediate",
                true,
                "30m",
            ),
        ];

        // Default sort: name ascending
        let default_rows = filter_and_sort_storage_classes(&classes, "", None);
        let names: Vec<&str> = default_rows.iter().map(|c| c.name.as_str()).collect();
        assert_eq!(names, vec!["archive", "fast-ssd", "standard"]);

        // Search filter matches provisioner
        let ebs = filter_and_sort_storage_classes(&classes, "ebs.csi", None);
        assert_eq!(ebs.len(), 1);
        assert_eq!(ebs[0].name, "fast-ssd");

        // Sort by expansion desc (true before false)
        let by_exp = filter_and_sort_storage_classes(
            &classes,
            "",
            Some(&SortState {
                column_id: "expansion".to_string(),
                direction: SortDirection::Desc,
            }),
        );
        assert!(by_exp[0].allow_volume_expansion);
        assert!(!by_exp[2].allow_volume_expansion);

        // Sort by age asc (30m < 2h < 10d)
        let by_age = filter_and_sort_storage_classes(
            &classes,
            "",
            Some(&SortState {
                column_id: "age".to_string(),
                direction: SortDirection::Asc,
            }),
        );
        let ages: Vec<&str> = by_age.iter().map(|c| c.age.as_str()).collect();
        assert_eq!(ages, vec!["30m", "2h", "10d"]);

        // Sort by provisioner asc
        let by_prov = filter_and_sort_storage_classes(
            &classes,
            "",
            Some(&SortState {
                column_id: "provisioner".to_string(),
                direction: SortDirection::Asc,
            }),
        );
        assert_eq!(by_prov[0].provisioner, "ebs.csi.aws.com");
    }

    #[test]
    fn test_storage_classes_empty_message_and_presentation_mode() {
        let mut state = StorageClassesState::default();
        assert_eq!(
            storage_classes_empty_message(&state),
            "No storage classes found in this cluster."
        );
        assert_eq!(
            storage_classes_presentation_mode(&state),
            StorageClassesPresentationMode::Table
        );

        state.status = StorageClassesLoadStatus::Error;
        assert_eq!(
            storage_classes_presentation_mode(&state),
            StorageClassesPresentationMode::ErrorCard
        );

        state.classes.push(sample_sc(
            "gp3",
            "ebs.csi.aws.com",
            "Delete",
            "WaitForFirstConsumer",
            true,
            "1d",
        ));
        assert_eq!(
            storage_classes_presentation_mode(&state),
            StorageClassesPresentationMode::Table
        );
        assert_eq!(
            storage_classes_empty_message(&state),
            "No storage classes match the current filter."
        );
    }
}
