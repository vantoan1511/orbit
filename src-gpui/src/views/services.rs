use std::cmp::Ordering;
use std::rc::Rc;

use gpui::{
    AnyElement, App, ClickEvent, ElementId, FontWeight, MouseDownEvent, SharedString, Window, div,
    prelude::*, px,
};
use lucide_icons::Icon;
use orbit_core::kubernetes::models::{EventInfo, ServiceInfo};

use crate::components::data_table::{
    DataTableCallbacks, DataTableEnhancementCallbacks, SortState, TableColumn,
    TableFilterChipGroup, TableFilterChipOption, TableFilterState, render_enhanced_data_table,
    sort_rows_stable,
};
use crate::components::{
    ContextMenuEntry, ContextMenuItem, StatusBadgeVariant, TextTooltip,
    render_status_badge_from_str,
};
use crate::state::{NamespaceFilterState, ServiceTypeFilter, ServicesLoadStatus, ServicesState};
use crate::theme::{OrbitTheme, apply_text_step};
pub use crate::views::{
    parse_age_seconds, render_action_button, render_resource_error_banner,
    render_resource_error_state, render_resource_view_header, render_truncated_cell,
    table_numeric_font,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ServicesPresentationMode {
    Table,
    ErrorCard,
}

pub fn service_columns() -> Vec<TableColumn> {
    vec![
        TableColumn::new("name", "Name")
            .flex_width(1.8, px(150.0))
            .locked(true),
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
    filter_and_sort_services_with_filters(
        services,
        namespaces,
        type_filter,
        &TableFilterState::default(),
        sort,
    )
}

pub fn filter_and_sort_services_with_filters<'a>(
    services: &'a [ServiceInfo],
    namespaces: &NamespaceFilterState,
    type_filter: ServiceTypeFilter,
    filters: &TableFilterState,
    sort: Option<&SortState>,
) -> Vec<&'a ServiceInfo> {
    let query = filters.search_query.trim().to_ascii_lowercase();
    let extra_type = filters
        .extra_filters
        .get("type")
        .map(|s| s.trim().to_ascii_lowercase())
        .filter(|s| !s.is_empty() && s != "all");

    let mut rows: Vec<&'a ServiceInfo> = services
        .iter()
        .filter(|svc| {
            namespaces.matches_resource_namespace(Some(&svc.namespace))
                && type_filter.matches(&svc.r#type)
                && extra_type
                    .as_deref()
                    .is_none_or(|t| svc.r#type.to_ascii_lowercase() == t)
        })
        .filter(|svc| {
            if query.is_empty() {
                return true;
            }
            svc.name.to_ascii_lowercase().contains(&query)
                || svc.namespace.to_ascii_lowercase().contains(&query)
                || svc.r#type.to_ascii_lowercase().contains(&query)
                || svc.cluster_ip.to_ascii_lowercase().contains(&query)
                || svc.external_ip.to_ascii_lowercase().contains(&query)
                || svc.ports.to_ascii_lowercase().contains(&query)
        })
        .collect();
    sort_rows_stable(&mut rows, sort, |a, b, col_id| {
        compare_services_by_col(a, b, col_id)
    });
    rows
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

fn render_service_cell(
    row_idx: usize,
    svc: &ServiceInfo,
    col: &TableColumn,
    theme: &OrbitTheme,
) -> AnyElement {
    match col.id.as_ref() {
        "name" => render_truncated_cell(
            ElementId::NamedInteger(SharedString::from("svc-name"), row_idx as u64),
            svc.name.clone(),
            Some(SharedString::from(svc.name.clone())),
            theme.text.primary,
            FontWeight::MEDIUM,
            false,
            theme,
        ),
        "namespace" => render_truncated_cell(
            ElementId::NamedInteger(SharedString::from("svc-ns"), row_idx as u64),
            svc.namespace.clone(),
            None,
            theme.text.secondary,
            FontWeight::NORMAL,
            false,
            theme,
        ),
        "type" => render_status_badge_from_str(&svc.r#type, StatusBadgeVariant::Inline, theme),
        "cluster_ip" => render_truncated_cell(
            ElementId::NamedInteger(SharedString::from("svc-cip"), row_idx as u64),
            svc.cluster_ip.clone(),
            None,
            if svc.cluster_ip == "-" || svc.cluster_ip == "None" {
                theme.text.muted
            } else {
                theme.text.secondary
            },
            FontWeight::NORMAL,
            true,
            theme,
        ),
        "external_ip" => render_truncated_cell(
            ElementId::NamedInteger(SharedString::from("svc-eip"), row_idx as u64),
            svc.external_ip.clone(),
            (svc.external_ip != "-").then(|| SharedString::from(svc.external_ip.clone())),
            if svc.external_ip == "-" {
                theme.text.muted
            } else if svc.external_ip == "<pending>" {
                theme.semantic.warning.solid
            } else {
                theme.text.secondary
            },
            FontWeight::NORMAL,
            true,
            theme,
        ),
        "ports" => {
            let (primary, extra) = format_service_ports_summary(&svc.ports);
            if extra == 0 {
                render_truncated_cell(
                    ElementId::NamedInteger(SharedString::from("svc-ports"), row_idx as u64),
                    primary.clone(),
                    (primary != "-").then(|| SharedString::from(primary)),
                    if svc.ports.is_empty() {
                        theme.text.muted
                    } else {
                        theme.text.primary
                    },
                    FontWeight::NORMAL,
                    true,
                    theme,
                )
            } else {
                let full_ports = SharedString::from(svc.ports.clone());
                div()
                    .id(ElementId::NamedInteger(
                        SharedString::from("svc-ports"),
                        row_idx as u64,
                    ))
                    .flex()
                    .items_center()
                    .gap_1p5()
                    .min_w(px(0.0))
                    .flex_1()
                    .overflow_hidden()
                    .tooltip(move |_window, cx| {
                        cx.new(|_| TextTooltip::new(full_ports.clone()).monospace(true))
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
        "endpoints" => render_truncated_cell(
            ElementId::NamedInteger(SharedString::from("svc-ep"), row_idx as u64),
            svc.endpoints.clone(),
            None,
            if svc.endpoints == "-" {
                theme.text.muted
            } else {
                theme.text.secondary
            },
            FontWeight::NORMAL,
            true,
            theme,
        ),
        "age" => apply_text_step(div(), theme.typography.size_xs)
            .font(theme.typography.mono_font(FontWeight::NORMAL))
            .text_color(theme.text.muted)
            .child(svc.age.clone())
            .into_any_element(),
        _ => div().into_any_element(),
    }
}

#[allow(clippy::too_many_arguments)]
pub fn build_service_context_menu_entries(
    svc: &ServiceInfo,
    on_copy_name: impl 'static + Fn(&mut Window, &mut App),
    on_filter_namespace: impl 'static + Fn(&mut Window, &mut App),
    on_refresh: impl 'static + Fn(&mut Window, &mut App),
    on_view_details: impl 'static + Fn(&mut Window, &mut App),
    on_view_yaml: impl 'static + Fn(&mut Window, &mut App),
    on_edit: impl 'static + Fn(&mut Window, &mut App),
    on_port_forward: impl 'static + Fn(&mut Window, &mut App),
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
        ContextMenuEntry::Item(
            ContextMenuItem::new("edit", "Edit")
                .icon(Icon::Pencil)
                .on_select(on_edit),
        ),
        ContextMenuEntry::Item(
            ContextMenuItem::new("port-forward", "Port Forward...")
                .icon(Icon::Cable)
                .on_select(on_port_forward),
        ),
        ContextMenuEntry::Separator,
        ContextMenuEntry::Item(
            ContextMenuItem::new("delete-service", "Delete Service")
                .icon(Icon::Trash2)
                .danger(true)
                .on_select(on_delete),
        ),
    ]
}

pub fn cached_filtered_services(
    state: &ServicesState,
    namespaces: &NamespaceFilterState,
) -> Rc<Vec<ServiceInfo>> {
    let key = crate::state::RowCacheKey::from_table(
        state.data_revision,
        namespaces.selection_revision,
        &state.table,
        state.type_filter as u64,
    );
    state.row_cache.get_or_compute(key, || {
        filter_and_sort_services_with_filters(
            &state.services,
            namespaces,
            state.type_filter,
            &state.table.filters,
            state.table.sort.as_ref(),
        )
        .into_iter()
        .cloned()
        .collect()
    })
}

#[allow(clippy::too_many_arguments)]
pub fn render_services_view(
    state: &ServicesState,
    namespaces: &NamespaceFilterState,
    theme: &OrbitTheme,
    on_sort: impl 'static + Fn(SharedString, &ClickEvent, &mut Window, &mut App),
    on_row_click: impl 'static + Fn(usize, SharedString, &ClickEvent, &mut Window, &mut App),
    on_row_quick_yaml: impl 'static + Fn(usize, SharedString, &ClickEvent, &mut Window, &mut App),
    on_row_context_menu: impl 'static + Fn(SharedString, &ServiceInfo, &MouseDownEvent, &mut Window, &mut App),
    on_type_filter: impl 'static + Fn(ServiceTypeFilter, &mut Window, &mut App),
    on_refresh: impl 'static + Fn(&mut Window, &mut App),
    enhancements: Option<DataTableEnhancementCallbacks>,
) -> AnyElement {
    let columns = service_columns();
    let filtered_rows = cached_filtered_services(state, namespaces);
    let total_count = state.services.len();
    let visible_count = filtered_rows.len();
    let is_loading = state.status == ServicesLoadStatus::Loading;
    let on_refresh_rc: super::ActionCallback = Rc::new(on_refresh);
    let on_type_filter_rc: super::ValueCallback<ServiceTypeFilter> = Rc::new(on_type_filter);

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

    let header = render_resource_view_header(
        count_label,
        Some(namespaces),
        if enhancements.is_some() {
            None
        } else {
            Some(type_chips.into_any_element())
        },
        theme,
    );

    let filter_groups = [TableFilterChipGroup {
        key: "type",
        default_value: "All",
        options: vec![
            TableFilterChipOption { value: "All", label: "All" },
            TableFilterChipOption { value: "ClusterIP", label: "ClusterIP" },
            TableFilterChipOption { value: "NodePort", label: "NodePort" },
            TableFilterChipOption { value: "LoadBalancer", label: "LoadBalancer" },
            TableFilterChipOption { value: "ExternalName", label: "ExternalName" },
        ],
    }];

    let error_banner = state.error_message.as_ref().map(|message| {
        render_resource_error_banner(
            "services-retry-btn",
            format!("Failed to load services: {message}"),
            is_loading,
            theme,
            on_refresh_rc.clone(),
        )
    });

    let mode = services_presentation_mode(state);
    let body = match mode {
        ServicesPresentationMode::ErrorCard => {
            let message = state
                .error_message
                .clone()
                .unwrap_or_else(|| "Unknown error".to_string());
            render_resource_error_state(
                "services-retry-btn",
                "Unable to Load Services",
                message,
                is_loading,
                theme,
                on_refresh_rc,
            )
        }
        ServicesPresentationMode::Table => {
            let mut table_state = state.table.clone();
            table_state.loading = is_loading && state.services.is_empty();
            table_state.refreshing = is_loading && !state.services.is_empty();
            table_state.empty_message = SharedString::from(services_empty_message(state));

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
                    key_rows
                        .get(idx)
                        .map(|s| format!("{}/{}", s.namespace, s.name))
                        .unwrap_or_default()
                },
                move |idx, col, col_theme| {
                    cell_rows
                        .get(idx)
                        .map(|s| render_service_cell(idx, s, col, col_theme))
                        .unwrap_or_else(|| div().into_any_element())
                },
            )
            .on_sort(on_sort)
            .on_row_click(on_row_click)
            .on_row_quick_yaml(on_row_quick_yaml)
            .on_empty_action(empty_action)
            .on_row_context_menu(move |idx, row_key, event, window, cx| {
                if let Some(svc) = ctx_rows.get(idx) {
                    on_row_context_menu(row_key, svc, event, window, cx);
                }
            });

            render_enhanced_data_table(
                "services-table",
                "Filter services...",
                &filter_groups,
                &columns,
                visible_count,
                &table_state,
                theme,
                callbacks,
                enhancements,
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
    use crate::components::data_table::{ColumnAlign, SortDirection};
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
                "edit",
                "port-forward",
                "delete-service",
            ]
        );
        let delete_item = entries
            .iter()
            .find_map(|e| match e {
                ContextMenuEntry::Item(item) if item.id.as_ref() == "delete-service" => Some(item),
                _ => None,
            })
            .expect("delete-service item");
        assert!(delete_item.danger);
        assert!(!delete_item.disabled);
        assert!(delete_item.on_select.is_some());
    }

    #[test]
    fn test_filter_events_for_service_matches_service_events() {
        let svc = make_service("api-svc", "prod", "ClusterIP", "10.96.0.1", "-", "80/TCP", "1 ready", "1m");
        let events = vec![
            EventInfo {
                uid: "e1".to_string(),
                name: String::new(),
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
                name: String::new(),
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

    #[test]
    fn test_filter_and_sort_services_applies_search_query_and_type_extra_filter() {
        let cols = service_columns();
        let name_col = cols.iter().find(|c| c.id.as_ref() == "name").expect("name col");
        assert!(name_col.locked);

        let svcs = vec![
            make_service("api", "default", "ClusterIP", "10.0.0.1", "-", "80/TCP", "1", "1m"),
            make_service("api-lb", "default", "LoadBalancer", "10.0.0.2", "1.2.3.4", "443/TCP", "1", "2m"),
            make_service("web-lb", "default", "LoadBalancer", "10.0.0.3", "1.2.3.5", "80/TCP", "1", "3m"),
        ];
        let ns = NamespaceFilterState::default();
        let mut filters = crate::components::TableFilterState {
            search_query: "api".to_string(),
            ..Default::default()
        };
        filters
            .extra_filters
            .insert("type".to_string(), "LoadBalancer".to_string());

        let matched = filter_and_sort_services_with_filters(
            &svcs,
            &ns,
            ServiceTypeFilter::All,
            &filters,
            None,
        );
        assert_eq!(matched.len(), 1);
        assert_eq!(matched[0].name, "api-lb");
    }

    #[test]
    fn services_cached_rows_reflect_live_endpoint_update() {
        use orbit_core::kubernetes::services::ServiceEndpointsSnapshot;
        use orbit_core::kubernetes::watchers::{WatchAction, WatchItem, WatchUpdate};

        let mut state = ServicesState::default();
        let _ = state.begin_load_for_cluster("ctx-a");
        state.watch_generation = 1;

        assert!(state.apply_watch_update(
            "ctx-a",
            1,
            WatchUpdate::Reset(vec![(
                "default/api".to_string(),
                make_service("api", "default", "ClusterIP", "10.0.0.1", "-", "80/TCP", "-", "1m"),
            )]),
        ));

        let ns = NamespaceFilterState::default();
        let before = cached_filtered_services(&state, &ns);
        assert_eq!(before[0].endpoints, "-");

        assert!(state.apply_endpoints_watch_update(
            "ctx-a",
            1,
            WatchUpdate::Delta(vec![WatchItem {
                key: "default/api".to_string(),
                action: WatchAction::Applied,
                item: ServiceEndpointsSnapshot {
                    endpoints: "3 ready".to_string(),
                    endpoints_list: vec![
                        "10.0.0.1:80".to_string(),
                        "10.0.0.2:80".to_string(),
                        "10.0.0.3:80".to_string(),
                    ],
                },
            }]),
        ));

        let after = cached_filtered_services(&state, &ns);
        assert!(!Rc::ptr_eq(&before, &after));
        assert_eq!(after[0].endpoints, "3 ready");
    }
}
