use crate::theme::OrbitTheme;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TitlebarDwmConfig {
    pub use_dark_mode: bool,
}

pub fn resolve_titlebar_dwm_config(theme: &OrbitTheme) -> TitlebarDwmConfig {
    TitlebarDwmConfig {
        use_dark_mode: theme.is_dark,
    }
}

#[cfg(target_os = "windows")]
pub fn sync_native_titlebar_appearance(window: &gpui::Window, theme: &OrbitTheme) {
    use core::ffi::c_void;
    use raw_window_handle::{HasWindowHandle, RawWindowHandle};

    const DWMWA_USE_IMMERSIVE_DARK_MODE: u32 = 20;

    #[link(name = "dwmapi")]
    unsafe extern "system" {
        fn DwmSetWindowAttribute(
            hwnd: isize,
            attr: u32,
            pv_attribute: *const c_void,
            cb_attribute: u32,
        ) -> i32;
    }

    let Ok(handle) = HasWindowHandle::window_handle(window) else {
        return;
    };
    let RawWindowHandle::Win32(win32) = handle.as_raw() else {
        return;
    };

    let hwnd = win32.hwnd.get();
    let cfg = resolve_titlebar_dwm_config(theme);
    let use_dark: i32 = i32::from(cfg.use_dark_mode);

    // SAFETY: `hwnd` is a live Win32 HWND obtained from GPUI's `Window`, and `use_dark`
    // references a stack-allocated `i32` with matching `size_of` byte length.
    unsafe {
        let _ = DwmSetWindowAttribute(
            hwnd,
            DWMWA_USE_IMMERSIVE_DARK_MODE,
            (&raw const use_dark).cast::<c_void>(),
            core::mem::size_of::<i32>() as u32,
        );
    }
}

#[cfg(not(target_os = "windows"))]
pub fn sync_native_titlebar_appearance(_window: &gpui::Window, _theme: &OrbitTheme) {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::theme::{AccentColor, SemanticColorsPreference, ThemeMode, UiStyle};
    use gpui::WindowAppearance;

    #[test]
    fn test_resolve_titlebar_dwm_config_tracks_polarity_for_all_styles() {
        for mode in [ThemeMode::Light, ThemeMode::Dark, ThemeMode::System] {
            for appearance in [WindowAppearance::Light, WindowAppearance::Dark] {
                for style in UiStyle::ALL {
                    let theme = OrbitTheme::resolve(
                        mode,
                        style,
                        AccentColor::Blue,
                        SemanticColorsPreference::default(),
                        appearance,
                    );
                    let cfg = resolve_titlebar_dwm_config(&theme);
                    assert_eq!(cfg, TitlebarDwmConfig { use_dark_mode: theme.is_dark });
                }
            }
        }
    }
}
