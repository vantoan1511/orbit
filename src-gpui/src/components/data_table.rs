use std::cmp::Ordering;
use std::ops::Range;
use std::rc::Rc;

use gpui::{
    AnyElement, App, ClickEvent, Div, ElementId, FontWeight, InteractiveElement, IntoElement,
    MouseButton, MouseDownEvent, Pixels, SharedString, StatefulInteractiveElement, Styled,
    UniformListScrollHandle, Window, div, prelude::*, px, uniform_list,
};
use lucide_icons::Icon;

use crate::icons::render_icon;
use crate::theme::{OrbitTheme, apply_text_step};

pub const TABLE_HEADER_HEIGHT: f32 = 34.0;
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
}

pub fn render_table_header(
    table_id: &str,
    columns: &[TableColumn],
    state: &DataTableState,
    theme: &OrbitTheme,
    on_sort: Option<SortCallback>,
) -> AnyElement {
    let hover_bg = theme.surface.bg_hover;
    let mut header_row = div()
        .id(SharedString::from(format!("{table_id}-header")))
        .h(px(TABLE_HEADER_HEIGHT))
        .flex_shrink_0()
        .w_full()
        .flex()
        .items_center()
        .bg(theme.surface.bg_panel)
        .border_b_1()
        .border_color(theme.border.default);

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

        let label = apply_text_step(div(), theme.typography.size_xs)
            .font_weight(FontWeight::SEMIBOLD)
            .text_color(if is_active_sort {
                theme.text.primary
            } else {
                theme.text.secondary
            })
            .child(col.header.clone());

        cell = cell.child(label);

        if let Some(icon) = sort_icon {
            let icon_color = if is_active_sort {
                theme.accent.accent
            } else {
                theme.text.muted
            };
            cell = cell.child(render_icon(icon, px(12.0), icon_color));
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
    let mono_font = theme.typography.mono_font(FontWeight::NORMAL);
    apply_column_alignment(apply_column_width(div(), col.width), col.align)
        .h_full()
        .px_3()
        .flex()
        .items_center()
        .overflow_hidden()
        .text_size(theme.typography.size_xs.font_size)
        .line_height(theme.typography.size_xs.line_height)
        .text_color(theme.text.primary)
        .when(col.monospace, |el| el.font(mono_font))
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
            let selected_key = state.selected_row_key.clone();
            let theme_snapshot = theme.clone();
            let list_id = SharedString::from(format!("{table_id_str}-rows"));
            let row_prefix = SharedString::from(format!("{table_id_str}-row"));

            uniform_list(
                ElementId::Name(list_id),
                row_count,
                move |range: Range<usize>, _window: &mut Window, _cx: &mut App| {
                    let mut items = Vec::with_capacity(range.len());
                    let hover_bg = theme_snapshot.surface.bg_hover;
                    let selected_bg = theme_snapshot.accent.soft;
                    let border_subtle = theme_snapshot.border.subtle;
                    let accent_bar = theme_snapshot.accent.accent;

                    for row_idx in range {
                        let key = (row_key_fn)(row_idx);
                        let is_selected = selected_key.as_deref() == Some(key.as_str());
                        let key_shared = SharedString::from(key);

                        let mut row_el = div()
                            .id(ElementId::NamedInteger(row_prefix.clone(), row_idx as u64))
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

    theme
        .style_card(div().id(table_id_str))
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
}
