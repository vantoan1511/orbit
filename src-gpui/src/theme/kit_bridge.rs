use gpui::App;
use gpui::component::theme::{Theme as KitTheme, ThemeMode as KitMode};

use crate::theme::OrbitTheme;

/// Pure projection of Orbit tokens onto a gpui-kit theme value.
pub fn apply_orbit_tokens(kit: &mut KitTheme, orbit: &OrbitTheme) {
    let colors = &mut kit.colors;
    colors.background = orbit.surface.bg_app.into();
    colors.foreground = orbit.text.primary.into();
    colors.secondary_foreground = orbit.text.primary.into();
    colors.accent_foreground = orbit.text.primary.into();
    colors.popover_foreground = orbit.text.primary.into();
    colors.caret = orbit.text.primary.into();
    colors.muted_foreground = orbit.text.muted.into();

    colors.border = orbit.border.default.into();
    colors.input = orbit.border.input.into();

    colors.muted = orbit.surface.control_bg.into();
    colors.secondary = orbit.surface.control_bg.into();
    colors.switch = orbit.surface.control_bg.into();
    colors.secondary_hover = orbit.surface.control_hover.into();
    colors.secondary_active = orbit.surface.bg_active.into();
    colors.list_active = orbit.surface.bg_active.into();
    colors.accent = orbit.surface.bg_hover.into();
    colors.list_hover = orbit.surface.bg_hover.into();

    colors.primary = orbit.accent.accent.into();
    colors.primary_hover = orbit.accent.hover.into();
    colors.primary_active = orbit.accent.active.into();
    colors.ring = orbit.accent.accent.into();
    colors.primary_foreground = orbit.text.on_accent.into();
    colors.selection = orbit.accent.selection.into();

    colors.switch_thumb = orbit.surface.bg_app.into();
    colors.popover = orbit.surface.overlay_bg.into();
    colors.overlay = orbit.surface.mask_bg.into();

    colors.success = orbit.semantic.success.solid.into();
    colors.warning = orbit.semantic.warning.solid.into();
    colors.danger = orbit.semantic.danger.solid.into();
    colors.info = orbit.semantic.info.solid.into();

    colors.title_bar = orbit.surface.bg_sidebar.into();
    colors.title_bar_border = orbit.border.default.into();
    colors.window_border = orbit.border.default.into();

    kit.radius = orbit.style.radius_control;
    kit.radius_lg = orbit.style.radius_card;
    kit.font_family = orbit.typography.ui_family.clone();
    kit.mono_font_family = orbit.typography.mono_family.clone();
    kit.font_size = orbit.typography.size_sm.font_size;
    kit.mono_font_size = orbit.typography.size_xs.font_size;
    kit.shadow = !orbit.style.control_shadows.is_empty();
}

/// Switches the Kit mode when it differs, then writes Orbit tokens through `KitTheme::update`.
pub fn sync_kit_theme(orbit: &OrbitTheme, cx: &mut App) {
    let target_mode = if orbit.is_dark {
        KitMode::Dark
    } else {
        KitMode::Light
    };

    let needs_mode_switch = cx
        .try_global::<KitTheme>()
        .is_none_or(|kit| kit.mode != target_mode);

    if needs_mode_switch {
        KitTheme::change(target_mode, None, cx);
    }

    KitTheme::update(cx, |kit| apply_orbit_tokens(kit, orbit));
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::theme::{AccentColor, SemanticColorsPreference, ThemeMode, UiStyle};
    use gpui::{Hsla, TestAppContext, WindowAppearance};

    #[test]
    fn test_apply_orbit_tokens_maps_dark_surface_and_accent() {
        let orbit = OrbitTheme::resolve(
            ThemeMode::Dark,
            UiStyle::Flat,
            AccentColor::default(),
            SemanticColorsPreference::default(),
            WindowAppearance::Dark,
        );
        let mut kit = KitTheme::default();
        apply_orbit_tokens(&mut kit, &orbit);

        assert_eq!(kit.background, Hsla::from(orbit.surface.bg_app));
        assert_eq!(kit.primary, Hsla::from(orbit.accent.accent));
        assert_eq!(kit.radius, orbit.style.radius_control);
        assert_eq!(kit.font_family, orbit.typography.ui_family);
    }

    #[test]
    fn test_apply_orbit_tokens_tracks_accent_change() {
        let blue = OrbitTheme::resolve(
            ThemeMode::Dark,
            UiStyle::Flat,
            AccentColor::Blue,
            SemanticColorsPreference::default(),
            WindowAppearance::Dark,
        );
        let emerald = OrbitTheme::resolve(
            ThemeMode::Dark,
            UiStyle::Flat,
            AccentColor::Emerald,
            SemanticColorsPreference::default(),
            WindowAppearance::Dark,
        );

        let mut kit_blue = KitTheme::default();
        let mut kit_emerald = KitTheme::default();
        apply_orbit_tokens(&mut kit_blue, &blue);
        apply_orbit_tokens(&mut kit_emerald, &emerald);

        assert_ne!(kit_blue.primary, kit_emerald.primary);
        assert_eq!(kit_emerald.primary, Hsla::from(emerald.accent.accent));
    }

    #[test]
    fn test_apply_orbit_tokens_maps_semantic_colors() {
        let orbit = OrbitTheme::resolve(
            ThemeMode::Dark,
            UiStyle::Flat,
            AccentColor::default(),
            SemanticColorsPreference::default(),
            WindowAppearance::Dark,
        );
        let mut kit = KitTheme::default();
        apply_orbit_tokens(&mut kit, &orbit);

        assert_eq!(kit.success, Hsla::from(orbit.semantic.success.solid));
        assert_eq!(kit.warning, Hsla::from(orbit.semantic.warning.solid));
        assert_eq!(kit.danger, Hsla::from(orbit.semantic.danger.solid));
        assert_eq!(kit.info, Hsla::from(orbit.semantic.info.solid));
    }

    #[gpui::test]
    fn test_sync_kit_theme_switches_mode_and_keeps_orbit_colors(cx: &mut TestAppContext) {
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
            sync_kit_theme(&light, cx);
            assert_eq!(KitTheme::global(cx).mode, KitMode::Light);
            assert_eq!(
                KitTheme::global(cx).background,
                Hsla::from(light.surface.bg_app)
            );

            sync_kit_theme(&dark, cx);
            let kit = KitTheme::global(cx);
            assert_eq!(kit.mode, KitMode::Dark);
            assert_eq!(kit.background, Hsla::from(dark.surface.bg_app));
            assert_eq!(kit.primary, Hsla::from(dark.accent.accent));
            assert_eq!(kit.radius, dark.style.radius_control);
        });
    }
}
