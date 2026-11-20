use gpui::Rgba;

use crate::theme::OrbitTheme;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TitlebarDwmConfig {
    pub use_dark_mode: bool,
    pub caption_colorref: u32,
    pub text_colorref: u32,
}

pub fn rgba_to_win32_colorref(color: Rgba) -> u32 {
    let r = (color.r.clamp(0.0, 1.0) * 255.0).round() as u32;
    let g = (color.g.clamp(0.0, 1.0) * 255.0).round() as u32;
    let b = (color.b.clamp(0.0, 1.0) * 255.0).round() as u32;
    (b << 16) | (g << 8) | r
}

pub fn resolve_titlebar_dwm_config(theme: &OrbitTheme) -> TitlebarDwmConfig {
    TitlebarDwmConfig {
        use_dark_mode: theme.is_dark,
        caption_colorref: rgba_to_win32_colorref(theme.surface.bg_panel),
        text_colorref: rgba_to_win32_colorref(theme.text.primary),
    }
}

#[cfg(target_os = "windows")]
pub fn sync_native_titlebar_appearance(window: &gpui::Window, theme: &OrbitTheme) {
    use core::ffi::c_void;
    use raw_window_handle::{HasWindowHandle, RawWindowHandle};

    const DWMWA_USE_IMMERSIVE_DARK_MODE: u32 = 20;
    const DWMWA_CAPTION_COLOR: u32 = 35;
    const DWMWA_TEXT_COLOR: u32 = 36;

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
    let caption = cfg.caption_colorref;
    let text = cfg.text_colorref;

    // SAFETY: `hwnd` is a live Win32 HWND obtained from GPUI's `Window`, and each pointer
    // references a stack-allocated Win32 attribute value with matching `size_of` byte length.
    unsafe {
        let _ = DwmSetWindowAttribute(
            hwnd,
            DWMWA_USE_IMMERSIVE_DARK_MODE,
            (&raw const use_dark).cast::<c_void>(),
            core::mem::size_of::<i32>() as u32,
        );
        let _ = DwmSetWindowAttribute(
            hwnd,
            DWMWA_CAPTION_COLOR,
            (&raw const caption).cast::<c_void>(),
            core::mem::size_of::<u32>() as u32,
        );
        let _ = DwmSetWindowAttribute(
            hwnd,
            DWMWA_TEXT_COLOR,
            (&raw const text).cast::<c_void>(),
            core::mem::size_of::<u32>() as u32,
        );
    }
}

#[cfg(not(target_os = "windows"))]
pub fn sync_native_titlebar_appearance(_window: &gpui::Window, _theme: &OrbitTheme) {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::theme::{AccentColor, SemanticColorsPreference, ThemeMode, UiStyle, rgb_hex};
    use gpui::WindowAppearance;

    #[test]
    fn test_rgba_to_win32_colorref_packs_bgr_channels() {
        assert_eq!(rgba_to_win32_colorref(rgb_hex(0x123456)), 0x0056_3412);
        assert_eq!(rgba_to_win32_colorref(rgb_hex(0xffffff)), 0x00ff_ffff);
        assert_eq!(rgba_to_win32_colorref(rgb_hex(0x000000)), 0x0000_0000);
    }

    #[test]
    fn test_resolve_titlebar_dwm_config_matches_theme_polarity_and_panel_surface() {
        let light_flat = OrbitTheme::resolve(
            ThemeMode::Light,
            UiStyle::Flat,
            AccentColor::Blue,
            SemanticColorsPreference::default(),
            WindowAppearance::Dark,
        );
        let light_cfg = resolve_titlebar_dwm_config(&light_flat);
        assert!(!light_cfg.use_dark_mode);
        assert_eq!(
            light_cfg.caption_colorref,
            rgba_to_win32_colorref(light_flat.surface.bg_panel)
        );
        assert_eq!(
            light_cfg.text_colorref,
            rgba_to_win32_colorref(light_flat.text.primary)
        );

        let dark_flat = OrbitTheme::resolve(
            ThemeMode::Dark,
            UiStyle::Flat,
            AccentColor::Blue,
            SemanticColorsPreference::default(),
            WindowAppearance::Light,
        );
        let dark_cfg = resolve_titlebar_dwm_config(&dark_flat);
        assert!(dark_cfg.use_dark_mode);
        assert_eq!(
            dark_cfg.caption_colorref,
            rgba_to_win32_colorref(dark_flat.surface.bg_panel)
        );
        assert_eq!(
            dark_cfg.text_colorref,
            rgba_to_win32_colorref(dark_flat.text.primary)
        );

        let light_paper = OrbitTheme::resolve(
            ThemeMode::Light,
            UiStyle::Paper,
            AccentColor::Blue,
            SemanticColorsPreference::default(),
            WindowAppearance::Dark,
        );
        let paper_cfg = resolve_titlebar_dwm_config(&light_paper);
        assert!(!paper_cfg.use_dark_mode);
        assert_eq!(
            paper_cfg.caption_colorref,
            rgba_to_win32_colorref(rgb_hex(0xf7f6f2))
        );
    }
}
