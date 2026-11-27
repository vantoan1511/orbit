use std::cmp::Ordering;
use std::rc::Rc;

use gpui::{
    AnyElement, App, ClickEvent, ElementId, FontWeight, MouseButton, MouseDownEvent, SharedString,
    Window, div, prelude::*, px,
};
use lucide_icons::Icon;
use orbit_core::kubernetes::port_forward::PersistedPortForward;

use crate::components::data_table::{
    DataTableCallbacks, SortState, TableColumn, render_data_table, sort_rows_stable,
};
use crate::components::{
    ContextMenuEntry, ContextMenuItem, SemanticStatus, StatusBadgeVariant, TextTooltip,
    parse_k8s_resource_kind, render_status_badge,
};
use crate::icons::render_icon;
use crate::state::{
    NamespaceFilterState, PortForwardKindFilter, PortForwardsLoadStatus, PortForwardsState,
};
use crate::theme::{OrbitTheme, apply_text_step};
use crate::views::{
    render_resource_error_banner, render_resource_view_header, render_truncated_cell,
    table_numeric_font,
};

pub fn format_port_forward_url(local_port: u16) -> String {
    format!("http://127.0.0.1:{local_port}")
}

pub fn port_forward_columns() -> Vec<TableColumn> {
    vec![
        TableColumn::new("select", "")
            .fixed_width(px(36.0))
            .sortable(false),
        TableColumn::new("name", "Resource").flex_width(1.8, px(160.0)),
        TableColumn::new("namespace", "Namespace").flex_width(1.0, px(110.0)),
        TableColumn::new("kind", "Kind").fixed_width(px(110.0)),
        TableColumn::new("local_port", "Local Port")
            .fixed_width(px(150.0))
            .right_numeric(),
        TableColumn::new("remote_port", "Remote Port")
            .fixed_width(px(110.0))
            .right_numeric(),
        TableColumn::new("status", "Status").fixed_width(px(115.0)),
        TableColumn::new("actions", "Actions")
            .fixed_width(px(108.0))
            .sortable(false),
    ]
}

pub fn compare_port_forwards_by_col(
    a: &PersistedPortForward,
    b: &PersistedPortForward,
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
            .then_with(|| a.local_port.cmp(&b.local_port))
    };

    match col_id {
        "name" => tie_break(),
        "namespace" => a
            .namespace
            .to_ascii_lowercase()
            .cmp(&b.namespace.to_ascii_lowercase())
            .then_with(tie_break),
        "kind" => a
            .kind
            .to_ascii_lowercase()
            .cmp(&b.kind.to_ascii_lowercase())
            .then_with(tie_break),
        "local_port" => a.local_port.cmp(&b.local_port).then_with(tie_break),
        "remote_port" => a.remote_port.cmp(&b.remote_port).then_with(tie_break),
        "status" => Ordering::Equal.then_with(tie_break),
        _ => tie_break(),
    }
}

pub fn filter_and_sort_port_forwards<'a>(
    forwards: &'a [PersistedPortForward],
    namespaces: &NamespaceFilterState,
    kind_filter: PortForwardKindFilter,
    search_query: &str,
    sort: Option<&SortState>,
) -> Vec<&'a PersistedPortForward> {
    let query = search_query.trim().to_ascii_lowercase();
    let mut rows: Vec<&'a PersistedPortForward> = forwards
        .iter()
        .filter(|pf| {
            namespaces.matches_resource_namespace(Some(&pf.namespace))
                && kind_filter.matches_kind(&pf.kind)
                && (query.is_empty()
                    || pf.name.to_ascii_lowercase().contains(&query)
                    || pf.namespace.to_ascii_lowercase().contains(&query)
                    || pf.kind.to_ascii_lowercase().contains(&query)
                    || pf.local_port.to_string().contains(&query)
                    || pf.remote_port.to_string().contains(&query))
        })
        .collect();
    sort_rows_stable(&mut rows, sort, |a, b, col_id| {
        compare_port_forwards_by_col(a, b, col_id)
    });
    rows
}

pub fn build_port_forward_context_menu_entries(
    pf: &PersistedPortForward,
    on_open_browser: impl 'static + Fn(&mut Window, &mut App),
    on_copy_url: impl 'static + Fn(&mut Window, &mut App),
    on_stop: impl 'static + Fn(&mut Window, &mut App),
) -> Vec<ContextMenuEntry> {
    vec![
        ContextMenuEntry::Item(
            ContextMenuItem::new(
                "open-browser",
                format!("Open in Browser ({})", format_port_forward_url(pf.local_port)),
            )
            .icon(Icon::ExternalLink)
            .on_select(on_open_browser),
        ),
        ContextMenuEntry::Item(
            ContextMenuItem::new("copy-url", "Copy Local URL")
                .icon(Icon::Copy)
                .on_select(on_copy_url),
        ),
        ContextMenuEntry::Separator,
        ContextMenuEntry::Item(
            ContextMenuItem::new("stop-port-forward", "Stop Port Forward")
                .icon(Icon::Square)
                .danger(true)
                .on_select(on_stop),
        ),
    ]
}

pub type PortForwardActionCallback = Rc<dyn Fn(&mut Window, &mut App)>;
pub type PortForwardValueCallback<T> = Rc<dyn Fn(T, &mut Window, &mut App)>;
pub type PortForwardSortCallback = Rc<dyn Fn(SharedString, &ClickEvent, &mut Window, &mut App)>;
pub type PortForwardRowClickCallback =
    Rc<dyn Fn(usize, SharedString, &ClickEvent, &mut Window, &mut App)>;
pub type PortForwardContextMenuCallback =
    Rc<dyn Fn(SharedString, &PersistedPortForward, &MouseDownEvent, &mut Window, &mut App)>;

pub struct PortForwardViewCallbacks {
    pub on_sort: PortForwardSortCallback,
    pub on_row_click: PortForwardRowClickCallback,
    pub on_row_context_menu: PortForwardContextMenuCallback,
    pub on_toggle_select: PortForwardValueCallback<String>,
    pub on_toggle_select_all: PortForwardValueCallback<Vec<String>>,
    pub on_kind_filter: PortForwardValueCallback<PortForwardKindFilter>,
    pub on_focus_search: PortForwardValueCallback<bool>,
    pub on_clear_search: PortForwardActionCallback,
    pub on_open_create_modal: PortForwardActionCallback,
    pub on_open_browser: PortForwardValueCallback<u16>,
    pub on_copy_url: PortForwardValueCallback<u16>,
    pub on_stop_tunnel: PortForwardValueCallback<String>,
    pub on_stop_selected: PortForwardActionCallback,
    pub on_stop_all: PortForwardActionCallback,
    pub on_refresh: PortForwardActionCallback,
}

pub fn render_port_forward_view(
    state: &PortForwardsState,
    namespaces: &NamespaceFilterState,
    theme: &OrbitTheme,
    callbacks: PortForwardViewCallbacks,
) -> AnyElement {
    let columns = port_forward_columns();
    let filtered_refs = filter_and_sort_port_forwards(
        &state.port_forwards,
        namespaces,
        state.kind_filter,
        &state.search_query,
        state.table.sort.as_ref(),
    );
    let filtered_rows: Rc<Vec<PersistedPortForward>> =
        Rc::new(filtered_refs.into_iter().cloned().collect());
    let visible_ids: Vec<String> = filtered_rows.iter().map(|pf| pf.id.clone()).collect();
    let total_count = state.port_forwards.len();
    let visible_count = filtered_rows.len();
    let is_loading = state.status == PortForwardsLoadStatus::Loading;

    let count_label = if namespaces.is_all_namespaces()
        && state.kind_filter == PortForwardKindFilter::All
        && state.search_query.trim().is_empty()
    {
        format!("{total_count} tunnels")
    } else {
        format!("{visible_count} of {total_count} tunnels")
    };

    let selected_visible_count = visible_ids
        .iter()
        .filter(|id| state.selected_ids.contains(*id))
        .count();
    let all_visible_selected = !visible_ids.is_empty() && selected_visible_count == visible_ids.len();

    let toolbar = render_port_forward_toolbar(
        state,
        all_visible_selected,
        &visible_ids,
        theme,
        &callbacks,
    );

    let header = render_resource_view_header(count_label, Some(namespaces), Some(toolbar), theme);

    let on_refresh_rc = callbacks.on_refresh.clone();
    let error_banner = state.error_message.as_ref().map(|message| {
        render_resource_error_banner(
            "port-forward-retry-btn",
            format!("Port forward error: {message}"),
            is_loading,
            theme,
            on_refresh_rc,
        )
    });

    let mut table_state = state.table.clone();
    table_state.loading = is_loading && state.port_forwards.is_empty();
    table_state.refreshing = is_loading && !state.port_forwards.is_empty();
    table_state.empty_message = SharedString::from(if state.port_forwards.is_empty() {
        "No active port forwards. Click '+ Forward Port' or right-click a Pod, Service, or Workload."
    } else {
        "No port forwards match the current filter."
    });

    let stopping_ids = state.stopping_ids.clone();
    let selected_ids = state.selected_ids.clone();

    let on_sort_cb = callbacks.on_sort.clone();
    let on_row_click_cb = callbacks.on_row_click.clone();
    let on_row_ctx_cb = callbacks.on_row_context_menu.clone();
    let cell_callbacks = PortForwardCellCallbacks {
        on_toggle_select: callbacks.on_toggle_select.clone(),
        on_open_browser: callbacks.on_open_browser.clone(),
        on_copy_url: callbacks.on_copy_url.clone(),
        on_stop_tunnel: callbacks.on_stop_tunnel.clone(),
    };

    let rows_for_key = filtered_rows.clone();
    let rows_for_ctx = filtered_rows.clone();
    let rows_for_cell = filtered_rows.clone();

    let dt_callbacks = DataTableCallbacks::new(
        move |idx| {
            rows_for_key
                .get(idx)
                .map(|pf| pf.id.clone())
                .unwrap_or_default()
        },
        move |row_idx, col, theme| {
            let Some(pf) = rows_for_cell.get(row_idx) else {
                return div().into_any_element();
            };
            let is_stopping = stopping_ids.contains(&pf.id);
            let is_selected = selected_ids.contains(&pf.id);
            render_port_forward_cell(
                row_idx,
                pf,
                col,
                is_selected,
                is_stopping,
                theme,
                &cell_callbacks,
            )
        },
    )
    .on_sort(move |col_id, event, window, cx| {
        on_sort_cb(col_id, event, window, cx);
    })
    .on_row_click(move |row_idx, key, event, window, cx| {
        on_row_click_cb(row_idx, key, event, window, cx);
    })
    .on_row_context_menu(move |row_idx, key, event, window, cx| {
        if let Some(pf) = rows_for_ctx.get(row_idx) {
            on_row_ctx_cb(key, pf, event, window, cx);
        }
    });

    let table = render_data_table(
        "port-forward-table",
        &columns,
        filtered_rows.len(),
        &table_state,
        theme,
        dt_callbacks,
    );

    div()
        .size_full()
        .flex()
        .flex_col()
        .bg(theme.surface.bg_app)
        .child(header)
        .when_some(error_banner, |el, banner| {
            el.child(div().px_4().pb_2().flex_shrink_0().child(banner))
        })
        .child(div().flex_1().min_h(px(0.0)).px_4().pb_4().child(table))
        .into_any_element()
}

fn render_port_forward_toolbar(
    state: &PortForwardsState,
    all_visible_selected: bool,
    visible_ids: &[String],
    theme: &OrbitTheme,
    callbacks: &PortForwardViewCallbacks,
) -> AnyElement {
    let on_create = callbacks.on_open_create_modal.clone();
    let on_kind_filter = callbacks.on_kind_filter.clone();
    let on_focus_search = callbacks.on_focus_search.clone();
    let on_clear_search = callbacks.on_clear_search.clone();
    let on_toggle_all = callbacks.on_toggle_select_all.clone();
    let on_stop_selected = callbacks.on_stop_selected.clone();
    let on_stop_all = callbacks.on_stop_all.clone();
    let visible_ids_owned = visible_ids.to_vec();

    let kind_chips = div()
        .flex()
        .items_center()
        .gap_1()
        .children(PortForwardKindFilter::all().iter().copied().map(|filter| {
            let active = state.kind_filter == filter;
            let cb = on_kind_filter.clone();
            let hover_bg = theme.surface.control_hover;
            div()
                .id(SharedString::from(format!("pf-kind-filter-{}", filter.label())))
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
                .on_click(move |_, window, cx| cb(filter, window, cx))
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

    let search_box = div()
        .id("pf-search-input")
        .h(px(26.0))
        .w(px(200.0))
        .px_2()
        .rounded(theme.style.radius_control)
        .bg(theme.surface.input_bg)
        .border_1()
        .border_color(if state.search_focused {
            theme.accent.accent
        } else {
            theme.border.input
        })
        .flex()
        .items_center()
        .gap_1p5()
        .cursor_pointer()
        .on_mouse_down(MouseButton::Left, move |_, window, cx| {
            window.prevent_default();
            cx.stop_propagation();
            on_focus_search(true, window, cx);
        })
        .child(render_icon(Icon::Search, px(12.0), theme.text.muted))
        .child(
            apply_text_step(div(), theme.typography.size_xs)
                .flex_1()
                .min_w(px(0.0))
                .truncate()
                .text_color(if state.search_query.is_empty() {
                    theme.text.muted
                } else {
                    theme.text.primary
                })
                .child(if state.search_query.is_empty() {
                    SharedString::from("Filter tunnels...")
                } else {
                    SharedString::from(state.search_query.clone())
                }),
        )
        .when(!state.search_query.is_empty(), |el| {
            el.child(
                div()
                    .id("pf-search-clear")
                    .w(px(16.0))
                    .h(px(16.0))
                    .rounded(theme.style.radius_control)
                    .flex()
                    .items_center()
                    .justify_center()
                    .cursor_pointer()
                    .hover(|s| s.bg(theme.surface.bg_hover))
                    .on_mouse_down(MouseButton::Left, move |_, window, cx| {
                        window.prevent_default();
                        cx.stop_propagation();
                        on_clear_search(window, cx);
                    })
                    .child(render_icon(Icon::X, px(11.0), theme.text.muted)),
            )
        });

    let selected_count = state.selected_ids.len();

    div()
        .flex()
        .items_center()
        .flex_wrap()
        .gap_2()
        .child(kind_chips)
        .child(search_box)
        .when(!visible_ids.is_empty(), |el| {
            el.child(
                div()
                    .id("pf-select-all-btn")
                    .h(px(26.0))
                    .px_2()
                    .rounded(theme.style.radius_control)
                    .border_1()
                    .border_color(theme.border.default)
                    .bg(theme.surface.control_bg)
                    .flex()
                    .items_center()
                    .gap_1()
                    .cursor_pointer()
                    .hover(|s| s.bg(theme.surface.control_hover))
                    .on_click(move |_, window, cx| {
                        on_toggle_all(visible_ids_owned.clone(), window, cx);
                    })
                    .child(
                        apply_text_step(div(), theme.typography.size_2xs)
                            .font_weight(FontWeight::MEDIUM)
                            .text_color(theme.text.secondary)
                            .child(if all_visible_selected {
                                "Deselect All"
                            } else {
                                "Select All"
                            }),
                    ),
            )
        })
        .when(selected_count > 0, |el| {
            el.child(
                div()
                    .id("pf-stop-selected-btn")
                    .h(px(26.0))
                    .px_2p5()
                    .rounded(theme.style.radius_control)
                    .bg(theme.semantic.danger.soft)
                    .border_1()
                    .border_color(theme.semantic.danger.solid)
                    .flex()
                    .items_center()
                    .gap_1()
                    .cursor_pointer()
                    .on_click(move |_, window, cx| on_stop_selected(window, cx))
                    .child(render_icon(
                        Icon::Square,
                        px(11.0),
                        theme.semantic.danger.solid,
                    ))
                    .child(
                        apply_text_step(div(), theme.typography.size_2xs)
                            .font_weight(FontWeight::SEMIBOLD)
                            .text_color(theme.semantic.danger.solid)
                            .child(format!("Stop Selected ({selected_count})")),
                    ),
            )
        })
        .when(!state.port_forwards.is_empty(), |el| {
            el.child(
                div()
                    .id("pf-stop-all-btn")
                    .h(px(26.0))
                    .px_2p5()
                    .rounded(theme.style.radius_control)
                    .bg(theme.semantic.danger.soft)
                    .border_1()
                    .border_color(theme.semantic.danger.solid)
                    .flex()
                    .items_center()
                    .gap_1()
                    .cursor_pointer()
                    .on_click(move |_, window, cx| on_stop_all(window, cx))
                    .child(render_icon(
                        Icon::Trash2,
                        px(11.0),
                        theme.semantic.danger.solid,
                    ))
                    .child(
                        apply_text_step(div(), theme.typography.size_2xs)
                            .font_weight(FontWeight::SEMIBOLD)
                            .text_color(theme.semantic.danger.solid)
                            .child("Stop All"),
                    ),
            )
        })
        .child(
            div()
                .id("pf-create-btn")
                .h(px(26.0))
                .px_2p5()
                .rounded(theme.style.radius_control)
                .bg(theme.accent.accent)
                .flex()
                .items_center()
                .gap_1()
                .cursor_pointer()
                .hover(|s| s.bg(theme.accent.hover))
                .on_click(move |_, window, cx| on_create(window, cx))
                .child(render_icon(Icon::Plus, px(12.0), gpui::white().into()))
                .child(
                    apply_text_step(div(), theme.typography.size_2xs)
                        .font_weight(FontWeight::SEMIBOLD)
                        .text_color(gpui::white())
                        .child("Forward Port"),
                ),
        )
        .into_any_element()
}

struct PortForwardCellCallbacks {
    on_toggle_select: PortForwardValueCallback<String>,
    on_open_browser: PortForwardValueCallback<u16>,
    on_copy_url: PortForwardValueCallback<u16>,
    on_stop_tunnel: PortForwardValueCallback<String>,
}

fn render_port_forward_cell(
    row_idx: usize,
    pf: &PersistedPortForward,
    col: &TableColumn,
    is_selected: bool,
    is_stopping: bool,
    theme: &OrbitTheme,
    callbacks: &PortForwardCellCallbacks,
) -> AnyElement {
    let on_toggle_select = &callbacks.on_toggle_select;
    let on_open_browser = &callbacks.on_open_browser;
    let on_copy_url = &callbacks.on_copy_url;
    let on_stop_tunnel = &callbacks.on_stop_tunnel;
    match col.id.as_ref() {
        "select" => {
            let cb = on_toggle_select.clone();
            let id_owned = pf.id.clone();
            div()
                .id(ElementId::NamedInteger(
                    SharedString::from("pf-select"),
                    row_idx as u64,
                ))
                .w(px(16.0))
                .h(px(16.0))
                .rounded(theme.style.radius_control)
                .border_1()
                .border_color(if is_selected {
                    theme.accent.accent
                } else {
                    theme.border.strong
                })
                .bg(if is_selected {
                    theme.accent.accent
                } else {
                    theme.surface.input_bg
                })
                .flex()
                .items_center()
                .justify_center()
                .cursor_pointer()
                .on_mouse_down(MouseButton::Left, move |_, window, cx| {
                    window.prevent_default();
                    cx.stop_propagation();
                    cb(id_owned.clone(), window, cx);
                })
                .when(is_selected, |el| {
                    el.child(render_icon(Icon::Check, px(11.0), gpui::white().into()))
                })
                .into_any_element()
        }
        "name" => render_truncated_cell(
            ElementId::NamedInteger(SharedString::from("pf-name"), row_idx as u64),
            pf.name.clone(),
            None,
            theme.text.primary,
            FontWeight::MEDIUM,
            false,
            theme,
        ),
        "namespace" => render_truncated_cell(
            ElementId::NamedInteger(SharedString::from("pf-ns"), row_idx as u64),
            pf.namespace.clone(),
            None,
            theme.text.secondary,
            FontWeight::NORMAL,
            false,
            theme,
        ),
        "kind" => {
            let kind_color = parse_k8s_resource_kind(&pf.kind)
                .map(|k| theme.k8s_color(k))
                .unwrap_or(theme.text.secondary);
            apply_text_step(div(), theme.typography.size_xs)
                .font_weight(FontWeight::MEDIUM)
                .text_color(kind_color)
                .child(pf.kind.clone())
                .into_any_element()
        }
        "local_port" => apply_text_step(div(), theme.typography.size_xs)
            .font(table_numeric_font(theme))
            .text_color(theme.text.primary)
            .child(format!("127.0.0.1:{}", pf.local_port))
            .into_any_element(),
        "remote_port" => apply_text_step(div(), theme.typography.size_xs)
            .font(table_numeric_font(theme))
            .text_color(theme.text.secondary)
            .child(pf.remote_port.to_string())
            .into_any_element(),
        "status" => {
            if is_stopping {
                render_status_badge(
                    "Stopping",
                    SemanticStatus::Warning,
                    StatusBadgeVariant::Pill,
                    theme,
                )
            } else {
                render_status_badge(
                    "Forwarding",
                    SemanticStatus::Success,
                    StatusBadgeVariant::Pill,
                    theme,
                )
            }
        }
        "actions" => {
            let local_port = pf.local_port;
            let pf_id = pf.id.clone();
            let open_cb = on_open_browser.clone();
            let copy_cb = on_copy_url.clone();
            let stop_cb = on_stop_tunnel.clone();

            div()
                .flex()
                .items_center()
                .gap_1()
                .child(render_inline_icon_action(
                    ElementId::NamedInteger(SharedString::from("pf-open"), row_idx as u64),
                    Icon::ExternalLink,
                    "Open in Browser",
                    theme.text.secondary,
                    theme,
                    move |window, cx| open_cb(local_port, window, cx),
                ))
                .child(render_inline_icon_action(
                    ElementId::NamedInteger(SharedString::from("pf-copy"), row_idx as u64),
                    Icon::Copy,
                    "Copy Local URL",
                    theme.text.secondary,
                    theme,
                    move |window, cx| copy_cb(local_port, window, cx),
                ))
                .child(render_inline_icon_action(
                    ElementId::NamedInteger(SharedString::from("pf-stop"), row_idx as u64),
                    Icon::Square,
                    "Stop Port Forward",
                    theme.semantic.danger.solid,
                    theme,
                    move |window, cx| stop_cb(pf_id.clone(), window, cx),
                ))
                .into_any_element()
        }
        _ => div().into_any_element(),
    }
}

fn render_inline_icon_action(
    id: ElementId,
    icon: Icon,
    tooltip: &'static str,
    color: gpui::Rgba,
    theme: &OrbitTheme,
    on_click: impl 'static + Fn(&mut Window, &mut App),
) -> AnyElement {
    div()
        .id(id)
        .w(px(24.0))
        .h(px(24.0))
        .rounded(theme.style.radius_control)
        .flex()
        .items_center()
        .justify_center()
        .cursor_pointer()
        .hover(|s| s.bg(theme.surface.bg_hover))
        .tooltip(move |_window, cx| cx.new(|_| TextTooltip::new(tooltip)).into())
        .on_mouse_down(MouseButton::Left, move |_, window, cx| {
            window.prevent_default();
            cx.stop_propagation();
            on_click(window, cx);
        })
        .child(render_icon(icon, px(13.0), color))
        .into_any_element()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::components::data_table::{ColumnAlign, SortDirection};

    fn sample_pf(
        namespace: &str,
        kind: &str,
        name: &str,
        local_port: u16,
        remote_port: u16,
    ) -> PersistedPortForward {
        PersistedPortForward {
            id: format!("{namespace}/{kind}/{name}:{local_port}:{remote_port}"),
            cluster_id: "ctx-a".to_string(),
            namespace: namespace.to_string(),
            kind: kind.to_string(),
            name: name.to_string(),
            local_port,
            remote_port,
        }
    }

    #[test]
    fn test_port_forward_columns_match_ac_and_numeric_alignment() {
        let cols = port_forward_columns();
        let headers: Vec<&str> = cols.iter().map(|c| c.header.as_ref()).collect();
        assert_eq!(
            headers,
            vec![
                "",
                "Resource",
                "Namespace",
                "Kind",
                "Local Port",
                "Remote Port",
                "Status",
                "Actions",
            ]
        );
        let local_col = cols.iter().find(|c| c.id == "local_port");
        let remote_col = cols.iter().find(|c| c.id == "remote_port");
        assert!(local_col.is_some_and(|c| c.align == ColumnAlign::Right && c.monospace));
        assert!(remote_col.is_some_and(|c| c.align == ColumnAlign::Right && c.monospace));
    }

    #[test]
    fn test_filter_and_sort_port_forwards_by_kind_namespace_search_and_port() {
        let forwards = vec![
            sample_pf("default", "Service", "api", 8080, 80),
            sample_pf("prod", "Pod", "worker-0", 9090, 90),
            sample_pf("default", "Deployment", "web", 3000, 3000),
        ];
        let mut ns = NamespaceFilterState::default();
        let sort = SortState {
            column_id: "local_port".to_string(),
            direction: SortDirection::Asc,
        };

        let all_sorted = filter_and_sort_port_forwards(
            &forwards,
            &ns,
            PortForwardKindFilter::All,
            "",
            Some(&sort),
        );
        assert_eq!(
            all_sorted.iter().map(|p| p.local_port).collect::<Vec<_>>(),
            vec![3000, 8080, 9090]
        );

        let svc_only = filter_and_sort_port_forwards(
            &forwards,
            &ns,
            PortForwardKindFilter::Service,
            "",
            None,
        );
        assert_eq!(svc_only.len(), 1);
        assert_eq!(svc_only[0].name, "api");

        let port_search = filter_and_sort_port_forwards(
            &forwards,
            &ns,
            PortForwardKindFilter::All,
            "9090",
            None,
        );
        assert_eq!(port_search.len(), 1);
        assert_eq!(port_search[0].name, "worker-0");

        ns.cluster_id = Some("ctx-a".to_string());
        ns.toggle_namespace("default");
        let ns_filtered = filter_and_sort_port_forwards(
            &forwards,
            &ns,
            PortForwardKindFilter::All,
            "",
            None,
        );
        assert_eq!(ns_filtered.len(), 2);
    }

    #[test]
    fn test_build_port_forward_context_menu_entries_contains_open_copy_and_stop() {
        let pf = sample_pf("default", "Service", "api", 8080, 80);
        let entries = build_port_forward_context_menu_entries(&pf, |_, _| {}, |_, _| {}, |_, _| {});
        let ids: Vec<String> = entries
            .iter()
            .filter_map(|e| match e {
                ContextMenuEntry::Item(item) => Some(item.id.to_string()),
                ContextMenuEntry::Separator => None,
            })
            .collect();
        assert_eq!(ids, vec!["open-browser", "copy-url", "stop-port-forward"]);
    }

    #[test]
    fn test_port_forward_local_url_formatting() {
        assert_eq!(format_port_forward_url(8080), "http://127.0.0.1:8080");
    }
}
