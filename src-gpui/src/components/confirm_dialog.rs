use std::rc::Rc;

use gpui::{
    AnyElement, App, FontWeight, MouseButton, Rgba, SharedString, Window, div, prelude::*, px,
};
use lucide_icons::Icon;

use crate::icons::render_icon;
use crate::theme::OrbitTheme;

pub type ConfirmDialogCallback = Rc<dyn Fn(&mut Window, &mut App)>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConfirmDialogKeyResult {
    Ignored,
    Handled,
    Cancel,
    Submit,
}

#[derive(Debug, Clone)]
pub struct ConfirmDialogSpec {
    pub cluster_name: String,
    pub kind: String,
    pub namespace: String,
    pub name: String,
    pub title: String,
    pub description: String,
    pub submit_label: String,
    pub submitting_label: String,
    pub icon: Icon,
    pub destructive: bool,
    pub requires_typed_confirmation: bool,
}

impl PartialEq for ConfirmDialogSpec {
    fn eq(&self, other: &Self) -> bool {
        self.cluster_name == other.cluster_name
            && self.kind == other.kind
            && self.namespace == other.namespace
            && self.name == other.name
            && self.title == other.title
            && self.description == other.description
            && self.submit_label == other.submit_label
            && self.submitting_label == other.submitting_label
            && char::from(self.icon) == char::from(other.icon)
            && self.destructive == other.destructive
            && self.requires_typed_confirmation == other.requires_typed_confirmation
    }
}

impl Eq for ConfirmDialogSpec {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConfirmDialogState {
    pub spec: ConfirmDialogSpec,
    pub typed_confirmation: String,
    pub submitting: bool,
    pub error: Option<String>,
}

impl ConfirmDialogState {
    pub fn new(spec: ConfirmDialogSpec) -> Self {
        Self {
            spec,
            typed_confirmation: String::new(),
            submitting: false,
            error: None,
        }
    }

    pub fn is_typed_match(&self) -> bool {
        self.typed_confirmation.trim() == self.spec.name
    }

    pub fn can_submit(&self) -> bool {
        !self.submitting
            && (!self.spec.requires_typed_confirmation || self.is_typed_match())
    }

    pub fn active_submit_label(&self) -> &str {
        if self.submitting {
            &self.spec.submitting_label
        } else {
            &self.spec.submit_label
        }
    }

    pub fn formatted_target(&self) -> String {
        format_dialog_target(&self.spec.namespace, &self.spec.name)
    }

    pub fn handle_key_input(
        &mut self,
        key: &str,
        key_char: Option<&str>,
    ) -> ConfirmDialogKeyResult {
        if key == "escape" {
            return if self.submitting {
                ConfirmDialogKeyResult::Handled
            } else {
                ConfirmDialogKeyResult::Cancel
            };
        }
        if key == "enter" {
            return if self.can_submit() {
                ConfirmDialogKeyResult::Submit
            } else {
                ConfirmDialogKeyResult::Handled
            };
        }
        if self.submitting {
            return ConfirmDialogKeyResult::Handled;
        }
        if self.spec.requires_typed_confirmation {
            if key == "backspace" {
                self.typed_confirmation.pop();
                self.error = None;
                return ConfirmDialogKeyResult::Handled;
            }
            if let Some(ch_str) = key_char
                && !ch_str.is_empty()
                && !ch_str.chars().any(char::is_control)
            {
                self.typed_confirmation.push_str(ch_str);
                self.error = None;
                return ConfirmDialogKeyResult::Handled;
            }
        }
        ConfirmDialogKeyResult::Handled
    }
}

#[derive(Clone)]
pub struct ConfirmDialogCallbacks {
    pub on_close: ConfirmDialogCallback,
    pub on_submit: ConfirmDialogCallback,
}

pub fn format_dialog_target(namespace: &str, name: &str) -> String {
    let trimmed_ns = namespace.trim();
    if trimmed_ns.is_empty() {
        name.to_string()
    } else {
        format!("{trimmed_ns}/{name}")
    }
}

pub fn render_confirm_dialog(
    state: &ConfirmDialogState,
    theme: &OrbitTheme,
    callbacks: &ConfirmDialogCallbacks,
) -> AnyElement {
    let header_icon_color = if state.spec.destructive {
        theme.semantic.danger.solid
    } else {
        theme.accent.accent
    };
    let is_typed_match = state.is_typed_match();
    let has_typed_input = !state.typed_confirmation.is_empty();

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
        .child(render_dialog_header(
            state.spec.icon,
            header_icon_color,
            state.spec.title.clone(),
            theme,
            callbacks.on_close.clone(),
        ))
        .child(
            div()
                .px_5()
                .py_4()
                .flex()
                .flex_col()
                .gap_3()
                .child(render_dialog_target_summary(
                    &state.spec.cluster_name,
                    &state.spec.kind,
                    &state.spec.namespace,
                    &state.spec.name,
                    theme,
                ))
                .child(
                    div()
                        .text_xs()
                        .text_color(theme.text.secondary)
                        .child(state.spec.description.clone()),
                )
                .when(state.spec.requires_typed_confirmation, |el| {
                    let input_border = if is_typed_match {
                        theme.semantic.success.solid
                    } else if has_typed_input {
                        theme.semantic.danger.solid
                    } else {
                        theme.border.input
                    };
                    el.child(
                        div()
                            .flex()
                            .flex_col()
                            .gap_1p5()
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(theme.text.secondary)
                                    .child(format!(
                                        "Type {} to confirm deletion:",
                                        state.spec.name
                                    )),
                            )
                            .child(
                                div()
                                    .id("confirm-dialog-typed-input")
                                    .h(px(30.0))
                                    .px_2p5()
                                    .rounded(theme.style.radius_control)
                                    .bg(theme.surface.input_bg)
                                    .border_1()
                                    .border_color(input_border)
                                    .flex()
                                    .items_center()
                                    .justify_between()
                                    .gap_2()
                                    .child(
                                        div()
                                            .flex()
                                            .items_center()
                                            .min_w_0()
                                            .font_family(theme.typography.mono_family.clone())
                                            .text_xs()
                                            .text_color(if has_typed_input {
                                                theme.text.primary
                                            } else {
                                                theme.text.muted
                                            })
                                            .child(if has_typed_input {
                                                state.typed_confirmation.clone()
                                            } else {
                                                state.spec.name.clone()
                                            }),
                                    )
                                    .when(is_typed_match, |el| {
                                        el.child(render_icon(
                                            Icon::Check,
                                            px(13.0),
                                            theme.semantic.success.solid,
                                        ))
                                    }),
                            ),
                    )
                })
                .when_some(state.error.as_ref(), |el, err| {
                    el.child(render_dialog_error_box(err, theme))
                }),
        )
        .child(render_dialog_footer(
            state.active_submit_label(),
            state.can_submit(),
            state.submitting,
            state.spec.destructive,
            theme,
            callbacks.on_close.clone(),
            callbacks.on_submit.clone(),
        ))
        .into_any_element()
}

pub(crate) fn render_dialog_header(
    icon: Icon,
    icon_color: Rgba,
    title: impl Into<SharedString>,
    theme: &OrbitTheme,
    on_close: ConfirmDialogCallback,
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
                .child(render_icon(icon, px(15.0), icon_color))
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

pub(crate) fn render_dialog_target_summary(
    cluster_name: &str,
    kind: &str,
    namespace: &str,
    name: &str,
    theme: &OrbitTheme,
) -> AnyElement {
    let trimmed_ns = namespace.trim();
    let scope_row_label = if trimmed_ns.is_empty() {
        "Scope"
    } else {
        "Namespace"
    };
    let scope_row_value = if trimmed_ns.is_empty() {
        "Cluster-scoped".to_string()
    } else {
        trimmed_ns.to_string()
    };

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
                .child(div().text_color(theme.text.muted).child(scope_row_label))
                .child(
                    div()
                        .font_weight(FontWeight::MEDIUM)
                        .text_color(theme.text.primary)
                        .child(scope_row_value),
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
                        .child(format_dialog_target(namespace, name)),
                ),
        )
        .into_any_element()
}

pub(crate) fn render_dialog_error_box(message: &str, theme: &OrbitTheme) -> AnyElement {
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

pub(crate) fn render_dialog_footer(
    submit_label: &str,
    can_submit: bool,
    submitting: bool,
    destructive: bool,
    theme: &OrbitTheme,
    on_cancel: ConfirmDialogCallback,
    on_submit: ConfirmDialogCallback,
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

    fn sample_cluster_delete_spec() -> ConfirmDialogSpec {
        ConfirmDialogSpec {
            cluster_name: "prod-ctx".to_string(),
            kind: "Node".to_string(),
            namespace: String::new(),
            name: "worker-1".to_string(),
            title: "Delete Node".to_string(),
            description: "Permanently deletes this cluster-scoped resource.".to_string(),
            submit_label: "Delete Node".to_string(),
            submitting_label: "Deleting...".to_string(),
            icon: Icon::Trash2,
            destructive: true,
            requires_typed_confirmation: true,
        }
    }

    #[test]
    fn test_confirm_dialog_requires_exact_typed_name_for_cluster_scoped_delete() {
        let mut state = ConfirmDialogState::new(sample_cluster_delete_spec());
        assert!(state.spec.requires_typed_confirmation);
        assert!(!state.can_submit());
        assert_eq!(
            state.handle_key_input("enter", None),
            ConfirmDialogKeyResult::Handled
        );

        for ch in "worker-2".chars() {
            let s = ch.to_string();
            assert_eq!(
                state.handle_key_input(&s, Some(&s)),
                ConfirmDialogKeyResult::Handled
            );
        }
        assert!(!state.can_submit());
        assert_eq!(
            state.handle_key_input("enter", None),
            ConfirmDialogKeyResult::Handled
        );

        assert_eq!(
            state.handle_key_input("backspace", None),
            ConfirmDialogKeyResult::Handled
        );
        assert_eq!(
            state.handle_key_input("1", Some("1")),
            ConfirmDialogKeyResult::Handled
        );
        assert!(state.can_submit());
        assert_eq!(
            state.handle_key_input("enter", None),
            ConfirmDialogKeyResult::Submit
        );
    }

    #[test]
    fn test_confirm_dialog_namespaced_delete_allows_single_click_submit() {
        let mut state = ConfirmDialogState::new(ConfirmDialogSpec {
            cluster_name: "prod-ctx".to_string(),
            kind: "Deployment".to_string(),
            namespace: "prod".to_string(),
            name: "api".to_string(),
            title: "Delete Deployment".to_string(),
            description: "Permanently deletes this resource.".to_string(),
            submit_label: "Delete Deployment".to_string(),
            submitting_label: "Deleting...".to_string(),
            icon: Icon::Trash2,
            destructive: true,
            requires_typed_confirmation: false,
        });
        assert!(!state.spec.requires_typed_confirmation);
        assert!(state.can_submit());
        assert!(state.spec.destructive);
        assert_eq!(
            state.handle_key_input("enter", None),
            ConfirmDialogKeyResult::Submit
        );
    }
}
