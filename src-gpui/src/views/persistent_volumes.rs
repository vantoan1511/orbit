use std::cmp::Ordering;
use std::rc::Rc;

use gpui::{
    AnyElement, App, ClickEvent, ElementId, FontWeight, MouseDownEvent, SharedString, Window, div,
    prelude::*, px,
};
use lucide_icons::Icon;
use orbit_core::kubernetes::models::{EventInfo, PersistentVolumeInfo};
use orbit_core::kubernetes::nodes::parse_memory_quantity;

use crate::components::data_table::{
    DataTableCallbacks, SortState, TableColumn, render_data_table, sort_rows_stable,
};
use crate::components::{
    ContextMenuEntry, ContextMenuItem, StatusBadgeVariant, TextTooltip,
    render_status_badge_from_str,
};
use crate::icons::render_icon;
use crate::state::{PersistentVolumesLoadStatus, PersistentVolumesState};
use crate::theme::{OrbitTheme, apply_text_step};
use crate::views::{
    parse_age_seconds, render_drawer_events_list, render_drawer_key_values, render_drawer_section,
    render_resource_error_banner, render_resource_error_state, render_resource_view_header,
    render_truncated_cell,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PersistentVolumesPresentationMode {
    Table,
    ErrorCard,
}

pub fn persistent_volume_columns() -> Vec<TableColumn> {
    vec![
        TableColumn::new("name", "Name").flex_width(1.8, px(160.0)),
        TableColumn::new("capacity", "Capacity")
            .fixed_width(px(105.0))
            .right_numeric(),
        TableColumn::new("access_mode", "Access Mode").flex_width(1.2, px(130.0)),
        TableColumn::new("reclaim_policy", "Reclaim").fixed_width(px(96.0)),
        TableColumn::new("storage_class", "Storage Class")
            .flex_width(1.2, px(120.0))
            .monospace(true),
        TableColumn::new("status", "Status").fixed_width(px(110.0)),
        TableColumn::new("age", "Age")
            .fixed_width(px(76.0))
            .right_numeric(),
    ]
}

pub fn parse_storage_capacity_gib(capacity: &str) -> Option<f64> {
    let trimmed = capacity.trim();
    if trimmed.is_empty() || trimmed == "-" {
        return None;
    }
    Some(parse_memory_quantity(trimmed))
}

fn compare_persistent_volumes_by_col(
    a: &PersistentVolumeInfo,
    b: &PersistentVolumeInfo,
    col_id: &str,
) -> Ordering {
    let tie_break = || a.name.to_ascii_lowercase().cmp(&b.name.to_ascii_lowercase());

    match col_id {
        "name" => tie_break(),
        "capacity" => match (
            parse_storage_capacity_gib(&a.capacity),
            parse_storage_capacity_gib(&b.capacity),
        ) {
            (Some(av), Some(bv)) => av.total_cmp(&bv).then_with(tie_break),
            (Some(_), None) => Ordering::Less,
            (None, Some(_)) => Ordering::Greater,
            (None, None) => tie_break(),
        },
        "access_mode" => a
            .access_mode
            .to_ascii_lowercase()
            .cmp(&b.access_mode.to_ascii_lowercase())
            .then_with(tie_break),
        "reclaim_policy" => a
            .reclaim_policy
            .to_ascii_lowercase()
            .cmp(&b.reclaim_policy.to_ascii_lowercase())
            .then_with(tie_break),
        "storage_class" => a
            .storage_class
            .to_ascii_lowercase()
            .cmp(&b.storage_class.to_ascii_lowercase())
            .then_with(tie_break),
        "status" => a
            .status
            .to_ascii_lowercase()
            .cmp(&b.status.to_ascii_lowercase())
            .then_with(tie_break),
        "age" => parse_age_seconds(&a.age)
            .cmp(&parse_age_seconds(&b.age))
            .then_with(tie_break),
        _ => tie_break(),
    }
}

pub fn sort_persistent_volumes<'a>(
    pvs: &'a [PersistentVolumeInfo],
    sort: Option<&SortState>,
) -> Vec<&'a PersistentVolumeInfo> {
    let mut rows: Vec<&'a PersistentVolumeInfo> = pvs.iter().collect();
    if sort.is_some() {
        sort_rows_stable(&mut rows, sort, |a, b, col_id| {
            compare_persistent_volumes_by_col(a, b, col_id)
        });
    } else {
        rows.sort_by_key(|a| a.name.to_ascii_lowercase());
    }
    rows
}

pub fn persistent_volumes_empty_message(_state: &PersistentVolumesState) -> &'static str {
    "No persistent volumes found in this cluster."
}

pub fn persistent_volumes_presentation_mode(
    state: &PersistentVolumesState,
) -> PersistentVolumesPresentationMode {
    if state.status == PersistentVolumesLoadStatus::Error && state.persistent_volumes.is_empty() {
        PersistentVolumesPresentationMode::ErrorCard
    } else {
        PersistentVolumesPresentationMode::Table
    }
}

fn render_persistent_volume_cell(
    row_idx: usize,
    pv: &PersistentVolumeInfo,
    col: &TableColumn,
    theme: &OrbitTheme,
) -> AnyElement {
    let is_failed = pv.status.eq_ignore_ascii_case("failed");

    match col.id.as_ref() {
        "name" => {
            let tip = SharedString::from(pv.name.clone());
            div()
                .id(ElementId::NamedInteger(
                    SharedString::from("pv-name"),
                    row_idx as u64,
                ))
                .flex()
                .items_center()
                .gap_1p5()
                .min_w(px(0.0))
                .flex_1()
                .overflow_hidden()
                .tooltip(move |_window, cx| cx.new(|_| TextTooltip::new(tip.clone())).into())
                .when(is_failed, |el| {
                    el.child(render_icon(
                        Icon::AlertCircle,
                        px(13.0),
                        theme.semantic.danger.solid,
                    ))
                })
                .child(
                    apply_text_step(div(), theme.typography.size_xs)
                        .min_w(px(0.0))
                        .flex_1()
                        .font_weight(FontWeight::MEDIUM)
                        .text_color(theme.text.primary)
                        .truncate()
                        .child(pv.name.clone()),
                )
                .into_any_element()
        }
        "capacity" => apply_text_step(div(), theme.typography.size_xs)
            .font(theme.typography.mono_font(FontWeight::NORMAL))
            .text_color(if pv.capacity == "-" {
                theme.text.muted
            } else {
                theme.text.primary
            })
            .truncate()
            .child(pv.capacity.clone())
            .into_any_element(),
        "access_mode" => render_truncated_cell(
            ElementId::NamedInteger(SharedString::from("pv-mode"), row_idx as u64),
            pv.access_mode.clone(),
            None,
            if pv.access_mode == "-" {
                theme.text.muted
            } else {
                theme.text.secondary
            },
            FontWeight::NORMAL,
            false,
            theme,
        ),
        "reclaim_policy" => render_truncated_cell(
            ElementId::NamedInteger(SharedString::from("pv-reclaim"), row_idx as u64),
            pv.reclaim_policy.clone(),
            None,
            if pv.reclaim_policy == "-" {
                theme.text.muted
            } else {
                theme.text.secondary
            },
            FontWeight::NORMAL,
            false,
            theme,
        ),
        "storage_class" => render_truncated_cell(
            ElementId::NamedInteger(SharedString::from("pv-sc"), row_idx as u64),
            pv.storage_class.clone(),
            None,
            if pv.storage_class == "-" {
                theme.text.muted
            } else {
                theme.text.secondary
            },
            FontWeight::NORMAL,
            true,
            theme,
        ),
        "status" => render_status_badge_from_str(&pv.status, StatusBadgeVariant::Inline, theme),
        "age" => apply_text_step(div(), theme.typography.size_xs)
            .font(theme.typography.mono_font(FontWeight::NORMAL))
            .text_color(theme.text.muted)
            .child(pv.age.clone())
            .into_any_element(),
        _ => div().into_any_element(),
    }
}

pub fn build_persistent_volume_context_menu_entries(
    _pv: &PersistentVolumeInfo,
    on_copy_name: impl 'static + Fn(&mut Window, &mut App),
    on_refresh: impl 'static + Fn(&mut Window, &mut App),
    on_view_details: impl 'static + Fn(&mut Window, &mut App),
    on_view_yaml: impl 'static + Fn(&mut Window, &mut App),
    on_edit: impl 'static + Fn(&mut Window, &mut App),
    on_delete: impl 'static + Fn(&mut Window, &mut App),
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
        ContextMenuEntry::Separator,
        ContextMenuEntry::Item(
            ContextMenuItem::new("delete-persistent-volume", "Delete PersistentVolume")
                .icon(Icon::Trash2)
                .danger(true)
                .on_select(on_delete),
        ),
    ]
}

#[allow(clippy::too_many_arguments)]
pub fn render_persistent_volumes_view(
    state: &PersistentVolumesState,
    theme: &OrbitTheme,
    on_sort: impl 'static + Fn(SharedString, &ClickEvent, &mut Window, &mut App),
    on_row_click: impl 'static + Fn(usize, SharedString, &ClickEvent, &mut Window, &mut App),
    on_row_quick_yaml: impl 'static + Fn(usize, SharedString, &ClickEvent, &mut Window, &mut App),
    on_row_context_menu: impl 'static
        + Fn(SharedString, &PersistentVolumeInfo, &MouseDownEvent, &mut Window, &mut App),
    on_refresh: impl 'static + Fn(&mut Window, &mut App),
) -> AnyElement {
    let columns = persistent_volume_columns();
    let cache_key =
        crate::state::RowCacheKey::from_table(state.data_revision, 0, &state.table, 0);
    let sorted_rows = state.row_cache.get_or_compute(cache_key, || {
        sort_persistent_volumes(&state.persistent_volumes, state.table.sort.as_ref())
            .into_iter()
            .cloned()
            .collect()
    });
    let total_count = sorted_rows.len();
    let is_loading = state.status == PersistentVolumesLoadStatus::Loading;
    let presentation_mode = persistent_volumes_presentation_mode(state);
    let on_refresh_rc: super::ActionCallback = Rc::new(on_refresh);

    let header = render_resource_view_header(format!("{total_count} volumes"), None, None, theme);

    let error_banner = if presentation_mode == PersistentVolumesPresentationMode::Table {
        state.error_message.as_ref().map(|message| {
            render_resource_error_banner(
                "persistent-volumes-retry-btn",
                format!("Failed to load persistent volumes: {message}"),
                is_loading,
                theme,
                on_refresh_rc.clone(),
            )
        })
    } else {
        None
    };

    let body = match presentation_mode {
        PersistentVolumesPresentationMode::ErrorCard => {
            let message = state
                .error_message
                .clone()
                .unwrap_or_else(|| "Unknown error".to_string());
            render_resource_error_state(
                "persistent-volumes-retry-btn",
                "Unable to Load PersistentVolumes",
                message,
                is_loading,
                theme,
                on_refresh_rc,
            )
        }
        PersistentVolumesPresentationMode::Table => {
            let mut table_state = state.table.clone();
            table_state.loading = is_loading && state.persistent_volumes.is_empty();
            table_state.refreshing = is_loading && !state.persistent_volumes.is_empty();
            table_state.empty_message = SharedString::from(persistent_volumes_empty_message(state));

            let key_rows = sorted_rows.clone();
            let cell_rows = sorted_rows.clone();
            let ctx_rows = sorted_rows.clone();
            let callbacks = DataTableCallbacks::new(
                move |idx| {
                    key_rows
                        .get(idx)
                        .map(|pv| pv.name.clone())
                        .unwrap_or_default()
                },
                move |idx, col, col_theme| {
                    cell_rows
                        .get(idx)
                        .map(|pv| render_persistent_volume_cell(idx, pv, col, col_theme))
                        .unwrap_or_else(|| div().into_any_element())
                },
            )
            .on_sort(on_sort)
            .on_row_click(on_row_click)
            .on_row_quick_yaml(on_row_quick_yaml)
            .on_row_context_menu(move |idx, row_key, event, window, cx| {
                if let Some(pv) = ctx_rows.get(idx) {
                    on_row_context_menu(row_key, pv, event, window, cx);
                }
            });

            render_data_table(
                "persistent-volumes-table",
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
        .when_some(error_banner, |el, banner| el.child(banner))
        .child(body)
        .into_any_element()
}

pub fn filter_events_for_persistent_volume<'a>(
    events: &'a [EventInfo],
    pv: &PersistentVolumeInfo,
) -> Vec<&'a EventInfo> {
    if pv.name.is_empty() {
        return Vec::new();
    }
    events
        .iter()
        .filter(|ev| {
            (ev.object_kind == "PersistentVolume" && ev.object_name == pv.name)
                || ev.message.contains(&pv.name)
        })
        .collect()
}

pub fn render_persistent_volume_drawer_overview(
    pv: &PersistentVolumeInfo,
    theme: &OrbitTheme,
) -> AnyElement {
    let info_rows = vec![
        ("Name", pv.name.clone(), true),
        ("Status", pv.status.clone(), false),
        ("Capacity", pv.capacity.clone(), true),
        ("Access Mode", pv.access_mode.clone(), false),
        ("Reclaim Policy", pv.reclaim_policy.clone(), false),
        ("Storage Class", pv.storage_class.clone(), true),
        ("Volume Mode", pv.volume_mode.clone(), false),
        ("Age", pv.age.clone(), true),
    ];

    let failure_panel = pv
        .reason
        .as_deref()
        .map(str::trim)
        .filter(|reason| !reason.is_empty())
        .map(|reason| {
            render_drawer_section(
                "FAILURE REASON",
                div()
                    .w_full()
                    .px_3()
                    .py_2()
                    .rounded(theme.style.radius_control)
                    .border_1()
                    .border_color(theme.semantic.danger.solid)
                    .bg(theme.semantic.danger.soft)
                    .flex()
                    .items_start()
                    .gap_2()
                    .child(render_icon(
                        Icon::AlertCircle,
                        px(14.0),
                        theme.semantic.danger.solid,
                    ))
                    .child(
                        apply_text_step(div(), theme.typography.size_xs)
                            .text_color(theme.text.primary)
                            .child(reason.to_string()),
                    )
                    .into_any_element(),
                theme,
            )
        });

    div()
        .w_full()
        .flex()
        .flex_col()
        .gap_4()
        .child(render_drawer_section(
            "VOLUME INFORMATION",
            render_drawer_key_values(info_rows, theme),
            theme,
        ))
        .when_some(failure_panel, |el, panel| el.child(panel))
        .into_any_element()
}

pub fn render_persistent_volume_drawer_events(
    events: &[&EventInfo],
    loading: bool,
    theme: &OrbitTheme,
) -> AnyElement {
    render_drawer_events_list(
        events,
        loading,
        "No recent events for this persistent volume.",
        theme,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::components::data_table::{ColumnAlign, SortDirection};

    fn sample_pv(
        name: &str,
        capacity: &str,
        access_mode: &str,
        reclaim_policy: &str,
        storage_class: &str,
        status: &str,
        age: &str,
    ) -> PersistentVolumeInfo {
        PersistentVolumeInfo {
            name: name.to_string(),
            capacity: capacity.to_string(),
            access_mode: access_mode.to_string(),
            reclaim_policy: reclaim_policy.to_string(),
            status: status.to_string(),
            storage_class: storage_class.to_string(),
            age: age.to_string(),
            volume_mode: "Filesystem".to_string(),
            reason: None,
        }
    }

    #[test]
    fn test_persistent_volume_columns_cover_required_fields_and_alignment() {
        let cols = persistent_volume_columns();
        let ids: Vec<&str> = cols.iter().map(|c| c.id.as_ref()).collect();
        assert_eq!(
            ids,
            vec![
                "name",
                "capacity",
                "access_mode",
                "reclaim_policy",
                "storage_class",
                "status",
                "age",
            ]
        );

        let capacity = cols.iter().find(|c| c.id.as_ref() == "capacity").unwrap();
        assert_eq!(capacity.align, ColumnAlign::Right);
        assert!(capacity.monospace);

        let storage_class = cols
            .iter()
            .find(|c| c.id.as_ref() == "storage_class")
            .unwrap();
        assert!(storage_class.monospace);

        let age = cols.iter().find(|c| c.id.as_ref() == "age").unwrap();
        assert_eq!(age.align, ColumnAlign::Right);
        assert!(age.monospace);
    }

    #[test]
    fn test_parse_storage_capacity_gib_orders_k8s_quantities_numerically() {
        assert_eq!(parse_storage_capacity_gib(""), None);
        assert_eq!(parse_storage_capacity_gib("-"), None);

        let mi512 = parse_storage_capacity_gib("512Mi").unwrap();
        let gi5 = parse_storage_capacity_gib("5Gi").unwrap();
        let gi10 = parse_storage_capacity_gib("10Gi").unwrap();
        let gi100 = parse_storage_capacity_gib("100Gi").unwrap();
        let ti1 = parse_storage_capacity_gib("1Ti").unwrap();

        assert!(mi512 < gi5);
        assert!(gi5 < gi10);
        assert!(gi10 < gi100);
        assert!(gi100 < ti1);
    }

    #[test]
    fn test_sort_persistent_volumes_supports_all_columns_and_default_name_order() {
        let pvs = vec![
            sample_pv(
                "pv-zeta",
                "5Gi",
                "ReadWriteMany",
                "Retain",
                "standard",
                "Released",
                "10d",
            ),
            sample_pv(
                "pv-alpha",
                "100Gi",
                "ReadWriteOnce",
                "Delete",
                "gp3",
                "Bound",
                "2h",
            ),
            sample_pv(
                "pv-mid",
                "10Gi",
                "ReadOnlyMany",
                "Recycle",
                "ebs-sc",
                "Available",
                "3d",
            ),
        ];

        // Default order (None) sorts by name ascending
        let default_sorted: Vec<&str> = sort_persistent_volumes(&pvs, None)
            .into_iter()
            .map(|p| p.name.as_str())
            .collect();
        assert_eq!(default_sorted, vec!["pv-alpha", "pv-mid", "pv-zeta"]);

        // Capacity sorts numerically: 5Gi < 10Gi < 100Gi (not lexicographic "100Gi" < "5Gi")
        let by_cap_asc: Vec<&str> = sort_persistent_volumes(
            &pvs,
            Some(&SortState {
                column_id: "capacity".to_string(),
                direction: SortDirection::Asc,
            }),
        )
        .into_iter()
        .map(|p| p.name.as_str())
        .collect();
        assert_eq!(by_cap_asc, vec!["pv-zeta", "pv-mid", "pv-alpha"]);

        let by_cap_desc: Vec<&str> = sort_persistent_volumes(
            &pvs,
            Some(&SortState {
                column_id: "capacity".to_string(),
                direction: SortDirection::Desc,
            }),
        )
        .into_iter()
        .map(|p| p.name.as_str())
        .collect();
        assert_eq!(by_cap_desc, vec!["pv-alpha", "pv-mid", "pv-zeta"]);

        // Access mode
        let by_access: Vec<&str> = sort_persistent_volumes(
            &pvs,
            Some(&SortState {
                column_id: "access_mode".to_string(),
                direction: SortDirection::Asc,
            }),
        )
        .into_iter()
        .map(|p| p.name.as_str())
        .collect();
        assert_eq!(by_access, vec!["pv-mid", "pv-zeta", "pv-alpha"]);

        // Reclaim policy
        let by_reclaim: Vec<&str> = sort_persistent_volumes(
            &pvs,
            Some(&SortState {
                column_id: "reclaim_policy".to_string(),
                direction: SortDirection::Asc,
            }),
        )
        .into_iter()
        .map(|p| p.name.as_str())
        .collect();
        assert_eq!(by_reclaim, vec!["pv-alpha", "pv-mid", "pv-zeta"]);

        // Storage class
        let by_sc: Vec<&str> = sort_persistent_volumes(
            &pvs,
            Some(&SortState {
                column_id: "storage_class".to_string(),
                direction: SortDirection::Asc,
            }),
        )
        .into_iter()
        .map(|p| p.name.as_str())
        .collect();
        assert_eq!(by_sc, vec!["pv-mid", "pv-alpha", "pv-zeta"]);

        // Status
        let by_status: Vec<&str> = sort_persistent_volumes(
            &pvs,
            Some(&SortState {
                column_id: "status".to_string(),
                direction: SortDirection::Asc,
            }),
        )
        .into_iter()
        .map(|p| p.name.as_str())
        .collect();
        assert_eq!(by_status, vec!["pv-mid", "pv-alpha", "pv-zeta"]);

        // Age sorts numerically: 2h < 3d < 10d
        let by_age: Vec<&str> = sort_persistent_volumes(
            &pvs,
            Some(&SortState {
                column_id: "age".to_string(),
                direction: SortDirection::Asc,
            }),
        )
        .into_iter()
        .map(|p| p.name.as_str())
        .collect();
        assert_eq!(by_age, vec!["pv-alpha", "pv-mid", "pv-zeta"]);
    }

    #[test]
    fn test_persistent_volumes_empty_message_and_presentation_mode() {
        let mut state = PersistentVolumesState::default();
        assert_eq!(
            persistent_volumes_empty_message(&state),
            "No persistent volumes found in this cluster."
        );
        assert_eq!(
            persistent_volumes_presentation_mode(&state),
            PersistentVolumesPresentationMode::Table
        );

        state.status = PersistentVolumesLoadStatus::Error;
        state.error_message = Some("forbidden".to_string());
        assert_eq!(
            persistent_volumes_presentation_mode(&state),
            PersistentVolumesPresentationMode::ErrorCard
        );

        state.persistent_volumes.push(sample_pv(
            "pv-cached",
            "10Gi",
            "ReadWriteOnce",
            "Retain",
            "gp3",
            "Bound",
            "1d",
        ));
        assert_eq!(
            persistent_volumes_presentation_mode(&state),
            PersistentVolumesPresentationMode::Table
        );
    }

    #[test]
    fn test_filter_events_for_persistent_volume_matches_kind_or_message() {
        let pv = sample_pv(
            "pv-data-01",
            "10Gi",
            "ReadWriteOnce",
            "Retain",
            "gp3",
            "Bound",
            "1d",
        );

        let events = vec![
            EventInfo {
                uid: "ev-1".to_string(),
                name: String::new(),
                r#type: "Normal".to_string(),
                reason: "ProvisioningSucceeded".to_string(),
                message: "Successfully provisioned volume".to_string(),
                object_kind: "PersistentVolume".to_string(),
                object_name: "pv-data-01".to_string(),
                namespace: String::new(),
                count: 1,
                time: "2m".to_string(),
                first_seen: "2m ago".to_string(),
                last_seen: "2m ago".to_string(),
                source: "ebs-csi".to_string(),
                labels: Default::default(),
                annotations: Default::default(),
            },
            EventInfo {
                uid: "ev-2".to_string(),
                name: String::new(),
                r#type: "Warning".to_string(),
                reason: "FailedMount".to_string(),
                message: "Unable to attach pv-data-01 to node-1".to_string(),
                object_kind: "Pod".to_string(),
                object_name: "api-0".to_string(),
                namespace: "prod".to_string(),
                count: 3,
                time: "1m".to_string(),
                first_seen: "1m ago".to_string(),
                last_seen: "1m ago".to_string(),
                source: "kubelet".to_string(),
                labels: Default::default(),
                annotations: Default::default(),
            },
            EventInfo {
                uid: "ev-3".to_string(),
                name: String::new(),
                r#type: "Normal".to_string(),
                reason: "Scheduled".to_string(),
                message: "Successfully assigned pod".to_string(),
                object_kind: "Pod".to_string(),
                object_name: "other-0".to_string(),
                namespace: "default".to_string(),
                count: 1,
                time: "5m".to_string(),
                first_seen: "5m ago".to_string(),
                last_seen: "5m ago".to_string(),
                source: "scheduler".to_string(),
                labels: Default::default(),
                annotations: Default::default(),
            },
        ];

        let matched = filter_events_for_persistent_volume(&events, &pv);
        let uids: Vec<&str> = matched.iter().map(|e| e.uid.as_str()).collect();
        assert_eq!(uids, vec!["ev-1", "ev-2"]);
    }

    #[test]
    fn test_build_persistent_volume_context_menu_entries_matches_cluster_scoped_actions() {
        let pv = sample_pv(
            "pv-data-01",
            "10Gi",
            "ReadWriteOnce",
            "Retain",
            "gp3",
            "Bound",
            "1d",
        );

        let entries = build_persistent_volume_context_menu_entries(
            &pv,
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
                "refresh",
                "view-details",
                "view-yaml",
                "edit",
                "delete-persistent-volume",
            ]
        );
        assert!(!item_ids.contains(&"filter-namespace"));

        let delete_item = entries
            .iter()
            .find_map(|e| match e {
                ContextMenuEntry::Item(item)
                    if item.id.as_ref() == "delete-persistent-volume" =>
                {
                    Some(item)
                }
                _ => None,
            })
            .unwrap();
        assert!(delete_item.danger);
        assert!(!delete_item.disabled);
        assert!(delete_item.on_select.is_some());
    }
}
