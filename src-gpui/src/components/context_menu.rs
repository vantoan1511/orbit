use std::rc::Rc;

use gpui::{
    AnyElement, App, FontWeight, InteractiveElement, IntoElement, MouseButton, MouseDownEvent,
    Pixels, Point, SharedString, Size, StatefulInteractiveElement, Styled, Window, div, point,
    prelude::*, px, size,
};
use lucide_icons::Icon;

use crate::icons::render_icon;
use crate::theme::{OrbitTheme, apply_text_step};

pub const CONTEXT_MENU_WIDTH: f32 = 220.0;
pub const CONTEXT_MENU_PADDING_Y: f32 = 4.0;
pub const CONTEXT_MENU_ITEM_HEIGHT: f32 = 28.0;
pub const CONTEXT_MENU_SEPARATOR_HEIGHT: f32 = 9.0;
pub const CONTEXT_MENU_VIEWPORT_MARGIN: f32 = 8.0;

pub type ContextMenuCallback = Rc<dyn Fn(&mut Window, &mut App)>;

#[derive(Clone)]
pub struct ContextMenuItem {
    pub id: SharedString,
    pub label: SharedString,
    pub icon: Option<Icon>,
    pub shortcut: Option<SharedString>,
    pub disabled: bool,
    pub danger: bool,
    pub on_select: Option<ContextMenuCallback>,
}

impl ContextMenuItem {
    pub fn new(id: impl Into<SharedString>, label: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            icon: None,
            shortcut: None,
            disabled: false,
            danger: false,
            on_select: None,
        }
    }

    pub fn icon(mut self, icon: Icon) -> Self {
        self.icon = Some(icon);
        self
    }

    pub fn shortcut(mut self, shortcut: impl Into<SharedString>) -> Self {
        self.shortcut = Some(shortcut.into());
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn danger(mut self, danger: bool) -> Self {
        self.danger = danger;
        self
    }

    pub fn on_select(mut self, callback: impl 'static + Fn(&mut Window, &mut App)) -> Self {
        self.on_select = Some(Rc::new(callback));
        self
    }
}

#[derive(Clone)]
pub enum ContextMenuEntry {
    Item(ContextMenuItem),
    Separator,
}

impl ContextMenuEntry {
    pub fn is_enabled_item(&self) -> bool {
        matches!(self, Self::Item(item) if !item.disabled)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContextMenuKeyAction {
    Ignored,
    Navigated,
    Closed,
    Activated,
}

#[derive(Clone, Default)]
pub struct ContextMenuState {
    pub is_open: bool,
    pub anchor: Point<Pixels>,
    pub entries: Vec<ContextMenuEntry>,
    pub highlighted_index: Option<usize>,
}

impl ContextMenuState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn open(&mut self, anchor: Point<Pixels>, entries: Vec<ContextMenuEntry>) {
        self.is_open = true;
        self.anchor = anchor;
        self.highlighted_index = entries.iter().position(ContextMenuEntry::is_enabled_item);
        self.entries = entries;
    }

    pub fn close(&mut self) {
        self.is_open = false;
        self.entries.clear();
        self.highlighted_index = None;
    }

    pub fn move_highlight_next(&mut self) {
        if !self.is_open || self.entries.is_empty() {
            return;
        }
        let len = self.entries.len();
        let start = match self.highlighted_index {
            Some(idx) => (idx + 1) % len,
            None => 0,
        };
        for offset in 0..len {
            let idx = (start + offset) % len;
            if self.entries[idx].is_enabled_item() {
                self.highlighted_index = Some(idx);
                return;
            }
        }
        self.highlighted_index = None;
    }

    pub fn move_highlight_prev(&mut self) {
        if !self.is_open || self.entries.is_empty() {
            return;
        }
        let len = self.entries.len();
        let start = match self.highlighted_index {
            Some(idx) => (idx + len - 1) % len,
            None => len - 1,
        };
        for offset in 0..len {
            let idx = (start + len - offset) % len;
            if self.entries[idx].is_enabled_item() {
                self.highlighted_index = Some(idx);
                return;
            }
        }
        self.highlighted_index = None;
    }

    pub fn set_highlight(&mut self, index: usize) {
        if !self.is_open {
            return;
        }
        if self
            .entries
            .get(index)
            .is_some_and(ContextMenuEntry::is_enabled_item)
        {
            self.highlighted_index = Some(index);
        }
    }

    pub fn activate_highlighted(&mut self) -> Option<ContextMenuCallback> {
        let index = self.highlighted_index?;
        self.activate_at(index)
    }

    pub fn activate_at(&mut self, index: usize) -> Option<ContextMenuCallback> {
        if !self.is_open {
            return None;
        }
        let callback = match self.entries.get(index) {
            Some(ContextMenuEntry::Item(item)) if !item.disabled => item.on_select.clone(),
            _ => return None,
        };
        self.close();
        callback
    }

    pub fn handle_key(&mut self, key: &str) -> (ContextMenuKeyAction, Option<ContextMenuCallback>) {
        if !self.is_open {
            return (ContextMenuKeyAction::Ignored, None);
        }
        match key {
            "escape" => {
                self.close();
                (ContextMenuKeyAction::Closed, None)
            }
            "down" | "arrowdown" => {
                self.move_highlight_next();
                (ContextMenuKeyAction::Navigated, None)
            }
            "up" | "arrowup" => {
                self.move_highlight_prev();
                (ContextMenuKeyAction::Navigated, None)
            }
            "enter" | "return" => {
                if let Some(cb) = self.activate_highlighted() {
                    (ContextMenuKeyAction::Activated, Some(cb))
                } else {
                    (ContextMenuKeyAction::Ignored, None)
                }
            }
            _ => (ContextMenuKeyAction::Ignored, None),
        }
    }
}

pub fn estimate_context_menu_size(entries: &[ContextMenuEntry], width: Pixels) -> Size<Pixels> {
    let content_height: f32 = entries
        .iter()
        .map(|entry| match entry {
            ContextMenuEntry::Item(_) => CONTEXT_MENU_ITEM_HEIGHT,
            ContextMenuEntry::Separator => CONTEXT_MENU_SEPARATOR_HEIGHT,
        })
        .sum();
    let total_height = (CONTEXT_MENU_PADDING_Y * 2.0) + content_height;
    size(width, px(total_height))
}

pub fn clamp_context_menu_position(
    anchor: Point<Pixels>,
    menu_size: Size<Pixels>,
    viewport: Size<Pixels>,
    margin: Pixels,
) -> Point<Pixels> {
    let margin_f: f32 = margin.into();
    let vw: f32 = viewport.width.into();
    let vh: f32 = viewport.height.into();
    let mw: f32 = menu_size.width.into();
    let mh: f32 = menu_size.height.into();
    let ax: f32 = anchor.x.into();
    let ay: f32 = anchor.y.into();

    let max_x = (vw - mw - margin_f).max(margin_f);
    let max_y = (vh - mh - margin_f).max(margin_f);

    let x = ax.max(margin_f).min(max_x);
    let y = ay.max(margin_f).min(max_y);

    point(px(x), px(y))
}

type ContextMenuIndexCallback = Rc<dyn Fn(usize, &mut Window, &mut App)>;

fn render_context_menu_separator(idx: usize, theme: &OrbitTheme) -> AnyElement {
    div()
        .id(("context-menu-sep", idx))
        .h(px(CONTEXT_MENU_SEPARATOR_HEIGHT))
        .flex()
        .items_center()
        .px_2()
        .child(div().w_full().h(px(1.0)).bg(theme.border.default))
        .into_any_element()
}

fn render_context_menu_item(
    idx: usize,
    item: &ContextMenuItem,
    is_highlighted: bool,
    theme: &OrbitTheme,
    on_hover: ContextMenuIndexCallback,
    on_activate: ContextMenuIndexCallback,
) -> AnyElement {
    let text_color = if item.disabled {
        theme.text.disabled
    } else if item.danger {
        theme.semantic.danger.solid
    } else {
        theme.text.primary
    };
    let icon_color = if item.disabled {
        theme.text.disabled
    } else if item.danger {
        theme.semantic.danger.solid
    } else {
        theme.text.muted
    };

    let mut row = div()
        .id(("context-menu-item", idx))
        .mx_1()
        .px_2()
        .h(px(CONTEXT_MENU_ITEM_HEIGHT))
        .rounded(theme.style.radius_control)
        .flex()
        .items_center()
        .justify_between()
        .gap_2();

    if is_highlighted {
        let highlight_bg = if item.danger {
            theme.semantic.danger.soft
        } else {
            theme.surface.bg_hover
        };
        row = row.bg(highlight_bg);
    }

    if !item.disabled {
        row = row
            .cursor_pointer()
            .on_hover(move |hovered, window, cx| {
                if *hovered {
                    on_hover(idx, window, cx);
                }
            })
            .on_mouse_down(MouseButton::Left, move |_event: &MouseDownEvent, window, cx| {
                cx.stop_propagation();
                on_activate(idx, window, cx);
            });
    }

    let left = div()
        .flex()
        .items_center()
        .gap_2()
        .min_w_0()
        .flex_1()
        .child(match item.icon {
            Some(icon) => render_icon(icon, px(13.0), icon_color),
            None => div().w(px(13.0)).h(px(13.0)),
        })
        .child(
            apply_text_step(div(), theme.typography.size_xs)
                .font_weight(FontWeight::MEDIUM)
                .text_color(text_color)
                .truncate()
                .child(item.label.clone()),
        );

    row = row.child(left);

    if let Some(shortcut) = &item.shortcut {
        row = row.child(
            apply_text_step(div(), theme.typography.size_2xs)
                .font(theme.typography.mono_font(FontWeight::NORMAL))
                .text_color(if item.disabled {
                    theme.text.disabled
                } else {
                    theme.text.muted
                })
                .child(shortcut.clone()),
        );
    }

    row.into_any_element()
}

pub fn render_context_menu_overlay(
    state: &ContextMenuState,
    viewport: Size<Pixels>,
    theme: &OrbitTheme,
    on_dismiss: impl Fn(&mut Window, &mut App) + 'static,
    on_hover: impl Fn(usize, &mut Window, &mut App) + 'static,
    on_activate: impl Fn(usize, &mut Window, &mut App) + 'static,
) -> Option<AnyElement> {
    if !state.is_open {
        return None;
    }

    let menu_width = px(CONTEXT_MENU_WIDTH);
    let menu_size = estimate_context_menu_size(&state.entries, menu_width);
    let pos = clamp_context_menu_position(
        state.anchor,
        menu_size,
        viewport,
        px(CONTEXT_MENU_VIEWPORT_MARGIN),
    );

    let on_dismiss = Rc::new(on_dismiss);
    let on_dismiss_left = Rc::clone(&on_dismiss);
    let on_dismiss_right = Rc::clone(&on_dismiss);
    let on_hover: ContextMenuIndexCallback = Rc::new(on_hover);
    let on_activate: ContextMenuIndexCallback = Rc::new(on_activate);

    let mut menu_box = theme
        .style_overlay(div().id("orbit-context-menu"))
        .absolute()
        .left(pos.x)
        .top(pos.y)
        .w(menu_width)
        .py(px(CONTEXT_MENU_PADDING_Y))
        .flex()
        .flex_col()
        .on_mouse_down(MouseButton::Left, |_event, _window, cx| {
            cx.stop_propagation();
        })
        .on_mouse_down(MouseButton::Right, |_event, _window, cx| {
            cx.stop_propagation();
        });

    for (idx, entry) in state.entries.iter().enumerate() {
        let child = match entry {
            ContextMenuEntry::Separator => render_context_menu_separator(idx, theme),
            ContextMenuEntry::Item(item) => {
                let is_highlighted = !item.disabled && state.highlighted_index == Some(idx);
                render_context_menu_item(
                    idx,
                    item,
                    is_highlighted,
                    theme,
                    Rc::clone(&on_hover),
                    Rc::clone(&on_activate),
                )
            }
        };
        menu_box = menu_box.child(child);
    }

    Some(
        div()
            .id("orbit-context-menu-overlay")
            .absolute()
            .inset_0()
            .child(
                div()
                    .id("orbit-context-menu-backdrop")
                    .absolute()
                    .inset_0()
                    .on_mouse_down(MouseButton::Left, move |_event, window, cx| {
                        cx.stop_propagation();
                        on_dismiss_left(window, cx);
                    })
                    .on_mouse_down(MouseButton::Right, move |_event, window, cx| {
                        cx.stop_propagation();
                        on_dismiss_right(window, cx);
                    }),
            )
            .child(menu_box)
            .into_any_element(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::Cell;

    #[test]
    fn test_context_menu_keyboard_navigation_skips_disabled_and_separators() {
        let activated = Rc::new(Cell::new(0u32));
        let activated_first = Rc::clone(&activated);
        let activated_last = Rc::clone(&activated);

        let mut state = ContextMenuState::new();
        state.open(
            point(px(100.0), px(100.0)),
            vec![
                ContextMenuEntry::Item(
                    ContextMenuItem::new("first", "First")
                        .on_select(move |_, _| activated_first.set(1)),
                ),
                ContextMenuEntry::Separator,
                ContextMenuEntry::Item(ContextMenuItem::new("disabled", "Disabled").disabled(true)),
                ContextMenuEntry::Item(
                    ContextMenuItem::new("last", "Last")
                        .on_select(move |_, _| activated_last.set(4)),
                ),
            ],
        );

        assert_eq!(state.highlighted_index, Some(0));

        let (action, cb) = state.handle_key("arrowdown");
        assert_eq!(action, ContextMenuKeyAction::Navigated);
        assert!(cb.is_none());
        assert_eq!(state.highlighted_index, Some(3));

        let (action, _) = state.handle_key("arrowdown");
        assert_eq!(action, ContextMenuKeyAction::Navigated);
        assert_eq!(state.highlighted_index, Some(0));

        let (action, _) = state.handle_key("arrowup");
        assert_eq!(action, ContextMenuKeyAction::Navigated);
        assert_eq!(state.highlighted_index, Some(3));

        let (action, cb) = state.handle_key("enter");
        assert_eq!(action, ContextMenuKeyAction::Activated);
        assert!(cb.is_some());
        assert!(!state.is_open);
        assert_eq!(state.highlighted_index, None);

        // Reopen and verify Escape closes without callback
        state.open(
            point(px(10.0), px(10.0)),
            vec![ContextMenuEntry::Item(ContextMenuItem::new("a", "A"))],
        );
        assert!(state.is_open);
        let (action, cb) = state.handle_key("escape");
        assert_eq!(action, ContextMenuKeyAction::Closed);
        assert!(cb.is_none());
        assert!(!state.is_open);
    }

    #[test]
    fn test_context_menu_viewport_clamping_prevents_offscreen_overflow() {
        let menu_size = size(px(220.0), px(150.0));
        let viewport = size(px(800.0), px(600.0));
        let margin = px(8.0);

        // Normal unclipped coordinate
        let normal = clamp_context_menu_position(
            point(px(120.0), px(200.0)),
            menu_size,
            viewport,
            margin,
        );
        assert_eq!(normal, point(px(120.0), px(200.0)));

        // Bottom-right overflow
        let overflow_br = clamp_context_menu_position(
            point(px(780.0), px(590.0)),
            menu_size,
            viewport,
            margin,
        );
        assert_eq!(overflow_br, point(px(572.0), px(442.0)));

        // Top-left negative/margin clamping
        let overflow_tl = clamp_context_menu_position(
            point(px(-20.0), px(2.0)),
            menu_size,
            viewport,
            margin,
        );
        assert_eq!(overflow_tl, point(px(8.0), px(8.0)));

        // Tight viewport smaller than menu size clamps to margin
        let tight_viewport = size(px(100.0), px(80.0));
        let tight = clamp_context_menu_position(
            point(px(50.0), px(50.0)),
            menu_size,
            tight_viewport,
            margin,
        );
        assert_eq!(tight, point(px(8.0), px(8.0)));
    }

    #[test]
    fn test_context_menu_estimate_size_accounts_for_items_and_separators() {
        let entries = vec![
            ContextMenuEntry::Item(ContextMenuItem::new("a", "A")),
            ContextMenuEntry::Separator,
            ContextMenuEntry::Item(ContextMenuItem::new("b", "B")),
        ];
        let measured = estimate_context_menu_size(&entries, px(220.0));
        let expected_h = (CONTEXT_MENU_PADDING_Y * 2.0)
            + (CONTEXT_MENU_ITEM_HEIGHT * 2.0)
            + CONTEXT_MENU_SEPARATOR_HEIGHT;
        assert_eq!(measured, size(px(220.0), px(expected_h)));
    }

    #[test]
    fn test_context_menu_disabled_item_cannot_be_highlighted_or_activated() {
        let mut state = ContextMenuState::new();
        state.open(
            point(px(10.0), px(10.0)),
            vec![
                ContextMenuEntry::Item(ContextMenuItem::new("disabled", "Disabled").disabled(true)),
                ContextMenuEntry::Separator,
                ContextMenuEntry::Item(
                    ContextMenuItem::new("enabled", "Enabled").on_select(|_, _| {}),
                ),
            ],
        );

        assert_eq!(state.highlighted_index, Some(2));
        state.set_highlight(0);
        assert_eq!(state.highlighted_index, Some(2));
        state.set_highlight(1);
        assert_eq!(state.highlighted_index, Some(2));

        assert!(state.activate_at(0).is_none());
        assert!(state.is_open);
        assert!(state.activate_at(1).is_none());
        assert!(state.is_open);

        assert!(state.activate_at(2).is_some());
        assert!(!state.is_open);
    }
}
