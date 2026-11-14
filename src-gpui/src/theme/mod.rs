pub mod palettes;
pub mod tokens;
pub mod typography;

pub use palettes::{
    AccentTokens, BorderTokens, K8sResourceColors, SemanticShade, SemanticTokens, StyleBundle,
    StyleTokens, SurfaceTokens, TextTokens,
};
pub use tokens::{
    AccentColor, K8sResourceKind, SemanticColorsPreference, SemanticPreset, SemanticRole,
    ThemeMode, UiStyle, rgb_hex, rgba_hex, rgba_u8,
};
pub use typography::{TypographyScaleStep, TypographyTokens, apply_text_step};

use gpui::{App, Global, Rgba, Styled, WindowAppearance};
use orbit_core::config::OrbitConfig;

#[derive(Debug, Clone)]
pub struct OrbitTheme {
    pub mode: ThemeMode,
    pub is_dark: bool,
    pub accent_id: AccentColor,
    pub semantic_pref: SemanticColorsPreference,
    pub surface: SurfaceTokens,
    pub border: BorderTokens,
    pub text: TextTokens,
    pub accent: AccentTokens,
    pub semantic: SemanticTokens,
    pub k8s: K8sResourceColors,
    pub style: StyleTokens,
    pub typography: TypographyTokens,
}

impl Global for OrbitTheme {}

impl OrbitTheme {
    pub fn resolve(
        mode: ThemeMode,
        ui_style: UiStyle,
        accent_id: AccentColor,
        semantic_pref: SemanticColorsPreference,
        window_appearance: WindowAppearance,
    ) -> Self {
        let is_dark = mode.resolve_is_dark(window_appearance);
        let accent = palettes::resolve_accent_tokens(accent_id, is_dark);
        let bundle = palettes::resolve_style_bundle(ui_style, is_dark, &accent);
        let semantic = palettes::resolve_semantic_tokens(&semantic_pref, is_dark);
        let k8s = palettes::resolve_k8s_colors(is_dark);

        Self {
            mode,
            is_dark,
            accent_id,
            semantic_pref,
            surface: bundle.surface,
            border: bundle.border,
            text: bundle.text,
            accent,
            semantic,
            k8s,
            style: bundle.style,
            typography: TypographyTokens::default(),
        }
    }

    pub fn from_config(config: &OrbitConfig, window_appearance: WindowAppearance) -> Self {
        let mode = ThemeMode::from_str_lossy(&config.theme_mode());
        let ui_style = UiStyle::from_str_lossy(&config.ui_style());
        let accent_id = AccentColor::from_str_lossy(&config.accent_color());
        let semantic_pref = SemanticColorsPreference::from_json(&config.semantic_colors());

        Self::resolve(mode, ui_style, accent_id, semantic_pref, window_appearance)
    }

    pub fn get(cx: &App) -> &Self {
        cx.global::<Self>()
    }

    pub fn with_ui_style(&self, ui_style: UiStyle, window_appearance: WindowAppearance) -> Self {
        Self::resolve(
            self.mode,
            ui_style,
            self.accent_id,
            self.semantic_pref.clone(),
            window_appearance,
        )
    }

    pub fn with_mode(&self, mode: ThemeMode, window_appearance: WindowAppearance) -> Self {
        Self::resolve(
            mode,
            self.style.ui_style,
            self.accent_id,
            self.semantic_pref.clone(),
            window_appearance,
        )
    }

    pub fn with_window_appearance(&self, window_appearance: WindowAppearance) -> Self {
        Self::resolve(
            self.mode,
            self.style.ui_style,
            self.accent_id,
            self.semantic_pref.clone(),
            window_appearance,
        )
    }

    pub fn k8s_color(&self, kind: K8sResourceKind) -> Rgba {
        self.k8s.get(kind)
    }

    pub fn style_card<E: Styled>(&self, el: E) -> E {
        let mut styled = el
            .bg(self.surface.bg_card)
            .border_1()
            .border_color(self.border.card)
            .rounded(self.style.radius_card);
        if !self.style.card_shadows.is_empty() {
            styled = styled.shadow(self.style.card_shadows.clone());
        }
        styled
    }

    pub fn style_overlay<E: Styled>(&self, el: E) -> E {
        let mut styled = el
            .bg(self.surface.overlay_bg)
            .border_1()
            .border_color(self.border.strong)
            .rounded(self.style.radius_card);
        if !self.style.overlay_shadows.is_empty() {
            styled = styled.shadow(self.style.overlay_shadows.clone());
        }
        styled
    }

    pub fn style_control<E: Styled>(&self, el: E) -> E {
        let mut styled = el
            .bg(self.surface.control_bg)
            .border_1()
            .border_color(self.border.default)
            .rounded(self.style.radius_control);
        if !self.style.control_shadows.is_empty() {
            styled = styled.shadow(self.style.control_shadows.clone());
        }
        styled
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gpui::{WindowBackgroundAppearance, px};

    #[test]
    fn test_orbit_theme_from_config_resolves_system_mode_and_style() {
        let mut config = OrbitConfig::default();
        config.values.insert("themeMode".to_string(), serde_json::json!("system"));
        config.values.insert("uiStyle".to_string(), serde_json::json!("liquidglass"));
        config.values.insert("accentColor".to_string(), serde_json::json!("emerald"));
        config.values.insert(
            "semanticColors".to_string(),
            serde_json::json!({ "preset": "colorblind" }),
        );

        let dark_theme = OrbitTheme::from_config(&config, WindowAppearance::Dark);
        assert!(dark_theme.is_dark);
        assert_eq!(dark_theme.mode, ThemeMode::System);
        assert_eq!(dark_theme.style.ui_style, UiStyle::LiquidGlass);
        assert_eq!(
            dark_theme.style.window_background,
            WindowBackgroundAppearance::Blurred
        );
        assert_eq!(dark_theme.style.radius_card, px(12.0));
        assert_eq!(dark_theme.style.radius_control, px(8.0));
        assert_eq!(dark_theme.accent_id, AccentColor::Emerald);
        assert_eq!(dark_theme.semantic.success.solid, rgb_hex(0x14b8a6));

        let light_theme = OrbitTheme::from_config(&config, WindowAppearance::Light);
        assert!(!light_theme.is_dark);
        assert_eq!(light_theme.style.ui_style, UiStyle::LiquidGlass);
        assert_eq!(light_theme.semantic.success.solid, rgb_hex(0x0f766e));

        let flat_theme = light_theme.with_ui_style(UiStyle::Flat, WindowAppearance::Light);
        assert_eq!(flat_theme.style.ui_style, UiStyle::Flat);
        assert_eq!(
            flat_theme.style.window_background,
            WindowBackgroundAppearance::Opaque
        );
    }
}
