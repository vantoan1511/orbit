use iced::{
    border::Radius,
    widget::{button, container, pick_list, row, text},
    Background, Border, Color, Element, Shadow, Theme,
};
use orbit_core::command::ResourceKind;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct OrbitPalette {
    pub is_dark: bool,
    pub bg_app: Color,
    pub bg_sidebar: Color,
    pub bg_panel: Color,
    pub bg_card: Color,
    pub bg_hover: Color,
    pub border: Color,
    pub text_primary: Color,
    pub text_secondary: Color,
    pub text_muted: Color,
    pub accent: Color,
    pub success: Color,
    pub warning: Color,
    pub danger: Color,
    pub info: Color,
}

impl OrbitPalette {
    pub fn dark() -> Self {
        Self {
            is_dark: true,
            bg_app: Color::from_rgb(0.035, 0.035, 0.043),      // #09090b
            bg_sidebar: Color::from_rgb(0.071, 0.071, 0.082),  // #121215
            bg_panel: Color::from_rgb(0.094, 0.094, 0.106),    // #18181b
            bg_card: Color::from_rgb(0.071, 0.071, 0.082),     // #121215
            bg_hover: Color::from_rgb(0.133, 0.133, 0.149),    // #222226
            border: Color::from_rgb(0.153, 0.153, 0.165),      // #27272a
            text_primary: Color::from_rgb(0.95, 0.96, 0.98),   // #f2f5fa
            text_secondary: Color::from_rgb(0.65, 0.68, 0.74),
            text_muted: Color::from_rgb(0.45, 0.48, 0.54),
            accent: Color::from_rgb(0.42, 0.66, 1.0),         // #6aa8ff
            success: Color::from_rgb(0.20, 0.78, 0.44),       // #33c770
            warning: Color::from_rgb(0.95, 0.77, 0.25),       // #f2c440
            danger: Color::from_rgb(0.92, 0.30, 0.30),        // #eb4d4d
            info: Color::from_rgb(0.38, 0.65, 0.98),          // #61a6fa
        }
    }

    pub fn light() -> Self {
        Self {
            is_dark: false,
            bg_app: Color::from_rgb(0.98, 0.98, 0.99),
            bg_sidebar: Color::from_rgb(0.94, 0.95, 0.97),
            bg_panel: Color::from_rgb(1.0, 1.0, 1.0),
            bg_card: Color::from_rgb(1.0, 1.0, 1.0),
            bg_hover: Color::from_rgb(0.92, 0.93, 0.96),
            border: Color::from_rgb(0.85, 0.87, 0.91),
            text_primary: Color::from_rgb(0.08, 0.09, 0.11),
            text_secondary: Color::from_rgb(0.35, 0.38, 0.44),
            text_muted: Color::from_rgb(0.55, 0.58, 0.65),
            accent: Color::from_rgb(0.31, 0.55, 1.0),
            success: Color::from_rgb(0.15, 0.68, 0.38),
            warning: Color::from_rgb(0.85, 0.65, 0.15),
            danger: Color::from_rgb(0.85, 0.25, 0.25),
            info: Color::from_rgb(0.25, 0.55, 0.90),
        }
    }

    pub fn status_color(&self, status: &str) -> Color {
        match status.to_ascii_lowercase().as_str() {
            "running" | "active" | "completed" | "succeeded" | "true" | "ready" | "healthy" | "bound" => {
                self.success
            }
            "pending" | "waiting" | "terminating" | "containercreating" | "unknown" | "false" => {
                self.warning
            }
            "crashloopbackoff" | "failed" | "error" | "degraded" | "offline" | "evicted" => {
                self.danger
            }
            _ => self.info,
        }
    }
}

pub fn status_color(status: &str, palette: &OrbitPalette) -> Color {
    palette.status_color(status)
}

pub fn resource_kind_color(kind: ResourceKind) -> Color {
    match kind {
        ResourceKind::Pod => Color::from_rgb(0.31, 0.55, 1.0),            // Blue
        ResourceKind::Deployment => Color::from_rgb(0.20, 0.78, 0.44),     // Green
        ResourceKind::StatefulSet => Color::from_rgb(0.15, 0.68, 0.85),    // Teal
        ResourceKind::DaemonSet => Color::from_rgb(0.85, 0.45, 0.95),      // Purple
        ResourceKind::ReplicaSet => Color::from_rgb(0.40, 0.80, 0.70),     // Mint
        ResourceKind::Job => Color::from_rgb(0.95, 0.60, 0.20),            // Orange
        ResourceKind::CronJob => Color::from_rgb(0.95, 0.75, 0.20),        // Gold
        ResourceKind::HorizontalPodAutoscaler => Color::from_rgb(0.90, 0.40, 0.60), // Pink
        ResourceKind::Node => Color::from_rgb(0.30, 0.70, 0.90),           // Cyan
        ResourceKind::Service => Color::from_rgb(0.45, 0.50, 0.95),        // Indigo
        ResourceKind::Ingress => Color::from_rgb(0.80, 0.50, 0.90),        // Violet
        ResourceKind::ConfigMap => Color::from_rgb(0.60, 0.75, 0.30),      // Lime
        ResourceKind::Secret => Color::from_rgb(0.95, 0.40, 0.40),         // Coral
        ResourceKind::Event => Color::from_rgb(0.70, 0.70, 0.70),          // Gray
        ResourceKind::PersistentVolume => Color::from_rgb(0.75, 0.55, 0.40), // Brown
        ResourceKind::PersistentVolumeClaim => Color::from_rgb(0.65, 0.50, 0.35),
        ResourceKind::StorageClass => Color::from_rgb(0.55, 0.45, 0.35),
        ResourceKind::Policy => Color::from_rgb(0.85, 0.35, 0.55),
        ResourceKind::Namespace => Color::from_rgb(0.40, 0.60, 0.80),
    }
}

pub fn sidebar_nav_button_style<'a>(
    palette: &'a OrbitPalette,
    is_active: bool,
) -> impl Fn(&Theme, button::Status) -> button::Style + 'a {
    move |_theme, status| {
        let is_hovered = matches!(status, button::Status::Hovered | button::Status::Pressed);
        let bg = if is_active {
            Some(Background::Color(palette.bg_hover))
        } else if is_hovered {
            Some(Background::Color(Color::from_rgba(palette.bg_hover.r, palette.bg_hover.g, palette.bg_hover.b, 0.6)))
        } else {
            None
        };
        let text_color = if is_active || is_hovered {
            palette.text_primary
        } else {
            palette.text_secondary
        };
        let border = Border {
            color: if is_active { palette.accent } else { Color::TRANSPARENT },
            width: if is_active { 1.0 } else { 0.0 },
            radius: Radius::from(4.0),
        };
        button::Style {
            background: bg,
            text_color,
            border,
            shadow: Shadow::default(),
        }
    }
}

pub fn tab_button_style<'a>(
    palette: &'a OrbitPalette,
    is_active: bool,
) -> impl Fn(&Theme, button::Status) -> button::Style + 'a {
    move |_theme, status| {
        let is_hovered = matches!(status, button::Status::Hovered | button::Status::Pressed);
        let bg = if is_active {
            Some(Background::Color(palette.bg_card))
        } else if is_hovered {
            Some(Background::Color(palette.bg_hover))
        } else {
            None
        };
        let text_color = if is_active || is_hovered {
            palette.text_primary
        } else {
            palette.text_muted
        };
        let border = if is_active {
            Border {
                color: palette.border,
                width: 1.0,
                radius: Radius::from(4.0),
            }
        } else {
            Border::default()
        };
        button::Style {
            background: bg,
            text_color,
            border,
            shadow: Shadow::default(),
        }
    }
}

pub fn tab_close_button_style<'a>(
    palette: &'a OrbitPalette,
) -> impl Fn(&Theme, button::Status) -> button::Style + 'a {
    move |_theme, status| {
        let is_hovered = matches!(status, button::Status::Hovered | button::Status::Pressed);
        let bg = if is_hovered {
            Some(Background::Color(Color::from_rgba(palette.danger.r, palette.danger.g, palette.danger.b, 0.15)))
        } else {
            None
        };
        let text_color = if is_hovered {
            palette.danger
        } else {
            palette.text_muted
        };
        button::Style {
            background: bg,
            text_color,
            border: Border::default(),
            shadow: Shadow::default(),
        }
    }
}

pub fn search_pill_style<'a>(
    palette: &'a OrbitPalette,
) -> impl Fn(&Theme, button::Status) -> button::Style + 'a {
    move |_theme, status| {
        let is_hovered = matches!(status, button::Status::Hovered | button::Status::Pressed);
        let bg = if is_hovered {
            Some(Background::Color(palette.bg_hover))
        } else {
            Some(Background::Color(palette.bg_card))
        };
        let text_color = if is_hovered {
            palette.text_primary
        } else {
            palette.text_secondary
        };
        button::Style {
            background: bg,
            text_color,
            border: Border {
                color: palette.border,
                width: 1.0,
                radius: Radius::from(6.0),
            },
            shadow: Shadow::default(),
        }
    }
}

pub fn pick_list_style<'a>(
    palette: &'a OrbitPalette,
) -> impl Fn(&Theme, pick_list::Status) -> pick_list::Style + 'a {
    move |_theme, status| {
        let is_hovered = matches!(status, pick_list::Status::Hovered | pick_list::Status::Opened);
        let border_color = if is_hovered { palette.accent } else { palette.border };
        pick_list::Style {
            text_color: palette.text_primary,
            placeholder_color: palette.text_muted,
            handle_color: if is_hovered { palette.text_primary } else { palette.text_secondary },
            background: Background::Color(palette.bg_card),
            border: Border {
                color: border_color,
                width: 1.0,
                radius: Radius::from(4.0),
            },
        }
    }
}

pub fn table_row_style<'a>(
    palette: &'a OrbitPalette,
) -> impl Fn(&Theme, button::Status) -> button::Style + 'a {
    move |_theme, status| {
        let is_hovered = matches!(status, button::Status::Hovered | button::Status::Pressed);
        let bg = if is_hovered {
            Some(Background::Color(palette.bg_hover))
        } else {
            None
        };
        button::Style {
            background: bg,
            text_color: palette.text_primary,
            border: Border::default(),
            shadow: Shadow::default(),
        }
    }
}

pub fn action_button_style<'a>(
    palette: &'a OrbitPalette,
    is_destructive: bool,
) -> impl Fn(&Theme, button::Status) -> button::Style + 'a {
    move |_theme, status| {
        let is_hovered = matches!(status, button::Status::Hovered | button::Status::Pressed);
        let (bg, text_color, border_color) = if is_destructive {
            if is_hovered {
                (
                    Some(Background::Color(Color::from_rgba(palette.danger.r, palette.danger.g, palette.danger.b, 0.2))),
                    palette.danger,
                    palette.danger,
                )
            } else {
                (
                    Some(Background::Color(palette.bg_card)),
                    palette.danger,
                    Color::from_rgba(palette.danger.r, palette.danger.g, palette.danger.b, 0.4),
                )
            }
        } else if is_hovered {
            (
                Some(Background::Color(palette.bg_hover)),
                palette.text_primary,
                palette.accent,
            )
        } else {
            (
                Some(Background::Color(palette.bg_card)),
                palette.text_primary,
                palette.border,
            )
        };
        button::Style {
            background: bg,
            text_color,
            border: Border {
                color: border_color,
                width: 1.0,
                radius: Radius::from(4.0),
            },
            shadow: Shadow::default(),
        }
    }
}

pub fn header_container_style<'a>(
    palette: &'a OrbitPalette,
) -> impl Fn(&Theme) -> container::Style + 'a {
    move |_theme| container::Style {
        text_color: Some(palette.text_primary),
        background: Some(Background::Color(palette.bg_panel)),
        border: Border {
            color: palette.border,
            width: 1.0,
            radius: Radius::default(),
        },
        shadow: Shadow::default(),
    }
}

pub fn sidebar_container_style<'a>(
    palette: &'a OrbitPalette,
) -> impl Fn(&Theme) -> container::Style + 'a {
    move |_theme| container::Style {
        text_color: Some(palette.text_primary),
        background: Some(Background::Color(palette.bg_sidebar)),
        border: Border {
            color: palette.border,
            width: 1.0,
            radius: Radius::default(),
        },
        shadow: Shadow::default(),
    }
}

pub fn drawer_container_style<'a>(
    palette: &'a OrbitPalette,
) -> impl Fn(&Theme) -> container::Style + 'a {
    move |_theme| container::Style {
        text_color: Some(palette.text_primary),
        background: Some(Background::Color(palette.bg_sidebar)),
        border: Border {
            color: palette.border,
            width: 1.0,
            radius: Radius::default(),
        },
        shadow: Shadow::default(),
    }
}

pub fn card_container_style<'a>(
    palette: &'a OrbitPalette,
) -> impl Fn(&Theme) -> container::Style + 'a {
    move |_theme| container::Style {
        text_color: Some(palette.text_primary),
        background: Some(Background::Color(palette.bg_card)),
        border: Border {
            color: palette.border,
            width: 1.0,
            radius: Radius::from(6.0),
        },
        shadow: Shadow::default(),
    }
}

pub fn status_pill<'a, Message: 'a>(palette: &'a OrbitPalette, status: &str) -> Element<'a, Message> {
    let color = palette.status_color(status);
    let border_color = Color::from_rgba(color.r, color.g, color.b, 0.35);
    let fill_color = Color::from_rgba(color.r, color.g, color.b, 0.12);

    container(
        row![
            text("●").size(10).color(color),
            text(status.to_string()).size(11).color(palette.text_primary),
        ]
        .spacing(4)
        .align_y(iced::Alignment::Center)
        .padding([2, 6]),
    )
    .style(move |_theme| container::Style {
        text_color: Some(palette.text_primary),
        background: Some(Background::Color(fill_color)),
        border: Border {
            color: border_color,
            width: 1.0,
            radius: Radius::from(10.0),
        },
        shadow: Shadow::default(),
    })
    .into()
}

pub fn status_dot_pill<'a, Message: 'a>(
    palette: &'a OrbitPalette,
    is_connected: bool,
    is_loading: bool,
    label: &str,
) -> Element<'a, Message> {
    let (dot_color, symbol) = if is_loading {
        (palette.warning, "●")
    } else if is_connected {
        (palette.success, "●")
    } else {
        (palette.danger, "○")
    };

    container(
        row![
            text(symbol).size(10).color(dot_color),
            text(label.to_string()).size(11).color(palette.text_primary),
        ]
        .spacing(4)
        .align_y(iced::Alignment::Center)
        .padding([3, 8]),
    )
    .style(move |_theme| container::Style {
        text_color: Some(palette.text_primary),
        background: Some(Background::Color(palette.bg_card)),
        border: Border {
            color: palette.border,
            width: 1.0,
            radius: Radius::from(10.0),
        },
        shadow: Shadow::default(),
    })
    .into()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_noir_palette_exact_colors_and_style_helpers() {
        let dark = OrbitPalette::dark();
        assert!(dark.is_dark);

        // Slate/Zinc exact dark tokens
        assert_eq!(dark.bg_app, Color::from_rgb(0.035, 0.035, 0.043));
        assert_eq!(dark.bg_sidebar, Color::from_rgb(0.071, 0.071, 0.082));
        assert_eq!(dark.bg_panel, Color::from_rgb(0.094, 0.094, 0.106));
        assert_eq!(dark.border, Color::from_rgb(0.153, 0.153, 0.165));
        assert_eq!(dark.bg_hover, Color::from_rgb(0.133, 0.133, 0.149));

        // Style helper closures return without panic
        let nav_active = sidebar_nav_button_style(&dark, true)(&iced::Theme::Dark, iced::widget::button::Status::Active);
        let nav_hover = sidebar_nav_button_style(&dark, false)(&iced::Theme::Dark, iced::widget::button::Status::Hovered);
        assert!(nav_active.background.is_some());
        assert!(nav_hover.background.is_some());

        let tab_style = tab_button_style(&dark, true)(&iced::Theme::Dark, iced::widget::button::Status::Active);
        assert!(tab_style.background.is_some());

        let tab_close = tab_close_button_style(&dark)(&iced::Theme::Dark, iced::widget::button::Status::Hovered);
        assert_eq!(tab_close.text_color, dark.danger);

        let search_style = search_pill_style(&dark)(&iced::Theme::Dark, iced::widget::button::Status::Active);
        assert_eq!(search_style.border.radius, iced::border::Radius::from(6.0));

        let pick_style = pick_list_style(&dark)(&iced::Theme::Dark, iced::widget::pick_list::Status::Active);
        assert_eq!(pick_style.text_color, dark.text_primary);

        let row_style = table_row_style(&dark)(&iced::Theme::Dark, iced::widget::button::Status::Active);
        assert!(row_style.background.is_none() || row_style.background == Some(iced::Background::Color(Color::TRANSPARENT)));

        let action_normal = action_button_style(&dark, false)(&iced::Theme::Dark, iced::widget::button::Status::Active);
        let action_destruct = action_button_style(&dark, true)(&iced::Theme::Dark, iced::widget::button::Status::Active);
        assert_eq!(action_destruct.text_color, dark.danger);
        assert_ne!(action_normal.text_color, dark.danger);

        let header_style = header_container_style(&dark)(&iced::Theme::Dark);
        assert_eq!(header_style.background, Some(iced::Background::Color(dark.bg_panel)));

        let sidebar_style = sidebar_container_style(&dark)(&iced::Theme::Dark);
        assert_eq!(sidebar_style.background, Some(iced::Background::Color(dark.bg_sidebar)));

        let card_style = card_container_style(&dark)(&iced::Theme::Dark);
        assert_eq!(card_style.background, Some(iced::Background::Color(dark.bg_card)));

        // Status pill elements render without panic
        let _pill: iced::Element<'_, ()> = status_pill(&dark, "Running");
        let _dot_pill: iced::Element<'_, ()> = status_dot_pill(&dark, true, false, "Connected");
    }

    #[test]
    fn test_noir_dark_and_light_palettes_contrast_and_semantic_mapping() {
        let dark = OrbitPalette::dark();
        let light = OrbitPalette::light();

        assert!(dark.is_dark);
        assert!(!light.is_dark);
        assert_ne!(dark.bg_app, light.bg_app);
        assert_ne!(dark.text_primary, light.text_primary);

        assert_eq!(dark.status_color("Running"), dark.success);
        assert_eq!(dark.status_color("Pending"), dark.warning);
        assert_eq!(dark.status_color("CrashLoopBackOff"), dark.danger);
        assert_eq!(dark.status_color("CustomOther"), dark.info);
    }

    #[test]
    fn test_resource_kind_color_mapping() {
        let all_kinds = [
            ResourceKind::Pod,
            ResourceKind::Deployment,
            ResourceKind::StatefulSet,
            ResourceKind::DaemonSet,
            ResourceKind::ReplicaSet,
            ResourceKind::Job,
            ResourceKind::CronJob,
            ResourceKind::HorizontalPodAutoscaler,
            ResourceKind::Node,
            ResourceKind::Service,
            ResourceKind::Ingress,
            ResourceKind::ConfigMap,
            ResourceKind::Secret,
            ResourceKind::Event,
            ResourceKind::PersistentVolume,
            ResourceKind::PersistentVolumeClaim,
            ResourceKind::StorageClass,
            ResourceKind::Policy,
            ResourceKind::Namespace,
        ];

        for kind in all_kinds {
            let color = resource_kind_color(kind);
            assert_eq!(color.a, 1.0, "Resource color for {:?} must be opaque", kind);
        }
    }
}
