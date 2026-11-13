use iced::{
    border::Radius,
    widget::{container, row, text},
    Background, Border, Color, Element, Length,
};
use orbit_core::command::ResourceKind;

use crate::theme::{resource_kind_color, OrbitPalette, Typography};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BadgeVariant {
    Success,
    Warning,
    Danger,
    Info,
    Neutral,
    Custom(Color),
}

impl BadgeVariant {
    pub fn color(&self, palette: &OrbitPalette) -> Color {
        match self {
            Self::Success => palette.success,
            Self::Warning => palette.warning,
            Self::Danger => palette.danger,
            Self::Info => palette.info,
            Self::Neutral => palette.text_secondary,
            Self::Custom(c) => *c,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BadgeSize {
    Small,
    Medium,
}

impl BadgeSize {
    pub fn text_size(&self) -> u16 {
        match self {
            Self::Small => Typography::CAPTION,
            Self::Medium => Typography::SECONDARY,
        }
    }

    pub fn padding(&self) -> [u16; 2] {
        match self {
            Self::Small => [2, 6],
            Self::Medium => [3, 8],
        }
    }

    pub fn dot_size(&self) -> f32 {
        match self {
            Self::Small => 6.0,
            Self::Medium => 7.0,
        }
    }
}

pub fn badge<'a, Message: 'a>(
    label: impl Into<String>,
    variant: BadgeVariant,
    size: BadgeSize,
    show_dot: bool,
    palette: &'a OrbitPalette,
) -> Element<'a, Message> {
    let color = variant.color(palette);
    let bg_color = Color::from_rgba(color.r, color.g, color.b, 0.12);

    let mut content = row![].spacing(5).align_y(iced::Alignment::Center);

    if show_dot {
        let dot = container(row![])
            .width(Length::Fixed(size.dot_size()))
            .height(Length::Fixed(size.dot_size()))
            .style(move |_theme| container::Style {
                background: Some(Background::Color(color)),
                border: Border {
                    color: Color::TRANSPARENT,
                    width: 0.0,
                    radius: Radius::from(999.0),
                },
                ..Default::default()
            });
        content = content.push(dot);
    }

    content = content.push(
        text(label.into())
            .size(size.text_size())
            .color(color),
    );

    container(content)
        .padding(size.padding())
        .style(move |_theme| container::Style {
            background: Some(Background::Color(bg_color)),
            border: Border {
                color: Color::TRANSPARENT,
                width: 0.0,
                radius: Radius::from(999.0),
            },
            ..Default::default()
        })
        .into()
}

pub fn status_pill<'a, Message: 'a>(
    status: &'a str,
    size: BadgeSize,
    palette: &'a OrbitPalette,
) -> Element<'a, Message> {
    let color = palette.status_color(status);
    let variant = if color == palette.success {
        BadgeVariant::Success
    } else if color == palette.warning {
        BadgeVariant::Warning
    } else if color == palette.danger {
        BadgeVariant::Danger
    } else {
        BadgeVariant::Info
    };
    badge(status, variant, size, false, palette)
}

pub fn status_dot_pill<'a, Message: 'a>(
    label: impl Into<String>,
    status: &str,
    size: BadgeSize,
    palette: &'a OrbitPalette,
) -> Element<'a, Message> {
    let color = palette.status_color(status);
    let variant = if color == palette.success {
        BadgeVariant::Success
    } else if color == palette.warning {
        BadgeVariant::Warning
    } else if color == palette.danger {
        BadgeVariant::Danger
    } else {
        BadgeVariant::Info
    };
    badge(label, variant, size, true, palette)
}

pub fn resource_kind_display(kind: ResourceKind) -> &'static str {
    match kind {
        ResourceKind::Namespace => "Namespace",
        ResourceKind::Pod => "Pod",
        ResourceKind::Deployment => "Deployment",
        ResourceKind::StatefulSet => "StatefulSet",
        ResourceKind::DaemonSet => "DaemonSet",
        ResourceKind::ReplicaSet => "ReplicaSet",
        ResourceKind::Job => "Job",
        ResourceKind::CronJob => "CronJob",
        ResourceKind::HorizontalPodAutoscaler => "HPA",
        ResourceKind::Node => "Node",
        ResourceKind::Service => "Service",
        ResourceKind::Ingress => "Ingress",
        ResourceKind::ConfigMap => "ConfigMap",
        ResourceKind::Secret => "Secret",
        ResourceKind::Event => "Event",
        ResourceKind::PersistentVolume => "PV",
        ResourceKind::PersistentVolumeClaim => "PVC",
        ResourceKind::StorageClass => "StorageClass",
        ResourceKind::Policy => "Policy",
    }
}

pub fn resource_kind_badge<'a, Message: 'a>(
    kind: ResourceKind,
    size: BadgeSize,
    palette: &'a OrbitPalette,
) -> Element<'a, Message> {
    let color = resource_kind_color(kind);
    badge(
        resource_kind_display(kind),
        BadgeVariant::Custom(color),
        size,
        false,
        palette,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_badge_variants_and_sizes() {
        let palette = OrbitPalette::dark();
        let variants = [
            BadgeVariant::Success,
            BadgeVariant::Warning,
            BadgeVariant::Danger,
            BadgeVariant::Info,
            BadgeVariant::Neutral,
            BadgeVariant::Custom(Color::from_rgb(0.5, 0.2, 0.8)),
        ];

        for variant in variants {
            for size in [BadgeSize::Small, BadgeSize::Medium] {
                let _el: Element<'_, ()> = badge("test", variant, size, false, &palette);
                let _el_with_dot: Element<'_, ()> = badge("test", variant, size, true, &palette);
            }
        }
    }

    #[test]
    fn test_status_dot_pill_rendering() {
        let palette = OrbitPalette::dark();
        let _ready: Element<'_, ()> = status_dot_pill("Ready", "running", BadgeSize::Small, &palette);
        let _warning: Element<'_, ()> = status_dot_pill("Pending", "pending", BadgeSize::Medium, &palette);
        let _offline: Element<'_, ()> = status_dot_pill("Offline", "failed", BadgeSize::Small, &palette);
        let _status: Element<'_, ()> = status_pill("Active", BadgeSize::Small, &palette);
        let _res: Element<'_, ()> = resource_kind_badge(ResourceKind::Pod, BadgeSize::Small, &palette);
    }
}
