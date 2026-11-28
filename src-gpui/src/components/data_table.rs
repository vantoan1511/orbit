use std::cmp::Ordering;
use std::collections::{HashMap, HashSet};
use std::ops::Range;
use std::rc::Rc;

use gpui::{
    AnyElement, App, ClickEvent, CursorStyle, Div, ElementId, Entity, Font, FontWeight,
    InteractiveElement, IntoElement, MouseButton, MouseDownEvent, Pixels, RenderOnce, Rgba,
    SharedString, StatefulInteractiveElement, Styled, TextAlign, UniformListScrollHandle, Window,
    div, prelude::*, px, uniform_list,
};
use gpui::component::table::{
    Column as KitColumn, ColumnSort, DataTable as KitDataTable, TableDelegate, TableState,
};
use lucide_icons::Icon;
use orbit_core::config::{
    ALLOWED_TABLE_PAGE_SIZES, DEFAULT_TABLE_PAGE_SIZE, MAX_TABLE_COLUMN_WIDTH,
    MIN_TABLE_COLUMN_WIDTH, PersistedClusterTableFilters, PersistedSortDirection,
    PersistedTableConfig, PersistedTablePreferences,
};

use crate::components::tooltip::TextTooltip;
use crate::icons::render_icon;
use crate::theme::{OrbitTheme, apply_text_step};

pub const TABLE_HEADER_HEIGHT: f32 = 32.0;
pub const TABLE_ROW_HEIGHT: f32 = 36.0;
pub const TABLE_PAGINATOR_HEIGHT: f32 = 32.0;
pub const TOOLBAR_CONTROL_HEIGHT: f32 = 28.0;
pub const TOOLBAR_SEARCH_WIDTH: f32 = 220.0;
pub const COLUMN_POPOVER_WIDTH: f32 = 240.0;
pub const COLUMN_POPOVER_MAX_HEIGHT: f32 = 260.0;
pub const DEFAULT_SKELETON_ROWS: usize = 6;
pub const DEFAULT_EMPTY_MESSAGE: &str = "No resources found.";
pub const DEFAULT_MIN_COLUMN_WIDTH: f32 = MIN_TABLE_COLUMN_WIDTH as f32;
pub const MAX_COLUMN_WIDTH: f32 = MAX_TABLE_COLUMN_WIDTH as f32;
pub const COLUMN_RESIZE_HANDLE_WIDTH: f32 = 6.0;

#[derive(Debug, Clone, PartialEq)]
pub struct ActiveColumnResize {
    pub column_id: String,
    pub start_pointer_x: f32,
    pub start_width: f32,
    pub min_width: f32,
}

pub fn clamp_column_width(width: f32, min_width: f32) -> f32 {
    let floor = if min_width.is_finite() {
        min_width.clamp(DEFAULT_MIN_COLUMN_WIDTH, MAX_COLUMN_WIDTH)
    } else {
        DEFAULT_MIN_COLUMN_WIDTH
    };
    if !width.is_finite() {
        return floor;
    }
    width.clamp(floor, MAX_COLUMN_WIDTH)
}

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
    pub default_visible: bool,
    pub locked: bool,
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
            default_visible: true,
            locked: false,
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

    pub fn default_visible(mut self, default_visible: bool) -> Self {
        self.default_visible = if self.locked { true } else { default_visible };
        self
    }

    pub fn locked(mut self, locked: bool) -> Self {
        self.locked = locked;
        if locked {
            self.default_visible = true;
        }
        self
    }

    pub fn min_resize_width(&self) -> f32 {
        DEFAULT_MIN_COLUMN_WIDTH
    }
}

pub fn column_sort_from_state(col: &TableColumn, sort: Option<&SortState>) -> Option<ColumnSort> {
    if !col.sortable {
        return None;
    }
    match sort {
        Some(active) if active.column_id == col.id.as_ref() => Some(match active.direction {
            SortDirection::Asc => ColumnSort::Ascending,
            SortDirection::Desc => ColumnSort::Descending,
        }),
        _ => Some(ColumnSort::Default),
    }
}

pub fn sort_direction_from_kit(sort: ColumnSort) -> Option<SortDirection> {
    match sort {
        ColumnSort::Ascending => Some(SortDirection::Asc),
        ColumnSort::Descending => Some(SortDirection::Desc),
        ColumnSort::Default => None,
    }
}

pub fn to_kit_column(col: &TableColumn, state: &DataTableState) -> KitColumn {
    let min_w = match col.width {
        ColumnWidth::Flex { min_width, .. } => {
            f32::from(min_width).clamp(col.min_resize_width(), MAX_COLUMN_WIDTH)
        }
        ColumnWidth::Fixed(_) => col.min_resize_width(),
    };
    let width = match state.resolve_column_width(col) {
        ColumnWidth::Fixed(w) => clamp_column_width(f32::from(w), min_w),
        ColumnWidth::Flex { min_width, .. } => {
            clamp_column_width(f32::from(min_width).max(120.0), min_w)
        }
    };

    let mut kit_col = KitColumn::new(col.id.clone(), col.header.clone())
        .width(px(width))
        .min_width(px(min_w))
        .max_width(px(MAX_COLUMN_WIDTH))
        .resizable(true)
        .movable(false);

    kit_col.align = match col.align {
        ColumnAlign::Left => TextAlign::Left,
        ColumnAlign::Right => TextAlign::Right,
        ColumnAlign::Center => TextAlign::Center,
    };
    kit_col.sort = column_sort_from_state(col, state.sort.as_ref());
    kit_col
}

pub type KitSortCallback = Rc<dyn Fn(SharedString, Option<SortDirection>, &mut Window, &mut App)>;
pub type RowKeyFn = Rc<dyn Fn(usize) -> String>;
pub type RenderCellFn = Rc<dyn Fn(usize, &TableColumn, &OrbitTheme) -> AnyElement>;

pub struct OrbitTableDelegate {
    pub columns: Vec<TableColumn>,
    pub kit_columns: Vec<KitColumn>,
    pub row_count: usize,
    pub row_offset: usize,
    pub row_key: RowKeyFn,
    pub render_cell: RenderCellFn,
    pub selected_row_key: Option<String>,
    pub sort: Option<SortState>,
    pub theme: OrbitTheme,
    pub on_sort: Option<KitSortCallback>,
    pub on_row_click: Option<RowClickCallback>,
    pub on_row_context_menu: Option<RowContextMenuCallback>,
    pub on_row_quick_yaml: Option<RowClickCallback>,
}

impl OrbitTableDelegate {
    pub fn new(
        columns: Vec<TableColumn>,
        row_count: usize,
        row_offset: usize,
        row_key: RowKeyFn,
        render_cell: RenderCellFn,
        state: &DataTableState,
        theme: OrbitTheme,
    ) -> Self {
        let kit_columns = columns.iter().map(|col| to_kit_column(col, state)).collect();
        Self {
            columns,
            kit_columns,
            row_count,
            row_offset,
            row_key,
            render_cell,
            selected_row_key: state.selected_row_key.clone(),
            sort: state.sort.clone(),
            theme,
            on_sort: None,
            on_row_click: None,
            on_row_context_menu: None,
            on_row_quick_yaml: None,
        }
    }

    pub fn with_on_sort(mut self, on_sort: KitSortCallback) -> Self {
        self.on_sort = Some(on_sort);
        self
    }

    pub fn with_on_row_click(mut self, on_row_click: Option<RowClickCallback>) -> Self {
        self.on_row_click = on_row_click;
        self
    }

    pub fn with_on_row_context_menu(
        mut self,
        on_row_context_menu: Option<RowContextMenuCallback>,
    ) -> Self {
        self.on_row_context_menu = on_row_context_menu;
        self
    }

    pub fn with_on_row_quick_yaml(mut self, on_row_quick_yaml: Option<RowClickCallback>) -> Self {
        self.on_row_quick_yaml = on_row_quick_yaml;
        self
    }

    #[allow(clippy::too_many_arguments)]
    pub fn sync_from(
        &mut self,
        columns: Vec<TableColumn>,
        row_count: usize,
        row_offset: usize,
        row_key: RowKeyFn,
        render_cell: RenderCellFn,
        selected_row_key: Option<String>,
        sort: Option<SortState>,
        kit_columns: Vec<KitColumn>,
        theme: OrbitTheme,
    ) {
        self.kit_columns = kit_columns;
        self.columns = columns;
        self.row_count = row_count;
        self.row_offset = row_offset;
        self.row_key = row_key;
        self.render_cell = render_cell;
        self.selected_row_key = selected_row_key;
        self.sort = sort;
        self.theme = theme;
    }

    pub fn row_key_at(&self, local_row_ix: usize) -> Option<(usize, SharedString)> {
        if local_row_ix >= self.row_count {
            return None;
        }
        let abs_row_ix = self.row_offset + local_row_ix;
        Some((abs_row_ix, SharedString::from((self.row_key)(abs_row_ix))))
    }
}

impl TableDelegate for OrbitTableDelegate {
    fn columns_count(&self, _cx: &App) -> usize {
        self.kit_columns.len()
    }

    fn rows_count(&self, _cx: &App) -> usize {
        self.row_count
    }

    fn column(&self, col_ix: usize, _cx: &App) -> KitColumn {
        self.kit_columns.get(col_ix).cloned().unwrap_or_default()
    }

    fn render_th(
        &mut self,
        col_ix: usize,
        _window: &mut Window,
        _cx: &mut Context<TableState<Self>>,
    ) -> impl IntoElement {
        let Some(col) = self.columns.get(col_ix) else {
            return div().into_any_element();
        };
        let is_active = self
            .sort
            .as_ref()
            .is_some_and(|s| s.column_id == col.id.as_ref());
        let header_style = resolve_table_header_style(is_active, &self.theme);
        apply_column_alignment(div().w_full().min_w(px(0.0)), col.align)
            .flex()
            .items_center()
            .text_size(self.theme.typography.size_xs.font_size)
            .line_height(self.theme.typography.size_xs.line_height)
            .text_color(header_style.text_color)
            .font(self.theme.typography.ui_font(header_style.font_weight))
            .truncate()
            .child(col.header.clone())
            .into_any_element()
    }

    fn render_td(
        &mut self,
        row_ix: usize,
        col_ix: usize,
        _window: &mut Window,
        _cx: &mut Context<TableState<Self>>,
    ) -> impl IntoElement {
        let Some(col) = self.columns.get(col_ix) else {
            return div().into_any_element();
        };
        let abs_row_ix = self.row_offset + row_ix;
        let content = (self.render_cell)(abs_row_ix, col, &self.theme);
        let cell_font = resolve_table_cell_font(col, &self.theme);
        apply_column_alignment(div().w_full().min_w(px(0.0)), col.align)
            .h_full()
            .flex()
            .items_center()
            .overflow_hidden()
            .text_size(self.theme.typography.size_xs.font_size)
            .line_height(self.theme.typography.size_xs.line_height)
            .text_color(self.theme.text.primary)
            .when_some(cell_font, |el, font| el.font(font))
            .child(content)
            .into_any_element()
    }

    fn perform_sort(
        &mut self,
        col_ix: usize,
        sort: ColumnSort,
        window: &mut Window,
        cx: &mut Context<TableState<Self>>,
    ) {
        let Some(col_id) = self.columns.get(col_ix).map(|c| c.id.clone()) else {
            return;
        };
        let mapped = sort_direction_from_kit(sort);
        self.sort = mapped.map(|direction| SortState {
            column_id: col_id.to_string(),
            direction,
        });
        for (idx, kit_col) in self.kit_columns.iter_mut().enumerate() {
            if let Some(col) = self.columns.get(idx) {
                kit_col.sort = column_sort_from_state(col, self.sort.as_ref());
            }
        }
        if let Some(on_sort) = self.on_sort.clone() {
            (on_sort)(col_id, mapped, window, cx);
        }
    }

    fn fit_column(
        &mut self,
        col_ix: usize,
        _window: &mut Window,
        _cx: &mut Context<TableState<Self>>,
    ) -> Option<Pixels> {
        let kit_col = self.kit_columns.get(col_ix)?;
        let min_w = f32::from(kit_col.min_width);
        let target = clamp_column_width(min_w.max(120.0), min_w);
        Some(px(target))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PaginationMode {
    /// Slices `0..row_count` locally using `first` and `page_size`.
    Client,
    /// Rows passed in are already the current page; `total_records` drives paginator bounds.
    Lazy { total_records: usize },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PageSlice {
    pub range: Range<usize>,
    pub clamped_first: usize,
    pub page: usize,
    pub total_pages: usize,
    pub total_records: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TablePaginationState {
    pub enabled: bool,
    pub first: usize,
    pub page_size: usize,
    pub mode: PaginationMode,
}

impl Default for TablePaginationState {
    fn default() -> Self {
        Self {
            enabled: false,
            first: 0,
            page_size: DEFAULT_TABLE_PAGE_SIZE,
            mode: PaginationMode::Client,
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TableFilterState {
    pub search_query: String,
    pub search_focused: bool,
    pub extra_filters: HashMap<String, String>,
}

pub fn paginate_rows_range(
    row_count: usize,
    first: usize,
    page_size: usize,
    mode: PaginationMode,
) -> PageSlice {
    let effective_size = page_size.max(1);
    let total_records = match mode {
        PaginationMode::Client => row_count,
        PaginationMode::Lazy { total_records } => total_records,
    };

    if total_records == 0 {
        return PageSlice {
            range: 0..0,
            clamped_first: 0,
            page: 0,
            total_pages: 1,
            total_records: 0,
        };
    }

    let total_pages = total_records.div_ceil(effective_size);
    let clamped_page = (first / effective_size).min(total_pages - 1);
    let clamped_first = clamped_page * effective_size;

    let range = match mode {
        PaginationMode::Client => {
            let end = (clamped_first + effective_size).min(row_count);
            clamped_first.min(end)..end
        }
        PaginationMode::Lazy { .. } => 0..row_count.min(effective_size),
    };

    PageSlice {
        range,
        clamped_first,
        page: clamped_page,
        total_pages,
        total_records,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TableBodyMode {
    LoadingSkeleton,
    Empty,
    Rows,
    RefreshingRows,
}

#[derive(Debug, Clone)]
pub struct DataTableState {
    pub sort: Option<SortState>,
    pub selected_row_key: Option<String>,
    pub scroll_handle: UniformListScrollHandle,
    pub loading: bool,
    pub refreshing: bool,
    pub empty_message: SharedString,
    pub pagination: TablePaginationState,
    pub filters: TableFilterState,
    pub hidden_columns: HashSet<String>,
    pub config_popover_open: bool,
    pub column_widths: HashMap<String, f32>,
    pub active_resize: Option<ActiveColumnResize>,
    pub kit_table: Option<Entity<TableState<OrbitTableDelegate>>>,
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
            refreshing: false,
            empty_message: SharedString::from(DEFAULT_EMPTY_MESSAGE),
            pagination: TablePaginationState::default(),
            filters: TableFilterState::default(),
            hidden_columns: HashSet::new(),
            config_popover_open: false,
            column_widths: HashMap::new(),
            active_resize: None,
            kit_table: None,
        }
    }

    pub fn with_empty_message(mut self, message: impl Into<SharedString>) -> Self {
        self.empty_message = message.into();
        self
    }

    pub fn with_pagination(mut self) -> Self {
        self.pagination.enabled = true;
        self.pagination.mode = PaginationMode::Client;
        self
    }

    pub fn with_lazy_pagination(mut self, total_records: usize) -> Self {
        self.pagination.enabled = true;
        self.pagination.mode = PaginationMode::Lazy { total_records };
        self
    }

    pub fn reset_page(&mut self) {
        self.pagination.first = 0;
        self.scroll_handle.scroll_to_item(0, gpui::ScrollStrategy::Top);
    }

    pub fn set_search_query(&mut self, query: impl Into<String>) {
        self.filters.search_query = query.into();
        self.reset_page();
    }

    pub fn set_extra_filter(&mut self, key: impl Into<String>, value: impl Into<String>) {
        let key = key.into();
        let value = value.into();
        if value.trim().is_empty() {
            self.filters.extra_filters.remove(&key);
        } else {
            self.filters.extra_filters.insert(key, value);
        }
        self.reset_page();
    }

    pub fn clear_filters(&mut self) {
        self.filters.search_query.clear();
        self.filters.extra_filters.clear();
        self.reset_page();
    }

    pub fn has_active_filters(&self, defaults: &[(&str, &str)]) -> bool {
        if !self.filters.search_query.trim().is_empty() {
            return true;
        }
        for (key, value) in &self.filters.extra_filters {
            let trimmed = value.trim();
            if trimmed.is_empty() {
                continue;
            }
            let is_default = defaults
                .iter()
                .any(|(def_k, def_v)| *def_k == key.as_str() && def_v.eq_ignore_ascii_case(trimmed));
            if !is_default {
                return true;
            }
        }
        false
    }

    pub fn visible_columns(&self, all_columns: &[TableColumn]) -> Vec<TableColumn> {
        all_columns
            .iter()
            .filter(|col| col.locked || !self.hidden_columns.contains(col.id.as_ref()))
            .cloned()
            .collect()
    }

    pub fn resolved_visible_columns(&self, all_columns: &[TableColumn]) -> Vec<TableColumn> {
        self.visible_columns(all_columns)
            .into_iter()
            .map(|mut col| {
                col.width = self.resolve_column_width(&col);
                col
            })
            .collect()
    }

    pub fn toggle_column(&mut self, col_id: &str, locked: bool) -> bool {
        if locked {
            return false;
        }
        if !self.hidden_columns.remove(col_id) {
            self.hidden_columns.insert(col_id.to_string());
        }
        true
    }

    pub fn set_all_columns_visible(&mut self, all_columns: &[TableColumn], visible: bool) {
        if visible {
            self.hidden_columns.clear();
        } else {
            self.hidden_columns = all_columns
                .iter()
                .filter(|col| !col.locked)
                .map(|col| col.id.to_string())
                .collect();
        }
    }

    pub fn reset_columns(&mut self, all_columns: &[TableColumn]) {
        self.hidden_columns = all_columns
            .iter()
            .filter(|col| !col.locked && !col.default_visible)
            .map(|col| col.id.to_string())
            .collect();
        self.reset_column_widths();
    }

    pub fn has_custom_columns(&self, all_columns: &[TableColumn]) -> bool {
        let default_hidden: HashSet<&str> = all_columns
            .iter()
            .filter(|col| !col.locked && !col.default_visible)
            .map(|col| col.id.as_ref())
            .collect();
        let current_hidden: HashSet<&str> = all_columns
            .iter()
            .filter(|col| !col.locked && self.hidden_columns.contains(col.id.as_ref()))
            .map(|col| col.id.as_ref())
            .collect();
        current_hidden != default_hidden || !self.column_widths.is_empty()
    }

    pub fn resolve_column_width(&self, col: &TableColumn) -> ColumnWidth {
        if let Some(&width) = self.column_widths.get(col.id.as_ref()) {
            ColumnWidth::Fixed(px(clamp_column_width(width, col.min_resize_width())))
        } else {
            col.width
        }
    }

    pub fn estimate_rendered_column_width(
        &self,
        col_id: &str,
        visible_columns: &[TableColumn],
        container_width: f32,
    ) -> f32 {
        let mut fixed_total = 0.0_f32;
        let mut total_flex_weight = 0.0_f32;
        let mut target_col: Option<&TableColumn> = None;

        for col in visible_columns {
            if col.id.as_ref() == col_id {
                target_col = Some(col);
            }
            match self.resolve_column_width(col) {
                ColumnWidth::Fixed(w) => fixed_total += f32::from(w),
                ColumnWidth::Flex { weight, .. } => total_flex_weight += weight.max(0.0),
            }
        }

        let Some(col) = target_col else {
            return DEFAULT_MIN_COLUMN_WIDTH;
        };

        match self.resolve_column_width(col) {
            ColumnWidth::Fixed(w) => clamp_column_width(f32::from(w), col.min_resize_width()),
            ColumnWidth::Flex { weight, min_width } => {
                let remaining = (container_width - fixed_total).max(0.0);
                let share = if total_flex_weight > 0.0 {
                    remaining * (weight.max(0.0) / total_flex_weight)
                } else {
                    f32::from(min_width)
                };
                clamp_column_width(share.max(f32::from(min_width)), col.min_resize_width())
            }
        }
    }

    pub fn start_column_resize(
        &mut self,
        col_id: &str,
        pointer_x: f32,
        visible_columns: &[TableColumn],
        container_width: f32,
    ) -> bool {
        let Some(col) = visible_columns.iter().find(|c| c.id.as_ref() == col_id) else {
            return false;
        };
        let start_width =
            self.estimate_rendered_column_width(col_id, visible_columns, container_width);
        let min_width = col.min_resize_width();
        self.column_widths.insert(col_id.to_string(), start_width);
        self.active_resize = Some(ActiveColumnResize {
            column_id: col_id.to_string(),
            start_pointer_x: pointer_x,
            start_width,
            min_width,
        });
        true
    }

    pub fn update_column_resize(&mut self, pointer_x: f32) -> bool {
        let Some(active) = &self.active_resize else {
            return false;
        };
        let next_width =
            clamp_column_width(active.start_width + (pointer_x - active.start_pointer_x), active.min_width);
        let prev = self.column_widths.insert(active.column_id.clone(), next_width);
        prev != Some(next_width)
    }

    pub fn end_column_resize(&mut self) -> bool {
        self.active_resize.take().is_some()
    }

    pub fn reset_column_width(&mut self, col_id: &str) -> bool {
        self.column_widths.remove(col_id).is_some()
    }

    pub fn reset_column_widths(&mut self) {
        self.column_widths.clear();
        self.active_resize = None;
    }

    pub fn apply_kit_column_widths(&mut self, columns: &[TableColumn], widths: &[Pixels]) -> bool {
        let mut changed = false;
        for (col, width) in columns.iter().zip(widths.iter()) {
            let min_w = match col.width {
                ColumnWidth::Flex { min_width, .. } => {
                    f32::from(min_width).clamp(col.min_resize_width(), MAX_COLUMN_WIDTH)
                }
                ColumnWidth::Fixed(_) => col.min_resize_width(),
            };
            let clamped = clamp_column_width(f32::from(*width), min_w);
            let prev = self.column_widths.get(col.id.as_ref()).copied();
            if prev.is_none_or(|p| (p - clamped).abs() > 0.5) {
                self.column_widths.insert(col.id.to_string(), clamped);
                changed = true;
            }
        }
        changed
    }

    pub fn sync_selected_row(&self, row_keys: &[String]) -> Option<usize> {
        let selected = self.selected_row_key.as_deref()?;
        row_keys.iter().position(|k| k == selected)
    }

    pub fn hydrate_from_preferences(
        &mut self,
        table_id: &str,
        cluster_id: Option<&str>,
        prefs: &PersistedTablePreferences,
        columns: &[TableColumn],
    ) {
        if let Some(table_cfg) = prefs.tables.get(table_id) {
            self.pagination.page_size = table_cfg
                .page_size
                .filter(|sz| ALLOWED_TABLE_PAGE_SIZES.contains(sz))
                .unwrap_or(DEFAULT_TABLE_PAGE_SIZE);

            let locked_ids: HashSet<&str> = columns
                .iter()
                .filter(|c| c.locked)
                .map(|c| c.id.as_ref())
                .collect();
            let valid_ids: HashSet<&str> = columns.iter().map(|c| c.id.as_ref()).collect();

            self.hidden_columns = table_cfg
                .hidden_columns
                .iter()
                .filter(|id| {
                    let s = id.as_str();
                    !locked_ids.contains(s) && (valid_ids.is_empty() || valid_ids.contains(s))
                })
                .cloned()
                .collect();

            self.sort = match (&table_cfg.sort_column, table_cfg.sort_direction) {
                (Some(col), Some(dir)) if valid_ids.is_empty() || valid_ids.contains(col.as_str()) => {
                    Some(SortState {
                        column_id: col.clone(),
                        direction: match dir {
                            PersistedSortDirection::Asc => SortDirection::Asc,
                            PersistedSortDirection::Desc => SortDirection::Desc,
                        },
                    })
                }
                _ => None,
            };

            self.column_widths = table_cfg
                .column_widths
                .iter()
                .filter_map(|(col_id, &width)| {
                    let trimmed = col_id.trim();
                    if trimmed.is_empty() || width == 0 {
                        return None;
                    }
                    if !valid_ids.is_empty() && !valid_ids.contains(trimmed) {
                        return None;
                    }
                    let min_width = columns
                        .iter()
                        .find(|c| c.id.as_ref() == trimmed)
                        .map(|c| c.min_resize_width())
                        .unwrap_or(DEFAULT_MIN_COLUMN_WIDTH);
                    Some((trimmed.to_string(), clamp_column_width(width as f32, min_width)))
                })
                .collect();
            self.active_resize = None;
        } else {
            self.pagination.page_size = DEFAULT_TABLE_PAGE_SIZE;
            self.reset_columns(columns);
            self.sort = None;
        }

        if let Some(cluster_filters) = cluster_id
            .and_then(|cid| prefs.clusters.get(cid))
            .and_then(|tables| tables.get(table_id))
        {
            self.filters.search_query = cluster_filters.search_query.clone();
            self.filters.extra_filters = cluster_filters.extra_filters.clone();
        } else {
            self.filters.search_query.clear();
            self.filters.extra_filters.clear();
        }

        self.reset_page();
    }

    pub fn to_persisted_config(&self) -> PersistedTableConfig {
        let mut hidden_columns: Vec<String> = self.hidden_columns.iter().cloned().collect();
        hidden_columns.sort();

        let column_widths: HashMap<String, u32> = self
            .column_widths
            .iter()
            .filter_map(|(col_id, &width)| {
                let trimmed = col_id.trim();
                if trimmed.is_empty() {
                    None
                } else {
                    let clamped =
                        clamp_column_width(width, DEFAULT_MIN_COLUMN_WIDTH).round() as u32;
                    Some((trimmed.to_string(), clamped))
                }
            })
            .collect();

        PersistedTableConfig {
            page_size: if self.pagination.enabled {
                Some(self.pagination.page_size)
            } else {
                None
            },
            hidden_columns,
            sort_column: self.sort.as_ref().map(|s| s.column_id.clone()),
            sort_direction: self.sort.as_ref().map(|s| match s.direction {
                SortDirection::Asc => PersistedSortDirection::Asc,
                SortDirection::Desc => PersistedSortDirection::Desc,
            }),
            column_widths,
        }
    }

    pub fn to_persisted_filters(&self) -> PersistedClusterTableFilters {
        PersistedClusterTableFilters {
            search_query: self.filters.search_query.trim().to_string(),
            extra_filters: self.filters.extra_filters.clone(),
        }
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

    pub fn move_selection_by_delta(
        &mut self,
        delta: isize,
        visible_row_keys: &[String],
    ) -> Option<usize> {
        if visible_row_keys.is_empty() {
            return None;
        }
        let current = self
            .selected_row_key
            .as_deref()
            .and_then(|key| visible_row_keys.iter().position(|k| k == key));
        let next_idx = match current {
            Some(idx) if delta >= 0 => (idx + delta as usize).min(visible_row_keys.len() - 1),
            Some(idx) => idx.saturating_sub(delta.unsigned_abs()),
            None if delta >= 0 => 0,
            None => visible_row_keys.len() - 1,
        };
        self.selected_row_key = visible_row_keys.get(next_idx).cloned();
        if self.pagination.enabled {
            let page_size = self.pagination.page_size.max(1);
            let target_first = (next_idx / page_size) * page_size;
            self.pagination.first = target_first;
            let local_idx = next_idx - target_first;
            self.scroll_handle
                .scroll_to_item(local_idx, gpui::ScrollStrategy::Center);
        } else {
            self.scroll_handle
                .scroll_to_item(next_idx, gpui::ScrollStrategy::Center);
        }
        Some(next_idx)
    }

    pub fn body_mode(&self, row_count: usize) -> TableBodyMode {
        if self.loading && row_count == 0 {
            TableBodyMode::LoadingSkeleton
        } else if row_count == 0 {
            TableBodyMode::Empty
        } else if self.refreshing {
            TableBodyMode::RefreshingRows
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
pub type PageChangeCallback = Rc<dyn Fn(usize, usize, &mut Window, &mut App)>;
pub type ColumnResizeStartCallback = Rc<dyn Fn(SharedString, f32, &mut Window, &mut App)>;

#[derive(Clone)]
pub struct EmptyStateAction {
    pub label: SharedString,
    pub icon: Icon,
    pub on_click: TableActionCallback,
}

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
    pub on_page_change: Option<PageChangeCallback>,
    pub on_column_resize_start: Option<ColumnResizeStartCallback>,
    pub on_empty_action: Option<EmptyStateAction>,
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
            on_page_change: None,
            on_column_resize_start: None,
            on_empty_action: None,
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

    pub fn on_page_change(
        mut self,
        callback: impl 'static + Fn(usize, usize, &mut Window, &mut App),
    ) -> Self {
        self.on_page_change = Some(Rc::new(callback));
        self
    }

    pub fn on_column_resize_start(
        mut self,
        callback: impl 'static + Fn(SharedString, f32, &mut Window, &mut App),
    ) -> Self {
        self.on_column_resize_start = Some(Rc::new(callback));
        self
    }

    pub fn on_empty_action(mut self, action: EmptyStateAction) -> Self {
        self.on_empty_action = Some(action);
        self
    }
}

pub fn render_table_header(
    table_id: &str,
    columns: &[TableColumn],
    state: &DataTableState,
    theme: &OrbitTheme,
    on_sort: Option<SortCallback>,
    on_column_resize_start: Option<ColumnResizeStartCallback>,
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
        .relative()
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

        if col_idx + 1 < columns.len()
            && let Some(resize_cb) = on_column_resize_start.clone()
        {
            let resize_col_id = col.id.clone();
            let is_active_resize = state
                .active_resize
                .as_ref()
                .is_some_and(|r| r.column_id == col.id.as_ref());
            let divider_color = if is_active_resize {
                theme.accent.accent
            } else {
                theme.border.subtle
            };
            let handle_hover = theme.accent.soft;

            cell = cell.child(
                div()
                    .id(ElementId::NamedInteger(
                        SharedString::from(format!("{table_id}-hdr-resize")),
                        col_idx as u64,
                    ))
                    .absolute()
                    .top_0()
                    .bottom_0()
                    .right(px(-(COLUMN_RESIZE_HANDLE_WIDTH / 2.0)))
                    .w(px(COLUMN_RESIZE_HANDLE_WIDTH))
                    .flex()
                    .items_center()
                    .justify_center()
                    .cursor(CursorStyle::ResizeLeftRight)
                    .hover(move |s| s.bg(handle_hover))
                    .on_mouse_down(MouseButton::Left, move |event, window, cx| {
                        cx.stop_propagation();
                        resize_cb(
                            resize_col_id.clone(),
                            f32::from(event.position.x),
                            window,
                            cx,
                        );
                    })
                    .on_click(|_, _, cx| {
                        cx.stop_propagation();
                    })
                    .child(div().w(px(1.0)).h_full().bg(divider_color)),
            );
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
    render_empty_state_with_action(table_id, empty_message, None, theme)
}

pub fn render_empty_state_with_action(
    table_id: &str,
    empty_message: SharedString,
    action: Option<&EmptyStateAction>,
    theme: &OrbitTheme,
) -> AnyElement {
    let hover_bg = theme.surface.control_hover;
    div()
        .id(SharedString::from(format!("{table_id}-empty")))
        .flex_1()
        .w_full()
        .min_h(px(160.0))
        .flex()
        .flex_col()
        .items_center()
        .justify_center()
        .gap_2p5()
        .p_6()
        .child(render_icon(Icon::Info, px(20.0), theme.text.muted))
        .child(
            apply_text_step(div(), theme.typography.size_xs)
                .font_weight(FontWeight::MEDIUM)
                .text_color(theme.text.muted)
                .child(empty_message),
        )
        .when_some(action, |el, act| {
            let on_click = act.on_click.clone();
            el.child(
                div()
                    .id(SharedString::from(format!("{table_id}-empty-action")))
                    .h(px(28.0))
                    .px_3()
                    .rounded(theme.style.radius_control)
                    .border_1()
                    .border_color(theme.border.default)
                    .bg(theme.surface.bg_card)
                    .flex()
                    .items_center()
                    .gap_1p5()
                    .cursor_pointer()
                    .hover(move |s| s.bg(hover_bg))
                    .on_click(move |_, window, cx| on_click(window, cx))
                    .child(render_icon(act.icon, px(13.0), theme.text.secondary))
                    .child(
                        apply_text_step(div(), theme.typography.size_xs)
                            .font_weight(FontWeight::MEDIUM)
                            .text_color(theme.text.primary)
                            .child(act.label.clone()),
                    ),
            )
        })
        .into_any_element()
}

pub fn format_paginator_report(slice: &PageSlice) -> String {
    if slice.total_records == 0 {
        return "Showing 0 of 0".to_string();
    }
    let start = slice.clamped_first + 1;
    let end = (slice.clamped_first + slice.range.len()).min(slice.total_records);
    format!("Showing {start}\u{2013}{end} of {}", slice.total_records)
}

#[derive(Debug, Clone)]
pub struct TableFilterChipOption {
    pub value: &'static str,
    pub label: &'static str,
}

#[derive(Debug, Clone)]
pub struct TableFilterChipGroup {
    pub key: &'static str,
    pub default_value: &'static str,
    pub options: Vec<TableFilterChipOption>,
}

pub type TableActionCallback = Rc<dyn Fn(&mut Window, &mut App)>;
pub type TableBoolCallback = Rc<dyn Fn(bool, &mut Window, &mut App)>;
pub type TableExtraFilterCallback = Rc<dyn Fn(String, String, &mut Window, &mut App)>;
pub type TableColumnToggleCallback = Rc<dyn Fn(String, bool, &mut Window, &mut App)>;

pub struct TableToolbarCallbacks {
    pub on_focus_search: TableBoolCallback,
    pub on_clear_search: TableActionCallback,
    pub on_extra_filter: TableExtraFilterCallback,
    pub on_clear_filters: TableActionCallback,
    pub on_toggle_columns_popover: TableActionCallback,
}

pub struct ColumnConfigCallbacks {
    pub on_close: TableActionCallback,
    pub on_toggle_column: TableColumnToggleCallback,
    pub on_toggle_all: TableBoolCallback,
    pub on_reset_defaults: TableActionCallback,
}

pub struct DataTableEnhancementCallbacks {
    pub on_page_change: PageChangeCallback,
    pub on_column_resize_start: ColumnResizeStartCallback,
    pub toolbar: TableToolbarCallbacks,
    pub column_config: ColumnConfigCallbacks,
}

pub fn render_table_toolbar(
    table_id: &str,
    search_placeholder: &str,
    filter_groups: &[TableFilterChipGroup],
    state: &DataTableState,
    theme: &OrbitTheme,
    callbacks: TableToolbarCallbacks,
) -> AnyElement {
    let defaults: Vec<(&str, &str)> = filter_groups
        .iter()
        .map(|g| (g.key, g.default_value))
        .collect();
    let has_active_filters = state.has_active_filters(&defaults);

    let search_box = render_toolbar_search_box(
        table_id,
        search_placeholder,
        state,
        theme,
        callbacks.on_focus_search.clone(),
        callbacks.on_clear_search.clone(),
    );

    let mut left_group = div()
        .flex()
        .items_center()
        .flex_wrap()
        .gap_2()
        .child(search_box);

    for group in filter_groups {
        left_group = left_group.child(render_toolbar_filter_group(
            table_id,
            group,
            state,
            theme,
            callbacks.on_extra_filter.clone(),
        ));
    }

    if has_active_filters {
        let on_clear_filters = callbacks.on_clear_filters.clone();
        let hover_bg = theme.surface.control_hover;
        left_group = left_group.child(
            div()
                .id(SharedString::from(format!("{table_id}-clear-filters")))
                .h(px(TOOLBAR_CONTROL_HEIGHT))
                .px_2p5()
                .rounded(theme.style.radius_control)
                .border_1()
                .border_color(theme.border.default)
                .bg(theme.surface.bg_panel)
                .flex()
                .items_center()
                .gap_1()
                .cursor_pointer()
                .hover(move |s| s.bg(hover_bg))
                .on_click(move |_, window, cx| on_clear_filters(window, cx))
                .child(render_icon(Icon::FilterX, px(12.0), theme.text.secondary))
                .child(
                    apply_text_step(div(), theme.typography.size_xs)
                        .font_weight(FontWeight::MEDIUM)
                        .text_color(theme.text.secondary)
                        .child("Clear Filters"),
                ),
        );
    }

    let cols_btn = render_toolbar_columns_button(
        table_id,
        state.config_popover_open,
        theme,
        callbacks.on_toggle_columns_popover,
    );

    div()
        .id(SharedString::from(format!("{table_id}-toolbar")))
        .px_4()
        .py_2()
        .flex_shrink_0()
        .flex()
        .items_center()
        .justify_between()
        .gap_2()
        .flex_wrap()
        .child(left_group)
        .child(cols_btn)
        .into_any_element()
}

fn render_toolbar_search_box(
    table_id: &str,
    search_placeholder: &str,
    state: &DataTableState,
    theme: &OrbitTheme,
    on_focus_search: TableBoolCallback,
    on_clear_search: TableActionCallback,
) -> AnyElement {
    div()
        .id(SharedString::from(format!("{table_id}-search-input")))
        .h(px(TOOLBAR_CONTROL_HEIGHT))
        .w(px(TOOLBAR_SEARCH_WIDTH))
        .px_2()
        .rounded(theme.style.radius_control)
        .bg(theme.surface.input_bg)
        .border_1()
        .border_color(if state.filters.search_focused {
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
                .text_color(if state.filters.search_query.is_empty() {
                    theme.text.muted
                } else {
                    theme.text.primary
                })
                .child(if state.filters.search_query.is_empty() {
                    SharedString::from(search_placeholder.to_string())
                } else {
                    SharedString::from(state.filters.search_query.clone())
                }),
        )
        .when(
            !state.filters.search_focused && state.filters.search_query.is_empty(),
            |el| {
                el.child(
                    apply_text_step(div(), theme.typography.size_3xs)
                        .px_1()
                        .rounded(theme.style.radius_control)
                        .bg(theme.surface.control_bg)
                        .border_1()
                        .border_color(theme.border.default)
                        .font(theme.typography.mono_font(FontWeight::MEDIUM))
                        .text_color(theme.text.muted)
                        .child("/"),
                )
            },
        )
        .when(!state.filters.search_query.is_empty(), |el| {
            let hover_bg = theme.surface.bg_hover;
            el.child(
                div()
                    .id(SharedString::from(format!("{table_id}-search-clear")))
                    .w(px(16.0))
                    .h(px(16.0))
                    .rounded(theme.style.radius_control)
                    .flex()
                    .items_center()
                    .justify_center()
                    .cursor_pointer()
                    .hover(move |s| s.bg(hover_bg))
                    .on_mouse_down(MouseButton::Left, move |_, window, cx| {
                        window.prevent_default();
                        cx.stop_propagation();
                        on_clear_search(window, cx);
                    })
                    .child(render_icon(Icon::X, px(11.0), theme.text.muted)),
            )
        })
        .into_any_element()
}

fn render_toolbar_filter_group(
    table_id: &str,
    group: &TableFilterChipGroup,
    state: &DataTableState,
    theme: &OrbitTheme,
    on_extra_filter: TableExtraFilterCallback,
) -> AnyElement {
    let active_val = state
        .filters
        .extra_filters
        .get(group.key)
        .map(String::as_str)
        .unwrap_or(group.default_value);
    let group_key = group.key;

    div()
        .flex()
        .items_center()
        .gap_1()
        .p_0p5()
        .rounded(theme.style.radius_control)
        .border_1()
        .border_color(theme.border.default)
        .bg(theme.surface.bg_panel)
        .children(group.options.iter().map(|opt| {
            let is_active = active_val.eq_ignore_ascii_case(opt.value);
            let opt_val = opt.value.to_string();
            let on_extra = on_extra_filter.clone();
            let hover_bg = theme.surface.control_hover;
            div()
                .id(SharedString::from(format!(
                    "{table_id}-filter-{group_key}-{}",
                    opt.value
                )))
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
                .on_click(move |_, window, cx| {
                    on_extra(group_key.to_string(), opt_val.clone(), window, cx);
                })
                .child(
                    apply_text_step(div(), theme.typography.size_xs)
                        .font_weight(if is_active {
                            FontWeight::SEMIBOLD
                        } else {
                            FontWeight::MEDIUM
                        })
                        .child(opt.label),
                )
        }))
        .into_any_element()
}

fn render_toolbar_columns_button(
    table_id: &str,
    cols_open: bool,
    theme: &OrbitTheme,
    on_toggle_cols: TableActionCallback,
) -> AnyElement {
    let cols_hover = theme.surface.control_hover;
    div()
        .id(SharedString::from(format!("{table_id}-columns-btn")))
        .h(px(TOOLBAR_CONTROL_HEIGHT))
        .px_2p5()
        .rounded(theme.style.radius_control)
        .border_1()
        .border_color(if cols_open {
            theme.accent.accent
        } else {
            theme.border.default
        })
        .bg(if cols_open {
            theme.accent.soft
        } else {
            theme.surface.bg_panel
        })
        .flex()
        .items_center()
        .gap_1p5()
        .cursor_pointer()
        .hover(move |s| s.bg(cols_hover))
        .tooltip(|_, cx| cx.new(|_| TextTooltip::new("Configure columns")).into())
        .on_mouse_down(MouseButton::Left, |_, _, cx| {
            cx.stop_propagation();
        })
        .on_click(move |_, window, cx| on_toggle_cols(window, cx))
        .child(render_icon(
            Icon::SlidersHorizontal,
            px(13.0),
            if cols_open {
                theme.accent.accent
            } else {
                theme.text.secondary
            },
        ))
        .child(
            apply_text_step(div(), theme.typography.size_xs)
                .font_weight(FontWeight::MEDIUM)
                .text_color(if cols_open {
                    theme.accent.accent
                } else {
                    theme.text.secondary
                })
                .child("Columns"),
        )
        .into_any_element()
}

pub fn render_column_config_popover(
    table_id: &str,
    columns: &[TableColumn],
    state: &DataTableState,
    theme: &OrbitTheme,
    callbacks: ColumnConfigCallbacks,
) -> AnyElement {
    let all_unlocked_visible = columns
        .iter()
        .filter(|c| !c.locked)
        .all(|c| !state.hidden_columns.contains(c.id.as_ref()));
    let has_custom = state.has_custom_columns(columns);

    let on_close = callbacks.on_close.clone();
    let on_toggle_all = callbacks.on_toggle_all.clone();
    let on_reset = callbacks.on_reset_defaults.clone();
    let hover_bg = theme.surface.bg_hover;

    let mut panel = div()
        .id(SharedString::from(format!("{table_id}-col-popover")))
        .absolute()
        .top(px(40.0))
        .right(px(16.0))
        .w(px(COLUMN_POPOVER_WIDTH))
        .rounded(theme.style.radius_card)
        .bg(theme.surface.bg_panel)
        .border_1()
        .border_color(theme.border.strong)
        .shadow_lg()
        .p_2()
        .flex()
        .flex_col()
        .gap_1()
        .on_mouse_down(MouseButton::Left, |_, _, cx| {
            cx.stop_propagation();
        });

    // All Columns toggle row
    panel = panel.child(
        div()
            .id(SharedString::from(format!("{table_id}-col-all")))
            .h(px(28.0))
            .px_2()
            .rounded(theme.style.radius_control)
            .flex()
            .items_center()
            .justify_between()
            .cursor_pointer()
            .hover(move |s| s.bg(hover_bg))
            .on_click(move |_, window, cx| {
                on_toggle_all(!all_unlocked_visible, window, cx);
            })
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_2()
                    .child(render_checkbox_box(all_unlocked_visible, false, theme))
                    .child(
                        apply_text_step(div(), theme.typography.size_xs)
                            .font_weight(FontWeight::SEMIBOLD)
                            .text_color(theme.text.primary)
                            .child("All Columns"),
                    ),
            )
            .child(
                apply_text_step(div(), theme.typography.size_xs)
                    .text_color(theme.text.muted)
                    .child(format!(
                        "{}/{}",
                        state.visible_columns(columns).len(),
                        columns.len()
                    )),
            ),
    );

    let mut list = div()
        .id(SharedString::from(format!("{table_id}-col-list")))
        .max_h(px(COLUMN_POPOVER_MAX_HEIGHT))
        .overflow_y_scroll()
        .flex()
        .flex_col()
        .gap_0p5()
        .py_1()
        .border_t_1()
        .border_b_1()
        .border_color(theme.border.subtle);

    for (idx, col) in columns.iter().enumerate() {
        let is_visible = col.locked || !state.hidden_columns.contains(col.id.as_ref());
        let col_id = col.id.to_string();
        let locked = col.locked;
        let on_toggle_col = callbacks.on_toggle_column.clone();
        let row_hover = theme.surface.bg_hover;

        list = list.child(
            div()
                .id(ElementId::NamedInteger(
                    SharedString::from(format!("{table_id}-col-opt")),
                    idx as u64,
                ))
                .h(px(26.0))
                .px_2()
                .rounded(theme.style.radius_control)
                .flex()
                .items_center()
                .justify_between()
                .when(!locked, |el| {
                    el.cursor_pointer().hover(move |s| s.bg(row_hover))
                })
                .when(!locked, |el| {
                    el.on_click(move |_, window, cx| {
                        on_toggle_col(col_id.clone(), locked, window, cx);
                    })
                })
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap_2()
                        .child(render_checkbox_box(is_visible, locked, theme))
                        .child(
                            apply_text_step(div(), theme.typography.size_xs)
                                .text_color(if locked {
                                    theme.text.muted
                                } else {
                                    theme.text.primary
                                })
                                .child(col.header.clone()),
                        ),
                )
                .when(locked, |el| {
                    el.child(render_icon(Icon::Lock, px(11.0), theme.text.muted))
                }),
        );
    }

    panel = panel.child(list);

    let reset_hover = theme.surface.control_hover;
    panel = panel.child(
        div()
            .id(SharedString::from(format!("{table_id}-col-reset")))
            .h(px(26.0))
            .px_2()
            .rounded(theme.style.radius_control)
            .flex()
            .items_center()
            .justify_center()
            .gap_1p5()
            .when(has_custom, |el| {
                el.cursor_pointer()
                    .hover(move |s| s.bg(reset_hover))
                    .on_click(move |_, window, cx| on_reset(window, cx))
            })
            .child(render_icon(
                Icon::RotateCcw,
                px(12.0),
                if has_custom {
                    theme.text.secondary
                } else {
                    theme.text.disabled
                },
            ))
            .child(
                apply_text_step(div(), theme.typography.size_xs)
                    .font_weight(FontWeight::MEDIUM)
                    .text_color(if has_custom {
                        theme.text.secondary
                    } else {
                        theme.text.disabled
                    })
                    .child("Reset to Defaults"),
            ),
    );

    div()
        .id(SharedString::from(format!("{table_id}-col-popover-backdrop")))
        .absolute()
        .inset_0()
        .on_mouse_down(MouseButton::Left, move |_, window, cx| {
            cx.stop_propagation();
            on_close(window, cx);
        })
        .child(panel)
        .into_any_element()
}

fn render_checkbox_box(checked: bool, disabled: bool, theme: &OrbitTheme) -> AnyElement {
    let bg = if checked {
        if disabled {
            theme.surface.bg_hover
        } else {
            theme.accent.accent
        }
    } else {
        theme.surface.input_bg
    };
    let border = if checked && !disabled {
        theme.accent.accent
    } else {
        theme.border.default
    };

    div()
        .w(px(14.0))
        .h(px(14.0))
        .rounded(theme.style.radius_control)
        .bg(bg)
        .border_1()
        .border_color(border)
        .flex()
        .items_center()
        .justify_center()
        .when(checked, |el| {
            el.child(render_icon(
                Icon::Check,
                px(10.0),
                if disabled {
                    theme.text.muted
                } else {
                    theme.text.on_accent
                },
            ))
        })
        .into_any_element()
}

pub fn render_table_paginator(
    table_id: &str,
    slice: &PageSlice,
    page_size: usize,
    theme: &OrbitTheme,
    on_page_change: Option<PageChangeCallback>,
) -> AnyElement {
    let report = apply_text_step(div(), theme.typography.size_xs)
        .font(theme.typography.mono_font(FontWeight::NORMAL))
        .text_color(theme.text.secondary)
        .child(format_paginator_report(slice));

    let size_pills = render_paginator_size_pills(table_id, page_size, theme, on_page_change.clone());
    let nav_controls = render_paginator_nav_controls(table_id, slice, page_size, theme, on_page_change);

    div()
        .id(SharedString::from(format!("{table_id}-paginator")))
        .h(px(TABLE_PAGINATOR_HEIGHT))
        .px_4()
        .flex_shrink_0()
        .w_full()
        .bg(theme.surface.bg_panel)
        .border_t_1()
        .border_color(theme.border.default)
        .flex()
        .items_center()
        .justify_between()
        .child(report)
        .child(
            div()
                .flex()
                .items_center()
                .gap_4()
                .child(size_pills)
                .child(nav_controls),
        )
        .into_any_element()
}

fn render_paginator_size_pills(
    table_id: &str,
    page_size: usize,
    theme: &OrbitTheme,
    on_page_change: Option<PageChangeCallback>,
) -> AnyElement {
    let effective_size = page_size.max(1);
    div()
        .flex()
        .items_center()
        .gap_1()
        .child(
            apply_text_step(div(), theme.typography.size_xs)
                .text_color(theme.text.muted)
                .mr_1()
                .child("Rows:"),
        )
        .children(ALLOWED_TABLE_PAGE_SIZES.iter().map(|&sz| {
            let active = sz == effective_size;
            let cb = on_page_change.clone();
            let hover_bg = theme.surface.control_hover;
            div()
                .id(SharedString::from(format!("{table_id}-pg-size-{sz}")))
                .h(px(22.0))
                .px_1p5()
                .rounded(theme.style.radius_control)
                .flex()
                .items_center()
                .cursor_pointer()
                .when(active, |el| {
                    el.bg(theme.accent.soft).text_color(theme.accent.accent)
                })
                .when(!active, |el| {
                    el.text_color(theme.text.secondary)
                        .hover(move |s| s.bg(hover_bg))
                })
                .when_some(cb, |el, cb| {
                    el.on_click(move |_, window, cx| cb(0, sz, window, cx))
                })
                .child(
                    apply_text_step(div(), theme.typography.size_xs)
                        .font(theme.typography.mono_font(if active {
                            FontWeight::SEMIBOLD
                        } else {
                            FontWeight::NORMAL
                        }))
                        .child(sz.to_string()),
                )
        }))
        .into_any_element()
}

fn render_paginator_nav_controls(
    table_id: &str,
    slice: &PageSlice,
    page_size: usize,
    theme: &OrbitTheme,
    on_page_change: Option<PageChangeCallback>,
) -> AnyElement {
    let at_first = slice.page == 0;
    let at_last = slice.total_records == 0 || slice.page + 1 >= slice.total_pages;
    let effective_size = page_size.max(1);
    let last_first = slice.total_pages.saturating_sub(1) * effective_size;

    let nav_btn = |id_suffix: &str,
                   icon: Icon,
                   tooltip_label: &'static str,
                   disabled: bool,
                   target_first: usize| {
        let cb = on_page_change.clone();
        let hover_bg = theme.surface.control_hover;
        div()
            .id(SharedString::from(format!("{table_id}-pg-{id_suffix}")))
            .w(px(24.0))
            .h(px(24.0))
            .rounded(theme.style.radius_control)
            .flex()
            .items_center()
            .justify_center()
            .tooltip(move |_, cx| cx.new(|_| TextTooltip::new(tooltip_label)).into())
            .when(!disabled, |el| {
                el.cursor_pointer().hover(move |s| s.bg(hover_bg))
            })
            .when(!disabled, |el| {
                el.when_some(cb, |el, cb| {
                    el.on_click(move |_, window, cx| cb(target_first, effective_size, window, cx))
                })
            })
            .child(render_icon(
                icon,
                px(13.0),
                if disabled {
                    theme.text.disabled
                } else {
                    theme.text.secondary
                },
            ))
    };

    div()
        .flex()
        .items_center()
        .gap_1()
        .child(nav_btn("first", Icon::ChevronsLeft, "First page", at_first, 0))
        .child(nav_btn(
            "prev",
            Icon::ChevronLeft,
            "Previous page",
            at_first,
            slice.clamped_first.saturating_sub(effective_size),
        ))
        .child(
            apply_text_step(div(), theme.typography.size_xs)
                .font(theme.typography.mono_font(FontWeight::MEDIUM))
                .text_color(theme.text.primary)
                .px_2()
                .child(format!("{} / {}", slice.page + 1, slice.total_pages)),
        )
        .child(nav_btn(
            "next",
            Icon::ChevronRight,
            "Next page",
            at_last,
            (slice.clamped_first + effective_size).min(last_first),
        ))
        .child(nav_btn(
            "last",
            Icon::ChevronsRight,
            "Last page",
            at_last,
            last_first,
        ))
        .into_any_element()
}

#[allow(clippy::too_many_arguments)]
pub fn render_enhanced_data_table<K, C>(
    table_id: impl Into<SharedString>,
    search_placeholder: &str,
    filter_groups: &[TableFilterChipGroup],
    columns: &[TableColumn],
    row_count: usize,
    state: &DataTableState,
    theme: &OrbitTheme,
    mut callbacks: DataTableCallbacks<K, C>,
    enhancements: Option<DataTableEnhancementCallbacks>,
) -> AnyElement
where
    K: 'static + Fn(usize) -> String,
    C: 'static + Fn(usize, &TableColumn, &OrbitTheme) -> AnyElement,
{
    let table_id = table_id.into();
    let Some(enh) = enhancements else {
        return render_data_table(table_id, columns, row_count, state, theme, callbacks);
    };

    callbacks = callbacks
        .on_page_change(move |first, size, window, cx| {
            (enh.on_page_change)(first, size, window, cx);
        })
        .on_column_resize_start(move |col_id, pointer_x, window, cx| {
            (enh.on_column_resize_start)(col_id, pointer_x, window, cx);
        });
    let toolbar = render_table_toolbar(
        table_id.as_ref(),
        search_placeholder,
        filter_groups,
        state,
        theme,
        enh.toolbar,
    );
    let table = render_data_table(
        table_id.clone(),
        columns,
        row_count,
        state,
        theme,
        callbacks,
    );
    let popover = if state.config_popover_open {
        Some(render_column_config_popover(
            table_id.as_ref(),
            columns,
            state,
            theme,
            enh.column_config,
        ))
    } else {
        None
    };

    div()
        .relative()
        .flex_1()
        .min_h(px(0.0))
        .flex()
        .flex_col()
        .child(toolbar)
        .child(table)
        .when_some(popover, |el, pop| el.child(pop))
        .into_any_element()
}

#[derive(IntoElement)]
struct KitDataTableSync {
    entity: Entity<TableState<OrbitTableDelegate>>,
    columns: Vec<TableColumn>,
    kit_columns: Vec<KitColumn>,
    row_count: usize,
    row_offset: usize,
    row_key: RowKeyFn,
    render_cell: RenderCellFn,
    selected_row_key: Option<String>,
    sort: Option<SortState>,
    theme: OrbitTheme,
    on_sort: Option<SortCallback>,
    on_row_click: Option<RowClickCallback>,
    on_row_context_menu: Option<RowContextMenuCallback>,
    on_row_quick_yaml: Option<RowClickCallback>,
}

impl RenderOnce for KitDataTableSync {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let entity = self.entity.clone();
        entity.update(cx, |table, cx| {
            let needs_refresh = table.delegate().kit_columns.len() != self.kit_columns.len()
                || table
                    .delegate()
                    .kit_columns
                    .iter()
                    .zip(self.kit_columns.iter())
                    .any(|(prev, next)| {
                        prev.key != next.key || prev.width != next.width || prev.sort != next.sort
                    });

            let bridged_sort = self.on_sort.clone().map(|legacy_cb| -> KitSortCallback {
                Rc::new(move |col_id, _dir, window, cx| {
                    let event = ClickEvent::Keyboard(gpui::KeyboardClickEvent::default());
                    (legacy_cb)(col_id, &event, window, cx);
                })
            });

            let delegate = table.delegate_mut();
            delegate.sync_from(
                self.columns,
                self.row_count,
                self.row_offset,
                self.row_key,
                self.render_cell,
                self.selected_row_key,
                self.sort,
                self.kit_columns,
                self.theme,
            );
            delegate.on_sort = bridged_sort;
            delegate.on_row_click = self.on_row_click;
            delegate.on_row_context_menu = self.on_row_context_menu;
            delegate.on_row_quick_yaml = self.on_row_quick_yaml;

            if needs_refresh {
                table.refresh(cx);
            }
        });

        div()
            .flex_1()
            .w_full()
            .min_h(px(0.0))
            .child(KitDataTable::new(&entity).stripe(false).bordered(true))
    }
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
    let visible_cols = state.resolved_visible_columns(columns);

    let page_slice = if state.pagination.enabled {
        Some(paginate_rows_range(
            row_count,
            state.pagination.first,
            state.pagination.page_size,
            state.pagination.mode,
        ))
    } else {
        None
    };

    let (render_count, row_offset) = match &page_slice {
        Some(slice) => {
            let offset = match state.pagination.mode {
                PaginationMode::Client => slice.range.start,
                PaginationMode::Lazy { .. } => 0,
            };
            (slice.range.len(), offset)
        }
        None => (row_count, 0),
    };

    let body_mode = state.body_mode(render_count);
    let uses_kit_table = state.kit_table.is_some()
        && matches!(
            body_mode,
            TableBodyMode::Rows | TableBodyMode::RefreshingRows
        );
    let header = if uses_kit_table {
        None
    } else {
        Some(render_table_header(
            table_id_str.as_ref(),
            &visible_cols,
            state,
            theme,
            callbacks.on_sort.clone(),
            callbacks.on_column_resize_start.clone(),
        ))
    };

    let body = match body_mode {
        TableBodyMode::LoadingSkeleton => render_loading_skeleton(
            table_id_str.as_ref(),
            &visible_cols,
            DEFAULT_SKELETON_ROWS,
            theme,
        ),
        TableBodyMode::Empty => render_empty_state_with_action(
            table_id_str.as_ref(),
            state.empty_message.clone(),
            callbacks.on_empty_action.as_ref(),
            theme,
        ),
        TableBodyMode::Rows | TableBodyMode::RefreshingRows => {
            if let Some(kit_table) = state.kit_table.clone() {
                let kit_columns = visible_cols
                    .iter()
                    .map(|col| to_kit_column(col, state))
                    .collect();
                KitDataTableSync {
                    entity: kit_table,
                    columns: visible_cols,
                    kit_columns,
                    row_count: render_count,
                    row_offset,
                    row_key: Rc::new(callbacks.row_key),
                    render_cell: Rc::new(callbacks.render_cell),
                    selected_row_key: state.selected_row_key.clone(),
                    sort: state.sort.clone(),
                    theme: theme.clone(),
                    on_sort: callbacks.on_sort.clone(),
                    on_row_click: callbacks.on_row_click.clone(),
                    on_row_context_menu: callbacks.on_row_context_menu.clone(),
                    on_row_quick_yaml: callbacks.on_row_quick_yaml.clone(),
                }
                .into_any_element()
            } else {
                let cols: Rc<[TableColumn]> = Rc::from(visible_cols);
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
                    render_count,
                    move |range: Range<usize>, _window: &mut Window, _cx: &mut App| {
                        let mut items = Vec::with_capacity(range.len());
                        let surface = resolve_data_table_surface_style(&theme_snapshot);
                        let hover_bg = theme_snapshot.surface.bg_hover;
                        let selected_bg = theme_snapshot.accent.soft;
                        let border_subtle = surface.row_separator;
                        let accent_bar = theme_snapshot.accent.accent;

                        for local_idx in range {
                            let row_idx = row_offset + local_idx;
                            let key = (row_key_fn)(row_idx);
                            let is_selected = selected_key.as_deref() == Some(key.as_str());
                            let key_shared = SharedString::from(key);
                            let row_group =
                                SharedString::from(format!("{row_group_prefix}-{row_idx}"));

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
                                        click_cb(
                                            row_idx,
                                            row_key_for_click.clone(),
                                            event,
                                            window,
                                            cx,
                                        );
                                    });
                            }

                            if let Some(ctx_cb) = on_row_context_menu.clone() {
                                let row_key_for_ctx = key_shared.clone();
                                row_el = row_el.on_mouse_down(
                                    MouseButton::Right,
                                    move |event, window, cx| {
                                        cx.stop_propagation();
                                        ctx_cb(
                                            row_idx,
                                            row_key_for_ctx.clone(),
                                            event,
                                            window,
                                            cx,
                                        );
                                    },
                                );
                            }

                            for col in cols.iter() {
                                let cell_content = (render_cell_fn)(row_idx, col, &theme_snapshot);
                                row_el = row_el.child(render_table_cell(
                                    col,
                                    &theme_snapshot,
                                    cell_content,
                                ));
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
                                            .child(render_icon(
                                                Icon::FileCode,
                                                px(13.0),
                                                icon_color,
                                            )),
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
                                            .child(render_icon(
                                                Icon::Ellipsis,
                                                px(13.0),
                                                icon_color,
                                            )),
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
                .track_scroll(&state.scroll_handle)
                .flex_1()
                .w_full()
                .min_h(px(0.0))
                .into_any_element()
            }
        }
    };

    let surface = resolve_data_table_surface_style(theme);
    let paginator = page_slice.as_ref().map(|slice| {
        render_table_paginator(
            table_id_str.as_ref(),
            slice,
            state.pagination.page_size,
            theme,
            callbacks.on_page_change.clone(),
        )
    });

    let refreshing_bar_id = SharedString::from(format!("{table_id_str}-refreshing-bar"));
    div()
        .id(table_id_str)
        .relative()
        .bg(surface.background)
        .border_t_1()
        .border_color(surface.top_hairline)
        .flex()
        .flex_col()
        .flex_1()
        .w_full()
        .min_h(px(0.0))
        .overflow_hidden()
        .when(body_mode == TableBodyMode::RefreshingRows, |el| {
            el.child(
                div()
                    .id(refreshing_bar_id)
                    .absolute()
                    .top_0()
                    .left_0()
                    .right_0()
                    .h(px(2.0))
                    .bg(theme.accent.accent),
            )
        })
        .when_some(header, |el, header| el.child(header))
        .child(body)
        .when_some(paginator, |el, paginator| el.child(paginator))
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
    fn test_data_table_body_mode_distinguishes_cold_load_empty_rows_and_warm_refresh() {
        let mut state = DataTableState::new();

        state.loading = true;
        state.refreshing = false;
        assert_eq!(state.body_mode(0), TableBodyMode::LoadingSkeleton);

        state.loading = false;
        state.refreshing = false;
        assert_eq!(state.body_mode(0), TableBodyMode::Empty);
        assert_eq!(state.body_mode(5), TableBodyMode::Rows);

        state.loading = false;
        state.refreshing = true;
        assert_eq!(state.body_mode(5), TableBodyMode::RefreshingRows);

        // Populated table must NEVER revert to loading skeleton during refresh
        state.loading = true;
        state.refreshing = true;
        assert_eq!(state.body_mode(5), TableBodyMode::RefreshingRows);
    }

    #[test]
    fn test_data_table_keyboard_row_selection_navigation_and_pagination_sync() {
        let keys: Vec<String> = ["r0", "r1", "r2", "r3", "r4"]
            .into_iter()
            .map(String::from)
            .collect();
        let mut state = DataTableState::new().with_pagination();
        state.pagination.page_size = 2;

        // Empty list returns None and leaves selection untouched
        assert_eq!(state.move_selection_by_delta(1, &[]), None);
        assert_eq!(state.selected_row_key, None);

        // Down from unselected -> selects first row
        assert_eq!(state.move_selection_by_delta(1, &keys), Some(0));
        assert_eq!(state.selected_row_key.as_deref(), Some("r0"));
        assert_eq!(state.pagination.first, 0);

        // Down -> r1 (same page)
        assert_eq!(state.move_selection_by_delta(1, &keys), Some(1));
        assert_eq!(state.selected_row_key.as_deref(), Some("r1"));
        assert_eq!(state.pagination.first, 0);

        // Down -> r2 (advances to page 2)
        assert_eq!(state.move_selection_by_delta(1, &keys), Some(2));
        assert_eq!(state.selected_row_key.as_deref(), Some("r2"));
        assert_eq!(state.pagination.first, 2);

        // Up -> r1 (returns to page 1)
        assert_eq!(state.move_selection_by_delta(-1, &keys), Some(1));
        assert_eq!(state.selected_row_key.as_deref(), Some("r1"));
        assert_eq!(state.pagination.first, 0);

        // Clamp at top
        assert_eq!(state.move_selection_by_delta(-1, &keys), Some(0));
        assert_eq!(state.move_selection_by_delta(-1, &keys), Some(0));
        assert_eq!(state.selected_row_key.as_deref(), Some("r0"));

        // Clamp at bottom
        state.selected_row_key = Some("r4".to_string());
        assert_eq!(state.move_selection_by_delta(1, &keys), Some(4));
        assert_eq!(state.selected_row_key.as_deref(), Some("r4"));
        assert_eq!(state.pagination.first, 4);

        // Up from unselected -> selects last row
        state.clear_selection();
        assert_eq!(state.move_selection_by_delta(-1, &keys), Some(4));
        assert_eq!(state.selected_row_key.as_deref(), Some("r4"));
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

    #[test]
    fn test_paginate_rows_range_client_slicing_and_clamping() {
        let first_page = paginate_rows_range(95, 0, 25, PaginationMode::Client);
        assert_eq!(first_page.range, 0..25);
        assert_eq!(first_page.clamped_first, 0);
        assert_eq!(first_page.page, 0);
        assert_eq!(first_page.total_pages, 4);
        assert_eq!(first_page.total_records, 95);

        let last_page = paginate_rows_range(95, 75, 25, PaginationMode::Client);
        assert_eq!(last_page.range, 75..95);
        assert_eq!(last_page.clamped_first, 75);
        assert_eq!(last_page.page, 3);
        assert_eq!(last_page.total_pages, 4);

        let clamped = paginate_rows_range(95, 150, 25, PaginationMode::Client);
        assert_eq!(clamped.range, 75..95);
        assert_eq!(clamped.clamped_first, 75);
        assert_eq!(clamped.page, 3);
        assert_eq!(clamped.total_pages, 4);

        let empty = paginate_rows_range(0, 50, 25, PaginationMode::Client);
        assert_eq!(empty.range, 0..0);
        assert_eq!(empty.clamped_first, 0);
        assert_eq!(empty.page, 0);
        assert_eq!(empty.total_pages, 1);
        assert_eq!(empty.total_records, 0);
    }

    #[test]
    fn test_paginate_rows_range_lazy_mode_uses_total_records_without_reslicing() {
        let lazy = paginate_rows_range(
            25,
            50,
            25,
            PaginationMode::Lazy { total_records: 250 },
        );
        assert_eq!(lazy.range, 0..25);
        assert_eq!(lazy.clamped_first, 50);
        assert_eq!(lazy.page, 2);
        assert_eq!(lazy.total_pages, 10);
        assert_eq!(lazy.total_records, 250);
    }

    #[test]
    fn test_column_visibility_respects_locked_columns_and_reset_defaults() {
        let columns = vec![
            TableColumn::new("name", "Name").locked(true),
            TableColumn::new("status", "Status"),
            TableColumn::new("node", "Node").default_visible(false),
        ];

        let mut state = DataTableState::new();
        state.reset_columns(&columns);
        assert!(!state.has_custom_columns(&columns));
        assert_eq!(
            state
                .visible_columns(&columns)
                .iter()
                .map(|c| c.id.as_ref())
                .collect::<Vec<_>>(),
            vec!["name", "status"]
        );

        // Showing all columns when one has default_visible(false) is a custom state
        state.set_all_columns_visible(&columns, true);
        assert!(state.has_custom_columns(&columns));
        state.reset_columns(&columns);
        assert!(!state.has_custom_columns(&columns));

        // Locked column ignores toggle_column
        assert!(!state.toggle_column("name", true));
        assert!(!state.hidden_columns.contains("name"));

        // Non-locked column toggles
        assert!(state.toggle_column("status", false));
        assert!(state.hidden_columns.contains("status"));
        assert!(state.has_custom_columns(&columns));

        // set_all_columns_visible(false) keeps locked column visible
        state.set_all_columns_visible(&columns, false);
        assert!(!state.hidden_columns.contains("name"));
        assert!(state.hidden_columns.contains("status"));
        assert!(state.hidden_columns.contains("node"));

        // reset_columns restores default_visible
        state.reset_columns(&columns);
        assert!(!state.has_custom_columns(&columns));
        assert!(!state.hidden_columns.contains("status"));
        assert!(state.hidden_columns.contains("node"));
    }

    #[test]
    fn test_table_filter_state_active_detection_clear_and_page_reset() {
        let mut state = DataTableState::new().with_pagination();
        state.pagination.first = 50;

        assert!(!state.has_active_filters(&[("status", "All")]));

        state.set_search_query("nginx");
        assert_eq!(state.pagination.first, 0);
        assert!(state.has_active_filters(&[("status", "All")]));

        state.pagination.first = 25;
        state.set_extra_filter("status", "Running");
        assert_eq!(state.pagination.first, 0);
        assert!(state.has_active_filters(&[("status", "All")]));

        state.pagination.first = 75;
        state.clear_filters();
        assert_eq!(state.pagination.first, 0);
        assert!(state.filters.search_query.is_empty());
        assert!(state.filters.extra_filters.is_empty());
        assert!(!state.has_active_filters(&[("status", "All")]));
    }

    #[test]
    fn test_data_table_state_hydrates_and_serializes_persisted_preferences() {
        use orbit_core::config::{
            PersistedClusterTableFilters, PersistedSortDirection, PersistedTableConfig,
            PersistedTablePreferences,
        };
        use std::collections::HashMap;

        let mut prefs = PersistedTablePreferences::default();
        prefs.tables.insert(
            "pods".to_string(),
            PersistedTableConfig {
                page_size: Some(100),
                hidden_columns: vec!["node".to_string()],
                sort_column: Some("name".to_string()),
                sort_direction: Some(PersistedSortDirection::Desc),
                ..Default::default()
            },
        );
        let mut extra = HashMap::new();
        extra.insert("status".to_string(), "Running".to_string());
        let mut cluster_map = HashMap::new();
        cluster_map.insert(
            "pods".to_string(),
            PersistedClusterTableFilters {
                search_query: "api".to_string(),
                extra_filters: extra,
            },
        );
        prefs.clusters.insert("prod-ctx".to_string(), cluster_map);

        let columns = vec![
            TableColumn::new("name", "Name").locked(true),
            TableColumn::new("node", "Node"),
        ];

        let mut state = DataTableState::new().with_pagination();
        state.hydrate_from_preferences("pods", Some("prod-ctx"), &prefs, &columns);

        assert_eq!(state.pagination.page_size, 100);
        assert!(state.hidden_columns.contains("node"));
        assert_eq!(
            state.sort,
            Some(SortState {
                column_id: "name".to_string(),
                direction: SortDirection::Desc,
            })
        );
        assert_eq!(state.filters.search_query, "api");
        assert_eq!(
            state.filters.extra_filters.get("status").map(String::as_str),
            Some("Running")
        );

        let config_out = state.to_persisted_config();
        assert_eq!(config_out.page_size, Some(100));
        assert_eq!(config_out.hidden_columns, vec!["node".to_string()]);
        assert_eq!(config_out.sort_column.as_deref(), Some("name"));
        assert_eq!(config_out.sort_direction, Some(PersistedSortDirection::Desc));

        let filters_out = state.to_persisted_filters();
        assert_eq!(filters_out.search_query, "api");
        assert_eq!(
            filters_out.extra_filters.get("status").map(String::as_str),
            Some("Running")
        );
    }

    #[test]
    fn test_paginator_report_label_formats_populated_and_empty_ranges() {
        let populated = paginate_rows_range(95, 0, 25, PaginationMode::Client);
        assert_eq!(format_paginator_report(&populated), "Showing 1–25 of 95");

        let partial = paginate_rows_range(95, 75, 25, PaginationMode::Client);
        assert_eq!(format_paginator_report(&partial), "Showing 76–95 of 95");

        let empty = paginate_rows_range(0, 0, 25, PaginationMode::Client);
        assert_eq!(format_paginator_report(&empty), "Showing 0 of 0");
    }

    #[test]
    fn test_data_table_callbacks_accept_optional_page_change_handler() {
        let callbacks = DataTableCallbacks::new(
            |idx| format!("row-{idx}"),
            |_, _, _| div().into_any_element(),
        )
        .on_page_change(|_, _, _, _| {});
        assert!(callbacks.on_page_change.is_some());
    }

    #[test]
    fn test_visible_columns_filters_out_hidden_but_retains_locked_columns_for_render() {
        let cols = vec![
            TableColumn::new("name", "Name").locked(true),
            TableColumn::new("namespace", "Namespace"),
            TableColumn::new("status", "Status"),
            TableColumn::new("age", "Age"),
        ];
        let mut state = DataTableState::new();
        state.hidden_columns.insert("name".to_string()); // should be retained due to locked
        state.hidden_columns.insert("status".to_string());

        let visible = state.visible_columns(&cols);
        assert_eq!(
            visible.iter().map(|c| c.id.as_ref()).collect::<Vec<_>>(),
            vec!["name", "namespace", "age"]
        );
    }

    #[test]
    fn test_column_width_clamping_and_layout_resolution() {
        assert_eq!(clamp_column_width(10.0, DEFAULT_MIN_COLUMN_WIDTH), DEFAULT_MIN_COLUMN_WIDTH);
        assert_eq!(clamp_column_width(2500.0, DEFAULT_MIN_COLUMN_WIDTH), MAX_COLUMN_WIDTH);
        assert_eq!(clamp_column_width(f32::NAN, DEFAULT_MIN_COLUMN_WIDTH), DEFAULT_MIN_COLUMN_WIDTH);
        assert_eq!(
            clamp_column_width(f32::INFINITY, DEFAULT_MIN_COLUMN_WIDTH),
            DEFAULT_MIN_COLUMN_WIDTH
        );
        assert_eq!(clamp_column_width(240.0, DEFAULT_MIN_COLUMN_WIDTH), 240.0);

        let col_flex = TableColumn::new("name", "Name").flex_width(2.0, px(160.0));
        let col_fixed = TableColumn::new("age", "Age").fixed_width(px(80.0));

        let mut state = DataTableState::new();
        assert_eq!(state.resolve_column_width(&col_flex), col_flex.width);
        assert_eq!(state.resolve_column_width(&col_fixed), col_fixed.width);

        state.column_widths.insert("name".to_string(), 320.0);
        state.column_widths.insert("age".to_string(), 12.0);
        assert_eq!(
            state.resolve_column_width(&col_flex),
            ColumnWidth::Fixed(px(320.0))
        );
        assert_eq!(
            state.resolve_column_width(&col_fixed),
            ColumnWidth::Fixed(px(DEFAULT_MIN_COLUMN_WIDTH))
        );
    }

    #[test]
    fn test_column_resize_drag_lifecycle_and_flex_estimation() {
        let cols = vec![
            TableColumn::new("name", "Name").flex_width(2.0, px(140.0)),
            TableColumn::new("status", "Status").flex_width(1.0, px(100.0)),
            TableColumn::new("age", "Age").fixed_width(px(80.0)),
        ];
        let mut state = DataTableState::new();
        // container_width = 680.0 => fixed = 80.0 => remaining = 600.0 => name = 400.0, status = 200.0
        let est_name = state.estimate_rendered_column_width("name", &cols, 680.0);
        let est_status = state.estimate_rendered_column_width("status", &cols, 680.0);
        let est_age = state.estimate_rendered_column_width("age", &cols, 680.0);
        assert!((est_name - 400.0).abs() < 0.01);
        assert!((est_status - 200.0).abs() < 0.01);
        assert!((est_age - 80.0).abs() < 0.01);

        assert!(state.start_column_resize("name", 500.0, &cols, 680.0));
        assert!(state.active_resize.is_some());
        assert_eq!(state.column_widths.get("name").copied(), Some(400.0));

        assert!(state.update_column_resize(560.0));
        assert_eq!(state.column_widths.get("name").copied(), Some(460.0));

        // Negative delta clamped at floor
        assert!(state.update_column_resize(50.0));
        assert_eq!(
            state.column_widths.get("name").copied(),
            Some(DEFAULT_MIN_COLUMN_WIDTH)
        );

        assert!(state.end_column_resize());
        assert!(!state.end_column_resize());
        assert!(state.active_resize.is_none());
    }

    #[test]
    fn test_column_widths_hydration_serialization_and_reset_defaults() {
        use orbit_core::config::{PersistedTableConfig, PersistedTablePreferences};
        use std::collections::HashMap;

        let cols = vec![
            TableColumn::new("name", "Name").locked(true),
            TableColumn::new("status", "Status"),
        ];
        let mut widths = HashMap::new();
        widths.insert("name".to_string(), 310);
        widths.insert("status".to_string(), 10);
        widths.insert("unknown".to_string(), 200);

        let mut prefs = PersistedTablePreferences::default();
        prefs.tables.insert(
            "pods".to_string(),
            PersistedTableConfig {
                column_widths: widths,
                ..Default::default()
            },
        );

        let mut state = DataTableState::new();
        state.hydrate_from_preferences("pods", None, &prefs, &cols);
        assert_eq!(state.column_widths.get("name").copied(), Some(310.0));
        assert_eq!(
            state.column_widths.get("status").copied(),
            Some(DEFAULT_MIN_COLUMN_WIDTH)
        );
        assert!(!state.column_widths.contains_key("unknown"));
        assert!(state.has_custom_columns(&cols));

        let persisted = state.to_persisted_config();
        assert_eq!(persisted.column_widths.get("name").copied(), Some(310));
        assert_eq!(
            persisted.column_widths.get("status").copied(),
            Some(DEFAULT_MIN_COLUMN_WIDTH as u32)
        );

        state.reset_columns(&cols);
        assert!(state.column_widths.is_empty());
        assert!(state.active_resize.is_none());
        assert!(!state.has_custom_columns(&cols));
        assert!(state.to_persisted_config().column_widths.is_empty());
    }

    #[test]
    fn test_data_table_callbacks_accept_optional_column_resize_start_handler() {
        let default_callbacks =
            DataTableCallbacks::new(|idx| format!("row-{idx}"), |_, _, _| div().into_any_element());
        assert!(default_callbacks.on_column_resize_start.is_none());

        let with_resize = DataTableCallbacks::new(
            |idx| format!("row-{idx}"),
            |_, _, _| div().into_any_element(),
        )
        .on_column_resize_start(|_, _, _, _| {});
        assert!(with_resize.on_column_resize_start.is_some());

        let cols = vec![
            TableColumn::new("name", "Name").flex_width(2.0, px(140.0)),
            TableColumn::new("age", "Age").fixed_width(px(80.0)),
        ];
        let mut state = DataTableState::new();
        state.column_widths.insert("name".to_string(), 260.0);
        let resolved = state.resolved_visible_columns(&cols);
        assert_eq!(resolved.len(), 2);
        assert_eq!(resolved[0].width, ColumnWidth::Fixed(px(260.0)));
        assert_eq!(resolved[1].width, ColumnWidth::Fixed(px(80.0)));
    }

    #[test]
    fn test_to_kit_column_maps_width_bounds_alignment_and_sort() {
        let col = TableColumn::new("cpu", "CPU")
            .flex_width(1.0, px(80.0))
            .right_numeric();
        let mut state = DataTableState::new();
        state.column_widths.insert("cpu".to_string(), 140.0);
        state.sort = Some(SortState {
            column_id: "cpu".to_string(),
            direction: SortDirection::Desc,
        });

        let kit_col = to_kit_column(&col, &state);
        assert_eq!(kit_col.key.as_ref(), "cpu");
        assert_eq!(kit_col.name.as_ref(), "CPU");
        assert_eq!(kit_col.width, px(140.0));
        assert_eq!(kit_col.min_width, px(80.0));
        assert_eq!(kit_col.max_width, px(MAX_COLUMN_WIDTH));
        assert_eq!(kit_col.align, gpui::TextAlign::Right);
        assert_eq!(kit_col.sort, Some(gpui::component::table::ColumnSort::Descending));
        assert!(kit_col.resizable);
        assert!(!kit_col.movable);

        let non_sortable = TableColumn::new("actions", "").sortable(false);
        assert_eq!(column_sort_from_state(&non_sortable, state.sort.as_ref()), None);
        assert_eq!(
            sort_direction_from_kit(gpui::component::table::ColumnSort::Ascending),
            Some(SortDirection::Asc)
        );
        assert_eq!(
            sort_direction_from_kit(gpui::component::table::ColumnSort::Descending),
            Some(SortDirection::Desc)
        );
        assert_eq!(
            sort_direction_from_kit(gpui::component::table::ColumnSort::Default),
            None
        );
    }

    #[gpui::test]
    fn test_orbit_table_delegate_counts_columns_rows_and_sorts(cx: &mut gpui::TestAppContext) {
        type SortSpy = Rc<std::cell::RefCell<Option<(String, Option<SortDirection>)>>>;
        let sort_spy: SortSpy = Rc::new(std::cell::RefCell::new(None));
        let sort_spy_clone = sort_spy.clone();

        let cols = vec![
            TableColumn::new("name", "Name").flex_width(2.0, px(120.0)),
            TableColumn::new("status", "Status").fixed_width(px(100.0)),
            TableColumn::new("age", "Age").right_numeric(),
        ];
        let state = DataTableState::new();
        let theme = OrbitTheme::from_config(
            &orbit_core::config::OrbitConfig::default(),
            gpui::WindowAppearance::Dark,
        );

        let delegate = OrbitTableDelegate::new(
            cols,
            5,
            2,
            Rc::new(|idx| format!("row-{idx}")),
            Rc::new(|row_idx, col, _| div().child(format!("{row_idx}:{}", col.id)).into_any_element()),
            &state,
            theme,
        )
        .with_on_sort(Rc::new(move |col_id, dir, _window, _cx| {
            *sort_spy_clone.borrow_mut() = Some((col_id.to_string(), dir));
        }));

        let window_handle = cx.open_window(Default::default(), |_, _| gpui::Empty);

        let table_entity = window_handle
            .update(cx, |_, window, cx| {
                cx.new(|cx| TableState::new(delegate, window, cx))
            })
            .expect("table entity should be created");

        window_handle
            .update(cx, |_, window, cx| {
                table_entity.update(cx, |table, cx| {
                    assert_eq!(table.delegate().columns_count(cx), 3);
                    assert_eq!(table.delegate().rows_count(cx), 5);
                    assert_eq!(table.delegate().column(1, cx).key.as_ref(), "status");

                    let fitted = table.delegate_mut().fit_column(0, window, cx);
                    assert!(fitted.is_some());
                    let fitted_px = f32::from(fitted.unwrap());
                    assert!((120.0..=MAX_COLUMN_WIDTH).contains(&fitted_px));

                    table
                        .delegate_mut()
                        .perform_sort(2, ColumnSort::Descending, window, cx);
                });
            })
            .expect("table update should succeed");

        assert_eq!(
            *sort_spy.borrow(),
            Some(("age".to_string(), Some(SortDirection::Desc)))
        );
    }

    #[test]
    fn test_apply_column_widths_changed_clamps_and_persists() {
        let cols = vec![
            TableColumn::new("name", "Name").fixed_width(px(120.0)),
            TableColumn::new("status", "Status").fixed_width(px(100.0)),
            TableColumn::new("age", "Age").right_numeric(),
        ];
        let mut state = DataTableState::new();
        let widths = vec![px(10.0), px(220.0), px(9999.0)];

        assert!(state.apply_kit_column_widths(&cols, &widths));
        assert_eq!(
            state.column_widths.get("name").copied(),
            Some(cols[0].min_resize_width())
        );
        assert_eq!(state.column_widths.get("status").copied(), Some(220.0));
        assert_eq!(
            state.column_widths.get("age").copied(),
            Some(MAX_COLUMN_WIDTH)
        );

        let persisted = state.to_persisted_config();
        assert_eq!(
            persisted.column_widths.get("name").copied(),
            Some(cols[0].min_resize_width().round() as u32)
        );
        assert_eq!(persisted.column_widths.get("status").copied(), Some(220));
        assert_eq!(
            persisted.column_widths.get("age").copied(),
            Some(MAX_COLUMN_WIDTH.round() as u32)
        );

        // Second identical call reports no changes
        assert!(!state.apply_kit_column_widths(&cols, &widths));
        assert_eq!(
            state.sync_selected_row(&["a".to_string(), "b".to_string()]),
            None
        );
        state.select_row("b");
        assert_eq!(
            state.sync_selected_row(&["a".to_string(), "b".to_string()]),
            Some(1)
        );
    }
}
