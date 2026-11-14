use gpui::{AnyElement, FontWeight, Rgba, SharedString, div, prelude::*, px};

use crate::theme::{OrbitTheme, apply_text_step};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SemanticStatus {
    Success,
    Warning,
    Danger,
    Info,
    Neutral,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum StatusBadgeVariant {
    #[default]
    Inline,
    Pill,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct StatusBadgeStyle {
    pub dot_color: Rgba,
    pub text_color: Rgba,
    pub bg_color: Option<Rgba>,
}

pub fn classify_status(status: &str) -> SemanticStatus {
    match status.trim().to_ascii_lowercase().as_str() {
        "healthy" | "running" | "connected" | "ready" | "active" | "bound" | "available"
        | "completed" | "succeeded" | "normal" | "enforced" => SemanticStatus::Success,
        "pending" | "degraded" | "progressing" | "containercreating" | "podinitializing"
        | "released" | "warning" | "scalinglimited" | "suspended" | "paused" | "connecting" => {
            SemanticStatus::Warning
        }
        "failed" | "error" | "crashloopbackoff" | "terminating" | "notready" | "lost"
        | "unreachable" | "oomkilled" | "imagepullbackoff" | "errimagepull" | "backoff"
        | "evicted" => SemanticStatus::Danger,
        "info" | "informational" | "audit" | "clusterip" | "loadbalancer" | "nodeport"
        | "externalname" => SemanticStatus::Info,
        _ => SemanticStatus::Neutral,
    }
}

pub fn status_badge_style(
    status: SemanticStatus,
    variant: StatusBadgeVariant,
    theme: &OrbitTheme,
) -> StatusBadgeStyle {
    match status {
        SemanticStatus::Success => {
            let shade = theme.semantic.success;
            StatusBadgeStyle {
                dot_color: shade.solid,
                text_color: shade.solid,
                bg_color: match variant {
                    StatusBadgeVariant::Inline => None,
                    StatusBadgeVariant::Pill => Some(shade.soft),
                },
            }
        }
        SemanticStatus::Warning => {
            let shade = theme.semantic.warning;
            StatusBadgeStyle {
                dot_color: shade.solid,
                text_color: shade.solid,
                bg_color: match variant {
                    StatusBadgeVariant::Inline => None,
                    StatusBadgeVariant::Pill => Some(shade.soft),
                },
            }
        }
        SemanticStatus::Danger => {
            let shade = theme.semantic.danger;
            StatusBadgeStyle {
                dot_color: shade.solid,
                text_color: shade.solid,
                bg_color: match variant {
                    StatusBadgeVariant::Inline => None,
                    StatusBadgeVariant::Pill => Some(shade.soft),
                },
            }
        }
        SemanticStatus::Info => {
            let shade = theme.semantic.info;
            StatusBadgeStyle {
                dot_color: shade.solid,
                text_color: shade.solid,
                bg_color: match variant {
                    StatusBadgeVariant::Inline => None,
                    StatusBadgeVariant::Pill => Some(shade.soft),
                },
            }
        }
        SemanticStatus::Neutral => StatusBadgeStyle {
            dot_color: theme.text.muted,
            text_color: match variant {
                StatusBadgeVariant::Inline => theme.text.muted,
                StatusBadgeVariant::Pill => theme.text.secondary,
            },
            bg_color: match variant {
                StatusBadgeVariant::Inline => None,
                StatusBadgeVariant::Pill => Some(theme.surface.control_bg),
            },
        },
    }
}

pub fn render_status_badge(
    label: impl Into<SharedString>,
    status: SemanticStatus,
    variant: StatusBadgeVariant,
    theme: &OrbitTheme,
) -> AnyElement {
    let style = status_badge_style(status, variant, theme);
    let label_str = label.into();
    let base = apply_text_step(div(), theme.typography.size_xs)
        .flex()
        .items_center()
        .gap_1p5()
        .font_weight(match variant {
            StatusBadgeVariant::Inline => FontWeight::MEDIUM,
            StatusBadgeVariant::Pill => FontWeight::SEMIBOLD,
        })
        .text_color(style.text_color)
        .when_some(style.bg_color, |el, bg| {
            el.bg(bg)
                .px_2()
                .py_0p5()
                .rounded(theme.style.radius_control)
        });

    base.child(
        div()
            .w(px(6.0))
            .h(px(6.0))
            .rounded_full()
            .flex_shrink_0()
            .bg(style.dot_color),
    )
    .child(label_str)
    .into_any_element()
}

pub fn render_status_badge_from_str(
    status_label: &str,
    variant: StatusBadgeVariant,
    theme: &OrbitTheme,
) -> AnyElement {
    let trimmed = status_label.trim();
    let display = if trimmed.is_empty() {
        "Unknown"
    } else {
        trimmed
    };
    let semantic = classify_status(trimmed);
    render_status_badge(SharedString::from(display.to_string()), semantic, variant, theme)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::theme::{
        AccentColor, SemanticColorsPreference, SemanticPreset, ThemeMode, UiStyle, rgb_hex,
    };
    use gpui::WindowAppearance;

    #[test]
    fn test_classify_status_maps_k8s_states_to_semantic_tokens() {
        for s in [
            "Healthy",
            "healthy",
            "Running",
            "running",
            "Connected",
            "Ready",
            "Active",
            "Bound",
            "Available",
            "Completed",
            "Succeeded",
            "Normal",
            "Enforced",
            "  Running  ",
        ] {
            assert_eq!(
                classify_status(s),
                SemanticStatus::Success,
                "expected {s:?} to classify as Success"
            );
        }

        for s in [
            "Pending",
            "pending",
            "Degraded",
            "Progressing",
            "ContainerCreating",
            "PodInitializing",
            "Released",
            "Warning",
            "ScalingLimited",
            "Suspended",
            "Paused",
            "Connecting",
        ] {
            assert_eq!(
                classify_status(s),
                SemanticStatus::Warning,
                "expected {s:?} to classify as Warning"
            );
        }

        for s in [
            "Failed",
            "failed",
            "Error",
            "CrashLoopBackOff",
            "Terminating",
            "NotReady",
            "Lost",
            "Unreachable",
            "OOMKilled",
            "ImagePullBackOff",
            "ErrImagePull",
            "BackOff",
            "Evicted",
        ] {
            assert_eq!(
                classify_status(s),
                SemanticStatus::Danger,
                "expected {s:?} to classify as Danger"
            );
        }

        for s in [
            "Info",
            "Informational",
            "Audit",
            "ClusterIP",
            "LoadBalancer",
            "NodePort",
            "ExternalName",
        ] {
            assert_eq!(
                classify_status(s),
                SemanticStatus::Info,
                "expected {s:?} to classify as Info"
            );
        }

        for s in ["Unknown", "unknown", "Disconnected", "", "   ", "CustomState"] {
            assert_eq!(
                classify_status(s),
                SemanticStatus::Neutral,
                "expected {s:?} to classify as Neutral"
            );
        }
    }

    #[test]
    fn test_status_badge_style_resolves_theme_tokens_across_modes_and_presets() {
        let dark_default = OrbitTheme::resolve(
            ThemeMode::Dark,
            UiStyle::Flat,
            AccentColor::Blue,
            SemanticColorsPreference::default(),
            WindowAppearance::Dark,
        );
        let light_default = OrbitTheme::resolve(
            ThemeMode::Light,
            UiStyle::Flat,
            AccentColor::Blue,
            SemanticColorsPreference::default(),
            WindowAppearance::Light,
        );
        let dark_colorblind = OrbitTheme::resolve(
            ThemeMode::Dark,
            UiStyle::Flat,
            AccentColor::Blue,
            SemanticColorsPreference {
                preset: SemanticPreset::Colorblind,
                success: "teal".to_string(),
                warning: "amber".to_string(),
                danger: "crimson".to_string(),
                info: "sky".to_string(),
            },
            WindowAppearance::Dark,
        );

        let inline_success = status_badge_style(
            SemanticStatus::Success,
            StatusBadgeVariant::Inline,
            &dark_default,
        );
        assert_eq!(inline_success.dot_color, dark_default.semantic.success.solid);
        assert_eq!(inline_success.text_color, dark_default.semantic.success.solid);
        assert_eq!(inline_success.bg_color, None);

        let pill_danger = status_badge_style(
            SemanticStatus::Danger,
            StatusBadgeVariant::Pill,
            &dark_default,
        );
        assert_eq!(pill_danger.dot_color, dark_default.semantic.danger.solid);
        assert_eq!(pill_danger.text_color, dark_default.semantic.danger.solid);
        assert_eq!(pill_danger.bg_color, Some(dark_default.semantic.danger.soft));

        let light_warning = status_badge_style(
            SemanticStatus::Warning,
            StatusBadgeVariant::Inline,
            &light_default,
        );
        assert_eq!(light_warning.dot_color, light_default.semantic.warning.solid);
        assert_ne!(light_warning.dot_color, dark_default.semantic.warning.solid);

        let cb_success = status_badge_style(
            SemanticStatus::Success,
            StatusBadgeVariant::Pill,
            &dark_colorblind,
        );
        assert_eq!(cb_success.dot_color, rgb_hex(0x14b8a6));
        assert_eq!(cb_success.bg_color, Some(dark_colorblind.semantic.success.soft));

        let pill_info = status_badge_style(
            SemanticStatus::Info,
            StatusBadgeVariant::Pill,
            &dark_default,
        );
        assert_eq!(pill_info.dot_color, dark_default.semantic.info.solid);
        assert_eq!(pill_info.bg_color, Some(dark_default.semantic.info.soft));

        let inline_neutral = status_badge_style(
            SemanticStatus::Neutral,
            StatusBadgeVariant::Inline,
            &dark_default,
        );
        assert_eq!(inline_neutral.dot_color, dark_default.text.muted);
        assert_eq!(inline_neutral.text_color, dark_default.text.muted);
        assert_eq!(inline_neutral.bg_color, None);

        let pill_neutral = status_badge_style(
            SemanticStatus::Neutral,
            StatusBadgeVariant::Pill,
            &dark_default,
        );
        assert_eq!(pill_neutral.dot_color, dark_default.text.muted);
        assert_eq!(pill_neutral.text_color, dark_default.text.secondary);
        assert_eq!(pill_neutral.bg_color, Some(dark_default.surface.control_bg));
    }
}
