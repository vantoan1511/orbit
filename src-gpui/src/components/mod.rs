pub mod context_menu;
pub mod data_table;
pub mod resource_drawer;
pub mod resource_kind;
pub mod status_badge;
pub mod tab_bar;
pub mod titlebar;
pub mod tooltip;
pub mod yaml_viewer;

pub use context_menu::{
    CONTEXT_MENU_ITEM_HEIGHT, CONTEXT_MENU_PADDING_Y, CONTEXT_MENU_SEPARATOR_HEIGHT,
    CONTEXT_MENU_VIEWPORT_MARGIN, CONTEXT_MENU_WIDTH, ContextMenuCallback, ContextMenuEntry,
    ContextMenuItem, ContextMenuKeyAction, ContextMenuState, clamp_context_menu_position,
    estimate_context_menu_size, render_context_menu_overlay,
};

pub use data_table::{
    ColumnAlign, ColumnWidth, DataTableCallbacks, DataTableState, DataTableSurfaceStyle,
    RowClickCallback, RowContextMenuCallback, RowHoverActionSpec, SortCallback, SortDirection,
    SortState, TableBodyMode, TableColumn, apply_column_alignment, apply_column_width,
    render_data_table, render_empty_state, render_loading_skeleton, render_table_cell,
    render_table_header, resolve_data_table_surface_style, resolve_row_hover_actions,
    sort_rows_stable,
};
pub use resource_drawer::{
    DRAWER_MIN_WORKSPACE_MARGIN, DEFAULT_DRAWER_WIDTH, MAX_DRAWER_WIDTH, MIN_DRAWER_WIDTH,
    ResourceDrawerProps, ResourceDrawerState, ResourceDrawerTabDef, ResourceDrawerTabId,
    ResourceDrawerTarget, clamp_drawer_width, effective_drawer_width, render_resource_drawer,
};
pub use resource_kind::{
    ResourceKindMeta, parse_k8s_resource_kind, render_resource_kind_badge,
    render_resource_kind_icon, resource_kind_meta, resource_kind_meta_from_str,
};
pub use status_badge::{
    SemanticStatus, StatusBadgeStyle, StatusBadgeVariant, classify_status, render_status_badge,
    render_status_badge_from_str, status_badge_style,
};
pub use tab_bar::{
    DraggedWorkspaceTab, TAB_BAR_HEIGHT, TabBarCallbacks, TabContextMenuCallbacks,
    TabItemPresentation, TabPickerCallbacks, build_tab_context_menu_entries,
    render_tab_resource_picker_overlay, render_workspace_tab_bar, tab_icon_color,
    tab_item_presentation,
};
pub use titlebar::{
    MACOS_TRAFFIC_LIGHT_INSET, TITLEBAR_HEIGHT, TitlebarLayoutSpec, TitlebarPlatform,
    WINDOW_CONTROL_WIDTH, WindowChromeStyle, maximize_control_icon, maximize_control_tooltip,
    render_custom_titlebar, resolve_titlebar_layout_spec, resolve_window_chrome_style,
};
pub use tooltip::TextTooltip;
pub use yaml_viewer::{
    CachedYamlDocument, CachedYamlLine, YAML_LINE_HEIGHT, YAML_MIN_GUTTER_DIGITS,
    YamlDocumentCache, YamlHighlightSpan, YamlLoadStatus, YamlToken, YamlTokenKind,
    YamlViewerProps, render_yaml_viewer, tokenize_yaml_line, tokenize_yaml_line_spans,
    yaml_token_color,
};
