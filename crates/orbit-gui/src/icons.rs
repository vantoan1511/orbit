use iced::{widget::text, Color, Element, Font};
pub use lucide_icons::{Icon, LUCIDE_FONT_BYTES};
use orbit_core::command::ResourceKind;

use crate::state::tabs::TabContent;

pub const LUCIDE_FONT: Font = Font::with_name("lucide");

pub fn icon<'a, Message: 'a>(icon: Icon, size: u16, color: Color) -> Element<'a, Message> {
    text(char::from(icon))
        .font(LUCIDE_FONT)
        .size(size)
        .color(color)
        .into()
}

pub fn button_icon<'a, Message: 'a>(icon: Icon, size: u16) -> Element<'a, Message> {
    text(char::from(icon))
        .font(LUCIDE_FONT)
        .size(size)
        .into()
}

pub fn resource_kind_icon(kind: ResourceKind) -> Icon {
    match kind {
        ResourceKind::Namespace => Icon::Folder,
        ResourceKind::Pod => Icon::Box,
        ResourceKind::Deployment => Icon::Layers,
        ResourceKind::StatefulSet => Icon::Database,
        ResourceKind::DaemonSet => Icon::Cpu,
        ResourceKind::ReplicaSet => Icon::Copy,
        ResourceKind::Job => Icon::Play,
        ResourceKind::CronJob => Icon::Clock,
        ResourceKind::HorizontalPodAutoscaler => Icon::Scale,
        ResourceKind::Node => Icon::Server,
        ResourceKind::Service => Icon::Network,
        ResourceKind::Ingress => Icon::Globe,
        ResourceKind::ConfigMap => Icon::FileCode,
        ResourceKind::Secret => Icon::Key,
        ResourceKind::Event => Icon::Bell,
        ResourceKind::PersistentVolume => Icon::HardDrive,
        ResourceKind::PersistentVolumeClaim => Icon::FolderArchive,
        ResourceKind::StorageClass => Icon::Layers2,
        ResourceKind::Policy => Icon::ShieldCheck,
    }
}

pub fn tab_content_icon(content: &TabContent) -> Icon {
    match content {
        TabContent::Dashboard => Icon::LayoutDashboard,
        TabContent::ResourceTable(kind) => resource_kind_icon(*kind),
        TabContent::Settings => Icon::Settings,
        TabContent::YamlEditor { .. } => Icon::FileCode,
        TabContent::LogViewer { .. } => Icon::Terminal,
        #[cfg(debug_assertions)]
        TabContent::Showcase => Icon::Palette,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_all_resource_kinds_have_valid_icon_mapping() {
        let kinds = [
            ResourceKind::Namespace,
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
        ];

        for kind in kinds {
            let icon = resource_kind_icon(kind);
            let _c = char::from(icon);
        }
    }

    #[test]
    fn test_tab_content_icon_mapping() {
        assert_eq!(
            char::from(tab_content_icon(&TabContent::Dashboard)),
            char::from(Icon::LayoutDashboard)
        );
        assert_eq!(
            char::from(tab_content_icon(&TabContent::Settings)),
            char::from(Icon::Settings)
        );
        assert_eq!(
            char::from(tab_content_icon(&TabContent::ResourceTable(ResourceKind::Pod))),
            char::from(Icon::Box)
        );
        assert_eq!(
            char::from(tab_content_icon(&TabContent::YamlEditor {
                kind: "Pod".to_string(),
                name: "test".to_string(),
                namespace: None,
            })),
            char::from(Icon::FileCode)
        );
        assert_eq!(
            char::from(tab_content_icon(&TabContent::LogViewer {
                namespace: "default".to_string(),
                pod: "test".to_string(),
                container: None,
            })),
            char::from(Icon::Terminal)
        );
    }

    #[test]
    fn test_window_control_icons() {
        let _ = char::from(Icon::Minus);
        let _ = char::from(Icon::Square);
        let _ = char::from(Icon::Copy);
        let _ = char::from(Icon::X);

        let _el: Element<'_, ()> = button_icon(Icon::Minus, 12);
        let _el: Element<'_, ()> = button_icon(Icon::Square, 12);
        let _el: Element<'_, ()> = button_icon(Icon::X, 12);
    }

    #[cfg(debug_assertions)]
    #[test]
    fn test_showcase_tab_icon_mapping() {
        assert_eq!(
            char::from(tab_content_icon(&TabContent::Showcase)),
            char::from(Icon::Palette)
        );
    }
}
