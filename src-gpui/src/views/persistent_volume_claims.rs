use std::cmp::Ordering;
use std::rc::Rc;

use gpui::{
    AnyElement, App, ClickEvent, ElementId, FontWeight, MouseDownEvent, SharedString, Window, div,
    prelude::*, px,
};
use lucide_icons::Icon;
use orbit_core::kubernetes::models::{EventInfo, PersistentVolumeClaimInfo};

use crate::components::data_table::{
    DataTableCallbacks, SortState, TableColumn, render_data_table, sort_rows_stable,
};
use crate::components::{
    ContextMenuEntry, ContextMenuItem, StatusBadgeVariant, TextTooltip,
    render_status_badge_from_str,
};
use crate::icons::render_icon;
use crate::state::{
    NamespaceFilterState, PersistentVolumeClaimsLoadStatus, PersistentVolumeClaimsState,
};
use crate::theme::{OrbitTheme, apply_text_step};
use crate::views::persistent_volumes::parse_storage_capacity_gib;
use crate::views::{
    parse_age_seconds, render_drawer_events_list, render_drawer_key_values, render_drawer_section,
    render_resource_error_banner, render_resource_error_state, render_resource_view_header,
    render_truncated_cell,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PersistentVolumeClaimsPresentationMode {
    Table,
    ErrorCard,
}

pub fn persistent_volume_claim_columns() -> Vec<TableColumn> {
    vec![
        TableColumn::new("name", "Name").flex_width(1.6, px(150.0)),
        TableColumn::new("namespace", "Namespace").flex_width(1.0, px(105.0)),
        TableColumn::new("volume", "Volume")
            .flex_width(1.4, px(130.0))
            .monospace(true),
        TableColumn::new("capacity", "Capacity")
            .fixed_width(px(105.0))
            .right_numeric(),
        TableColumn::new("access_mode", "Access Mode").flex_width(1.2, px(125.0)),
        TableColumn::new("storage_class", "Storage Class")
            .flex_width(1.1, px(115.0))
            .monospace(true),
        TableColumn::new("status", "Status").fixed_width(px(104.0)),
        TableColumn::new("age", "Age")
            .fixed_width(px(76.0))
            .right_numeric(),
    ]
}

pub fn compare_persistent_volume_claims_by_col(
    a: &PersistentVolumeClaimInfo,
    b: &PersistentVolumeClaimInfo,
    col_id: &str,
) -> Ordering {
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
        "volume" => a
            .volume
            .to_ascii_lowercase()
            .cmp(&b.volume.to_ascii_lowercase())
            .then_with(tie_break),
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

pub fn filter_and_sort_persistent_volume_claims<'a>(
    pvcs: &'a [PersistentVolumeClaimInfo],
    namespaces: &NamespaceFilterState,
    sort: Option<&SortState>,
) -> Vec<&'a PersistentVolumeClaimInfo> {
    let mut rows: Vec<&'a PersistentVolumeClaimInfo> = pvcs
        .iter()
        .filter(|pvc| namespaces.matches_resource_namespace(Some(&pvc.namespace)))
        .collect();
    if sort.is_some() {
        sort_rows_stable(&mut rows, sort, |a, b, col_id| {
            compare_persistent_volume_claims_by_col(a, b, col_id)
        });
    } else {
        rows.sort_by(|a, b| compare_persistent_volume_claims_by_col(a, b, "name"));
    }
    rows
}

pub fn persistent_volume_claims_empty_message(state: &PersistentVolumeClaimsState) -> &'static str {
    if state.persistent_volume_claims.is_empty() {
        "No persistent volume claims found in this cluster."
    } else {
        "No persistent volume claims found in the selected namespace(s)."
    }
}

pub fn persistent_volume_claims_presentation_mode(
    state: &PersistentVolumeClaimsState,
) -> PersistentVolumeClaimsPresentationMode {
    if state.status == PersistentVolumeClaimsLoadStatus::Error
        && state.persistent_volume_claims.is_empty()
    {
        PersistentVolumeClaimsPresentationMode::ErrorCard
    } else {
        PersistentVolumeClaimsPresentationMode::Table
    }
}

fn render_persistent_volume_claim_cell(
    row_idx: usize,
    pvc: &PersistentVolumeClaimInfo,
    col: &TableColumn,
    theme: &OrbitTheme,
) -> AnyElement {
    let is_lost = pvc.status.eq_ignore_ascii_case("lost");

    match col.id.as_ref() {
        "name" => {
            let tip = SharedString::from(pvc.name.clone());
            div()
                .id(ElementId::NamedInteger(
                    SharedString::from("pvc-name"),
                    row_idx as u64,
                ))
                .flex()
                .items_center()
                .gap_1p5()
                .min_w(px(0.0))
                .flex_1()
                .overflow_hidden()
                .tooltip(move |_window, cx| cx.new(|_| TextTooltip::new(tip.clone())).into())
                .when(is_lost, |el| {
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
                        .child(pvc.name.clone()),
                )
                .into_any_element()
        }
        "namespace" => render_truncated_cell(
            ElementId::NamedInteger(SharedString::from("pvc-ns"), row_idx as u64),
            pvc.namespace.clone(),
            None,
            theme.text.secondary,
            FontWeight::NORMAL,
            false,
            theme,
        ),
        "volume" => render_truncated_cell(
            ElementId::NamedInteger(SharedString::from("pvc-vol"), row_idx as u64),
            pvc.volume.clone(),
            (pvc.volume != "-").then(|| SharedString::from(pvc.volume.clone())),
            if pvc.volume == "-" {
                theme.text.muted
            } else {
                theme.text.secondary
            },
            FontWeight::NORMAL,
            true,
            theme,
        ),
        "capacity" => apply_text_step(div(), theme.typography.size_xs)
            .font(theme.typography.mono_font(FontWeight::NORMAL))
            .text_color(if pvc.capacity == "-" {
                theme.text.muted
            } else {
                theme.text.primary
            })
            .truncate()
            .child(pvc.capacity.clone())
            .into_any_element(),
        "access_mode" => render_truncated_cell(
            ElementId::NamedInteger(SharedString::from("pvc-mode"), row_idx as u64),
            pvc.access_mode.clone(),
            None,
            if pvc.access_mode == "-" {
                theme.text.muted
            } else {
                theme.text.secondary
            },
            FontWeight::NORMAL,
            false,
            theme,
        ),
        "storage_class" => render_truncated_cell(
            ElementId::NamedInteger(SharedString::from("pvc-sc"), row_idx as u64),
            pvc.storage_class.clone(),
            None,
            if pvc.storage_class == "-" {
                theme.text.muted
            } else {
                theme.text.secondary
            },
            FontWeight::NORMAL,
            true,
            theme,
        ),
        "status" => render_status_badge_from_str(&pvc.status, StatusBadgeVariant::Inline, theme),
        "age" => apply_text_step(div(), theme.typography.size_xs)
            .font(theme.typography.mono_font(FontWeight::NORMAL))
            .text_color(theme.text.muted)
            .child(pvc.age.clone())
            .into_any_element(),
        _ => div().into_any_element(),
    }
}

pub fn build_persistent_volume_claim_context_menu_entries(
    pvc: &PersistentVolumeClaimInfo,
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
                format!("Filter by \"{}\"", pvc.namespace),
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
            ContextMenuItem::new(
                "delete-persistent-volume-claim",
                "Delete PersistentVolumeClaim",
            )
            .icon(Icon::Trash2)
            .danger(true)
            .on_select(on_delete),
        ),
    ]
}

#[allow(clippy::too_many_arguments)]
pub fn render_persistent_volume_claims_view(
    state: &PersistentVolumeClaimsState,
    namespaces: &NamespaceFilterState,
    theme: &OrbitTheme,
    on_sort: impl 'static + Fn(SharedString, &ClickEvent, &mut Window, &mut App),
    on_row_click: impl 'static + Fn(usize, SharedString, &ClickEvent, &mut Window, &mut App),
    on_row_quick_yaml: impl 'static + Fn(usize, SharedString, &ClickEvent, &mut Window, &mut App),
    on_row_context_menu: impl 'static
        + Fn(SharedString, &PersistentVolumeClaimInfo, &MouseDownEvent, &mut Window, &mut App),
    on_refresh: impl 'static + Fn(&mut Window, &mut App),
) -> AnyElement {
    let columns = persistent_volume_claim_columns();
    let cache_key = crate::state::RowCacheKey::from_table(
        state.data_revision,
        namespaces.selection_revision,
        &state.table,
        0,
    );
    let filtered_rows = state.row_cache.get_or_compute(cache_key, || {
        filter_and_sort_persistent_volume_claims(
            &state.persistent_volume_claims,
            namespaces,
            state.table.sort.as_ref(),
        )
        .into_iter()
        .cloned()
        .collect()
    });
    let total_count = state.persistent_volume_claims.len();
    let visible_count = filtered_rows.len();
    let is_loading = state.status == PersistentVolumeClaimsLoadStatus::Loading;
    let presentation_mode = persistent_volume_claims_presentation_mode(state);
    let on_refresh_rc: super::ActionCallback = Rc::new(on_refresh);

    let count_label = if namespaces.is_all_namespaces() {
        format!("{total_count} claims")
    } else {
        format!("{visible_count} of {total_count} claims")
    };

    let header = render_resource_view_header(count_label, Some(namespaces), None, theme);

    let error_banner = if presentation_mode == PersistentVolumeClaimsPresentationMode::Table {
        state.error_message.as_ref().map(|message| {
            render_resource_error_banner(
                "persistent-volume-claims-retry-btn",
                format!("Failed to load persistent volume claims: {message}"),
                is_loading,
                theme,
                on_refresh_rc.clone(),
            )
        })
    } else {
        None
    };

    let body = match presentation_mode {
        PersistentVolumeClaimsPresentationMode::ErrorCard => {
            let message = state
                .error_message
                .clone()
                .unwrap_or_else(|| "Unknown error".to_string());
            render_resource_error_state(
                "persistent-volume-claims-retry-btn",
                "Unable to Load Volume Claims",
                message,
                is_loading,
                theme,
                on_refresh_rc,
            )
        }
        PersistentVolumeClaimsPresentationMode::Table => {
            let mut table_state = state.table.clone();
            table_state.loading = is_loading && state.persistent_volume_claims.is_empty();
            table_state.refreshing = is_loading && !state.persistent_volume_claims.is_empty();
            table_state.empty_message =
                SharedString::from(persistent_volume_claims_empty_message(state));

            let key_rows = filtered_rows.clone();
            let cell_rows = filtered_rows.clone();
            let ctx_rows = filtered_rows.clone();
            let callbacks = DataTableCallbacks::new(
                move |idx| {
                    key_rows
                        .get(idx)
                        .map(|pvc| format!("{}/{}", pvc.namespace, pvc.name))
                        .unwrap_or_default()
                },
                move |idx, col, col_theme| {
                    cell_rows
                        .get(idx)
                        .map(|pvc| render_persistent_volume_claim_cell(idx, pvc, col, col_theme))
                        .unwrap_or_else(|| div().into_any_element())
                },
            )
            .on_sort(on_sort)
            .on_row_click(on_row_click)
            .on_row_quick_yaml(on_row_quick_yaml)
            .on_row_context_menu(move |idx, row_key, event, window, cx| {
                if let Some(pvc) = ctx_rows.get(idx) {
                    on_row_context_menu(row_key, pvc, event, window, cx);
                }
            });

            render_data_table(
                "persistent-volume-claims-table",
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

pub fn filter_events_for_persistent_volume_claim<'a>(
    events: &'a [EventInfo],
    pvc: &PersistentVolumeClaimInfo,
) -> Vec<&'a EventInfo> {
    if pvc.name.is_empty() {
        return Vec::new();
    }
    events
        .iter()
        .filter(|ev| {
            ev.namespace == pvc.namespace
                && ((ev.object_kind == "PersistentVolumeClaim" && ev.object_name == pvc.name)
                    || ev.message.contains(&pvc.name))
        })
        .collect()
}

pub fn render_persistent_volume_claim_drawer_overview(
    pvc: &PersistentVolumeClaimInfo,
    theme: &OrbitTheme,
) -> AnyElement {
    let info_rows = vec![
        ("Name", pvc.name.clone(), true),
        ("Namespace", pvc.namespace.clone(), false),
        ("Status", pvc.status.clone(), false),
        ("Bound Volume", pvc.volume.clone(), true),
        ("Capacity", pvc.capacity.clone(), true),
        ("Access Mode", pvc.access_mode.clone(), false),
        ("Storage Class", pvc.storage_class.clone(), true),
        ("Age", pvc.age.clone(), true),
    ];

    div()
        .w_full()
        .flex()
        .flex_col()
        .gap_4()
        .child(render_drawer_section(
            "CLAIM INFORMATION",
            render_drawer_key_values(info_rows, theme),
            theme,
        ))
        .into_any_element()
}

pub fn render_persistent_volume_claim_drawer_events(
    events: &[&EventInfo],
    loading: bool,
    theme: &OrbitTheme,
) -> AnyElement {
    render_drawer_events_list(
        events,
        loading,
        "No recent events for this persistent volume claim.",
        theme,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::components::data_table::{ColumnAlign, SortDirection};

    #[allow(clippy::too_many_arguments)]
    fn sample_pvc(
        name: &str,
        namespace: &str,
        volume: &str,
        capacity: &str,
        access_mode: &str,
        storage_class: &str,
        status: &str,
        age: &str,
    ) -> PersistentVolumeClaimInfo {
        PersistentVolumeClaimInfo {
            name: name.to_string(),
            namespace: namespace.to_string(),
            status: status.to_string(),
            volume: volume.to_string(),
            capacity: capacity.to_string(),
            access_mode: access_mode.to_string(),
            storage_class: storage_class.to_string(),
            age: age.to_string(),
        }
    }

    #[test]
    fn test_persistent_volume_claim_columns_cover_required_fields_and_alignment() {
        let cols = persistent_volume_claim_columns();
        let ids: Vec<&str> = cols.iter().map(|c| c.id.as_ref()).collect();
        assert_eq!(
            ids,
            vec![
                "name",
                "namespace",
                "volume",
                "capacity",
                "access_mode",
                "storage_class",
                "status",
                "age",
            ]
        );

        let volume = cols.iter().find(|c| c.id.as_ref() == "volume").unwrap();
        assert!(volume.monospace);

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
    fn test_filter_and_sort_persistent_volume_claims_respects_namespace_and_all_columns() {
        let pvcs = vec![
            sample_pvc(
                "zeta-data",
                "prod",
                "pv-03",
                "20Gi",
                "ReadWriteMany",
                "gp3",
                "Bound",
                "4d",
            ),
            sample_pvc(
                "alpha-cache",
                "staging",
                "pv-01",
                "512Mi",
                "ReadWriteOnce",
                "gp2",
                "Lost",
                "15m",
            ),
            sample_pvc(
                "beta-db",
                "prod",
                "-",
                "5Gi",
                "ReadOnlyMany",
                "standard",
                "Pending",
                "2h",
            ),
            sample_pvc(
                "gamma-archive",
                "dev",
                "pv-04",
                "1Ti",
                "ReadWriteOnce",
                "ebs-sc",
                "Bound",
                "10d",
            ),
            sample_pvc("delta-unset", "prod", "-", "-", "-", "-", "Pending", "1h"),
        ];

        // All namespaces + default sort (name asc)
        let mut ns_filter = NamespaceFilterState::default();
        let _ = ns_filter.begin_load_for_cluster("ctx-a");
        let all_default = filter_and_sort_persistent_volume_claims(&pvcs, &ns_filter, None);
        let all_names: Vec<&str> = all_default.iter().map(|p| p.name.as_str()).collect();
        assert_eq!(
            all_names,
            vec![
                "alpha-cache",
                "beta-db",
                "delta-unset",
                "gamma-archive",
                "zeta-data",
            ]
        );

        // Single namespace filter
        ns_filter.toggle_namespace("prod");
        let prod_only = filter_and_sort_persistent_volume_claims(&pvcs, &ns_filter, None);
        let prod_names: Vec<&str> = prod_only.iter().map(|p| p.name.as_str()).collect();
        assert_eq!(prod_names, vec!["beta-db", "delta-unset", "zeta-data"]);

        // Multi-namespace filter
        ns_filter.toggle_namespace("staging");
        let multi_ns = filter_and_sort_persistent_volume_claims(&pvcs, &ns_filter, None);
        assert_eq!(multi_ns.len(), 4);

        // Reset to all namespaces for column sort checks
        ns_filter.select_all_namespaces();

        // Numeric capacity sort ascending ("-" sorts after numeric values)
        let by_cap_asc = filter_and_sort_persistent_volume_claims(
            &pvcs,
            &ns_filter,
            Some(&SortState {
                column_id: "capacity".into(),
                direction: SortDirection::Asc,
            }),
        );
        let cap_names: Vec<&str> = by_cap_asc.iter().map(|p| p.name.as_str()).collect();
        assert_eq!(
            cap_names,
            vec![
                "alpha-cache",   // 512Mi
                "beta-db",       // 5Gi
                "zeta-data",     // 20Gi
                "gamma-archive", // 1Ti
                "delta-unset",   // -
            ]
        );

        // Numeric capacity sort descending
        let by_cap_desc = filter_and_sort_persistent_volume_claims(
            &pvcs,
            &ns_filter,
            Some(&SortState {
                column_id: "capacity".into(),
                direction: SortDirection::Desc,
            }),
        );
        let cap_desc_names: Vec<&str> = by_cap_desc.iter().map(|p| p.name.as_str()).collect();
        assert_eq!(
            cap_desc_names,
            vec![
                "delta-unset",
                "gamma-archive",
                "zeta-data",
                "beta-db",
                "alpha-cache",
            ]
        );

        // Age sort ascending
        let by_age = filter_and_sort_persistent_volume_claims(
            &pvcs,
            &ns_filter,
            Some(&SortState {
                column_id: "age".into(),
                direction: SortDirection::Asc,
            }),
        );
        let age_names: Vec<&str> = by_age.iter().map(|p| p.name.as_str()).collect();
        assert_eq!(
            age_names,
            vec![
                "alpha-cache",   // 15m
                "delta-unset",   // 1h
                "beta-db",       // 2h
                "zeta-data",     // 4d
                "gamma-archive", // 10d
            ]
        );

        // Status, volume, access_mode, storage_class sorts
        let by_status = filter_and_sort_persistent_volume_claims(
            &pvcs,
            &ns_filter,
            Some(&SortState {
                column_id: "status".into(),
                direction: SortDirection::Asc,
            }),
        );
        assert_eq!(by_status[0].status, "Bound");
        assert_eq!(by_status[2].status, "Lost");
        assert_eq!(by_status[3].status, "Pending");

        let by_volume = filter_and_sort_persistent_volume_claims(
            &pvcs,
            &ns_filter,
            Some(&SortState {
                column_id: "volume".into(),
                direction: SortDirection::Asc,
            }),
        );
        assert_eq!(by_volume[0].volume, "-");
        assert_eq!(by_volume[2].volume, "pv-01");

        let by_access = filter_and_sort_persistent_volume_claims(
            &pvcs,
            &ns_filter,
            Some(&SortState {
                column_id: "access_mode".into(),
                direction: SortDirection::Asc,
            }),
        );
        assert_eq!(by_access[0].access_mode, "-");
        assert_eq!(by_access[1].access_mode, "ReadOnlyMany");

        let by_sc = filter_and_sort_persistent_volume_claims(
            &pvcs,
            &ns_filter,
            Some(&SortState {
                column_id: "storage_class".into(),
                direction: SortDirection::Asc,
            }),
        );
        assert_eq!(by_sc[0].storage_class, "-");
        assert_eq!(by_sc[1].storage_class, "ebs-sc");
    }

    #[test]
    fn test_persistent_volume_claims_empty_message_and_presentation_mode() {
        let mut state = PersistentVolumeClaimsState::default();
        assert_eq!(
            persistent_volume_claims_empty_message(&state),
            "No persistent volume claims found in this cluster."
        );
        assert_eq!(
            persistent_volume_claims_presentation_mode(&state),
            PersistentVolumeClaimsPresentationMode::Table
        );

        state.status = PersistentVolumeClaimsLoadStatus::Error;
        state.error_message = Some("forbidden".to_string());
        assert_eq!(
            persistent_volume_claims_presentation_mode(&state),
            PersistentVolumeClaimsPresentationMode::ErrorCard
        );

        state.persistent_volume_claims.push(sample_pvc(
            "data-pg-0",
            "prod",
            "pv-01",
            "20Gi",
            "ReadWriteOnce",
            "gp3",
            "Bound",
            "2d",
        ));
        assert_eq!(
            persistent_volume_claims_empty_message(&state),
            "No persistent volume claims found in the selected namespace(s)."
        );
        assert_eq!(
            persistent_volume_claims_presentation_mode(&state),
            PersistentVolumeClaimsPresentationMode::Table
        );
    }

    #[test]
    fn test_persistent_volume_claim_context_menu_entries_include_namespace_filter_and_actions() {
        let pvc = sample_pvc(
            "data-pg-0",
            "prod",
            "pv-01",
            "20Gi",
            "ReadWriteOnce",
            "gp3",
            "Bound",
            "2d",
        );
        let entries = build_persistent_volume_claim_context_menu_entries(
            &pvc,
            |_, _| {},
            |_, _| {},
            |_, _| {},
            |_, _| {},
            |_, _| {},
            |_, _| {},
        );
        assert_eq!(entries.len(), 8);

        let labels: Vec<Option<&str>> = entries
            .iter()
            .map(|entry| match entry {
                ContextMenuEntry::Item(item) => Some(item.label.as_ref()),
                ContextMenuEntry::Separator => None,
            })
            .collect();
        assert_eq!(
            labels,
            vec![
                Some("Copy Name"),
                Some("Filter by \"prod\""),
                Some("Refresh"),
                None,
                Some("View Details"),
                Some("View YAML"),
                None,
                Some("Delete PersistentVolumeClaim"),
            ]
        );

        match &entries[7] {
            ContextMenuEntry::Item(item) => {
                assert!(item.danger);
                assert!(!item.disabled);
                assert!(item.on_select.is_some());
            }
            ContextMenuEntry::Separator => panic!("Expected Delete item at index 7"),
        }
    }

    #[test]
    fn test_filter_events_for_persistent_volume_claim_matches_namespace_and_object() {
        let pvc = sample_pvc(
            "data-pg-0",
            "prod",
            "pv-01",
            "20Gi",
            "ReadWriteOnce",
            "gp3",
            "Bound",
            "2d",
        );
        let events = vec![
            EventInfo {
                uid: "ev-1".to_string(),
                name: String::new(),
                r#type: "Normal".to_string(),
                reason: "ProvisioningSucceeded".to_string(),
                message: "Successfully provisioned volume".to_string(),
                object_kind: "PersistentVolumeClaim".to_string(),
                object_name: "data-pg-0".to_string(),
                namespace: "prod".to_string(),
                source: "ebs-csi".to_string(),
                count: 1,
                time: "10m".to_string(),
                first_seen: "10m ago".to_string(),
                last_seen: "10m ago".to_string(),
                labels: Default::default(),
                annotations: Default::default(),
            },
            EventInfo {
                uid: "ev-2".to_string(),
                name: String::new(),
                r#type: "Warning".to_string(),
                reason: "ProvisioningFailed".to_string(),
                message: "Same name in other namespace".to_string(),
                object_kind: "PersistentVolumeClaim".to_string(),
                object_name: "data-pg-0".to_string(),
                namespace: "staging".to_string(),
                source: "ebs-csi".to_string(),
                count: 1,
                time: "5m".to_string(),
                first_seen: "5m ago".to_string(),
                last_seen: "5m ago".to_string(),
                labels: Default::default(),
                annotations: Default::default(),
            },
            EventInfo {
                uid: "ev-3".to_string(),
                name: String::new(),
                r#type: "Normal".to_string(),
                reason: "Attached".to_string(),
                message: "Pod mounted claim data-pg-0".to_string(),
                object_kind: "Pod".to_string(),
                object_name: "pg-0".to_string(),
                namespace: "prod".to_string(),
                source: "kubelet".to_string(),
                count: 1,
                time: "2m".to_string(),
                first_seen: "2m ago".to_string(),
                last_seen: "2m ago".to_string(),
                labels: Default::default(),
                annotations: Default::default(),
            },
        ];

        let matched = filter_events_for_persistent_volume_claim(&events, &pvc);
        let uids: Vec<&str> = matched.iter().map(|e| e.uid.as_str()).collect();
        assert_eq!(uids, vec!["ev-1", "ev-3"]);
    }
}
