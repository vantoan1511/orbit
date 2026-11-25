use std::rc::Rc;

use gpui::{AnyElement, App, FontWeight, MouseButton, Window, div, prelude::*, px};
use lucide_icons::Icon;

use crate::icons::render_icon;
use crate::theme::OrbitTheme;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActionToastKind {
    Success,
    Error,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActionToast {
    pub id: u64,
    pub kind: ActionToastKind,
    pub title: String,
    pub message: String,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ActionToastState {
    current: Option<ActionToast>,
    next_id: u64,
}

impl ActionToastState {
    pub fn show_success(&mut self, title: impl Into<String>, message: impl Into<String>) -> u64 {
        self.push(ActionToastKind::Success, title, message)
    }

    pub fn show_error(&mut self, title: impl Into<String>, message: impl Into<String>) -> u64 {
        self.push(ActionToastKind::Error, title, message)
    }

    fn push(
        &mut self,
        kind: ActionToastKind,
        title: impl Into<String>,
        message: impl Into<String>,
    ) -> u64 {
        self.next_id = self.next_id.saturating_add(1);
        let id = self.next_id;
        self.current = Some(ActionToast {
            id,
            kind,
            title: title.into(),
            message: message.into(),
        });
        id
    }

    pub fn dismiss(&mut self) {
        self.current = None;
    }

    pub fn dismiss_if_id(&mut self, id: u64) -> bool {
        if self.current.as_ref().is_some_and(|toast| toast.id == id) {
            self.current = None;
            true
        } else {
            false
        }
    }

    pub fn reset_for_disconnect(&mut self) {
        self.dismiss();
    }

    pub fn current(&self) -> Option<&ActionToast> {
        self.current.as_ref()
    }
}

pub type ToastDismissCallback = Rc<dyn Fn(&mut Window, &mut App)>;

pub fn render_action_toast_overlay(
    state: &ActionToastState,
    theme: &OrbitTheme,
    on_dismiss: ToastDismissCallback,
) -> Option<AnyElement> {
    let toast = state.current()?;
    let (accent_color, icon) = match toast.kind {
        ActionToastKind::Success => (theme.semantic.success.solid, Icon::CheckCircle2),
        ActionToastKind::Error => (theme.semantic.danger.solid, Icon::AlertCircle),
    };

    Some(
        div()
            .id("action-toast-overlay")
            .absolute()
            .right(px(16.0))
            .bottom(px(36.0))
            .w(px(340.0))
            .child(
                theme
                    .style_card(div())
                    .px_3p5()
                    .py_2p5()
                    .bg(theme.surface.bg_card)
                    .border_1()
                    .border_color(accent_color)
                    .shadow_lg()
                    .flex()
                    .items_start()
                    .gap_2p5()
                    .on_mouse_down(MouseButton::Left, |_, window, cx| {
                        window.prevent_default();
                        cx.stop_propagation();
                    })
                    .child(
                        div()
                            .pt(px(1.0))
                            .child(render_icon(icon, px(15.0), accent_color)),
                    )
                    .child(
                        div()
                            .flex_1()
                            .min_w_0()
                            .flex()
                            .flex_col()
                            .gap_0p5()
                            .child(
                                div()
                                    .text_xs()
                                    .font_weight(FontWeight::SEMIBOLD)
                                    .text_color(theme.text.primary)
                                    .child(toast.title.clone()),
                            )
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(theme.text.secondary)
                                    .child(toast.message.clone()),
                            ),
                    )
                    .child(
                        div()
                            .id("action-toast-close-btn")
                            .w(px(20.0))
                            .h(px(20.0))
                            .rounded(theme.style.radius_control)
                            .flex()
                            .items_center()
                            .justify_center()
                            .cursor_pointer()
                            .hover(|s| s.bg(theme.surface.bg_hover))
                            .on_mouse_down(MouseButton::Left, move |_, window, cx| {
                                window.prevent_default();
                                cx.stop_propagation();
                                on_dismiss(window, cx);
                            })
                            .child(render_icon(Icon::X, px(12.0), theme.text.muted)),
                    ),
            )
            .into_any_element(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_action_toast_show_dismiss_and_stale_id_guard() {
        let mut state = ActionToastState::default();
        assert!(state.current().is_none());

        let first_id = state.show_success("Scaled", "Scaled web to 3 replicas.");
        assert_eq!(state.current().map(|t| t.id), Some(first_id));

        let second_id = state.show_error("Failed", "Permission denied.");
        assert_ne!(first_id, second_id);
        assert_eq!(state.current().map(|t| t.id), Some(second_id));

        assert!(
            !state.dismiss_if_id(first_id),
            "stale timer id must not dismiss newer toast"
        );
        assert!(state.current().is_some());

        assert!(state.dismiss_if_id(second_id));
        assert!(state.current().is_none());
    }
}
