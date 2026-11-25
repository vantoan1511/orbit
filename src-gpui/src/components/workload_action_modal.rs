use std::rc::Rc;

use gpui::{AnyElement, App, FontWeight, MouseButton, SharedString, Window, div, prelude::*, px};
use lucide_icons::Icon;

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
    pub cluster_name: String,
    pub action: WorkloadActionKind,
    pub kind: String,
    pub namespace: String,
    pub name: String,
    pub submitting: bool,
    pub error: Option<String>,
}

impl ConfirmModalState {
    pub fn can_submit(&self) -> bool {
        !self.submitting
    }

    pub fn handle_key_input(&mut self, key: &str) -> WorkloadActionModalKeyResult {
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
        WorkloadActionModalKeyResult::Handled
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
        self.dialog = WorkloadActionDialogState::Confirm(ConfirmModalState {
            cluster_name: cluster_name.into(),
            action,
            kind: kind.into(),
            namespace: namespace.into(),
            name: name.into(),
            submitting: false,
            error: None,
        });
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
                confirm.submitting = submitting;
                if submitting {
                    confirm.error = None;
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
                confirm.submitting = false;
                confirm.error = Some(msg);
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
            WorkloadActionDialogState::Confirm(confirm) => confirm.handle_key_input(key),
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
        WorkloadActionDialogState::Confirm(confirm) => {
            render_confirm_modal_card(confirm, theme, callbacks)
        }
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
        .child(render_modal_header(
            Icon::SlidersHorizontal,
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
                .child(render_target_summary(
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
                                    .child("Scaling to 0 replicas will stop all pods for this workload."),
                            ),
                    )
                })
                .when_some(scale.error.as_ref(), |el, err| {
                    el.child(render_error_box(err, theme))
                }),
        )
        .child(render_modal_footer(
            if scale.submitting {
                "Scaling..."
            } else {
                "Apply Scale"
            },
            can_submit,
            scale.submitting,
            false,
            theme,
            on_cancel_btn,
            on_submit_btn,
        ))
        .into_any_element()
}

fn render_confirm_modal_card(
    confirm: &ConfirmModalState,
    theme: &OrbitTheme,
    callbacks: &WorkloadActionModalCallbacks,
) -> AnyElement {
    let on_close_btn = callbacks.on_close.clone();
    let on_cancel_btn = callbacks.on_close.clone();
    let on_submit_btn = callbacks.on_submit.clone();
    let is_pod_restart = confirm.action == WorkloadActionKind::RestartPod;
    let title = if is_pod_restart {
        "Restart Pod".to_string()
    } else {
        format!("Restart {}", confirm.kind)
    };
    let description = if is_pod_restart {
        "Deleting this pod allows its managing controller to recreate a fresh replacement pod."
    } else {
        "Triggers a rolling restart by patching the pod template's kubectl.kubernetes.io/restartedAt annotation."
    };
    let submit_label = if confirm.submitting {
        "Restarting..."
    } else if is_pod_restart {
        "Delete & Restart Pod"
    } else {
        "Trigger Restart"
    };

    theme
        .style_card(div())
        .id("workload-confirm-modal-card")
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
        .child(render_modal_header(
            Icon::RotateCw,
            title,
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
                .child(render_target_summary(
                    &confirm.cluster_name,
                    &confirm.kind,
                    &confirm.namespace,
                    &confirm.name,
                    theme,
                ))
                .child(
                    div()
                        .text_xs()
                        .text_color(theme.text.secondary)
                        .child(description),
                )
                .when_some(confirm.error.as_ref(), |el, err| {
                    el.child(render_error_box(err, theme))
                }),
        )
        .child(render_modal_footer(
            submit_label,
            confirm.can_submit(),
            confirm.submitting,
            is_pod_restart,
            theme,
            on_cancel_btn,
            on_submit_btn,
        ))
        .into_any_element()
}

fn render_modal_header(
    icon: Icon,
    title: impl Into<SharedString>,
    theme: &OrbitTheme,
    on_close: ModalActionCallback,
) -> AnyElement {
    div()
        .px_5()
        .py_3()
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
                .child(render_icon(icon, px(15.0), theme.accent.accent))
                .child(
                    div()
                        .text_sm()
                        .font_weight(FontWeight::SEMIBOLD)
                        .text_color(theme.text.primary)
                        .child(title.into()),
                ),
        )
        .child(
            div()
                .id("workload-action-modal-close-btn")
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
                    on_close(window, cx);
                })
                .child(render_icon(Icon::X, px(14.0), theme.text.muted)),
        )
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
        .flex()
        .flex_col()
        .gap_1()
        .child(
            div()
                .flex()
                .items_center()
                .justify_between()
                .text_xs()
                .child(div().text_color(theme.text.muted).child("Cluster"))
                .child(
                    div()
                        .font_weight(FontWeight::MEDIUM)
                        .text_color(theme.text.primary)
                        .child(cluster_name.to_string()),
                ),
        )
        .child(
            div()
                .flex()
                .items_center()
                .justify_between()
                .text_xs()
                .child(div().text_color(theme.text.muted).child(kind.to_string()))
                .child(
                    div()
                        .font_family(theme.typography.mono_family.clone())
                        .text_xs()
                        .font_weight(FontWeight::MEDIUM)
                        .text_color(theme.text.primary)
                        .child(format!("{namespace}/{name}")),
                ),
        )
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

fn render_error_box(message: &str, theme: &OrbitTheme) -> AnyElement {
    div()
        .px_3()
        .py_2()
        .rounded(theme.style.radius_control)
        .bg(theme.semantic.danger.soft)
        .border_1()
        .border_color(theme.semantic.danger.solid)
        .text_xs()
        .text_color(theme.text.primary)
        .child(message.to_string())
        .into_any_element()
}

fn render_modal_footer(
    submit_label: &str,
    can_submit: bool,
    submitting: bool,
    destructive: bool,
    theme: &OrbitTheme,
    on_cancel: ModalActionCallback,
    on_submit: ModalActionCallback,
) -> AnyElement {
    let primary_bg = if destructive {
        theme.semantic.danger.solid
    } else {
        theme.accent.accent
    };

    div()
        .px_5()
        .py_3()
        .border_t_1()
        .border_color(theme.border.default)
        .flex()
        .items_center()
        .justify_end()
        .gap_2()
        .child(
            div()
                .id("workload-action-cancel-btn")
                .px_3()
                .py_1p5()
                .rounded(theme.style.radius_control)
                .bg(theme.surface.control_bg)
                .border_1()
                .border_color(theme.border.default)
                .text_xs()
                .font_weight(FontWeight::MEDIUM)
                .text_color(if submitting {
                    theme.text.disabled
                } else {
                    theme.text.primary
                })
                .when(!submitting, |el| {
                    el.cursor_pointer()
                        .hover(|s| s.bg(theme.surface.control_hover))
                        .on_mouse_down(MouseButton::Left, move |_, window, cx| {
                            window.prevent_default();
                            cx.stop_propagation();
                            on_cancel(window, cx);
                        })
                })
                .child("Cancel"),
        )
        .child(
            div()
                .id("workload-action-submit-btn")
                .px_3()
                .py_1p5()
                .rounded(theme.style.radius_control)
                .bg(primary_bg)
                .text_xs()
                .font_weight(FontWeight::SEMIBOLD)
                .text_color(theme.text.on_accent)
                .when(can_submit, |el| {
                    el.cursor_pointer()
                        .on_mouse_down(MouseButton::Left, move |_, window, cx| {
                            window.prevent_default();
                            cx.stop_propagation();
                            on_submit(window, cx);
                        })
                })
                .when(!can_submit, |el| el.opacity(0.45))
                .child(submit_label.to_string()),
        )
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
    fn test_scale_modal_zero_triggers_scale_down_warning_and_allows_submit() {
        let mut scale = ScaleModalState::new("prod-ctx", "Deployment", "default", "web", 3);
        scale.input_value = "0".to_string();
        assert!(scale.is_scale_to_zero());
        assert!(scale.can_submit());
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
        let mut confirm = ConfirmModalState {
            cluster_name: "prod-ctx".to_string(),
            action: WorkloadActionKind::Restart,
            kind: "Deployment".to_string(),
            namespace: "default".to_string(),
            name: "web".to_string(),
            submitting: false,
            error: None,
        };

        assert_eq!(
            confirm.handle_key_input("enter"),
            WorkloadActionModalKeyResult::Submit
        );
        assert_eq!(
            confirm.handle_key_input("escape"),
            WorkloadActionModalKeyResult::Cancel
        );

        confirm.submitting = true;
        assert_eq!(
            confirm.handle_key_input("enter"),
            WorkloadActionModalKeyResult::Handled
        );
        assert_eq!(
            confirm.handle_key_input("escape"),
            WorkloadActionModalKeyResult::Handled
        );
    }
}
