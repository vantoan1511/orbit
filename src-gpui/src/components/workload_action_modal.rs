use std::rc::Rc;

use gpui::{AnyElement, App, FontWeight, MouseButton, SharedString, Window, div, prelude::*, px};
use lucide_icons::Icon;

use crate::components::confirm_dialog::{
    ConfirmDialogCallbacks, ConfirmDialogKeyResult, ConfirmDialogSpec, ConfirmDialogState,
    render_confirm_dialog, render_dialog_error_box, render_dialog_footer, render_dialog_header,
    render_dialog_target_summary,
};
use crate::icons::render_icon;
use crate::state::WorkloadActionKind;
use crate::theme::OrbitTheme;

pub const MAX_SCALE_REPLICAS: i32 = 10_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WorkloadActionModalKeyResult {
    Ignored,
    Handled,
    Cancel,
    Submit,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScaleModalState {
    pub cluster_name: String,
    pub kind: String,
    pub namespace: String,
    pub name: String,
    pub current_replicas: i32,
    pub input_value: String,
    pub submitting: bool,
    pub error: Option<String>,
}

impl ScaleModalState {
    pub fn new(
        cluster_name: impl Into<String>,
        kind: impl Into<String>,
        namespace: impl Into<String>,
        name: impl Into<String>,
        current_replicas: i32,
    ) -> Self {
        let clamped = current_replicas.max(0);
        Self {
            cluster_name: cluster_name.into(),
            kind: kind.into(),
            namespace: namespace.into(),
            name: name.into(),
            current_replicas: clamped,
            input_value: clamped.to_string(),
            submitting: false,
            error: None,
        }
    }

    pub fn parsed_replicas(&self) -> Option<i32> {
        let trimmed = self.input_value.trim();
        if trimmed.is_empty() || !trimmed.bytes().all(|b| b.is_ascii_digit()) {
            return None;
        }
        let value = trimmed.parse::<i32>().ok()?;
        if (0..=MAX_SCALE_REPLICAS).contains(&value) {
            Some(value)
        } else {
            None
        }
    }

    pub fn is_scale_to_zero(&self) -> bool {
        self.parsed_replicas() == Some(0)
    }

    pub fn is_danger(&self) -> bool {
        self.is_scale_to_zero()
    }

    pub fn submit_label(&self) -> &'static str {
        if self.submitting {
            "Scaling..."
        } else if self.is_scale_to_zero() {
            "Scale to 0"
        } else {
            "Apply Scale"
        }
    }

    pub fn can_submit(&self) -> bool {
        !self.submitting
            && self
                .parsed_replicas()
                .is_some_and(|replicas| replicas != self.current_replicas)
    }

    pub fn increment(&mut self) {
        if self.submitting {
            return;
        }
        let next = self
            .parsed_replicas()
            .unwrap_or(0)
            .saturating_add(1)
            .min(MAX_SCALE_REPLICAS);
        self.input_value = next.to_string();
        self.error = None;
    }

    pub fn decrement(&mut self) {
        if self.submitting {
            return;
        }
        let next = self.parsed_replicas().unwrap_or(0).saturating_sub(1).max(0);
        self.input_value = next.to_string();
        self.error = None;
    }

    pub fn handle_key_input(
        &mut self,
        key: &str,
        key_char: Option<&str>,
    ) -> WorkloadActionModalKeyResult {
        if key == "escape" {
            return if self.submitting {
                WorkloadActionModalKeyResult::Handled
            } else {
                WorkloadActionModalKeyResult::Cancel
            };
        }
        if key == "enter" {
            return if self.can_submit() {
                WorkloadActionModalKeyResult::Submit
            } else {
                WorkloadActionModalKeyResult::Handled
            };
        }
        if self.submitting {
            return WorkloadActionModalKeyResult::Handled;
        }
        if key == "backspace" {
            self.input_value.pop();
            self.error = None;
            return WorkloadActionModalKeyResult::Handled;
        }
        if key == "up" {
            self.increment();
            return WorkloadActionModalKeyResult::Handled;
        }
        if key == "down" {
            self.decrement();
            return WorkloadActionModalKeyResult::Handled;
        }
        if let Some(ch_str) = key_char {
            let mut chars = ch_str.chars();
            if let (Some(ch), None) = (chars.next(), chars.next())
                && ch.is_ascii_digit()
            {
                if self.input_value == "0" {
                    self.input_value = ch.to_string();
                } else if self.input_value.len() < 5 {
                    self.input_value.push(ch);
                }
                self.error = None;
            }
        }
        WorkloadActionModalKeyResult::Handled
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConfirmModalState {
    pub action: WorkloadActionKind,
    pub dialog: ConfirmDialogState,
}

impl ConfirmModalState {
    pub fn new(
        cluster_name: impl Into<String>,
        action: WorkloadActionKind,
        kind: impl Into<String>,
        namespace: impl Into<String>,
        name: impl Into<String>,
    ) -> Self {
        let cluster_name = cluster_name.into();
        let kind = kind.into();
        let namespace = namespace.into();
        let name = name.into();
        let is_cluster_scoped = namespace.trim().is_empty();
        let destructive = matches!(
            action,
            WorkloadActionKind::Delete | WorkloadActionKind::RestartPod
        );
        let requires_typed_confirmation = action == WorkloadActionKind::Delete && is_cluster_scoped;

        let (icon, title, description, submit_label, submitting_label) = match action {
            WorkloadActionKind::Delete => (
                Icon::Trash2,
                format!("Delete {kind}"),
                if is_cluster_scoped {
                    "Permanently deletes this cluster-scoped resource. This action is destructive and cannot be undone."
                } else {
                    "Permanently deletes this resource from the cluster. This action is destructive and cannot be undone."
                }
                .to_string(),
                format!("Delete {kind}"),
                "Deleting...".to_string(),
            ),
            WorkloadActionKind::RestartPod => (
                Icon::RotateCw,
                "Restart Pod".to_string(),
                "Deleting this pod allows its managing controller to recreate a fresh replacement pod."
                    .to_string(),
                "Delete & Restart Pod".to_string(),
                "Restarting...".to_string(),
            ),
            _ => (
                Icon::RotateCw,
                format!("Restart {kind}"),
                "Triggers a rolling restart by patching the pod template's kubectl.kubernetes.io/restartedAt annotation."
                    .to_string(),
                "Trigger Restart".to_string(),
                "Restarting...".to_string(),
            ),
        };

        Self {
            action,
            dialog: ConfirmDialogState::new(ConfirmDialogSpec {
                cluster_name,
                kind,
                namespace,
                name,
                title,
                description,
                submit_label,
                submitting_label,
                icon,
                destructive,
                requires_typed_confirmation,
            }),
        }
    }

    pub fn can_submit(&self) -> bool {
        self.dialog.can_submit()
    }

    pub fn is_danger(&self) -> bool {
        self.dialog.spec.destructive
    }

    pub fn title(&self) -> &str {
        &self.dialog.spec.title
    }

    pub fn description(&self) -> &str {
        &self.dialog.spec.description
    }

    pub fn submit_label(&self) -> &str {
        self.dialog.active_submit_label()
    }

    pub fn formatted_target(&self) -> String {
        self.dialog.formatted_target()
    }

    pub fn handle_key_input(
        &mut self,
        key: &str,
        key_char: Option<&str>,
    ) -> WorkloadActionModalKeyResult {
        match self.dialog.handle_key_input(key, key_char) {
            ConfirmDialogKeyResult::Ignored => WorkloadActionModalKeyResult::Ignored,
            ConfirmDialogKeyResult::Handled => WorkloadActionModalKeyResult::Handled,
            ConfirmDialogKeyResult::Cancel => WorkloadActionModalKeyResult::Cancel,
            ConfirmDialogKeyResult::Submit => WorkloadActionModalKeyResult::Submit,
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub enum WorkloadActionDialogState {
    #[default]
    Closed,
    Scale(ScaleModalState),
    Confirm(ConfirmModalState),
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct WorkloadActionModalState {
    pub dialog: WorkloadActionDialogState,
}

impl WorkloadActionModalState {
    pub fn is_open(&self) -> bool {
        !matches!(self.dialog, WorkloadActionDialogState::Closed)
    }

    pub fn open_scale(
        &mut self,
        cluster_name: impl Into<String>,
        kind: impl Into<String>,
        namespace: impl Into<String>,
        name: impl Into<String>,
        current_replicas: i32,
    ) {
        self.dialog = WorkloadActionDialogState::Scale(ScaleModalState::new(
            cluster_name,
            kind,
            namespace,
            name,
            current_replicas,
        ));
    }

    pub fn open_confirm(
        &mut self,
        cluster_name: impl Into<String>,
        action: WorkloadActionKind,
        kind: impl Into<String>,
        namespace: impl Into<String>,
        name: impl Into<String>,
    ) {
        self.dialog = WorkloadActionDialogState::Confirm(ConfirmModalState::new(
            cluster_name,
            action,
            kind,
            namespace,
            name,
        ));
    }

    pub fn close(&mut self) {
        self.dialog = WorkloadActionDialogState::Closed;
    }

    pub fn reset_for_disconnect(&mut self) {
        self.close();
    }

    pub fn set_submitting(&mut self, submitting: bool) {
        match &mut self.dialog {
            WorkloadActionDialogState::Closed => {}
            WorkloadActionDialogState::Scale(scale) => {
                scale.submitting = submitting;
                if submitting {
                    scale.error = None;
                }
            }
            WorkloadActionDialogState::Confirm(confirm) => {
                confirm.dialog.submitting = submitting;
                if submitting {
                    confirm.dialog.error = None;
                }
            }
        }
    }

    pub fn set_error(&mut self, message: impl Into<String>) {
        let msg = message.into();
        match &mut self.dialog {
            WorkloadActionDialogState::Closed => {}
            WorkloadActionDialogState::Scale(scale) => {
                scale.submitting = false;
                scale.error = Some(msg);
            }
            WorkloadActionDialogState::Confirm(confirm) => {
                confirm.dialog.submitting = false;
                confirm.dialog.error = Some(msg);
            }
        }
    }

    pub fn handle_key_input(
        &mut self,
        key: &str,
        key_char: Option<&str>,
    ) -> WorkloadActionModalKeyResult {
        match &mut self.dialog {
            WorkloadActionDialogState::Closed => WorkloadActionModalKeyResult::Ignored,
            WorkloadActionDialogState::Scale(scale) => scale.handle_key_input(key, key_char),
            WorkloadActionDialogState::Confirm(confirm) => confirm.handle_key_input(key, key_char),
        }
    }
}

pub type ModalActionCallback = Rc<dyn Fn(&mut Window, &mut App)>;

#[derive(Clone)]
pub struct WorkloadActionModalCallbacks {
    pub on_close: ModalActionCallback,
    pub on_submit: ModalActionCallback,
    pub on_decrement: ModalActionCallback,
    pub on_increment: ModalActionCallback,
}

pub fn render_workload_action_modal(
    state: &WorkloadActionModalState,
    theme: &OrbitTheme,
    callbacks: &WorkloadActionModalCallbacks,
) -> Option<AnyElement> {
    let card = match &state.dialog {
        WorkloadActionDialogState::Closed => return None,
        WorkloadActionDialogState::Scale(scale) => render_scale_modal_card(scale, theme, callbacks),
        WorkloadActionDialogState::Confirm(confirm) => render_confirm_dialog(
            &confirm.dialog,
            theme,
            &ConfirmDialogCallbacks {
                on_close: callbacks.on_close.clone(),
                on_submit: callbacks.on_submit.clone(),
            },
        ),
    };

    let on_close_backdrop = callbacks.on_close.clone();
    Some(
        div()
            .id("workload-action-modal-backdrop")
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
            .into_any_element(),
    )
}

fn render_scale_modal_card(
    scale: &ScaleModalState,
    theme: &OrbitTheme,
    callbacks: &WorkloadActionModalCallbacks,
) -> AnyElement {
    let on_close_btn = callbacks.on_close.clone();
    let on_cancel_btn = callbacks.on_close.clone();
    let on_submit_btn = callbacks.on_submit.clone();
    let on_decrement = callbacks.on_decrement.clone();
    let on_increment = callbacks.on_increment.clone();
    let can_submit = scale.can_submit();
    let is_zero = scale.is_scale_to_zero();
    let is_invalid = scale.parsed_replicas().is_none();

    theme
        .style_card(div())
        .id("workload-scale-modal-card")
        .w(px(420.0))
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
        .child(render_dialog_header(
            Icon::SlidersHorizontal,
            if scale.is_danger() {
                theme.semantic.danger.solid
            } else {
                theme.accent.accent
            },
            format!("Scale {}", scale.kind),
            theme,
            on_close_btn,
        ))
        .child(
            div()
                .px_5()
                .py_4()
                .flex()
                .flex_col()
                .gap_3()
                .child(render_dialog_target_summary(
                    &scale.cluster_name,
                    &scale.kind,
                    &scale.namespace,
                    &scale.name,
                    theme,
                ))
                .child(
                    div()
                        .flex()
                        .items_center()
                        .justify_between()
                        .gap_3()
                        .child(
                            div()
                                .flex()
                                .flex_col()
                                .gap_0p5()
                                .child(
                                    div()
                                        .text_xs()
                                        .font_weight(FontWeight::MEDIUM)
                                        .text_color(theme.text.primary)
                                        .child("Desired Replicas"),
                                )
                                .child(
                                    div()
                                        .text_xs()
                                        .text_color(theme.text.muted)
                                        .child(format!("Current: {}", scale.current_replicas)),
                                ),
                        )
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .gap_1p5()
                                .child(render_stepper_button(
                                    "scale-decrement-btn",
                                    Icon::Minus,
                                    scale.submitting || scale.parsed_replicas().unwrap_or(0) <= 0,
                                    theme,
                                    on_decrement,
                                ))
                                .child(
                                    div()
                                        .min_w(px(72.0))
                                        .h(px(30.0))
                                        .px_2p5()
                                        .rounded(theme.style.radius_control)
                                        .bg(theme.surface.input_bg)
                                        .border_1()
                                        .border_color(if is_invalid {
                                            theme.semantic.danger.solid
                                        } else {
                                            theme.border.input
                                        })
                                        .flex()
                                        .items_center()
                                        .justify_center()
                                        .text_sm()
                                        .font_weight(FontWeight::SEMIBOLD)
                                        .text_color(theme.text.primary)
                                        .child(if scale.input_value.is_empty() {
                                            SharedString::from(" ")
                                        } else {
                                            SharedString::from(scale.input_value.clone())
                                        }),
                                )
                                .child(render_stepper_button(
                                    "scale-increment-btn",
                                    Icon::Plus,
                                    scale.submitting
                                        || scale.parsed_replicas().unwrap_or(0) >= MAX_SCALE_REPLICAS,
                                    theme,
                                    on_increment,
                                )),
                        ),
                )
                .when(is_invalid, |el| {
                    el.child(
                        div()
                            .text_xs()
                            .text_color(theme.semantic.danger.solid)
                            .child(format!("Enter an integer between 0 and {MAX_SCALE_REPLICAS}.")),
                    )
                })
                .when(is_zero, |el| {
                    el.child(
                        div()
                            .px_3()
                            .py_2()
                            .rounded(theme.style.radius_control)
                            .bg(theme.semantic.danger.soft)
                            .border_1()
                            .border_color(theme.semantic.danger.solid)
                            .flex()
                            .items_center()
                            .gap_2()
                            .child(render_icon(
                                Icon::AlertTriangle,
                                px(14.0),
                                theme.semantic.danger.solid,
                            ))
                            .child(
                                div()
                                    .flex_1()
                                    .text_xs()
                                    .text_color(theme.text.primary)
                                    .child(
                                        "Scaling to 0 replicas will stop all pods for this workload.",
                                    ),
                            ),
                    )
                })
                .when_some(scale.error.as_ref(), |el, err| {
                    el.child(render_dialog_error_box(err, theme))
                }),
        )
        .child(render_dialog_footer(
            scale.submit_label(),
            can_submit,
            scale.submitting,
            scale.is_danger(),
            theme,
            on_cancel_btn,
            on_submit_btn,
        ))
        .into_any_element()
}

fn render_stepper_button(
    id: &'static str,
    icon: Icon,
    disabled: bool,
    theme: &OrbitTheme,
    on_click: ModalActionCallback,
) -> AnyElement {
    div()
        .id(id)
        .w(px(30.0))
        .h(px(30.0))
        .rounded(theme.style.radius_control)
        .bg(theme.surface.control_bg)
        .border_1()
        .border_color(theme.border.default)
        .flex()
        .items_center()
        .justify_center()
        .when(!disabled, |el| {
            el.cursor_pointer()
                .hover(|s| s.bg(theme.surface.control_hover))
                .on_mouse_down(MouseButton::Left, move |_, window, cx| {
                    window.prevent_default();
                    cx.stop_propagation();
                    on_click(window, cx);
                })
        })
        .when(disabled, |el| el.opacity(0.45))
        .child(render_icon(
            icon,
            px(13.0),
            if disabled {
                theme.text.disabled
            } else {
                theme.text.primary
            },
        ))
        .into_any_element()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scale_modal_prefills_current_replicas_and_validates_numeric_bounds() {
        let mut scale = ScaleModalState::new("prod-ctx", "Deployment", "default", "web", 3);
        assert_eq!(scale.input_value, "3");
        assert_eq!(scale.parsed_replicas(), Some(3));
        assert!(
            !scale.can_submit(),
            "unchanged replicas should disable submit"
        );

        scale.input_value = "".to_string();
        assert_eq!(scale.parsed_replicas(), None);
        assert!(!scale.can_submit());

        scale.input_value = "abc".to_string();
        assert_eq!(scale.parsed_replicas(), None);
        assert!(!scale.can_submit());

        scale.input_value = "10001".to_string();
        assert_eq!(scale.parsed_replicas(), None);
        assert!(!scale.can_submit());

        scale.input_value = "5".to_string();
        assert_eq!(scale.parsed_replicas(), Some(5));
        assert!(scale.can_submit());
    }

    #[test]
    fn test_scale_modal_zero_elevates_to_danger_styling_and_label() {
        let mut scale = ScaleModalState::new("prod-ctx", "Deployment", "default", "web", 3);
        assert!(!scale.is_scale_to_zero());
        assert!(!scale.is_danger());
        assert_eq!(scale.submit_label(), "Apply Scale");

        scale.input_value = "0".to_string();
        assert!(scale.is_scale_to_zero());
        assert!(scale.is_danger());
        assert_eq!(scale.submit_label(), "Scale to 0");
        assert!(scale.can_submit());

        scale.submitting = true;
        assert_eq!(scale.submit_label(), "Scaling...");
    }

    #[test]
    fn test_scale_modal_keyboard_input_digits_backspace_enter_escape() {
        let mut scale = ScaleModalState::new("prod-ctx", "Deployment", "default", "web", 3);

        assert_eq!(
            scale.handle_key_input("x", Some("x")),
            WorkloadActionModalKeyResult::Handled
        );
        assert_eq!(scale.input_value, "3");

        assert_eq!(
            scale.handle_key_input("backspace", None),
            WorkloadActionModalKeyResult::Handled
        );
        assert_eq!(scale.input_value, "");

        assert_eq!(
            scale.handle_key_input("5", Some("5")),
            WorkloadActionModalKeyResult::Handled
        );
        assert_eq!(scale.input_value, "5");

        assert_eq!(
            scale.handle_key_input("enter", None),
            WorkloadActionModalKeyResult::Submit
        );

        assert_eq!(
            scale.handle_key_input("escape", None),
            WorkloadActionModalKeyResult::Cancel
        );
    }

    #[test]
    fn test_confirm_modal_keyboard_enter_confirms_and_escape_cancels() {
        let mut confirm = ConfirmModalState::new(
            "prod-ctx",
            WorkloadActionKind::Restart,
            "Deployment",
            "default",
            "web",
        );

        assert_eq!(
            confirm.handle_key_input("enter", None),
            WorkloadActionModalKeyResult::Submit
        );
        assert_eq!(
            confirm.handle_key_input("escape", None),
            WorkloadActionModalKeyResult::Cancel
        );

        confirm.dialog.submitting = true;
        assert_eq!(
            confirm.handle_key_input("enter", None),
            WorkloadActionModalKeyResult::Handled
        );
        assert_eq!(
            confirm.handle_key_input("escape", None),
            WorkloadActionModalKeyResult::Handled
        );
    }

    #[test]
    fn test_delete_confirm_modal_state_lifecycle_and_keys() {
        let mut modal = WorkloadActionModalState::default();
        modal.open_confirm("prod-ctx", WorkloadActionKind::Delete, "Deployment", "prod", "api");
        let WorkloadActionDialogState::Confirm(ref confirm) = modal.dialog else {
            panic!("expected Confirm dialog");
        };
        assert!(!confirm.dialog.spec.requires_typed_confirmation);
        assert!(confirm.can_submit());
        assert_eq!(confirm.formatted_target(), "prod/api");

        assert_eq!(
            modal.handle_key_input("enter", None),
            WorkloadActionModalKeyResult::Submit
        );
        modal.set_submitting(true);
        assert_eq!(
            modal.handle_key_input("escape", None),
            WorkloadActionModalKeyResult::Handled
        );

        modal.set_error("Permission denied");
        let WorkloadActionDialogState::Confirm(ref confirm) = modal.dialog else {
            panic!("expected Confirm dialog");
        };
        assert!(!confirm.dialog.submitting);
        assert_eq!(confirm.dialog.error.as_deref(), Some("Permission denied"));
        assert!(confirm.can_submit());
        assert_eq!(
            modal.handle_key_input("escape", None),
            WorkloadActionModalKeyResult::Cancel
        );

        // Cluster-scoped resource deletion requires exact typed resource name
        modal.open_confirm("prod-ctx", WorkloadActionKind::Delete, "Node", "", "worker-1");
        let WorkloadActionDialogState::Confirm(ref node_confirm) = modal.dialog else {
            panic!("expected Confirm dialog for Node");
        };
        assert!(node_confirm.dialog.spec.requires_typed_confirmation);
        assert!(!node_confirm.can_submit());
        assert_eq!(node_confirm.formatted_target(), "worker-1");
        assert_eq!(
            modal.handle_key_input("enter", None),
            WorkloadActionModalKeyResult::Handled
        );

        for ch in "worker-1".chars() {
            let s = ch.to_string();
            assert_eq!(
                modal.handle_key_input(&s, Some(&s)),
                WorkloadActionModalKeyResult::Handled
            );
        }
        assert_eq!(
            modal.handle_key_input("enter", None),
            WorkloadActionModalKeyResult::Submit
        );
    }

    #[test]
    fn test_confirm_modal_copy_and_target_format_for_delete() {
        let namespaced = ConfirmModalState::new(
            "prod-ctx",
            WorkloadActionKind::Delete,
            "Deployment",
            "prod",
            "api",
        );
        assert_eq!(namespaced.title(), "Delete Deployment");
        assert!(namespaced.is_danger());
        assert_eq!(namespaced.submit_label(), "Delete Deployment");
        assert_eq!(namespaced.formatted_target(), "prod/api");
        assert!(namespaced.description().contains("cannot be undone"));

        let mut cluster_scoped = ConfirmModalState::new(
            "prod-ctx",
            WorkloadActionKind::Delete,
            "Node",
            "",
            "worker-1",
        );
        assert_eq!(cluster_scoped.title(), "Delete Node");
        assert!(cluster_scoped.is_danger());
        assert!(cluster_scoped.dialog.spec.requires_typed_confirmation);
        assert_eq!(cluster_scoped.submit_label(), "Delete Node");
        assert_eq!(cluster_scoped.formatted_target(), "worker-1");
        assert!(cluster_scoped.description().contains("cluster-scoped"));

        cluster_scoped.dialog.submitting = true;
        assert_eq!(cluster_scoped.submit_label(), "Deleting...");
    }
}
