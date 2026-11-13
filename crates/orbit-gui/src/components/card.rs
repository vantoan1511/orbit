use iced::{
    Background, Border, Color, Element, Length,
    border::Radius,
    widget::{column, container, row, text},
};

use crate::icons::{Icon, icon};
use crate::theme::{
    FONT_UI_BOLD, FONT_UI_SEMIBOLD, OrbitPalette, Typography, card_container_style,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AlertLevel {
    Info,
    Success,
    Warning,
    Danger,
}

impl AlertLevel {
    pub fn color(&self, palette: &OrbitPalette) -> Color {
        match self {
            Self::Info => palette.info,
            Self::Success => palette.success,
            Self::Warning => palette.warning,
            Self::Danger => palette.danger,
        }
    }

    pub fn icon(&self) -> Icon {
        match self {
            Self::Info => Icon::Info,
            Self::Success => Icon::Check,
            Self::Warning => Icon::TriangleAlert,
            Self::Danger => Icon::CircleAlert,
        }
    }
}

pub fn alert_banner<'a, Message: 'a>(
    level: AlertLevel,
    title: impl Into<String>,
    description: impl Into<String>,
    palette: &'a OrbitPalette,
) -> Element<'a, Message> {
    let accent_color = level.color(palette);
    let bg_color = Color::from_rgba(accent_color.r, accent_color.g, accent_color.b, 0.08);

    let icon_el = icon(level.icon(), 16, accent_color);
    let title_el = text(title.into())
        .size(Typography::BODY)
        .font(FONT_UI_SEMIBOLD)
        .color(palette.text_primary);
    let desc_el = text(description.into())
        .size(Typography::SECONDARY)
        .color(palette.text_secondary);

    let text_col = column![title_el, desc_el].spacing(2).width(Length::Fill);

    let content = row![icon_el, text_col]
        .spacing(12)
        .align_y(iced::Alignment::Start)
        .width(Length::Fill);

    container(content)
        .padding(12)
        .width(Length::Fill)
        .style(move |_theme| container::Style {
            background: Some(Background::Color(bg_color)),
            border: Border {
                color: Color::TRANSPARENT,
                width: 0.0,
                radius: Radius::from(8.0),
            },
            ..Default::default()
        })
        .into()
}

pub fn container_card<'a, Message: 'a>(
    content: impl Into<Element<'a, Message>>,
    palette: &'a OrbitPalette,
) -> Element<'a, Message> {
    container(content)
        .style(card_container_style(palette))
        .padding(16)
        .width(Length::Fill)
        .into()
}

pub fn metric_label(label: &str) -> String {
    label.to_uppercase()
}

pub fn metric_card<'a, Message: 'a>(
    title: impl Into<String>,
    value: impl Into<String>,
    subtitle: impl Into<String>,
    watermark_icon: Option<Icon>,
    palette: &'a OrbitPalette,
) -> Element<'a, Message> {
    let title_str = title.into();
    let text_col = column![
        text(metric_label(&title_str))
            .size(Typography::CAPTION)
            .color(palette.text_secondary)
            .font(FONT_UI_SEMIBOLD),
        text(value.into())
            .size(Typography::METRIC)
            .color(palette.text_primary)
            .font(FONT_UI_BOLD),
        text(subtitle.into())
            .size(Typography::SECONDARY)
            .color(palette.text_muted),
    ]
    .spacing(6)
    .width(Length::Fill);

    let card_content = if let Some(ic) = watermark_icon {
        let icon_el = icon(ic, 20, palette.icon_watermark());
        row![text_col, icon_el]
            .spacing(8)
            .align_y(iced::Alignment::Start)
    } else {
        row![text_col]
    };

    container(card_content)
        .padding(16)
        .width(Length::FillPortion(1))
        .style(card_container_style(palette))
        .into()
}

pub fn section_card<'a, Message: 'a + Clone>(
    title: impl Into<String>,
    subtitle: impl Into<String>,
    rows: Vec<Element<'a, Message>>,
    palette: &'a OrbitPalette,
) -> Element<'a, Message> {
    let header = column![
        text(title.into())
            .size(Typography::SUBHEADING)
            .font(FONT_UI_SEMIBOLD)
            .color(palette.text_primary),
        text(subtitle.into())
            .size(Typography::SECONDARY)
            .color(palette.text_secondary),
    ]
    .spacing(2);

    let mut content = column![header].spacing(6);
    for row_el in rows {
        content = content.push(row_el);
    }

    container(content)
        .style(card_container_style(palette))
        .padding(16)
        .width(Length::Fill)
        .into()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_icon_watermark_visible_opacity() {
        let dark = OrbitPalette::dark();
        let light = OrbitPalette::light();

        for palette in [&dark, &light] {
            let a = palette.icon_watermark().a;
            assert!(
                (0.40..=0.50).contains(&a),
                "watermark alpha {} out of [0.40, 0.50]",
                a
            );
        }
    }

    #[test]
    fn test_metric_label_uppercased() {
        assert_eq!(metric_label("Nodes"), "NODES");
        assert_eq!(metric_label("Active Pods"), "ACTIVE PODS");
        assert_eq!(metric_label("deployments"), "DEPLOYMENTS");
    }

    #[test]
    fn test_metric_card_rendering() {
        let palette = OrbitPalette::dark();
        let _card_with_icon: Element<'_, ()> = metric_card(
            "NODES",
            "3/3",
            "Ready / Total",
            Some(Icon::Server),
            &palette,
        );
        let _card_no_icon: Element<'_, ()> =
            metric_card("DEPLOYMENTS", "12", "Active", None, &palette);
    }

    #[test]
    fn test_section_card_rendering() {
        let palette = OrbitPalette::dark();
        let rows: Vec<Element<'_, ()>> = vec![text("Row 1").into(), text("Row 2").into()];
        let _section: Element<'_, ()> =
            section_card("Section Title", "Section Subtitle", rows, &palette);
        let _container: Element<'_, ()> = container_card(text("Content"), &palette);
    }

    #[test]
    fn test_alert_banner_rendering() {
        let dark = OrbitPalette::dark();
        let light = OrbitPalette::light();

        for palette in [&dark, &light] {
            let _info: Element<'_, ()> =
                alert_banner(AlertLevel::Info, "Note", "Cluster state synced", palette);
            let _success: Element<'_, ()> =
                alert_banner(AlertLevel::Success, "Success", "Deployment scaled", palette);
            let _warning: Element<'_, ()> =
                alert_banner(AlertLevel::Warning, "Warning", "High memory usage", palette);
            let _danger: Element<'_, ()> =
                alert_banner(AlertLevel::Danger, "Error", "Pod crashed", palette);
        }
    }

    #[test]
    fn test_card_elevation_and_radii() {
        let dark = OrbitPalette::dark();
        let light = OrbitPalette::light();

        let card_dark = card_container_style(&dark)(&iced::Theme::Dark);
        assert_eq!(card_dark.border.radius, Radius::from(8.0));
        assert_eq!(card_dark.border.width, 0.0);
        assert_eq!(card_dark.border.color, Color::TRANSPARENT);
        assert_eq!(card_dark.background, Some(Background::Color(dark.bg_card)));
        assert_eq!(card_dark.shadow, dark.card_shadow());

        let card_light = card_container_style(&light)(&iced::Theme::Light);
        assert_eq!(card_light.border.radius, Radius::from(8.0));
        assert_eq!(card_light.border.width, 0.0);
        assert_eq!(card_light.border.color, Color::TRANSPARENT);
        assert_eq!(card_light.shadow, light.card_shadow());
        assert_eq!(card_light.shadow.blur_radius, 3.0);
        if let Some(iced::Background::Color(bg)) = card_light.background {
            assert_eq!(bg.a, 1.0);
        } else {
            panic!("Expected background color for light card");
        }
    }
}
