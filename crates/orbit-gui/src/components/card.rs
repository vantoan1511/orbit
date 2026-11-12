use iced::{
    border::Radius,
    widget::{column, container, row, text},
    Background, Border, Color, Element, Length,
};

use crate::icons::{icon, Icon};
use crate::theme::{card_container_style, OrbitPalette};
use super::form_row::form_divider;

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
    let border_color = Color::from_rgba(accent_color.r, accent_color.g, accent_color.b, 0.4);
    let bg_color = Color::from_rgba(accent_color.r, accent_color.g, accent_color.b, 0.08);

    let icon_el = icon(level.icon(), 16, accent_color);
    let title_el = text(title.into()).size(13).color(palette.text_primary);
    let desc_el = text(description.into()).size(12).color(palette.text_secondary);

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
                color: border_color,
                width: 1.0,
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

pub fn metric_card<'a, Message: 'a>(
    title: impl Into<String>,
    value: impl Into<String>,
    subtitle: impl Into<String>,
    watermark_icon: Option<Icon>,
    palette: &'a OrbitPalette,
) -> Element<'a, Message> {
    let text_col = column![
        text(title.into()).size(12).color(palette.text_secondary),
        text(value.into()).size(24).color(palette.text_primary),
        text(subtitle.into()).size(11).color(palette.text_muted),
    ]
    .spacing(4)
    .width(Length::Fill);

    let card_content = if let Some(ic) = watermark_icon {
        let watermark_color = Color::from_rgba(
            palette.text_primary.r,
            palette.text_primary.g,
            palette.text_primary.b,
            0.18,
        );
        let icon_el = icon(ic, 24, watermark_color);
        row![text_col, icon_el]
            .spacing(8)
            .align_y(iced::Alignment::Start)
    } else {
        row![text_col]
    };

    container(card_content)
        .padding(14)
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
        text(title.into()).size(14).color(palette.text_primary),
        text(subtitle.into()).size(12).color(palette.text_secondary),
    ]
    .spacing(2);

    let mut content = column![header].spacing(4);
    for row_el in rows {
        content = content.push(form_divider(palette)).push(row_el);
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
    fn test_metric_card_rendering() {
        let palette = OrbitPalette::dark();
        let _card_with_icon: Element<'_, ()> = metric_card(
            "NODES",
            "3/3",
            "Ready / Total",
            Some(Icon::Server),
            &palette,
        );
        let _card_no_icon: Element<'_, ()> = metric_card(
            "DEPLOYMENTS",
            "12",
            "Active",
            None,
            &palette,
        );
    }

    #[test]
    fn test_section_card_rendering() {
        let palette = OrbitPalette::dark();
        let rows: Vec<Element<'_, ()>> = vec![
            text("Row 1").into(),
            text("Row 2").into(),
        ];
        let _section: Element<'_, ()> = section_card(
            "Section Title",
            "Section Subtitle",
            rows,
            &palette,
        );
        let _container: Element<'_, ()> = container_card(text("Content"), &palette);
    }

    #[test]
    fn test_alert_banner_rendering() {
        let dark = OrbitPalette::dark();
        let light = OrbitPalette::light();

        for palette in [&dark, &light] {
            let _info: Element<'_, ()> = alert_banner(
                AlertLevel::Info,
                "Note",
                "Cluster state synced",
                palette,
            );
            let _success: Element<'_, ()> = alert_banner(
                AlertLevel::Success,
                "Success",
                "Deployment scaled",
                palette,
            );
            let _warning: Element<'_, ()> = alert_banner(
                AlertLevel::Warning,
                "Warning",
                "High memory usage",
                palette,
            );
            let _danger: Element<'_, ()> = alert_banner(
                AlertLevel::Danger,
                "Error",
                "Pod crashed",
                palette,
            );
        }
    }

    #[test]
    fn test_card_elevation_and_radii() {
        let palette = OrbitPalette::dark();
        let card_style = card_container_style(&palette)(&iced::Theme::Dark);
        assert_eq!(card_style.border.radius, Radius::from(8.0));
        assert_eq!(card_style.shadow, palette.card_shadow());
    }
}
