use std::cmp::Ordering;
use std::rc::Rc;

use gpui::{
    AnyElement, App, ClickEvent, FontWeight, MouseDownEvent, Rgba, SharedString, Window, div,
    prelude::*, px,
};
use lucide_icons::Icon;
use orbit_core::kubernetes::models::{EventInfo, ServiceInfo};

use crate::components::data_table::{
    ColumnAlign, DataTableCallbacks, SortState, TableColumn, render_data_table, sort_rows_stable,
};
use crate::components::{
    ContextMenuEntry, ContextMenuItem, StatusBadgeVariant, render_status_badge_from_str,
};
use crate::icons::render_icon;
use crate::state::{NamespaceFilterState, ServiceTypeFilter, ServicesLoadStatus, ServicesState};
use crate::theme::{K8sResourceKind, OrbitTheme, apply_text_step};
pub use crate::views::{parse_age_seconds, render_action_button};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ServicesPresentationMode {
    Table,
    ErrorCard,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ServiceSummary {
    pub total: usize,
    pub cluster_ip: usize,
    pub node_port: usize,
    pub load_balancer: usize,
    pub external_name: usize,
}

#[derive(Debug, Clone)]
pub struct ServiceMetricCard {
    pub title: &'static str,
    pub primary_value: String,
    pub subtitle: String,
    pub accent_color: Rgba,
}

pub fn service_columns() -> Vec<TableColumn> {
    vec![
        TableColumn::new("name", "Name").flex_width(1.8, px(150.0)),
        TableColumn::new("namespace", "Namespace").flex_width(1.0, px(105.0)),
        TableColumn::new("type", "Type").fixed_width(px(120.0)),
        TableColumn::new("cluster_ip", "Cluster IP")
            .fixed_width(px(124.0))
            .monospace(true),
        TableColumn::new("external_ip", "External IP")
            .flex_width(1.3, px(130.0))
            .monospace(true),
        TableColumn::new("ports", "Ports")
            .flex_width(1.5, px(140.0))
            .monospace(true),
        TableColumn::new("endpoints", "Endpoints")
            .flex_width(1.1, px(105.0))
            .monospace(true),
        TableColumn::new("age", "Age")
            .fixed_width(px(76.0))
            .right_numeric(),
    ]
}

pub fn format_service_ports_summary(ports: &str) -> (String, usize) {
    let lines: Vec<&str> = ports
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && *line != "-")
        .collect();
    if lines.is_empty() {
        ("-".to_string(), 0)
    } else {
        (lines[0].to_string(), lines.len().saturating_sub(1))
    }
}

pub fn compare_services_by_col(a: &ServiceInfo, b: &ServiceInfo, col_id: &str) -> Ordering {
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
        "type" => a
            .r#type
            .to_ascii_lowercase()
            .cmp(&b.r#type.to_ascii_lowercase())
            .then_with(tie_break),
        "cluster_ip" => a
            .cluster_ip
            .to_ascii_lowercase()
            .cmp(&b.cluster_ip.to_ascii_lowercase())
            .then_with(tie_break),
        "external_ip" => a
            .external_ip
            .to_ascii_lowercase()
            .cmp(&b.external_ip.to_ascii_lowercase())
            .then_with(tie_break),
        "ports" => a
            .ports
            .to_ascii_lowercase()
            .cmp(&b.ports.to_ascii_lowercase())
            .then_with(tie_break),
        "endpoints" => a
            .endpoints
            .to_ascii_lowercase()
            .cmp(&b.endpoints.to_ascii_lowercase())
            .then_with(tie_break),
        "age" => parse_age_seconds(&a.age)
            .cmp(&parse_age_seconds(&b.age))
            .then_with(tie_break),
        _ => tie_break(),
    }
}

pub fn filter_and_sort_services<'a>(
    services: &'a [ServiceInfo],
    namespaces: &NamespaceFilterState,
    type_filter: ServiceTypeFilter,
    sort: Option<&SortState>,
) -> Vec<&'a ServiceInfo> {
    let mut rows: Vec<&'a ServiceInfo> = services
        .iter()
        .filter(|svc| {
            namespaces.matches_resource_namespace(Some(&svc.namespace))
                && type_filter.matches(&svc.r#type)
        })
        .collect();
    sort_rows_stable(&mut rows, sort, |a, b, col_id| {
        compare_services_by_col(a, b, col_id)
    });
    rows
}

pub fn compute_service_summary(services: &[&ServiceInfo]) -> ServiceSummary {
    let mut summary = ServiceSummary {
        total: services.len(),
        ..Default::default()
    };

    for svc in services {
        match svc.r#type.as_str() {
            "ClusterIP" => summary.cluster_ip += 1,
            "NodePort" => summary.node_port += 1,
            "LoadBalancer" => summary.load_balancer += 1,
            "ExternalName" => summary.external_name += 1,
            _ => {}
        }
    }

    summary
}

pub fn service_summary_cards(
    services: &[&ServiceInfo],
    namespaces: &NamespaceFilterState,
    theme: &OrbitTheme,
) -> Vec<ServiceMetricCard> {
    let summary = compute_service_summary(services);
    let pct = |count: usize, total: usize| -> f64 {
        if total == 0 {
            0.0
        } else {
            (count as f64 / total as f64) * 100.0
        }
    };

    let total_subtitle = if !namespaces.is_all_namespaces() {
        format!("Scope: {}", namespaces.summary_label())
    } else {
        "All namespaces".to_string()
    };

    vec![
        ServiceMetricCard {
            title: "Total Services",
            primary_value: summary.total.to_string(),
            subtitle: total_subtitle,
            accent_color: theme.k8s_color(K8sResourceKind::Service),
        },
        ServiceMetricCard {
            title: "ClusterIP",
            primary_value: summary.cluster_ip.to_string(),
            subtitle: format!("{:.0}% of services", pct(summary.cluster_ip, summary.total)),
            accent_color: theme.semantic.info.solid,
        },
        ServiceMetricCard {
            title: "LoadBalancer",
            primary_value: summary.load_balancer.to_string(),
            subtitle: format!(
                "{:.0}% of services",
                pct(summary.load_balancer, summary.total)
            ),
            accent_color: theme.semantic.success.solid,
        },
        ServiceMetricCard {
            title: "NodePort / Ext",
            primary_value: (summary.node_port + summary.external_name).to_string(),
            subtitle: format!(
                "{} NodePort · {} ExternalName",
                summary.node_port, summary.external_name
            ),
            accent_color: theme.semantic.warning.solid,
        },
    ]
}

pub fn services_empty_message(state: &ServicesState) -> &'static str {
    if state.services.is_empty() {
        "No services found in this cluster."
    } else if state.type_filter != ServiceTypeFilter::All {
        "No services match the selected type or namespace filter."
    } else {
        "No services found in the selected namespace(s)."
    }
}

pub fn services_presentation_mode(state: &ServicesState) -> ServicesPresentationMode {
    if state.status == ServicesLoadStatus::Error && state.services.is_empty() {
        ServicesPresentationMode::ErrorCard
    } else {
        ServicesPresentationMode::Table
    }
}

fn render_service_cell(svc: &ServiceInfo, col: &TableColumn, theme: &OrbitTheme) -> AnyElement {
    match col.id.as_ref() {
        "name" => apply_text_step(div(), theme.typography.size_xs)
            .font_weight(FontWeight::MEDIUM)
            .text_color(theme.text.primary)
            .truncate()
            .child(svc.name.clone())
            .into_any_element(),
        "namespace" => apply_text_step(div(), theme.typography.size_xs)
            .text_color(theme.text.secondary)
            .truncate()
            .child(svc.namespace.clone())
            .into_any_element(),
        "type" => render_status_badge_from_str(&svc.r#type, StatusBadgeVariant::Inline, theme),
        "cluster_ip" => apply_text_step(div(), theme.typography.size_xs)
            .font(theme.typography.mono_font(FontWeight::NORMAL))
            .text_color(if svc.cluster_ip == "-" || svc.cluster_ip == "None" {
                theme.text.muted
            } else {
                theme.text.secondary
            })
            .truncate()
            .child(svc.cluster_ip.clone())
            .into_any_element(),
        "external_ip" => apply_text_step(div(), theme.typography.size_xs)
            .font(theme.typography.mono_font(FontWeight::NORMAL))
            .text_color(if svc.external_ip == "-" {
                theme.text.muted
            } else if svc.external_ip == "<pending>" {
                theme.semantic.warning.solid
            } else {
                theme.text.secondary
            })
            .truncate()
            .child(svc.external_ip.clone())
            .into_any_element(),
        "ports" => {
            let (primary, extra) = format_service_ports_summary(&svc.ports);
            if extra == 0 {
                apply_text_step(div(), theme.typography.size_xs)
                    .font(theme.typography.mono_font(FontWeight::NORMAL))
                    .text_color(if primary == "-" {
                        theme.text.muted
                    } else {
                        theme.text.primary
                    })
                    .truncate()
                    .child(primary)
                    .into_any_element()
            } else {
                div()
                    .flex()
                    .items_center()
                    .gap_1p5()
                    .min_w(px(0.0))
                    .child(
                        apply_text_step(div(), theme.typography.size_xs)
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
                            .font(theme.typography.mono_font(FontWeight::MEDIUM))
                            .text_color(theme.text.muted)
                            .flex_shrink_0()
                            .child(format!("+{extra}")),
                    )
                    .into_any_element()
            }
        }
        "endpoints" => apply_text_step(div(), theme.typography.size_xs)
            .font(theme.typography.mono_font(FontWeight::NORMAL))
            .text_color(if svc.endpoints == "-" {
                theme.text.muted
            } else {
                theme.text.secondary
            })
            .truncate()
            .child(svc.endpoints.clone())
            .into_any_element(),
        "age" => apply_text_step(div(), theme.typography.size_xs)
            .font(theme.typography.mono_font(FontWeight::NORMAL))
            .text_color(theme.text.muted)
            .child(svc.age.clone())
            .into_any_element(),
        _ => div().into_any_element(),
    }
}

fn render_summary_cards_row(
    services: &[&ServiceInfo],
    namespaces: &NamespaceFilterState,
    theme: &OrbitTheme,
) -> AnyElement {
    let cards = service_summary_cards(services, namespaces, theme);
    div()
        .grid()
        .grid_cols(4)
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

pub fn build_service_context_menu_entries(
    svc: &ServiceInfo,
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
                format!("Filter by \"{}\"", svc.namespace),
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
            ContextMenuItem::new("delete-service", "Delete Service")
                .icon(Icon::Trash2)
                .danger(true)
                .disabled(true),
        ),
    ]
}

#[allow(clippy::too_many_arguments)]
pub fn render_services_view(
    state: &ServicesState,
    namespaces: &NamespaceFilterState,
    theme: &OrbitTheme,
    on_sort: impl 'static + Fn(SharedString, &ClickEvent, &mut Window, &mut App),
    on_row_click: impl 'static + Fn(usize, SharedString, &ClickEvent, &mut Window, &mut App),
    on_row_context_menu: impl 'static + Fn(SharedString, &ServiceInfo, &MouseDownEvent, &mut Window, &mut App),
    on_type_filter: impl 'static + Fn(ServiceTypeFilter, &mut Window, &mut App),
    on_refresh: impl 'static + Fn(&mut Window, &mut App),
) -> AnyElement {
    let columns = service_columns();
    let filtered_refs = filter_and_sort_services(
        &state.services,
        namespaces,
        state.type_filter,
        state.table.sort.as_ref(),
    );
    let summary_row = render_summary_cards_row(&filtered_refs, namespaces, theme);
    let filtered_rows: Rc<Vec<ServiceInfo>> =
        Rc::new(filtered_refs.into_iter().cloned().collect());
    let total_count = state.services.len();
    let visible_count = filtered_rows.len();
    let is_loading = state.status == ServicesLoadStatus::Loading;
    let on_refresh_rc: Rc<dyn Fn(&mut Window, &mut App)> = Rc::new(on_refresh);
    let on_type_filter_rc: Rc<dyn Fn(ServiceTypeFilter, &mut Window, &mut App)> =
        Rc::new(on_type_filter);

    let count_label = if namespaces.is_all_namespaces() && state.type_filter == ServiceTypeFilter::All
    {
        format!("{total_count} services")
    } else {
        format!("{visible_count} of {total_count} services")
    };

    let type_chips = div()
        .flex()
        .items_center()
        .gap_1()
        .children(ServiceTypeFilter::all().iter().copied().map(|filter| {
            let active = state.type_filter == filter;
            let on_filter = on_type_filter_rc.clone();
            let hover_bg = theme.surface.control_hover;
            div()
                .id(SharedString::from(format!("svc-type-filter-{}", filter.label())))
                .h(px(24.0))
                .px_2()
                .rounded(theme.style.radius_control)
                .border_1()
                .border_color(if active {
                    theme.accent.accent
                } else {
                    theme.border.default
                })
                .bg(if active {
                    theme.accent.soft
                } else {
                    theme.surface.control_bg
                })
                .flex()
                .items_center()
                .cursor_pointer()
                .when(!active, |el| el.hover(move |s| s.bg(hover_bg)))
                .on_click(move |_, window, cx| on_filter(filter, window, cx))
                .child(
                    apply_text_step(div(), theme.typography.size_2xs)
                        .font_weight(FontWeight::MEDIUM)
                        .text_color(if active {
                            theme.accent.accent
                        } else {
                            theme.text.secondary
                        })
                        .child(filter.label()),
                )
        }));

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
        .child(div().flex().items_center().gap_2().child(type_chips));

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
                            .child(format!("Failed to load services: {message}")),
                    ),
            )
            .child(render_action_button(
                "services-retry-btn",
                Icon::RotateCcw,
                "Retry",
                is_loading,
                theme,
                on_refresh_rc.clone(),
            ))
    });

    let mode = services_presentation_mode(state);
    let body = match mode {
        ServicesPresentationMode::ErrorCard => {
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
                                .child("Unable to Load Services"),
                        ),
                )
                .child(
                    apply_text_step(div(), theme.typography.size_xs)
                        .text_color(theme.text.secondary)
                        .child(message),
                )
                .child(render_action_button(
                    "services-retry-btn",
                    Icon::RotateCcw,
                    "Retry",
                    is_loading,
                    theme,
                    on_refresh_rc,
                ))
                .into_any_element()
        }
        ServicesPresentationMode::Table => {
            let mut table_state = state.table.clone();
            table_state.loading = is_loading && state.services.is_empty();
            table_state.empty_message = SharedString::from(services_empty_message(state));

            let key_rows = filtered_rows.clone();
            let cell_rows = filtered_rows.clone();
            let ctx_rows = filtered_rows.clone();
            let callbacks = DataTableCallbacks::new(
                move |idx| {
                    key_rows
                        .get(idx)
                        .map(|s| format!("{}/{}", s.namespace, s.name))
                        .unwrap_or_default()
                },
                move |idx, col, col_theme| {
                    cell_rows
                        .get(idx)
                        .map(|s| render_service_cell(s, col, col_theme))
                        .unwrap_or_else(|| div().into_any_element())
                },
            )
            .on_sort(on_sort)
            .on_row_click(on_row_click)
            .on_row_context_menu(move |idx, row_key, event, window, cx| {
                if let Some(svc) = ctx_rows.get(idx) {
                    on_row_context_menu(row_key, svc, event, window, cx);
                }
            });

            render_data_table(
                "services-table",
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
        .child(summary_row)
        .child(toolbar)
        .when(mode == ServicesPresentationMode::Table, |el| {
            el.when_some(error_banner, |el, banner| el.child(banner))
        })
        .child(body)
        .into_any_element()
}

pub fn filter_events_for_service<'a>(
    events: &'a [EventInfo],
    svc: &ServiceInfo,
) -> Vec<&'a EventInfo> {
    events
        .iter()
        .filter(|ev| {
            ev.namespace == svc.namespace
                && ((ev.object_kind == "Service" && ev.object_name == svc.name)
                    || ev.message.contains(&svc.name))
        })
        .collect()
}

pub fn render_service_drawer_overview(svc: &ServiceInfo, theme: &OrbitTheme) -> AnyElement {
    let summary = crate::views::render_drawer_key_values(
        vec![
            ("Name", svc.name.clone(), true),
            ("Namespace", svc.namespace.clone(), true),
            ("Type", svc.r#type.clone(), false),
            ("Cluster IP", svc.cluster_ip.clone(), true),
            ("External IP", svc.external_ip.clone(), true),
            ("Session Affinity", svc.session_affinity.clone(), false),
            (
                "Internal Traffic Policy",
                svc.internal_traffic_policy
                    .clone()
                    .unwrap_or_else(|| "-".to_string()),
                false,
            ),
            ("Age", svc.age.clone(), true),
        ],
        theme,
    );

    let mono = theme.typography.mono_font(FontWeight::NORMAL);
    let ports_section = {
        let content = if svc.ports_list.is_empty() {
            apply_text_step(div(), theme.typography.size_xs)
                .text_color(theme.text.muted)
                .child("No ports configured.")
                .into_any_element()
        } else {
            div()
                .w_full()
                .flex()
                .flex_col()
                .gap_1p5()
                .children(svc.ports_list.iter().map(|p| {
                    let protocol = p.protocol.as_deref().unwrap_or("TCP");
                    let target = orbit_core::kubernetes::services::format_service_port_target(p);
                    let node_port = p
                        .node_port
                        .map(|np| format!(" · NodePort {np}"))
                        .unwrap_or_default();
                    let label = p.name.clone().unwrap_or_else(|| "unnamed".to_string());
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
                            apply_text_step(div(), theme.typography.size_xs)
                                .font_weight(FontWeight::MEDIUM)
                                .text_color(theme.text.primary)
                                .child(label),
                        )
                        .child(
                            apply_text_step(div(), theme.typography.size_2xs)
                                .font(mono.clone())
                                .text_color(theme.text.secondary)
                                .child(format!("{}/{protocol} -> {target}{node_port}", p.port)),
                        )
                }))
                .into_any_element()
        };
        crate::views::render_drawer_section("PORTS", content, theme)
    };

    let endpoints_section = {
        let content = if svc.endpoints_list.is_empty() {
            apply_text_step(div(), theme.typography.size_xs)
                .text_color(theme.text.muted)
                .child("No ready endpoints.")
                .into_any_element()
        } else {
            div()
                .w_full()
                .flex()
                .flex_wrap()
                .gap_1p5()
                .children(svc.endpoints_list.iter().map(|ep| {
                    apply_text_step(div(), theme.typography.size_2xs)
                        .px_2()
                        .py_0p5()
                        .rounded(theme.style.radius_control)
                        .bg(theme.surface.control_bg)
                        .border_1()
                        .border_color(theme.border.default)
                        .font(mono.clone())
                        .text_color(theme.text.primary)
                        .child(ep.clone())
                }))
                .into_any_element()
        };
        crate::views::render_drawer_section("ENDPOINTS", content, theme)
    };

    div()
        .id("service-drawer-overview")
        .flex_1()
        .min_h_0()
        .overflow_y_scroll()
        .p_4()
        .flex()
        .flex_col()
        .gap_4()
        .child(crate::views::render_drawer_section("SUMMARY", summary, theme))
        .child(ports_section)
        .child(endpoints_section)
        .child(crate::views::render_drawer_kv_badges(
            "SELECTORS",
            &svc.selector,
            "No selectors",
            theme,
        ))
        .child(crate::views::render_drawer_kv_badges(
            "LABELS",
            &svc.labels,
            "No labels",
            theme,
        ))
        .into_any_element()
}

pub fn render_service_drawer_events(
    events: &[EventInfo],
    loading: bool,
    svc: &ServiceInfo,
    theme: &OrbitTheme,
) -> AnyElement {
    let matched = filter_events_for_service(events, svc);
    div()
        .id("service-drawer-events")
        .flex_1()
        .min_h_0()
        .overflow_y_scroll()
        .p_4()
        .child(crate::views::render_drawer_events_list(
            &matched,
            loading,
            "No events found for this service.",
            theme,
        ))
        .into_any_element()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::components::data_table::SortDirection;
    use std::collections::BTreeMap;

    #[allow(clippy::too_many_arguments)]
    fn make_service(
        name: &str,
        namespace: &str,
        svc_type: &str,
        cluster_ip: &str,
        external_ip: &str,
        ports: &str,
        endpoints: &str,
        age: &str,
    ) -> ServiceInfo {
        ServiceInfo {
            name: name.to_string(),
            namespace: namespace.to_string(),
            r#type: svc_type.to_string(),
            cluster_ip: cluster_ip.to_string(),
            external_ip: external_ip.to_string(),
            ports: ports.to_string(),
            endpoints: endpoints.to_string(),
            age: age.to_string(),
            session_affinity: "None".to_string(),
            internal_traffic_policy: None,
            created: String::new(),
            uid: format!("uid-{name}"),
            selector: BTreeMap::new(),
            labels: BTreeMap::new(),
            ports_list: Vec::new(),
            endpoints_list: Vec::new(),
            events: Vec::new(),
        }
    }

    #[test]
    fn test_service_columns_cover_required_fields_and_alignment() {
        let cols = service_columns();
        let ids: Vec<&str> = cols.iter().map(|c| c.id.as_ref()).collect();
        assert_eq!(
            ids,
            vec![
                "name",
                "namespace",
                "type",
                "cluster_ip",
                "external_ip",
                "ports",
                "endpoints",
                "age",
            ]
        );
        assert!(cols[3].monospace);
        assert!(cols[4].monospace);
        assert!(cols[5].monospace);
        assert!(cols[6].monospace);
        assert_eq!(cols[7].align, ColumnAlign::Right);
    }

    #[test]
    fn test_filter_and_sort_services_respects_namespaces_type_and_columns() {
        let services = vec![
            make_service("web", "prod", "LoadBalancer", "10.96.0.2", "34.1.1.1", "443/TCP", "2 ready", "5d"),
            make_service("api", "prod", "ClusterIP", "10.96.0.1", "-", "80/TCP", "1 ready", "30s"),
            make_service("db", "staging", "ExternalName", "None", "db.local", "-", "-", "2h"),
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
        ns.toggle_namespace("prod");

        let filtered = filter_and_sort_services(
            &services,
            &ns,
            ServiceTypeFilter::All,
            Some(&SortState {
                column_id: "age".to_string(),
                direction: SortDirection::Asc,
            }),
        );
        assert_eq!(filtered.len(), 2);
        assert_eq!(filtered[0].name, "api");
        assert_eq!(filtered[1].name, "web");

        let lb_only = filter_and_sort_services(
            &services,
            &ns,
            ServiceTypeFilter::LoadBalancer,
            None,
        );
        assert_eq!(lb_only.len(), 1);
        assert_eq!(lb_only[0].name, "web");
    }

    #[test]
    fn test_services_presentation_mode_and_empty_messages() {
        let mut state = ServicesState::default();
        assert_eq!(services_presentation_mode(&state), ServicesPresentationMode::Table);
        assert_eq!(
            services_empty_message(&state),
            "No services found in this cluster."
        );

        state.status = ServicesLoadStatus::Error;
        state.error_message = Some("forbidden".to_string());
        assert_eq!(
            services_presentation_mode(&state),
            ServicesPresentationMode::ErrorCard
        );

        state.services.push(make_service(
            "api",
            "prod",
            "ClusterIP",
            "10.96.0.1",
            "-",
            "80/TCP",
            "1 ready",
            "1m",
        ));
        assert_eq!(services_presentation_mode(&state), ServicesPresentationMode::Table);
        assert_eq!(
            services_empty_message(&state),
            "No services found in the selected namespace(s)."
        );

        state.type_filter = ServiceTypeFilter::NodePort;
        assert_eq!(
            services_empty_message(&state),
            "No services match the selected type or namespace filter."
        );
    }

    #[test]
    fn test_format_service_ports_summary_and_context_menu_entries() {
        assert_eq!(format_service_ports_summary("-"), ("-".to_string(), 0));
        assert_eq!(
            format_service_ports_summary("80/TCP\n443/TCP\n8080/TCP"),
            ("80/TCP".to_string(), 2)
        );

        let svc = make_service("api", "prod", "ClusterIP", "10.96.0.1", "-", "80/TCP", "1 ready", "1m");
        let entries = build_service_context_menu_entries(
            &svc,
            |_, _| {},
            |_, _| {},
            |_, _| {},
            |_, _| {},
            |_, _| {},
        );
        let ids: Vec<&str> = entries
            .iter()
            .filter_map(|e| match e {
                ContextMenuEntry::Item(item) => Some(item.id.as_ref()),
                ContextMenuEntry::Separator => None,
            })
            .collect();
        assert_eq!(
            ids,
            vec![
                "copy-name",
                "filter-namespace",
                "refresh",
                "view-details",
                "view-yaml",
                "delete-service",
            ]
        );
    }

    #[test]
    fn test_filter_events_for_service_matches_service_events() {
        let svc = make_service("api-svc", "prod", "ClusterIP", "10.96.0.1", "-", "80/TCP", "1 ready", "1m");
        let events = vec![
            EventInfo {
                uid: "e1".to_string(),
                r#type: "Normal".to_string(),
                reason: "UpdatedLoadBalancer".to_string(),
                object_kind: "Service".to_string(),
                object_name: "api-svc".to_string(),
                namespace: "prod".to_string(),
                message: "Updated load balancer".to_string(),
                source: "service-controller".to_string(),
                count: 1,
                time: "1m".to_string(),
                first_seen: String::new(),
                last_seen: String::new(),
                labels: BTreeMap::new(),
                annotations: BTreeMap::new(),
            },
            EventInfo {
                uid: "e2".to_string(),
                r#type: "Normal".to_string(),
                reason: "Scheduled".to_string(),
                object_kind: "Pod".to_string(),
                object_name: "other".to_string(),
                namespace: "prod".to_string(),
                message: "Unrelated".to_string(),
                source: "scheduler".to_string(),
                count: 1,
                time: "1m".to_string(),
                first_seen: String::new(),
                last_seen: String::new(),
                labels: BTreeMap::new(),
                annotations: BTreeMap::new(),
            },
        ];

        let matched = filter_events_for_service(&events, &svc);
        assert_eq!(matched.len(), 1);
        assert_eq!(matched[0].uid, "e1");
    }
}
