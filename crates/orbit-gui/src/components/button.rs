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
        match self {
            Self::Small | Self::Medium => 4.0,
            Self::Large => 6.0,
        }
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
                ButtonVariant::Ghost => None,
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

        let is_hovered = matches!(status, button::Status::Hovered | button::Status::Pressed);

        match variant {
            ButtonVariant::Primary => {
                let bg = if is_hovered {
                    Color::from_rgba(palette.accent.r, palette.accent.g, palette.accent.b, 0.85)
                } else {
                    palette.accent
                };
                button::Style {
                    background: Some(Background::Color(bg)),
                    text_color: Color::WHITE,
                    border: Border {
                        color: palette.accent,
                        width: 1.0,
                        radius,
                    },
                    shadow: Shadow::default(),
                }
            }
            ButtonVariant::Secondary => {
                let (bg, border_color) = if is_hovered {
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
            ButtonVariant::Ghost => {
                let (bg, text_color, border) = if is_hovered {
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
                let (bg, text_color, border_color) = if is_hovered {
                    (palette.danger, Color::WHITE, palette.danger)
                } else {
                    (
                        Color::from_rgba(palette.danger.r, palette.danger.g, palette.danger.b, 0.15),
                        palette.danger,
                        Color::from_rgba(palette.danger.r, palette.danger.g, palette.danger.b, 0.35),
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
                    shadow: Shadow::default(),
                }
            }
            ButtonVariant::IconOnly => {
                let (bg, text_color, border_color) = if is_hovered {
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
}
