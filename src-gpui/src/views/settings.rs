use std::rc::Rc;

use gpui::component::{Disableable as _, Sizable as _, button::Button, switch::Switch};
use gpui::{
    Anchor, AnyElement, App, FontWeight, MouseButton, Window, WindowAppearance, anchored, deferred,
    div, point, prelude::*, px,
};
use lucide_icons::Icon;

use crate::dropdown_chevron_icon;
use crate::icons::render_icon;
use crate::theme::{OrbitTheme, ThemeMode, apply_text_step};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SettingsDropdownKind {
    ThemeMode,
}

impl SettingsDropdownKind {
    const fn option_count(self) -> usize {
        match self {
            Self::ThemeMode => ThemeMode::ALL.len(),
        }
    }

    const fn trigger_id(self) -> &'static str {
        match self {
            Self::ThemeMode => "settings-theme-mode-trigger",
        }
    }

    const fn menu_id(self) -> &'static str {
        match self {
            Self::ThemeMode => "settings-theme-mode-menu",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SettingsDropdownKeyAction {
    Ignored,
    Navigated,
    Closed,
    SelectThemeMode(ThemeMode),
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SettingsDropdownState {
    pub open: Option<SettingsDropdownKind>,
    pub highlighted_index: usize,
}

impl SettingsDropdownState {
    pub fn is_open(&self, kind: SettingsDropdownKind) -> bool {
        self.open == Some(kind)
    }

    pub fn is_any_open(&self) -> bool {
        self.open.is_some()
    }

    pub fn toggle(&mut self, kind: SettingsDropdownKind, current_index: usize) {
        if self.open == Some(kind) {
            self.close();
        } else {
            let max_idx = kind.option_count().saturating_sub(1);
            self.open = Some(kind);
            self.highlighted_index = current_index.min(max_idx);
        }
    }

    pub fn close(&mut self) {
        self.open = None;
    }

    pub fn set_highlight(&mut self, index: usize) {
        if let Some(kind) = self.open
            && index < kind.option_count()
        {
            self.highlighted_index = index;
        }
    }

    pub fn move_highlight(&mut self, delta: isize) {
        let Some(kind) = self.open else {
            return;
        };
        let len = kind.option_count() as isize;
        if len <= 0 {
            return;
        }
        let next = (self.highlighted_index as isize + delta).rem_euclid(len);
        self.highlighted_index = next as usize;
    }

    pub fn handle_key(&mut self, key: &str) -> SettingsDropdownKeyAction {
        let Some(kind) = self.open else {
            return SettingsDropdownKeyAction::Ignored;
        };
        match key {
            "escape" => {
                self.close();
                SettingsDropdownKeyAction::Closed
            }
            "down" | "arrowdown" => {
                self.move_highlight(1);
                SettingsDropdownKeyAction::Navigated
            }
            "up" | "arrowup" => {
                self.move_highlight(-1);
                SettingsDropdownKeyAction::Navigated
            }
            "enter" | "return" => {
                let idx = self.highlighted_index;
                self.close();
                match kind {
                    SettingsDropdownKind::ThemeMode => ThemeMode::ALL
                        .get(idx)
                        .copied()
                        .map(SettingsDropdownKeyAction::SelectThemeMode)
                        .unwrap_or(SettingsDropdownKeyAction::Closed),
                }
            }
            _ => SettingsDropdownKeyAction::Ignored,
        }
    }
}

pub fn theme_mode_dropdown_label(mode: ThemeMode, resolved_system_is_dark: bool) -> String {
    match mode {
        ThemeMode::System => {
            let resolved = if resolved_system_is_dark {
                "Dark"
            } else {
                "Light"
            };
            format!("{} ({resolved})", mode.label())
        }
        _ => mode.label().to_string(),
    }
}

pub fn theme_mode_index(mode: ThemeMode) -> usize {
    ThemeMode::ALL
        .iter()
        .position(|&candidate| candidate == mode)
        .unwrap_or(0)
}

pub struct SettingsViewModel<'a> {
    pub theme: &'a OrbitTheme,
    pub resolved_system_is_dark: bool,
    pub enable_experience_features: bool,
    pub config_file_path: Option<String>,
    pub sidebar_status: String,
    pub save_error: Option<&'a str>,
    pub dropdown: &'a SettingsDropdownState,
}

pub type SettingsActionCallback = Rc<dyn Fn(&mut Window, &mut App)>;
pub type SettingsValueCallback<T> = Rc<dyn Fn(T, &mut Window, &mut App)>;

pub struct SettingsCallbacks {
    pub on_select_theme_mode: SettingsValueCallback<ThemeMode>,
    pub on_toggle_experience_features: SettingsValueCallback<bool>,
    pub on_reset_appearance: SettingsActionCallback,
    pub on_toggle_dropdown: SettingsValueCallback<SettingsDropdownKind>,
    pub on_hover_dropdown_option: SettingsValueCallback<usize>,
}

pub fn is_appearance_modified(theme: &OrbitTheme) -> bool {
    theme.mode != ThemeMode::Dark
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
    let config_path = model
        .config_file_path
        .clone()
        .unwrap_or_else(|| "~/.orbit/config.json".to_string());

    div()
        .id("settings-view-root")
        .w_full()
        .flex()
        .flex_col()
        .gap_8()
        .max_w(px(880.0))
        .pb_8()
        .child(render_settings_header(
            theme,
            modified,
            &callbacks.on_reset_appearance,
        ))
        .when_some(model.save_error, |el, error_msg| {
            el.child(render_save_error_banner(theme, error_msg))
        })
        .child(render_settings_section(
            theme,
            "APPEARANCE",
            vec![render_setting_row(
                theme,
                "Theme Mode",
                None,
                "Switch between dark, light, or follow your OS appearance.",
                render_theme_mode_dropdown(
                    theme,
                    model.resolved_system_is_dark,
                    model.dropdown,
                    callbacks,
                ),
            )],
        ))
        .child(render_settings_section(
            theme,
            "EXPERIENCE FEATURES",
            vec![render_setting_row(
                theme,
                "Native GPUI Preview Shell",
                Some(render_experience_status_badge(
                    theme,
                    model.enable_experience_features,
                )),
                "Sets enableExperienceFeatures in config.json for desktop shell previews.",
                render_experience_toggle(
                    theme,
                    model.enable_experience_features,
                    &callbacks.on_toggle_experience_features,
                ),
            )],
        ))
        .child(render_settings_section(
            theme,
            "SHELL & STORAGE",
            vec![
                render_setting_row(
                    theme,
                    "Config File",
                    None,
                    "Shared configuration file location for appearance and shell preferences.",
                    render_mono_value(theme, config_path),
                ),
                render_setting_row(
                    theme,
                    "Sidebar Layout",
                    None,
                    "Current workspace sidebar collapse state and width.",
                    render_mono_value(theme, model.sidebar_status.clone()),
                ),
            ],
        ))
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
                        .child("Appearance and native shell preferences saved to ~/.orbit/config.json."),
                ),
        )
        .child(
            Button::new("settings-reset-appearance-btn")
                .outline()
                .small()
                .accessibility_label("Reset Defaults")
                .disabled(!modified)
                .on_click({
                    let on_reset = Rc::clone(on_reset);
                    move |_, window, cx| on_reset(window, cx)
                })
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap_1p5()
                        .child(render_icon(Icon::RotateCcw, px(13.0), theme.text.secondary))
                        .child("Reset Defaults"),
                ),
        )
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

fn render_settings_section(
    theme: &OrbitTheme,
    eyebrow: &'static str,
    rows: Vec<AnyElement>,
) -> AnyElement {
    div()
        .flex()
        .flex_col()
        .gap_2()
        .w_full()
        .child(
            apply_text_step(div(), theme.typography.size_2xs)
                .font_weight(FontWeight::SEMIBOLD)
                .text_color(theme.text.muted)
                .px_1()
                .child(eyebrow),
        )
        .child(
            div()
                .w_full()
                .p_1p5()
                .rounded(theme.style.radius_card)
                .bg(theme.surface.bg_panel)
                .flex()
                .flex_col()
                .gap_0p5()
                .children(rows),
        )
        .into_any_element()
}

fn render_setting_row(
    theme: &OrbitTheme,
    title: &'static str,
    badge: Option<AnyElement>,
    description: &'static str,
    control: AnyElement,
) -> AnyElement {
    div()
        .w_full()
        .px_3()
        .py_2p5()
        .rounded(theme.style.radius_control)
        .flex()
        .flex_row()
        .items_center()
        .justify_between()
        .gap_6()
        .child(
            div()
                .flex_1()
                .min_w_0()
                .flex()
                .flex_col()
                .gap_0p5()
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap_2()
                        .child(
                            apply_text_step(div(), theme.typography.size_sm)
                                .font_weight(FontWeight::MEDIUM)
                                .text_color(theme.text.primary)
                                .child(title),
                        )
                        .when_some(badge, |el, b| el.child(b)),
                )
                .child(
                    apply_text_step(div(), theme.typography.size_xs)
                        .text_color(theme.text.secondary)
                        .child(description),
                ),
        )
        .child(
            div()
                .flex_shrink_0()
                .flex()
                .items_center()
                .justify_end()
                .child(control),
        )
        .into_any_element()
}

const fn theme_mode_icon(mode: ThemeMode) -> Icon {
    match mode {
        ThemeMode::Dark => Icon::Moon,
        ThemeMode::Light => Icon::Sun,
        ThemeMode::System => Icon::Monitor,
    }
}

fn render_dropdown_control(
    theme: &OrbitTheme,
    kind: SettingsDropdownKind,
    is_open: bool,
    leading_icon: Option<Icon>,
    label: String,
    on_toggle: &SettingsValueCallback<SettingsDropdownKind>,
    options: Vec<AnyElement>,
) -> AnyElement {
    let control_hover = theme.surface.control_hover;
    let toggle_cb = Rc::clone(on_toggle);

    let trigger = theme
        .style_control(div().id(kind.trigger_id()))
        .w(px(210.0))
        .h(theme.typography.control_height_sm)
        .px(theme.typography.control_padding_x_sm)
        .flex()
        .items_center()
        .justify_between()
        .gap_2()
        .cursor_pointer()
        .hover(move |s| s.bg(control_hover))
        .when(is_open, |el| el.border_color(theme.accent.accent))
        .on_mouse_down(MouseButton::Left, move |_, window, cx| {
            cx.stop_propagation();
            toggle_cb(kind, window, cx);
        })
        .child(
            div()
                .flex()
                .items_center()
                .gap_2()
                .min_w_0()
                .flex_1()
                .when_some(leading_icon, |el, icon| {
                    el.child(render_icon(icon, px(13.0), theme.text.secondary))
                })
                .child(
                    apply_text_step(div(), theme.typography.size_xs)
                        .font_weight(FontWeight::MEDIUM)
                        .text_color(theme.text.primary)
                        .truncate()
                        .child(label),
                ),
        )
        .child(render_icon(
            dropdown_chevron_icon(is_open),
            px(13.0),
            theme.text.secondary,
        ));

    #[cfg(test)]
    let trigger = {
        use gpui::test::TestSupportExt as _;
        trigger.test_support()
    };

    div()
        .flex()
        .flex_col()
        .child(trigger)
        .when(is_open, |el| {
            let menu = theme
                .style_overlay(div().id(kind.menu_id()))
                .w(px(230.0))
                .p_1()
                .flex()
                .flex_col()
                .gap_0p5()
                .on_mouse_down(MouseButton::Left, |_, _, cx| cx.stop_propagation())
                .children(options);

            el.child(
                deferred(
                    anchored()
                        .anchor(Anchor::TopLeft)
                        .offset(point(px(0.0), px(4.0)))
                        .snap_to_window_with_margin(px(8.0))
                        .child(menu),
                )
                .with_priority(2),
            )
        })
        .into_any_element()
}

struct DropdownOptionProps {
    id_prefix: &'static str,
    idx: usize,
    is_selected: bool,
    is_highlighted: bool,
    leading_icon: Option<Icon>,
    label: &'static str,
    trailing_meta: Option<AnyElement>,
}

fn render_dropdown_option(
    theme: &OrbitTheme,
    props: DropdownOptionProps,
    on_hover: SettingsValueCallback<usize>,
    on_select: SettingsActionCallback,
) -> AnyElement {
    let idx = props.idx;
    let row_bg = if props.is_selected {
        Some(theme.accent.soft)
    } else if props.is_highlighted {
        Some(theme.surface.bg_hover)
    } else {
        None
    };

    div()
        .id((props.id_prefix, idx))
        .px_2p5()
        .py_1p5()
        .rounded(theme.style.radius_control)
        .flex()
        .items_center()
        .justify_between()
        .gap_2()
        .cursor_pointer()
        .when_some(row_bg, |el, bg| el.bg(bg))
        .on_hover(move |hovered, window, cx| {
            if *hovered {
                on_hover(idx, window, cx);
            }
        })
        .on_mouse_down(MouseButton::Left, move |_, window, cx| {
            cx.stop_propagation();
            on_select(window, cx);
        })
        .child(
            div()
                .flex()
                .items_center()
                .gap_2()
                .min_w_0()
                .when_some(props.leading_icon, |el, icon| {
                    let color = if props.is_selected {
                        theme.accent.accent
                    } else {
                        theme.text.secondary
                    };
                    el.child(render_icon(icon, px(13.0), color))
                })
                .child(
                    apply_text_step(div(), theme.typography.size_xs)
                        .font_weight(FontWeight::MEDIUM)
                        .text_color(theme.text.primary)
                        .child(props.label),
                )
                .when_some(props.trailing_meta, |el, meta| el.child(meta)),
        )
        .when(props.is_selected, |el| {
            el.child(render_icon(Icon::Check, px(13.0), theme.accent.accent))
        })
        .into_any_element()
}

fn render_theme_mode_dropdown(
    theme: &OrbitTheme,
    resolved_system_is_dark: bool,
    dropdown: &SettingsDropdownState,
    callbacks: &SettingsCallbacks,
) -> AnyElement {
    let is_open = dropdown.is_open(SettingsDropdownKind::ThemeMode);
    let label = theme_mode_dropdown_label(theme.mode, resolved_system_is_dark);

    let options = if is_open {
        ThemeMode::ALL
            .iter()
            .copied()
            .enumerate()
            .map(|(idx, mode)| {
                let on_select_mode = Rc::clone(&callbacks.on_select_theme_mode);
                let resolved_hint =
                    (mode == ThemeMode::System).then_some(if resolved_system_is_dark {
                        "(Dark)"
                    } else {
                        "(Light)"
                    });
                let trailing_meta = resolved_hint.map(|hint| {
                    apply_text_step(div(), theme.typography.size_2xs)
                        .text_color(theme.text.muted)
                        .child(hint)
                        .into_any_element()
                });

                render_dropdown_option(
                    theme,
                    DropdownOptionProps {
                        id_prefix: "settings-theme-mode-opt",
                        idx,
                        is_selected: theme.mode == mode,
                        is_highlighted: dropdown.highlighted_index == idx,
                        leading_icon: Some(theme_mode_icon(mode)),
                        label: mode.label(),
                        trailing_meta,
                    },
                    Rc::clone(&callbacks.on_hover_dropdown_option),
                    Rc::new(move |window, cx| on_select_mode(mode, window, cx)),
                )
            })
            .collect()
    } else {
        Vec::new()
    };

    render_dropdown_control(
        theme,
        SettingsDropdownKind::ThemeMode,
        is_open,
        Some(theme_mode_icon(theme.mode)),
        label,
        &callbacks.on_toggle_dropdown,
        options,
    )
}

fn render_experience_status_badge(theme: &OrbitTheme, enabled: bool) -> AnyElement {
    apply_text_step(div(), theme.typography.size_2xs)
        .px_1p5()
        .py_0p5()
        .rounded(theme.style.radius_control)
        .bg(if enabled {
            theme.semantic.success.soft
        } else {
            theme.surface.bg_hover
        })
        .text_color(if enabled {
            theme.semantic.success.solid
        } else {
            theme.text.muted
        })
        .font_weight(FontWeight::SEMIBOLD)
        .child(experience_features_status_label(enabled))
        .into_any_element()
}

fn render_experience_toggle(
    _theme: &OrbitTheme,
    enabled: bool,
    on_toggle: &SettingsValueCallback<bool>,
) -> AnyElement {
    let callback = Rc::clone(on_toggle);
    Switch::new("settings-experience-toggle")
        .accessibility_label("Native GPUI Preview Shell")
        .checked(enabled)
        .on_click(move |checked, window, cx| callback(*checked, window, cx))
        .into_any_element()
}

fn render_mono_value(theme: &OrbitTheme, value: String) -> AnyElement {
    apply_text_step(div(), theme.typography.size_xs)
        .font(theme.typography.mono_font(FontWeight::NORMAL))
        .text_color(theme.text.secondary)
        .child(value)
        .into_any_element()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::theme::{AccentColor, SemanticColorsPreference, UiStyle};

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

        let system_mode = default_theme.with_mode(ThemeMode::System, WindowAppearance::Dark);
        assert!(is_appearance_modified(&system_mode));

        let non_default_style_and_accent = OrbitTheme::resolve(
            ThemeMode::Dark,
            UiStyle::Glass,
            AccentColor::Emerald,
            SemanticColorsPreference::default(),
            WindowAppearance::Dark,
        );
        assert!(!is_appearance_modified(&non_default_style_and_accent));

        assert_eq!(system_mode_hint(WindowAppearance::Dark), "Dark");
        assert_eq!(system_mode_hint(WindowAppearance::VibrantDark), "Dark");
        assert_eq!(system_mode_hint(WindowAppearance::Light), "Light");
        assert_eq!(system_mode_hint(WindowAppearance::VibrantLight), "Light");

        assert_eq!(experience_features_status_label(true), "Enabled");
        assert_eq!(experience_features_status_label(false), "Disabled");
    }

    #[test]
    fn test_settings_dropdown_toggle_switch_and_close() {
        let mut state = SettingsDropdownState::default();
        assert!(!state.is_any_open());
        assert!(!state.is_open(SettingsDropdownKind::ThemeMode));

        state.toggle(SettingsDropdownKind::ThemeMode, 1);
        assert!(state.is_any_open());
        assert!(state.is_open(SettingsDropdownKind::ThemeMode));
        assert_eq!(state.highlighted_index, 1);

        // Toggling the already-open dropdown closes it
        state.toggle(SettingsDropdownKind::ThemeMode, 1);
        assert!(!state.is_any_open());
        assert_eq!(state.open, None);

        // Open again and verify explicit close()
        state.toggle(SettingsDropdownKind::ThemeMode, 2);
        assert!(state.is_open(SettingsDropdownKind::ThemeMode));
        assert_eq!(state.highlighted_index, 2);
        state.close();
        assert_eq!(state.open, None);
        assert!(!state.is_any_open());
    }

    #[test]
    fn test_settings_dropdown_keyboard_navigation_wraps_and_activates() {
        let mut state = SettingsDropdownState::default();
        assert_eq!(
            state.handle_key("arrowdown"),
            SettingsDropdownKeyAction::Ignored
        );

        // ThemeMode: 0 = Dark, 1 = Light, 2 = System
        state.toggle(SettingsDropdownKind::ThemeMode, 0);
        assert_eq!(
            state.handle_key("arrowup"),
            SettingsDropdownKeyAction::Navigated
        );
        assert_eq!(state.highlighted_index, ThemeMode::ALL.len() - 1);
        assert_eq!(
            state.handle_key("arrowdown"),
            SettingsDropdownKeyAction::Navigated
        );
        assert_eq!(state.highlighted_index, 0);
        assert_eq!(
            state.handle_key("arrowdown"),
            SettingsDropdownKeyAction::Navigated
        );
        assert_eq!(state.highlighted_index, 1);
        assert_eq!(
            state.handle_key("enter"),
            SettingsDropdownKeyAction::SelectThemeMode(ThemeMode::Light)
        );
        assert!(!state.is_any_open());

        // Escape closes without selecting
        state.toggle(SettingsDropdownKind::ThemeMode, 2);
        assert_eq!(
            state.handle_key("escape"),
            SettingsDropdownKeyAction::Closed
        );
        assert!(!state.is_any_open());
    }

    #[test]
    fn test_settings_dropdown_labels_and_indices() {
        assert_eq!(theme_mode_dropdown_label(ThemeMode::Dark, false), "Dark");
        assert_eq!(theme_mode_dropdown_label(ThemeMode::Light, true), "Light");
        assert_eq!(
            theme_mode_dropdown_label(ThemeMode::System, true),
            "System (Dark)"
        );
        assert_eq!(
            theme_mode_dropdown_label(ThemeMode::System, false),
            "System (Light)"
        );

        for (expected_idx, mode) in ThemeMode::ALL.iter().copied().enumerate() {
            assert_eq!(theme_mode_index(mode), expected_idx);
        }
    }

    #[gpui::test]
    fn test_settings_view_hides_ui_style_and_accent_controls(cx: &mut gpui::TestAppContext) {
        use gpui::test::TestWindowExt as _;

        struct SettingsViewHarness {
            theme: OrbitTheme,
            dropdown: SettingsDropdownState,
        }

        impl gpui::Render for SettingsViewHarness {
            fn render(
                &mut self,
                _window: &mut Window,
                _cx: &mut gpui::Context<Self>,
            ) -> impl IntoElement {
                let model = SettingsViewModel {
                    theme: &self.theme,
                    resolved_system_is_dark: true,
                    enable_experience_features: false,
                    config_file_path: Some("~/.orbit/config.json".to_string()),
                    sidebar_status: "Expanded (220px)".to_string(),
                    save_error: None,
                    dropdown: &self.dropdown,
                };
                let callbacks = SettingsCallbacks {
                    on_select_theme_mode: Rc::new(|_, _, _| {}),
                    on_toggle_experience_features: Rc::new(|_, _, _| {}),
                    on_reset_appearance: Rc::new(|_, _| {}),
                    on_toggle_dropdown: Rc::new(|_, _, _| {}),
                    on_hover_dropdown_option: Rc::new(|_, _, _| {}),
                };
                div()
                    .p_4()
                    .child(render_settings_view(&model, &callbacks))
            }
        }

        let theme = OrbitTheme::resolve(
            ThemeMode::Dark,
            UiStyle::Flat,
            AccentColor::Blue,
            SemanticColorsPreference::default(),
            WindowAppearance::Dark,
        );

        let window = cx.update(|cx| {
            gpui::init(cx);
            crate::theme::kit_bridge::sync_kit_theme(&theme, cx);
            gpui::open_window(gpui::WindowOptions::default(), cx, move |_, cx| {
                cx.new(|_| SettingsViewHarness {
                    theme: theme.clone(),
                    dropdown: SettingsDropdownState::default(),
                })
            })
        });
        let Ok((window, _view)) = window else {
            panic!("failed to open test window");
        };

        cx.update(|cx| {
            let _ = window.update(cx, |_, window, cx| {
                window.render_frame(cx);
                assert!(window.try_find("settings-theme-mode-trigger").is_some());
                assert!(window.try_find("settings-reset-appearance-btn").is_some());
                assert!(window.try_find("settings-experience-toggle").is_some());
                assert!(window.try_find("settings-ui-style-trigger").is_none());
                assert!(
                    window
                        .try_find(("settings-accent-swatch", AccentColor::Blue as u32))
                        .is_none()
                );
            });
        });
    }

    #[gpui::test]
    fn test_experience_switch_reports_requested_value(cx: &mut gpui::TestAppContext) {
        use gpui::test::TestWindowExt as _;
        use std::cell::RefCell;

        struct SwitchHarness {
            theme: OrbitTheme,
            enabled: bool,
            requested: Rc<RefCell<Vec<bool>>>,
        }

        impl gpui::Render for SwitchHarness {
            fn render(
                &mut self,
                _window: &mut Window,
                _cx: &mut gpui::Context<Self>,
            ) -> impl IntoElement {
                let requested = Rc::clone(&self.requested);
                let cb: SettingsValueCallback<bool> = Rc::new(move |value, _, _| {
                    requested.borrow_mut().push(value);
                });
                div()
                    .p_4()
                    .child(render_experience_toggle(&self.theme, self.enabled, &cb))
            }
        }

        let theme = OrbitTheme::resolve(
            ThemeMode::Dark,
            UiStyle::Flat,
            AccentColor::Blue,
            SemanticColorsPreference::default(),
            WindowAppearance::Dark,
        );
        let requested = Rc::new(RefCell::new(Vec::new()));
        let requested_for_view = Rc::clone(&requested);

        let window = cx.update(|cx| {
            gpui::init(cx);
            crate::theme::kit_bridge::sync_kit_theme(&theme, cx);
            let options = gpui::WindowOptions::default();
            gpui::open_window(options, cx, move |_, cx| {
                cx.new(|_| SwitchHarness {
                    theme: theme.clone(),
                    enabled: false,
                    requested: requested_for_view,
                })
            })
        });
        let Ok((window, _view)) = window else {
            panic!("failed to open test window");
        };

        cx.update(|cx| {
            let _ = window.update(cx, |_, window, cx| {
                window.render_frame(cx);
                assert_eq!(
                    window.find("settings-experience-toggle").checked(),
                    Some(false)
                );
                window.click("settings-experience-toggle", cx);
                // Controlled component: remains unchecked until owner updates state.
                assert_eq!(
                    window.find("settings-experience-toggle").checked(),
                    Some(false)
                );
            });
        });

        assert_eq!(*requested.borrow(), vec![true]);
    }

    #[gpui::test]
    fn test_reset_defaults_button_disabled_until_modified(cx: &mut gpui::TestAppContext) {
        use gpui::test::TestWindowExt as _;
        use std::cell::Cell;

        struct ResetButtonHarness {
            theme: OrbitTheme,
            modified: bool,
            clicks: Rc<Cell<usize>>,
        }

        impl gpui::Render for ResetButtonHarness {
            fn render(
                &mut self,
                _window: &mut Window,
                _cx: &mut gpui::Context<Self>,
            ) -> impl IntoElement {
                let clicks = Rc::clone(&self.clicks);
                let on_reset: SettingsActionCallback = Rc::new(move |_, _| {
                    clicks.set(clicks.get() + 1);
                });
                div().p_4().child(render_settings_header(
                    &self.theme,
                    self.modified,
                    &on_reset,
                ))
            }
        }

        let theme = OrbitTheme::resolve(
            ThemeMode::Dark,
            UiStyle::Flat,
            AccentColor::Blue,
            SemanticColorsPreference::default(),
            WindowAppearance::Dark,
        );
        let clicks = Rc::new(Cell::new(0));
        let clicks_for_view = Rc::clone(&clicks);

        let window = cx.update(|cx| {
            gpui::init(cx);
            crate::theme::kit_bridge::sync_kit_theme(&theme, cx);
            gpui::open_window(gpui::WindowOptions::default(), cx, move |_, cx| {
                cx.new(|_| ResetButtonHarness {
                    theme: theme.clone(),
                    modified: false,
                    clicks: clicks_for_view,
                })
            })
        });
        let Ok((window, view)) = window else {
            panic!("failed to open test window");
        };

        cx.update(|cx| {
            let _ = window.update(cx, |_, window, cx| {
                window.render_frame(cx);
                assert_eq!(
                    window.find("settings-reset-appearance-btn").disabled(),
                    Some(true)
                );
                window.click("settings-reset-appearance-btn", cx);
            });
        });
        assert_eq!(clicks.get(), 0);

        cx.update(|cx| {
            view.update(cx, |harness, cx| {
                harness.modified = true;
                cx.notify();
            });
            let _ = window.update(cx, |_, window, cx| {
                window.render_frame(cx);
                assert_ne!(
                    window.find("settings-reset-appearance-btn").disabled(),
                    Some(true)
                );
                window.click("settings-reset-appearance-btn", cx);
            });
        });
        assert_eq!(clicks.get(), 1);
    }
}
