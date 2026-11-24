use gpui::{AnyElement, App, FontWeight, Window, div, prelude::*, px};
use lucide_icons::Icon;

use crate::components::{SemanticStatus, StatusBadgeVariant, render_status_badge};
use crate::icons::render_icon;
use crate::state::ClusterConnectionState;
use crate::theme::{OrbitTheme, apply_text_step};

#[derive(Debug, Clone)]
pub struct OfflineViewModel {
    pub badge_label: &'static str,
    pub title: String,
    pub cluster_id: Option<String>,
    pub server_url: Option<String>,
    pub error_message: String,
    pub can_retry: bool,
    pub can_disconnect: bool,
    pub icon: Icon,
}

pub fn offline_view_model(state: &ClusterConnectionState) -> Option<OfflineViewModel> {
    match state {
        ClusterConnectionState::Unreachable {
            cluster_id,
            display_name,
            server_url,
            error_message,
        } => Some(OfflineViewModel {
            badge_label: "Cluster Offline",
            title: display_name.clone(),
            cluster_id: Some(cluster_id.clone()),
            server_url: server_url.clone(),
            error_message: error_message.clone(),
            can_retry: true,
            can_disconnect: true,
            icon: Icon::ServerOff,
        }),
        ClusterConnectionState::Error {
            cluster_id,
            display_name,
            error_message,
        } => Some(OfflineViewModel {
            badge_label: "Cluster Offline",
            title: display_name
                .clone()
                .or_else(|| cluster_id.clone())
                .unwrap_or_else(|| "Cluster Connection Failed".to_string()),
            cluster_id: cluster_id.clone(),
            server_url: None,
            error_message: error_message.clone(),
            can_retry: cluster_id.is_some(),
            can_disconnect: cluster_id.is_some(),
            icon: Icon::ServerOff,
        }),
        _ => None,
    }
}

pub fn render_offline_view(
    model: &OfflineViewModel,
    theme: &OrbitTheme,
    on_retry: impl Fn(&mut Window, &mut App) + 'static,
    on_switch_cluster: impl Fn(&mut Window, &mut App) + 'static,
    on_disconnect: impl Fn(&mut Window, &mut App) + 'static,
) -> AnyElement {
    let mut card = theme
        .style_card(div())
        .p_6()
        .border_color(theme.semantic.danger.solid)
        .flex()
        .flex_col()
        .gap_4()
        .child(render_offline_header(model, theme));

    if let Some(ref cid) = model.cluster_id {
        card = card.child(
            apply_text_step(div(), theme.typography.size_xs)
                .font(theme.typography.mono_font(FontWeight::NORMAL))
                .text_color(theme.text.secondary)
                .child(format!("Context: {cid}")),
        );
    }

    if let Some(ref url) = model.server_url {
        card = card.child(
            apply_text_step(div(), theme.typography.size_xs)
                .font(theme.typography.mono_font(FontWeight::NORMAL))
                .text_color(theme.text.muted)
                .child(format!("API Server: {url}")),
        );
    }

    card = card
        .child(
            apply_text_step(div(), theme.typography.size_xs)
                .p_3()
                .rounded(theme.style.radius_control)
                .bg(theme.surface.bg_panel)
                .border_1()
                .border_color(theme.border.default)
                .text_color(theme.text.secondary)
                .child(model.error_message.clone()),
        )
        .child(render_offline_actions(
            model.can_retry,
            model.can_disconnect,
            theme,
            on_retry,
            on_switch_cluster,
            on_disconnect,
        ));

    card.into_any_element()
}

fn render_offline_header(model: &OfflineViewModel, theme: &OrbitTheme) -> AnyElement {
    div()
        .flex()
        .items_center()
        .gap_3()
        .child(
            div()
                .w(px(36.0))
                .h(px(36.0))
                .rounded(theme.style.radius_control)
                .bg(theme.semantic.danger.soft)
                .flex()
                .items_center()
                .justify_center()
                .child(render_icon(
                    model.icon,
                    px(18.0),
                    theme.semantic.danger.solid,
                )),
        )
        .child(
            div()
                .flex()
                .flex_col()
                .gap_0p5()
                .child(render_status_badge(
                    model.badge_label,
                    SemanticStatus::Danger,
                    StatusBadgeVariant::Pill,
                    theme,
                ))
                .child(
                    apply_text_step(div(), theme.typography.size_lg)
                        .font_weight(FontWeight::BOLD)
                        .text_color(theme.text.primary)
                        .child(model.title.clone()),
                ),
        )
        .into_any_element()
}

fn render_offline_actions(
    can_retry: bool,
    can_disconnect: bool,
    theme: &OrbitTheme,
    on_retry: impl Fn(&mut Window, &mut App) + 'static,
    on_switch_cluster: impl Fn(&mut Window, &mut App) + 'static,
    on_disconnect: impl Fn(&mut Window, &mut App) + 'static,
) -> AnyElement {
    let control_hover = theme.surface.control_hover;
    let mut actions = div().flex().items_center().gap_2p5();

    if can_retry {
        actions = actions.child(
            theme
                .style_control(div().id("offline-retry-btn"))
                .px_3()
                .py_1p5()
                .bg(theme.accent.soft)
                .border_color(theme.accent.accent)
                .text_color(theme.accent.accent)
                .font_weight(FontWeight::SEMIBOLD)
                .cursor_pointer()
                .hover(move |s| s.bg(control_hover))
                .flex()
                .items_center()
                .gap_1p5()
                .child(render_icon(
                    Icon::RefreshCw,
                    px(12.0),
                    theme.accent.accent,
                ))
                .child(apply_text_step(div(), theme.typography.size_xs).child("Retry Connection"))
                .on_click(move |_, window, cx| {
                    on_retry(window, cx);
                }),
        );
    }

    actions = actions.child(
        theme
            .style_control(div().id("offline-switch-cluster-btn"))
            .px_3()
            .py_1p5()
            .text_color(theme.text.primary)
            .font_weight(FontWeight::MEDIUM)
            .cursor_pointer()
            .hover(move |s| s.bg(control_hover))
            .flex()
            .items_center()
            .gap_1p5()
            .child(render_icon(Icon::Server, px(12.0), theme.text.secondary))
            .child(apply_text_step(div(), theme.typography.size_xs).child("Switch Cluster"))
            .on_click(move |_, window, cx| {
                on_switch_cluster(window, cx);
            }),
    );

    if can_disconnect {
        actions = actions.child(
            theme
                .style_control(div().id("offline-disconnect-cluster-btn"))
                .px_3()
                .py_1p5()
                .border_color(theme.semantic.danger.solid)
                .text_color(theme.semantic.danger.solid)
                .font_weight(FontWeight::MEDIUM)
                .cursor_pointer()
                .hover(move |s| s.bg(control_hover))
                .flex()
                .items_center()
                .gap_1p5()
                .child(render_icon(
                    Icon::Unplug,
                    px(12.0),
                    theme.semantic.danger.solid,
                ))
                .child(apply_text_step(div(), theme.typography.size_xs).child("Disconnect"))
                .on_click(move |_, window, cx| {
                    on_disconnect(window, cx);
                }),
        );
    }

    actions.into_any_element()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::icons::icon_char;

    #[test]
    fn test_offline_presentation_distinguishes_unreachable_and_connection_error() {
        let unreachable = ClusterConnectionState::Unreachable {
            cluster_id: "prod-ctx".to_string(),
            display_name: "Production".to_string(),
            server_url: Some("https://10.0.0.10:6443".to_string()),
            error_message: "connection refused".to_string(),
        };
        let vm_unreachable =
            offline_view_model(&unreachable).expect("Unreachable should produce OfflineViewModel");
        assert_eq!(vm_unreachable.badge_label, "Cluster Offline");
        assert!(vm_unreachable.can_retry);
        assert!(vm_unreachable.can_disconnect);
        assert_eq!(
            vm_unreachable.server_url.as_deref(),
            Some("https://10.0.0.10:6443")
        );
        assert_eq!(vm_unreachable.error_message, "connection refused");
        assert_eq!(icon_char(vm_unreachable.icon), icon_char(Icon::ServerOff));

        let err_with_id = ClusterConnectionState::Error {
            cluster_id: Some("dev-ctx".to_string()),
            display_name: Some("Dev".to_string()),
            error_message: "Invalid cert".to_string(),
        };
        let vm_err_with_id =
            offline_view_model(&err_with_id).expect("Error should produce OfflineViewModel");
        assert_eq!(vm_err_with_id.badge_label, "Cluster Offline");
        assert!(vm_err_with_id.can_retry);
        assert!(vm_err_with_id.can_disconnect);
        assert_eq!(vm_err_with_id.error_message, "Invalid cert");
        assert_eq!(icon_char(vm_err_with_id.icon), icon_char(Icon::ServerOff));

        let err_without_id = ClusterConnectionState::Error {
            cluster_id: None,
            display_name: None,
            error_message: "Bridge thread failed".to_string(),
        };
        let vm_err_without_id =
            offline_view_model(&err_without_id).expect("Error should produce OfflineViewModel");
        assert_eq!(vm_err_without_id.badge_label, "Cluster Offline");
        assert!(!vm_err_without_id.can_retry);
        assert!(!vm_err_without_id.can_disconnect);
        assert!(vm_err_without_id.server_url.is_none());
        assert_eq!(vm_err_without_id.error_message, "Bridge thread failed");
        assert_eq!(
            icon_char(vm_err_without_id.icon),
            icon_char(Icon::ServerOff)
        );
    }

    #[test]
    fn test_offline_view_model_sets_can_disconnect_for_cluster_states() {
        let disconnected = offline_view_model(&ClusterConnectionState::Disconnected);
        assert!(disconnected.is_none());

        let connecting = offline_view_model(&ClusterConnectionState::Connecting {
            cluster_id: "ctx-1".to_string(),
            display_name: "Ctx 1".to_string(),
        });
        assert!(connecting.is_none());

        let unreachable = offline_view_model(&ClusterConnectionState::Unreachable {
            cluster_id: "ctx-1".to_string(),
            display_name: "Ctx 1".to_string(),
            server_url: None,
            error_message: "down".to_string(),
        })
        .expect("unreachable model");
        assert!(unreachable.can_disconnect);
    }
}
