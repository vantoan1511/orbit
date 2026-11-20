use std::cmp::Ordering;
use std::rc::Rc;

use gpui::{
    AnyElement, App, ClickEvent, FontWeight, MouseDownEvent, SharedString, Window, div, prelude::*,
    px,
};
use lucide_icons::Icon;
use orbit_core::kubernetes::models::{EventInfo, JobInfo};

use crate::components::data_table::{
    DataTableCallbacks, SortState, TableColumn, render_data_table, sort_rows_stable,
};
use crate::components::{
    ContextMenuEntry, ContextMenuItem, StatusBadgeVariant, render_status_badge_from_str,
};
use crate::icons::render_icon;
use crate::state::{JobsLoadStatus, JobsState, NamespaceFilterState};
use crate::theme::{OrbitTheme, apply_text_step};
use crate::views::{format_images_summary, parse_age_seconds, render_action_button};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JobsPresentationMode {
    Table,
    ErrorCard,
}

pub fn job_columns() -> Vec<TableColumn> {
    vec![
        TableColumn::new("name", "Name").flex_width(2.0, px(160.0)),
        TableColumn::new("namespace", "Namespace").flex_width(1.1, px(110.0)),
        TableColumn::new("status", "Status").fixed_width(px(110.0)),
        TableColumn::new("completions", "Completions")
            .fixed_width(px(108.0))
            .right_numeric(),
        TableColumn::new("duration", "Duration")
            .fixed_width(px(115.0))
            .right_numeric()
            .monospace(true),
        TableColumn::new("images", "Images")
            .flex_width(1.8, px(160.0))
            .monospace(true),
        TableColumn::new("age", "Age")
            .fixed_width(px(76.0))
            .right_numeric(),
    ]
}

pub fn parse_completions_pair(completions: &str) -> (i32, i32) {
    let mut parts = completions.trim().split('/');
    let Some(succeeded_str) = parts.next() else {
        return (0, 0);
    };
    let Some(desired_str) = parts.next() else {
        return (0, 0);
    };
    if parts.next().is_some() {
        return (0, 0);
    }

    match (
        succeeded_str.trim().parse::<i32>(),
        desired_str.trim().parse::<i32>(),
    ) {
        (Ok(succeeded), Ok(desired)) => (succeeded, desired),
        _ => (0, 0),
    }
}

pub fn parse_duration_seconds(duration: Option<&str>) -> u64 {
    let Some(raw) = duration else {
        return u64::MAX;
    };
    let trimmed = raw
        .trim()
        .strip_suffix("(running)")
        .unwrap_or(raw.trim())
        .trim();
    if trimmed.is_empty() {
        return u64::MAX;
    }

    let mut total: u64 = 0;
    let mut parsed_any = false;
    for part in trimmed.split_whitespace() {
        let (num_str, multiplier) = if let Some(n) = part.strip_suffix('d') {
            (n, 86_400_u64)
        } else if let Some(n) = part.strip_suffix('h') {
            (n, 3_600_u64)
        } else if let Some(n) = part.strip_suffix('m') {
            (n, 60_u64)
        } else if let Some(n) = part.strip_suffix('s') {
            (n, 1_u64)
        } else {
            return u64::MAX;
        };

        let Ok(value) = num_str.parse::<u64>() else {
            return u64::MAX;
        };
        total = total.saturating_add(value.saturating_mul(multiplier));
        parsed_any = true;
    }

    if parsed_any { total } else { u64::MAX }
}

fn compare_jobs_by_col(a: &JobInfo, b: &JobInfo, col_id: &str) -> Ordering {
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
        "completions" => parse_completions_pair(&a.completions)
            .cmp(&parse_completions_pair(&b.completions))
            .then_with(tie_break),
        "duration" => parse_duration_seconds(a.duration.as_deref())
            .cmp(&parse_duration_seconds(b.duration.as_deref()))
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

pub fn filter_and_sort_jobs<'a>(
    jobs: &'a [JobInfo],
    namespaces: &NamespaceFilterState,
    sort: Option<&SortState>,
) -> Vec<&'a JobInfo> {
    let mut rows: Vec<&'a JobInfo> = jobs
        .iter()
        .filter(|job| namespaces.matches_resource_namespace(Some(&job.namespace)))
        .collect();
    sort_rows_stable(&mut rows, sort, |a, b, col_id| {
        compare_jobs_by_col(a, b, col_id)
    });
    rows
}

pub fn jobs_empty_message(state: &JobsState) -> &'static str {
    if state.jobs.is_empty() {
        "No jobs found in this cluster."
    } else {
        "No jobs found in the selected namespace(s)."
    }
}

pub fn jobs_presentation_mode(state: &JobsState) -> JobsPresentationMode {
    if state.status == JobsLoadStatus::Error && state.jobs.is_empty() {
        JobsPresentationMode::ErrorCard
    } else {
        JobsPresentationMode::Table
    }
}

fn render_job_cell(job: &JobInfo, col: &TableColumn, theme: &OrbitTheme) -> AnyElement {
    match col.id.as_ref() {
        "name" => apply_text_step(div(), theme.typography.size_xs)
            .font_weight(FontWeight::MEDIUM)
            .text_color(theme.text.primary)
            .truncate()
            .child(job.name.clone())
            .into_any_element(),
        "namespace" => apply_text_step(div(), theme.typography.size_xs)
            .text_color(theme.text.secondary)
            .truncate()
            .child(job.namespace.clone())
            .into_any_element(),
        "status" => render_status_badge_from_str(&job.status, StatusBadgeVariant::Inline, theme),
        "completions" => apply_text_step(div(), theme.typography.size_xs)
            .font(theme.typography.mono_font(FontWeight::MEDIUM))
            .text_color(theme.text.primary)
            .child(job.completions.clone())
            .into_any_element(),
        "duration" => {
            let (text, color) = match job.duration.as_deref() {
                Some(d) if !d.trim().is_empty() => (d.to_string(), theme.text.secondary),
                _ => ("-".to_string(), theme.text.muted),
            };
            apply_text_step(div(), theme.typography.size_xs)
                .font(theme.typography.mono_font(FontWeight::NORMAL))
                .text_color(color)
                .child(text)
                .into_any_element()
        }
        "images" => {
            let (primary, extra) = format_images_summary(&job.images);
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
            .child(job.age.clone())
            .into_any_element(),
        _ => div().into_any_element(),
    }
}

pub fn build_job_context_menu_entries(
    job: &JobInfo,
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
                format!("Filter by \"{}\"", job.namespace),
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
            ContextMenuItem::new("delete-job", "Delete Job")
                .icon(Icon::Trash2)
                .danger(true)
                .disabled(true),
        ),
    ]
}

#[allow(clippy::too_many_arguments)]
pub fn render_jobs_view(
    state: &JobsState,
    namespaces: &NamespaceFilterState,
    theme: &OrbitTheme,
    on_sort: impl 'static + Fn(SharedString, &ClickEvent, &mut Window, &mut App),
    on_row_click: impl 'static + Fn(usize, SharedString, &ClickEvent, &mut Window, &mut App),
    on_row_context_menu: impl 'static
    + Fn(SharedString, &JobInfo, &MouseDownEvent, &mut Window, &mut App),
    on_refresh: impl 'static + Fn(&mut Window, &mut App),
) -> AnyElement {
    let columns = job_columns();
    let filtered_rows: Rc<Vec<JobInfo>> = Rc::new(
        filter_and_sort_jobs(&state.jobs, namespaces, state.table.sort.as_ref())
            .into_iter()
            .cloned()
            .collect(),
    );
    let total_count = state.jobs.len();
    let visible_count = filtered_rows.len();
    let is_loading = state.status == JobsLoadStatus::Loading;
    type RefreshCallback = Rc<dyn Fn(&mut Window, &mut App)>;
    let on_refresh_rc: RefreshCallback = Rc::new(on_refresh);

    let count_label = if namespaces.is_all_namespaces() {
        format!("{total_count} jobs")
    } else {
        format!("{visible_count} of {total_count} jobs")
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
        );

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
                            .child(format!("Failed to load jobs: {message}")),
                    ),
            )
            .child(render_action_button(
                "jobs-retry-btn",
                Icon::RotateCcw,
                "Retry",
                is_loading,
                theme,
                on_refresh_rc.clone(),
            ))
    });

    let body = match jobs_presentation_mode(state) {
        JobsPresentationMode::ErrorCard => {
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
                                .child("Unable to Load Jobs"),
                        ),
                )
                .child(
                    apply_text_step(div(), theme.typography.size_xs)
                        .text_color(theme.text.secondary)
                        .child(message),
                )
                .child(render_action_button(
                    "jobs-retry-btn",
                    Icon::RotateCcw,
                    "Retry",
                    is_loading,
                    theme,
                    on_refresh_rc,
                ))
                .into_any_element()
        }
        JobsPresentationMode::Table => {
            let mut table_state = state.table.clone();
            table_state.loading = is_loading && state.jobs.is_empty();
            table_state.empty_message = SharedString::from(jobs_empty_message(state));

            let key_rows = filtered_rows.clone();
            let cell_rows = filtered_rows.clone();
            let ctx_rows = filtered_rows.clone();
            let callbacks = DataTableCallbacks::new(
                move |idx| {
                    key_rows
                        .get(idx)
                        .map(|job| format!("{}/{}", job.namespace, job.name))
                        .unwrap_or_default()
                },
                move |idx, col, col_theme| {
                    cell_rows
                        .get(idx)
                        .map(|job| render_job_cell(job, col, col_theme))
                        .unwrap_or_else(|| div().into_any_element())
                },
            )
            .on_sort(on_sort)
            .on_row_click(on_row_click)
            .on_row_context_menu(move |idx, row_key, event, window, cx| {
                if let Some(job) = ctx_rows.get(idx) {
                    on_row_context_menu(row_key, job, event, window, cx);
                }
            });

            render_data_table(
                "jobs-table",
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
            jobs_presentation_mode(state) == JobsPresentationMode::Table,
            |el| el.when_some(error_banner, |el, banner| el.child(banner)),
        )
        .child(body)
        .into_any_element()
}

pub fn filter_events_for_job<'a>(events: &'a [EventInfo], job: &JobInfo) -> Vec<&'a EventInfo> {
    let prefix = format!("{}-", job.name);
    events
        .iter()
        .filter(|ev| {
            ev.namespace == job.namespace
                && ((ev.object_kind == "Job" && ev.object_name == job.name)
                    || (ev.object_kind == "Pod" && ev.object_name.starts_with(&prefix))
                    || ev.message.contains(&job.name))
        })
        .collect()
}

pub fn render_job_drawer_overview(job: &JobInfo, theme: &OrbitTheme) -> AnyElement {
    let summary = crate::views::render_drawer_key_values(
        vec![
            ("Name", job.name.clone(), true),
            ("Namespace", job.namespace.clone(), true),
            ("Status", job.status.clone(), false),
            ("Completions", job.completions.clone(), true),
            (
                "Duration",
                job.duration
                    .clone()
                    .filter(|d| !d.trim().is_empty())
                    .unwrap_or_else(|| "-".to_string()),
                true,
            ),
            ("Age", job.age.clone(), true),
        ],
        theme,
    );

    let containers_section =
        crate::views::render_drawer_containers_section(&job.containers, &job.images, theme);

    div()
        .id("job-drawer-overview")
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
            &job.labels,
            "No labels",
            theme,
        ))
        .child(crate::views::render_drawer_kv_badges(
            "ANNOTATIONS",
            &job.annotations,
            "No annotations",
            theme,
        ))
        .into_any_element()
}

pub fn render_job_drawer_events(
    events: &[EventInfo],
    loading: bool,
    job: &JobInfo,
    theme: &OrbitTheme,
) -> AnyElement {
    let matched = filter_events_for_job(events, job);
    div()
        .id("job-drawer-events")
        .flex_1()
        .min_h_0()
        .overflow_y_scroll()
        .p_4()
        .child(crate::views::render_drawer_events_list(
            &matched,
            loading,
            "No events found for this job.",
            theme,
        ))
        .into_any_element()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::components::data_table::{ColumnAlign, SortDirection};
    use orbit_core::kubernetes::models::{ContainerImageInfo, NamespaceInfo};

    #[allow(clippy::too_many_arguments)]
    fn make_job(
        name: &str,
        namespace: &str,
        status: &str,
        completions: &str,
        duration: Option<&str>,
        image: &str,
        age: &str,
    ) -> JobInfo {
        JobInfo {
            kind: "Job".to_string(),
            name: name.to_string(),
            namespace: namespace.to_string(),
            status: status.to_string(),
            completions: completions.to_string(),
            duration: duration.map(ToString::to_string),
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
    fn test_job_columns_schema_and_alignment() {
        let cols = job_columns();
        let ids: Vec<&str> = cols.iter().map(|c| c.id.as_ref()).collect();
        let labels: Vec<&str> = cols.iter().map(|c| c.header.as_ref()).collect();
        assert_eq!(
            ids,
            vec![
                "name",
                "namespace",
                "status",
                "completions",
                "duration",
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
                "Completions",
                "Duration",
                "Images",
                "Age",
            ]
        );
        assert_eq!(cols[3].align, ColumnAlign::Right);
        assert_eq!(cols[4].align, ColumnAlign::Right);
        assert!(cols[4].monospace);
        assert!(cols[5].monospace);
        assert_eq!(cols[6].align, ColumnAlign::Right);
    }

    #[test]
    fn test_parse_completions_pair_and_duration_seconds() {
        assert_eq!(parse_completions_pair("2/5"), (2, 5));
        assert_eq!(parse_completions_pair("10/10"), (10, 10));
        assert_eq!(parse_completions_pair("invalid"), (0, 0));
        assert_eq!(parse_completions_pair("1/2/3"), (0, 0));

        assert_eq!(parse_duration_seconds(Some("45s")), 45);
        assert_eq!(parse_duration_seconds(Some("2m 15s")), 135);
        assert_eq!(parse_duration_seconds(Some("1h 30m")), 5400);
        assert_eq!(parse_duration_seconds(Some("12s (running)")), 12);
        assert_eq!(parse_duration_seconds(None), u64::MAX);
        assert_eq!(parse_duration_seconds(Some("unknown")), u64::MAX);
    }

    #[test]
    fn test_filter_and_sort_jobs_by_namespace_and_columns() {
        let items = vec![
            make_job(
                "nightly-etl",
                "data",
                "Succeeded",
                "10/10",
                Some("1h 30m"),
                "ghcr.io/orbit/etl:v2",
                "10d",
            ),
            make_job(
                "db-migrate",
                "prod",
                "Active",
                "2/5",
                Some("45s"),
                "ghcr.io/orbit/migrate:v1",
                "2h",
            ),
            make_job(
                "cache-warm",
                "prod",
                "Failed",
                "0/1",
                Some("2m 15s"),
                "busybox:1.36",
                "45m",
            ),
            make_job(
                "pending-job",
                "staging",
                "Unknown",
                "0/1",
                None,
                "alpine:3.19",
                "5m",
            ),
        ];

        let mut ns = NamespaceFilterState::default();
        ns.begin_load_for_cluster("ctx-a");
        ns.apply_loaded(
            "ctx-a",
            1,
            vec![
                NamespaceInfo {
                    name: "prod".to_string(),
                    status: "Active".to_string(),
                    is_system: false,
                    age: "10d".to_string(),
                    labels: Default::default(),
                    annotations: Default::default(),
                    uid: "uid-prod".to_string(),
                    created: String::new(),
                },
                NamespaceInfo {
                    name: "data".to_string(),
                    status: "Active".to_string(),
                    is_system: false,
                    age: "10d".to_string(),
                    labels: Default::default(),
                    annotations: Default::default(),
                    uid: "uid-data".to_string(),
                    created: String::new(),
                },
            ],
        );

        // Single namespace filter
        ns.toggle_namespace("prod");
        let by_name_asc = SortState {
            column_id: "name".to_string(),
            direction: SortDirection::Asc,
        };
        let filtered = filter_and_sort_jobs(&items, &ns, Some(&by_name_asc));
        assert_eq!(
            filtered.iter().map(|j| j.name.as_str()).collect::<Vec<_>>(),
            vec!["cache-warm", "db-migrate"]
        );

        // Multiple namespaces filter
        ns.toggle_namespace("data");
        let multi = filter_and_sort_jobs(&items, &ns, Some(&by_name_asc));
        assert_eq!(multi.len(), 3);

        let all_ns = NamespaceFilterState::default();

        // Namespace asc/desc
        let by_ns_desc = SortState {
            column_id: "namespace".to_string(),
            direction: SortDirection::Desc,
        };
        assert_eq!(
            filter_and_sort_jobs(&items, &all_ns, Some(&by_ns_desc))[0].name,
            "pending-job"
        );

        // Status asc
        let by_status_asc = SortState {
            column_id: "status".to_string(),
            direction: SortDirection::Asc,
        };
        assert_eq!(
            filter_and_sort_jobs(&items, &all_ns, Some(&by_status_asc))[0].name,
            "db-migrate"
        );

        // Completions numeric sort ("2/5" < "10/10")
        let by_completions_desc = SortState {
            column_id: "completions".to_string(),
            direction: SortDirection::Desc,
        };
        let sorted_comp = filter_and_sort_jobs(&items, &all_ns, Some(&by_completions_desc));
        assert_eq!(sorted_comp[0].name, "nightly-etl");
        assert_eq!(sorted_comp[1].name, "db-migrate");

        // Duration sort ("45s" < "2m 15s" < "1h 30m" < None)
        let by_duration_asc = SortState {
            column_id: "duration".to_string(),
            direction: SortDirection::Asc,
        };
        assert_eq!(
            filter_and_sort_jobs(&items, &all_ns, Some(&by_duration_asc))
                .iter()
                .map(|j| j.name.as_str())
                .collect::<Vec<_>>(),
            vec!["db-migrate", "cache-warm", "nightly-etl", "pending-job"]
        );

        // Images asc
        let by_images_asc = SortState {
            column_id: "images".to_string(),
            direction: SortDirection::Asc,
        };
        assert_eq!(
            filter_and_sort_jobs(&items, &all_ns, Some(&by_images_asc))[0].name,
            "pending-job"
        );

        // Age asc ("5m" < "45m" < "2h" < "10d")
        let by_age_asc = SortState {
            column_id: "age".to_string(),
            direction: SortDirection::Asc,
        };
        assert_eq!(
            filter_and_sort_jobs(&items, &all_ns, Some(&by_age_asc))
                .iter()
                .map(|j| j.name.as_str())
                .collect::<Vec<_>>(),
            vec!["pending-job", "cache-warm", "db-migrate", "nightly-etl"]
        );
    }

    #[test]
    fn test_jobs_empty_message_and_presentation_mode() {
        let mut state = JobsState::default();
        assert_eq!(jobs_empty_message(&state), "No jobs found in this cluster.");
        assert_eq!(jobs_presentation_mode(&state), JobsPresentationMode::Table);

        // Initial error with no cached rows -> ErrorCard
        state.status = JobsLoadStatus::Error;
        state.error_message = Some("forbidden".to_string());
        assert_eq!(
            jobs_presentation_mode(&state),
            JobsPresentationMode::ErrorCard
        );

        // Error with cached rows -> Table (shows inline banner) + filtered empty message
        state.jobs.push(make_job(
            "db-migrate",
            "prod",
            "Succeeded",
            "1/1",
            Some("10s"),
            "busybox:1.36",
            "1m",
        ));
        assert_eq!(
            jobs_empty_message(&state),
            "No jobs found in the selected namespace(s)."
        );
        assert_eq!(jobs_presentation_mode(&state), JobsPresentationMode::Table);
    }

    #[test]
    fn test_filter_events_for_job_matches_job_pods_and_message() {
        let job = make_job(
            "db-migrate",
            "prod",
            "Succeeded",
            "1/1",
            Some("30s"),
            "busybox:1.36",
            "5m",
        );

        let mk_event = |uid: &str, ns: &str, kind: &str, obj: &str, msg: &str| EventInfo {
            uid: uid.to_string(),
            time: "1m".to_string(),
            r#type: "Normal".to_string(),
            reason: "Completed".to_string(),
            object_name: obj.to_string(),
            object_kind: kind.to_string(),
            message: msg.to_string(),
            namespace: ns.to_string(),
            source: "job-controller".to_string(),
            first_seen: String::new(),
            last_seen: String::new(),
            count: 1,
            labels: Default::default(),
            annotations: Default::default(),
        };

        let events = vec![
            mk_event("e1", "prod", "Job", "db-migrate", "Job completed"),
            mk_event("e2", "prod", "Pod", "db-migrate-abc12", "Started container"),
            mk_event("e3", "prod", "Node", "node-1", "Scheduled db-migrate pod"),
            mk_event("e4", "staging", "Job", "db-migrate", "Other namespace"),
            mk_event("e5", "prod", "Job", "other-job", "Unrelated job"),
        ];

        let matched = filter_events_for_job(&events, &job);
        let uids: Vec<&str> = matched.iter().map(|e| e.uid.as_str()).collect();
        assert_eq!(uids, vec!["e1", "e2", "e3"]);
    }

    #[test]
    fn test_build_job_context_menu_entries_structure() {
        let job = make_job(
            "db-migrate",
            "prod",
            "Succeeded",
            "1/1",
            Some("30s"),
            "busybox:1.36",
            "5m",
        );
        let entries = build_job_context_menu_entries(
            &job,
            |_, _| {},
            |_, _| {},
            |_, _| {},
            |_, _| {},
            |_, _| {},
        );
        assert_eq!(entries.len(), 8);

        let ids: Vec<Option<&str>> = entries
            .iter()
            .map(|e| match e {
                ContextMenuEntry::Item(item) => Some(item.id.as_ref()),
                ContextMenuEntry::Separator => None,
            })
            .collect();
        assert_eq!(
            ids,
            vec![
                Some("copy-name"),
                Some("filter-namespace"),
                Some("refresh"),
                None,
                Some("view-details"),
                Some("view-yaml"),
                None,
                Some("delete-job"),
            ]
        );

        match &entries[1] {
            ContextMenuEntry::Item(item) => assert_eq!(item.label.as_ref(), "Filter by \"prod\""),
            ContextMenuEntry::Separator => panic!("Expected filter-namespace item"),
        }
        match &entries[7] {
            ContextMenuEntry::Item(item) => {
                assert_eq!(item.label.as_ref(), "Delete Job");
                assert!(item.danger);
                assert!(item.disabled);
            }
            ContextMenuEntry::Separator => panic!("Expected delete-job item"),
        }
    }
}
