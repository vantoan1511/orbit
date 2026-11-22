use std::rc::Rc;

use gpui::{
    AnyElement, App, Context, FocusHandle, FontWeight, InteractiveElement, IntoElement,
    MouseButton, MouseDownEvent, Pixels, Point, Render, Rgba, ScrollWheelEvent, SharedString,
    StatefulInteractiveElement, Styled, Window, div, prelude::*, px,
};
use lucide_icons::Icon;

use crate::components::context_menu::{ContextMenuEntry, ContextMenuItem};
use crate::icons::render_icon;
use crate::navigation::WorkspaceView;
use crate::state::{TabDropPosition, WorkspaceTabsState};
use crate::theme::{OrbitTheme, apply_text_step};

pub const TAB_BAR_HEIGHT: f32 = 32.0;

pub fn tab_icon_color(_view: WorkspaceView, is_active: bool, theme: &OrbitTheme) -> Rgba {
    let nav = theme.nav_selection();
    if is_active {
        nav.active_fg
    } else {
        nav.inactive_fg
    }
}

#[derive(Debug, Clone)]
pub struct TabItemPresentation {
    pub icon: Icon,
    pub icon_color: Rgba,
    pub text_color: Rgba,
    pub surface: Rgba,
    pub has_bottom_border: bool,
    pub has_top_accent_stripe: bool,
    pub close_always_visible: bool,
}

pub fn tab_item_presentation(
    view: WorkspaceView,
    is_active: bool,
    theme: &OrbitTheme,
) -> TabItemPresentation {
    if is_active {
        TabItemPresentation {
            icon: view.icon(),
            icon_color: tab_icon_color(view, true, theme),
            text_color: theme.text.primary,
            surface: theme.surface.bg_app,
            has_bottom_border: false,
            has_top_accent_stripe: false,
            close_always_visible: true,
        }
    } else {
        TabItemPresentation {
            icon: view.icon(),
            icon_color: theme.text.muted,
            text_color: theme.text.muted,
            surface: theme.surface.bg_sidebar,
            has_bottom_border: true,
            has_top_accent_stripe: false,
            close_always_visible: false,
        }
    }
}

#[derive(Clone)]
pub struct DraggedWorkspaceTab {
    pub view: WorkspaceView,
    pub index: usize,
    pub label: &'static str,
    pub icon: Icon,
    pub icon_color: Rgba,
    pub theme: OrbitTheme,
}

impl Render for DraggedWorkspaceTab {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .px_3()
            .h(px(TAB_BAR_HEIGHT))
            .flex()
            .items_center()
            .gap_1p5()
            .bg(self.theme.surface.bg_card)
            .border_1()
            .border_color(self.theme.border.strong)
            .rounded(self.theme.style.radius_control)
            .shadow_md()
            .child(render_icon(self.icon, px(13.0), self.icon_color))
            .child(
                apply_text_step(div(), self.theme.typography.size_xs)
                    .font_weight(FontWeight::MEDIUM)
                    .text_color(self.theme.text.primary)
                    .child(self.label),
            )
    }
}

pub type TabActionCallback = Rc<dyn Fn(&mut Window, &mut App)>;
pub type TabViewCallback = Rc<dyn Fn(WorkspaceView, &mut Window, &mut App)>;
pub type TabContextMenuCallback =
    Rc<dyn Fn(WorkspaceView, usize, Point<Pixels>, &mut Window, &mut App)>;
pub type TabDragStartCallback = Rc<dyn Fn(usize, &mut Window, &mut App)>;
pub type TabDragMoveCallback = Rc<dyn Fn(usize, TabDropPosition, &mut Window, &mut App)>;
pub type TabDropCallback = Rc<dyn Fn(usize, usize, TabDropPosition, &mut Window, &mut App)>;

#[derive(Clone)]
pub struct TabContextMenuCallbacks {
    pub on_move_left: TabViewCallback,
    pub on_move_right: TabViewCallback,
    pub on_close: TabViewCallback,
    pub on_close_others: TabViewCallback,
    pub on_close_right: TabViewCallback,
    pub on_close_all: TabActionCallback,
}

impl TabContextMenuCallbacks {
    pub fn noop() -> Self {
        Self {
            on_move_left: Rc::new(|_, _, _| {}),
            on_move_right: Rc::new(|_, _, _| {}),
            on_close: Rc::new(|_, _, _| {}),
            on_close_others: Rc::new(|_, _, _| {}),
            on_close_right: Rc::new(|_, _, _| {}),
            on_close_all: Rc::new(|_, _| {}),
        }
    }
}

pub fn build_tab_context_menu_entries(
    view: WorkspaceView,
    index: usize,
    total: usize,
    callbacks: &TabContextMenuCallbacks,
) -> Vec<ContextMenuEntry> {
    let close_shortcut = if cfg!(target_os = "macos") {
        "⌘W"
    } else {
        "Ctrl+W"
    };
    let is_first = index == 0;
    let is_last = index + 1 >= total;
    let is_only = total <= 1;

    let on_move_left = callbacks.on_move_left.clone();
    let on_move_right = callbacks.on_move_right.clone();
    let on_close = callbacks.on_close.clone();
    let on_close_others = callbacks.on_close_others.clone();
    let on_close_right = callbacks.on_close_right.clone();
    let on_close_all = callbacks.on_close_all.clone();

    vec![
        ContextMenuEntry::Item(
            ContextMenuItem::new("move-left", "Move Tab Left")
                .icon(Icon::ArrowLeft)
                .disabled(is_first)
                .on_select(move |window, cx| on_move_left(view, window, cx)),
        ),
        ContextMenuEntry::Item(
            ContextMenuItem::new("move-right", "Move Tab Right")
                .icon(Icon::ArrowRight)
                .disabled(is_last)
                .on_select(move |window, cx| on_move_right(view, window, cx)),
        ),
        ContextMenuEntry::Separator,
        ContextMenuEntry::Item(
            ContextMenuItem::new("close", "Close Tab")
                .icon(Icon::X)
                .shortcut(close_shortcut)
                .on_select(move |window, cx| on_close(view, window, cx)),
        ),
        ContextMenuEntry::Item(
            ContextMenuItem::new("close-others", "Close Others")
                .disabled(is_only)
                .on_select(move |window, cx| on_close_others(view, window, cx)),
        ),
        ContextMenuEntry::Item(
            ContextMenuItem::new("close-right", "Close Tabs to the Right")
                .disabled(is_last)
                .on_select(move |window, cx| on_close_right(view, window, cx)),
        ),
        ContextMenuEntry::Separator,
        ContextMenuEntry::Item(
            ContextMenuItem::new("close-all", "Close All")
                .danger(true)
                .on_select(move |window, cx| on_close_all(window, cx)),
        ),
    ]
}

#[derive(Clone)]
pub struct TabBarCallbacks {
    pub on_select: TabViewCallback,
    pub on_close: TabViewCallback,
    pub on_context_menu: TabContextMenuCallback,
    pub on_drag_start: TabDragStartCallback,
    pub on_drag_move: TabDragMoveCallback,
    pub on_drop: TabDropCallback,
    pub on_toggle_picker: TabActionCallback,
}

pub fn render_workspace_tab_bar(
    tabs: &mut WorkspaceTabsState,
    theme: &OrbitTheme,
    callbacks: TabBarCallbacks,
) -> AnyElement {
    let open_tabs = tabs.open_tabs().to_vec();
    let active_tab = tabs.active_tab();
    let drop_target = tabs.drop_target;
    let dragging_index = tabs.dragging_index;

    if tabs.pending_scroll_to_active {
        if let Some(active_idx) = active_tab.and_then(|a| open_tabs.iter().position(|v| *v == a)) {
            tabs.scroll_handle.scroll_to_item(active_idx);
        }
        tabs.pending_scroll_to_active = false;
    }

    let scroll_handle = tabs.scroll_handle.clone();
    let wheel_scroll_handle = tabs.scroll_handle.clone();

    let mut strip = div()
        .id("workspace-tab-strip")
        .flex_1()
        .min_w_0()
        .h(px(TAB_BAR_HEIGHT))
        .flex()
        .flex_row()
        .items_center()
        .overflow_x_scroll()
        .track_scroll(&scroll_handle)
        .on_scroll_wheel(move |event: &ScrollWheelEvent, _window, cx| {
            let delta = event.delta.pixel_delta(px(16.0));
            let mut offset = wheel_scroll_handle.offset();
            offset.x -= delta.x + delta.y;
            wheel_scroll_handle.set_offset(offset);
            cx.stop_propagation();
        });

    for (index, view) in open_tabs.into_iter().enumerate() {
        let is_active = active_tab == Some(view);
        strip = strip.child(render_tab_item(
            view,
            index,
            is_active,
            dragging_index == Some(index),
            drop_target.and_then(|(idx, pos)| if idx == index { Some(pos) } else { None }),
            theme,
            &callbacks,
        ));
    }

    strip = strip.child(
        div()
            .flex_1()
            .min_w(px(8.0))
            .h(px(TAB_BAR_HEIGHT))
            .border_b_1()
            .border_color(theme.border.default),
    );

    let on_toggle_picker = callbacks.on_toggle_picker.clone();
    let new_tab_btn = div()
        .id("workspace-new-tab-btn")
        .w(px(32.0))
        .h(px(TAB_BAR_HEIGHT))
        .flex_shrink_0()
        .flex()
        .items_center()
        .justify_center()
        .cursor_pointer()
        .border_l_1()
        .border_b_1()
        .border_color(theme.border.default)
        .hover(|s| s.bg(theme.surface.bg_hover))
        .on_mouse_down(MouseButton::Left, move |_event, window, cx| {
            cx.stop_propagation();
            on_toggle_picker(window, cx);
        })
        .child(render_icon(Icon::Plus, px(14.0), theme.text.secondary));

    div()
        .id("workspace-tab-bar")
        .w_full()
        .h(px(TAB_BAR_HEIGHT))
        .flex_shrink_0()
        .flex()
        .flex_row()
        .items_center()
        .bg(theme.surface.bg_sidebar)
        .child(strip)
        .child(new_tab_btn)
        .into_any_element()
}

fn render_tab_item(
    view: WorkspaceView,
    index: usize,
    is_active: bool,
    is_being_dragged: bool,
    drop_pos: Option<TabDropPosition>,
    theme: &OrbitTheme,
    callbacks: &TabBarCallbacks,
) -> AnyElement {
    let pres = tab_item_presentation(view, is_active, theme);
    let group_name: SharedString = format!("workspace-tab-{}", view.id()).into();
    let tab_id: SharedString = format!("workspace-tab-item-{}", view.id()).into();
    let close_id: SharedString = format!("workspace-tab-close-{}", view.id()).into();

    let on_select = callbacks.on_select.clone();
    let on_close_middle = callbacks.on_close.clone();
    let on_close_btn = callbacks.on_close.clone();
    let on_context_menu = callbacks.on_context_menu.clone();
    let on_drag_start = callbacks.on_drag_start.clone();
    let on_drag_move = callbacks.on_drag_move.clone();
    let on_drop = callbacks.on_drop.clone();
    let hover_bg = theme.surface.bg_hover;

    let mut tab_el = div()
        .id(tab_id)
        .group(group_name.clone())
        .relative()
        .h(px(TAB_BAR_HEIGHT))
        .px_3()
        .flex()
        .items_center()
        .gap_1p5()
        .flex_shrink_0()
        .cursor_pointer()
        .bg(pres.surface)
        .border_r_1()
        .border_color(theme.border.default);

    if pres.has_bottom_border {
        tab_el = tab_el
            .border_b_1()
            .border_color(theme.border.default)
            .hover(move |s| s.bg(hover_bg));
    }
    if is_being_dragged {
        tab_el = tab_el.opacity(0.45);
    }

    let drag_payload = DraggedWorkspaceTab {
        view,
        index,
        label: view.label(),
        icon: pres.icon,
        icon_color: pres.icon_color,
        theme: theme.clone(),
    };

    let mut close_btn = div()
        .id(close_id)
        .w(px(16.0))
        .h(px(16.0))
        .rounded(px(3.0))
        .flex()
        .items_center()
        .justify_center()
        .cursor_pointer()
        .hover(move |s| s.bg(hover_bg))
        .on_mouse_down(MouseButton::Left, move |_event, window, cx| {
            cx.stop_propagation();
            on_close_btn(view, window, cx);
        })
        .child(render_icon(Icon::X, px(11.0), theme.text.muted));

    close_btn = if pres.close_always_visible {
        close_btn.opacity(1.0)
    } else {
        close_btn
            .opacity(0.0)
            .group_hover(group_name, |s| s.opacity(1.0))
    };

    tab_el = tab_el
        .on_mouse_down(MouseButton::Left, move |_event, window, cx| {
            on_select(view, window, cx);
        })
        .on_mouse_down(MouseButton::Middle, move |_event, window, cx| {
            cx.stop_propagation();
            on_close_middle(view, window, cx);
        })
        .on_mouse_down(
            MouseButton::Right,
            move |event: &MouseDownEvent, window, cx| {
                cx.stop_propagation();
                on_context_menu(view, index, event.position, window, cx);
            },
        )
        .on_drag(drag_payload, move |dragged, _position, window, cx| {
            on_drag_start(dragged.index, window, cx);
            cx.new(|_| dragged.clone())
        })
        .on_drag_move::<DraggedWorkspaceTab>(move |event, window, cx| {
            let midpoint = event.bounds.left() + event.bounds.size.width * 0.5;
            let pos = if event.event.position.x < midpoint {
                TabDropPosition::Before
            } else {
                TabDropPosition::After
            };
            on_drag_move(index, pos, window, cx);
        })
        .on_drop::<DraggedWorkspaceTab>(move |dragged, window, cx| {
            let pos = drop_pos.unwrap_or(TabDropPosition::Before);
            on_drop(dragged.index, index, pos, window, cx);
        })
        .child(render_icon(pres.icon, px(13.0), pres.icon_color))
        .child(
            apply_text_step(div(), theme.typography.size_xs)
                .font_weight(if is_active {
                    FontWeight::MEDIUM
                } else {
                    FontWeight::NORMAL
                })
                .text_color(pres.text_color)
                .child(view.label()),
        )
        .child(close_btn);

    if let Some(pos) = drop_pos {
        let indicator = div()
            .absolute()
            .top_0()
            .bottom_0()
            .w(px(2.0))
            .bg(theme.accent.accent);
        tab_el = tab_el.child(match pos {
            TabDropPosition::Before => indicator.left_0(),
            TabDropPosition::After => indicator.right_0(),
        });
    }

    tab_el.into_any_element()
}

pub fn render_tab_resource_picker_overlay(
    tabs: &WorkspaceTabsState,
    theme: &OrbitTheme,
    search_focus: &FocusHandle,
    on_select_view: TabViewCallback,
    on_close: TabActionCallback,
) -> AnyElement {
    let views = tabs.filtered_picker_views();
    let search_text = if tabs.picker_query.is_empty() {
        apply_text_step(div(), theme.typography.size_xs)
            .text_color(theme.text.muted)
            .child("Search views (e.g. Pods, Deployments, Nodes)...")
    } else {
        apply_text_step(div(), theme.typography.size_xs)
            .text_color(theme.text.primary)
            .child(tabs.picker_query.clone())
    };

    let popover = theme
        .style_card(div())
        .id("tab-resource-picker-popover")
        .w(px(340.0))
        .bg(theme.surface.bg_card)
        .border_1()
        .border_color(theme.border.strong)
        .shadow_lg()
        .flex()
        .flex_col()
        .on_mouse_down(MouseButton::Left, |_event, _window, cx| {
            cx.stop_propagation();
        })
        .child(
            div()
                .track_focus(search_focus)
                .px_3()
                .py_2()
                .border_b_1()
                .border_color(theme.border.default)
                .flex()
                .items_center()
                .gap_2()
                .child(render_icon(Icon::Search, px(14.0), theme.text.muted))
                .child(div().flex_1().min_w_0().child(search_text)),
        )
        .child(render_picker_list(&views, theme, on_select_view));

    div()
        .id("tab-resource-picker-backdrop")
        .absolute()
        .inset_0()
        .on_mouse_down(MouseButton::Left, move |_event, window, cx| {
            on_close(window, cx);
        })
        .child(
            div()
                .absolute()
                .top(px(82.0))
                .right(px(16.0))
                .child(popover),
        )
        .into_any_element()
}

fn render_picker_list(
    views: &[WorkspaceView],
    theme: &OrbitTheme,
    on_select_view: TabViewCallback,
) -> AnyElement {
    let mut list = div()
        .id("tab-picker-list")
        .max_h(px(320.0))
        .overflow_y_scroll()
        .flex()
        .flex_col()
        .gap_0p5()
        .p_1p5();

    if views.is_empty() {
        return list
            .child(
                div()
                    .px_3()
                    .py_6()
                    .flex()
                    .items_center()
                    .justify_center()
                    .child(
                        apply_text_step(div(), theme.typography.size_xs)
                            .text_color(theme.text.muted)
                            .child("No matching workspace views"),
                    ),
            )
            .into_any_element();
    }

    for &view in views {
        let on_pick = on_select_view.clone();
        let icon_color = tab_icon_color(view, true, theme);
        let category_label = view.category().map(|c| c.label()).unwrap_or("Application");

        list = list.child(
            div()
                .id(SharedString::from(format!("tab-picker-item-{}", view.id())))
                .px_2p5()
                .py_1p5()
                .rounded(theme.style.radius_control)
                .flex()
                .items_center()
                .justify_between()
                .gap_2()
                .cursor_pointer()
                .hover(|s| s.bg(theme.surface.bg_hover))
                .on_mouse_down(MouseButton::Left, move |_event, window, cx| {
                    cx.stop_propagation();
                    on_pick(view, window, cx);
                })
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap_2()
                        .min_w_0()
                        .child(render_icon(view.icon(), px(14.0), icon_color))
                        .child(
                            apply_text_step(div(), theme.typography.size_xs)
                                .font_weight(FontWeight::MEDIUM)
                                .text_color(theme.text.primary)
                                .truncate()
                                .child(view.label()),
                        ),
                )
                .child(
                    apply_text_step(div(), theme.typography.size_xs)
                        .text_color(theme.text.muted)
                        .flex_shrink_0()
                        .child(category_label),
                ),
        );
    }

    list.into_any_element()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::theme::{AccentColor, SemanticColorsPreference, ThemeMode, UiStyle};

    fn find_item<'a>(entries: &'a [ContextMenuEntry], id: &str) -> &'a ContextMenuItem {
        entries
            .iter()
            .find_map(|e| match e {
                ContextMenuEntry::Item(it) if it.id.as_ref() == id => Some(it),
                _ => None,
            })
            .expect("expected context menu item")
    }

    #[test]
    fn test_tab_item_presentation_uses_seamless_bg_app_and_no_bottom_border_on_active() {
        for mode in [ThemeMode::Dark, ThemeMode::Light] {
            let theme = OrbitTheme::resolve(
                mode,
                UiStyle::Flat,
                AccentColor::default(),
                SemanticColorsPreference::default(),
                gpui::WindowAppearance::Dark,
            );
            let active = tab_item_presentation(WorkspaceView::Deployments, true, &theme);
            assert_eq!(active.surface, theme.surface.bg_app);
            assert!(!active.has_bottom_border);
            assert!(!active.has_top_accent_stripe);
            assert!(active.close_always_visible);
            assert_eq!(active.text_color, theme.text.primary);
            assert_eq!(active.icon_color, theme.text.primary);

            let inactive = tab_item_presentation(WorkspaceView::Deployments, false, &theme);
            assert_eq!(inactive.surface, theme.surface.bg_sidebar);
            assert!(inactive.has_bottom_border);
            assert!(!inactive.has_top_accent_stripe);
            assert!(!inactive.close_always_visible);
            assert_eq!(inactive.icon_color, theme.text.muted);

            for view in [
                WorkspaceView::Deployments,
                WorkspaceView::Pods,
                WorkspaceView::Overview,
                WorkspaceView::Logs,
            ] {
                assert_eq!(tab_icon_color(view, true, &theme), theme.text.primary);
                assert_eq!(tab_icon_color(view, false, &theme), theme.text.muted);
            }
        }
    }

    #[test]
    fn test_build_tab_context_menu_entries_enables_disables_by_position_and_count() {
        let callbacks = TabContextMenuCallbacks::noop();
        let first_of_three = build_tab_context_menu_entries(WorkspaceView::Overview, 0, 3, &callbacks);

        assert!(find_item(&first_of_three, "move-left").disabled);
        assert!(!find_item(&first_of_three, "move-right").disabled);
        assert!(!find_item(&first_of_three, "close-others").disabled);
        assert!(!find_item(&first_of_three, "close-right").disabled);
        assert!(first_of_three
            .iter()
            .any(|e| matches!(e, ContextMenuEntry::Separator)));

        let last_of_three = build_tab_context_menu_entries(WorkspaceView::Nodes, 2, 3, &callbacks);
        assert!(!find_item(&last_of_three, "move-left").disabled);
        assert!(find_item(&last_of_three, "move-right").disabled);
        assert!(!find_item(&last_of_three, "close-others").disabled);
        assert!(find_item(&last_of_three, "close-right").disabled);

        let single = build_tab_context_menu_entries(WorkspaceView::Overview, 0, 1, &callbacks);
        assert!(find_item(&single, "move-left").disabled);
        assert!(find_item(&single, "move-right").disabled);
        assert!(find_item(&single, "close-others").disabled);
        assert!(find_item(&single, "close-right").disabled);
    }
}
