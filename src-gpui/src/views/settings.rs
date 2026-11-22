use std::rc::Rc;

use gpui::{AnyElement, App, FontWeight, Window, WindowAppearance, div, prelude::*, px};
use lucide_icons::Icon;

use crate::icons::render_icon;
use crate::theme::{
    AccentColor, OrbitTheme, ThemeMode, UiStyle, apply_text_step, palettes::resolve_accent_tokens,
};
use crate::views::{render_action_button, render_detail_row};

pub struct SettingsViewModel<'a> {
    pub theme: &'a OrbitTheme,
    pub resolved_system_is_dark: bool,
    pub enable_experience_features: bool,
    pub config_file_path: Option<String>,
    pub sidebar_status: String,
    pub save_error: Option<&'a str>,
}

pub type SettingsActionCallback = Rc<dyn Fn(&mut Window, &mut App)>;
pub type SettingsValueCallback<T> = Rc<dyn Fn(T, &mut Window, &mut App)>;

pub struct SettingsCallbacks {
    pub on_select_theme_mode: SettingsValueCallback<ThemeMode>,
    pub on_select_ui_style: SettingsValueCallback<UiStyle>,
    pub on_select_accent_color: SettingsValueCallback<AccentColor>,
    pub on_toggle_experience_features: SettingsValueCallback<bool>,
    pub on_reset_appearance: SettingsActionCallback,
}

pub fn is_appearance_modified(theme: &OrbitTheme) -> bool {
    theme.mode != ThemeMode::Dark
        || theme.style.ui_style != UiStyle::Flat
        || theme.accent_id != AccentColor::Blue
}

pub fn system_mode_hint(window_appearance: WindowAppearance) -> &'static str {
    if ThemeMode::System.resolve_is_dark(window_appearance) {
        "Dark"
    } else {
        "Light"
    }
}

pub fn experience_features_status_label(enabled: bool) -> &'static str {
    if enabled { "Enabled" } else { "Disabled" }
}

pub fn render_settings_view(
    model: &SettingsViewModel<'_>,
    callbacks: &SettingsCallbacks,
) -> AnyElement {
    let theme = model.theme;
    let modified = is_appearance_modified(theme);

    div()
        .id("settings-scroll-container")
        .flex_1()
        .w_full()
        .overflow_y_scroll()
        .child(
            div()
                .flex()
                .flex_col()
                .gap_8()
                .w_full()
                .max_w(px(960.0))
                .pb_8()
                .child(render_settings_header(theme, modified, &callbacks.on_reset_appearance))
                .when_some(model.save_error, |el, error_msg| {
                    el.child(render_save_error_banner(theme, error_msg))
                })
                .child(render_two_column_section(
                    theme,
                    "THEME MODE",
                    "Color Appearance",
                    "Switch between dark noir, daylight, or follow your OS preference.",
                    render_theme_mode_cards(
                        theme,
                        model.resolved_system_is_dark,
                        &callbacks.on_select_theme_mode,
                    ),
                ))
                .child(render_two_column_section(
                    theme,
                    "UI STYLE & AESTHETIC",
                    "Surface Material",
                    "Select how panels, cards, and overlays render across the workspace.",
                    render_ui_style_cards(theme, &callbacks.on_select_ui_style),
                ))
                .child(render_two_column_section(
                    theme,
                    "ACCENT COLOR",
                    "Interactive Focus",
                    "Primary accent applied to active controls, selections, and focus rings.",
                    render_accent_section(theme, &callbacks.on_select_accent_color),
                ))
                .child(render_two_column_section(
                    theme,
                    "EXPERIENCE FEATURES",
                    "Preview Shell",
                    "Enable experimental experience features such as the native GPUI preview shell.",
                    render_experience_section(
                        theme,
                        model.enable_experience_features,
                        &callbacks.on_toggle_experience_features,
                    ),
                ))
                .child(render_two_column_section(
                    theme,
                    "SHELL & STORAGE",
                    "Runtime Diagnostics",
                    "Shared configuration file location and current workspace shell layout.",
                    render_storage_well(model),
                )),
        )
        .into_any_element()
}

fn render_settings_header(
    theme: &OrbitTheme,
    modified: bool,
    on_reset: &SettingsActionCallback,
) -> AnyElement {
    div()
        .flex()
        .items_center()
        .justify_between()
        .gap_4()
        .child(
            div()
                .flex()
                .flex_col()
                .gap_1()
                .child(
                    apply_text_step(div(), theme.typography.size_xl)
                        .font_weight(FontWeight::BOLD)
                        .text_color(theme.text.primary)
                        .child("Settings"),
                )
                .child(
                    apply_text_step(div(), theme.typography.size_xs)
                        .text_color(theme.text.secondary)
                        .child("Appearance, surface aesthetics, and native shell preferences saved to ~/.orbit/config.json."),
                ),
        )
        .child(render_action_button(
            "settings-reset-appearance-btn",
            Icon::RotateCcw,
            "Reset Defaults",
            !modified,
            theme,
            Rc::clone(on_reset),
        ))
        .into_any_element()
}

fn render_save_error_banner(theme: &OrbitTheme, message: &str) -> AnyElement {
    div()
        .flex()
        .items_center()
        .gap_2()
        .px_3()
        .py_2()
        .rounded(theme.style.radius_control)
        .bg(theme.semantic.danger.soft)
        .border_1()
        .border_color(theme.semantic.danger.solid)
        .child(render_icon(
            Icon::AlertTriangle,
            px(14.0),
            theme.semantic.danger.solid,
        ))
        .child(
            apply_text_step(div(), theme.typography.size_xs)
                .text_color(theme.semantic.danger.solid)
                .font_weight(FontWeight::MEDIUM)
                .child(format!("Failed to save settings: {message}")),
        )
        .into_any_element()
}

fn render_two_column_section(
    theme: &OrbitTheme,
    eyebrow: &'static str,
    title: &'static str,
    description: &'static str,
    content: AnyElement,
) -> AnyElement {
    div()
        .flex()
        .flex_row()
        .items_start()
        .gap_8()
        .w_full()
        .child(
            div()
                .w(px(230.0))
                .flex_shrink_0()
                .flex()
                .flex_col()
                .gap_1p5()
                .child(
                    apply_text_step(div(), theme.typography.size_2xs)
                        .font_weight(FontWeight::SEMIBOLD)
                        .text_color(theme.text.muted)
                        .child(eyebrow),
                )
                .child(
                    apply_text_step(div(), theme.typography.size_sm)
                        .font_weight(FontWeight::SEMIBOLD)
                        .text_color(theme.text.primary)
                        .child(title),
                )
                .child(
                    apply_text_step(div(), theme.typography.size_xs)
                        .text_color(theme.text.secondary)
                        .child(description),
                ),
        )
        .child(div().flex_1().min_w_0().child(content))
        .into_any_element()
}

fn render_theme_mode_cards(
    theme: &OrbitTheme,
    resolved_system_is_dark: bool,
    on_select: &SettingsValueCallback<ThemeMode>,
) -> AnyElement {
    let mut row = div().flex().flex_row().gap_3().w_full();

    for mode in ThemeMode::ALL {
        let is_selected = theme.mode == mode;
        let icon = match mode {
            ThemeMode::Dark => Icon::Moon,
            ThemeMode::Light => Icon::Sun,
            ThemeMode::System => Icon::Monitor,
        };
        let subtitle = match mode {
            ThemeMode::System => {
                let resolved = if resolved_system_is_dark { "Dark" } else { "Light" };
                format!("{} (Currently {resolved})", mode.description())
            }
            _ => mode.description().to_string(),
        };

        let border_color = if is_selected {
            theme.accent.accent
        } else {
            theme.border.card
        };
        let bg_color = if is_selected {
            theme.accent.soft
        } else {
            theme.surface.bg_card
        };
        let hover_bg = if is_selected {
            theme.accent.soft
        } else {
            theme.surface.bg_hover
        };
        let callback = Rc::clone(on_select);

        row = row.child(
            div()
                .id(("settings-theme-mode", mode as u32))
                .flex_1()
                .min_w_0()
                .p_3()
                .rounded(theme.style.radius_card)
                .border_1()
                .border_color(border_color)
                .bg(bg_color)
                .cursor_pointer()
                .hover(move |s| s.bg(hover_bg))
                .on_click(move |_, window, cx| callback(mode, window, cx))
                .flex()
                .flex_col()
                .gap_2()
                .child(
                    div()
                        .flex()
                        .items_center()
                        .justify_between()
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .gap_2()
                                .child(
                                    div()
                                        .w(px(26.0))
                                        .h(px(26.0))
                                        .rounded(theme.style.radius_control)
                                        .bg(theme.surface.bg_panel)
                                        .flex()
                                        .items_center()
                                        .justify_center()
                                        .child(render_icon(
                                            icon,
                                            px(14.0),
                                            if is_selected {
                                                theme.accent.accent
                                            } else {
                                                theme.text.secondary
                                            },
                                        )),
                                )
                                .child(
                                    apply_text_step(div(), theme.typography.size_sm)
                                        .font_weight(FontWeight::SEMIBOLD)
                                        .text_color(theme.text.primary)
                                        .child(mode.label()),
                                ),
                        )
                        .when(is_selected, |el| {
                            el.child(render_icon(Icon::Check, px(14.0), theme.accent.accent))
                        }),
                )
                .child(
                    apply_text_step(div(), theme.typography.size_xs)
                        .text_color(theme.text.secondary)
                        .child(subtitle),
                ),
        );
    }

    row.into_any_element()
}

fn render_ui_style_cards(
    theme: &OrbitTheme,
    on_select: &SettingsValueCallback<UiStyle>,
) -> AnyElement {
    let mut col = div().flex().flex_col().gap_3().w_full();

    for chunk in UiStyle::ALL.chunks(2) {
        let mut row = div().flex().flex_row().gap_3().w_full();
        for &style in chunk {
            let is_selected = theme.style.ui_style == style;
            let border_color = if is_selected {
                theme.accent.accent
            } else {
                theme.border.card
            };
            let bg_color = if is_selected {
                theme.accent.soft
            } else {
                theme.surface.bg_card
            };
            let hover_bg = if is_selected {
                theme.accent.soft
            } else {
                theme.surface.bg_hover
            };
            let callback = Rc::clone(on_select);

            row = row.child(
                div()
                    .id(("settings-ui-style", style as u32))
                    .flex_1()
                    .min_w_0()
                    .p_3()
                    .rounded(theme.style.radius_card)
                    .border_1()
                    .border_color(border_color)
                    .bg(bg_color)
                    .cursor_pointer()
                    .hover(move |s| s.bg(hover_bg))
                    .on_click(move |_, window, cx| callback(style, window, cx))
                    .flex()
                    .flex_col()
                    .gap_1p5()
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .justify_between()
                            .gap_2()
                            .child(
                                div()
                                    .flex()
                                    .items_center()
                                    .gap_2()
                                    .child(
                                        apply_text_step(div(), theme.typography.size_sm)
                                            .font_weight(FontWeight::SEMIBOLD)
                                            .text_color(theme.text.primary)
                                            .child(style.label()),
                                    )
                                    .when_some(style.badge(), |el, badge| {
                                        el.child(
                                            apply_text_step(div(), theme.typography.size_2xs)
                                                .px_1p5()
                                                .py_0p5()
                                                .rounded(theme.style.radius_control)
                                                .bg(theme.surface.bg_panel)
                                                .text_color(theme.text.secondary)
                                                .font_weight(FontWeight::MEDIUM)
                                                .child(badge),
                                        )
                                    }),
                            )
                            .when(is_selected, |el| {
                                el.child(render_icon(Icon::Check, px(14.0), theme.accent.accent))
                            }),
                    )
                    .child(
                        apply_text_step(div(), theme.typography.size_xs)
                            .text_color(theme.text.secondary)
                            .child(style.description()),
                    ),
            );
        }
        col = col.child(row);
    }

    col.into_any_element()
}

fn render_accent_section(
    theme: &OrbitTheme,
    on_select: &SettingsValueCallback<AccentColor>,
) -> AnyElement {
    let mut swatch_row = div().flex().flex_row().items_center().gap_3();

    for accent in AccentColor::ALL {
        let is_selected = theme.accent_id == accent;
        let tokens = resolve_accent_tokens(accent, theme.is_dark);
        let callback = Rc::clone(on_select);

        swatch_row = swatch_row.child(
            div()
                .id(("settings-accent-swatch", accent as u32))
                .flex()
                .flex_col()
                .items_center()
                .gap_1()
                .cursor_pointer()
                .on_click(move |_, window, cx| callback(accent, window, cx))
                .child(
                    div()
                        .w(px(28.0))
                        .h(px(28.0))
                        .rounded_full()
                        .border_2()
                        .border_color(if is_selected {
                            theme.text.primary
                        } else {
                            theme.border.default
                        })
                        .p(px(2.0))
                        .flex()
                        .items_center()
                        .justify_center()
                        .child(
                            div()
                                .w_full()
                                .h_full()
                                .rounded_full()
                                .bg(tokens.accent)
                                .flex()
                                .items_center()
                                .justify_center()
                                .when(is_selected, |el| {
                                    el.child(render_icon(
                                        Icon::Check,
                                        px(12.0),
                                        theme.surface.bg_app,
                                    ))
                                }),
                        ),
                )
                .child(
                    apply_text_step(div(), theme.typography.size_2xs)
                        .font_weight(if is_selected {
                            FontWeight::SEMIBOLD
                        } else {
                            FontWeight::NORMAL
                        })
                        .text_color(if is_selected {
                            theme.text.primary
                        } else {
                            theme.text.muted
                        })
                        .child(accent.label()),
                ),
        );
    }

    let preview_strip = div()
        .p_3()
        .rounded(theme.style.radius_card)
        .bg(theme.surface.bg_panel)
        .border_1()
        .border_color(theme.border.default)
        .flex()
        .items_center()
        .justify_between()
        .gap_3()
        .child(
            div()
                .flex()
                .items_center()
                .gap_2()
                .child(
                    div()
                        .px_2p5()
                        .py_1()
                        .rounded(theme.style.radius_control)
                        .bg(theme.accent.accent)
                        .child(
                            apply_text_step(div(), theme.typography.size_xs)
                                .font_weight(FontWeight::SEMIBOLD)
                                .text_color(theme.surface.bg_app)
                                .child("Primary Action"),
                        ),
                )
                .child(
                    div()
                        .px_2p5()
                        .py_1()
                        .rounded(theme.style.radius_control)
                        .bg(theme.accent.soft)
                        .border_1()
                        .border_color(theme.accent.accent)
                        .child(
                            apply_text_step(div(), theme.typography.size_xs)
                                .font_weight(FontWeight::MEDIUM)
                                .text_color(theme.accent.accent)
                                .child("Active Selection"),
                        ),
                ),
        )
        .child(
            div()
                .flex()
                .items_center()
                .gap_2()
                .child(render_semantic_pill("Healthy", theme.semantic.success.soft, theme.semantic.success.solid, theme))
                .child(render_semantic_pill("Pending", theme.semantic.warning.soft, theme.semantic.warning.solid, theme))
                .child(render_semantic_pill("Error", theme.semantic.danger.soft, theme.semantic.danger.solid, theme)),
        );

    div()
        .flex()
        .flex_col()
        .gap_3()
        .child(swatch_row)
        .child(preview_strip)
        .into_any_element()
}

fn render_semantic_pill(
    label: &'static str,
    bg: gpui::Rgba,
    fg: gpui::Rgba,
    theme: &OrbitTheme,
) -> AnyElement {
    apply_text_step(div(), theme.typography.size_2xs)
        .px_2()
        .py_0p5()
        .rounded(theme.style.radius_control)
        .bg(bg)
        .text_color(fg)
        .font_weight(FontWeight::SEMIBOLD)
        .child(label)
        .into_any_element()
}

fn render_experience_section(
    theme: &OrbitTheme,
    enabled: bool,
    on_toggle: &SettingsValueCallback<bool>,
) -> AnyElement {
    let next_enabled = !enabled;
    let callback = Rc::clone(on_toggle);
    let track_bg = if enabled {
        theme.accent.accent
    } else {
        theme.surface.bg_panel
    };
    let track_border = if enabled {
        theme.accent.accent
    } else {
        theme.border.strong
    };

    div()
        .p_3()
        .rounded(theme.style.radius_card)
        .bg(theme.surface.bg_card)
        .border_1()
        .border_color(theme.border.card)
        .flex()
        .items_center()
        .justify_between()
        .gap_4()
        .child(
            div()
                .flex()
                .flex_col()
                .gap_1()
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap_2()
                        .child(
                            apply_text_step(div(), theme.typography.size_sm)
                                .font_weight(FontWeight::SEMIBOLD)
                                .text_color(theme.text.primary)
                                .child("Native GPUI Preview Shell"),
                        )
                        .child(
                            apply_text_step(div(), theme.typography.size_2xs)
                                .px_1p5()
                                .py_0p5()
                                .rounded(theme.style.radius_control)
                                .bg(if enabled {
                                    theme.semantic.success.soft
                                } else {
                                    theme.surface.bg_panel
                                })
                                .text_color(if enabled {
                                    theme.semantic.success.solid
                                } else {
                                    theme.text.muted
                                })
                                .font_weight(FontWeight::SEMIBOLD)
                                .child(experience_features_status_label(enabled)),
                        ),
                )
                .child(
                    apply_text_step(div(), theme.typography.size_xs)
                        .text_color(theme.text.secondary)
                        .child("Sets enableExperienceFeatures in config.json for desktop shell previews."),
                ),
        )
        .child(
            div()
                .id("settings-experience-toggle")
                .w(px(38.0))
                .h(px(22.0))
                .rounded_full()
                .bg(track_bg)
                .border_1()
                .border_color(track_border)
                .p(px(2.0))
                .cursor_pointer()
                .flex()
                .items_center()
                .justify_end()
                .when(!enabled, |el| el.justify_start())
                .on_click(move |_, window, cx| callback(next_enabled, window, cx))
                .child(
                    div()
                        .w(px(16.0))
                        .h(px(16.0))
                        .rounded_full()
                        .bg(if enabled {
                            theme.surface.bg_app
                        } else {
                            theme.text.secondary
                        }),
                ),
        )
        .into_any_element()
}

fn render_storage_well(model: &SettingsViewModel<'_>) -> AnyElement {
    let theme = model.theme;
    let config_path = model
        .config_file_path
        .clone()
        .unwrap_or_else(|| "~/.orbit/config.json".to_string());

    div()
        .p_3()
        .rounded(theme.style.radius_card)
        .bg(theme.surface.bg_panel)
        .flex()
        .flex_col()
        .gap_2()
        .child(render_detail_row("Config File", config_path, true, theme))
        .child(render_detail_row(
            "Sidebar Layout",
            model.sidebar_status.clone(),
            true,
            theme,
        ))
        .into_any_element()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::theme::SemanticColorsPreference;

    #[test]
    fn test_settings_view_model_modification_detection_and_labels() {
        let default_theme = OrbitTheme::resolve(
            ThemeMode::Dark,
            UiStyle::Flat,
            AccentColor::Blue,
            SemanticColorsPreference::default(),
            WindowAppearance::Dark,
        );
        assert!(!is_appearance_modified(&default_theme));

        let custom_mode = default_theme.with_mode(ThemeMode::Light, WindowAppearance::Dark);
        assert!(is_appearance_modified(&custom_mode));

        let custom_style = default_theme.with_ui_style(UiStyle::Glass, WindowAppearance::Dark);
        assert!(is_appearance_modified(&custom_style));

        let custom_accent =
            default_theme.with_accent_color(AccentColor::Emerald, WindowAppearance::Dark);
        assert!(is_appearance_modified(&custom_accent));

        assert_eq!(system_mode_hint(WindowAppearance::Dark), "Dark");
        assert_eq!(system_mode_hint(WindowAppearance::VibrantDark), "Dark");
        assert_eq!(system_mode_hint(WindowAppearance::Light), "Light");
        assert_eq!(system_mode_hint(WindowAppearance::VibrantLight), "Light");

        assert_eq!(experience_features_status_label(true), "Enabled");
        assert_eq!(experience_features_status_label(false), "Disabled");
    }
}
