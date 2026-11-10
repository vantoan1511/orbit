use iced::Color;
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
            bg_app: Color::from_rgb(0.06, 0.07, 0.09),      // #0f1217
            bg_sidebar: Color::from_rgb(0.08, 0.09, 0.11),  // #14171c
            bg_panel: Color::from_rgb(0.09, 0.10, 0.13),    // #171a21
            bg_card: Color::from_rgb(0.11, 0.12, 0.16),     // #1c1f29
            bg_hover: Color::from_rgb(0.16, 0.18, 0.23),    // #292e3b
            border: Color::from_rgb(0.18, 0.20, 0.25),      // #2e3340
            text_primary: Color::from_rgb(0.95, 0.96, 0.98), // #f2f5fa
            text_secondary: Color::from_rgb(0.72, 0.75, 0.80),
            text_muted: Color::from_rgb(0.50, 0.53, 0.60),
            accent: Color::from_rgb(0.42, 0.66, 1.0),       // #6aa8ff
            success: Color::from_rgb(0.20, 0.78, 0.44),     // #33c770
            warning: Color::from_rgb(0.95, 0.77, 0.25),     // #f2c440
            danger: Color::from_rgb(0.92, 0.30, 0.30),      // #eb4d4d
            info: Color::from_rgb(0.38, 0.65, 0.98),        // #61a6fa
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

#[cfg(test)]
mod tests {
    use super::*;

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
