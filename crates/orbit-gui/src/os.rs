use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BackdropKind {
    Mica,
    Acrylic,
    None,
}

/// Threshold classifier mapping Windows NT build numbers to the appropriate backdrop kind.
/// Pure, deterministic, and testable contract as specified in PRD §3.
pub fn backdrop_for_build(build: u32) -> BackdropKind {
    if build >= 22000 {
        BackdropKind::Mica
    } else if build >= 17763 {
        BackdropKind::Acrylic
    } else {
        BackdropKind::None
    }
}

#[cfg(target_os = "windows")]
pub fn apply_backdrop(
    window: &impl raw_window_handle::HasWindowHandle,
    is_dark: bool,
) -> Result<BackdropKind, String> {
    match window_vibrancy::apply_mica(window, Some(is_dark)) {
        Ok(()) => {
            tracing::debug!("Applied Windows 11 Mica backdrop (dark: {is_dark})");
            Ok(BackdropKind::Mica)
        }
        Err(e) => {
            tracing::debug!("Mica backdrop unavailable or failed, attempting Acrylic fallback: {e}");
            let tint = if is_dark {
                (20, 20, 24, 180)
            } else {
                (245, 245, 248, 180)
            };
            match window_vibrancy::apply_acrylic(window, Some(tint)) {
                Ok(()) => {
                    tracing::debug!("Applied Windows 10 Acrylic backdrop");
                    Ok(BackdropKind::Acrylic)
                }
                Err(err) => {
                    let err_msg = format!("Failed to apply backdrop (Mica: {e}, Acrylic: {err})");
                    tracing::debug!("{err_msg}");
                    Err(err_msg)
                }
            }
        }
    }
}

#[cfg(not(target_os = "windows"))]
pub fn apply_backdrop(
    _window: &impl raw_window_handle::HasWindowHandle,
    _is_dark: bool,
) -> Result<BackdropKind, String> {
    Ok(BackdropKind::None)
}

pub fn open_browser_url(url: &str) -> Result<(), String> {
    open::that(url).map_err(|e| e.to_string())
}

pub fn pick_kubeconfig_file() -> Option<PathBuf> {
    rfd::FileDialog::new()
        .add_filter("Kubeconfig / YAML", &["yaml", "yml", "conf", "config", "*"])
        .pick_file()
}

pub fn save_log_file(default_filename: &str) -> Option<PathBuf> {
    rfd::FileDialog::new()
        .set_file_name(default_filename)
        .add_filter("Log Files", &["log", "txt"])
        .save_file()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_backdrop_for_build() {
        assert_eq!(backdrop_for_build(22621), BackdropKind::Mica);
        assert_eq!(backdrop_for_build(19045), BackdropKind::Acrylic);
        assert_eq!(backdrop_for_build(17763), BackdropKind::Acrylic);
        assert_eq!(backdrop_for_build(10240), BackdropKind::None);
    }
}
