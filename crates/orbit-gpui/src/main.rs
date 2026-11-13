mod app;
mod bridge;
mod state;
mod theme;

use crate::app::OrbitGpuiApp;
use gpui::{
    App, AppContext, Application, Bounds, TitlebarOptions, WindowBounds, WindowOptions, px, size,
};

fn main() {
    Application::new().run(|cx: &mut App| {
        let bounds = Bounds::centered(None, size(px(1100.0), px(720.0)), cx);
        let options = WindowOptions {
            window_bounds: Some(WindowBounds::Windowed(bounds)),
            titlebar: Some(TitlebarOptions {
                title: Some("Orbit".into()),
                appears_transparent: true,
                traffic_light_position: None,
            }),
            ..Default::default()
        };

        if let Err(err) = cx.open_window(options, |_, cx| cx.new(OrbitGpuiApp::new)) {
            eprintln!("Failed to open Orbit GPUI window: {err}");
            cx.quit();
        }
    });
}
