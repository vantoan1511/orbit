use std::cell::Cell;
use std::rc::Rc;

use gpui::base::InteractiveElementExt as _;
use gpui::{
    AnyElement, App, Decorations, InteractiveElement, IntoElement, MouseButton, MouseDownEvent,
    MouseMoveEvent, Rgba, StatefulInteractiveElement, Styled, Window, WindowControlArea, div,
    prelude::*, px,
};
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

/// Returns `true` when the OS/platform window procedure handles window control
/// clicks (`Min`/`Max`/`Close`) and caption drag/double-click directly via
/// `WM_NCHITTEST` (`WindowControlArea`).
pub const fn uses_native_window_controls(platform: TitlebarPlatform) -> bool {
    matches!(platform, TitlebarPlatform::Windows)
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
    platform: TitlebarPlatform,
    on_click: impl Fn(&mut Window, &mut App) + 'static,
) -> AnyElement {
    let mut button = div()
        .id(spec.id)
        .group(spec.group_name)
        .w(px(WINDOW_CONTROL_WIDTH))
        .h_full()
        .flex()
        .items_center()
        .justify_center()
        .flex_shrink_0()
        .occlude()
        .window_control_area(spec.control_area)
        .hover(move |s| s.bg(spec.hover_bg))
        .tooltip(move |_window, cx| cx.new(|_| TextTooltip::new(spec.tooltip_label)).into())
        .child(
            render_icon(spec.icon, px(13.0), spec.fg)
                .group_hover(spec.group_name, move |s| s.text_color(spec.hover_fg)),
        );

    // On Windows, `gpui-pre-windows` handles `HTMINBUTTON` / `HTMAXBUTTON` / `HTCLOSE`
    // in `WM_NCLBUTTONUP` only when `DispatchEventResult.propagate` stays `true`.
    if !uses_native_window_controls(platform) {
        button = button
            .on_mouse_down(MouseButton::Left, |_event, window, cx| {
                window.prevent_default();
                cx.stop_propagation();
            })
            .on_click(move |_event, window, cx| {
                on_click(window, cx);
                cx.stop_propagation();
            });
    }

    #[cfg(test)]
    let button = {
        use gpui::test::TestSupportExt as _;
        button.test_support()
    };

    button.into_any_element()
}

pub fn render_custom_titlebar(
    slots: TitlebarSlots,
    is_window_active: bool,
    is_maximized: bool,
    decorations: Decorations,
    theme: &OrbitTheme,
    window: &mut Window,
    cx: &mut App,
) -> AnyElement {
    let platform = TitlebarPlatform::current();
    let layout = resolve_titlebar_layout_spec(platform, decorations);
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

    #[cfg(test)]
    let left_slot = {
        use gpui::test::TestSupportExt as _;
        left_slot.test_support()
    };

    let drag_region = div()
        .id("orbit-titlebar-drag-region")
        .flex_1()
        .min_w(px(32.0))
        .h_full();

    #[cfg(test)]
    let drag_region = {
        use gpui::test::TestSupportExt as _;
        drag_region.test_support()
    };

    let mut drag_surface = div()
        .id("orbit-titlebar-drag-surface")
        .flex_1()
        .min_w(px(0.0))
        .h_full()
        .flex()
        .items_center()
        .window_control_area(WindowControlArea::Drag)
        .child(left_slot)
        .child(drag_region);

    if let Some(right_controls) = slots.right_controls {
        let right_slot = div()
            .id("orbit-titlebar-right-slot")
            .h_full()
            .flex()
            .items_center()
            .gap_1p5()
            .pr_2()
            .flex_shrink_0()
            .child(right_controls);

        #[cfg(test)]
        let right_slot = {
            use gpui::test::TestSupportExt as _;
            right_slot.test_support()
        };

        drag_surface = drag_surface.child(right_slot);
    }

    match platform {
        TitlebarPlatform::Windows => {}
        TitlebarPlatform::MacOs => {
            drag_surface = drag_surface.on_mouse_down(
                MouseButton::Left,
                |event: &MouseDownEvent, window, _cx| {
                    if event.click_count == 2 && !window.default_prevented() {
                        window.titlebar_double_click();
                    }
                },
            );
        }
        TitlebarPlatform::Linux => {
            let should_move = window
                .use_keyed_state("orbit-titlebar-should-move", cx, |_window, _cx| {
                    Rc::new(Cell::new(false))
                })
                .read(cx)
                .clone();
            let should_move_down = Rc::clone(&should_move);
            let should_move_move = Rc::clone(&should_move);
            let should_move_up = should_move;

            drag_surface = drag_surface
                .on_double_click(|_event, window, _cx| {
                    if !window.default_prevented() {
                        window.zoom_window();
                    }
                })
                .on_mouse_down(MouseButton::Left, move |_event, window, _cx| {
                    should_move_down.set(!window.default_prevented());
                })
                .on_mouse_move(move |event: &MouseMoveEvent, window, _cx| {
                    if event.pressed_button == Some(MouseButton::Left) && should_move_move.get() {
                        should_move_move.set(false);
                        window.start_window_move();
                    }
                })
                .on_mouse_up(MouseButton::Left, move |_event, _window, _cx| {
                    should_move_up.set(false);
                });
        }
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

    #[cfg(test)]
    let drag_surface = {
        use gpui::test::TestSupportExt as _;
        drag_surface.test_support()
    };

    bar = bar.child(drag_surface);

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
            platform,
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
            platform,
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
            platform,
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

    #[test]
    fn test_uses_native_window_controls_by_platform() {
        assert!(uses_native_window_controls(TitlebarPlatform::Windows));
        assert!(!uses_native_window_controls(TitlebarPlatform::MacOs));
        assert!(!uses_native_window_controls(TitlebarPlatform::Linux));
    }

    #[gpui::test]
    fn test_windows_titlebar_controls_and_empty_space_propagate_for_native_hit_testing(
        cx: &mut gpui::TestAppContext,
    ) {
        use gpui::test::TestWindowExt as _;
        use std::cell::Cell;
        use std::rc::Rc;

        struct TitlebarHarness {
            theme: OrbitTheme,
            is_maximized: bool,
            left_clicks: Rc<Cell<usize>>,
            right_clicks: Rc<Cell<usize>>,
        }

        impl gpui::Render for TitlebarHarness {
            fn render(
                &mut self,
                window: &mut Window,
                cx: &mut gpui::Context<Self>,
            ) -> impl IntoElement {
                let left_clicks = Rc::clone(&self.left_clicks);
                let right_clicks = Rc::clone(&self.right_clicks);
                let left_pill = {
                    use gpui::test::TestSupportExt as _;
                    div()
                        .id("test-left-pill")
                        .occlude()
                        .w(px(64.0))
                        .h(px(TITLEBAR_CONTROL_HEIGHT))
                        .on_mouse_down(gpui::MouseButton::Left, move |_, window, cx| {
                            left_clicks.set(left_clicks.get() + 1);
                            window.prevent_default();
                            cx.stop_propagation();
                        })
                        .test_support()
                };
                let right_pill = {
                    use gpui::test::TestSupportExt as _;
                    div()
                        .id("test-right-pill")
                        .occlude()
                        .w(px(64.0))
                        .h(px(TITLEBAR_CONTROL_HEIGHT))
                        .on_mouse_down(gpui::MouseButton::Left, move |_, window, cx| {
                            right_clicks.set(right_clicks.get() + 1);
                            window.prevent_default();
                            cx.stop_propagation();
                        })
                        .test_support()
                };

                div().size_full().flex().flex_col().child(render_custom_titlebar(
                    TitlebarSlots {
                        left_controls: Some(left_pill.into_any_element()),
                        right_controls: Some(right_pill.into_any_element()),
                    },
                    true,
                    self.is_maximized,
                    Decorations::Client {
                        tiling: Tiling::default(),
                    },
                    &self.theme,
                    window,
                    cx,
                ))
            }
        }

        let theme = OrbitTheme::resolve(
            ThemeMode::Dark,
            UiStyle::Flat,
            AccentColor::Blue,
            SemanticColorsPreference::default(),
            WindowAppearance::Dark,
        );
        let left_clicks = Rc::new(Cell::new(0));
        let right_clicks = Rc::new(Cell::new(0));
        let left_clicks_for_view = Rc::clone(&left_clicks);
        let right_clicks_for_view = Rc::clone(&right_clicks);

        let opened = cx.update(|cx| {
            gpui::init(cx);
            crate::theme::kit_bridge::sync_kit_theme(&theme, cx);
            gpui::open_window(gpui::WindowOptions::default(), cx, move |_, cx| {
                cx.new(|_| TitlebarHarness {
                    theme: theme.clone(),
                    is_maximized: false,
                    left_clicks: left_clicks_for_view,
                    right_clicks: right_clicks_for_view,
                })
            })
        });
        let Ok((window, view)) = opened else {
            panic!("failed to open test window");
        };

        let dispatch_mouse = |window: &mut Window,
                              cx: &mut gpui::App,
                              pos: gpui::Point<gpui::Pixels>,
                              click_count: usize| {
            let down = window.dispatch_event(
                gpui::PlatformInput::MouseDown(gpui::MouseDownEvent {
                    position: pos,
                    button: gpui::MouseButton::Left,
                    modifiers: gpui::Modifiers::default(),
                    click_count,
                    first_mouse: false,
                }),
                cx,
            );
            let up = window.dispatch_event(
                gpui::PlatformInput::MouseUp(gpui::MouseUpEvent {
                    position: pos,
                    button: gpui::MouseButton::Left,
                    modifiers: gpui::Modifiers::default(),
                    click_count,
                }),
                cx,
            );
            (down, up)
        };

        cx.update(|cx| {
            let _ = window.update(cx, |_, window, cx| {
                window.render_frame(cx);
                assert!(window.try_find("orbit-titlebar-maximize").is_some());
                assert!(window.try_find("orbit-titlebar-drag-surface").is_some());

                // Interactive pills receive clicks and stop propagation.
                let left_pill_center = window.find("test-left-pill").bounds().center();
                let (down, _up) = dispatch_mouse(window, cx, left_pill_center, 1);
                assert!(!down.propagate);

                let right_pill_center = window.find("test-right-pill").bounds().center();
                let (down, _up) = dispatch_mouse(window, cx, right_pill_center, 1);
                assert!(!down.propagate);

                if TitlebarPlatform::current() == TitlebarPlatform::Windows {
                    // Maximize button must propagate so Windows native HTMAXBUTTON handles it.
                    let max_center = window.find("orbit-titlebar-maximize").bounds().center();
                    let (down, up) = dispatch_mouse(window, cx, max_center, 1);
                    assert!(down.propagate);
                    assert!(up.propagate);

                    // Empty space in left slot, drag region, and right slot must propagate.
                    let left_slot = window.find("orbit-titlebar-left-slot").bounds();
                    let empty_left = gpui::point(left_slot.left() + px(2.0), left_slot.top() + px(2.0));
                    let (down_1, _) = dispatch_mouse(window, cx, empty_left, 1);
                    let (down_2, _) = dispatch_mouse(window, cx, empty_left, 2);
                    assert!(down_1.propagate);
                    assert!(down_2.propagate);

                    let drag_center = window.find("orbit-titlebar-drag-region").bounds().center();
                    let (down_1, _) = dispatch_mouse(window, cx, drag_center, 1);
                    let (down_2, _) = dispatch_mouse(window, cx, drag_center, 2);
                    assert!(down_1.propagate);
                    assert!(down_2.propagate);

                    let right_slot = window.find("orbit-titlebar-right-slot").bounds();
                    let empty_right = gpui::point(right_slot.right() - px(2.0), right_slot.top() + px(2.0));
                    let (down_1, _) = dispatch_mouse(window, cx, empty_right, 1);
                    let (down_2, _) = dispatch_mouse(window, cx, empty_right, 2);
                    assert!(down_1.propagate);
                    assert!(down_2.propagate);
                }
            });
        });

        assert_eq!(left_clicks.get(), 1);
        assert_eq!(right_clicks.get(), 1);

        // Toggle is_maximized = true and verify Restore button still propagates on Windows.
        cx.update(|cx| {
            view.update(cx, |harness, cx| {
                harness.is_maximized = true;
                cx.notify();
            });
            let _ = window.update(cx, |_, window, cx| {
                window.render_frame(cx);
                if TitlebarPlatform::current() == TitlebarPlatform::Windows {
                    let max_center = window.find("orbit-titlebar-maximize").bounds().center();
                    let (down, up) = dispatch_mouse(window, cx, max_center, 1);
                    assert!(down.propagate);
                    assert!(up.propagate);
                }
            });
        });
    }
}
