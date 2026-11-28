use gpui::App;
use gpui::component::theme::{Theme as KitTheme, ThemeMode as KitMode};

use crate::theme::OrbitTheme;

/// Maps Orbit's resolved dark polarity to gpui-kit's stock light/dark theme mode.
pub const fn kit_mode_for_orbit(is_dark: bool) -> KitMode {
    if is_dark {
        KitMode::Dark
    } else {
        KitMode::Light
    }
}

/// Switches gpui-kit between its stock Default Light and Default Dark themes when polarity differs.
pub fn sync_kit_theme(orbit: &OrbitTheme, cx: &mut App) {
    let target_mode = kit_mode_for_orbit(orbit.is_dark);

    let needs_mode_switch = cx
        .try_global::<KitTheme>()
        .is_none_or(|kit| kit.mode != target_mode);

    if needs_mode_switch {
        KitTheme::change(target_mode, None, cx);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::theme::{AccentColor, SemanticColorsPreference, ThemeMode, UiStyle};
    use gpui::{Hsla, TestAppContext, WindowAppearance};

    #[test]
    fn test_kit_mode_for_orbit_maps_polarity() {
        assert_eq!(kit_mode_for_orbit(false), KitMode::Light);
        assert_eq!(kit_mode_for_orbit(true), KitMode::Dark);
    }

    #[gpui::test]
    fn test_sync_kit_theme_uses_stock_default_light_and_dark_without_orbit_overrides(
        cx: &mut TestAppContext,
    ) {
        let light = OrbitTheme::resolve(
            ThemeMode::Light,
            UiStyle::Flat,
            AccentColor::Blue,
            SemanticColorsPreference::default(),
            WindowAppearance::Light,
        );
        let dark = OrbitTheme::resolve(
            ThemeMode::Dark,
            UiStyle::Glass,
            AccentColor::Emerald,
            SemanticColorsPreference::default(),
            WindowAppearance::Dark,
        );

        cx.update(|cx| {
            gpui::init(cx);

            KitTheme::change(KitMode::Light, None, cx);
            let stock_light_bg = KitTheme::global(cx).background;
            let stock_light_primary = KitTheme::global(cx).primary;
            let stock_light_radius = KitTheme::global(cx).radius;
            let stock_light_name = KitTheme::global(cx).theme_name().clone();
            assert_eq!(stock_light_name.as_ref(), "Default Light");

            KitTheme::change(KitMode::Dark, None, cx);
            let stock_dark_bg = KitTheme::global(cx).background;
            let stock_dark_primary = KitTheme::global(cx).primary;
            let stock_dark_radius = KitTheme::global(cx).radius;
            let stock_dark_name = KitTheme::global(cx).theme_name().clone();
            assert_eq!(stock_dark_name.as_ref(), "Default Dark");

            sync_kit_theme(&light, cx);
            let kit = KitTheme::global(cx);
            assert_eq!(kit.mode, KitMode::Light);
            assert_eq!(kit.theme_name(), &stock_light_name);
            assert_eq!(kit.background, stock_light_bg);
            assert_eq!(kit.primary, stock_light_primary);
            assert_eq!(kit.radius, stock_light_radius);
            assert_ne!(kit.primary, Hsla::from(light.accent.accent));

            sync_kit_theme(&dark, cx);
            let kit = KitTheme::global(cx);
            assert_eq!(kit.mode, KitMode::Dark);
            assert_eq!(kit.theme_name(), &stock_dark_name);
            assert_eq!(kit.background, stock_dark_bg);
            assert_eq!(kit.primary, stock_dark_primary);
            assert_eq!(kit.radius, stock_dark_radius);
            assert_ne!(kit.background, Hsla::from(dark.surface.bg_app));
            assert_ne!(kit.primary, Hsla::from(dark.accent.accent));

            // Calling sync_kit_theme again with the same polarity is an idempotent no-op.
            sync_kit_theme(&dark, cx);
            let kit = KitTheme::global(cx);
            assert_eq!(kit.mode, KitMode::Dark);
            assert_eq!(kit.theme_name(), &stock_dark_name);
            assert_eq!(kit.background, stock_dark_bg);
            assert_eq!(kit.primary, stock_dark_primary);
            assert_eq!(kit.radius, stock_dark_radius);
        });
    }
}
