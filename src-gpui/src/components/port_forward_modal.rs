use std::collections::BTreeSet;
use std::rc::Rc;

use gpui::{
    AnyElement, App, FontWeight, MouseButton, SharedString, Window, div, prelude::*, px,
};
use lucide_icons::Icon;
use orbit_core::kubernetes::models::{
    DeploymentInfo, PodInfo, ReplicaSetInfo, ServiceInfo, StatefulSetInfo,
};

use crate::icons::render_icon;
use crate::theme::OrbitTheme;

pub const PORT_FORWARD_KINDS: &[&str] = &[
    "Service",
    "Pod",
    "Deployment",
    "StatefulSet",
    "ReplicaSet",
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PortForwardResourceOption {
    pub namespace: String,
    pub kind: String,
    pub name: String,
    pub ports: Vec<u16>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PortForwardModalMode {
    Targeted {
        detected_ports: Vec<u16>,
    },
    Create {
        available_namespaces: Vec<String>,
        available_resources: Vec<PortForwardResourceOption>,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PortForwardModalField {
    ResourceName,
    LocalPort,
    RemotePort,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PortForwardModalKeyResult {
    Ignored,
    Handled,
    Cancel,
    Submit,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PortForwardModalState {
    pub cluster_id: String,
    pub cluster_name: String,
    pub mode: PortForwardModalMode,
    pub namespace: String,
    pub kind: String,
    pub name: String,
    pub local_port_input: String,
    pub remote_port_input: String,
    pub active_field: PortForwardModalField,
    pub active_local_ports: Vec<u16>,
    pub submitting: bool,
    pub error: Option<String>,
}

impl PortForwardModalState {
    pub fn new_targeted(
        cluster_id: impl Into<String>,
        cluster_name: impl Into<String>,
        namespace: impl Into<String>,
        kind: impl Into<String>,
        name: impl Into<String>,
        detected_ports: Vec<u16>,
        active_local_ports: Vec<u16>,
    ) -> Self {
        let initial_port = detected_ports.first().copied().unwrap_or(8080);
        Self {
            cluster_id: cluster_id.into(),
            cluster_name: cluster_name.into(),
            mode: PortForwardModalMode::Targeted { detected_ports },
            namespace: namespace.into(),
            kind: kind.into(),
            name: name.into(),
            local_port_input: initial_port.to_string(),
            remote_port_input: initial_port.to_string(),
            active_field: PortForwardModalField::LocalPort,
            active_local_ports,
            submitting: false,
            error: None,
        }
    }

    pub fn new_create(
        cluster_id: impl Into<String>,
        cluster_name: impl Into<String>,
        mut available_namespaces: Vec<String>,
        available_resources: Vec<PortForwardResourceOption>,
        active_local_ports: Vec<u16>,
    ) -> Self {
        if available_namespaces.is_empty() {
            available_namespaces.push("default".to_string());
        }
        let initial_namespace = available_namespaces
            .first()
            .cloned()
            .unwrap_or_else(|| "default".to_string());
        let initial_kind = "Service".to_string();
        let matching_resource = available_resources
            .iter()
            .find(|r| r.namespace == initial_namespace && r.kind.eq_ignore_ascii_case(&initial_kind))
            .or_else(|| {
                available_resources
                    .iter()
                    .find(|r| r.kind.eq_ignore_ascii_case(&initial_kind))
            })
            .cloned();

        let (name, initial_port) = if let Some(res) = matching_resource {
            let port = res.ports.first().copied().unwrap_or(8080);
            (res.name, port)
        } else {
            (String::new(), 8080)
        };

        Self {
            cluster_id: cluster_id.into(),
            cluster_name: cluster_name.into(),
            mode: PortForwardModalMode::Create {
                available_namespaces,
                available_resources,
            },
            namespace: initial_namespace,
            kind: initial_kind,
            name,
            local_port_input: initial_port.to_string(),
            remote_port_input: initial_port.to_string(),
            active_field: PortForwardModalField::LocalPort,
            active_local_ports,
            submitting: false,
            error: None,
        }
    }

    pub fn parsed_local_port(&self) -> Option<u16> {
        parse_valid_port(&self.local_port_input)
    }

    pub fn parsed_remote_port(&self) -> Option<u16> {
        parse_valid_port(&self.remote_port_input)
    }

    pub fn is_local_port_in_use(&self) -> bool {
        self.parsed_local_port()
            .is_some_and(|port| self.active_local_ports.contains(&port))
    }

    pub fn can_submit(&self) -> bool {
        !self.submitting
            && !self.namespace.trim().is_empty()
            && !self.name.trim().is_empty()
            && self.parsed_local_port().is_some()
            && self.parsed_remote_port().is_some()
            && !self.is_local_port_in_use()
    }

    pub fn detected_ports(&self) -> Vec<u16> {
        match &self.mode {
            PortForwardModalMode::Targeted { detected_ports } => detected_ports.clone(),
            PortForwardModalMode::Create {
                available_resources,
                ..
            } => available_resources
                .iter()
                .find(|r| {
                    r.namespace == self.namespace
                        && r.kind.eq_ignore_ascii_case(&self.kind)
                        && r.name == self.name
                })
                .map(|r| r.ports.clone())
                .unwrap_or_default(),
        }
    }

    pub fn matching_resources(&self) -> Vec<PortForwardResourceOption> {
        match &self.mode {
            PortForwardModalMode::Targeted { .. } => Vec::new(),
            PortForwardModalMode::Create {
                available_resources,
                ..
            } => available_resources
                .iter()
                .filter(|r| r.namespace == self.namespace && r.kind.eq_ignore_ascii_case(&self.kind))
                .cloned()
                .collect(),
        }
    }

    pub fn select_detected_port(&mut self, port: u16) {
        if self.submitting {
            return;
        }
        self.remote_port_input = port.to_string();
        self.local_port_input = port.to_string();
        self.error = None;
    }

    pub fn set_kind(&mut self, kind: &str) {
        if self.submitting {
            return;
        }
        self.kind = kind.to_string();
        self.sync_selected_resource_for_namespace_and_kind();
        self.error = None;
    }

    pub fn set_namespace(&mut self, namespace: &str) {
        if self.submitting {
            return;
        }
        self.namespace = namespace.to_string();
        self.sync_selected_resource_for_namespace_and_kind();
        self.error = None;
    }

    pub fn select_resource(&mut self, name: &str) {
        if self.submitting {
            return;
        }
        self.name = name.to_string();
        let ports = self.detected_ports();
        if let Some(first) = ports.first().copied() {
            self.local_port_input = first.to_string();
            self.remote_port_input = first.to_string();
        }
        self.error = None;
    }

    fn sync_selected_resource_for_namespace_and_kind(&mut self) {
        let matches = self.matching_resources();
        if let Some(first) = matches.first() {
            self.name = first.name.clone();
            if let Some(port) = first.ports.first().copied() {
                self.local_port_input = port.to_string();
                self.remote_port_input = port.to_string();
            }
        } else {
            self.name.clear();
        }
    }

    pub fn cycle_field(&mut self, reverse: bool) {
        let is_create = matches!(self.mode, PortForwardModalMode::Create { .. });
        self.active_field = match (self.active_field, is_create, reverse) {
            (PortForwardModalField::ResourceName, _, false) => PortForwardModalField::LocalPort,
            (PortForwardModalField::ResourceName, _, true) => PortForwardModalField::RemotePort,
            (PortForwardModalField::LocalPort, _, false) => PortForwardModalField::RemotePort,
            (PortForwardModalField::LocalPort, true, true) => PortForwardModalField::ResourceName,
            (PortForwardModalField::LocalPort, false, true) => PortForwardModalField::RemotePort,
            (PortForwardModalField::RemotePort, true, false) => PortForwardModalField::ResourceName,
            (PortForwardModalField::RemotePort, false, false) => PortForwardModalField::LocalPort,
            (PortForwardModalField::RemotePort, _, true) => PortForwardModalField::LocalPort,
        };
    }

    pub fn handle_key_input(
        &mut self,
        key: &str,
        key_char: Option<&str>,
    ) -> PortForwardModalKeyResult {
        if key == "escape" {
            return if self.submitting {
                PortForwardModalKeyResult::Handled
            } else {
                PortForwardModalKeyResult::Cancel
            };
        }
        if key == "enter" {
            return if self.can_submit() {
                PortForwardModalKeyResult::Submit
            } else {
                PortForwardModalKeyResult::Handled
            };
        }
        if self.submitting {
            return PortForwardModalKeyResult::Handled;
        }
        if key == "tab" {
            self.cycle_field(false);
            return PortForwardModalKeyResult::Handled;
        }
        if key == "backspace" {
            match self.active_field {
                PortForwardModalField::ResourceName => {
                    self.name.pop();
                }
                PortForwardModalField::LocalPort => {
                    self.local_port_input.pop();
                }
                PortForwardModalField::RemotePort => {
                    self.remote_port_input.pop();
                }
            }
            self.error = None;
            return PortForwardModalKeyResult::Handled;
        }
        if let Some(ch_str) = key_char {
            let mut chars = ch_str.chars();
            if let (Some(ch), None) = (chars.next(), chars.next()) {
                match self.active_field {
                    PortForwardModalField::ResourceName => {
                        if !ch.is_control() && self.name.len() < 128 {
                            self.name.push(ch);
                            self.error = None;
                        }
                    }
                    PortForwardModalField::LocalPort => {
                        if ch.is_ascii_digit() && self.local_port_input.len() < 5 {
                            self.local_port_input.push(ch);
                            self.error = None;
                        }
                    }
                    PortForwardModalField::RemotePort => {
                        if ch.is_ascii_digit() && self.remote_port_input.len() < 5 {
                            self.remote_port_input.push(ch);
                            self.error = None;
                        }
                    }
                }
            }
        }
        PortForwardModalKeyResult::Handled
    }
}

fn parse_valid_port(raw: &str) -> Option<u16> {
    let trimmed = raw.trim();
    if trimmed.is_empty() || !trimmed.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    let port = trimmed.parse::<u16>().ok()?;
    if port >= 1 { Some(port) } else { None }
}

pub fn extract_ports_from_port_str(raw: &str) -> Vec<u16> {
    let mut ports = BTreeSet::new();
    for segment in raw.split([',', '\n', ' ']) {
        let token = segment.trim();
        if token.is_empty() || token == "-" {
            continue;
        }
        let port_part = token.split('/').next().unwrap_or("");
        let primary = port_part.split(':').next().unwrap_or("");
        if let Ok(port) = primary.trim().parse::<u16>()
            && port >= 1
        {
            ports.insert(port);
        }
    }
    ports.into_iter().collect()
}

pub fn extract_ports_from_service(service: &ServiceInfo) -> Vec<u16> {
    let mut ports = BTreeSet::new();
    for sp in &service.ports_list {
        if sp.port > 0
            && let Ok(port) = u16::try_from(sp.port)
        {
            ports.insert(port);
        }
    }
    if ports.is_empty() {
        return extract_ports_from_port_str(&service.ports);
    }
    ports.into_iter().collect()
}

pub fn extract_ports_from_pod(pod: &PodInfo) -> Vec<u16> {
    let mut ports = BTreeSet::new();
    for container in &pod.containers {
        if let Some(ref raw_ports) = container.ports {
            for port in extract_ports_from_port_str(raw_ports) {
                ports.insert(port);
            }
        }
    }
    ports.into_iter().collect()
}

pub fn extract_ports_from_deployment(_deployment: &DeploymentInfo) -> Vec<u16> {
    Vec::new()
}

pub fn extract_ports_from_statefulset(_statefulset: &StatefulSetInfo) -> Vec<u16> {
    Vec::new()
}

pub fn extract_ports_from_replicaset(_replicaset: &ReplicaSetInfo) -> Vec<u16> {
    Vec::new()
}

pub type PortForwardActionCallback = Rc<dyn Fn(&mut Window, &mut App)>;
pub type PortForwardValueCallback<T> = Rc<dyn Fn(T, &mut Window, &mut App)>;

pub struct PortForwardModalCallbacks {
    pub on_close: PortForwardActionCallback,
    pub on_submit: PortForwardActionCallback,
    pub on_focus_field: PortForwardValueCallback<PortForwardModalField>,
    pub on_select_detected_port: PortForwardValueCallback<u16>,
    pub on_select_kind: PortForwardValueCallback<String>,
    pub on_select_namespace: PortForwardValueCallback<String>,
    pub on_select_resource: PortForwardValueCallback<String>,
}

pub fn render_port_forward_modal(
    state: &PortForwardModalState,
    theme: &OrbitTheme,
    callbacks: &PortForwardModalCallbacks,
) -> AnyElement {
    let on_close_backdrop = callbacks.on_close.clone();
    let on_close_btn = callbacks.on_close.clone();
    let on_cancel_btn = callbacks.on_close.clone();
    let on_submit_btn = callbacks.on_submit.clone();

    let can_submit = state.can_submit();
    let local_invalid = state.parsed_local_port().is_none();
    let remote_invalid = state.parsed_remote_port().is_none();
    let local_collision = state.is_local_port_in_use();
    let detected_ports = state.detected_ports();

    let mut body = div().px_5().py_4().flex().flex_col().gap_3();

    match &state.mode {
        PortForwardModalMode::Targeted { .. } => {
            body = body.child(render_target_summary(
                &state.cluster_name,
                &state.kind,
                &state.namespace,
                &state.name,
                theme,
            ));
        }
        PortForwardModalMode::Create {
            available_namespaces,
            ..
        } => {
            let matching_resources = state.matching_resources();
            body = body
                .child(render_kind_selector(&state.kind, theme, &callbacks.on_select_kind))
                .child(render_namespace_selector(
                    available_namespaces,
                    &state.namespace,
                    theme,
                    &callbacks.on_select_namespace,
                ))
                .child(render_resource_selector(
                    state,
                    &matching_resources,
                    theme,
                    callbacks,
                ));
        }
    }

    if !detected_ports.is_empty() {
        body = body.child(render_detected_ports_chips(
            &detected_ports,
            state.parsed_remote_port(),
            theme,
            &callbacks.on_select_detected_port,
        ));
    }

    body = body
        .child(
            div()
                .grid()
                .grid_cols(2)
                .gap_3()
                .child(render_port_input(
                    PortInputProps {
                        id: "pf-local-port-input",
                        label: "Local Port",
                        value: &state.local_port_input,
                        focused: state.active_field == PortForwardModalField::LocalPort,
                        invalid: local_invalid || local_collision,
                        field: PortForwardModalField::LocalPort,
                    },
                    theme,
                    &callbacks.on_focus_field,
                ))
                .child(render_port_input(
                    PortInputProps {
                        id: "pf-remote-port-input",
                        label: "Remote Port",
                        value: &state.remote_port_input,
                        focused: state.active_field == PortForwardModalField::RemotePort,
                        invalid: remote_invalid,
                        field: PortForwardModalField::RemotePort,
                    },
                    theme,
                    &callbacks.on_focus_field,
                )),
        )
        .when(local_invalid || remote_invalid, |el| {
            el.child(
                div()
                    .text_xs()
                    .text_color(theme.semantic.danger.solid)
                    .child("Enter valid port numbers between 1 and 65535."),
            )
        })
        .when(local_collision, |el| {
            el.child(
                div()
                    .px_3()
                    .py_2()
                    .rounded(theme.style.radius_control)
                    .bg(theme.semantic.warning.soft)
                    .border_1()
                    .border_color(theme.semantic.warning.solid)
                    .flex()
                    .items_center()
                    .gap_2()
                    .child(render_icon(
                        Icon::AlertTriangle,
                        px(14.0),
                        theme.semantic.warning.solid,
                    ))
                    .child(
                        div()
                            .flex_1()
                            .text_xs()
                            .text_color(theme.text.primary)
                            .child(format!(
                                "Local port {} is already in use by another active tunnel.",
                                state.local_port_input.trim()
                            )),
                    ),
            )
        })
        .when_some(state.error.as_ref(), |el, err| {
            el.child(
                div()
                    .px_3()
                    .py_2()
                    .rounded(theme.style.radius_control)
                    .bg(theme.semantic.danger.soft)
                    .border_1()
                    .border_color(theme.semantic.danger.solid)
                    .text_xs()
                    .text_color(theme.semantic.danger.solid)
                    .child(err.clone()),
            )
        });

    let card = theme
        .style_card(div())
        .id("port-forward-modal-card")
        .w(px(460.0))
        .bg(theme.surface.bg_card)
        .border_1()
        .border_color(theme.border.strong)
        .shadow_lg()
        .flex()
        .flex_col()
        .overflow_hidden()
        .on_mouse_down(MouseButton::Left, |_, window, cx| {
            window.prevent_default();
            cx.stop_propagation();
        })
        .child(
            div()
                .px_5()
                .py_3p5()
                .border_b_1()
                .border_color(theme.border.default)
                .flex()
                .items_center()
                .justify_between()
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap_2()
                        .child(render_icon(Icon::Cable, px(16.0), theme.accent.accent))
                        .child(
                            div()
                                .text_sm()
                                .font_weight(FontWeight::SEMIBOLD)
                                .text_color(theme.text.primary)
                                .child("Port Forward"),
                        ),
                )
                .child(
                    div()
                        .id("port-forward-modal-close")
                        .w(px(24.0))
                        .h(px(24.0))
                        .rounded(theme.style.radius_control)
                        .flex()
                        .items_center()
                        .justify_center()
                        .cursor_pointer()
                        .hover(|s| s.bg(theme.surface.bg_hover))
                        .on_mouse_down(MouseButton::Left, move |_, window, cx| {
                            window.prevent_default();
                            cx.stop_propagation();
                            on_close_btn(window, cx);
                        })
                        .child(render_icon(Icon::X, px(14.0), theme.text.muted)),
                ),
        )
        .child(body)
        .child(
            div()
                .px_5()
                .py_3()
                .border_t_1()
                .border_color(theme.border.default)
                .bg(theme.surface.bg_panel)
                .flex()
                .items_center()
                .justify_end()
                .gap_2()
                .child(
                    div()
                        .id("port-forward-modal-cancel")
                        .px_3()
                        .py_1p5()
                        .rounded(theme.style.radius_control)
                        .border_1()
                        .border_color(theme.border.default)
                        .bg(theme.surface.bg_card)
                        .text_xs()
                        .font_weight(FontWeight::MEDIUM)
                        .text_color(theme.text.primary)
                        .cursor_pointer()
                        .hover(|s| s.bg(theme.surface.bg_hover))
                        .on_mouse_down(MouseButton::Left, move |_, window, cx| {
                            window.prevent_default();
                            cx.stop_propagation();
                            on_cancel_btn(window, cx);
                        })
                        .child("Cancel"),
                )
                .child(
                    div()
                        .id("port-forward-modal-submit")
                        .px_3p5()
                        .py_1p5()
                        .rounded(theme.style.radius_control)
                        .bg(if can_submit {
                            theme.accent.accent
                        } else {
                            theme.surface.bg_hover
                        })
                        .text_xs()
                        .font_weight(FontWeight::SEMIBOLD)
                        .text_color(if can_submit {
                            gpui::white().into()
                        } else {
                            theme.text.muted
                        })
                        .when(can_submit, |el| {
                            el.cursor_pointer()
                                .hover(|s| s.bg(theme.accent.hover))
                                .on_mouse_down(MouseButton::Left, move |_, window, cx| {
                                    window.prevent_default();
                                    cx.stop_propagation();
                                    on_submit_btn(window, cx);
                                })
                        })
                        .child(if state.submitting {
                            "Starting..."
                        } else {
                            "Start Forward"
                        }),
                ),
        );

    div()
        .id("port-forward-modal-backdrop")
        .absolute()
        .inset_0()
        .bg(theme.surface.overlay_bg)
        .flex()
        .items_center()
        .justify_center()
        .on_mouse_down(MouseButton::Left, move |_, window, cx| {
            window.prevent_default();
            cx.stop_propagation();
            on_close_backdrop(window, cx);
        })
        .child(card)
        .into_any_element()
}

fn render_target_summary(
    cluster_name: &str,
    kind: &str,
    namespace: &str,
    name: &str,
    theme: &OrbitTheme,
) -> AnyElement {
    div()
        .px_3()
        .py_2p5()
        .rounded(theme.style.radius_control)
        .bg(theme.surface.bg_panel)
        .border_1()
        .border_color(theme.border.default)
        .flex()
        .flex_col()
        .gap_1()
        .child(
            div()
                .flex()
                .items_center()
                .justify_between()
                .child(
                    div()
                        .text_xs()
                        .font_weight(FontWeight::SEMIBOLD)
                        .text_color(theme.text.primary)
                        .child(format!("{kind}/{name}")),
                )
                .child(
                    div()
                        .text_xs()
                        .text_color(theme.text.muted)
                        .child(cluster_name.to_string()),
                ),
        )
        .child(
            div()
                .text_xs()
                .text_color(theme.text.secondary)
                .child(format!("Namespace: {namespace}")),
        )
        .into_any_element()
}

fn render_kind_selector(
    active_kind: &str,
    theme: &OrbitTheme,
    on_select_kind: &PortForwardValueCallback<String>,
) -> AnyElement {
    div()
        .flex()
        .flex_col()
        .gap_1p5()
        .child(
            div()
                .text_xs()
                .font_weight(FontWeight::MEDIUM)
                .text_color(theme.text.secondary)
                .child("Resource Kind"),
        )
        .child(
            div()
                .flex()
                .flex_wrap()
                .gap_1p5()
                .children(PORT_FORWARD_KINDS.iter().map(|&kind| {
                    let is_active = active_kind.eq_ignore_ascii_case(kind);
                    let cb = on_select_kind.clone();
                    let kind_owned = kind.to_string();
                    div()
                        .id(SharedString::from(format!("pf-kind-{kind}")))
                        .px_2p5()
                        .py_1()
                        .rounded(theme.style.radius_control)
                        .border_1()
                        .border_color(if is_active {
                            theme.accent.accent
                        } else {
                            theme.border.default
                        })
                        .bg(if is_active {
                            theme.accent.soft
                        } else {
                            theme.surface.bg_panel
                        })
                        .text_xs()
                        .font_weight(FontWeight::MEDIUM)
                        .text_color(if is_active {
                            theme.accent.accent
                        } else {
                            theme.text.secondary
                        })
                        .cursor_pointer()
                        .on_mouse_down(MouseButton::Left, move |_, window, cx| {
                            window.prevent_default();
                            cx.stop_propagation();
                            cb(kind_owned.clone(), window, cx);
                        })
                        .child(kind)
                })),
        )
        .into_any_element()
}

fn render_namespace_selector(
    namespaces: &[String],
    active_ns: &str,
    theme: &OrbitTheme,
    on_select_namespace: &PortForwardValueCallback<String>,
) -> AnyElement {
    div()
        .flex()
        .flex_col()
        .gap_1p5()
        .child(
            div()
                .text_xs()
                .font_weight(FontWeight::MEDIUM)
                .text_color(theme.text.secondary)
                .child("Namespace"),
        )
        .child(
            div()
                .flex()
                .flex_wrap()
                .gap_1p5()
                .max_h(px(84.0))
                .overflow_hidden()
                .children(namespaces.iter().take(12).map(|ns| {
                    let is_active = ns == active_ns;
                    let cb = on_select_namespace.clone();
                    let ns_owned = ns.clone();
                    div()
                        .id(SharedString::from(format!("pf-ns-{ns}")))
                        .px_2()
                        .py_0p5()
                        .rounded(theme.style.radius_control)
                        .border_1()
                        .border_color(if is_active {
                            theme.accent.accent
                        } else {
                            theme.border.default
                        })
                        .bg(if is_active {
                            theme.accent.soft
                        } else {
                            theme.surface.bg_panel
                        })
                        .text_xs()
                        .text_color(if is_active {
                            theme.accent.accent
                        } else {
                            theme.text.secondary
                        })
                        .cursor_pointer()
                        .on_mouse_down(MouseButton::Left, move |_, window, cx| {
                            window.prevent_default();
                            cx.stop_propagation();
                            cb(ns_owned.clone(), window, cx);
                        })
                        .child(ns.clone())
                })),
        )
        .into_any_element()
}

fn render_resource_selector(
    state: &PortForwardModalState,
    matching_resources: &[PortForwardResourceOption],
    theme: &OrbitTheme,
    callbacks: &PortForwardModalCallbacks,
) -> AnyElement {
    let on_focus = callbacks.on_focus_field.clone();
    let is_focused = state.active_field == PortForwardModalField::ResourceName;

    div()
        .flex()
        .flex_col()
        .gap_1p5()
        .child(
            div()
                .text_xs()
                .font_weight(FontWeight::MEDIUM)
                .text_color(theme.text.secondary)
                .child("Resource Name"),
        )
        .child(
            div()
                .id("pf-resource-name-input")
                .h(px(30.0))
                .px_2p5()
                .rounded(theme.style.radius_control)
                .bg(theme.surface.input_bg)
                .border_1()
                .border_color(if is_focused {
                    theme.accent.accent
                } else {
                    theme.border.input
                })
                .flex()
                .items_center()
                .text_xs()
                .text_color(if state.name.is_empty() {
                    theme.text.muted
                } else {
                    theme.text.primary
                })
                .cursor_pointer()
                .on_mouse_down(MouseButton::Left, move |_, window, cx| {
                    window.prevent_default();
                    cx.stop_propagation();
                    on_focus(PortForwardModalField::ResourceName, window, cx);
                })
                .child(if state.name.is_empty() {
                    SharedString::from("Type or select a resource...")
                } else {
                    SharedString::from(state.name.clone())
                }),
        )
        .when(!matching_resources.is_empty(), |el| {
            el.child(
                div()
                    .flex()
                    .flex_wrap()
                    .gap_1()
                    .max_h(px(90.0))
                    .overflow_hidden()
                    .children(matching_resources.iter().take(12).map(|res| {
                        let is_selected = res.name == state.name;
                        let cb = callbacks.on_select_resource.clone();
                        let name_owned = res.name.clone();
                        div()
                            .id(SharedString::from(format!("pf-res-{}", res.name)))
                            .px_2()
                            .py_0p5()
                            .rounded(theme.style.radius_control)
                            .border_1()
                            .border_color(if is_selected {
                                theme.accent.accent
                            } else {
                                theme.border.default
                            })
                            .bg(if is_selected {
                                theme.accent.soft
                            } else {
                                theme.surface.bg_panel
                            })
                            .text_xs()
                            .text_color(if is_selected {
                                theme.accent.accent
                            } else {
                                theme.text.secondary
                            })
                            .cursor_pointer()
                            .on_mouse_down(MouseButton::Left, move |_, window, cx| {
                                window.prevent_default();
                                cx.stop_propagation();
                                cb(name_owned.clone(), window, cx);
                            })
                            .child(res.name.clone())
                    })),
            )
        })
        .into_any_element()
}

fn render_detected_ports_chips(
    ports: &[u16],
    active_remote: Option<u16>,
    theme: &OrbitTheme,
    on_select_port: &PortForwardValueCallback<u16>,
) -> AnyElement {
    div()
        .flex()
        .flex_col()
        .gap_1p5()
        .child(
            div()
                .text_xs()
                .font_weight(FontWeight::MEDIUM)
                .text_color(theme.text.secondary)
                .child("Detected Ports"),
        )
        .child(div().flex().flex_wrap().gap_1p5().children(ports.iter().map(
            |&port| {
                let is_active = active_remote == Some(port);
                let cb = on_select_port.clone();
                div()
                    .id(SharedString::from(format!("pf-detected-port-{port}")))
                    .px_2()
                    .py_0p5()
                    .rounded(theme.style.radius_control)
                    .border_1()
                    .border_color(if is_active {
                        theme.accent.accent
                    } else {
                        theme.border.default
                    })
                    .bg(if is_active {
                        theme.accent.soft
                    } else {
                        theme.surface.bg_panel
                    })
                    .text_xs()
                    .font_weight(FontWeight::MEDIUM)
                    .text_color(if is_active {
                        theme.accent.accent
                    } else {
                        theme.text.primary
                    })
                    .cursor_pointer()
                    .on_mouse_down(MouseButton::Left, move |_, window, cx| {
                        window.prevent_default();
                        cx.stop_propagation();
                        cb(port, window, cx);
                    })
                    .child(format!(":{port}"))
            },
        )))
        .into_any_element()
}

struct PortInputProps<'a> {
    id: &'static str,
    label: &'static str,
    value: &'a str,
    focused: bool,
    invalid: bool,
    field: PortForwardModalField,
}

fn render_port_input(
    props: PortInputProps<'_>,
    theme: &OrbitTheme,
    on_focus_field: &PortForwardValueCallback<PortForwardModalField>,
) -> AnyElement {
    let cb = on_focus_field.clone();
    let field = props.field;
    div()
        .flex()
        .flex_col()
        .gap_1p5()
        .child(
            div()
                .text_xs()
                .font_weight(FontWeight::MEDIUM)
                .text_color(theme.text.secondary)
                .child(props.label),
        )
        .child(
            div()
                .id(props.id)
                .h(px(32.0))
                .px_2p5()
                .rounded(theme.style.radius_control)
                .bg(theme.surface.input_bg)
                .border_1()
                .border_color(if props.invalid {
                    theme.semantic.danger.solid
                } else if props.focused {
                    theme.accent.accent
                } else {
                    theme.border.input
                })
                .flex()
                .items_center()
                .text_sm()
                .font_weight(FontWeight::SEMIBOLD)
                .text_color(theme.text.primary)
                .cursor_pointer()
                .on_mouse_down(MouseButton::Left, move |_, window, cx| {
                    window.prevent_default();
                    cx.stop_propagation();
                    cb(field, window, cx);
                })
                .child(if props.value.is_empty() {
                    SharedString::from(" ")
                } else {
                    SharedString::from(props.value.to_string())
                }),
        )
        .into_any_element()
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use orbit_core::kubernetes::models::PodContainer;

    use super::*;

    #[test]
    fn test_targeted_modal_prefills_first_detected_port_and_validates_range() {
        let mut modal = PortForwardModalState::new_targeted(
            "ctx-a",
            "dev",
            "default",
            "Service",
            "api",
            vec![9000, 9090],
            Vec::new(),
        );
        assert_eq!(modal.parsed_local_port(), Some(9000));
        assert_eq!(modal.parsed_remote_port(), Some(9000));
        assert!(modal.can_submit());

        modal.local_port_input = "0".to_string();
        assert_eq!(modal.parsed_local_port(), None);
        assert!(!modal.can_submit());

        modal.local_port_input = "65536".to_string();
        assert_eq!(modal.parsed_local_port(), None);
        assert!(!modal.can_submit());

        modal.local_port_input = String::new();
        assert_eq!(modal.parsed_local_port(), None);
        assert!(!modal.can_submit());

        modal.local_port_input = "65535".to_string();
        assert_eq!(modal.parsed_local_port(), Some(65535));
        assert!(modal.can_submit());

        let fallback = PortForwardModalState::new_targeted(
            "ctx-a",
            "dev",
            "default",
            "Deployment",
            "web",
            Vec::new(),
            Vec::new(),
        );
        assert_eq!(fallback.parsed_local_port(), Some(8080));
        assert_eq!(fallback.parsed_remote_port(), Some(8080));
    }

    #[test]
    fn test_modal_rejects_local_port_in_use_collision() {
        let mut modal = PortForwardModalState::new_targeted(
            "ctx-a",
            "dev",
            "default",
            "Service",
            "api",
            vec![8080],
            vec![8080],
        );
        assert!(modal.is_local_port_in_use());
        assert!(!modal.can_submit());

        modal.local_port_input = "8081".to_string();
        assert!(!modal.is_local_port_in_use());
        assert!(modal.can_submit());
    }

    #[test]
    fn test_create_modal_switches_kind_and_namespace_and_updates_resource_selection() {
        let resources = vec![
            PortForwardResourceOption {
                namespace: "default".to_string(),
                kind: "Service".to_string(),
                name: "api-svc".to_string(),
                ports: vec![8080],
            },
            PortForwardResourceOption {
                namespace: "prod".to_string(),
                kind: "Pod".to_string(),
                name: "worker-0".to_string(),
                ports: vec![9090],
            },
        ];
        let mut modal = PortForwardModalState::new_create(
            "ctx-a",
            "dev",
            vec!["default".to_string(), "prod".to_string()],
            resources,
            Vec::new(),
        );
        assert_eq!(modal.kind, "Service");
        assert_eq!(modal.namespace, "default");
        assert_eq!(modal.name, "api-svc");

        modal.set_namespace("prod");
        modal.set_kind("Pod");
        assert_eq!(modal.name, "worker-0");
        assert_eq!(modal.parsed_local_port(), Some(9090));
        assert_eq!(modal.parsed_remote_port(), Some(9090));
    }

    #[test]
    fn test_modal_keyboard_input_digits_backspace_tab_enter_escape() {
        let mut modal = PortForwardModalState::new_targeted(
            "ctx-a",
            "dev",
            "default",
            "Pod",
            "web",
            vec![80],
            Vec::new(),
        );
        assert_eq!(modal.active_field, PortForwardModalField::LocalPort);
        assert_eq!(
            modal.handle_key_input("backspace", None),
            PortForwardModalKeyResult::Handled
        );
        assert_eq!(modal.local_port_input, "8");
        assert_eq!(
            modal.handle_key_input("digit", Some("4")),
            PortForwardModalKeyResult::Handled
        );
        assert_eq!(modal.local_port_input, "84");

        assert_eq!(
            modal.handle_key_input("tab", None),
            PortForwardModalKeyResult::Handled
        );
        assert_eq!(modal.active_field, PortForwardModalField::RemotePort);

        assert_eq!(
            modal.handle_key_input("enter", None),
            PortForwardModalKeyResult::Submit
        );
        assert_eq!(
            modal.handle_key_input("escape", None),
            PortForwardModalKeyResult::Cancel
        );
    }

    #[test]
    fn test_extract_ports_from_service_pod_and_workloads() {
        let svc = ServiceInfo {
            name: "api".to_string(),
            namespace: "default".to_string(),
            r#type: "ClusterIP".to_string(),
            cluster_ip: "10.0.0.1".to_string(),
            external_ip: "-".to_string(),
            ports: "80/TCP, 443/TCP".to_string(),
            endpoints: "1".to_string(),
            age: "1d".to_string(),
            session_affinity: "None".to_string(),
            internal_traffic_policy: None,
            created: "2026-01-01".to_string(),
            uid: "uid-1".to_string(),
            selector: BTreeMap::new(),
            labels: BTreeMap::new(),
            ports_list: Vec::new(),
            endpoints_list: Vec::new(),
            events: Vec::new(),
        };
        assert_eq!(extract_ports_from_service(&svc), vec![80, 443]);
        assert_eq!(
            extract_ports_from_port_str("80:30080/TCP, 443/TCP"),
            vec![80, 443]
        );

        let pod = PodInfo {
            name: "web-0".to_string(),
            namespace: "default".to_string(),
            status: "Running".to_string(),
            age: "1d".to_string(),
            cpu: Some("10m".to_string()),
            memory: Some("32Mi".to_string()),
            cpu_cores: None,
            memory_bytes: None,
            cpu_pct: None,
            memory_pct: None,
            node: Some("node-1".to_string()),
            restarts: 0,
            images: vec!["nginx".to_string()],
            labels: BTreeMap::new(),
            annotations: BTreeMap::new(),
            ip: Some("10.244.0.2".to_string()),
            node_ip: None,
            controlled_by: None,
            qos_class: Some("Burstable".to_string()),
            containers: vec![PodContainer {
                name: "app".to_string(),
                image: "nginx".to_string(),
                status: "Running".to_string(),
                ready: "1/1".to_string(),
                restarts: 0,
                ports: Some("80/TCP, 443/TCP".to_string()),
            }],
        };
        assert_eq!(extract_ports_from_pod(&pod), vec![80, 443]);
    }
}
