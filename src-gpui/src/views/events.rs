use std::cmp::Ordering;
use std::rc::Rc;

use gpui::{
    AnyElement, App, ClickEvent, FontWeight, MouseDownEvent, Rgba, SharedString, Window, div,
    prelude::*, px,
};
use lucide_icons::Icon;
use orbit_core::kubernetes::models::EventInfo;

use crate::components::data_table::{
    DataTableCallbacks, SortState, TableColumn, render_data_table, sort_rows_stable,
};
use crate::components::{
    ContextMenuEntry, ContextMenuItem, StatusBadgeVariant, render_status_badge_from_str,
    resource_kind_meta_from_str,
};
use crate::icons::render_icon;
use crate::state::{EventTypeFilter, EventsLoadStatus, EventsState, NamespaceFilterState};
use crate::theme::{OrbitTheme, apply_text_step};
use crate::views::{parse_age_seconds, render_action_button};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EventsPresentationMode {
    Table,
    ErrorCard,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EventSummary {
    pub total: usize,
    pub normal: usize,
    pub warning: usize,
    pub other: usize,
}

#[derive(Debug, Clone)]
pub struct EventMetricCard {
    pub title: &'static str,
    pub primary_value: String,
    pub subtitle: String,
    pub accent_color: Rgba,
}

pub fn event_columns() -> Vec<TableColumn> {
    vec![
        TableColumn::new("type", "Type").fixed_width(px(110.0)),
        TableColumn::new("reason", "Reason").flex_width(1.1, px(120.0)),
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
    let mut rows: Vec<&'a EventInfo> = events
        .iter()
        .filter(|ev| namespaces.matches_resource_namespace(Some(&ev.namespace)))
        .filter(|ev| type_filter.matches(&ev.r#type))
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

pub fn compute_event_summary(events: &[&EventInfo]) -> EventSummary {
    let total = events.len();
    let mut normal = 0_usize;
    let mut warning = 0_usize;

    for ev in events {
        if ev.r#type.eq_ignore_ascii_case("normal") {
            normal += 1;
        } else if ev.r#type.eq_ignore_ascii_case("warning") {
            warning += 1;
        }
    }

    let other = total.saturating_sub(normal + warning);

    EventSummary {
        total,
        normal,
        warning,
        other,
    }
}

pub fn event_summary_cards(
    events: &[&EventInfo],
    namespaces: &NamespaceFilterState,
    theme: &OrbitTheme,
) -> [EventMetricCard; 3] {
    let summary = compute_event_summary(events);
    let normal_pct = (summary.normal.min(summary.total) * 100)
        .checked_div(summary.total)
        .unwrap_or(0);
    let warning_pct = (summary.warning.min(summary.total) * 100)
        .checked_div(summary.total)
        .unwrap_or(0);

    let total_subtitle = if summary.other > 0 {
        format!("{} other type", summary.other)
    } else if namespaces.is_all_namespaces() {
        "All namespaces".to_string()
    } else {
        format!("Namespace: {}", namespaces.summary_label())
    };

    [
        EventMetricCard {
            title: "Total Events",
            primary_value: summary.total.to_string(),
            subtitle: total_subtitle,
            accent_color: theme.accent.accent,
        },
        EventMetricCard {
            title: "Normal",
            primary_value: summary.normal.to_string(),
            subtitle: format!("{normal_pct}% of events"),
            accent_color: theme.semantic.success.solid,
        },
        EventMetricCard {
            title: "Warning",
            primary_value: summary.warning.to_string(),
            subtitle: format!("{warning_pct}% of events"),
            accent_color: theme.semantic.warning.solid,
        },
    ]
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

fn render_event_cell(ev: &EventInfo, col: &TableColumn, theme: &OrbitTheme) -> AnyElement {
    match col.id.as_ref() {
        "type" => render_status_badge_from_str(&ev.r#type, StatusBadgeVariant::Inline, theme),
        "reason" => apply_text_step(div(), theme.typography.size_xs)
            .font_weight(FontWeight::MEDIUM)
            .text_color(theme.text.primary)
            .truncate()
            .child(if ev.reason.is_empty() {
                "-".to_string()
            } else {
                ev.reason.clone()
            })
            .into_any_element(),
        "object" => {
            let meta = resource_kind_meta_from_str(&ev.object_kind, theme);
            let label = format_involved_object(&ev.object_kind, &ev.object_name);
            div()
                .flex()
                .items_center()
                .gap_2()
                .min_w(px(0.0))
                .child(render_icon(meta.icon, px(14.0), theme.text.muted))
                .child(
                    apply_text_step(div(), theme.typography.size_xs)
                        .font_weight(FontWeight::MEDIUM)
                        .text_color(theme.text.primary)
                        .truncate()
                        .child(label),
                )
                .into_any_element()
        }
        "namespace" => apply_text_step(div(), theme.typography.size_xs)
            .text_color(if ev.namespace.is_empty() {
                theme.text.muted
            } else {
                theme.text.secondary
            })
            .truncate()
            .child(if ev.namespace.is_empty() {
                "-".to_string()
            } else {
                ev.namespace.clone()
            })
            .into_any_element(),
        "message" => apply_text_step(div(), theme.typography.size_xs)
            .text_color(theme.text.secondary)
            .truncate()
            .child(if ev.message.is_empty() {
                "-".to_string()
            } else {
                ev.message.clone()
            })
            .into_any_element(),
        "count" => apply_text_step(div(), theme.typography.size_xs)
            .font(theme.typography.mono_font(FontWeight::MEDIUM))
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

fn render_summary_cards_row(
    events: &[&EventInfo],
    namespaces: &NamespaceFilterState,
    theme: &OrbitTheme,
) -> AnyElement {
    let cards = event_summary_cards(events, namespaces, theme);
    div()
        .grid()
        .grid_cols(3)
        .gap_3()
        .flex_shrink_0()
        .children(cards.into_iter().map(|card| {
            theme
                .style_card(div())
                .p_3()
                .flex()
                .flex_col()
                .gap_1()
                .child(
                    div()
                        .flex()
                        .items_center()
                        .justify_between()
                        .child(
                            apply_text_step(div(), theme.typography.size_xs)
                                .font_weight(FontWeight::MEDIUM)
                                .text_color(theme.text.muted)
                                .child(card.title),
                        )
                        .child(div().w(px(6.0)).h(px(6.0)).rounded_full().bg(card.accent_color)),
                )
                .child(
                    apply_text_step(div(), theme.typography.size_lg)
                        .font_weight(FontWeight::BOLD)
                        .text_color(theme.text.primary)
                        .child(card.primary_value),
                )
                .child(
                    apply_text_step(div(), theme.typography.size_xs)
                        .text_color(theme.text.secondary)
                        .truncate()
                        .child(card.subtitle),
                )
        }))
        .into_any_element()
}

fn render_type_filter_pills(
    active: EventTypeFilter,
    theme: &OrbitTheme,
    on_type_filter: Rc<dyn Fn(EventTypeFilter, &mut Window, &mut App)>,
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

pub fn build_event_context_menu_entries(
    _event: &EventInfo,
    on_copy_object: impl 'static + Fn(&mut Window, &mut App),
    on_copy_message: impl 'static + Fn(&mut Window, &mut App),
    on_filter_namespace: impl 'static + Fn(&mut Window, &mut App),
    on_refresh: impl 'static + Fn(&mut Window, &mut App),
    on_view_details: impl 'static + Fn(&mut Window, &mut App),
    on_view_yaml: impl 'static + Fn(&mut Window, &mut App),
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
                .disabled(true),
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
) -> AnyElement {
    let columns = event_columns();
    let ns_filtered: Vec<&EventInfo> = state
        .events
        .iter()
        .filter(|ev| namespaces.matches_resource_namespace(Some(&ev.namespace)))
        .collect();
    let total_ns_count = ns_filtered.len();

    let filtered_rows: Rc<Vec<EventInfo>> = Rc::new(
        filter_and_sort_events(
            &state.events,
            namespaces,
            state.type_filter,
            state.table.sort.as_ref(),
        )
        .into_iter()
        .cloned()
        .collect(),
    );
    let visible_count = filtered_rows.len();
    let is_loading = state.status == EventsLoadStatus::Loading;
    let on_refresh_rc: Rc<dyn Fn(&mut Window, &mut App)> = Rc::new(on_refresh);
    let on_type_filter_rc: Rc<dyn Fn(EventTypeFilter, &mut Window, &mut App)> =
        Rc::new(on_type_filter);

    let count_label = if state.type_filter == EventTypeFilter::All {
        format!("{total_ns_count} events")
    } else {
        format!("{visible_count} of {total_ns_count} events")
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
        .child(
            div()
                .flex()
                .items_center()
                .gap_2()
                .child(render_type_filter_pills(
                    state.type_filter,
                    theme,
                    on_type_filter_rc,
                )),
        );

    let summary_row = render_summary_cards_row(&ns_filtered, namespaces, theme);

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
                            .child(format!("Failed to load events: {message}")),
                    ),
            )
            .child(render_action_button(
                "events-retry-btn",
                Icon::RotateCcw,
                "Retry",
                is_loading,
                theme,
                on_refresh_rc.clone(),
            ))
    });

    let body = match events_presentation_mode(state) {
        EventsPresentationMode::ErrorCard => {
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
                                .child("Unable to Load Events"),
                        ),
                )
                .child(
                    apply_text_step(div(), theme.typography.size_xs)
                        .text_color(theme.text.secondary)
                        .child(message),
                )
                .child(render_action_button(
                    "events-retry-btn",
                    Icon::RotateCcw,
                    "Retry",
                    is_loading,
                    theme,
                    on_refresh_rc,
                ))
                .into_any_element()
        }
        EventsPresentationMode::Table => {
            let mut table_state = state.table.clone();
            table_state.loading = is_loading && state.events.is_empty();
            table_state.empty_message = SharedString::from(events_empty_message(state, namespaces));

            let key_rows = filtered_rows.clone();
            let cell_rows = filtered_rows.clone();
            let ctx_rows = filtered_rows.clone();

            div()
                .flex_1()
                .min_h(px(0.0))
                .flex()
                .flex_col()
                .child(render_data_table(
                    "events-table",
                    &columns,
                    filtered_rows.len(),
                    &table_state,
                    theme,
                    DataTableCallbacks {
                        row_key: move |idx| {
                            let ev = &key_rows[idx];
                            if !ev.uid.is_empty() {
                                ev.uid.clone()
                            } else {
                                format!("{}/{}/{}", ev.namespace, ev.reason, ev.object_name)
                            }
                        },
                        render_cell: move |row_idx, col, theme| {
                            render_event_cell(&cell_rows[row_idx], col, theme)
                        },
                        on_sort: Some(Rc::new(on_sort)),
                        on_row_click: Some(Rc::new(on_row_click)),
                        on_row_context_menu: Some(Rc::new(
                            move |idx, row_key, event, window, cx| {
                                if let Some(ev) = ctx_rows.get(idx) {
                                    on_row_context_menu(row_key, ev, event, window, cx);
                                }
                            },
                        )),
                    },
                ))
                .into_any_element()
        }
    };

    div()
        .flex_1()
        .min_h(px(0.0))
        .flex()
        .flex_col()
        .gap_3()
        .child(summary_row)
        .child(toolbar)
        .when_some(error_banner, |el, banner| el.child(banner))
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
        .gap_5()
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
    use gpui::WindowAppearance;
    use orbit_core::config::OrbitConfig;

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
    fn test_compute_event_summary_counts_normal_warning_and_other() {
        let events = vec![
            sample_event("1", "Normal", "Scheduled", "Pod", "web-1", "default", 1, "1m"),
            sample_event("2", "Normal", "Pulled", "Pod", "web-2", "default", 1, "2m"),
            sample_event("3", "Warning", "BackOff", "Pod", "api-1", "default", 4, "3m"),
            sample_event("4", "Custom", "Info", "Node", "node-1", "default", 1, "4m"),
        ];
        let refs: Vec<&EventInfo> = events.iter().collect();

        let summary = compute_event_summary(&refs);
        assert_eq!(
            summary,
            EventSummary {
                total: 4,
                normal: 2,
                warning: 1,
                other: 1,
            }
        );

        let theme = OrbitTheme::from_config(&OrbitConfig::default(), WindowAppearance::Dark);
        let cards = event_summary_cards(&refs, &NamespaceFilterState::default(), &theme);
        assert_eq!(cards[0].title, "Total Events");
        assert_eq!(cards[0].primary_value, "4");
        assert_eq!(cards[1].title, "Normal");
        assert_eq!(cards[1].primary_value, "2");
        assert_eq!(cards[1].subtitle, "50% of events");
        assert_eq!(cards[2].title, "Warning");
        assert_eq!(cards[2].primary_value, "1");
        assert_eq!(cards[2].subtitle, "25% of events");
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
}
