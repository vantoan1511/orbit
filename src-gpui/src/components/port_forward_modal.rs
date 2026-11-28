use std::collections::BTreeSet;
use std::rc::Rc;

use gpui::component::{
    Sizable as _,
    input::{Input, InputState},
};
use gpui::{
    AnyElement, App, Entity, Focusable as _, FontWeight, MouseButton, SharedString, Window, div,
    prelude::*, px,
};
use lucide_icons::Icon;
use orbit_core::kubernetes::models::{
    DeploymentInfo, PodInfo, ReplicaSetInfo, ServiceInfo, StatefulSetInfo,
};

use crate::icons::render_icon;
use crate::theme::OrbitTheme;

pub const PORT_FORWARD_KINDS: &[&str] =
    &["Service", "Pod", "Deployment", "StatefulSet", "ReplicaSet"];

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
            .find(|r| {
                r.namespace == initial_namespace && r.kind.eq_ignore_ascii_case(&initial_kind)
            })
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
                .filter(|r| {
                    r.namespace == self.namespace && r.kind.eq_ignore_ascii_case(&self.kind)
                })
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
            return match self.active_field {
                PortForwardModalField::ResourceName => {
                    self.name.pop();
                    self.error = None;
                    PortForwardModalKeyResult::Handled
                }
                PortForwardModalField::LocalPort | PortForwardModalField::RemotePort => {
                    PortForwardModalKeyResult::Ignored
                }
            };
        }
        if let Some(ch_str) = key_char {
            let mut chars = ch_str.chars();
            if let (Some(ch), None) = (chars.next(), chars.next()) {
                return match self.active_field {
                    PortForwardModalField::ResourceName => {
                        if !ch.is_control() && self.name.len() < 128 {
                            self.name.push(ch);
                            self.error = None;
                        }
                        PortForwardModalKeyResult::Handled
                    }
                    PortForwardModalField::LocalPort | PortForwardModalField::RemotePort => {
                        PortForwardModalKeyResult::Ignored
                    }
                };
            }
        }
        PortForwardModalKeyResult::Ignored
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

pub struct PortForwardInputs {
    pub local: Entity<InputState>,
    pub remote: Entity<InputState>,
}

impl PortForwardInputs {
    pub fn new(
        local_initial: &str,
        remote_initial: &str,
        window: &mut Window,
        cx: &mut App,
    ) -> Self {
        let local_init = local_initial.to_string();
        let remote_init = remote_initial.to_string();
        let local = cx.new(|cx| {
            InputState::new(window, cx)
                .placeholder("e.g. 8080")
                .default_value(local_init)
        });
        let remote = cx.new(|cx| {
            InputState::new(window, cx)
                .placeholder("e.g. 8080")
                .default_value(remote_init)
        });
        Self { local, remote }
    }

    pub fn sync_from_state(
        &self,
        state: &PortForwardModalState,
        window: &mut Window,
        cx: &mut App,
    ) {
        let local_val = state.local_port_input.clone();
        let remote_val = state.remote_port_input.clone();
        self.local.update(cx, |input, cx| {
            if input.value().as_ref() != local_val.as_str() {
                input.set_value(local_val, window, cx);
            }
        });
        self.remote.update(cx, |input, cx| {
            if input.value().as_ref() != remote_val.as_str() {
                input.set_value(remote_val, window, cx);
            }
        });
    }

    pub fn focus_field(&self, field: PortForwardModalField, window: &mut Window, cx: &mut App) {
        match field {
            PortForwardModalField::LocalPort => {
                self.local.read(cx).focus_handle(cx).focus(window, cx);
            }
            PortForwardModalField::RemotePort => {
                self.remote.read(cx).focus_handle(cx).focus(window, cx);
            }
            PortForwardModalField::ResourceName => {}
        }
    }
}

pub fn render_port_forward_modal(
    state: &PortForwardModalState,
    inputs: &PortForwardInputs,
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
                .child(render_kind_selector(
                    &state.kind,
                    theme,
                    &callbacks.on_select_kind,
                ))
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
                    "port-forward-local-port",
                    "Local Port",
                    &inputs.local,
                    local_invalid || local_collision,
                    state.submitting,
                    theme,
                ))
                .child(render_port_input(
                    "port-forward-remote-port",
                    "Remote Port",
                    &inputs.remote,
                    remote_invalid,
                    state.submitting,
                    theme,
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
        .child(
            div()
                .flex()
                .flex_wrap()
                .gap_1p5()
                .children(ports.iter().map(|&port| {
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
                })),
        )
        .into_any_element()
}

fn render_port_input(
    id: &'static str,
    label: &'static str,
    input_state: &Entity<InputState>,
    invalid: bool,
    disabled: bool,
    theme: &OrbitTheme,
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
                .child(label),
        )
        .child(
            Input::new(input_state)
                .id(id)
                .small()
                .disabled(disabled)
                .when(invalid, |el| el.border_color(theme.semantic.danger.solid)),
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
        // Character editing for port fields is owned by gpui-kit Input (no double-typing).
        assert_eq!(
            modal.handle_key_input("backspace", None),
            PortForwardModalKeyResult::Ignored
        );
        assert_eq!(
            modal.handle_key_input("digit", Some("4")),
            PortForwardModalKeyResult::Ignored
        );
        assert_eq!(modal.local_port_input, "80");

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
    fn test_port_inputs_sync_into_state_validation() {
        let mut modal = PortForwardModalState::new_targeted(
            "ctx-a",
            "dev",
            "default",
            "Service",
            "api",
            vec![8080],
            Vec::new(),
        );
        modal.local_port_input = "abc".to_string();
        assert_eq!(modal.parsed_local_port(), None);
        assert!(!modal.can_submit());

        modal.local_port_input = "8080".to_string();
        assert_eq!(modal.parsed_local_port(), Some(8080));
        assert!(modal.can_submit());
    }

    #[gpui::test]
    fn test_port_forward_local_input_updates_modal_state(cx: &mut gpui::TestAppContext) {
        use gpui::component::input::InputEvent;
        use gpui::test::TestWindowExt as _;

        struct ModalHarness {
            state: PortForwardModalState,
            inputs: PortForwardInputs,
            theme: OrbitTheme,
            _subscriptions: Vec<gpui::Subscription>,
        }

        impl ModalHarness {
            fn new(window: &mut Window, cx: &mut gpui::Context<Self>) -> Self {
                let state = PortForwardModalState::new_targeted(
                    "ctx-a",
                    "dev",
                    "default",
                    "Service",
                    "api",
                    vec![8080, 3000],
                    Vec::new(),
                );
                let inputs = PortForwardInputs::new(
                    &state.local_port_input,
                    &state.remote_port_input,
                    window,
                    cx,
                );
                let local_sub = cx.subscribe_in(
                    &inputs.local,
                    window,
                    |this, input, event: &InputEvent, _window, cx| {
                        if let InputEvent::Change = event {
                            this.state.local_port_input = input.read(cx).value().to_string();
                            cx.notify();
                        }
                    },
                );
                let remote_sub = cx.subscribe_in(
                    &inputs.remote,
                    window,
                    |this, input, event: &InputEvent, _window, cx| {
                        if let InputEvent::Change = event {
                            this.state.remote_port_input = input.read(cx).value().to_string();
                            cx.notify();
                        }
                    },
                );
                let theme = OrbitTheme::resolve(
                    crate::theme::ThemeMode::Dark,
                    crate::theme::UiStyle::Flat,
                    crate::theme::AccentColor::Blue,
                    crate::theme::SemanticColorsPreference::default(),
                    gpui::WindowAppearance::Dark,
                );
                Self {
                    state,
                    inputs,
                    theme,
                    _subscriptions: vec![local_sub, remote_sub],
                }
            }
        }

        impl gpui::Render for ModalHarness {
            fn render(
                &mut self,
                _window: &mut Window,
                _cx: &mut gpui::Context<Self>,
            ) -> impl IntoElement {
                let callbacks = PortForwardModalCallbacks {
                    on_close: Rc::new(|_, _| {}),
                    on_submit: Rc::new(|_, _| {}),
                    on_focus_field: Rc::new(|_, _, _| {}),
                    on_select_detected_port: Rc::new(|_, _, _| {}),
                    on_select_kind: Rc::new(|_, _, _| {}),
                    on_select_namespace: Rc::new(|_, _, _| {}),
                    on_select_resource: Rc::new(|_, _, _| {}),
                };
                render_port_forward_modal(&self.state, &self.inputs, &self.theme, &callbacks)
            }
        }

        let opened = cx.update(|cx| {
            gpui::init(cx);
            let theme = OrbitTheme::resolve(
                crate::theme::ThemeMode::Dark,
                crate::theme::UiStyle::Flat,
                crate::theme::AccentColor::Blue,
                crate::theme::SemanticColorsPreference::default(),
                gpui::WindowAppearance::Dark,
            );
            crate::theme::kit_bridge::sync_kit_theme(&theme, cx);
            gpui::open_window(gpui::WindowOptions::default(), cx, |window, cx| {
                cx.new(|cx| ModalHarness::new(window, cx))
            })
        });
        let Ok((window, view)) = opened else {
            panic!("failed to open test window");
        };

        cx.update(|cx| {
            let _ = window.update(cx, |_, window, cx| {
                window.render_frame(cx);
                let local_input = view.read(cx).inputs.local.clone();
                local_input.update(cx, |input, cx| input.set_value("", window, cx));
                let local_focus = local_input.read(cx).focus_handle(cx);
                local_focus.focus(window, cx);
                window.render_frame(cx);
                window.input("9090", cx);
            });
        });

        let local_after_input = cx.update(|cx| view.read(cx).state.local_port_input.clone());
        assert_eq!(local_after_input, "9090");

        cx.update(|cx| {
            let _ = window.update(cx, |_, window, cx| {
                view.update(cx, |harness, cx| {
                    harness.state.select_detected_port(3000);
                    harness.inputs.sync_from_state(&harness.state, window, cx);
                });
            });
        });

        let synced_local = cx.update(|cx| view.read(cx).inputs.local.read(cx).value().to_string());
        let synced_remote =
            cx.update(|cx| view.read(cx).inputs.remote.read(cx).value().to_string());
        assert_eq!(synced_local, "3000");
        assert_eq!(synced_remote, "3000");
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
