use iced::{
    border::Radius,
    widget::{button, container, text},
    Background, Border, Color, Element, Length, Shadow, Theme,
};

use crate::theme::OrbitPalette;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ButtonVariant {
    Primary,
    Secondary,
    Outline,
    Ghost,
    Destructive,
    IconOnly,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ButtonSize {
    Small,
    Medium,
    Large,
}

impl ButtonSize {
    pub fn text_size(&self) -> u16 {
        match self {
            Self::Small => 11,
            Self::Medium => 13,
            Self::Large => 14,
        }
    }

    pub fn padding(&self) -> [u16; 2] {
        match self {
            Self::Small => [3, 8],
            Self::Medium => [6, 12],
            Self::Large => [8, 16],
        }
    }

    pub fn radius(&self) -> f32 {
        6.0
    }
}

pub fn button_style<'a>(
    variant: ButtonVariant,
    size: ButtonSize,
    palette: &'a OrbitPalette,
    is_disabled: bool,
) -> impl Fn(&Theme, button::Status) -> button::Style + 'a {
    move |_theme, status| {
        let radius = Radius::from(size.radius());
        if is_disabled {
            let bg = match variant {
                ButtonVariant::Ghost | ButtonVariant::Outline => None,
                _ => Some(Background::Color(Color::from_rgba(
                    palette.bg_hover.r,
                    palette.bg_hover.g,
                    palette.bg_hover.b,
                    0.3,
                ))),
            };
            return button::Style {
                background: bg,
                text_color: palette.text_muted,
                border: Border {
                    color: Color::from_rgba(
                        palette.border.r,
                        palette.border.g,
                        palette.border.b,
                        0.4,
                    ),
                    width: 1.0,
                    radius,
                },
                shadow: Shadow::default(),
            };
        }

        let is_pressed = matches!(status, button::Status::Pressed);
        let is_hovered = matches!(status, button::Status::Hovered);
        let pressed_bg = palette.pressed_bg();

        match variant {
            ButtonVariant::Primary => {
                let (bg, shadow) = if is_pressed {
                    (
                        Color::from_rgba(palette.accent.r, palette.accent.g, palette.accent.b, 0.75),
                        palette.primary_button_shadow(true),
                    )
                } else if is_hovered {
                    (
                        Color::from_rgba(palette.accent.r, palette.accent.g, palette.accent.b, 0.90),
                        palette.primary_button_shadow(false),
                    )
                } else {
                    (palette.accent, palette.primary_button_shadow(false))
                };
                button::Style {
                    background: Some(Background::Color(bg)),
                    text_color: Color::WHITE,
                    border: Border {
                        color: palette.accent,
                        width: 1.0,
                        radius,
                    },
                    shadow,
                }
            }
            ButtonVariant::Secondary => {
                let (bg, border_color) = if is_pressed {
                    (pressed_bg, palette.accent)
                } else if is_hovered {
                    (palette.bg_hover, palette.accent)
                } else {
                    (palette.bg_card, palette.border)
                };
                button::Style {
                    background: Some(Background::Color(bg)),
                    text_color: palette.text_primary,
                    border: Border {
                        color: border_color,
                        width: 1.0,
                        radius,
                    },
                    shadow: Shadow::default(),
                }
            }
            ButtonVariant::Outline => {
                let (bg, border_color) = if is_pressed {
                    (Some(Background::Color(pressed_bg)), palette.accent)
                } else if is_hovered {
                    (Some(Background::Color(palette.bg_hover)), palette.accent)
                } else {
                    (None, palette.border)
                };
                button::Style {
                    background: bg,
                    text_color: palette.text_primary,
                    border: Border {
                        color: border_color,
                        width: 1.0,
                        radius,
                    },
                    shadow: Shadow::default(),
                }
            }
            ButtonVariant::Ghost => {
                let (bg, text_color, border) = if is_pressed {
                    (
                        Some(Background::Color(pressed_bg)),
                        palette.text_primary,
                        Border {
                            color: palette.accent,
                            width: 1.0,
                            radius,
                        },
                    )
                } else if is_hovered {
                    (
                        Some(Background::Color(palette.bg_hover)),
                        palette.text_primary,
                        Border {
                            color: palette.border,
                            width: 1.0,
                            radius,
                        },
                    )
                } else {
                    (
                        None,
                        palette.text_secondary,
                        Border {
                            radius,
                            ..Default::default()
                        },
                    )
                };
                button::Style {
                    background: bg,
                    text_color,
                    border,
                    shadow: Shadow::default(),
                }
            }
            ButtonVariant::Destructive => {
                let (bg, text_color, border_color, shadow) = if is_pressed {
                    (
                        Color::from_rgba(palette.danger.r, palette.danger.g, palette.danger.b, 0.75),
                        Color::WHITE,
                        palette.danger,
                        Shadow {
                            color: Color::from_rgba(palette.danger.r, palette.danger.g, palette.danger.b, 0.20),
                            offset: iced::Vector::new(0.0, 1.0),
                            blur_radius: 2.0,
                        },
                    )
                } else if is_hovered {
                    (
                        palette.danger,
                        Color::WHITE,
                        palette.danger,
                        Shadow {
                            color: Color::from_rgba(palette.danger.r, palette.danger.g, palette.danger.b, 0.35),
                            offset: iced::Vector::new(0.0, 2.0),
                            blur_radius: 4.0,
                        },
                    )
                } else {
                    (
                        Color::from_rgba(palette.danger.r, palette.danger.g, palette.danger.b, 0.15),
                        palette.danger,
                        Color::from_rgba(palette.danger.r, palette.danger.g, palette.danger.b, 0.35),
                        Shadow::default(),
                    )
                };
                button::Style {
                    background: Some(Background::Color(bg)),
                    text_color,
                    border: Border {
                        color: border_color,
                        width: 1.0,
                        radius,
                    },
                    shadow,
                }
            }
            ButtonVariant::IconOnly => {
                let (bg, text_color, border_color) = if is_pressed {
                    (Some(Background::Color(pressed_bg)), palette.text_primary, palette.accent)
                } else if is_hovered {
                    (Some(Background::Color(palette.bg_hover)), palette.text_primary, palette.accent)
                } else {
                    (None, palette.text_secondary, palette.border)
                };
                button::Style {
                    background: bg,
                    text_color,
                    border: Border {
                        color: border_color,
                        width: 1.0,
                        radius,
                    },
                    shadow: Shadow::default(),
                }
            }
        }
    }
}

pub fn custom_button<'a, Message: 'a + Clone>(
    content: impl Into<Element<'a, Message>>,
    variant: ButtonVariant,
    size: ButtonSize,
    on_press: Option<Message>,
    palette: &'a OrbitPalette,
) -> Element<'a, Message> {
    let is_disabled = on_press.is_none();
    let mut btn = button(
        container(content)
            .center_x(Length::Shrink)
            .center_y(Length::Shrink),
    )
    .padding(size.padding())
    .style(button_style(variant, size, palette, is_disabled));

    if let Some(msg) = on_press {
        btn = btn.on_press(msg);
    }

    btn.into()
}

pub fn primary_button<'a, Message: 'a + Clone>(
    label: impl Into<String>,
    on_press: Option<Message>,
    palette: &'a OrbitPalette,
) -> Element<'a, Message> {
    custom_button(
        text(label.into()).size(ButtonSize::Medium.text_size()),
        ButtonVariant::Primary,
        ButtonSize::Medium,
        on_press,
        palette,
    )
}

pub fn secondary_button<'a, Message: 'a + Clone>(
    label: impl Into<String>,
    on_press: Option<Message>,
    palette: &'a OrbitPalette,
) -> Element<'a, Message> {
    custom_button(
        text(label.into()).size(ButtonSize::Medium.text_size()),
        ButtonVariant::Secondary,
        ButtonSize::Medium,
        on_press,
        palette,
    )
}

pub fn outline_button<'a, Message: 'a + Clone>(
    label: impl Into<String>,
    on_press: Option<Message>,
    palette: &'a OrbitPalette,
) -> Element<'a, Message> {
    custom_button(
        text(label.into()).size(ButtonSize::Medium.text_size()),
        ButtonVariant::Outline,
        ButtonSize::Medium,
        on_press,
        palette,
    )
}

pub fn ghost_button<'a, Message: 'a + Clone>(
    label: impl Into<String>,
    on_press: Option<Message>,
    palette: &'a OrbitPalette,
) -> Element<'a, Message> {
    custom_button(
        text(label.into()).size(ButtonSize::Medium.text_size()),
        ButtonVariant::Ghost,
        ButtonSize::Medium,
        on_press,
        palette,
    )
}

pub fn destructive_button<'a, Message: 'a + Clone>(
    label: impl Into<String>,
    on_press: Option<Message>,
    palette: &'a OrbitPalette,
) -> Element<'a, Message> {
    custom_button(
        text(label.into()).size(ButtonSize::Medium.text_size()),
        ButtonVariant::Destructive,
        ButtonSize::Medium,
        on_press,
        palette,
    )
}

pub fn icon_button<'a, Message: 'a + Clone>(
    icon_el: impl Into<Element<'a, Message>>,
    size: ButtonSize,
    on_press: Option<Message>,
    palette: &'a OrbitPalette,
) -> Element<'a, Message> {
    custom_button(icon_el, ButtonVariant::IconOnly, size, on_press, palette)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_button_variants_and_states() {
        let palette = OrbitPalette::dark();
        let variants = [
            ButtonVariant::Primary,
            ButtonVariant::Secondary,
            ButtonVariant::Outline,
            ButtonVariant::Ghost,
            ButtonVariant::Destructive,
            ButtonVariant::IconOnly,
        ];

        for variant in variants {
            let style_fn = button_style(variant, ButtonSize::Medium, &palette, false);
            let normal = style_fn(&Theme::Dark, button::Status::Active);
            let hovered = style_fn(&Theme::Dark, button::Status::Hovered);

            let large_style_fn = button_style(variant, ButtonSize::Large, &palette, false);
            let large_normal = large_style_fn(&Theme::Dark, button::Status::Active);
            assert_eq!(large_normal.border.radius, Radius::from(6.0));

            match variant {
                ButtonVariant::Primary => {
                    assert_eq!(normal.text_color, Color::WHITE);
                    assert_eq!(hovered.text_color, Color::WHITE);
                }
                ButtonVariant::Secondary => {
                    assert_eq!(normal.text_color, palette.text_primary);
                    assert_eq!(normal.border.color, palette.border);
                    assert_eq!(hovered.border.color, palette.accent);
                }
                ButtonVariant::Outline => {
                    assert_eq!(normal.text_color, palette.text_primary);
                    assert_eq!(normal.border.color, palette.border);
                    assert_eq!(hovered.border.color, palette.accent);
                }
                ButtonVariant::Ghost => {
                    assert_eq!(normal.text_color, palette.text_secondary);
                    assert_eq!(hovered.text_color, palette.text_primary);
                }
                ButtonVariant::Destructive => {
                    assert_eq!(normal.text_color, palette.danger);
                    assert_eq!(hovered.text_color, Color::WHITE);
                }
                ButtonVariant::IconOnly => {
                    assert_eq!(normal.text_color, palette.text_secondary);
                    assert_eq!(hovered.text_color, palette.text_primary);
                }
            }
        }
    }

    #[test]
    fn test_disabled_button_styling() {
        let palette = OrbitPalette::dark();
        let style_fn = button_style(ButtonVariant::Primary, ButtonSize::Medium, &palette, true);
        let normal = style_fn(&Theme::Dark, button::Status::Active);
        let hovered = style_fn(&Theme::Dark, button::Status::Hovered);

        assert_eq!(normal.text_color, palette.text_muted);
        assert_eq!(hovered.text_color, palette.text_muted);

        let _btn: Element<'_, ()> = custom_button("test", ButtonVariant::Primary, ButtonSize::Small, None, &palette);
        let _prim: Element<'_, ()> = primary_button("ok", Some(()), &palette);
        let _sec: Element<'_, ()> = secondary_button("cancel", Some(()), &palette);
        let _ghost: Element<'_, ()> = ghost_button("edit", Some(()), &palette);
        let _destr: Element<'_, ()> = destructive_button("delete", Some(()), &palette);
        let _ico: Element<'_, ()> = icon_button(text("X"), ButtonSize::Small, Some(()), &palette);
    }

    #[test]
    fn test_outline_button_styling() {
        let palette = OrbitPalette::dark();
        let style_fn = button_style(ButtonVariant::Outline, ButtonSize::Medium, &palette, false);
        let normal = style_fn(&Theme::Dark, button::Status::Active);
        let hovered = style_fn(&Theme::Dark, button::Status::Hovered);

        assert_eq!(normal.background, None);
        assert_eq!(normal.text_color, palette.text_primary);
        assert_eq!(normal.border.color, palette.border);

        assert_eq!(hovered.background, Some(Background::Color(palette.bg_hover)));
        assert_eq!(hovered.text_color, palette.text_primary);
        assert_eq!(hovered.border.color, palette.accent);

        let disabled_style_fn = button_style(ButtonVariant::Outline, ButtonSize::Medium, &palette, true);
        let disabled = disabled_style_fn(&Theme::Dark, button::Status::Active);
        assert_eq!(disabled.text_color, palette.text_muted);
        assert_eq!(disabled.background, None);

        let _outline: Element<'_, ()> = outline_button("outline", Some(()), &palette);
    }

    #[test]
    fn test_button_micro_interactions_and_standardized_radius() {
        let palette = OrbitPalette::dark();

        // 1. Standardized 6.0px radii for all sizes
        assert_eq!(ButtonSize::Small.radius(), 6.0);
        assert_eq!(ButtonSize::Medium.radius(), 6.0);
        assert_eq!(ButtonSize::Large.radius(), 6.0);

        // 2. Primary button micro-interactions
        let primary_style = button_style(ButtonVariant::Primary, ButtonSize::Medium, &palette, false);
        let normal = primary_style(&Theme::Dark, button::Status::Active);
        let hovered = primary_style(&Theme::Dark, button::Status::Hovered);
        let pressed = primary_style(&Theme::Dark, button::Status::Pressed);

        assert_eq!(normal.shadow.blur_radius, 6.0);
        assert_eq!(normal.shadow.offset.y, 2.0);
        assert_eq!(hovered.shadow.blur_radius, 6.0);
        assert_eq!(pressed.shadow.blur_radius, 3.0);
        assert_eq!(pressed.shadow.offset.y, 1.0);

        // Pressed background has deeper/different tint than normal
        assert_ne!(pressed.background, normal.background);

        // 3. Secondary & Outline micro-interactions
        let sec_style = button_style(ButtonVariant::Secondary, ButtonSize::Medium, &palette, false);
        let sec_hovered = sec_style(&Theme::Dark, button::Status::Hovered);
        let sec_pressed = sec_style(&Theme::Dark, button::Status::Pressed);

        if let (Some(Background::Color(h_bg)), Some(Background::Color(p_bg))) =
            (sec_hovered.background, sec_pressed.background)
        {
            assert!(p_bg.r < h_bg.r);
        } else {
            panic!("Secondary hovered and pressed must have solid backgrounds");
        }

        let out_style = button_style(ButtonVariant::Outline, ButtonSize::Medium, &palette, false);
        let out_hovered = out_style(&Theme::Dark, button::Status::Hovered);
        let out_pressed = out_style(&Theme::Dark, button::Status::Pressed);

        if let (Some(Background::Color(h_bg)), Some(Background::Color(p_bg))) =
            (out_hovered.background, out_pressed.background)
        {
            assert!(p_bg.r < h_bg.r);
        } else {
            panic!("Outline hovered and pressed must have solid backgrounds");
        }
    }
}
