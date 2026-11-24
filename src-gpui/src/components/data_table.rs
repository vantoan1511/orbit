use std::cmp::Ordering;
use std::ops::Range;
use std::rc::Rc;

use gpui::{
    AnyElement, App, ClickEvent, Div, ElementId, Font, FontWeight, InteractiveElement, IntoElement,
    MouseButton, MouseDownEvent, Pixels, Rgba, SharedString, StatefulInteractiveElement, Styled,
    UniformListScrollHandle, Window, div, prelude::*, px, uniform_list,
};
use lucide_icons::Icon;

use crate::components::tooltip::TextTooltip;
use crate::icons::render_icon;
use crate::theme::{OrbitTheme, apply_text_step};

pub const TABLE_HEADER_HEIGHT: f32 = 32.0;
pub const TABLE_ROW_HEIGHT: f32 = 36.0;
pub const DEFAULT_SKELETON_ROWS: usize = 6;
pub const DEFAULT_EMPTY_MESSAGE: &str = "No resources found.";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ColumnAlign {
    #[default]
    Left,
    Right,
    Center,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ColumnWidth {
    Fixed(Pixels),
    Flex { weight: f32, min_width: Pixels },
}

impl Default for ColumnWidth {
    fn default() -> Self {
        Self::Flex {
            weight: 1.0,
            min_width: px(120.0),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortDirection {
    Asc,
    Desc,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SortState {
    pub column_id: String,
    pub direction: SortDirection,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TableColumn {
    pub id: SharedString,
    pub header: SharedString,
    pub width: ColumnWidth,
    pub align: ColumnAlign,
    pub sortable: bool,
    pub monospace: bool,
}

impl TableColumn {
    pub fn new(id: impl Into<SharedString>, header: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            header: header.into(),
            width: ColumnWidth::default(),
            align: ColumnAlign::Left,
            sortable: true,
            monospace: false,
        }
    }

    pub fn fixed_width(mut self, width: Pixels) -> Self {
        self.width = ColumnWidth::Fixed(width);
        self
    }

    pub fn flex_width(mut self, weight: f32, min_width: Pixels) -> Self {
        self.width = ColumnWidth::Flex { weight, min_width };
        self
    }

    pub fn align(mut self, align: ColumnAlign) -> Self {
        self.align = align;
        self
    }

    pub fn right_numeric(mut self) -> Self {
        self.align = ColumnAlign::Right;
        self.monospace = true;
        self
    }

    pub fn sortable(mut self, sortable: bool) -> Self {
        self.sortable = sortable;
        self
    }

    pub fn monospace(mut self, monospace: bool) -> Self {
        self.monospace = monospace;
        self
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TableBodyMode {
    LoadingSkeleton,
    Empty,
    Rows,
}

#[derive(Debug, Clone)]
pub struct DataTableState {
    pub sort: Option<SortState>,
    pub selected_row_key: Option<String>,
    pub scroll_handle: UniformListScrollHandle,
    pub loading: bool,
    pub empty_message: SharedString,
}

impl Default for DataTableState {
    fn default() -> Self {
        Self::new()
    }
}

impl DataTableState {
    pub fn new() -> Self {
        Self {
            sort: None,
            selected_row_key: None,
            scroll_handle: UniformListScrollHandle::new(),
            loading: false,
            empty_message: SharedString::from(DEFAULT_EMPTY_MESSAGE),
        }
    }

    pub fn with_empty_message(mut self, message: impl Into<SharedString>) -> Self {
        self.empty_message = message.into();
        self
    }

    pub fn toggle_sort(&mut self, column_id: &str) {
        self.sort = match &self.sort {
            Some(current) if current.column_id == column_id => match current.direction {
                SortDirection::Asc => Some(SortState {
                    column_id: column_id.to_string(),
                    direction: SortDirection::Desc,
                }),
                SortDirection::Desc => None,
            },
            _ => Some(SortState {
                column_id: column_id.to_string(),
                direction: SortDirection::Asc,
            }),
        };
    }

    pub fn select_row(&mut self, row_key: impl Into<String>) {
        self.selected_row_key = Some(row_key.into());
    }

    pub fn clear_selection(&mut self) {
        self.selected_row_key = None;
    }

    pub fn is_selected(&self, row_key: &str) -> bool {
        self.selected_row_key.as_deref() == Some(row_key)
    }

    pub fn body_mode(&self, row_count: usize) -> TableBodyMode {
        if self.loading && row_count == 0 {
            TableBodyMode::LoadingSkeleton
        } else if row_count == 0 {
            TableBodyMode::Empty
        } else {
            TableBodyMode::Rows
        }
    }

    pub fn sort_indicator(&self, column_id: &str) -> Option<Icon> {
        match &self.sort {
            Some(active) if active.column_id == column_id => match active.direction {
                SortDirection::Asc => Some(Icon::ArrowUp),
                SortDirection::Desc => Some(Icon::ArrowDown),
            },
            _ => Some(Icon::ArrowUpDown),
        }
    }

    pub fn sort_indicator_for_column(&self, col: &TableColumn) -> Option<Icon> {
        if !col.sortable {
            return None;
        }
        self.sort_indicator(col.id.as_ref())
    }
}

pub fn sort_rows_stable<T, F>(rows: &mut [T], sort: Option<&SortState>, mut cmp_by_col: F)
where
    F: FnMut(&T, &T, &str) -> Ordering,
{
    let Some(sort_state) = sort else {
        return;
    };
    let col_id = sort_state.column_id.as_str();
    let direction = sort_state.direction;
    rows.sort_by(|a, b| {
        let ord = cmp_by_col(a, b, col_id);
        match direction {
            SortDirection::Asc => ord,
            SortDirection::Desc => ord.reverse(),
        }
    });
}

pub fn apply_column_width<E: Styled>(mut el: E, width: ColumnWidth) -> E {
    match width {
        ColumnWidth::Fixed(w) => el.w(w).flex_shrink_0(),
        ColumnWidth::Flex { weight, min_width } => {
            el = el.min_w(min_width).flex_basis(px(0.0));
            el.style().flex_grow = Some(weight.max(0.0));
            el
        }
    }
}

pub fn apply_column_alignment<E: Styled>(el: E, align: ColumnAlign) -> E {
    match align {
        ColumnAlign::Left => el.justify_start().text_left(),
        ColumnAlign::Right => el.justify_end().text_right(),
        ColumnAlign::Center => el.justify_center().text_center(),
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TableHeaderStyle {
    pub text_color: Rgba,
    pub font_weight: FontWeight,
    pub icon_color: Rgba,
}

/// 3-tier ink contract for table headers:
/// - Inactive header: `theme.text.muted`, `FontWeight::MEDIUM`, muted sort indicator.
/// - Active sort header: `theme.text.primary`, `FontWeight::MEDIUM`, accent sort indicator.
pub fn resolve_table_header_style(is_active_sort: bool, theme: &OrbitTheme) -> TableHeaderStyle {
    if is_active_sort {
        TableHeaderStyle {
            text_color: theme.text.primary,
            font_weight: FontWeight::MEDIUM,
            icon_color: theme.accent.accent,
        }
    } else {
        TableHeaderStyle {
            text_color: theme.text.muted,
            font_weight: FontWeight::MEDIUM,
            icon_color: theme.text.muted,
        }
    }
}

/// Right-aligned columns place the sort icon BEFORE the header label so the label's
/// right edge aligns pixel-flush with right-aligned numeric cell digits below.
/// Left and Center columns place the sort icon AFTER the header label.
pub fn table_header_icon_before_label(align: ColumnAlign) -> bool {
    align == ColumnAlign::Right
}

/// Returns tabular monospace font (`FontWeight::NORMAL`, `tnum = 1`) for monospace/numeric
/// columns (`col.monospace == true`), or `None` to inherit standard UI typography.
pub fn resolve_table_cell_font(col: &TableColumn, theme: &OrbitTheme) -> Option<Font> {
    if col.monospace {
        Some(theme.typography.mono_font(FontWeight::NORMAL))
    } else {
        None
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DataTableSurfaceStyle {
    pub background: Rgba,
    pub header_background: Rgba,
    pub top_hairline: Rgba,
    pub header_separator: Rgba,
    pub row_separator: Rgba,
    pub uses_card_chrome: bool,
}

pub fn resolve_data_table_surface_style(theme: &OrbitTheme) -> DataTableSurfaceStyle {
    DataTableSurfaceStyle {
        background: theme.surface.bg_app,
        header_background: theme.surface.bg_panel,
        top_hairline: theme.border.default,
        header_separator: theme.border.default,
        row_separator: theme.border.subtle,
        uses_card_chrome: false,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RowHoverActionSpec {
    pub show_quick_yaml: bool,
    pub show_more_actions: bool,
}

impl RowHoverActionSpec {
    pub const fn is_visible(&self) -> bool {
        self.show_quick_yaml || self.show_more_actions
    }
}

pub const fn resolve_row_hover_actions(
    has_quick_yaml: bool,
    has_context_menu: bool,
) -> RowHoverActionSpec {
    RowHoverActionSpec {
        show_quick_yaml: has_quick_yaml,
        show_more_actions: has_context_menu,
    }
}

pub type SortCallback = Rc<dyn Fn(SharedString, &ClickEvent, &mut Window, &mut App)>;
pub type RowClickCallback = Rc<dyn Fn(usize, SharedString, &ClickEvent, &mut Window, &mut App)>;
pub type RowContextMenuCallback =
    Rc<dyn Fn(usize, SharedString, &MouseDownEvent, &mut Window, &mut App)>;

pub struct DataTableCallbacks<K, C>
where
    K: 'static + Fn(usize) -> String,
    C: 'static + Fn(usize, &TableColumn, &OrbitTheme) -> AnyElement,
{
    pub row_key: K,
    pub render_cell: C,
    pub on_sort: Option<SortCallback>,
    pub on_row_click: Option<RowClickCallback>,
    pub on_row_context_menu: Option<RowContextMenuCallback>,
    pub on_row_quick_yaml: Option<RowClickCallback>,
}

impl<K, C> DataTableCallbacks<K, C>
where
    K: 'static + Fn(usize) -> String,
    C: 'static + Fn(usize, &TableColumn, &OrbitTheme) -> AnyElement,
{
    pub fn new(row_key: K, render_cell: C) -> Self {
        Self {
            row_key,
            render_cell,
            on_sort: None,
            on_row_click: None,
            on_row_context_menu: None,
            on_row_quick_yaml: None,
        }
    }

    pub fn on_sort(
        mut self,
        callback: impl 'static + Fn(SharedString, &ClickEvent, &mut Window, &mut App),
    ) -> Self {
        self.on_sort = Some(Rc::new(callback));
        self
    }

    pub fn on_row_click(
        mut self,
        callback: impl 'static + Fn(usize, SharedString, &ClickEvent, &mut Window, &mut App),
    ) -> Self {
        self.on_row_click = Some(Rc::new(callback));
        self
    }

    pub fn on_row_context_menu(
        mut self,
        callback: impl 'static + Fn(usize, SharedString, &MouseDownEvent, &mut Window, &mut App),
    ) -> Self {
        self.on_row_context_menu = Some(Rc::new(callback));
        self
    }

    pub fn on_row_quick_yaml(
        mut self,
        callback: impl 'static + Fn(usize, SharedString, &ClickEvent, &mut Window, &mut App),
    ) -> Self {
        self.on_row_quick_yaml = Some(Rc::new(callback));
        self
    }
}

pub fn render_table_header(
    table_id: &str,
    columns: &[TableColumn],
    state: &DataTableState,
    theme: &OrbitTheme,
    on_sort: Option<SortCallback>,
) -> AnyElement {
    let surface = resolve_data_table_surface_style(theme);
    let hover_bg = theme.surface.bg_hover;
    let mut header_row = div()
        .id(SharedString::from(format!("{table_id}-header")))
        .h(px(TABLE_HEADER_HEIGHT))
        .flex_shrink_0()
        .w_full()
        .flex()
        .items_center()
        .bg(surface.header_background)
        .border_b_1()
        .border_color(surface.header_separator);

    for (col_idx, col) in columns.iter().enumerate() {
        let is_active_sort = state
            .sort
            .as_ref()
            .is_some_and(|s| s.column_id == col.id.as_ref());
        let sort_icon = state.sort_indicator_for_column(col);
        let col_id = col.id.clone();
        let sort_cb = on_sort.clone();
        let sortable = col.sortable;

        let mut cell = apply_column_alignment(
            apply_column_width(
                div().id(ElementId::NamedInteger(
                    SharedString::from(format!("{table_id}-hdr-col")),
                    col_idx as u64,
                )),
                col.width,
            ),
            col.align,
        )
        .h_full()
        .px_3()
        .flex()
        .items_center()
        .gap_1()
        .overflow_hidden()
        .when(sortable, |el| {
            el.cursor_pointer().hover(move |style| style.bg(hover_bg))
        });

        if sortable && let Some(cb) = sort_cb {
            cell = cell.on_click(move |event, window, cx| {
                cb(col_id.clone(), event, window, cx);
            });
        }

        let header_style = resolve_table_header_style(is_active_sort, theme);
        let label = apply_text_step(div(), theme.typography.size_xs)
            .font_weight(header_style.font_weight)
            .text_color(header_style.text_color)
            .child(col.header.clone());

        let icon_el = sort_icon.map(|icon| render_icon(icon, px(12.0), header_style.icon_color));

        if table_header_icon_before_label(col.align) {
            if let Some(icon_el) = icon_el {
                cell = cell.child(icon_el);
            }
            cell = cell.child(label);
        } else {
            cell = cell.child(label);
            if let Some(icon_el) = icon_el {
                cell = cell.child(icon_el);
            }
        }

        header_row = header_row.child(cell);
    }

    header_row.into_any_element()
}

pub fn render_table_cell(
    col: &TableColumn,
    theme: &OrbitTheme,
    content: impl IntoElement,
) -> Div {
    let cell_font = resolve_table_cell_font(col, theme);
    apply_column_alignment(
        apply_column_width(div().min_w(px(0.0)), col.width),
        col.align,
    )
    .h_full()
    .px_3()
    .flex()
    .items_center()
    .overflow_hidden()
    .text_size(theme.typography.size_xs.font_size)
    .line_height(theme.typography.size_xs.line_height)
    .text_color(theme.text.primary)
    .when_some(cell_font, |el, font| el.font(font))
    .child(content)
}

pub fn render_loading_skeleton(
    table_id: &str,
    columns: &[TableColumn],
    skeleton_rows: usize,
    theme: &OrbitTheme,
) -> AnyElement {
    let row_count = skeleton_rows.max(1);
    let mut container = div()
        .id(SharedString::from(format!("{table_id}-skeleton")))
        .flex()
        .flex_col()
        .flex_1()
        .w_full()
        .overflow_hidden();

    for row_idx in 0..row_count {
        let mut row = div()
            .id(ElementId::NamedInteger(
                SharedString::from(format!("{table_id}-skeleton-row")),
                row_idx as u64,
            ))
            .h(px(TABLE_ROW_HEIGHT))
            .flex_shrink_0()
            .w_full()
            .flex()
            .items_center()
            .border_b_1()
            .border_color(theme.border.subtle);

        for col in columns {
            let bar_width = match col.align {
                ColumnAlign::Right => px(44.0),
                ColumnAlign::Center => px(52.0),
                ColumnAlign::Left => px(96.0),
            };
            let placeholder = div()
                .h(px(10.0))
                .w(bar_width)
                .rounded(theme.style.radius_control)
                .bg(theme.surface.bg_hover);
            row = row.child(render_table_cell(col, theme, placeholder));
        }

        container = container.child(row);
    }

    container.into_any_element()
}

pub fn render_empty_state(
    table_id: &str,
    empty_message: SharedString,
    theme: &OrbitTheme,
) -> AnyElement {
    div()
        .id(SharedString::from(format!("{table_id}-empty")))
        .flex_1()
        .w_full()
        .min_h(px(160.0))
        .flex()
        .flex_col()
        .items_center()
        .justify_center()
        .gap_2()
        .p_6()
        .child(render_icon(Icon::Info, px(20.0), theme.text.muted))
        .child(
            apply_text_step(div(), theme.typography.size_xs)
                .font_weight(FontWeight::MEDIUM)
                .text_color(theme.text.muted)
                .child(empty_message),
        )
        .into_any_element()
}

pub fn render_data_table<K, C>(
    table_id: impl Into<SharedString>,
    columns: &[TableColumn],
    row_count: usize,
    state: &DataTableState,
    theme: &OrbitTheme,
    callbacks: DataTableCallbacks<K, C>,
) -> AnyElement
where
    K: 'static + Fn(usize) -> String,
    C: 'static + Fn(usize, &TableColumn, &OrbitTheme) -> AnyElement,
{
    let table_id_str = table_id.into();
    let header = render_table_header(
        table_id_str.as_ref(),
        columns,
        state,
        theme,
        callbacks.on_sort.clone(),
    );

    let body = match state.body_mode(row_count) {
        TableBodyMode::LoadingSkeleton => render_loading_skeleton(
            table_id_str.as_ref(),
            columns,
            DEFAULT_SKELETON_ROWS,
            theme,
        ),
        TableBodyMode::Empty => render_empty_state(
            table_id_str.as_ref(),
            state.empty_message.clone(),
            theme,
        ),
        TableBodyMode::Rows => {
            let cols: Rc<[TableColumn]> = Rc::from(columns.to_vec());
            let row_key_fn = Rc::new(callbacks.row_key);
            let render_cell_fn = Rc::new(callbacks.render_cell);
            let on_row_click = callbacks.on_row_click.clone();
            let on_row_context_menu = callbacks.on_row_context_menu.clone();
            let on_row_quick_yaml = callbacks.on_row_quick_yaml.clone();
            let hover_actions = resolve_row_hover_actions(
                on_row_quick_yaml.is_some(),
                on_row_context_menu.is_some(),
            );
            let selected_key = state.selected_row_key.clone();
            let theme_snapshot = theme.clone();
            let list_id = SharedString::from(format!("{table_id_str}-rows"));
            let row_prefix = SharedString::from(format!("{table_id_str}-row"));
            let row_group_prefix = SharedString::from(format!("{table_id_str}-row-grp"));
            let yaml_btn_prefix = SharedString::from(format!("{table_id_str}-row-yaml"));
            let more_btn_prefix = SharedString::from(format!("{table_id_str}-row-more"));

            uniform_list(
                ElementId::Name(list_id),
                row_count,
                move |range: Range<usize>, _window: &mut Window, _cx: &mut App| {
                    let mut items = Vec::with_capacity(range.len());
                    let surface = resolve_data_table_surface_style(&theme_snapshot);
                    let hover_bg = theme_snapshot.surface.bg_hover;
                    let selected_bg = theme_snapshot.accent.soft;
                    let border_subtle = surface.row_separator;
                    let accent_bar = theme_snapshot.accent.accent;

                    for row_idx in range {
                        let key = (row_key_fn)(row_idx);
                        let is_selected = selected_key.as_deref() == Some(key.as_str());
                        let key_shared = SharedString::from(key);
                        let row_group = SharedString::from(format!("{row_group_prefix}-{row_idx}"));

                        let mut row_el = div()
                            .id(ElementId::NamedInteger(row_prefix.clone(), row_idx as u64))
                            .group(row_group.clone())
                            .relative()
                            .h(px(TABLE_ROW_HEIGHT))
                            .w_full()
                            .flex()
                            .items_center()
                            .border_b_1()
                            .border_color(border_subtle)
                            .cursor_pointer()
                            .when(is_selected, |el| el.bg(selected_bg))
                            .hover(move |style| {
                                style.bg(if is_selected { selected_bg } else { hover_bg })
                            })
                            .when(is_selected, |el| {
                                el.child(
                                    div()
                                        .absolute()
                                        .left_0()
                                        .top_0()
                                        .bottom_0()
                                        .w(px(2.0))
                                        .bg(accent_bar),
                                )
                            });

                        if let Some(click_cb) = on_row_click.clone() {
                            let row_key_for_click = key_shared.clone();
                            row_el = row_el
                                .on_mouse_down(MouseButton::Left, |_, _, cx| {
                                    cx.stop_propagation();
                                })
                                .on_click(move |event, window, cx| {
                                    click_cb(row_idx, row_key_for_click.clone(), event, window, cx);
                                });
                        }

                        if let Some(ctx_cb) = on_row_context_menu.clone() {
                            let row_key_for_ctx = key_shared.clone();
                            row_el = row_el.on_mouse_down(
                                MouseButton::Right,
                                move |event, window, cx| {
                                    cx.stop_propagation();
                                    ctx_cb(row_idx, row_key_for_ctx.clone(), event, window, cx);
                                },
                            );
                        }

                        for col in cols.iter() {
                            let cell_content = (render_cell_fn)(row_idx, col, &theme_snapshot);
                            row_el =
                                row_el.child(render_table_cell(col, &theme_snapshot, cell_content));
                        }

                        if hover_actions.is_visible() {
                            let pill_bg = theme_snapshot.surface.bg_hover;
                            let pill_border = theme_snapshot.border.default;
                            let btn_hover_bg = theme_snapshot.surface.bg_active;
                            let icon_color = theme_snapshot.text.secondary;

                            let mut pill = div()
                                .h(px(24.0))
                                .px_1()
                                .gap_1()
                                .flex()
                                .items_center()
                                .rounded(theme_snapshot.style.radius_control)
                                .bg(pill_bg)
                                .border_1()
                                .border_color(pill_border)
                                .opacity(0.0)
                                .group_hover(row_group, |s| s.opacity(1.0));

                            if let Some(yaml_cb) = on_row_quick_yaml.clone() {
                                let row_key_for_yaml = key_shared.clone();
                                pill = pill.child(
                                    div()
                                        .id(ElementId::NamedInteger(
                                            yaml_btn_prefix.clone(),
                                            row_idx as u64,
                                        ))
                                        .w(px(20.0))
                                        .h(px(20.0))
                                        .rounded(theme_snapshot.style.radius_control)
                                        .flex()
                                        .items_center()
                                        .justify_center()
                                        .cursor_pointer()
                                        .hover(move |s| s.bg(btn_hover_bg))
                                        .tooltip(|_window, cx| {
                                            cx.new(|_| TextTooltip::new("View YAML")).into()
                                        })
                                        .on_mouse_down(MouseButton::Left, |_, _, cx| {
                                            cx.stop_propagation();
                                        })
                                        .on_click(move |event, window, cx| {
                                            cx.stop_propagation();
                                            yaml_cb(
                                                row_idx,
                                                row_key_for_yaml.clone(),
                                                event,
                                                window,
                                                cx,
                                            );
                                        })
                                        .child(render_icon(Icon::FileCode, px(13.0), icon_color)),
                                );
                            }

                            if let Some(ctx_cb) = on_row_context_menu.clone() {
                                let row_key_for_more = key_shared.clone();
                                pill = pill.child(
                                    div()
                                        .id(ElementId::NamedInteger(
                                            more_btn_prefix.clone(),
                                            row_idx as u64,
                                        ))
                                        .w(px(20.0))
                                        .h(px(20.0))
                                        .rounded(theme_snapshot.style.radius_control)
                                        .flex()
                                        .items_center()
                                        .justify_center()
                                        .cursor_pointer()
                                        .hover(move |s| s.bg(btn_hover_bg))
                                        .tooltip(|_window, cx| {
                                            cx.new(|_| TextTooltip::new("More actions")).into()
                                        })
                                        .on_mouse_down(
                                            MouseButton::Left,
                                            move |event, window, cx| {
                                                cx.stop_propagation();
                                                ctx_cb(
                                                    row_idx,
                                                    row_key_for_more.clone(),
                                                    event,
                                                    window,
                                                    cx,
                                                );
                                            },
                                        )
                                        .child(render_icon(Icon::Ellipsis, px(13.0), icon_color)),
                                );
                            }

                            row_el = row_el.child(
                                div()
                                    .absolute()
                                    .right_2()
                                    .top_0()
                                    .bottom_0()
                                    .flex()
                                    .items_center()
                                    .child(pill),
                            );
                        }

                        items.push(row_el);
                    }
                    items
                },
            )
            .track_scroll(state.scroll_handle.clone())
            .flex_1()
            .w_full()
            .min_h(px(0.0))
            .into_any_element()
        }
    };

    let surface = resolve_data_table_surface_style(theme);
    div()
        .id(table_id_str)
        .bg(surface.background)
        .border_t_1()
        .border_color(surface.top_hairline)
        .flex()
        .flex_col()
        .flex_1()
        .w_full()
        .min_h(px(0.0))
        .overflow_hidden()
        .child(header)
        .child(body)
        .into_any_element()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Clone, PartialEq, Eq)]
    struct SamplePodRow {
        id: usize,
        name: &'static str,
        restarts: u32,
    }

    #[test]
    fn test_data_table_sort_cycle_and_selection_state() {
        let mut state = DataTableState::new();
        assert_eq!(state.sort, None);
        assert_eq!(state.selected_row_key, None);

        state.toggle_sort("name");
        assert_eq!(
            state.sort,
            Some(SortState {
                column_id: "name".to_string(),
                direction: SortDirection::Asc,
            })
        );

        state.toggle_sort("name");
        assert_eq!(
            state.sort,
            Some(SortState {
                column_id: "name".to_string(),
                direction: SortDirection::Desc,
            })
        );

        state.toggle_sort("name");
        assert_eq!(state.sort, None);

        state.toggle_sort("name");
        state.toggle_sort("name");
        assert_eq!(
            state.sort.as_ref().map(|s| s.direction),
            Some(SortDirection::Desc)
        );
        state.toggle_sort("restarts");
        assert_eq!(
            state.sort,
            Some(SortState {
                column_id: "restarts".to_string(),
                direction: SortDirection::Asc,
            })
        );

        state.select_row("default/pod-a");
        assert!(state.is_selected("default/pod-a"));
        assert!(!state.is_selected("default/pod-b"));

        state.clear_selection();
        assert!(!state.is_selected("default/pod-a"));
        assert_eq!(state.selected_row_key, None);
    }

    #[test]
    fn test_sort_rows_stable_orders_asc_desc_and_preserves_ties() {
        let base_rows = vec![
            SamplePodRow {
                id: 1,
                name: "pod-b",
                restarts: 2,
            },
            SamplePodRow {
                id: 2,
                name: "pod-a",
                restarts: 5,
            },
            SamplePodRow {
                id: 3,
                name: "pod-c",
                restarts: 2,
            },
        ];

        let cmp = |a: &SamplePodRow, b: &SamplePodRow, col: &str| match col {
            "name" => a.name.cmp(b.name),
            "restarts" => a.restarts.cmp(&b.restarts),
            _ => Ordering::Equal,
        };

        let mut rows_asc = base_rows.clone();
        let sort_asc = SortState {
            column_id: "restarts".to_string(),
            direction: SortDirection::Asc,
        };
        sort_rows_stable(&mut rows_asc, Some(&sort_asc), cmp);
        assert_eq!(
            rows_asc.iter().map(|r| r.id).collect::<Vec<_>>(),
            vec![1, 3, 2]
        );

        let mut rows_desc = base_rows.clone();
        let sort_desc = SortState {
            column_id: "restarts".to_string(),
            direction: SortDirection::Desc,
        };
        sort_rows_stable(&mut rows_desc, Some(&sort_desc), cmp);
        assert_eq!(
            rows_desc.iter().map(|r| r.id).collect::<Vec<_>>(),
            vec![2, 1, 3]
        );

        let mut rows_none = base_rows.clone();
        sort_rows_stable(&mut rows_none, None, cmp);
        assert_eq!(rows_none, base_rows);
    }

    #[test]
    fn test_table_body_mode_transitions_for_loading_empty_and_populated() {
        let mut state = DataTableState::new();
        assert_eq!(state.empty_message.as_ref(), DEFAULT_EMPTY_MESSAGE);

        state.loading = true;
        assert_eq!(state.body_mode(0), TableBodyMode::LoadingSkeleton);
        assert_eq!(state.body_mode(5), TableBodyMode::Rows);

        state.loading = false;
        assert_eq!(state.body_mode(0), TableBodyMode::Empty);
        assert_eq!(state.body_mode(5), TableBodyMode::Rows);
    }

    #[test]
    fn test_column_alignment_and_header_sort_indicator_helpers() {
        let col_name = TableColumn::new("name", "Name").flex_width(2.0, px(180.0));
        let col_restarts = TableColumn::new("restarts", "Restarts")
            .fixed_width(px(90.0))
            .right_numeric();
        let col_actions = TableColumn::new("actions", "Actions")
            .fixed_width(px(80.0))
            .align(ColumnAlign::Center)
            .sortable(false);

        assert_eq!(col_name.align, ColumnAlign::Left);
        assert!(!col_name.monospace);
        assert!(col_name.sortable);

        assert_eq!(col_restarts.align, ColumnAlign::Right);
        assert!(col_restarts.monospace);
        assert!(col_restarts.sortable);

        assert_eq!(col_actions.align, ColumnAlign::Center);
        assert!(!col_actions.sortable);

        let mut state = DataTableState::new();
        assert_eq!(
            state.sort_indicator_for_column(&col_name).map(char::from),
            Some(char::from(Icon::ArrowUpDown))
        );
        assert!(state.sort_indicator_for_column(&col_actions).is_none());

        state.toggle_sort("name");
        assert_eq!(
            state.sort_indicator_for_column(&col_name).map(char::from),
            Some(char::from(Icon::ArrowUp))
        );
        assert_eq!(
            state
                .sort_indicator_for_column(&col_restarts)
                .map(char::from),
            Some(char::from(Icon::ArrowUpDown))
        );

        state.toggle_sort("name");
        assert_eq!(
            state.sort_indicator_for_column(&col_name).map(char::from),
            Some(char::from(Icon::ArrowDown))
        );
    }

    #[test]
    fn test_data_table_callbacks_accept_optional_row_context_menu_handler() {
        let default_callbacks =
            DataTableCallbacks::new(|idx| format!("row-{idx}"), |_, _, _| div().into_any_element());
        assert!(default_callbacks.on_row_context_menu.is_none());

        let with_ctx = DataTableCallbacks::new(
            |idx| format!("row-{idx}"),
            |_, _, _| div().into_any_element(),
        )
        .on_row_context_menu(|_, _, _, _, _| {});
        assert!(with_ctx.on_row_context_menu.is_some());
    }

    #[test]
    fn test_data_table_row_click_callback_builder_preserves_state_expectations() {
        let with_both = DataTableCallbacks::new(
            |idx| format!("row-{idx}"),
            |_, _, _| div().into_any_element(),
        )
        .on_row_click(|_, _, _, _, _| {})
        .on_row_context_menu(|_, _, _, _, _| {});

        assert!(with_both.on_row_click.is_some());
        assert!(with_both.on_row_context_menu.is_some());
    }

    #[test]
    fn test_table_header_style_uses_muted_inactive_and_primary_active_without_bolding() {
        use crate::theme::{AccentColor, SemanticColorsPreference, ThemeMode, UiStyle};

        for (mode, appearance) in [
            (ThemeMode::Light, gpui::WindowAppearance::Light),
            (ThemeMode::Dark, gpui::WindowAppearance::Dark),
        ] {
            let theme = OrbitTheme::resolve(
                mode,
                UiStyle::Flat,
                AccentColor::Blue,
                SemanticColorsPreference::default(),
                appearance,
            );

            let inactive = resolve_table_header_style(false, &theme);
            assert_eq!(inactive.text_color, theme.text.muted);
            assert_eq!(inactive.font_weight, FontWeight::MEDIUM);
            assert_eq!(inactive.icon_color, theme.text.muted);

            let active = resolve_table_header_style(true, &theme);
            assert_eq!(active.text_color, theme.text.primary);
            assert_eq!(active.font_weight, FontWeight::MEDIUM);
            assert_eq!(active.icon_color, theme.accent.accent);
        }
    }

    #[test]
    fn test_table_header_icon_order_aligns_edges_for_left_center_and_right_columns() {
        assert!(table_header_icon_before_label(ColumnAlign::Right));
        assert!(!table_header_icon_before_label(ColumnAlign::Left));
        assert!(!table_header_icon_before_label(ColumnAlign::Center));
    }

    #[test]
    fn test_table_cell_font_enables_tabular_numerals_for_right_numeric_and_monospace_columns() {
        use crate::theme::{AccentColor, SemanticColorsPreference, ThemeMode, UiStyle};

        let theme = OrbitTheme::resolve(
            ThemeMode::Dark,
            UiStyle::Flat,
            AccentColor::Blue,
            SemanticColorsPreference::default(),
            gpui::WindowAppearance::Dark,
        );

        let col_name = TableColumn::new("name", "Name");
        let col_replicas = TableColumn::new("replicas", "Replicas").right_numeric();
        let col_actions = TableColumn::new("actions", "Actions").align(ColumnAlign::Center);

        assert!(resolve_table_cell_font(&col_name, &theme).is_none());
        assert!(resolve_table_cell_font(&col_actions, &theme).is_none());

        let font = resolve_table_cell_font(&col_replicas, &theme)
            .expect("right_numeric column should resolve tabular monospace font");
        assert_eq!(font.weight, FontWeight::NORMAL);
        assert!(
            font.features
                .tag_value_list()
                .iter()
                .any(|(tag, val)| tag == "tnum" && *val == 1)
        );
    }

    #[test]
    fn test_table_dimensions_follow_4pt_8pt_scale() {
        assert_eq!(TABLE_HEADER_HEIGHT % 4.0, 0.0);
        assert_eq!(TABLE_ROW_HEIGHT % 4.0, 0.0);
        assert_eq!(TABLE_HEADER_HEIGHT, 32.0);
        assert!((32.0..=36.0).contains(&TABLE_ROW_HEIGHT));
    }

    #[test]
    fn test_data_table_surface_style_uses_full_bleed_hairlines_without_card_chrome() {
        use crate::theme::{AccentColor, SemanticColorsPreference, ThemeMode, UiStyle};

        for (mode, appearance) in [
            (ThemeMode::Light, gpui::WindowAppearance::Light),
            (ThemeMode::Dark, gpui::WindowAppearance::Dark),
        ] {
            let theme = OrbitTheme::resolve(
                mode,
                UiStyle::Flat,
                AccentColor::Blue,
                SemanticColorsPreference::default(),
                appearance,
            );
            let style = resolve_data_table_surface_style(&theme);
            assert!(!style.uses_card_chrome);
            assert_eq!(style.background, theme.surface.bg_app);
            assert_eq!(style.header_background, theme.surface.bg_panel);
            assert_eq!(style.top_hairline, theme.border.default);
            assert_eq!(style.header_separator, theme.border.default);
            assert_eq!(style.row_separator, theme.border.subtle);
            assert_ne!(style.background, style.header_background);
        }
    }

    #[test]
    fn test_row_hover_actions_and_quick_yaml_callback_builder() {
        let none = resolve_row_hover_actions(false, false);
        assert!(!none.is_visible());
        assert!(!none.show_quick_yaml);
        assert!(!none.show_more_actions);

        let both = resolve_row_hover_actions(true, true);
        assert!(both.is_visible());
        assert!(both.show_quick_yaml);
        assert!(both.show_more_actions);

        let more_only = resolve_row_hover_actions(false, true);
        assert!(more_only.is_visible());
        assert!(!more_only.show_quick_yaml);
        assert!(more_only.show_more_actions);

        let callbacks = DataTableCallbacks::new(
            |idx| format!("row-{idx}"),
            |_, _, _| div().into_any_element(),
        )
        .on_row_quick_yaml(|_, _, _, _, _| {});
        assert!(callbacks.on_row_quick_yaml.is_some());
    }
}
