//! System tray integration and CLI argument parsing for Orbit.

use tauri::{
    menu::{MenuBuilder, MenuItemBuilder},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    AppHandle, Manager,
};

pub const TRAY_MENU_OPEN: &str = "open";
pub const TRAY_MENU_QUIT: &str = "quit";

/// Action to perform when the user requests closing the main application window.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WindowCloseAction {
    HideToTray,
    ExitApplication,
}

/// Determines whether to hide the window to the tray or exit the application based on the close-to-tray setting.
pub fn determine_window_close_action(close_to_tray_enabled: bool) -> WindowCloseAction {
    if close_to_tray_enabled {
        WindowCloseAction::HideToTray
    } else {
        WindowCloseAction::ExitApplication
    }
}

/// Returns the system tray tooltip text based on development mode.
pub fn tray_tooltip_label() -> &'static str {
    if crate::config::OrbitConfig::is_dev_mode() {
        "Orbit [Dev]"
    } else {
        "Orbit"
    }
}

/// Returns the system tray "Open Orbit" menu item label based on development mode.
pub fn tray_open_item_label() -> &'static str {
    if crate::config::OrbitConfig::is_dev_mode() {
        "Open Orbit [Dev]"
    } else {
        "Open Orbit"
    }
}

/// Detects if Orbit was launched in the background (e.g. at OS startup) via `--hidden` or `--minimized`.
pub fn is_hidden_launch<I, S>(args: I) -> bool
where
    I: IntoIterator<Item = S>,
    S: AsRef<str>,
{
    args.into_iter().any(|arg| {
        let s = arg.as_ref();
        s == "--hidden" || s == "--minimized"
    })
}

/// Reveals and focuses the primary Orbit application window.
pub fn show_main_window(app: &AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.set_decorations(false);
        let _ = window.show();
        let _ = window.unminimize();
        let _ = window.set_focus();
    }
}

/// Sets up the Orbit system tray icon, context menu, and click event handlers.
pub fn setup_tray(app: &AppHandle) -> tauri::Result<()> {
    let open_item = MenuItemBuilder::with_id(TRAY_MENU_OPEN, tray_open_item_label()).build(app)?;
    let quit_item = MenuItemBuilder::with_id(TRAY_MENU_QUIT, "Quit Orbit").build(app)?;
    let menu = MenuBuilder::new(app)
        .item(&open_item)
        .separator()
        .item(&quit_item)
        .build()?;

    let mut builder = TrayIconBuilder::new()
        .tooltip(tray_tooltip_label())
        .menu(&menu)
        .show_menu_on_left_click(false);

    if let Some(icon) = app.default_window_icon() {
        builder = builder.icon(icon.clone());
    }

    builder
        .on_menu_event(|app, event| match event.id().as_ref() {
            TRAY_MENU_OPEN => {
                show_main_window(app);
            }
            TRAY_MENU_QUIT => {
                app.exit(0);
            }
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                show_main_window(tray.app_handle());
            }
        })
        .build(app)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tray_menu_constants() {
        assert_eq!(TRAY_MENU_OPEN, "open");
        assert_eq!(TRAY_MENU_QUIT, "quit");
    }

    #[test]
    fn test_is_hidden_launch_with_hidden_flag() {
        let args = vec!["orbit.exe", "--hidden"];
        assert!(is_hidden_launch(args));
    }

    #[test]
    fn test_is_hidden_launch_with_minimized_flag() {
        let args = vec!["orbit.exe", "--minimized"];
        assert!(is_hidden_launch(args));
    }

    #[test]
    fn test_is_hidden_launch_normal_launch() {
        let args = vec!["orbit.exe"];
        assert!(!is_hidden_launch(args));
    }

    #[test]
    fn test_is_hidden_launch_empty() {
        let args: Vec<&str> = vec![];
        assert!(!is_hidden_launch(args));
    }

    #[test]
    fn test_tray_title_and_tooltip_labels() {
        let original_orbit_dev = std::env::var("ORBIT_DEV").ok();

        // 1. Dev mode
        unsafe {
            std::env::set_var("ORBIT_DEV", "1");
        }
        assert_eq!(tray_tooltip_label(), "Orbit [Dev]");
        assert_eq!(tray_open_item_label(), "Open Orbit [Dev]");

        // 2. Prod mode
        unsafe {
            std::env::set_var("ORBIT_DEV", "0");
        }
        assert_eq!(tray_tooltip_label(), "Orbit");
        assert_eq!(tray_open_item_label(), "Open Orbit");

        // Restore env
        unsafe {
            match original_orbit_dev {
                Some(val) => std::env::set_var("ORBIT_DEV", val),
                None => std::env::remove_var("ORBIT_DEV"),
            }
        }
    }

    #[test]
    fn test_determine_window_close_action() {
        assert_eq!(determine_window_close_action(true), WindowCloseAction::HideToTray);
        assert_eq!(determine_window_close_action(false), WindowCloseAction::ExitApplication);
    }
}

