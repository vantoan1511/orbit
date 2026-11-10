#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WindowCloseAction {
    HideToTray,
    ExitApplication,
}

pub fn determine_window_close_action(close_to_tray_enabled: bool) -> WindowCloseAction {
    if close_to_tray_enabled {
        WindowCloseAction::HideToTray
    } else {
        WindowCloseAction::ExitApplication
    }
}

pub fn is_hidden_launch<I, S>(args: I) -> bool
where
    I: IntoIterator<Item = S>,
    S: AsRef<str>,
{
    args.into_iter().any(|arg| {
        let s = arg.as_ref();
        s == "--hidden" || s == "-h" || s == "--minimized"
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tray_close_action_and_hidden_launch_flags() {
        assert_eq!(
            determine_window_close_action(true),
            WindowCloseAction::HideToTray
        );
        assert_eq!(
            determine_window_close_action(false),
            WindowCloseAction::ExitApplication
        );

        assert!(is_hidden_launch(["orbit", "--hidden"]));
        assert!(is_hidden_launch(vec!["orbit.exe".to_string(), "-h".to_string()]));
        assert!(is_hidden_launch(["orbit", "--minimized"]));
        assert!(!is_hidden_launch(["orbit"]));
        assert!(!is_hidden_launch(["orbit", "--debug"]));
    }
}
