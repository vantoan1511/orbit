use gpui::{AnyElement, Div, FontWeight, Pixels, Rgba, div, prelude::*, px};
use lucide_icons::Icon;

use crate::icons::render_icon;
use crate::theme::{K8sResourceKind, OrbitTheme, apply_text_step};

#[derive(Debug, Clone, Copy)]
pub struct ResourceKindMeta {
    pub label: &'static str,
    pub icon: Icon,
    pub color: Rgba,
    pub k8s_kind: Option<K8sResourceKind>,
}

impl PartialEq for ResourceKindMeta {
    fn eq(&self, other: &Self) -> bool {
        self.label == other.label
            && char::from(self.icon) == char::from(other.icon)
            && self.color == other.color
            && self.k8s_kind == other.k8s_kind
    }
}

pub fn parse_k8s_resource_kind(raw: &str) -> Option<K8sResourceKind> {
    match raw.trim().to_ascii_lowercase().as_str() {
        "deployment" | "deployments" | "deploy" => Some(K8sResourceKind::Deployment),
        "daemonset" | "daemonsets" | "ds" => Some(K8sResourceKind::DaemonSet),
        "statefulset" | "statefulsets" | "sts" => Some(K8sResourceKind::StatefulSet),
        "job" | "jobs" => Some(K8sResourceKind::Job),
        "pod" | "pods" | "po" => Some(K8sResourceKind::Pod),
        "replicaset" | "replicasets" | "rs" => Some(K8sResourceKind::ReplicaSet),
        "node" | "nodes" | "no" => Some(K8sResourceKind::Node),
        "secret" | "secrets" => Some(K8sResourceKind::Secret),
        "configmap" | "configmaps" | "cm" => Some(K8sResourceKind::ConfigMap),
        "service" | "services" | "svc" => Some(K8sResourceKind::Service),
        "ingress" | "ingresses" | "ing" => Some(K8sResourceKind::Ingress),
        "hpa" | "hpas" | "horizontalpodautoscaler" | "horizontalpodautoscalers" => {
            Some(K8sResourceKind::Hpa)
        }
        _ => None,
    }
}

pub fn resource_kind_meta(kind: K8sResourceKind, theme: &OrbitTheme) -> ResourceKindMeta {
    let (label, icon) = match kind {
        K8sResourceKind::Deployment => ("Deployment", Icon::Layers),
        K8sResourceKind::DaemonSet => ("DaemonSet", Icon::Ghost),
        K8sResourceKind::StatefulSet => ("StatefulSet", Icon::Database),
        K8sResourceKind::Job => ("Job", Icon::Hammer),
        K8sResourceKind::Pod => ("Pod", Icon::Box),
        K8sResourceKind::ReplicaSet => ("ReplicaSet", Icon::Copy),
        K8sResourceKind::Node => ("Node", Icon::Server),
        K8sResourceKind::Secret => ("Secret", Icon::KeyRound),
        K8sResourceKind::ConfigMap => ("ConfigMap", Icon::FileJson),
        K8sResourceKind::Service => ("Service", Icon::Network),
        K8sResourceKind::Ingress => ("Ingress", Icon::Globe),
        K8sResourceKind::Hpa => ("HPA", Icon::TrendingUp),
    };

    ResourceKindMeta {
        label,
        icon,
        color: theme.k8s_color(kind),
        k8s_kind: Some(kind),
    }
}

pub fn resource_kind_meta_from_str(kind: &str, theme: &OrbitTheme) -> ResourceKindMeta {
    if let Some(k8s_kind) = parse_k8s_resource_kind(kind) {
        return resource_kind_meta(k8s_kind, theme);
    }

    match kind.trim().to_ascii_lowercase().as_str() {
        "cronjob" | "cronjobs" | "cj" => ResourceKindMeta {
            label: "CronJob",
            icon: Icon::Clock,
            color: theme.k8s.job,
            k8s_kind: Some(K8sResourceKind::Job),
        },
        "namespace" | "namespaces" | "ns" => ResourceKindMeta {
            label: "Namespace",
            icon: Icon::FolderOpen,
            color: theme.text.secondary,
            k8s_kind: None,
        },
        "event" | "events" | "ev" => ResourceKindMeta {
            label: "Event",
            icon: Icon::Activity,
            color: theme.text.secondary,
            k8s_kind: None,
        },
        "persistentvolume" | "persistentvolumes" | "pv" | "pvs" => ResourceKindMeta {
            label: "PersistentVolume",
            icon: Icon::Archive,
            color: theme.text.secondary,
            k8s_kind: None,
        },
        "persistentvolumeclaim" | "persistentvolumeclaims" | "pvc" | "pvcs" => ResourceKindMeta {
            label: "PersistentVolumeClaim",
            icon: Icon::FileDown,
            color: theme.text.secondary,
            k8s_kind: None,
        },
        "storageclass" | "storageclasses" | "sc" => ResourceKindMeta {
            label: "StorageClass",
            icon: Icon::Settings2,
            color: theme.text.secondary,
            k8s_kind: None,
        },
        "networkpolicy" | "networkpolicies" | "netpol" => ResourceKindMeta {
            label: "NetworkPolicy",
            icon: Icon::Network,
            color: theme.text.secondary,
            k8s_kind: None,
        },
        "resourcequota" | "resourcequotas" | "quota" | "limitrange" | "limitranges" => {
            ResourceKindMeta {
                label: "ResourceQuota",
                icon: Icon::Sliders,
                color: theme.text.secondary,
                k8s_kind: None,
            }
        }
        "validatingwebhookconfiguration"
        | "mutatingwebhookconfiguration"
        | "admissionwebhook"
        | "webhook" => ResourceKindMeta {
            label: "AdmissionWebhook",
            icon: Icon::Webhook,
            color: theme.text.secondary,
            k8s_kind: None,
        },
        "policy" | "policies" | "podsecurity" => ResourceKindMeta {
            label: "Policy",
            icon: Icon::ShieldAlert,
            color: theme.text.secondary,
            k8s_kind: None,
        },
        _ => ResourceKindMeta {
            label: "Resource",
            icon: Icon::Box,
            color: theme.text.secondary,
            k8s_kind: None,
        },
    }
}

pub fn render_resource_kind_icon(kind: K8sResourceKind, size: Pixels, theme: &OrbitTheme) -> Div {
    let meta = resource_kind_meta(kind, theme);
    render_icon(meta.icon, size, meta.color)
}

pub fn render_resource_kind_badge(kind: &str, theme: &OrbitTheme) -> AnyElement {
    let meta = resource_kind_meta_from_str(kind, theme);
    apply_text_step(div(), theme.typography.size_xs)
        .flex()
        .items_center()
        .gap_1p5()
        .font_weight(FontWeight::MEDIUM)
        .text_color(theme.text.primary)
        .child(render_icon(meta.icon, px(14.0), meta.color))
        .child(meta.label)
        .into_any_element()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::navigation::WorkspaceView;
    use crate::theme::{AccentColor, SemanticColorsPreference, ThemeMode, UiStyle};
    use gpui::WindowAppearance;

    #[test]
    fn test_resource_kind_helpers_map_all_kinds_to_icons_and_theme_colors() {
        let dark_theme = OrbitTheme::resolve(
            ThemeMode::Dark,
            UiStyle::Flat,
            AccentColor::Blue,
            SemanticColorsPreference::default(),
            WindowAppearance::Dark,
        );
        let light_theme = OrbitTheme::resolve(
            ThemeMode::Light,
            UiStyle::Flat,
            AccentColor::Blue,
            SemanticColorsPreference::default(),
            WindowAppearance::Light,
        );

        let expected_views = [
            (K8sResourceKind::Deployment, WorkspaceView::Deployments, "Deployment"),
            (K8sResourceKind::DaemonSet, WorkspaceView::DaemonSets, "DaemonSet"),
            (K8sResourceKind::StatefulSet, WorkspaceView::StatefulSets, "StatefulSet"),
            (K8sResourceKind::Job, WorkspaceView::Jobs, "Job"),
            (K8sResourceKind::Pod, WorkspaceView::Pods, "Pod"),
            (K8sResourceKind::ReplicaSet, WorkspaceView::ReplicaSets, "ReplicaSet"),
            (K8sResourceKind::Node, WorkspaceView::Nodes, "Node"),
            (K8sResourceKind::Secret, WorkspaceView::Secrets, "Secret"),
            (K8sResourceKind::ConfigMap, WorkspaceView::ConfigMaps, "ConfigMap"),
            (K8sResourceKind::Service, WorkspaceView::Services, "Service"),
            (K8sResourceKind::Ingress, WorkspaceView::Ingresses, "Ingress"),
            (K8sResourceKind::Hpa, WorkspaceView::Hpas, "HPA"),
        ];

        for (kind, view, expected_label) in expected_views {
            let dark_meta = resource_kind_meta(kind, &dark_theme);
            assert_eq!(dark_meta.label, expected_label);
            assert_eq!(char::from(dark_meta.icon), char::from(view.icon()));
            assert_eq!(dark_meta.color, dark_theme.k8s_color(kind));
            assert_eq!(dark_meta.k8s_kind, Some(kind));

            let light_meta = resource_kind_meta(kind, &light_theme);
            assert_eq!(light_meta.color, light_theme.k8s_color(kind));
            assert_ne!(dark_meta.color, light_meta.color);
        }
    }

    #[test]
    fn test_resource_kind_string_parsing_handles_aliases_and_unknown() {
        let theme = OrbitTheme::resolve(
            ThemeMode::Dark,
            UiStyle::Flat,
            AccentColor::Blue,
            SemanticColorsPreference::default(),
            WindowAppearance::Dark,
        );

        assert_eq!(parse_k8s_resource_kind("Pod"), Some(K8sResourceKind::Pod));
        assert_eq!(parse_k8s_resource_kind("pods"), Some(K8sResourceKind::Pod));
        assert_eq!(
            parse_k8s_resource_kind("Deployment"),
            Some(K8sResourceKind::Deployment)
        );
        assert_eq!(
            parse_k8s_resource_kind("HorizontalPodAutoscaler"),
            Some(K8sResourceKind::Hpa)
        );
        assert_eq!(parse_k8s_resource_kind("hpa"), Some(K8sResourceKind::Hpa));
        assert_eq!(parse_k8s_resource_kind("hpas"), Some(K8sResourceKind::Hpa));
        assert_eq!(
            parse_k8s_resource_kind("Service"),
            Some(K8sResourceKind::Service)
        );
        assert_eq!(parse_k8s_resource_kind("Node"), Some(K8sResourceKind::Node));
        assert_eq!(parse_k8s_resource_kind("UnknownCustomResource"), None);

        let cronjob_meta = resource_kind_meta_from_str("CronJob", &theme);
        assert_eq!(cronjob_meta.label, "CronJob");
        assert_eq!(char::from(cronjob_meta.icon), char::from(Icon::Clock));
        assert_eq!(cronjob_meta.color, theme.k8s.job);
        assert_eq!(cronjob_meta.k8s_kind, Some(K8sResourceKind::Job));

        let ns_meta = resource_kind_meta_from_str("Namespace", &theme);
        assert_eq!(ns_meta.label, "Namespace");
        assert_eq!(char::from(ns_meta.icon), char::from(Icon::FolderOpen));
        assert_eq!(ns_meta.color, theme.text.secondary);

        let event_meta = resource_kind_meta_from_str("Event", &theme);
        assert_eq!(event_meta.label, "Event");
        assert_eq!(char::from(event_meta.icon), char::from(Icon::Activity));

        let pv_meta = resource_kind_meta_from_str("PersistentVolume", &theme);
        assert_eq!(pv_meta.label, "PersistentVolume");
        assert_eq!(char::from(pv_meta.icon), char::from(Icon::Archive));

        let pvc_meta = resource_kind_meta_from_str("PersistentVolumeClaim", &theme);
        assert_eq!(pvc_meta.label, "PersistentVolumeClaim");
        assert_eq!(char::from(pvc_meta.icon), char::from(Icon::FileDown));

        let sc_meta = resource_kind_meta_from_str("StorageClass", &theme);
        assert_eq!(sc_meta.label, "StorageClass");
        assert_eq!(char::from(sc_meta.icon), char::from(Icon::Settings2));

        let netpol_meta = resource_kind_meta_from_str("NetworkPolicy", &theme);
        assert_eq!(netpol_meta.label, "NetworkPolicy");
        assert_eq!(char::from(netpol_meta.icon), char::from(Icon::Network));

        let unknown_meta = resource_kind_meta_from_str("UnknownCustomResource", &theme);
        assert_eq!(unknown_meta.label, "Resource");
        assert_eq!(char::from(unknown_meta.icon), char::from(Icon::Box));
        assert_eq!(unknown_meta.color, theme.text.secondary);
        assert_eq!(unknown_meta.k8s_kind, None);
    }
}
