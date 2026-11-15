use std::cmp::Ordering;
use std::collections::BTreeMap;
use std::rc::Rc;

use gpui::{AnyElement, App, ClickEvent, FontWeight, Rgba, SharedString, Window, div, prelude::*, px};
use lucide_icons::Icon;
use orbit_core::kubernetes::models::NamespaceInfo;

use crate::components::data_table::{
    DataTableCallbacks, SortState, TableColumn, render_data_table, sort_rows_stable,
};
use crate::components::{
    SemanticStatus, StatusBadgeVariant, render_status_badge, render_status_badge_from_str,
    resource_kind_meta_from_str,
};
use crate::icons::render_icon;
use crate::state::{NamespaceFilterState, NamespaceLoadStatus, NamespacesViewState};
use crate::theme::{OrbitTheme, apply_text_step};
use crate::views::{parse_age_seconds, render_action_button};

pub const MAX_VISIBLE_LABELS: usize = 2;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NamespacesPresentationMode {
    Table,
    ErrorCard,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NamespaceSummary {
    pub total: usize,
    pub active: usize,
    pub terminating: usize,
    pub system: usize,
    pub user: usize,
}

#[derive(Debug, Clone)]
pub struct NamespaceMetricCard {
    pub title: &'static str,
    pub primary_value: String,
    pub subtitle: String,
    pub accent_color: Rgba,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LabelsSummary {
    pub visible: Vec<String>,
    pub extra: usize,
}

pub fn format_labels_summary(labels: &BTreeMap<String, String>) -> LabelsSummary {
    let visible = labels
        .iter()
        .take(MAX_VISIBLE_LABELS)
        .map(|(k, v)| format!("{k}={v}"))
        .collect();
    let extra = labels.len().saturating_sub(MAX_VISIBLE_LABELS);
    LabelsSummary { visible, extra }
}

pub fn namespace_columns() -> Vec<TableColumn> {
    vec![
        TableColumn::new("name", "Name").flex_width(1.8, px(170.0)),
        TableColumn::new("status", "Status").fixed_width(px(130.0)),
        TableColumn::new("system", "System").fixed_width(px(110.0)),
        TableColumn::new("labels", "Labels").flex_width(2.0, px(200.0)),
        TableColumn::new("age", "Age")
            .fixed_width(px(80.0))
            .right_numeric()
            .monospace(true),
    ]
}

fn compare_namespaces_by_col(a: &NamespaceInfo, b: &NamespaceInfo, col_id: &str) -> Ordering {
    let tie_break = || a.name.to_ascii_lowercase().cmp(&b.name.to_ascii_lowercase());

    match col_id {
        "name" => tie_break(),
        "status" => a
            .status
            .to_ascii_lowercase()
            .cmp(&b.status.to_ascii_lowercase())
            .then_with(tie_break),
        "system" => a.is_system.cmp(&b.is_system).then_with(tie_break),
        "labels" => a
            .labels
            .len()
            .cmp(&b.labels.len())
            .then_with(|| a.labels.keys().next().cmp(&b.labels.keys().next()))
            .then_with(tie_break),
        "age" => parse_age_seconds(&a.age)
            .cmp(&parse_age_seconds(&b.age))
            .then_with(tie_break),
        _ => tie_break(),
    }
}

pub fn sort_namespaces<'a>(
    namespaces: &'a [NamespaceInfo],
    sort: Option<&SortState>,
) -> Vec<&'a NamespaceInfo> {
    let mut rows: Vec<&'a NamespaceInfo> = namespaces.iter().collect();
    if sort.is_some() {
        sort_rows_stable(&mut rows, sort, |a, b, col_id| {
            compare_namespaces_by_col(a, b, col_id)
        });
    } else {
        rows.sort_by_key(|a| a.name.to_ascii_lowercase());
    }
    rows
}

pub fn compute_namespace_summary(namespaces: &[NamespaceInfo]) -> NamespaceSummary {
    let total = namespaces.len();
    let mut active = 0_usize;
    let mut terminating = 0_usize;
    let mut system = 0_usize;

    for ns in namespaces {
        if ns.status.eq_ignore_ascii_case("active") {
            active += 1;
        } else if ns.status.eq_ignore_ascii_case("terminating") {
            terminating += 1;
        }
        if ns.is_system {
            system += 1;
        }
    }

    let user = total.saturating_sub(system);

    NamespaceSummary {
        total,
        active,
        terminating,
        system,
        user,
    }
}

pub fn namespace_summary_cards(
    namespaces: &[NamespaceInfo],
    theme: &OrbitTheme,
) -> [NamespaceMetricCard; 3] {
    let summary = compute_namespace_summary(namespaces);
    let active_pct = (summary.active.min(summary.total) * 100)
        .checked_div(summary.total)
        .unwrap_or(0);
    let system_pct = (summary.system.min(summary.total) * 100)
        .checked_div(summary.total)
        .unwrap_or(0);

    let total_subtitle = if summary.terminating > 0 {
        format!("{} user • {} terminating", summary.user, summary.terminating)
    } else {
        format!("{} user namespaces", summary.user)
    };

    let active_subtitle = if summary.terminating > 0 {
        format!("{active_pct}% active • {} terminating", summary.terminating)
    } else {
        format!("{active_pct}% active")
    };

    [
        NamespaceMetricCard {
            title: "Total Namespaces",
            primary_value: summary.total.to_string(),
            subtitle: total_subtitle,
            accent_color: theme.accent.accent,
        },
        NamespaceMetricCard {
            title: "Active",
            primary_value: summary.active.to_string(),
            subtitle: active_subtitle,
            accent_color: theme.semantic.success.solid,
        },
        NamespaceMetricCard {
            title: "System",
            primary_value: summary.system.to_string(),
            subtitle: format!("{system_pct}% system • {} user", summary.user),
            accent_color: theme.semantic.info.solid,
        },
    ]
}

pub fn namespaces_empty_message(_state: &NamespaceFilterState) -> &'static str {
    "No namespaces found in this cluster."
}

pub fn namespaces_presentation_mode(state: &NamespaceFilterState) -> NamespacesPresentationMode {
    if state.status == NamespaceLoadStatus::Error && state.namespaces.is_empty() {
        NamespacesPresentationMode::ErrorCard
    } else {
        NamespacesPresentationMode::Table
    }
}

fn render_label_chip(text: String, muted: bool, theme: &OrbitTheme) -> AnyElement {
    apply_text_step(div(), theme.typography.size_xs)
        .font(theme.typography.mono_font(FontWeight::NORMAL))
        .px_1p5()
        .py_0p5()
        .rounded(theme.style.radius_control)
        .border_1()
        .border_color(theme.border.default)
        .bg(theme.surface.bg_panel)
        .text_color(if muted {
            theme.text.muted
        } else {
            theme.text.secondary
        })
        .truncate()
        .child(text)
        .into_any_element()
}

fn render_namespace_cell(
    ns: &NamespaceInfo,
    col: &TableColumn,
    theme: &OrbitTheme,
) -> AnyElement {
    match col.id.as_ref() {
        "name" => {
            let meta = resource_kind_meta_from_str("Namespace", theme);
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
                        .child(ns.name.clone()),
                )
                .into_any_element()
        }
        "status" => render_status_badge_from_str(&ns.status, StatusBadgeVariant::Inline, theme),
        "system" => {
            if ns.is_system {
                render_status_badge(
                    "System",
                    SemanticStatus::Info,
                    StatusBadgeVariant::Pill,
                    theme,
                )
            } else {
                apply_text_step(div(), theme.typography.size_xs)
                    .text_color(theme.text.muted)
                    .child("User")
                    .into_any_element()
            }
        }
        "labels" => {
            let summary = format_labels_summary(&ns.labels);
            if summary.visible.is_empty() {
                apply_text_step(div(), theme.typography.size_xs)
                    .text_color(theme.text.muted)
                    .child("-")
                    .into_any_element()
            } else {
                div()
                    .flex()
                    .items_center()
                    .gap_1()
                    .min_w(px(0.0))
                    .overflow_hidden()
                    .children(
                        summary
                            .visible
                            .into_iter()
                            .map(|entry| render_label_chip(entry, false, theme)),
                    )
                    .when(summary.extra > 0, |el| {
                        el.child(render_label_chip(format!("+{}", summary.extra), true, theme))
                    })
                    .into_any_element()
            }
        }
        "age" => apply_text_step(div(), theme.typography.size_xs)
            .font(theme.typography.mono_font(FontWeight::NORMAL))
            .text_color(theme.text.muted)
            .child(ns.age.clone())
            .into_any_element(),
        _ => div().into_any_element(),
    }
}

fn render_summary_cards_row(namespaces: &[NamespaceInfo], theme: &OrbitTheme) -> AnyElement {
    let cards = namespace_summary_cards(namespaces, theme);
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

#[allow(clippy::too_many_arguments)]
pub fn render_namespaces_view(
    state: &NamespaceFilterState,
    view_state: &NamespacesViewState,
    theme: &OrbitTheme,
    on_sort: impl 'static + Fn(SharedString, &ClickEvent, &mut Window, &mut App),
    on_row_click: impl 'static + Fn(usize, SharedString, &ClickEvent, &mut Window, &mut App),
    on_refresh: impl 'static + Fn(&mut Window, &mut App),
) -> AnyElement {
    let columns = namespace_columns();
    let sorted_rows: Rc<Vec<NamespaceInfo>> = Rc::new(
        sort_namespaces(&state.namespaces, view_state.table.sort.as_ref())
            .into_iter()
            .cloned()
            .collect(),
    );
    let total_count = sorted_rows.len();
    let is_loading = state.status == NamespaceLoadStatus::Loading;
    let on_refresh_rc: Rc<dyn Fn(&mut Window, &mut App)> = Rc::new(on_refresh);

    let toolbar = div()
        .flex()
        .items_center()
        .justify_between()
        .gap_3()
        .flex_shrink_0()
        .child(
            apply_text_step(div(), theme.typography.size_xs)
                .font_weight(FontWeight::MEDIUM)
                .text_color(theme.text.secondary)
                .child(format!("{total_count} namespaces")),
        )
        .child(render_action_button(
            "namespaces-refresh-btn",
            Icon::RefreshCw,
            if is_loading { "Refreshing..." } else { "Refresh" },
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
                            .child(format!("Failed to load namespaces: {message}")),
                    ),
            )
            .child(render_action_button(
                "namespaces-retry-btn",
                Icon::RotateCcw,
                "Retry",
                is_loading,
                theme,
                on_refresh_rc.clone(),
            ))
    });

    let body = match namespaces_presentation_mode(state) {
        NamespacesPresentationMode::ErrorCard => {
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
                                .child("Unable to Load Namespaces"),
                        ),
                )
                .child(
                    apply_text_step(div(), theme.typography.size_xs)
                        .text_color(theme.text.secondary)
                        .child(message),
                )
                .child(render_action_button(
                    "namespaces-retry-btn",
                    Icon::RotateCcw,
                    "Retry",
                    is_loading,
                    theme,
                    on_refresh_rc,
                ))
                .into_any_element()
        }
        NamespacesPresentationMode::Table => {
            let mut table_state = view_state.table.clone();
            table_state.loading = is_loading && state.namespaces.is_empty();
            table_state.empty_message = SharedString::from(namespaces_empty_message(state));

            let key_rows = sorted_rows.clone();
            let cell_rows = sorted_rows.clone();
            let callbacks = DataTableCallbacks::new(
                move |idx| {
                    key_rows
                        .get(idx)
                        .map(|ns| ns.name.clone())
                        .unwrap_or_default()
                },
                move |idx, col, col_theme| {
                    cell_rows
                        .get(idx)
                        .map(|ns| render_namespace_cell(ns, col, col_theme))
                        .unwrap_or_else(|| div().into_any_element())
                },
            )
            .on_sort(on_sort)
            .on_row_click(on_row_click);

            render_data_table(
                "namespaces-table",
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
        .gap_3()
        .child(render_summary_cards_row(&state.namespaces, theme))
        .child(toolbar)
        .when(
            namespaces_presentation_mode(state) == NamespacesPresentationMode::Table,
            |el| el.when_some(error_banner, |inner, banner| inner.child(banner)),
        )
        .child(body)
        .into_any_element()
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use gpui::WindowAppearance;
    use orbit_core::config::OrbitConfig;
    use orbit_core::kubernetes::models::NamespaceInfo;

    use super::*;
    use crate::components::data_table::{ColumnAlign, SortDirection, SortState};
    use crate::state::{NamespaceFilterState, NamespaceLoadStatus};
    use crate::theme::OrbitTheme;

    fn sample_ns(
        name: &str,
        status: &str,
        is_system: bool,
        age: &str,
        labels: &[(&str, &str)],
    ) -> NamespaceInfo {
        let mut map = BTreeMap::new();
        for (k, v) in labels {
            map.insert((*k).to_string(), (*v).to_string());
        }
        NamespaceInfo {
            name: name.to_string(),
            status: status.to_string(),
            is_system,
            age: age.to_string(),
            labels: map,
            annotations: BTreeMap::new(),
            uid: format!("uid-{name}"),
            created: "2026-01-01T00:00:00Z".to_string(),
        }
    }

    #[test]
    fn test_namespace_columns_cover_required_fields_and_age_alignment() {
        let cols = namespace_columns();
        let ids: Vec<&str> = cols.iter().map(|c| c.id.as_ref()).collect();
        assert_eq!(ids, vec!["name", "status", "system", "labels", "age"]);

        let age_col = cols.iter().find(|c| c.id.as_ref() == "age").expect("age col");
        assert_eq!(age_col.align, ColumnAlign::Right);
        assert!(age_col.monospace);
    }

    #[test]
    fn test_sort_namespaces_handles_name_status_system_labels_and_age() {
        let namespaces = vec![
            sample_ns("payments", "Active", false, "5d", &[("team", "core")]),
            sample_ns(
                "kube-system",
                "Active",
                true,
                "45s",
                &[("a", "1"), ("b", "2"), ("c", "3")],
            ),
            sample_ns("scratch", "Terminating", false, "Unknown", &[]),
        ];

        let by_name_asc: Vec<&str> = sort_namespaces(
            &namespaces,
            Some(&SortState {
                column_id: "name".to_string(),
                direction: SortDirection::Asc,
            }),
        )
        .into_iter()
        .map(|ns| ns.name.as_str())
        .collect();
        assert_eq!(by_name_asc, vec!["kube-system", "payments", "scratch"]);

        let by_status_desc: Vec<&str> = sort_namespaces(
            &namespaces,
            Some(&SortState {
                column_id: "status".to_string(),
                direction: SortDirection::Desc,
            }),
        )
        .into_iter()
        .map(|ns| ns.name.as_str())
        .collect();
        assert_eq!(by_status_desc[0], "scratch");

        let by_system_desc: Vec<&str> = sort_namespaces(
            &namespaces,
            Some(&SortState {
                column_id: "system".to_string(),
                direction: SortDirection::Desc,
            }),
        )
        .into_iter()
        .map(|ns| ns.name.as_str())
        .collect();
        assert_eq!(by_system_desc[0], "kube-system");

        let by_labels_asc: Vec<&str> = sort_namespaces(
            &namespaces,
            Some(&SortState {
                column_id: "labels".to_string(),
                direction: SortDirection::Asc,
            }),
        )
        .into_iter()
        .map(|ns| ns.name.as_str())
        .collect();
        assert_eq!(by_labels_asc, vec!["scratch", "payments", "kube-system"]);

        let by_age_asc: Vec<&str> = sort_namespaces(
            &namespaces,
            Some(&SortState {
                column_id: "age".to_string(),
                direction: SortDirection::Asc,
            }),
        )
        .into_iter()
        .map(|ns| ns.name.as_str())
        .collect();
        assert_eq!(by_age_asc, vec!["kube-system", "payments", "scratch"]);
    }

    #[test]
    fn test_format_labels_summary_truncates_after_two_labels() {
        let empty = BTreeMap::new();
        let summary_empty = format_labels_summary(&empty);
        assert!(summary_empty.visible.is_empty());
        assert_eq!(summary_empty.extra, 0);

        let two = BTreeMap::from([
            ("env".to_string(), "prod".to_string()),
            ("app".to_string(), "web".to_string()),
        ]);
        let summary_two = format_labels_summary(&two);
        assert_eq!(summary_two.visible, vec!["app=web", "env=prod"]);
        assert_eq!(summary_two.extra, 0);

        let four = BTreeMap::from([
            ("z".to_string(), "4".to_string()),
            ("b".to_string(), "2".to_string()),
            ("a".to_string(), "1".to_string()),
            ("c".to_string(), "3".to_string()),
        ]);
        let summary_four = format_labels_summary(&four);
        assert_eq!(summary_four.visible, vec!["a=1", "b=2"]);
        assert_eq!(summary_four.extra, 2);
    }

    #[test]
    fn test_compute_namespace_summary_and_cards_cover_total_active_and_system() {
        let namespaces = vec![
            sample_ns("default", "Active", true, "10d", &[]),
            sample_ns("kube-system", "Active", true, "10d", &[]),
            sample_ns("payments", "Active", false, "5d", &[]),
            sample_ns("old-ns", "Terminating", false, "1h", &[]),
        ];

        let summary = compute_namespace_summary(&namespaces);
        assert_eq!(summary.total, 4);
        assert_eq!(summary.active, 3);
        assert_eq!(summary.terminating, 1);
        assert_eq!(summary.system, 2);
        assert_eq!(summary.user, 2);

        for appearance in [WindowAppearance::Dark, WindowAppearance::Light] {
            let theme = OrbitTheme::from_config(&OrbitConfig::default(), appearance);
            let cards = namespace_summary_cards(&namespaces, &theme);
            assert_eq!(cards[0].title, "Total Namespaces");
            assert_eq!(cards[0].primary_value, "4");
            assert_eq!(cards[0].subtitle, "2 user • 1 terminating");
            assert_eq!(cards[0].accent_color, theme.accent.accent);

            assert_eq!(cards[1].title, "Active");
            assert_eq!(cards[1].primary_value, "3");
            assert_eq!(cards[1].subtitle, "75% active • 1 terminating");
            assert_eq!(cards[1].accent_color, theme.semantic.success.solid);

            assert_eq!(cards[2].title, "System");
            assert_eq!(cards[2].primary_value, "2");
            assert_eq!(cards[2].subtitle, "50% system • 2 user");
            assert_eq!(cards[2].accent_color, theme.semantic.info.solid);
        }
    }

    #[test]
    fn test_namespaces_view_empty_loading_and_error_presentation_modes() {
        let mut state = NamespaceFilterState::default();
        assert_eq!(
            namespaces_presentation_mode(&state),
            NamespacesPresentationMode::Table
        );
        assert_eq!(
            namespaces_empty_message(&state),
            "No namespaces found in this cluster."
        );

        state.begin_load_for_cluster("ctx-a");
        assert_eq!(state.status, NamespaceLoadStatus::Loading);
        assert_eq!(
            namespaces_presentation_mode(&state),
            NamespacesPresentationMode::Table
        );

        state.apply_failed("ctx-a", 1, "forbidden".to_string());
        assert_eq!(
            namespaces_presentation_mode(&state),
            NamespacesPresentationMode::ErrorCard
        );

        state
            .namespaces
            .push(sample_ns("default", "Active", true, "1d", &[]));
        assert_eq!(
            namespaces_presentation_mode(&state),
            NamespacesPresentationMode::Table
        );
    }
}
