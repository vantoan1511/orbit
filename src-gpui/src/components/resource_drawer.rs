use std::rc::Rc;

use gpui::{
    AnyElement, App, CursorStyle, FontWeight, MouseButton, SharedString, Styled, Window, div,
    prelude::*, px,
};
use lucide_icons::Icon;

use crate::components::resource_kind::render_resource_kind_badge;
use crate::components::status_badge::{StatusBadgeVariant, render_status_badge_from_str};
use crate::components::yaml_viewer::YamlLoadStatus;
use crate::icons::render_icon;
use crate::state::ClusterBridgeCommand;
use crate::theme::{OrbitTheme, apply_text_step};

pub const DEFAULT_DRAWER_WIDTH: f32 = 600.0;
pub const MIN_DRAWER_WIDTH: f32 = 380.0;
pub const MAX_DRAWER_WIDTH: f32 = 960.0;
pub const DRAWER_MIN_WORKSPACE_MARGIN: f32 = 240.0;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ResourceDrawerTabId {
    Overview,
    Events,
    Yaml,
    Custom(SharedString),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResourceDrawerTabDef {
    pub id: ResourceDrawerTabId,
    pub label: SharedString,
    pub badge: Option<SharedString>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ResourceDrawerTarget {
    Deployment {
        name: String,
        namespace: String,
    },
    Pod {
        name: String,
        namespace: String,
    },
    Service {
        name: String,
        namespace: String,
    },
    ConfigMap {
        name: String,
        namespace: String,
    },
    Node {
        name: String,
    },
    Namespace {
        name: String,
    },
    Event {
        uid: String,
        reason: String,
        namespace: String,
    },
}

impl ResourceDrawerTarget {
    pub fn kind(&self) -> &'static str {
        match self {
            Self::Deployment { .. } => "Deployment",
            Self::Pod { .. } => "Pod",
            Self::Service { .. } => "Service",
            Self::ConfigMap { .. } => "ConfigMap",
            Self::Node { .. } => "Node",
            Self::Namespace { .. } => "Namespace",
            Self::Event { .. } => "Event",
        }
    }

    pub fn name(&self) -> &str {
        match self {
            Self::Deployment { name, .. }
            | Self::Pod { name, .. }
            | Self::Service { name, .. }
            | Self::ConfigMap { name, .. }
            | Self::Node { name }
            | Self::Namespace { name } => name.as_str(),
            Self::Event { uid, .. } => uid.as_str(),
        }
    }

    pub fn display_title(&self) -> &str {
        match self {
            Self::Deployment { name, .. }
            | Self::Pod { name, .. }
            | Self::Service { name, .. }
            | Self::ConfigMap { name, .. }
            | Self::Node { name }
            | Self::Namespace { name } => name.as_str(),
            Self::Event { reason, .. } => reason.as_str(),
        }
    }

    pub fn namespace(&self) -> Option<&str> {
        match self {
            Self::Deployment { namespace, .. }
            | Self::Pod { namespace, .. }
            | Self::Service { namespace, .. }
            | Self::ConfigMap { namespace, .. }
            | Self::Event { namespace, .. } => Some(namespace.as_str()),
            Self::Node { .. } | Self::Namespace { .. } => None,
        }
    }

    pub fn supports_live_yaml(&self) -> bool {
        !matches!(self, Self::Event { .. })
    }
}

#[derive(Debug, Clone)]
pub struct ResourceDrawerState {
    pub is_open: bool,
    pub target: Option<ResourceDrawerTarget>,
    pub active_tab: ResourceDrawerTabId,
    pub width: f32,
    pub is_resizing: bool,
    pub yaml_cluster_id: Option<String>,
    pub yaml_generation: u64,
    pub yaml_status: YamlLoadStatus,
    pub yaml_content: Option<String>,
    pub yaml_error: Option<String>,
    pub yaml_copied: bool,
}

impl Default for ResourceDrawerState {
    fn default() -> Self {
        Self {
            is_open: false,
            target: None,
            active_tab: ResourceDrawerTabId::Overview,
            width: DEFAULT_DRAWER_WIDTH,
            is_resizing: false,
            yaml_cluster_id: None,
            yaml_generation: 0,
            yaml_status: YamlLoadStatus::Idle,
            yaml_content: None,
            yaml_error: None,
            yaml_copied: false,
        }
    }
}

impl ResourceDrawerState {
    pub fn open_or_switch(
        &mut self,
        cluster_id: Option<&str>,
        target: ResourceDrawerTarget,
        initial_tab: ResourceDrawerTabId,
    ) -> Option<ClusterBridgeCommand> {
        let same_target =
            self.target.as_ref() == Some(&target) && self.yaml_cluster_id.as_deref() == cluster_id;

        self.is_open = true;
        self.is_resizing = false;
        self.active_tab = initial_tab.clone();
        self.yaml_copied = false;

        if !same_target {
            self.target = Some(target);
            self.yaml_cluster_id = cluster_id.map(str::to_string);
            self.yaml_generation = self.yaml_generation.saturating_add(1);
            self.yaml_status = YamlLoadStatus::Idle;
            self.yaml_content = None;
            self.yaml_error = None;
        }

        if initial_tab == ResourceDrawerTabId::Yaml {
            self.ensure_yaml_loaded(cluster_id)
        } else {
            None
        }
    }

    pub fn select_tab(
        &mut self,
        cluster_id: Option<&str>,
        tab: ResourceDrawerTabId,
    ) -> Option<ClusterBridgeCommand> {
        self.active_tab = tab.clone();
        self.yaml_copied = false;
        if tab == ResourceDrawerTabId::Yaml {
            self.ensure_yaml_loaded(cluster_id)
        } else {
            None
        }
    }

    fn ensure_yaml_loaded(&mut self, cluster_id: Option<&str>) -> Option<ClusterBridgeCommand> {
        let target = self.target.as_ref()?;
        if !target.supports_live_yaml() {
            return None;
        }
        let same_cluster = self.yaml_cluster_id.as_deref() == cluster_id;
        if same_cluster
            && matches!(
                self.yaml_status,
                YamlLoadStatus::Loading | YamlLoadStatus::Loaded
            )
        {
            return None;
        }
        self.retry_yaml(cluster_id)
    }

    pub fn retry_yaml(&mut self, cluster_id: Option<&str>) -> Option<ClusterBridgeCommand> {
        let target = self.target.as_ref()?.clone();
        if !target.supports_live_yaml() {
            return None;
        }
        let Some(cid) = cluster_id.filter(|s| !s.trim().is_empty()) else {
            self.yaml_status = YamlLoadStatus::Error;
            self.yaml_content = None;
            self.yaml_error = Some("No active cluster connected.".to_string());
            return None;
        };

        self.yaml_cluster_id = Some(cid.to_string());
        self.yaml_generation = self.yaml_generation.saturating_add(1);
        self.yaml_status = YamlLoadStatus::Loading;
        self.yaml_content = None;
        self.yaml_error = None;
        self.yaml_copied = false;

        Some(ClusterBridgeCommand::LoadResourceYaml {
            cluster_id: cid.to_string(),
            generation: self.yaml_generation,
            kind: target.kind().to_string(),
            namespace: target.namespace().map(str::to_string),
            name: target.name().to_string(),
        })
    }

    fn matches_yaml_response(
        &self,
        cluster_id: &str,
        generation: u64,
        kind: &str,
        namespace: Option<&str>,
        name: &str,
    ) -> bool {
        let Some(target) = self.target.as_ref() else {
            return false;
        };
        self.is_open
            && self.yaml_generation == generation
            && self.yaml_cluster_id.as_deref() == Some(cluster_id)
            && target.kind() == kind
            && target.namespace() == namespace
            && target.name() == name
    }

    pub fn apply_yaml_loaded(
        &mut self,
        cluster_id: &str,
        generation: u64,
        kind: &str,
        namespace: Option<&str>,
        name: &str,
        yaml: String,
    ) -> bool {
        if !self.matches_yaml_response(cluster_id, generation, kind, namespace, name) {
            return false;
        }
        self.yaml_status = YamlLoadStatus::Loaded;
        self.yaml_content = Some(yaml);
        self.yaml_error = None;
        true
    }

    pub fn apply_yaml_failed(
        &mut self,
        cluster_id: &str,
        generation: u64,
        kind: &str,
        namespace: Option<&str>,
        name: &str,
        message: String,
    ) -> bool {
        if !self.matches_yaml_response(cluster_id, generation, kind, namespace, name) {
            return false;
        }
        self.yaml_status = YamlLoadStatus::Error;
        self.yaml_content = None;
        self.yaml_error = Some(message);
        true
    }

    pub fn close(&mut self) {
        self.is_open = false;
        self.is_resizing = false;
        self.yaml_copied = false;
        self.yaml_generation = self.yaml_generation.saturating_add(1);
    }

    pub fn reset_for_context_change(&mut self) {
        self.is_open = false;
        self.is_resizing = false;
        self.target = None;
        self.yaml_cluster_id = None;
        self.yaml_generation = self.yaml_generation.saturating_add(1);
        self.yaml_status = YamlLoadStatus::Idle;
        self.yaml_content = None;
        self.yaml_error = None;
        self.yaml_copied = false;
    }

    pub fn start_resize(&mut self) {
        self.is_resizing = true;
    }

    pub fn stop_resize(&mut self) {
        self.is_resizing = false;
    }

    pub fn update_resize_from_window_x(&mut self, window_width: f32, pointer_x: f32) {
        let desired = window_width - pointer_x;
        self.width = clamp_drawer_width(desired);
    }
}

pub fn clamp_drawer_width(width: f32) -> f32 {
    if !width.is_finite() {
        return DEFAULT_DRAWER_WIDTH;
    }
    width.clamp(MIN_DRAWER_WIDTH, MAX_DRAWER_WIDTH)
}

pub fn effective_drawer_width(requested_width: f32, window_width: f32) -> f32 {
    let clamped = clamp_drawer_width(requested_width);
    if !window_width.is_finite() || window_width <= 0.0 {
        return clamped;
    }
    let max_for_viewport = (window_width - DRAWER_MIN_WORKSPACE_MARGIN).max(200.0);
    clamped.min(max_for_viewport)
}

pub struct ResourceDrawerProps {
    pub title: SharedString,
    pub kind: SharedString,
    pub namespace: Option<SharedString>,
    pub age: Option<SharedString>,
    pub status: Option<SharedString>,
    pub tabs: Vec<ResourceDrawerTabDef>,
    pub active_tab: ResourceDrawerTabId,
    pub width: f32,
    pub window_width: f32,
    pub on_select_tab: Rc<dyn Fn(ResourceDrawerTabId, &mut Window, &mut App)>,
    pub on_close: Rc<dyn Fn(&mut Window, &mut App)>,
    pub on_start_resize: Rc<dyn Fn(&mut Window, &mut App)>,
    pub body: AnyElement,
}

pub fn render_resource_drawer(
    is_open: bool,
    props: Option<ResourceDrawerProps>,
    theme: &OrbitTheme,
) -> Option<AnyElement> {
    if !is_open {
        return None;
    }
    let props = props?;
    let drawer_width = effective_drawer_width(props.width, props.window_width);

    let on_close = props.on_close.clone();
    let close_button = div()
        .id("resource-drawer-close-btn")
        .w(px(28.0))
        .h(px(28.0))
        .rounded(theme.style.radius_control)
        .flex()
        .items_center()
        .justify_center()
        .cursor_pointer()
        .hover(|s| s.bg(theme.surface.bg_hover))
        .on_mouse_down(MouseButton::Left, move |_, window, cx| {
            cx.stop_propagation();
            on_close(window, cx);
        })
        .child(render_icon(Icon::X, px(15.0), theme.text.secondary));

    let mut meta_parts: Vec<SharedString> = Vec::new();
    if let Some(ns) = props.namespace.as_ref().filter(|s| !s.is_empty()) {
        meta_parts.push(SharedString::from(format!("Namespace: {ns}")));
    }
    if let Some(age) = props.age.as_ref().filter(|s| !s.is_empty()) {
        meta_parts.push(SharedString::from(format!("Age: {age}")));
    }

    let header = div()
        .w_full()
        .px_4()
        .pt_3p5()
        .pb_3()
        .border_b_1()
        .border_color(theme.border.default)
        .flex()
        .flex_col()
        .gap_2()
        .child(
            div()
                .w_full()
                .flex()
                .items_center()
                .justify_between()
                .gap_2()
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap_2()
                        .child(render_resource_kind_badge(props.kind.as_ref(), theme))
                        .when_some(props.status.as_ref(), |el, status| {
                            el.child(render_status_badge_from_str(
                                status.as_ref(),
                                StatusBadgeVariant::Pill,
                                theme,
                            ))
                        }),
                )
                .child(close_button),
        )
        .child(
            apply_text_step(div(), theme.typography.size_lg)
                .font_weight(FontWeight::BOLD)
                .text_color(theme.text.primary)
                .truncate()
                .child(props.title),
        )
        .when(!meta_parts.is_empty(), |el| {
            el.child(
                div()
                    .flex()
                    .items_center()
                    .gap_3()
                    .children(meta_parts.into_iter().map(|part| {
                        apply_text_step(div(), theme.typography.size_xs)
                            .text_color(theme.text.muted)
                            .child(part)
                    })),
            )
        });

    let tab_bar = div()
        .w_full()
        .px_4()
        .border_b_1()
        .border_color(theme.border.default)
        .bg(theme.surface.bg_panel)
        .flex()
        .items_center()
        .gap_1()
        .children(props.tabs.into_iter().enumerate().map(|(idx, tab)| {
            let is_active = tab.id == props.active_tab;
            let tab_id = tab.id.clone();
            let on_select_tab = props.on_select_tab.clone();
            div()
                .id(("resource-drawer-tab", idx))
                .px_3()
                .py_2()
                .border_b_2()
                .border_color(if is_active {
                    theme.accent.accent
                } else {
                    gpui::rgba(0x00000000)
                })
                .cursor_pointer()
                .hover(|s| s.bg(theme.surface.bg_hover))
                .on_mouse_down(MouseButton::Left, move |_, window, cx| {
                    cx.stop_propagation();
                    on_select_tab(tab_id.clone(), window, cx);
                })
                .flex()
                .items_center()
                .gap_1p5()
                .child(
                    apply_text_step(div(), theme.typography.size_xs)
                        .font_weight(if is_active {
                            FontWeight::SEMIBOLD
                        } else {
                            FontWeight::MEDIUM
                        })
                        .text_color(if is_active {
                            theme.text.primary
                        } else {
                            theme.text.secondary
                        })
                        .child(tab.label),
                )
                .when_some(tab.badge, |el, badge| {
                    el.child(
                        apply_text_step(div(), theme.typography.size_3xs)
                            .px_1p5()
                            .py(px(1.0))
                            .rounded(theme.style.radius_control)
                            .bg(theme.surface.control_bg)
                            .text_color(theme.text.secondary)
                            .font_weight(FontWeight::SEMIBOLD)
                            .child(badge),
                    )
                })
        }));

    let on_start_resize = props.on_start_resize;
    let resize_handle = div()
        .id("resource-drawer-resize-handle")
        .absolute()
        .top_0()
        .bottom_0()
        .left(px(-3.0))
        .w(px(6.0))
        .cursor(CursorStyle::ResizeLeftRight)
        .hover(|s| s.bg(theme.accent.soft))
        .on_mouse_down(MouseButton::Left, move |_, window, cx| {
            cx.stop_propagation();
            on_start_resize(window, cx);
        });

    let mut container = div()
        .id("resource-drawer-shell")
        .relative()
        .h_full()
        .w(px(drawer_width))
        .flex_shrink_0()
        .bg(theme.surface.bg_card)
        .border_l_1()
        .border_color(theme.border.strong)
        .flex()
        .flex_col()
        .on_mouse_down(MouseButton::Left, |_, _, cx| {
            cx.stop_propagation();
        })
        .on_mouse_down(MouseButton::Right, |_, _, cx| {
            cx.stop_propagation();
        })
        .child(resize_handle)
        .child(header)
        .child(tab_bar)
        .child(
            div()
                .flex_1()
                .min_h_0()
                .w_full()
                .flex()
                .flex_col()
                .child(props.body),
        );

    if !theme.style.overlay_shadows.is_empty() {
        container = container.shadow(theme.style.overlay_shadows.clone());
    }

    Some(container.into_any_element())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resource_drawer_open_switch_and_close_transitions() {
        let mut state = ResourceDrawerState::default();
        let dep_a = ResourceDrawerTarget::Deployment {
            name: "api".to_string(),
            namespace: "default".to_string(),
        };
        let dep_b = ResourceDrawerTarget::Deployment {
            name: "web".to_string(),
            namespace: "prod".to_string(),
        };

        let cmd = state.open_or_switch(
            Some("cluster-1"),
            dep_a.clone(),
            ResourceDrawerTabId::Overview,
        );
        assert!(cmd.is_none());
        assert!(state.is_open);
        assert_eq!(state.active_tab, ResourceDrawerTabId::Overview);
        assert_eq!(state.yaml_status, YamlLoadStatus::Idle);

        let cmd = state.select_tab(Some("cluster-1"), ResourceDrawerTabId::Yaml);
        match cmd {
            Some(ClusterBridgeCommand::LoadResourceYaml {
                cluster_id,
                generation,
                kind,
                namespace,
                name,
            }) => {
                assert_eq!(cluster_id, "cluster-1");
                assert_eq!(generation, 2);
                assert_eq!(kind, "Deployment");
                assert_eq!(namespace.as_deref(), Some("default"));
                assert_eq!(name, "api");
            }
            other => panic!("expected LoadResourceYaml, got {other:?}"),
        }
        let gen_a = state.yaml_generation;
        assert!(state.apply_yaml_loaded(
            "cluster-1",
            gen_a,
            "Deployment",
            Some("default"),
            "api",
            "kind: Deployment".to_string(),
        ));
        assert_eq!(state.yaml_status, YamlLoadStatus::Loaded);

        let cmd_b = state.open_or_switch(
            Some("cluster-1"),
            dep_b.clone(),
            ResourceDrawerTabId::Overview,
        );
        assert!(cmd_b.is_none());
        assert!(state.is_open);
        assert_eq!(state.target, Some(dep_b));
        assert!(state.yaml_generation > gen_a);
        assert_eq!(state.yaml_status, YamlLoadStatus::Idle);
        assert!(state.yaml_content.is_none());

        state.width = 720.0;
        state.close();
        assert!(!state.is_open);
        assert_eq!(state.width, 720.0);
    }

    #[test]
    fn test_resource_drawer_rejects_stale_yaml_responses() {
        let mut state = ResourceDrawerState::default();
        let dep_a = ResourceDrawerTarget::Deployment {
            name: "api".to_string(),
            namespace: "default".to_string(),
        };
        let dep_b = ResourceDrawerTarget::Deployment {
            name: "web".to_string(),
            namespace: "default".to_string(),
        };

        let _ = state.open_or_switch(Some("c1"), dep_a, ResourceDrawerTabId::Yaml);
        let gen_a = state.yaml_generation;

        let _ = state.open_or_switch(Some("c1"), dep_b, ResourceDrawerTabId::Yaml);
        let gen_b = state.yaml_generation;
        assert!(gen_b > gen_a);

        let stale_applied = state.apply_yaml_loaded(
            "c1",
            gen_a,
            "Deployment",
            Some("default"),
            "api",
            "stale".to_string(),
        );
        assert!(!stale_applied);
        assert_ne!(state.yaml_content.as_deref(), Some("stale"));

        let fresh_applied = state.apply_yaml_loaded(
            "c1",
            gen_b,
            "Deployment",
            Some("default"),
            "web",
            "fresh-yaml".to_string(),
        );
        assert!(fresh_applied);
        assert_eq!(state.yaml_content.as_deref(), Some("fresh-yaml"));
    }

    #[test]
    fn test_resource_drawer_resize_and_responsive_viewport_clamping() {
        let mut state = ResourceDrawerState::default();
        state.update_resize_from_window_x(1200.0, 1000.0);
        assert_eq!(state.width, MIN_DRAWER_WIDTH);

        state.update_resize_from_window_x(1400.0, 100.0);
        assert_eq!(state.width, MAX_DRAWER_WIDTH);

        assert_eq!(effective_drawer_width(700.0, 800.0), 560.0);
        assert!(800.0 - effective_drawer_width(700.0, 800.0) >= DRAWER_MIN_WORKSPACE_MARGIN);
    }

    #[test]
    fn test_resource_drawer_event_target_skips_bridge_yaml_fetch() {
        let mut state = ResourceDrawerState::default();
        let event = ResourceDrawerTarget::Event {
            uid: "evt-1".to_string(),
            reason: "Scheduled".to_string(),
            namespace: "default".to_string(),
        };

        let cmd = state.open_or_switch(Some("c1"), event, ResourceDrawerTabId::Yaml);
        assert!(cmd.is_none());
        assert!(state.is_open);
    }

    #[test]
    fn test_pod_drawer_target_metadata_and_yaml_support() {
        let pod = ResourceDrawerTarget::Pod {
            name: "api-0".to_string(),
            namespace: "prod".to_string(),
        };
        assert_eq!(pod.kind(), "Pod");
        assert_eq!(pod.name(), "api-0");
        assert_eq!(pod.display_title(), "api-0");
        assert_eq!(pod.namespace(), Some("prod"));
        assert!(pod.supports_live_yaml());
    }

    #[test]
    fn test_service_drawer_target_metadata_and_yaml_support() {
        let svc = ResourceDrawerTarget::Service {
            name: "api-svc".to_string(),
            namespace: "prod".to_string(),
        };
        assert_eq!(svc.kind(), "Service");
        assert_eq!(svc.name(), "api-svc");
        assert_eq!(svc.display_title(), "api-svc");
        assert_eq!(svc.namespace(), Some("prod"));
        assert!(svc.supports_live_yaml());
    }

    #[test]
    fn test_resource_drawer_target_configmap_metadata_and_yaml_command() {
        let target = ResourceDrawerTarget::ConfigMap {
            name: "app-config".to_string(),
            namespace: "prod".to_string(),
        };
        assert_eq!(target.kind(), "ConfigMap");
        assert_eq!(target.name(), "app-config");
        assert_eq!(target.display_title(), "app-config");
        assert_eq!(target.namespace(), Some("prod"));
        assert!(target.supports_live_yaml());

        let mut state = ResourceDrawerState::default();
        let cmd = state.open_or_switch(Some("ctx-a"), target, ResourceDrawerTabId::Yaml);
        assert_eq!(
            cmd,
            Some(ClusterBridgeCommand::LoadResourceYaml {
                cluster_id: "ctx-a".to_string(),
                generation: 2,
                kind: "ConfigMap".to_string(),
                namespace: Some("prod".to_string()),
                name: "app-config".to_string(),
            })
        );
    }
}
