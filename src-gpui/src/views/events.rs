use std::cmp::Ordering;
use std::rc::Rc;

use gpui::{
    AnyElement, App, ClickEvent, ElementId, FontWeight, MouseDownEvent, SharedString, Window, div,
    prelude::*, px,
};
use lucide_icons::Icon;
use orbit_core::kubernetes::models::EventInfo;

use crate::components::data_table::{
    DataTableCallbacks, DataTableEnhancementCallbacks, SortState, TableColumn,
    TableFilterChipGroup, TableFilterChipOption, TableFilterState, render_enhanced_data_table,
    sort_rows_stable,
};
use crate::components::{
    ContextMenuEntry, ContextMenuItem, StatusBadgeVariant, TextTooltip,
    render_status_badge_from_str, resource_kind_meta_from_str,
};
use crate::icons::render_icon;
use crate::state::{EventTypeFilter, EventsLoadStatus, EventsState, NamespaceFilterState};
use crate::theme::{OrbitTheme, apply_text_step};
use crate::views::{
    parse_age_seconds, render_resource_error_banner, render_resource_error_state,
    render_resource_view_header, render_truncated_cell, table_numeric_font,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EventsPresentationMode {
    Table,
    ErrorCard,
}

pub fn event_columns() -> Vec<TableColumn> {
    vec![
        TableColumn::new("type", "Type")
            .fixed_width(px(110.0))
            .locked(true),
        TableColumn::new("reason", "Reason")
            .flex_width(1.1, px(120.0))
            .locked(true),
        TableColumn::new("object", "Object").flex_width(1.8, px(180.0)),
        TableColumn::new("namespace", "Namespace").flex_width(1.0, px(105.0)),
        TableColumn::new("message", "Message").flex_width(2.6, px(220.0)),
        TableColumn::new("count", "Count")
            .fixed_width(px(76.0))
            .right_numeric(),
        TableColumn::new("last_seen", "Last Seen")
            .fixed_width(px(92.0))
            .right_numeric(),
    ]
}

pub fn format_involved_object(kind: &str, name: &str) -> String {
    let kind = kind.trim();
    let name = name.trim();
    match (kind.is_empty(), name.is_empty()) {
        (false, false) => format!("{kind}/{name}"),
        (true, false) => name.to_string(),
        (false, true) => kind.to_string(),
        (true, true) => "-".to_string(),
    }
}

fn event_tie_break(a: &EventInfo, b: &EventInfo) -> Ordering {
    a.uid
        .cmp(&b.uid)
        .then_with(|| {
            a.reason
                .to_ascii_lowercase()
                .cmp(&b.reason.to_ascii_lowercase())
        })
        .then_with(|| {
            a.object_name
                .to_ascii_lowercase()
                .cmp(&b.object_name.to_ascii_lowercase())
        })
}

pub fn compare_events_by_col(a: &EventInfo, b: &EventInfo, col_id: &str) -> Ordering {
    match col_id {
        "type" => a
            .r#type
            .to_ascii_lowercase()
            .cmp(&b.r#type.to_ascii_lowercase())
            .then_with(|| event_tie_break(a, b)),
        "reason" => a
            .reason
            .to_ascii_lowercase()
            .cmp(&b.reason.to_ascii_lowercase())
            .then_with(|| event_tie_break(a, b)),
        "object" => format_involved_object(&a.object_kind, &a.object_name)
            .to_ascii_lowercase()
            .cmp(&format_involved_object(&b.object_kind, &b.object_name).to_ascii_lowercase())
            .then_with(|| event_tie_break(a, b)),
        "namespace" => a
            .namespace
            .to_ascii_lowercase()
            .cmp(&b.namespace.to_ascii_lowercase())
            .then_with(|| event_tie_break(a, b)),
        "message" => a
            .message
            .to_ascii_lowercase()
            .cmp(&b.message.to_ascii_lowercase())
            .then_with(|| event_tie_break(a, b)),
        "count" => a.count.cmp(&b.count).then_with(|| event_tie_break(a, b)),
        "last_seen" => parse_age_seconds(&a.time)
            .cmp(&parse_age_seconds(&b.time))
            .then_with(|| event_tie_break(a, b)),
        _ => event_tie_break(a, b),
    }
}

pub fn filter_and_sort_events<'a>(
    events: &'a [EventInfo],
    namespaces: &NamespaceFilterState,
    type_filter: EventTypeFilter,
    sort: Option<&SortState>,
) -> Vec<&'a EventInfo> {
    filter_and_sort_events_with_filters(
        events,
        namespaces,
        type_filter,
        &TableFilterState::default(),
        sort,
    )
}

pub fn filter_and_sort_events_with_filters<'a>(
    events: &'a [EventInfo],
    namespaces: &NamespaceFilterState,
    type_filter: EventTypeFilter,
    filters: &TableFilterState,
    sort: Option<&SortState>,
) -> Vec<&'a EventInfo> {
    let query = filters.search_query.trim().to_ascii_lowercase();
    let extra_type = filters
        .extra_filters
        .get("type")
        .map(|s| s.trim().to_ascii_lowercase())
        .filter(|s| !s.is_empty() && s != "all");

    let mut rows: Vec<&'a EventInfo> = events
        .iter()
        .filter(|ev| namespaces.matches_resource_namespace(Some(&ev.namespace)))
        .filter(|ev| {
            type_filter.matches(&ev.r#type)
                && extra_type
                    .as_deref()
                    .is_none_or(|t| ev.r#type.to_ascii_lowercase() == t)
        })
        .filter(|ev| {
            if query.is_empty() {
                return true;
            }
            ev.reason.to_ascii_lowercase().contains(&query)
                || ev.object_kind.to_ascii_lowercase().contains(&query)
                || ev.object_name.to_ascii_lowercase().contains(&query)
                || ev.namespace.to_ascii_lowercase().contains(&query)
                || ev.message.to_ascii_lowercase().contains(&query)
        })
        .collect();

    if sort.is_some() {
        sort_rows_stable(&mut rows, sort, |a, b, col_id| {
            compare_events_by_col(a, b, col_id)
        });
    } else {
        rows.sort_by(|a, b| {
            parse_age_seconds(&a.time)
                .cmp(&parse_age_seconds(&b.time))
                .then_with(|| event_tie_break(a, b))
        });
    }

    rows
}

pub fn events_empty_message(state: &EventsState, _namespaces: &NamespaceFilterState) -> &'static str {
    if state.events.is_empty() {
        "No events found in this cluster."
    } else if state.type_filter != EventTypeFilter::All {
        "No events match the active namespace and type filters."
    } else {
        "No events found in the selected namespace(s)."
    }
}

pub fn events_presentation_mode(state: &EventsState) -> EventsPresentationMode {
    if state.status == EventsLoadStatus::Error && state.events.is_empty() {
        EventsPresentationMode::ErrorCard
    } else {
        EventsPresentationMode::Table
    }
}

fn display_last_seen(ev: &EventInfo) -> String {
    let trimmed = ev.time.trim();
    if !trimmed.is_empty() && !trimmed.eq_ignore_ascii_case("unknown") {
        trimmed.to_string()
    } else if !ev.last_seen.trim().is_empty() {
        ev.last_seen.clone()
    } else {
        "Unknown".to_string()
    }
}

fn render_event_cell(
    row_idx: usize,
    ev: &EventInfo,
    col: &TableColumn,
    theme: &OrbitTheme,
) -> AnyElement {
    match col.id.as_ref() {
        "type" => render_status_badge_from_str(&ev.r#type, StatusBadgeVariant::Inline, theme),
        "reason" => render_truncated_cell(
            ElementId::NamedInteger(SharedString::from("ev-reason"), row_idx as u64),
            if ev.reason.is_empty() {
                "-".to_string()
            } else {
                ev.reason.clone()
            },
            None,
            theme.text.primary,
            FontWeight::MEDIUM,
            false,
            theme,
        ),
        "object" => {
            let meta = resource_kind_meta_from_str(&ev.object_kind, theme);
            let label = format_involved_object(&ev.object_kind, &ev.object_name);
            let tip = SharedString::from(label.clone());
            div()
                .id(ElementId::NamedInteger(
                    SharedString::from("ev-obj"),
                    row_idx as u64,
                ))
                .flex()
                .items_center()
                .gap_2()
                .min_w(px(0.0))
                .flex_1()
                .overflow_hidden()
                .tooltip(move |_window, cx| cx.new(|_| TextTooltip::new(tip.clone())).into())
                .child(render_icon(meta.icon, px(14.0), theme.text.muted))
                .child(
                    apply_text_step(div(), theme.typography.size_xs)
                        .min_w(px(0.0))
                        .flex_1()
                        .font_weight(FontWeight::NORMAL)
                        .text_color(theme.text.primary)
                        .truncate()
                        .child(label),
                )
                .into_any_element()
        }
        "namespace" => render_truncated_cell(
            ElementId::NamedInteger(SharedString::from("ev-ns"), row_idx as u64),
            if ev.namespace.is_empty() {
                "-".to_string()
            } else {
                ev.namespace.clone()
            },
            None,
            if ev.namespace.is_empty() {
                theme.text.muted
            } else {
                theme.text.secondary
            },
            FontWeight::NORMAL,
            false,
            theme,
        ),
        "message" => {
            let text = if ev.message.is_empty() {
                "-".to_string()
            } else {
                ev.message.clone()
            };
            let tip = (!ev.message.is_empty()).then(|| SharedString::from(ev.message.clone()));
            render_truncated_cell(
                ElementId::NamedInteger(SharedString::from("ev-msg"), row_idx as u64),
                text,
                tip,
                theme.text.secondary,
                FontWeight::NORMAL,
                false,
                theme,
            )
        }
        "count" => apply_text_step(div(), theme.typography.size_xs)
            .font(table_numeric_font(theme))
            .text_color(theme.text.primary)
            .child(ev.count.to_string())
            .into_any_element(),
        "last_seen" => apply_text_step(div(), theme.typography.size_xs)
            .font(theme.typography.mono_font(FontWeight::NORMAL))
            .text_color(theme.text.muted)
            .child(display_last_seen(ev))
            .into_any_element(),
        _ => div().into_any_element(),
    }
}

fn render_type_filter_pills(
    active: EventTypeFilter,
    theme: &OrbitTheme,
    on_type_filter: super::ValueCallback<EventTypeFilter>,
) -> AnyElement {
    let options = [
        ("events-filter-all", "All", EventTypeFilter::All),
        ("events-filter-normal", "Normal", EventTypeFilter::Normal),
        ("events-filter-warning", "Warning", EventTypeFilter::Warning),
    ];

    div()
        .flex()
        .items_center()
        .gap_1()
        .p_0p5()
        .rounded(theme.style.radius_control)
        .border_1()
        .border_color(theme.border.default)
        .bg(theme.surface.bg_panel)
        .children(options.into_iter().map(|(id, label, filter)| {
            let is_active = active == filter;
            let on_filter = on_type_filter.clone();
            let hover_bg = theme.surface.control_hover;
            div()
                .id(id)
                .h(px(24.0))
                .px_2p5()
                .rounded(theme.style.radius_control)
                .flex()
                .items_center()
                .cursor_pointer()
                .when(is_active, |el| {
                    el.bg(theme.accent.soft).text_color(theme.accent.accent)
                })
                .when(!is_active, |el| {
                    el.text_color(theme.text.secondary)
                        .hover(move |style| style.bg(hover_bg))
                })
                .on_click(move |_, window, cx| on_filter(filter, window, cx))
                .child(
                    apply_text_step(div(), theme.typography.size_xs)
                        .font_weight(if is_active {
                            FontWeight::SEMIBOLD
                        } else {
                            FontWeight::MEDIUM
                        })
                        .child(label),
                )
        }))
        .into_any_element()
}

#[allow(clippy::too_many_arguments)]
pub fn build_event_context_menu_entries(
    _event: &EventInfo,
    on_copy_object: impl 'static + Fn(&mut Window, &mut App),
    on_copy_message: impl 'static + Fn(&mut Window, &mut App),
    on_filter_namespace: impl 'static + Fn(&mut Window, &mut App),
    on_refresh: impl 'static + Fn(&mut Window, &mut App),
    on_view_details: impl 'static + Fn(&mut Window, &mut App),
    on_view_yaml: impl 'static + Fn(&mut Window, &mut App),
    on_delete: impl 'static + Fn(&mut Window, &mut App),
) -> Vec<ContextMenuEntry> {
    vec![
        ContextMenuEntry::Item(
            ContextMenuItem::new("copy-object", "Copy Object")
                .icon(Icon::Copy)
                .on_select(on_copy_object),
        ),
        ContextMenuEntry::Item(
            ContextMenuItem::new("copy-message", "Copy Message")
                .icon(Icon::FileText)
                .on_select(on_copy_message),
        ),
        ContextMenuEntry::Item(
            ContextMenuItem::new("filter-namespace", "Filter by Namespace")
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
            ContextMenuItem::new("delete-event", "Delete Event")
                .icon(Icon::Trash2)
                .danger(true)
                .on_select(on_delete),
        ),
    ]
}

#[allow(clippy::too_many_arguments)]
pub fn render_events_view(
    state: &EventsState,
    namespaces: &NamespaceFilterState,
    theme: &OrbitTheme,
    on_sort: impl 'static + Fn(SharedString, &ClickEvent, &mut Window, &mut App),
    on_row_click: impl 'static + Fn(usize, SharedString, &ClickEvent, &mut Window, &mut App),
    on_row_context_menu: impl 'static
        + Fn(SharedString, &EventInfo, &MouseDownEvent, &mut Window, &mut App),
    on_type_filter: impl 'static + Fn(EventTypeFilter, &mut Window, &mut App),
    on_refresh: impl 'static + Fn(&mut Window, &mut App),
    enhancements: Option<DataTableEnhancementCallbacks>,
) -> AnyElement {
    let columns = event_columns();
    let ns_filtered: Vec<&EventInfo> = state
        .events
        .iter()
        .filter(|ev| namespaces.matches_resource_namespace(Some(&ev.namespace)))
        .collect();
    let total_ns_count = ns_filtered.len();

    let cache_key = crate::state::RowCacheKey::from_table(
        state.data_revision,
        namespaces.selection_revision,
        &state.table,
        state.type_filter as u64,
    );
    let filtered_rows = state.row_cache.get_or_compute(cache_key, || {
        filter_and_sort_events_with_filters(
            &state.events,
            namespaces,
            state.type_filter,
            &state.table.filters,
            state.table.sort.as_ref(),
        )
        .into_iter()
        .cloned()
        .collect()
    });
    let visible_count = filtered_rows.len();
    let is_loading = state.status == EventsLoadStatus::Loading;
    let on_refresh_rc: super::ActionCallback = Rc::new(on_refresh);
    let on_type_filter_rc: super::ValueCallback<EventTypeFilter> = Rc::new(on_type_filter);

    let count_label = if state.type_filter == EventTypeFilter::All {
        format!("{total_ns_count} events")
    } else {
        format!("{visible_count} of {total_ns_count} events")
    };

    let header = render_resource_view_header(
        count_label,
        Some(namespaces),
        if enhancements.is_some() {
            None
        } else {
            Some(render_type_filter_pills(
                state.type_filter,
                theme,
                on_type_filter_rc,
            ))
        },
        theme,
    );

    let filter_groups = [TableFilterChipGroup {
        key: "type",
        default_value: "All",
        options: vec![
            TableFilterChipOption { value: "All", label: "All" },
            TableFilterChipOption { value: "Normal", label: "Normal" },
            TableFilterChipOption { value: "Warning", label: "Warning" },
        ],
    }];

    let error_banner = state.error_message.as_ref().map(|message| {
        render_resource_error_banner(
            "events-retry-btn",
            format!("Failed to load events: {message}"),
            is_loading,
            theme,
            on_refresh_rc.clone(),
        )
    });

    let body = match events_presentation_mode(state) {
        EventsPresentationMode::ErrorCard => {
            let message = state
                .error_message
                .clone()
                .unwrap_or_else(|| "Unknown error".to_string());
            render_resource_error_state(
                "events-retry-btn",
                "Unable to Load Events",
                message,
                is_loading,
                theme,
                on_refresh_rc,
            )
        }
        EventsPresentationMode::Table => {
            let mut table_state = state.table.clone();
            table_state.loading = is_loading && state.events.is_empty();
            table_state.refreshing = is_loading && !state.events.is_empty();
            table_state.empty_message = SharedString::from(events_empty_message(state, namespaces));

            let key_rows = filtered_rows.clone();
            let cell_rows = filtered_rows.clone();
            let ctx_rows = filtered_rows.clone();

            let has_table_filters = table_state.has_active_filters(&[("type", "All")]);
            let empty_action = crate::views::build_empty_state_action(
                has_table_filters,
                enhancements
                    .as_ref()
                    .map(|e| e.toolbar.on_clear_filters.clone()),
                on_refresh_rc,
            );
            let callbacks = DataTableCallbacks::new(
                move |idx| {
                    let ev = &key_rows[idx];
                    if !ev.uid.is_empty() {
                        ev.uid.clone()
                    } else {
                        format!("{}/{}/{}", ev.namespace, ev.reason, ev.object_name)
                    }
                },
                move |row_idx, col, theme| {
                    render_event_cell(row_idx, &cell_rows[row_idx], col, theme)
                },
            )
            .on_sort(on_sort)
            .on_row_click(on_row_click)
            .on_empty_action(empty_action)
            .on_row_context_menu(move |idx, row_key, event, window, cx| {
                if let Some(ev) = ctx_rows.get(idx) {
                    on_row_context_menu(row_key, ev, event, window, cx);
                }
            });

            render_enhanced_data_table(
                "events-table",
                "Filter events...",
                &filter_groups,
                &columns,
                filtered_rows.len(),
                &table_state,
                theme,
                callbacks,
                enhancements,
            )
        }
    };

    div()
        .flex_1()
        .min_h(px(0.0))
        .flex()
        .flex_col()
        .gap_0()
        .child(header)
        .when(
            events_presentation_mode(state) == EventsPresentationMode::Table,
            |el| el.when_some(error_banner, |el, banner| el.child(banner)),
        )
        .child(body)
        .into_any_element()
}

pub fn render_event_drawer_overview(event: &EventInfo, theme: &OrbitTheme) -> AnyElement {
    let summary = crate::views::render_drawer_key_values(
        vec![
            ("Type", event.r#type.clone(), false),
            ("Reason", event.reason.clone(), false),
            (
                "Involved Object",
                format!("{}/{}", event.object_kind, event.object_name),
                true,
            ),
            (
                "Namespace",
                if event.namespace.is_empty() {
                    "-".to_string()
                } else {
                    event.namespace.clone()
                },
                true,
            ),
            ("Source", event.source.clone(), true),
            ("Count", event.count.to_string(), true),
            ("First Seen", event.first_seen.clone(), true),
            ("Last Seen", event.last_seen.clone(), true),
            ("UID", event.uid.clone(), true),
        ],
        theme,
    );

    let message_box = crate::views::render_drawer_section(
        "MESSAGE",
        div()
            .w_full()
            .p_3()
            .rounded(theme.style.radius_control)
            .bg(theme.surface.bg_panel)
            .border_1()
            .border_color(theme.border.default)
            .child(
                apply_text_step(div(), theme.typography.size_xs)
                    .text_color(theme.text.primary)
                    .child(event.message.clone()),
            )
            .into_any_element(),
        theme,
    );

    div()
        .id("event-drawer-overview")
        .flex_1()
        .min_h_0()
        .overflow_y_scroll()
        .p_4()
        .flex()
        .flex_col()
        .gap_4()
        .child(message_box)
        .child(crate::views::render_drawer_section("SUMMARY", summary, theme))
        .child(crate::views::render_drawer_kv_badges(
            "LABELS",
            &event.labels,
            "No labels",
            theme,
        ))
        .child(crate::views::render_drawer_kv_badges(
            "ANNOTATIONS",
            &event.annotations,
            "No annotations",
            theme,
        ))
        .into_any_element()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::components::data_table::{ColumnAlign, SortDirection};

    #[allow(clippy::too_many_arguments)]
    fn sample_event(
        uid: &str,
        event_type: &str,
        reason: &str,
        kind: &str,
        object_name: &str,
        namespace: &str,
        count: i32,
        time: &str,
    ) -> EventInfo {
        EventInfo {
            uid: uid.to_string(),
            name: String::new(),
            time: time.to_string(),
            r#type: event_type.to_string(),
            reason: reason.to_string(),
            object_name: object_name.to_string(),
            object_kind: kind.to_string(),
            message: format!("{reason} on {object_name}"),
            namespace: namespace.to_string(),
            source: "kubelet".to_string(),
            first_seen: "Nov 15, 2026, 07:00 PM".to_string(),
            last_seen: "Nov 15, 2026, 07:05 PM".to_string(),
            count,
            labels: Default::default(),
            annotations: Default::default(),
        }
    }

    #[test]
    fn test_event_columns_contract_and_alignment() {
        let cols = event_columns();
        let ids: Vec<&str> = cols.iter().map(|c| c.id.as_ref()).collect();
        assert_eq!(
            ids,
            vec![
                "type",
                "reason",
                "object",
                "namespace",
                "message",
                "count",
                "last_seen"
            ]
        );
        assert_eq!(cols[0].align, ColumnAlign::Left);
        assert_eq!(cols[5].align, ColumnAlign::Right);
        assert_eq!(cols[6].align, ColumnAlign::Right);
    }

    #[test]
    fn test_format_involved_object_formats_kind_and_name() {
        assert_eq!(format_involved_object("Pod", "api-123"), "Pod/api-123");
        assert_eq!(format_involved_object("", "api-123"), "api-123");
        assert_eq!(format_involved_object("Node", ""), "Node");
        assert_eq!(format_involved_object("", ""), "-");
    }

    #[test]
    fn test_filter_and_sort_events_respects_namespace_and_type_filter() {
        let events = vec![
            sample_event("1", "Normal", "Scheduled", "Pod", "web-1", "default", 1, "10m"),
            sample_event("2", "Warning", "BackOff", "Pod", "api-1", "prod", 5, "30s"),
            sample_event("3", "Normal", "Pulled", "Pod", "worker-1", "prod", 2, "2m"),
        ];

        let mut ns = NamespaceFilterState::default();
        ns.begin_load_for_cluster("ctx-a");
        ns.toggle_namespace("prod");

        // Default ordering: most recent first (30s before 2m).
        let all_prod = filter_and_sort_events(&events, &ns, EventTypeFilter::All, None);
        assert_eq!(
            all_prod.iter().map(|e| e.uid.as_str()).collect::<Vec<_>>(),
            vec!["2", "3"]
        );

        // Type filter: Warning only.
        let warning_prod = filter_and_sort_events(&events, &ns, EventTypeFilter::Warning, None);
        assert_eq!(
            warning_prod
                .iter()
                .map(|e| e.uid.as_str())
                .collect::<Vec<_>>(),
            vec!["2"]
        );

        // Type filter: Normal only.
        let normal_prod = filter_and_sort_events(&events, &ns, EventTypeFilter::Normal, None);
        assert_eq!(
            normal_prod
                .iter()
                .map(|e| e.uid.as_str())
                .collect::<Vec<_>>(),
            vec!["3"]
        );

        // Explicit sort by count descending across all namespaces.
        let all_ns = NamespaceFilterState::default();
        let sort_count_desc = SortState {
            column_id: "count".to_string(),
            direction: SortDirection::Desc,
        };
        let sorted_count =
            filter_and_sort_events(&events, &all_ns, EventTypeFilter::All, Some(&sort_count_desc));
        assert_eq!(
            sorted_count
                .iter()
                .map(|e| e.uid.as_str())
                .collect::<Vec<_>>(),
            vec!["2", "3", "1"]
        );

        // Explicit sort by reason ascending.
        let sort_reason_asc = SortState {
            column_id: "reason".to_string(),
            direction: SortDirection::Asc,
        };
        let sorted_reason =
            filter_and_sort_events(&events, &all_ns, EventTypeFilter::All, Some(&sort_reason_asc));
        assert_eq!(
            sorted_reason
                .iter()
                .map(|e| e.reason.as_str())
                .collect::<Vec<_>>(),
            vec!["BackOff", "Pulled", "Scheduled"]
        );
    }

    #[test]
    fn test_events_presentation_mode_and_empty_messages() {
        let ns = NamespaceFilterState::default();
        let mut state = EventsState {
            status: EventsLoadStatus::Error,
            error_message: Some("RBAC forbidden".to_string()),
            ..Default::default()
        };
        assert_eq!(
            events_presentation_mode(&state),
            EventsPresentationMode::ErrorCard
        );
        assert_eq!(
            events_empty_message(&state, &ns),
            "No events found in this cluster."
        );

        state.events.push(sample_event(
            "1",
            "Normal",
            "Scheduled",
            "Pod",
            "web-1",
            "default",
            1,
            "1m",
        ));
        assert_eq!(
            events_presentation_mode(&state),
            EventsPresentationMode::Table
        );
        assert_eq!(
            events_empty_message(&state, &ns),
            "No events found in the selected namespace(s)."
        );

        state.set_type_filter(EventTypeFilter::Warning);
        assert_eq!(
            events_empty_message(&state, &ns),
            "No events match the active namespace and type filters."
        );
    }

    #[test]
    fn test_build_event_context_menu_entries_enables_delete() {
        let ev = sample_event(
            "u1",
            "Normal",
            "Scheduled",
            "Pod",
            "web-1",
            "default",
            1,
            "1m",
        );
        let entries = build_event_context_menu_entries(
            &ev,
            |_, _| {},
            |_, _| {},
            |_, _| {},
            |_, _| {},
            |_, _| {},
            |_, _| {},
            |_, _| {},
        );
        let delete_item = entries
            .iter()
            .find_map(|e| match e {
                ContextMenuEntry::Item(item) if item.id.as_ref() == "delete-event" => Some(item),
                _ => None,
            })
            .expect("delete-event item");
        assert!(delete_item.danger);
        assert!(!delete_item.disabled);
        assert!(delete_item.on_select.is_some());
    }

    #[test]
    fn test_filter_and_sort_events_applies_search_query_and_type_extra_filter() {
        let cols = event_columns();
        let type_col = cols.iter().find(|c| c.id.as_ref() == "type").expect("type col");
        assert!(type_col.locked);

        let events = vec![
            sample_event("1", "Normal", "Scheduled", "Pod", "api-0", "default", 1, "1m"),
            sample_event("2", "Warning", "BackOff", "Pod", "api-0", "default", 3, "2m"),
            sample_event("3", "Warning", "FailedMount", "Pod", "db-0", "default", 2, "3m"),
        ];
        let ns = NamespaceFilterState::default();
        let mut filters = crate::components::TableFilterState {
            search_query: "api".to_string(),
            ..Default::default()
        };
        filters
            .extra_filters
            .insert("type".to_string(), "Warning".to_string());

        let matched = filter_and_sort_events_with_filters(
            &events,
            &ns,
            EventTypeFilter::All,
            &filters,
            None,
        );
        assert_eq!(matched.len(), 1);
        assert_eq!(matched[0].uid, "2");
    }
}
