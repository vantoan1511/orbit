use crate::state::ActivityCategory;
use gpui::App;
use lucide_icons::{Icon, LUCIDE_FONT_BYTES};
use orbit_core::command::ResourceKind;
use std::borrow::Cow;

pub const LUCIDE_FONT_FAMILY: &str = "lucide";

pub fn register_lucide_font(cx: &App) {
    if let Err(err) = cx
        .text_system()
        .add_fonts(vec![Cow::Borrowed(LUCIDE_FONT_BYTES)])
    {
        eprintln!("Failed to register Lucide font: {err}");
    }
}

pub fn icon_glyph(icon: Icon) -> String {
    char::from(icon).to_string()
}

pub fn category_icon(cat: ActivityCategory) -> Icon {
    match cat {
        ActivityCategory::Workloads => Icon::Layers,
        ActivityCategory::Network => Icon::Network,
        ActivityCategory::Storage => Icon::HardDrive,
        ActivityCategory::Config => Icon::FileText,
        ActivityCategory::Cluster => Icon::Server,
        ActivityCategory::Settings => Icon::Settings,
    }
}

pub fn resource_icon(kind: ResourceKind) -> Icon {
    match kind {
        ResourceKind::Pod => Icon::Box,
        ResourceKind::Deployment => Icon::Boxes,
        ResourceKind::DaemonSet => Icon::Copy,
        ResourceKind::StatefulSet => Icon::Database,
        ResourceKind::Node => Icon::Server,
        ResourceKind::Service => Icon::Globe,
        ResourceKind::ConfigMap => Icon::FileText,
        ResourceKind::Secret => Icon::Key,
        ResourceKind::Namespace => Icon::Folder,
        ResourceKind::ReplicaSet => Icon::Layers,
        ResourceKind::Job | ResourceKind::CronJob => Icon::Clock,
        ResourceKind::HorizontalPodAutoscaler => Icon::Gauge,
        ResourceKind::Ingress => Icon::Globe,
        ResourceKind::Event => Icon::Activity,
        ResourceKind::PersistentVolume
        | ResourceKind::PersistentVolumeClaim
        | ResourceKind::StorageClass => Icon::HardDrive,
        ResourceKind::Policy => Icon::Shield,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_all_categories_and_supported_kinds_map_to_non_empty_glyphs() {
        let categories = [
            ActivityCategory::Workloads,
            ActivityCategory::Network,
            ActivityCategory::Storage,
            ActivityCategory::Config,
            ActivityCategory::Cluster,
            ActivityCategory::Settings,
        ];
        for cat in categories {
            let glyph = icon_glyph(category_icon(cat));
            assert!(!glyph.is_empty(), "category {:?} mapped to empty glyph", cat);
        }

        let kinds = [
            ResourceKind::Pod,
            ResourceKind::Deployment,
            ResourceKind::DaemonSet,
            ResourceKind::StatefulSet,
            ResourceKind::Node,
            ResourceKind::Service,
            ResourceKind::ConfigMap,
            ResourceKind::Secret,
        ];
        for kind in kinds {
            let glyph = icon_glyph(resource_icon(kind));
            assert!(!glyph.is_empty(), "kind {:?} mapped to empty glyph", kind);
        }
    }
}
