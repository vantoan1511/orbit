use std::cmp::Ordering;
use std::rc::Rc;

use gpui::{
    AnyElement, App, ClickEvent, ElementId, FontWeight, MouseDownEvent, SharedString, Window, div,
    prelude::*, px,
};
use lucide_icons::Icon;
use orbit_core::kubernetes::ingresses::parse_ingress_rule_summary;
use orbit_core::kubernetes::models::{EventInfo, IngressInfo};

use crate::components::data_table::{
    DataTableCallbacks, SortState, TableColumn, render_data_table, sort_rows_stable,
};
use crate::components::{ContextMenuEntry, ContextMenuItem, TextTooltip};
use crate::navigation::WorkspaceView;
use crate::state::{IngressesLoadStatus, IngressesState, NamespaceFilterState};
use crate::theme::{OrbitTheme, apply_text_step};
pub use crate::views::{
    parse_age_seconds, render_action_button, render_resource_error_banner,
    render_resource_error_state, render_resource_view_header, render_truncated_cell,
    table_numeric_font,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IngressesPresentationMode {
    Table,
    ErrorCard,
}

pub fn ingress_columns() -> Vec<TableColumn> {
    vec![
        TableColumn::new("name", "Name").flex_width(1.6, px(150.0)),
        TableColumn::new("namespace", "Namespace").flex_width(1.0, px(105.0)),
        TableColumn::new("class_name", "Class").fixed_width(px(116.0)),
        TableColumn::new("hosts", "Hosts")
            .flex_width(1.8, px(160.0))
            .monospace(true),
        TableColumn::new("address", "Address")
            .flex_width(1.3, px(130.0))
            .monospace(true),
        TableColumn::new("ports", "Ports")
            .fixed_width(px(92.0))
            .monospace(true),
        TableColumn::new("age", "Age")
            .fixed_width(px(76.0))
            .right_numeric(),
    ]
}

pub fn format_ingress_hosts_summary(hosts: &str) -> (String, usize) {
    let parts: Vec<&str> = hosts
        .split(',')
        .map(str::trim)
        .filter(|part| !part.is_empty())
        .collect();
    if parts.is_empty() {
        ("*".to_string(), 0)
    } else {
        (parts[0].to_string(), parts.len().saturating_sub(1))
    }
}

pub fn compare_ingresses_by_col(a: &IngressInfo, b: &IngressInfo, col_id: &str) -> Ordering {
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
        "class_name" => a
            .class_name
            .as_deref()
            .unwrap_or("")
            .to_ascii_lowercase()
            .cmp(&b.class_name.as_deref().unwrap_or("").to_ascii_lowercase())
            .then_with(tie_break),
        "hosts" => a
            .hosts
            .to_ascii_lowercase()
            .cmp(&b.hosts.to_ascii_lowercase())
            .then_with(tie_break),
        "address" => a
            .address
            .to_ascii_lowercase()
            .cmp(&b.address.to_ascii_lowercase())
            .then_with(tie_break),
        "ports" => a
            .ports
            .to_ascii_lowercase()
            .cmp(&b.ports.to_ascii_lowercase())
            .then_with(tie_break),
        "age" => parse_age_seconds(&a.age)
            .cmp(&parse_age_seconds(&b.age))
            .then_with(tie_break),
        _ => tie_break(),
    }
}

pub fn filter_and_sort_ingresses<'a>(
    ingresses: &'a [IngressInfo],
    namespaces: &NamespaceFilterState,
    sort: Option<&SortState>,
) -> Vec<&'a IngressInfo> {
    let mut rows: Vec<&'a IngressInfo> = ingresses
        .iter()
        .filter(|ing| namespaces.matches_resource_namespace(Some(&ing.namespace)))
        .collect();
    sort_rows_stable(&mut rows, sort, |a, b, col_id| {
        compare_ingresses_by_col(a, b, col_id)
    });
    rows
}

pub fn ingresses_empty_message(state: &IngressesState) -> &'static str {
    if state.ingresses.is_empty() {
        "No ingresses found in this cluster."
    } else {
        "No ingresses found in the selected namespace(s)."
    }
}

pub fn ingresses_presentation_mode(state: &IngressesState) -> IngressesPresentationMode {
    if state.status == IngressesLoadStatus::Error && state.ingresses.is_empty() {
        IngressesPresentationMode::ErrorCard
    } else {
        IngressesPresentationMode::Table
    }
}

fn render_ingress_cell(
    row_idx: usize,
    ing: &IngressInfo,
    col: &TableColumn,
    theme: &OrbitTheme,
) -> AnyElement {
    match col.id.as_ref() {
        "name" => render_truncated_cell(
            ElementId::NamedInteger(SharedString::from("ing-name"), row_idx as u64),
            ing.name.clone(),
            Some(SharedString::from(ing.name.clone())),
            theme.text.primary,
            FontWeight::MEDIUM,
            false,
            theme,
        ),
        "namespace" => render_truncated_cell(
            ElementId::NamedInteger(SharedString::from("ing-ns"), row_idx as u64),
            ing.namespace.clone(),
            None,
            theme.text.secondary,
            FontWeight::NORMAL,
            false,
            theme,
        ),
        "class_name" => match ing.class_name.as_deref().filter(|c| !c.is_empty()) {
            Some(class_name) => apply_text_step(div(), theme.typography.size_2xs)
                .px_1p5()
                .py_0p5()
                .rounded(theme.style.radius_control)
                .bg(theme.surface.control_bg)
                .border_1()
                .border_color(theme.border.default)
                .font_weight(FontWeight::MEDIUM)
                .text_color(theme.text.secondary)
                .truncate()
                .child(class_name.to_string())
                .into_any_element(),
            None => apply_text_step(div(), theme.typography.size_xs)
                .text_color(theme.text.muted)
                .child("-")
                .into_any_element(),
        },
        "hosts" => {
            let (primary, extra) = format_ingress_hosts_summary(&ing.hosts);
            if extra == 0 {
                render_truncated_cell(
                    ElementId::NamedInteger(SharedString::from("ing-hosts"), row_idx as u64),
                    primary.clone(),
                    (primary != "*").then(|| SharedString::from(primary.clone())),
                    if primary == "*" {
                        theme.text.muted
                    } else {
                        theme.text.primary
                    },
                    FontWeight::NORMAL,
                    true,
                    theme,
                )
            } else {
                let full_hosts = SharedString::from(ing.hosts.clone());
                div()
                    .id(ElementId::NamedInteger(
                        SharedString::from("ing-hosts"),
                        row_idx as u64,
                    ))
                    .flex()
                    .items_center()
                    .gap_1p5()
                    .min_w(px(0.0))
                    .flex_1()
                    .overflow_hidden()
                    .tooltip(move |_window, cx| {
                        cx.new(|_| TextTooltip::new(full_hosts.clone()).monospace(true))
                            .into()
                    })
                    .child(
                        apply_text_step(div(), theme.typography.size_xs)
                            .min_w(px(0.0))
                            .flex_1()
                            .font(theme.typography.mono_font(FontWeight::NORMAL))
                            .text_color(theme.text.primary)
                            .truncate()
                            .child(primary),
                    )
                    .child(
                        apply_text_step(div(), theme.typography.size_2xs)
                            .px_1p5()
                            .py_0p5()
                            .rounded(theme.style.radius_control)
                            .bg(theme.surface.control_bg)
                            .border_1()
                            .border_color(theme.border.default)
                            .font(table_numeric_font(theme))
                            .text_color(theme.text.muted)
                            .flex_shrink_0()
                            .child(format!("+{extra}")),
                    )
                    .into_any_element()
            }
        }
        "address" => render_truncated_cell(
            ElementId::NamedInteger(SharedString::from("ing-addr"), row_idx as u64),
            ing.address.clone(),
            None,
            if ing.address == "-" {
                theme.text.muted
            } else {
                theme.text.secondary
            },
            FontWeight::NORMAL,
            true,
            theme,
        ),
        "ports" => render_truncated_cell(
            ElementId::NamedInteger(SharedString::from("ing-ports"), row_idx as u64),
            ing.ports.clone(),
            None,
            theme.text.secondary,
            FontWeight::NORMAL,
            true,
            theme,
        ),
        "age" => apply_text_step(div(), theme.typography.size_xs)
            .font(theme.typography.mono_font(FontWeight::NORMAL))
            .text_color(theme.text.muted)
            .child(ing.age.clone())
            .into_any_element(),
        _ => div().into_any_element(),
    }
}

pub fn build_ingress_context_menu_entries(
    ing: &IngressInfo,
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
                format!("Filter by \"{}\"", ing.namespace),
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
            ContextMenuItem::new("delete-ingress", "Delete Ingress")
                .icon(Icon::Trash2)
                .danger(true)
                .on_select(on_delete),
        ),
    ]
}

#[allow(clippy::too_many_arguments)]
pub fn render_ingresses_view(
    state: &IngressesState,
    namespaces: &NamespaceFilterState,
    theme: &OrbitTheme,
    on_sort: impl 'static + Fn(SharedString, &ClickEvent, &mut Window, &mut App),
    on_row_click: impl 'static + Fn(usize, SharedString, &ClickEvent, &mut Window, &mut App),
    on_row_quick_yaml: impl 'static + Fn(usize, SharedString, &ClickEvent, &mut Window, &mut App),
    on_row_context_menu: impl 'static + Fn(SharedString, &IngressInfo, &MouseDownEvent, &mut Window, &mut App),
    on_refresh: impl 'static + Fn(&mut Window, &mut App),
) -> AnyElement {
    let columns = ingress_columns();
    let filtered_refs =
        filter_and_sort_ingresses(&state.ingresses, namespaces, state.table.sort.as_ref());
    let filtered_rows: Rc<Vec<IngressInfo>> =
        Rc::new(filtered_refs.into_iter().cloned().collect());
    let total_count = state.ingresses.len();
    let visible_count = filtered_rows.len();
    let is_loading = state.status == IngressesLoadStatus::Loading;
    let on_refresh_rc: super::ActionCallback = Rc::new(on_refresh);

    let count_label = if namespaces.is_all_namespaces() {
        format!("{total_count} ingresses")
    } else {
        format!("{visible_count} of {total_count} ingresses")
    };

    let header = render_resource_view_header(
        WorkspaceView::Ingresses.label(),
        WorkspaceView::Ingresses.description(),
        count_label,
        Some(namespaces),
        None,
        theme,
    );

    let error_banner = state.error_message.as_ref().map(|message| {
        render_resource_error_banner(
            "ingresses-retry-btn",
            format!("Failed to load ingresses: {message}"),
            is_loading,
            theme,
            on_refresh_rc.clone(),
        )
    });

    let mode = ingresses_presentation_mode(state);
    let body = match mode {
        IngressesPresentationMode::ErrorCard => {
            let message = state
                .error_message
                .clone()
                .unwrap_or_else(|| "Unknown error".to_string());
            render_resource_error_state(
                "ingresses-retry-btn",
                "Unable to Load Ingresses",
                message,
                is_loading,
                theme,
                on_refresh_rc,
            )
        }
        IngressesPresentationMode::Table => {
            let mut table_state = state.table.clone();
            table_state.loading = is_loading && state.ingresses.is_empty();
            table_state.empty_message = SharedString::from(ingresses_empty_message(state));

            let key_rows = filtered_rows.clone();
            let cell_rows = filtered_rows.clone();
            let ctx_rows = filtered_rows.clone();
            let callbacks = DataTableCallbacks::new(
                move |idx| {
                    key_rows
                        .get(idx)
                        .map(|ing| format!("{}/{}", ing.namespace, ing.name))
                        .unwrap_or_default()
                },
                move |idx, col, col_theme| {
                    cell_rows
                        .get(idx)
                        .map(|ing| render_ingress_cell(idx, ing, col, col_theme))
                        .unwrap_or_else(|| div().into_any_element())
                },
            )
            .on_sort(on_sort)
            .on_row_click(on_row_click)
            .on_row_quick_yaml(on_row_quick_yaml)
            .on_row_context_menu(move |idx, row_key, event, window, cx| {
                if let Some(ing) = ctx_rows.get(idx) {
                    on_row_context_menu(row_key, ing, event, window, cx);
                }
            });

            render_data_table(
                "ingresses-table",
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
        .when(mode == IngressesPresentationMode::Table, |el| {
            el.when_some(error_banner, |el, banner| el.child(banner))
        })
        .child(body)
        .into_any_element()
}

pub fn filter_events_for_ingress<'a>(
    events: &'a [EventInfo],
    ing: &IngressInfo,
) -> Vec<&'a EventInfo> {
    events
        .iter()
        .filter(|ev| {
            ev.namespace == ing.namespace
                && ((ev.object_kind == "Ingress" && ev.object_name == ing.name)
                    || ev.message.contains(&ing.name))
        })
        .collect()
}

fn render_ingress_rules_list(ing: &IngressInfo, theme: &OrbitTheme) -> AnyElement {
    if ing.rules_summary.is_empty() {
        return apply_text_step(div(), theme.typography.size_xs)
            .text_color(theme.text.muted)
            .child("No HTTP routing rules configured.")
            .into_any_element();
    }

    let mono = theme.typography.mono_font(FontWeight::NORMAL);
    div()
        .w_full()
        .flex()
        .flex_col()
        .gap_1p5()
        .children(ing.rules_summary.iter().map(|raw_rule| {
            let parsed = parse_ingress_rule_summary(raw_rule, &ing.hosts);
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
                .gap_3()
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap_1p5()
                        .min_w(px(0.0))
                        .child(
                            apply_text_step(div(), theme.typography.size_xs)
                                .font(mono.clone())
                                .font_weight(FontWeight::MEDIUM)
                                .text_color(theme.text.primary)
                                .truncate()
                                .child(parsed.host),
                        )
                        .child(
                            apply_text_step(div(), theme.typography.size_2xs)
                                .font(mono.clone())
                                .text_color(theme.text.muted)
                                .truncate()
                                .child(parsed.path),
                        ),
                )
                .child(
                    apply_text_step(div(), theme.typography.size_2xs)
                        .font(mono.clone())
                        .text_color(theme.text.secondary)
                        .flex_shrink_0()
                        .child(parsed.backend),
                )
        }))
        .into_any_element()
}

pub fn render_ingress_drawer_overview(ing: &IngressInfo, theme: &OrbitTheme) -> AnyElement {
    let summary = crate::views::render_drawer_key_values(
        vec![
            ("Name", ing.name.clone(), true),
            ("Namespace", ing.namespace.clone(), true),
            (
                "Class",
                ing.class_name
                    .as_deref()
                    .filter(|c| !c.trim().is_empty())
                    .unwrap_or("-")
                    .to_string(),
                true,
            ),
            ("Hosts", ing.hosts.clone(), true),
            ("Address", ing.address.clone(), true),
            ("Ports", ing.ports.clone(), true),
            ("Age", ing.age.clone(), true),
            (
                "Created",
                if ing.created.is_empty() {
                    "-".to_string()
                } else {
                    ing.created.clone()
                },
                false,
            ),
        ],
        theme,
    );

    let rules_section = crate::views::render_drawer_section(
        "ROUTING RULES",
        render_ingress_rules_list(ing, theme),
        theme,
    );

    div()
        .id("ingress-drawer-overview")
        .flex_1()
        .min_h_0()
        .overflow_y_scroll()
        .p_4()
        .flex()
        .flex_col()
        .gap_4()
        .child(crate::views::render_drawer_section(
            "GENERAL", summary, theme,
        ))
        .child(rules_section)
        .child(crate::views::render_drawer_kv_badges(
            "LABELS",
            &ing.labels,
            "No labels",
            theme,
        ))
        .child(crate::views::render_drawer_kv_badges(
            "ANNOTATIONS",
            &ing.annotations,
            "No annotations",
            theme,
        ))
        .into_any_element()
}

pub fn render_ingress_drawer_rules(ing: &IngressInfo, theme: &OrbitTheme) -> AnyElement {
    let mono = theme.typography.mono_font(FontWeight::NORMAL);
    let content = if ing.rules_summary.is_empty() {
        apply_text_step(div(), theme.typography.size_xs)
            .text_color(theme.text.muted)
            .child("No HTTP routing rules configured for this ingress.")
            .into_any_element()
    } else {
        let header = div()
            .w_full()
            .px_2p5()
            .py_1()
            .border_b_1()
            .border_color(theme.border.default)
            .flex()
            .items_center()
            .gap_3()
            .child(
                apply_text_step(div(), theme.typography.size_2xs)
                    .flex_1()
                    .font_weight(FontWeight::SEMIBOLD)
                    .text_color(theme.text.muted)
                    .child("HOST"),
            )
            .child(
                apply_text_step(div(), theme.typography.size_2xs)
                    .w(px(120.0))
                    .font_weight(FontWeight::SEMIBOLD)
                    .text_color(theme.text.muted)
                    .child("PATH"),
            )
            .child(
                apply_text_step(div(), theme.typography.size_2xs)
                    .flex_1()
                    .font_weight(FontWeight::SEMIBOLD)
                    .text_color(theme.text.muted)
                    .child("BACKEND"),
            );

        div()
            .w_full()
            .rounded(theme.style.radius_control)
            .border_1()
            .border_color(theme.border.default)
            .bg(theme.surface.bg_panel)
            .flex()
            .flex_col()
            .child(header)
            .children(ing.rules_summary.iter().enumerate().map(|(idx, raw_rule)| {
                let parsed = parse_ingress_rule_summary(raw_rule, &ing.hosts);
                div()
                    .w_full()
                    .px_2p5()
                    .py_2()
                    .when(idx > 0, |el| el.border_t_1().border_color(theme.border.default))
                    .flex()
                    .items_center()
                    .gap_3()
                    .child(
                        apply_text_step(div(), theme.typography.size_xs)
                            .flex_1()
                            .min_w(px(0.0))
                            .font(mono.clone())
                            .font_weight(FontWeight::MEDIUM)
                            .text_color(theme.text.primary)
                            .truncate()
                            .child(parsed.host),
                    )
                    .child(
                        apply_text_step(div(), theme.typography.size_xs)
                            .w(px(120.0))
                            .min_w(px(0.0))
                            .font(mono.clone())
                            .text_color(theme.text.secondary)
                            .truncate()
                            .child(parsed.path),
                    )
                    .child(
                        apply_text_step(div(), theme.typography.size_xs)
                            .flex_1()
                            .min_w(px(0.0))
                            .font(mono.clone())
                            .text_color(theme.text.secondary)
                            .truncate()
                            .child(parsed.backend),
                    )
            }))
            .into_any_element()
    };

    div()
        .id("ingress-drawer-rules")
        .flex_1()
        .min_h_0()
        .overflow_y_scroll()
        .p_4()
        .flex()
        .flex_col()
        .gap_4()
        .child(crate::views::render_drawer_section("RULES", content, theme))
        .into_any_element()
}

pub fn render_ingress_drawer_events(
    events: &[EventInfo],
    loading: bool,
    ing: &IngressInfo,
    theme: &OrbitTheme,
) -> AnyElement {
    let matched = filter_events_for_ingress(events, ing);
    div()
        .id("ingress-drawer-events")
        .flex_1()
        .min_h_0()
        .overflow_y_scroll()
        .p_4()
        .child(crate::views::render_drawer_events_list(
            &matched,
            loading,
            "No events found for this ingress.",
            theme,
        ))
        .into_any_element()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::components::data_table::{ColumnAlign, SortDirection};
    use std::collections::BTreeMap;

    #[allow(clippy::too_many_arguments)]
    fn make_ingress(
        name: &str,
        namespace: &str,
        class_name: Option<&str>,
        hosts: &str,
        address: &str,
        ports: &str,
        age: &str,
    ) -> IngressInfo {
        IngressInfo {
            name: name.to_string(),
            namespace: namespace.to_string(),
            class_name: class_name.map(ToString::to_string),
            hosts: hosts.to_string(),
            address: address.to_string(),
            ports: ports.to_string(),
            age: age.to_string(),
            created: "Nov 18, 2026, 10:00 AM".to_string(),
            uid: format!("uid-{name}"),
            labels: BTreeMap::new(),
            annotations: BTreeMap::new(),
            rules_summary: vec![format!("{hosts} -> / (svc:80)")],
        }
    }

    #[test]
    fn test_ingress_columns_cover_required_fields_and_alignment() {
        let cols = ingress_columns();
        let ids: Vec<&str> = cols.iter().map(|c| c.id.as_ref()).collect();
        assert_eq!(
            ids,
            vec![
                "name",
                "namespace",
                "class_name",
                "hosts",
                "address",
                "ports",
                "age",
            ]
        );
        assert!(cols[3].monospace);
        assert!(cols[4].monospace);
        assert!(cols[5].monospace);
        assert_eq!(cols[6].align, ColumnAlign::Right);
        assert!(cols[6].monospace);
    }

    #[test]
    fn test_format_ingress_hosts_summary_handles_single_wildcard_and_multi_host() {
        assert_eq!(format_ingress_hosts_summary("*"), ("*".to_string(), 0));
        assert_eq!(format_ingress_hosts_summary(""), ("*".to_string(), 0));
        assert_eq!(
            format_ingress_hosts_summary("api.example.com"),
            ("api.example.com".to_string(), 0)
        );
        assert_eq!(
            format_ingress_hosts_summary("a.com, b.com, c.com"),
            ("a.com".to_string(), 2)
        );
    }

    #[test]
    fn test_filter_and_sort_ingresses_respects_namespace_and_sorts_all_columns() {
        let ingresses = vec![
            make_ingress(
                "zeta",
                "prod",
                Some("traefik"),
                "zeta.example.com",
                "34.10.0.2",
                "80, 443",
                "3d",
            ),
            make_ingress(
                "alpha",
                "staging",
                None,
                "alpha.example.com",
                "-",
                "80",
                "10m",
            ),
            make_ingress(
                "beta",
                "prod",
                Some("nginx"),
                "beta.example.com",
                "34.10.0.1",
                "80, 443",
                "2h",
            ),
        ];

        // All namespaces, sorted by age ascending (10m < 2h < 3d)
        let all_ns = NamespaceFilterState::default();
        let by_age = filter_and_sort_ingresses(
            &ingresses,
            &all_ns,
            Some(&SortState {
                column_id: "age".to_string(),
                direction: SortDirection::Asc,
            }),
        );
        assert_eq!(
            by_age.iter().map(|i| i.name.as_str()).collect::<Vec<_>>(),
            vec!["alpha", "beta", "zeta"]
        );

        // Sorted by class_name ascending (None -> "" < "nginx" < "traefik")
        let by_class = filter_and_sort_ingresses(
            &ingresses,
            &all_ns,
            Some(&SortState {
                column_id: "class_name".to_string(),
                direction: SortDirection::Asc,
            }),
        );
        assert_eq!(
            by_class.iter().map(|i| i.name.as_str()).collect::<Vec<_>>(),
            vec!["alpha", "beta", "zeta"]
        );

        // Sorted by hosts descending
        let by_hosts_desc = filter_and_sort_ingresses(
            &ingresses,
            &all_ns,
            Some(&SortState {
                column_id: "hosts".to_string(),
                direction: SortDirection::Desc,
            }),
        );
        assert_eq!(
            by_hosts_desc
                .iter()
                .map(|i| i.name.as_str())
                .collect::<Vec<_>>(),
            vec!["zeta", "beta", "alpha"]
        );

        // Sorted by address ascending ("-" < "34.10.0.1" < "34.10.0.2")
        let by_addr = filter_and_sort_ingresses(
            &ingresses,
            &all_ns,
            Some(&SortState {
                column_id: "address".to_string(),
                direction: SortDirection::Asc,
            }),
        );
        assert_eq!(
            by_addr.iter().map(|i| i.name.as_str()).collect::<Vec<_>>(),
            vec!["alpha", "beta", "zeta"]
        );

        // Sorted by ports descending ("80, 443" before "80")
        let by_ports_desc = filter_and_sort_ingresses(
            &ingresses,
            &all_ns,
            Some(&SortState {
                column_id: "ports".to_string(),
                direction: SortDirection::Desc,
            }),
        );
        assert_eq!(by_ports_desc[2].name, "alpha");

        // Namespace filter: single namespace "prod"
        let mut prod_ns = NamespaceFilterState::default();
        prod_ns.begin_load_for_cluster("ctx-a");
        prod_ns.apply_loaded(
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
                    name: "staging".to_string(),
                    status: "Active".to_string(),
                    age: "1d".to_string(),
                    is_system: false,
                    labels: BTreeMap::new(),
                    annotations: BTreeMap::new(),
                    uid: "ns-staging".to_string(),
                    created: String::new(),
                },
            ],
        );
        prod_ns.toggle_namespace("prod");
        let prod_only = filter_and_sort_ingresses(
            &ingresses,
            &prod_ns,
            Some(&SortState {
                column_id: "name".to_string(),
                direction: SortDirection::Asc,
            }),
        );
        assert_eq!(
            prod_only
                .iter()
                .map(|i| i.name.as_str())
                .collect::<Vec<_>>(),
            vec!["beta", "zeta"]
        );

        // Namespace filter: multi-namespace ("prod" + "staging")
        prod_ns.toggle_namespace("staging");
        let multi = filter_and_sort_ingresses(
            &ingresses,
            &prod_ns,
            Some(&SortState {
                column_id: "namespace".to_string(),
                direction: SortDirection::Asc,
            }),
        );
        assert_eq!(multi.len(), 3);
        assert_eq!(multi[2].namespace, "staging");
    }

    #[test]
    fn test_ingresses_empty_message_and_presentation_mode() {
        let mut state = IngressesState::default();
        assert_eq!(
            ingresses_empty_message(&state),
            "No ingresses found in this cluster."
        );
        assert_eq!(
            ingresses_presentation_mode(&state),
            IngressesPresentationMode::Table
        );

        state.status = IngressesLoadStatus::Error;
        state.error_message = Some("forbidden".to_string());
        assert_eq!(
            ingresses_presentation_mode(&state),
            IngressesPresentationMode::ErrorCard
        );

        state.ingresses.push(make_ingress(
            "web",
            "prod",
            Some("nginx"),
            "app.example.com",
            "1.2.3.4",
            "80, 443",
            "1d",
        ));
        assert_eq!(
            ingresses_empty_message(&state),
            "No ingresses found in the selected namespace(s)."
        );
        assert_eq!(
            ingresses_presentation_mode(&state),
            IngressesPresentationMode::Table
        );
    }

    #[test]
    fn test_filter_events_for_ingress_matches_namespace_and_kind_or_message() {
        let ing = make_ingress(
            "web-ingress",
            "prod",
            Some("nginx"),
            "app.example.com",
            "1.2.3.4",
            "80, 443",
            "1d",
        );

        let events = vec![
            EventInfo {
                uid: "ev-1".to_string(),
                name: String::new(),
                r#type: "Normal".to_string(),
                reason: "Sync".to_string(),
                message: "Scheduled for sync".to_string(),
                namespace: "prod".to_string(),
                object_kind: "Ingress".to_string(),
                object_name: "web-ingress".to_string(),
                source: "nginx-ingress-controller".to_string(),
                count: 1,
                time: "1m".to_string(),
                first_seen: String::new(),
                last_seen: "1m".to_string(),
                labels: BTreeMap::new(),
                annotations: BTreeMap::new(),
            },
            EventInfo {
                uid: "ev-2".to_string(),
                name: String::new(),
                r#type: "Warning".to_string(),
                reason: "Reload".to_string(),
                message: "Updated backend for web-ingress".to_string(),
                namespace: "prod".to_string(),
                object_kind: "Service".to_string(),
                object_name: "api-svc".to_string(),
                source: "controller".to_string(),
                count: 1,
                time: "2m".to_string(),
                first_seen: String::new(),
                last_seen: "2m".to_string(),
                labels: BTreeMap::new(),
                annotations: BTreeMap::new(),
            },
            EventInfo {
                uid: "ev-3".to_string(),
                name: String::new(),
                r#type: "Normal".to_string(),
                reason: "Sync".to_string(),
                message: "Scheduled for sync".to_string(),
                namespace: "staging".to_string(),
                object_kind: "Ingress".to_string(),
                object_name: "web-ingress".to_string(),
                source: "nginx-ingress-controller".to_string(),
                count: 1,
                time: "3m".to_string(),
                first_seen: String::new(),
                last_seen: "3m".to_string(),
                labels: BTreeMap::new(),
                annotations: BTreeMap::new(),
            },
        ];

        let matched = filter_events_for_ingress(&events, &ing);
        let uids: Vec<&str> = matched.iter().map(|ev| ev.uid.as_str()).collect();
        assert_eq!(uids, vec!["ev-1", "ev-2"]);
    }

    #[test]
    fn test_build_ingress_context_menu_entries_matches_standard_actions() {
        let ing = make_ingress(
            "web-ingress",
            "prod",
            Some("nginx"),
            "app.example.com",
            "1.2.3.4",
            "80, 443",
            "1d",
        );

        let entries = build_ingress_context_menu_entries(
            &ing,
            |_, _| {},
            |_, _| {},
            |_, _| {},
            |_, _| {},
            |_, _| {},
            |_, _| {},
        );

        let ids: Vec<Option<&str>> = entries
            .iter()
            .map(|entry| match entry {
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
                Some("delete-ingress"),
            ]
        );

        let delete_item = match entries.last() {
            Some(ContextMenuEntry::Item(item)) => item,
            _ => panic!("Expected delete-ingress item"),
        };
        assert!(delete_item.danger);
        assert!(!delete_item.disabled);
        assert!(delete_item.on_select.is_some());
    }
}
