use gpui::{
    App, Application, Bounds, Context, SharedString, Window, WindowBounds, WindowDecorations,
    WindowOptions, div, prelude::*, px, rgb, size,
};

mod theme {
    pub const BG_APP: u32 = 0x0b0f17;
    pub const BG_PANEL: u32 = 0x0f172a;
    pub const BG_CONTROL: u32 = 0x1e293b;
    pub const BG_CONTROL_HOVER: u32 = 0x334155;
    pub const BORDER: u32 = 0x1e293b;
    pub const BORDER_CONTROL: u32 = 0x334155;
    pub const TEXT_PRIMARY: u32 = 0xe2e8f0;
    pub const TEXT_SECONDARY: u32 = 0x94a3b8;
    pub const TEXT_MUTED: u32 = 0x64748b;
}

pub struct OrbitShell;

impl OrbitShell {
    pub const WINDOW_TITLE: &'static str = "Orbit (GPUI Preview)";
    pub const DEFAULT_WIDTH: f32 = 1200.0;
    pub const DEFAULT_HEIGHT: f32 = 800.0;

    pub fn default_window_bounds(cx: &App) -> Bounds<gpui::Pixels> {
        Bounds::centered(
            None,
            size(px(Self::DEFAULT_WIDTH), px(Self::DEFAULT_HEIGHT)),
            cx,
        )
    }

    pub fn default_window_options(bounds: Bounds<gpui::Pixels>) -> WindowOptions {
        WindowOptions {
            window_bounds: Some(WindowBounds::Windowed(bounds)),
            titlebar: Some(gpui::TitlebarOptions {
                title: Some(SharedString::from(Self::WINDOW_TITLE)),
                appears_transparent: false,
                traffic_light_position: None,
            }),
            window_decorations: Some(WindowDecorations::Server),
            is_movable: true,
            is_resizable: true,
            ..Default::default()
        }
    }
}

impl Render for OrbitShell {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .flex()
            .flex_col()
            .size_full()
            .bg(rgb(theme::BG_APP))
            .text_color(rgb(theme::TEXT_PRIMARY))
            .p_8()
            .gap_6()
            .child(
                div()
                    .flex()
                    .items_center()
                    .justify_between()
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .gap_1()
                            .child(
                                div()
                                    .text_xl()
                                    .font_weight(gpui::FontWeight::BOLD)
                                    .child("Orbit (GPUI Preview)"),
                            )
                            .child(
                                div()
                                    .text_sm()
                                    .text_color(rgb(theme::TEXT_SECONDARY))
                                    .child("Native GPU-accelerated desktop shell"),
                            ),
                    )
                    .child(
                        div()
                            .id("switch-to-old-ui")
                            .px_3()
                            .py_1p5()
                            .rounded_md()
                            .bg(rgb(theme::BG_CONTROL))
                            .hover(|style| style.bg(rgb(theme::BG_CONTROL_HOVER)))
                            .border_1()
                            .border_color(rgb(theme::BORDER_CONTROL))
                            .cursor_pointer()
                            .text_sm()
                            .font_weight(gpui::FontWeight::MEDIUM)
                            .child("Switch to Old UI")
                            .on_click(cx.listener(|_this, _event, _window, cx| {
                                cx.quit();
                            })),
                    ),
            )
            .child(
                div()
                    .flex_1()
                    .flex()
                    .items_center()
                    .justify_center()
                    .rounded_lg()
                    .border_1()
                    .border_color(rgb(theme::BORDER))
                    .bg(rgb(theme::BG_PANEL))
                    .child(
                        div()
                            .text_sm()
                            .text_color(rgb(theme::TEXT_MUTED))
                            .child("GPUI Shell Window ready. Close this window or click \"Switch to Old UI\" to return."),
                    ),
            )
    }
}

fn main() {
    Application::new().run(|cx: &mut App| {
        cx.on_window_closed(|cx| {
            if cx.windows().is_empty() {
                cx.quit();
            }
        })
        .detach();

        let bounds = OrbitShell::default_window_bounds(cx);
        let options = OrbitShell::default_window_options(bounds);
        if let Err(err) = cx.open_window(options, |_, cx| cx.new(|_| OrbitShell)) {
            eprintln!("Failed to open Orbit GPUI window: {err}");
            cx.quit();
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_window_options_and_shell_defaults() {
        assert_eq!(OrbitShell::WINDOW_TITLE, "Orbit (GPUI Preview)");
        const {
            assert!(OrbitShell::DEFAULT_WIDTH >= 800.0);
            assert!(OrbitShell::DEFAULT_HEIGHT >= 600.0);
        }

        let bounds = Bounds::new(
            gpui::point(px(0.0), px(0.0)),
            size(px(OrbitShell::DEFAULT_WIDTH), px(OrbitShell::DEFAULT_HEIGHT)),
        );
        let opts = OrbitShell::default_window_options(bounds);
        assert_eq!(opts.window_decorations, Some(WindowDecorations::Server));
        assert!(opts.is_resizable);
        assert!(opts.is_movable);
        match opts.window_bounds {
            Some(WindowBounds::Windowed(b)) => {
                assert!(f32::from(b.size.width) >= 800.0);
                assert!(f32::from(b.size.height) >= 600.0);
            }
            _ => panic!("Expected WindowBounds::Windowed"),
        }
        let title = opts
            .titlebar
            .as_ref()
            .and_then(|t| t.title.as_ref())
            .map(|s| s.as_ref());
        assert_eq!(title, Some("Orbit (GPUI Preview)"));
    }
}
