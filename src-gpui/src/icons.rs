use std::borrow::Cow;

use gpui::{
    App, Div, Font, FontFeatures, FontStyle, FontWeight, Pixels, Rgba, SharedString, div,
    prelude::*,
};
use lucide_icons::{Icon, LUCIDE_FONT_BYTES};

use crate::theme::ThemeMode;

pub const LUCIDE_FONT_FAMILY: &str = "lucide";

pub fn lucide_font() -> Font {
    Font {
        family: SharedString::from(LUCIDE_FONT_FAMILY),
        features: FontFeatures::default(),
        fallbacks: None,
        weight: FontWeight::NORMAL,
        style: FontStyle::Normal,
    }
}

pub fn icon_char(icon: Icon) -> char {
    char::from(icon)
}

pub const fn theme_mode_icon(mode: ThemeMode) -> Icon {
    match mode {
        ThemeMode::Dark => Icon::Moon,
        ThemeMode::Light => Icon::Sun,
        ThemeMode::System => Icon::Laptop,
    }
}

pub fn theme_mode_tooltip(mode: ThemeMode, is_dark: bool) -> String {
    match mode {
        ThemeMode::Dark => "Theme: Dark".to_string(),
        ThemeMode::Light => "Theme: Light".to_string(),
        ThemeMode::System => {
            let resolved = if is_dark { "Dark" } else { "Light" };
            format!("Theme: System ({resolved})")
        }
    }
}

pub fn register_lucide_font(cx: &App) -> gpui::Result<()> {
    cx.text_system()
        .add_fonts(vec![Cow::Borrowed(LUCIDE_FONT_BYTES)])
}

pub fn render_icon(icon: Icon, size: Pixels, color: Rgba) -> Div {
    div()
        .w(size)
        .h(size)
        .flex()
        .items_center()
        .justify_center()
        .flex_shrink_0()
        .font(lucide_font())
        .text_size(size)
        .line_height(size)
        .text_color(color)
        .child(icon_char(icon).to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lucide_font_bytes_and_glyph_conversion() {
        assert!(!LUCIDE_FONT_BYTES.is_empty());
        assert_eq!(lucide_font().family.as_ref(), LUCIDE_FONT_FAMILY);

        let server_char = icon_char(Icon::Server);
        let settings_char = icon_char(Icon::Settings);
        let dashboard_char = icon_char(Icon::LayoutDashboard);
        assert_ne!(server_char, '\0');
        assert_ne!(server_char, settings_char);
        assert_ne!(server_char, dashboard_char);
        assert_ne!(settings_char, dashboard_char);

        assert_eq!(
            icon_char(theme_mode_icon(ThemeMode::Dark)),
            icon_char(Icon::Moon)
        );
        assert_eq!(
            icon_char(theme_mode_icon(ThemeMode::Light)),
            icon_char(Icon::Sun)
        );
        assert_eq!(
            icon_char(theme_mode_icon(ThemeMode::System)),
            icon_char(Icon::Laptop)
        );
    }

    #[test]
    fn test_theme_mode_tooltip_formats_all_modes() {
        assert_eq!(theme_mode_tooltip(ThemeMode::Dark, true), "Theme: Dark");
        assert_eq!(theme_mode_tooltip(ThemeMode::Light, false), "Theme: Light");
        assert_eq!(
            theme_mode_tooltip(ThemeMode::System, true),
            "Theme: System (Dark)"
        );
        assert_eq!(
            theme_mode_tooltip(ThemeMode::System, false),
            "Theme: System (Light)"
        );
    }
}
