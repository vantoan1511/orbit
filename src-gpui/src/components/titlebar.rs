use gpui::{
    AnyElement, Decorations, InteractiveElement, IntoElement, Rgba, StatefulInteractiveElement,
    Styled, Window, WindowControlArea, div, prelude::*, px,
};
#[cfg(not(target_os = "windows"))]
use gpui::{MouseButton, MouseDownEvent};
use lucide_icons::Icon;

use crate::components::TextTooltip;
use crate::icons::render_icon;
use crate::theme::{OrbitTheme, rgb_hex};

pub const TITLEBAR_HEIGHT: f32 = 38.0;
pub const TITLEBAR_CONTROL_HEIGHT: f32 = 28.0;
pub const WINDOW_CONTROL_WIDTH: f32 = 44.0;
pub const MACOS_TRAFFIC_LIGHT_INSET: f32 = 76.0;
pub const MACOS_TRAFFIC_LIGHT_Y: f32 = 13.0;

#[derive(Default)]
pub struct TitlebarSlots {
    pub left_controls: Option<AnyElement>,
    pub right_controls: Option<AnyElement>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TitlebarPlatform {
    Windows,
    MacOs,
    Linux,
}

impl TitlebarPlatform {
    pub const fn current() -> Self {
        #[cfg(target_os = "macos")]
        {
            Self::MacOs
        }
        #[cfg(target_os = "windows")]
        {
            Self::Windows
        }
        #[cfg(not(any(target_os = "macos", target_os = "windows")))]
        {
            Self::Linux
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TitlebarLayoutSpec {
    pub left_inset_px: f32,
    pub show_window_controls: bool,
    pub height_px: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WindowChromeStyle {
    pub outer_border_color: Option<Rgba>,
    pub titlebar_bg: Rgba,
    pub titlebar_border: Rgba,
    pub title_fg: Rgba,
    pub icon_fg: Rgba,
    pub control_fg: Rgba,
    pub control_hover_bg: Rgba,
    pub close_hover_bg: Rgba,
    pub close_hover_fg: Rgba,
}

pub fn resolve_titlebar_layout_spec(
    platform: TitlebarPlatform,
    decorations: Decorations,
) -> TitlebarLayoutSpec {
    let (left_inset_px, show_window_controls) = match platform {
        TitlebarPlatform::MacOs => (MACOS_TRAFFIC_LIGHT_INSET, false),
        TitlebarPlatform::Windows => (0.0, true),
        TitlebarPlatform::Linux => (0.0, matches!(decorations, Decorations::Client { .. })),
    };

    TitlebarLayoutSpec {
        left_inset_px,
        show_window_controls,
        height_px: TITLEBAR_HEIGHT,
    }
}

pub fn resolve_window_chrome_style(
    theme: &OrbitTheme,
    is_window_active: bool,
    is_maximized: bool,
) -> WindowChromeStyle {
    let outer_border_color = if is_maximized {
        None
    } else if is_window_active {
        Some(theme.border.strong)
    } else {
        Some(theme.border.subtle)
    };

    let (title_fg, icon_fg, control_fg) = if is_window_active {
        (
            theme.text.primary,
            theme.text.secondary,
            theme.text.secondary,
        )
    } else {
        (theme.text.muted, theme.text.muted, theme.text.muted)
    };

    WindowChromeStyle {
        outer_border_color,
        titlebar_bg: theme.surface.bg_sidebar,
        titlebar_border: theme.border.default,
        title_fg,
        icon_fg,
        control_fg,
        control_hover_bg: theme.surface.bg_hover,
        close_hover_bg: theme.semantic.danger.solid,
        close_hover_fg: rgb_hex(0xffffff),
    }
}

pub const fn maximize_control_icon(is_maximized: bool) -> Icon {
    if is_maximized {
        Icon::Copy
    } else {
        Icon::Square
    }
}

pub const fn maximize_control_tooltip(is_maximized: bool) -> &'static str {
    if is_maximized {
        "Restore"
    } else {
        "Maximize"
    }
}

struct WindowControlSpec {
    id: &'static str,
    group_name: &'static str,
    icon: Icon,
    tooltip_label: &'static str,
    control_area: WindowControlArea,
    fg: Rgba,
    hover_bg: Rgba,
    hover_fg: Rgba,
}

fn render_window_control_button(
    spec: WindowControlSpec,
    on_click: impl Fn(&mut Window, &mut gpui::App) + 'static,
) -> AnyElement {
    div()
        .id(spec.id)
        .group(spec.group_name)
        .w(px(WINDOW_CONTROL_WIDTH))
        .h_full()
        .flex()
        .items_center()
        .justify_center()
        .flex_shrink_0()
        .window_control_area(spec.control_area)
        .hover(move |s| s.bg(spec.hover_bg))
        .tooltip(move |_window, cx| cx.new(|_| TextTooltip::new(spec.tooltip_label)).into())
        .on_click(move |_event, window, cx| {
            on_click(window, cx);
            cx.stop_propagation();
        })
        .child(
            render_icon(spec.icon, px(13.0), spec.fg)
                .group_hover(spec.group_name, move |s| s.text_color(spec.hover_fg)),
        )
        .into_any_element()
}

pub fn render_custom_titlebar(
    slots: TitlebarSlots,
    is_window_active: bool,
    is_maximized: bool,
    decorations: Decorations,
    theme: &OrbitTheme,
    _window: &Window,
) -> AnyElement {
    let layout = resolve_titlebar_layout_spec(TitlebarPlatform::current(), decorations);
    let chrome = resolve_window_chrome_style(theme, is_window_active, is_maximized);

    let mut left_slot = div()
        .id("orbit-titlebar-left-slot")
        .h_full()
        .flex()
        .items_center()
        .gap_2()
        .flex_shrink_0();
    left_slot = if layout.left_inset_px == 0.0 {
        left_slot.pl_3()
    } else {
        left_slot.pl_1()
    };
    left_slot = left_slot.child(render_icon(Icon::Orbit, px(15.0), chrome.icon_fg));
    if let Some(left_controls) = slots.left_controls {
        left_slot = left_slot.child(left_controls);
    }

    #[allow(unused_mut)]
    let mut drag_region = div()
        .id("orbit-titlebar-drag-region")
        .flex_1()
        .min_w(px(32.0))
        .h_full()
        .window_control_area(WindowControlArea::Drag);

    #[cfg(target_os = "macos")]
    {
        drag_region = drag_region.on_mouse_down(
            MouseButton::Left,
            |event: &MouseDownEvent, window, _cx| {
                if event.click_count == 2 {
                    window.titlebar_double_click();
                }
            },
        );
    }

    #[cfg(not(any(target_os = "windows", target_os = "macos")))]
    {
        drag_region = drag_region.on_mouse_down(
            MouseButton::Left,
            |event: &MouseDownEvent, window, _cx| {
                if event.click_count == 2 {
                    window.zoom_window();
                } else {
                    window.start_window_move();
                }
            },
        );
    }

    let mut bar = div()
        .id("orbit-custom-titlebar")
        .w_full()
        .h(px(layout.height_px))
        .flex_shrink_0()
        .flex()
        .items_center()
        .bg(chrome.titlebar_bg)
        .border_b_1()
        .border_color(chrome.titlebar_border);

    if layout.left_inset_px > 0.0 {
        bar = bar.child(div().w(px(layout.left_inset_px)).h_full().flex_shrink_0());
    }

    bar = bar.child(left_slot).child(drag_region);

    if let Some(right_controls) = slots.right_controls {
        bar = bar.child(
            div()
                .id("orbit-titlebar-right-slot")
                .h_full()
                .flex()
                .items_center()
                .gap_1p5()
                .pr_2()
                .flex_shrink_0()
                .child(right_controls),
        );
    }

    if layout.show_window_controls {
        let min_btn = render_window_control_button(
            WindowControlSpec {
                id: "orbit-titlebar-minimize",
                group_name: "orbit-titlebar-minimize-grp",
                icon: Icon::Minus,
                tooltip_label: "Minimize",
                control_area: WindowControlArea::Min,
                fg: chrome.control_fg,
                hover_bg: chrome.control_hover_bg,
                hover_fg: theme.text.primary,
            },
            |window, _cx| {
                window.minimize_window();
            },
        );

        let max_btn = render_window_control_button(
            WindowControlSpec {
                id: "orbit-titlebar-maximize",
                group_name: "orbit-titlebar-maximize-grp",
                icon: maximize_control_icon(is_maximized),
                tooltip_label: maximize_control_tooltip(is_maximized),
                control_area: WindowControlArea::Max,
                fg: chrome.control_fg,
                hover_bg: chrome.control_hover_bg,
                hover_fg: theme.text.primary,
            },
            |window, _cx| {
                window.zoom_window();
            },
        );

        let close_btn = render_window_control_button(
            WindowControlSpec {
                id: "orbit-titlebar-close",
                group_name: "orbit-titlebar-close-grp",
                icon: Icon::X,
                tooltip_label: "Close",
                control_area: WindowControlArea::Close,
                fg: chrome.control_fg,
                hover_bg: chrome.close_hover_bg,
                hover_fg: chrome.close_hover_fg,
            },
            |window, _cx| {
                window.remove_window();
            },
        );

        bar = bar.child(
            div()
                .h_full()
                .flex()
                .items_center()
                .flex_shrink_0()
                .child(min_btn)
                .child(max_btn)
                .child(close_btn),
        );
    }

    bar.into_any_element()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::theme::{AccentColor, SemanticColorsPreference, ThemeMode, UiStyle};
    use gpui::{Tiling, WindowAppearance};

    #[test]
    fn test_titlebar_layout_spec_respects_platform_and_decorations() {
        let client_decorations = Decorations::Client {
            tiling: Tiling::default(),
        };
        let server_decorations = Decorations::Server;

        let mac = resolve_titlebar_layout_spec(TitlebarPlatform::MacOs, client_decorations);
        assert_eq!(mac.left_inset_px, MACOS_TRAFFIC_LIGHT_INSET);
        assert!(!mac.show_window_controls);
        assert_eq!(mac.height_px, TITLEBAR_HEIGHT);

        let win = resolve_titlebar_layout_spec(TitlebarPlatform::Windows, client_decorations);
        assert_eq!(win.left_inset_px, 0.0);
        assert!(win.show_window_controls);
        assert_eq!(win.height_px, TITLEBAR_HEIGHT);

        let linux_client =
            resolve_titlebar_layout_spec(TitlebarPlatform::Linux, client_decorations);
        assert_eq!(linux_client.left_inset_px, 0.0);
        assert!(linux_client.show_window_controls);
        assert_eq!(linux_client.height_px, TITLEBAR_HEIGHT);

        let linux_server =
            resolve_titlebar_layout_spec(TitlebarPlatform::Linux, server_decorations);
        assert_eq!(linux_server.left_inset_px, 0.0);
        assert!(!linux_server.show_window_controls);
        assert_eq!(linux_server.height_px, TITLEBAR_HEIGHT);
        assert_eq!(linux_server.height_px, 38.0);
    }

    #[test]
    fn test_unified_titlebar_dimensions_and_macos_centering() {
        assert_eq!(TITLEBAR_HEIGHT, 38.0);
        assert_eq!(TITLEBAR_CONTROL_HEIGHT, 28.0);
        const {
            assert!(TITLEBAR_CONTROL_HEIGHT < TITLEBAR_HEIGHT);
        }
        assert_eq!(TITLEBAR_HEIGHT - TITLEBAR_CONTROL_HEIGHT, 10.0);
        assert_eq!(MACOS_TRAFFIC_LIGHT_INSET, 76.0);
        assert_eq!(MACOS_TRAFFIC_LIGHT_Y, 13.0);
        assert_eq!((TITLEBAR_HEIGHT - 12.0) / 2.0, MACOS_TRAFFIC_LIGHT_Y);

        let slots = TitlebarSlots::default();
        assert!(slots.left_controls.is_none());
        assert!(slots.right_controls.is_none());
    }

    #[test]
    fn test_window_chrome_style_handles_focus_maximized_and_all_themes() {
        for mode in [ThemeMode::Light, ThemeMode::Dark] {
            for style in UiStyle::ALL {
                let theme = OrbitTheme::resolve(
                    mode,
                    style,
                    AccentColor::Blue,
                    SemanticColorsPreference::default(),
                    WindowAppearance::Dark,
                );

                let active = resolve_window_chrome_style(&theme, true, false);
                assert_eq!(active.outer_border_color, Some(theme.border.strong));
                assert_eq!(active.title_fg, theme.text.primary);
                assert_eq!(active.icon_fg, theme.text.secondary);
                assert_eq!(active.control_fg, theme.text.secondary);
                assert_eq!(active.close_hover_fg, rgb_hex(0xffffff));

                let inactive = resolve_window_chrome_style(&theme, false, false);
                assert_eq!(inactive.outer_border_color, Some(theme.border.subtle));
                assert_eq!(inactive.title_fg, theme.text.muted);
                assert_eq!(inactive.icon_fg, theme.text.muted);
                assert_eq!(inactive.control_fg, theme.text.muted);
                assert_ne!(inactive.title_fg, active.title_fg);

                let maximized = resolve_window_chrome_style(&theme, true, true);
                assert_eq!(maximized.outer_border_color, None);
            }
        }
    }

    #[test]
    fn test_maximize_button_icon_and_label_reflect_window_state() {
        assert!(matches!(maximize_control_icon(false), Icon::Square));
        assert!(matches!(maximize_control_icon(true), Icon::Copy));
        assert_eq!(maximize_control_tooltip(false), "Maximize");
        assert_eq!(maximize_control_tooltip(true), "Restore");
    }
}
