use std::cmp::Ordering;
use std::rc::Rc;

use gpui::{
    AnyElement, App, ClickEvent, ElementId, FontWeight, MouseDownEvent, SharedString, Window, div,
    prelude::*, px,
};
use lucide_icons::Icon;
use orbit_core::kubernetes::models::{CronJobInfo, EventInfo};

use crate::components::data_table::{
    DataTableCallbacks, SortState, TableColumn, render_data_table, sort_rows_stable,
};
use crate::components::{
    ContextMenuEntry, ContextMenuItem, StatusBadgeVariant, render_status_badge_from_str,
};
use crate::state::{CronJobsLoadStatus, CronJobsState, NamespaceFilterState};
use crate::theme::{OrbitTheme, apply_text_step};
use crate::views::{
    nonzero_metric_text_color, parse_age_seconds, render_resource_error_banner,
    render_resource_error_state, render_resource_view_header, render_truncated_cell,
    table_numeric_font,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CronJobsPresentationMode {
    Table,
    ErrorCard,
}

pub fn cronjob_columns() -> Vec<TableColumn> {
    vec![
        TableColumn::new("name", "Name").flex_width(2.0, px(160.0)),
        TableColumn::new("namespace", "Namespace").flex_width(1.1, px(110.0)),
        TableColumn::new("schedule", "Schedule")
            .flex_width(1.2, px(130.0))
            .monospace(true),
        TableColumn::new("suspend", "Suspend").fixed_width(px(110.0)),
        TableColumn::new("active", "Active Jobs")
            .fixed_width(px(96.0))
            .right_numeric(),
        TableColumn::new("last_schedule", "Last Schedule")
            .fixed_width(px(120.0))
            .right_numeric(),
        TableColumn::new("age", "Age")
            .fixed_width(px(76.0))
            .right_numeric(),
    ]
}

pub fn cronjob_suspend_status(suspend: bool) -> &'static str {
    if suspend { "Suspended" } else { "Active" }
}

pub fn parse_last_schedule_seconds(last_schedule: Option<&str>) -> u64 {
    let Some(raw) = last_schedule else {
        return u64::MAX;
    };
    let trimmed = raw.trim().strip_suffix("ago").unwrap_or(raw.trim()).trim();
    if trimmed.is_empty() {
        return u64::MAX;
    }
    parse_age_seconds(trimmed)
}

fn compare_cronjobs_by_col(a: &CronJobInfo, b: &CronJobInfo, col_id: &str) -> Ordering {
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
        "schedule" => a
            .schedule
            .to_ascii_lowercase()
            .cmp(&b.schedule.to_ascii_lowercase())
            .then_with(tie_break),
        "suspend" => a.suspend.cmp(&b.suspend).then_with(tie_break),
        "active" => a.active.cmp(&b.active).then_with(tie_break),
        "last_schedule" => parse_last_schedule_seconds(a.last_schedule.as_deref())
            .cmp(&parse_last_schedule_seconds(b.last_schedule.as_deref()))
            .then_with(tie_break),
        "age" => parse_age_seconds(&a.age)
            .cmp(&parse_age_seconds(&b.age))
            .then_with(tie_break),
        _ => tie_break(),
    }
}

pub fn filter_and_sort_cronjobs<'a>(
    cronjobs: &'a [CronJobInfo],
    namespaces: &NamespaceFilterState,
    sort: Option<&SortState>,
) -> Vec<&'a CronJobInfo> {
    let mut rows: Vec<&'a CronJobInfo> = cronjobs
        .iter()
        .filter(|cj| namespaces.matches_resource_namespace(Some(&cj.namespace)))
        .collect();
    sort_rows_stable(&mut rows, sort, |a, b, col_id| {
        compare_cronjobs_by_col(a, b, col_id)
    });
    rows
}

pub fn cronjobs_empty_message(state: &CronJobsState) -> &'static str {
    if state.cronjobs.is_empty() {
        "No cronjobs found in this cluster."
    } else {
        "No cronjobs found in the selected namespace(s)."
    }
}

pub fn cronjobs_presentation_mode(state: &CronJobsState) -> CronJobsPresentationMode {
    if state.status == CronJobsLoadStatus::Error && state.cronjobs.is_empty() {
        CronJobsPresentationMode::ErrorCard
    } else {
        CronJobsPresentationMode::Table
    }
}

fn render_cronjob_cell(
    row_idx: usize,
    cj: &CronJobInfo,
    col: &TableColumn,
    theme: &OrbitTheme,
) -> AnyElement {
    match col.id.as_ref() {
        "name" => render_truncated_cell(
            ElementId::NamedInteger(SharedString::from("cj-name"), row_idx as u64),
            cj.name.clone(),
            Some(SharedString::from(cj.name.clone())),
            theme.text.primary,
            FontWeight::MEDIUM,
            false,
            theme,
        ),
        "namespace" => render_truncated_cell(
            ElementId::NamedInteger(SharedString::from("cj-ns"), row_idx as u64),
            cj.namespace.clone(),
            None,
            theme.text.secondary,
            FontWeight::NORMAL,
            false,
            theme,
        ),
        "schedule" => {
            let (text, color) = if cj.schedule.trim().is_empty() {
                ("-".to_string(), theme.text.muted)
            } else {
                (cj.schedule.clone(), theme.text.primary)
            };
            render_truncated_cell(
                ElementId::NamedInteger(SharedString::from("cj-sched"), row_idx as u64),
                text,
                None,
                color,
                FontWeight::NORMAL,
                true,
                theme,
            )
        }
        "suspend" => render_status_badge_from_str(
            cronjob_suspend_status(cj.suspend),
            StatusBadgeVariant::Inline,
            theme,
        ),
        "active" => apply_text_step(div(), theme.typography.size_xs)
            .font(table_numeric_font(theme))
            .text_color(nonzero_metric_text_color(cj.active, theme))
            .child(cj.active.to_string())
            .into_any_element(),
        "last_schedule" => {
            let (text, color) = match cj.last_schedule.as_deref() {
                Some(s) if !s.trim().is_empty() => (s.to_string(), theme.text.secondary),
                _ => ("-".to_string(), theme.text.muted),
            };
            apply_text_step(div(), theme.typography.size_xs)
                .font(theme.typography.mono_font(FontWeight::NORMAL))
                .text_color(color)
                .child(text)
                .into_any_element()
        }
        "age" => apply_text_step(div(), theme.typography.size_xs)
            .font(theme.typography.mono_font(FontWeight::NORMAL))
            .text_color(theme.text.muted)
            .child(cj.age.clone())
            .into_any_element(),
        _ => div().into_any_element(),
    }
}

#[allow(clippy::too_many_arguments)]
pub fn build_cronjob_context_menu_entries(
    cj: &CronJobInfo,
    on_copy_name: impl 'static + Fn(&mut Window, &mut App),
    on_filter_namespace: impl 'static + Fn(&mut Window, &mut App),
    on_refresh: impl 'static + Fn(&mut Window, &mut App),
    on_view_details: impl 'static + Fn(&mut Window, &mut App),
    on_view_yaml: impl 'static + Fn(&mut Window, &mut App),
    on_trigger_now: impl 'static + Fn(&mut Window, &mut App),
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
                format!("Filter by \"{}\"", cj.namespace),
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
            ContextMenuItem::new("trigger-now", "Trigger Now")
                .icon(Icon::Play)
                .on_select(on_trigger_now),
        ),
        ContextMenuEntry::Separator,
        ContextMenuEntry::Item(
            ContextMenuItem::new("delete-cronjob", "Delete CronJob")
                .icon(Icon::Trash2)
                .danger(true)
                .on_select(on_delete),
        ),
    ]
}

#[allow(clippy::too_many_arguments)]
pub fn render_cronjobs_view(
    state: &CronJobsState,
    namespaces: &NamespaceFilterState,
    theme: &OrbitTheme,
    on_sort: impl 'static + Fn(SharedString, &ClickEvent, &mut Window, &mut App),
    on_row_click: impl 'static + Fn(usize, SharedString, &ClickEvent, &mut Window, &mut App),
    on_row_quick_yaml: impl 'static + Fn(usize, SharedString, &ClickEvent, &mut Window, &mut App),
    on_row_context_menu: impl 'static
    + Fn(SharedString, &CronJobInfo, &MouseDownEvent, &mut Window, &mut App),
    on_refresh: impl 'static + Fn(&mut Window, &mut App),
) -> AnyElement {
    let columns = cronjob_columns();
    let filtered_rows: Rc<Vec<CronJobInfo>> = Rc::new(
        filter_and_sort_cronjobs(&state.cronjobs, namespaces, state.table.sort.as_ref())
            .into_iter()
            .cloned()
            .collect(),
    );
    let total_count = state.cronjobs.len();
    let visible_count = filtered_rows.len();
    let is_loading = state.status == CronJobsLoadStatus::Loading;
    let on_refresh_rc: super::ActionCallback = Rc::new(on_refresh);

    let count_label = if namespaces.is_all_namespaces() {
        format!("{total_count} cronjobs")
    } else {
        format!("{visible_count} of {total_count} cronjobs")
    };

    let header = render_resource_view_header(count_label, Some(namespaces), None, theme);

    let error_banner = state.error_message.as_ref().map(|message| {
        render_resource_error_banner(
            "cronjobs-retry-btn",
            format!("Failed to load cronjobs: {message}"),
            is_loading,
            theme,
            on_refresh_rc.clone(),
        )
    });

    let body = match cronjobs_presentation_mode(state) {
        CronJobsPresentationMode::ErrorCard => {
            let message = state
                .error_message
                .clone()
                .unwrap_or_else(|| "Unknown error".to_string());
            render_resource_error_state(
                "cronjobs-retry-btn",
                "Unable to Load CronJobs",
                message,
                is_loading,
                theme,
                on_refresh_rc,
            )
        }
        CronJobsPresentationMode::Table => {
            let mut table_state = state.table.clone();
            table_state.loading = is_loading && state.cronjobs.is_empty();
            table_state.refreshing = is_loading && !state.cronjobs.is_empty();
            table_state.empty_message = SharedString::from(cronjobs_empty_message(state));

            let key_rows = filtered_rows.clone();
            let cell_rows = filtered_rows.clone();
            let ctx_rows = filtered_rows.clone();
            let callbacks = DataTableCallbacks::new(
                move |idx| {
                    key_rows
                        .get(idx)
                        .map(|cj| format!("{}/{}", cj.namespace, cj.name))
                        .unwrap_or_default()
                },
                move |idx, col, col_theme| {
                    cell_rows
                        .get(idx)
                        .map(|cj| render_cronjob_cell(idx, cj, col, col_theme))
                        .unwrap_or_else(|| div().into_any_element())
                },
            )
            .on_sort(on_sort)
            .on_row_click(on_row_click)
            .on_row_quick_yaml(on_row_quick_yaml)
            .on_row_context_menu(move |idx, row_key, event, window, cx| {
                if let Some(cj) = ctx_rows.get(idx) {
                    on_row_context_menu(row_key, cj, event, window, cx);
                }
            });

            render_data_table(
                "cronjobs-table",
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
            cronjobs_presentation_mode(state) == CronJobsPresentationMode::Table,
            |el| el.when_some(error_banner, |el, banner| el.child(banner)),
        )
        .child(body)
        .into_any_element()
}

pub fn filter_events_for_cronjob<'a>(
    events: &'a [EventInfo],
    cronjob: &CronJobInfo,
) -> Vec<&'a EventInfo> {
    let prefix = format!("{}-", cronjob.name);
    events
        .iter()
        .filter(|ev| {
            ev.namespace == cronjob.namespace
                && ((ev.object_kind == "CronJob" && ev.object_name == cronjob.name)
                    || ((ev.object_kind == "Job" || ev.object_kind == "Pod")
                        && ev.object_name.starts_with(&prefix))
                    || ev.message.contains(&cronjob.name))
        })
        .collect()
}

pub fn render_cronjob_drawer_overview(cj: &CronJobInfo, theme: &OrbitTheme) -> AnyElement {
    let summary = crate::views::render_drawer_key_values(
        vec![
            ("Name", cj.name.clone(), true),
            ("Namespace", cj.namespace.clone(), true),
            (
                "Schedule",
                if cj.schedule.trim().is_empty() {
                    "-".to_string()
                } else {
                    cj.schedule.clone()
                },
                true,
            ),
            (
                "Status",
                cronjob_suspend_status(cj.suspend).to_string(),
                false,
            ),
            ("Active Jobs", cj.active.to_string(), true),
            (
                "Last Schedule",
                cj.last_schedule
                    .clone()
                    .filter(|s| !s.trim().is_empty())
                    .unwrap_or_else(|| "-".to_string()),
                true,
            ),
            ("Age", cj.age.clone(), true),
        ],
        theme,
    );

    let containers_section =
        crate::views::render_drawer_containers_section(&cj.containers, &cj.images, theme);

    div()
        .id("cronjob-drawer-overview")
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
            &cj.labels,
            "No labels",
            theme,
        ))
        .child(crate::views::render_drawer_kv_badges(
            "ANNOTATIONS",
            &cj.annotations,
            "No annotations",
            theme,
        ))
        .into_any_element()
}

pub fn render_cronjob_drawer_events(
    events: &[EventInfo],
    loading: bool,
    cj: &CronJobInfo,
    theme: &OrbitTheme,
) -> AnyElement {
    let matched = filter_events_for_cronjob(events, cj);
    div()
        .id("cronjob-drawer-events")
        .flex_1()
        .min_h_0()
        .overflow_y_scroll()
        .p_4()
        .child(crate::views::render_drawer_events_list(
            &matched,
            loading,
            "No events found for this cronjob.",
            theme,
        ))
        .into_any_element()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::components::data_table::{ColumnAlign, SortDirection};
    use crate::components::status_badge::{SemanticStatus, classify_status};
    use orbit_core::kubernetes::models::{ContainerImageInfo, NamespaceInfo};

    #[allow(clippy::too_many_arguments)]
    fn make_cronjob(
        name: &str,
        namespace: &str,
        schedule: &str,
        suspend: bool,
        active: i32,
        last_schedule: Option<&str>,
        age: &str,
    ) -> CronJobInfo {
        CronJobInfo {
            kind: "CronJob".to_string(),
            name: name.to_string(),
            namespace: namespace.to_string(),
            schedule: schedule.to_string(),
            suspend,
            active,
            last_schedule: last_schedule.map(ToString::to_string),
            age: age.to_string(),
            images: vec!["busybox:1.36".to_string()],
            containers: vec![ContainerImageInfo {
                name: name.to_string(),
                image: "busybox:1.36".to_string(),
            }],
            labels: Default::default(),
            annotations: Default::default(),
        }
    }

    #[test]
    fn test_cronjob_columns_schema_and_alignment() {
        let cols = cronjob_columns();
        let ids: Vec<&str> = cols.iter().map(|c| c.id.as_ref()).collect();
        let labels: Vec<&str> = cols.iter().map(|c| c.header.as_ref()).collect();
        assert_eq!(
            ids,
            vec![
                "name",
                "namespace",
                "schedule",
                "suspend",
                "active",
                "last_schedule",
                "age",
            ]
        );
        assert_eq!(
            labels,
            vec![
                "Name",
                "Namespace",
                "Schedule",
                "Suspend",
                "Active Jobs",
                "Last Schedule",
                "Age",
            ]
        );
        assert!(cols[2].monospace);
        for idx in [4, 5, 6] {
            assert_eq!(cols[idx].align, ColumnAlign::Right);
            assert!(cols[idx].monospace);
        }
        for idx in [0, 1, 2, 3] {
            assert_eq!(cols[idx].align, ColumnAlign::Left);
        }
    }

    #[test]
    fn test_cronjob_suspend_status_and_parse_last_schedule_seconds() {
        assert_eq!(cronjob_suspend_status(true), "Suspended");
        assert_eq!(
            classify_status(cronjob_suspend_status(true)),
            SemanticStatus::Warning
        );
        assert_eq!(cronjob_suspend_status(false), "Active");
        assert_eq!(
            classify_status(cronjob_suspend_status(false)),
            SemanticStatus::Success
        );

        assert_eq!(parse_last_schedule_seconds(Some("0s")), 0);
        assert_eq!(parse_last_schedule_seconds(Some("45s ago")), 45);
        assert_eq!(parse_last_schedule_seconds(Some("12m ago")), 720);
        assert_eq!(parse_last_schedule_seconds(Some("2h ago")), 7_200);
        assert_eq!(parse_last_schedule_seconds(Some("3d ago")), 259_200);
        assert_eq!(parse_last_schedule_seconds(Some("")), u64::MAX);
        assert_eq!(parse_last_schedule_seconds(None), u64::MAX);
    }

    #[test]
    fn test_filter_and_sort_cronjobs_across_columns_and_namespaces() {
        let cronjobs = vec![
            make_cronjob(
                "nightly-backup",
                "prod",
                "0 2 * * *",
                false,
                2,
                Some("10m ago"),
                "5d",
            ),
            make_cronjob(
                "audit-report",
                "staging",
                "*/5 * * * *",
                true,
                0,
                None,
                "2h",
            ),
            make_cronjob(
                "cache-warm",
                "prod",
                "0 * * * *",
                false,
                1,
                Some("30s ago"),
                "1d",
            ),
        ];

        // Namespace filter
        let mut ns_filter = NamespaceFilterState {
            cluster_id: Some("ctx-a".to_string()),
            namespaces: vec![
                NamespaceInfo {
                    name: "prod".to_string(),
                    status: "Active".to_string(),
                    age: "10d".to_string(),
                    created: String::new(),
                    uid: String::new(),
                    is_system: false,
                    labels: Default::default(),
                    annotations: Default::default(),
                },
                NamespaceInfo {
                    name: "staging".to_string(),
                    status: "Active".to_string(),
                    age: "10d".to_string(),
                    created: String::new(),
                    uid: String::new(),
                    is_system: false,
                    labels: Default::default(),
                    annotations: Default::default(),
                },
            ],
            ..Default::default()
        };
        ns_filter.toggle_namespace("prod");
        let prod_only = filter_and_sort_cronjobs(&cronjobs, &ns_filter, None);
        assert_eq!(prod_only.len(), 2);
        assert!(prod_only.iter().all(|c| c.namespace == "prod"));

        let all_ns = NamespaceFilterState::default();

        // Sort by name asc
        let by_name = filter_and_sort_cronjobs(
            &cronjobs,
            &all_ns,
            Some(&SortState {
                column_id: "name".to_string(),
                direction: SortDirection::Asc,
            }),
        );
        let names: Vec<&str> = by_name.iter().map(|c| c.name.as_str()).collect();
        assert_eq!(names, vec!["audit-report", "cache-warm", "nightly-backup"]);

        // Sort by namespace desc
        let by_ns_desc = filter_and_sort_cronjobs(
            &cronjobs,
            &all_ns,
            Some(&SortState {
                column_id: "namespace".to_string(),
                direction: SortDirection::Desc,
            }),
        );
        assert_eq!(by_ns_desc[0].namespace, "staging");

        // Sort by schedule asc
        let by_schedule = filter_and_sort_cronjobs(
            &cronjobs,
            &all_ns,
            Some(&SortState {
                column_id: "schedule".to_string(),
                direction: SortDirection::Asc,
            }),
        );
        assert_eq!(by_schedule[0].name, "audit-report");

        // Sort by suspend desc (true first)
        let by_suspend_desc = filter_and_sort_cronjobs(
            &cronjobs,
            &all_ns,
            Some(&SortState {
                column_id: "suspend".to_string(),
                direction: SortDirection::Desc,
            }),
        );
        assert_eq!(by_suspend_desc[0].name, "audit-report");

        // Sort by active desc
        let by_active_desc = filter_and_sort_cronjobs(
            &cronjobs,
            &all_ns,
            Some(&SortState {
                column_id: "active".to_string(),
                direction: SortDirection::Desc,
            }),
        );
        assert_eq!(by_active_desc[0].name, "nightly-backup");
        assert_eq!(by_active_desc[1].name, "cache-warm");
        assert_eq!(by_active_desc[2].name, "audit-report");

        // Sort by last_schedule asc (most recent first, None last)
        let by_last = filter_and_sort_cronjobs(
            &cronjobs,
            &all_ns,
            Some(&SortState {
                column_id: "last_schedule".to_string(),
                direction: SortDirection::Asc,
            }),
        );
        let last_names: Vec<&str> = by_last.iter().map(|c| c.name.as_str()).collect();
        assert_eq!(
            last_names,
            vec!["cache-warm", "nightly-backup", "audit-report"]
        );

        // Sort by age asc
        let by_age = filter_and_sort_cronjobs(
            &cronjobs,
            &all_ns,
            Some(&SortState {
                column_id: "age".to_string(),
                direction: SortDirection::Asc,
            }),
        );
        let age_names: Vec<&str> = by_age.iter().map(|c| c.name.as_str()).collect();
        assert_eq!(
            age_names,
            vec!["audit-report", "cache-warm", "nightly-backup"]
        );
    }

    #[test]
    fn test_cronjobs_empty_message_and_presentation_mode() {
        let mut state = CronJobsState::default();
        assert_eq!(
            cronjobs_empty_message(&state),
            "No cronjobs found in this cluster."
        );
        assert_eq!(
            cronjobs_presentation_mode(&state),
            CronJobsPresentationMode::Table
        );

        state.status = CronJobsLoadStatus::Error;
        assert_eq!(
            cronjobs_presentation_mode(&state),
            CronJobsPresentationMode::ErrorCard
        );

        state.cronjobs.push(make_cronjob(
            "nightly-backup",
            "prod",
            "0 2 * * *",
            false,
            0,
            None,
            "1d",
        ));
        assert_eq!(
            cronjobs_empty_message(&state),
            "No cronjobs found in the selected namespace(s)."
        );
        assert_eq!(
            cronjobs_presentation_mode(&state),
            CronJobsPresentationMode::Table
        );
    }

    #[test]
    fn test_filter_events_for_cronjob_matches_cronjob_child_jobs_pods_and_messages() {
        let cj = make_cronjob(
            "nightly-backup",
            "prod",
            "0 2 * * *",
            false,
            1,
            Some("1m ago"),
            "3d",
        );

        let mk_event = |uid: &str, ns: &str, kind: &str, obj: &str, msg: &str| EventInfo {
            uid: uid.to_string(),
            name: String::new(),
            time: "1m".to_string(),
            r#type: "Normal".to_string(),
            reason: "SuccessfulCreate".to_string(),
            object_name: obj.to_string(),
            object_kind: kind.to_string(),
            message: msg.to_string(),
            namespace: ns.to_string(),
            source: "cronjob-controller".to_string(),
            first_seen: String::new(),
            last_seen: String::new(),
            count: 1,
            labels: Default::default(),
            annotations: Default::default(),
        };

        let events = vec![
            mk_event(
                "1",
                "prod",
                "CronJob",
                "nightly-backup",
                "Created job nightly-backup-28910",
            ),
            mk_event("2", "prod", "Job", "nightly-backup-28910", "Job completed"),
            mk_event(
                "3",
                "prod",
                "Pod",
                "nightly-backup-28910-abcde",
                "Started container",
            ),
            mk_event(
                "4",
                "prod",
                "Namespace",
                "prod",
                "Error triggering nightly-backup",
            ),
            mk_event(
                "5",
                "staging",
                "CronJob",
                "nightly-backup",
                "Wrong namespace",
            ),
        ];

        let matched = filter_events_for_cronjob(&events, &cj);
        let uids: Vec<&str> = matched.iter().map(|e| e.uid.as_str()).collect();
        assert_eq!(uids, vec!["1", "2", "3", "4"]);
    }

    #[test]
    fn test_build_cronjob_context_menu_entries_structure() {
        let cj = make_cronjob("nightly-backup", "prod", "0 2 * * *", false, 0, None, "1d");
        let entries = build_cronjob_context_menu_entries(
            &cj,
            |_, _| {},
            |_, _| {},
            |_, _| {},
            |_, _| {},
            |_, _| {},
            |_, _| {},
            |_, _| {},
        );

        assert_eq!(entries.len(), 10);
        assert!(matches!(entries[3], ContextMenuEntry::Separator));
        assert!(matches!(entries[6], ContextMenuEntry::Separator));
        assert!(matches!(entries[8], ContextMenuEntry::Separator));

        let labels: Vec<&str> = entries
            .iter()
            .filter_map(|e| match e {
                ContextMenuEntry::Item(item) => Some(item.label.as_ref()),
                ContextMenuEntry::Separator => None,
            })
            .collect();
        assert_eq!(
            labels,
            vec![
                "Copy Name",
                "Filter by \"prod\"",
                "Refresh",
                "View Details",
                "View YAML",
                "Trigger Now",
                "Delete CronJob",
            ]
        );

        match &entries[9] {
            ContextMenuEntry::Item(delete_item) => {
                assert!(delete_item.danger);
                assert!(!delete_item.disabled);
                assert!(delete_item.on_select.is_some());
            }
            ContextMenuEntry::Separator => panic!("Expected Delete CronJob item"),
        }
    }
}
