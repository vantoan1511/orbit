use std::rc::Rc;
use std::time::{SystemTime, UNIX_EPOCH};

use gpui::{AnyElement, App, FontWeight, MouseButton, Styled, Window, div, prelude::*, px};
use lucide_icons::Icon;
use orbit_core::kubernetes::models::{ClusterInfo, UserProfileInfo};

use crate::components::TextTooltip;
use crate::icons::render_icon;
use crate::theme::{OrbitTheme, apply_text_step};

pub const SHELL_DRAWER_WIDTH: f32 = 420.0;
pub const MAX_NOTIFICATIONS: usize = 100;

pub type ShellDrawerVoidCallback = Rc<dyn Fn(&mut Window, &mut App)>;
pub type ShellDrawerFilterCallback = Rc<dyn Fn(NotificationFilterTab, &mut Window, &mut App)>;
pub type ShellDrawerIdCallback = Rc<dyn Fn(u64, &mut Window, &mut App)>;
pub type ShellDrawerTextCallback = Rc<dyn Fn(String, &mut Window, &mut App)>;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ActiveShellDrawer {
    #[default]
    None,
    Notifications,
    UserProfile,
}

impl ActiveShellDrawer {
    pub fn toggle(&mut self, target: Self) {
        *self = if *self == target { Self::None } else { target };
    }

    pub fn close(&mut self) -> bool {
        if *self == Self::None {
            false
        } else {
            *self = Self::None;
            true
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NotificationSeverity {
    Info,
    Success,
    Warning,
    Error,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum NotificationFilterTab {
    #[default]
    All,
    Unread,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NotificationItem {
    pub id: u64,
    pub title: String,
    pub message: String,
    pub severity: NotificationSeverity,
    pub timestamp_secs: u64,
    pub read: bool,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct NotificationCenterState {
    pub items: Vec<NotificationItem>,
    pub filter: NotificationFilterTab,
    next_id: u64,
}

impl NotificationCenterState {
    pub fn push(
        &mut self,
        severity: NotificationSeverity,
        title: impl Into<String>,
        message: impl Into<String>,
        timestamp_secs: u64,
    ) -> u64 {
        self.next_id = self.next_id.saturating_add(1);
        let id = self.next_id;
        self.items.insert(
            0,
            NotificationItem {
                id,
                title: title.into(),
                message: message.into(),
                severity,
                timestamp_secs,
                read: false,
            },
        );
        if self.items.len() > MAX_NOTIFICATIONS {
            self.items.truncate(MAX_NOTIFICATIONS);
        }
        id
    }

    pub fn unread_count(&self) -> usize {
        self.items.iter().filter(|item| !item.read).count()
    }

    pub fn visible_items(&self) -> Vec<&NotificationItem> {
        match self.filter {
            NotificationFilterTab::All => self.items.iter().collect(),
            NotificationFilterTab::Unread => self.items.iter().filter(|item| !item.read).collect(),
        }
    }

    pub fn mark_read(&mut self, id: u64) -> bool {
        let Some(item) = self.items.iter_mut().find(|item| item.id == id) else {
            return false;
        };
        if item.read {
            return false;
        }
        item.read = true;
        true
    }

    pub fn mark_all_read(&mut self) -> bool {
        let mut changed = false;
        for item in &mut self.items {
            if !item.read {
                item.read = true;
                changed = true;
            }
        }
        changed
    }

    pub fn remove(&mut self, id: u64) -> bool {
        let prev_len = self.items.len();
        self.items.retain(|item| item.id != id);
        self.items.len() != prev_len
    }

    pub fn clear_all(&mut self) -> bool {
        if self.items.is_empty() {
            false
        } else {
            self.items.clear();
            true
        }
    }
}

pub fn format_relative_time(age_secs: u64) -> String {
    if age_secs < 60 {
        "just now".to_string()
    } else if age_secs < 3_600 {
        format!("{}m ago", age_secs / 60)
    } else if age_secs < 86_400 {
        format!("{}h ago", age_secs / 3_600)
    } else {
        format!("{}d ago", age_secs / 86_400)
    }
}

pub fn notifications_tooltip(unread: usize) -> String {
    if unread == 0 {
        "Notifications".to_string()
    } else {
        format!("Notifications ({unread} unread)")
    }
}

pub struct NotificationDrawerCallbacks {
    pub on_close: ShellDrawerVoidCallback,
    pub on_select_filter: ShellDrawerFilterCallback,
    pub on_mark_read: ShellDrawerIdCallback,
    pub on_mark_all_read: ShellDrawerVoidCallback,
    pub on_remove: ShellDrawerIdCallback,
    pub on_clear_all: ShellDrawerVoidCallback,
    pub on_copy: ShellDrawerTextCallback,
}

pub struct ProfileDrawerCallbacks {
    pub on_close: ShellDrawerVoidCallback,
    pub on_switch_cluster: ShellDrawerTextCallback,
    pub on_add_cluster: ShellDrawerVoidCallback,
    pub on_refresh: ShellDrawerVoidCallback,
    pub on_open_settings: ShellDrawerVoidCallback,
}

fn current_unix_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

fn effective_shell_drawer_width(window_width: f32) -> f32 {
    if window_width > 0.0 {
        SHELL_DRAWER_WIDTH.min((window_width - 80.0).max(280.0))
    } else {
        SHELL_DRAWER_WIDTH
    }
}

fn render_close_button(
    id: &'static str,
    theme: &OrbitTheme,
    on_close: ShellDrawerVoidCallback,
) -> AnyElement {
    div()
        .id(id)
        .w(px(28.0))
        .h(px(28.0))
        .rounded(theme.style.radius_control)
        .flex()
        .items_center()
        .justify_center()
        .cursor_pointer()
        .hover(|s| s.bg(theme.surface.bg_hover))
        .tooltip(|_, cx| cx.new(|_| TextTooltip::new("Close")).into())
        .on_mouse_down(MouseButton::Left, move |_, window, cx| {
            cx.stop_propagation();
            on_close(window, cx);
        })
        .child(render_icon(Icon::X, px(15.0), theme.text.secondary))
        .into_any_element()
}

fn render_shell_drawer_backdrop(
    backdrop_id: &'static str,
    panel_id: &'static str,
    window_width: f32,
    theme: &OrbitTheme,
    on_close: ShellDrawerVoidCallback,
    children: Vec<AnyElement>,
) -> AnyElement {
    let drawer_width = effective_shell_drawer_width(window_width);

    let mut panel = div()
        .id(panel_id)
        .w(px(drawer_width))
        .h_full()
        .bg(theme.surface.bg_card)
        .border_l_1()
        .border_color(theme.border.default)
        .when(!theme.style.overlay_shadows.is_empty(), |el| {
            el.shadow(theme.style.overlay_shadows.clone())
        })
        .flex()
        .flex_col()
        .on_mouse_down(MouseButton::Left, |_, _, cx| {
            cx.stop_propagation();
        })
        .on_mouse_down(MouseButton::Right, |_, _, cx| {
            cx.stop_propagation();
        });

    for child in children {
        panel = panel.child(child);
    }

    div()
        .id(backdrop_id)
        .absolute()
        .inset_0()
        .flex()
        .justify_end()
        .bg(gpui::rgba(0x00000040))
        .on_mouse_down(MouseButton::Left, move |_, window, cx| {
            cx.stop_propagation();
            on_close(window, cx);
        })
        .child(panel)
        .into_any_element()
}

fn render_small_action_button(
    id: &'static str,
    label: &'static str,
    disabled: bool,
    theme: &OrbitTheme,
    on_click: ShellDrawerVoidCallback,
) -> AnyElement {
    let base = div()
        .id(id)
        .px_2()
        .py_1()
        .rounded(theme.style.radius_control)
        .border_1()
        .border_color(theme.border.default)
        .bg(theme.surface.control_bg);

    let styled = if disabled {
        base.opacity(0.45)
    } else {
        base.cursor_pointer()
            .hover(|s| s.bg(theme.surface.bg_hover))
            .on_mouse_down(MouseButton::Left, move |_, window, cx| {
                cx.stop_propagation();
                on_click(window, cx);
            })
    };

    styled
        .child(
            apply_text_step(div(), theme.typography.size_2xs)
                .font_weight(FontWeight::MEDIUM)
                .text_color(theme.text.secondary)
                .child(label),
        )
        .into_any_element()
}

pub fn render_notification_drawer(
    state: &NotificationCenterState,
    window_width: f32,
    theme: &OrbitTheme,
    callbacks: NotificationDrawerCallbacks,
) -> AnyElement {
    let unread = state.unread_count();
    let total = state.items.len();
    let now_secs = current_unix_secs();

    let header = div()
        .w_full()
        .px_4()
        .py_3()
        .border_b_1()
        .border_color(theme.border.default)
        .flex()
        .items_center()
        .justify_between()
        .gap_2()
        .child(
            div()
                .flex()
                .items_center()
                .gap_2()
                .child(
                    apply_text_step(div(), theme.typography.size_lg)
                        .font_weight(FontWeight::BOLD)
                        .text_color(theme.text.primary)
                        .child("Notifications"),
                )
                .when(unread > 0, |el| {
                    el.child(
                        apply_text_step(div(), theme.typography.size_3xs)
                            .px_1p5()
                            .py(px(1.0))
                            .rounded_full()
                            .bg(theme.semantic.danger.soft)
                            .text_color(theme.semantic.danger.solid)
                            .font_weight(FontWeight::SEMIBOLD)
                            .child(format!("{unread}")),
                    )
                }),
        )
        .child(
            div()
                .flex()
                .items_center()
                .gap_1p5()
                .child(render_small_action_button(
                    "notifications-mark-all-read-btn",
                    "Mark read",
                    unread == 0,
                    theme,
                    callbacks.on_mark_all_read.clone(),
                ))
                .child(render_small_action_button(
                    "notifications-clear-all-btn",
                    "Clear",
                    total == 0,
                    theme,
                    callbacks.on_clear_all.clone(),
                ))
                .child(render_close_button(
                    "notifications-drawer-close-btn",
                    theme,
                    callbacks.on_close.clone(),
                )),
        )
        .into_any_element();

    let filter_bar = div()
        .w_full()
        .px_4()
        .bg(theme.surface.bg_panel)
        .border_b_1()
        .border_color(theme.border.default)
        .flex()
        .items_center()
        .gap_2()
        .children(
            [
                (NotificationFilterTab::All, format!("All ({total})")),
                (NotificationFilterTab::Unread, format!("Unread ({unread})")),
            ]
            .into_iter()
            .enumerate()
            .map(|(idx, (tab, label))| {
                let is_active = state.filter == tab;
                let on_select = callbacks.on_select_filter.clone();
                div()
                    .id(("notifications-filter-tab", idx))
                    .px_2p5()
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
                        on_select(tab, window, cx);
                    })
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
                            .child(label),
                    )
            }),
        )
        .into_any_element();

    let visible = state.visible_items();
    let body = if visible.is_empty() {
        let (empty_icon, empty_title, empty_subtitle) = match state.filter {
            NotificationFilterTab::All => (
                Icon::BellOff,
                "No notifications yet",
                "Cluster errors, workload actions, and exports will appear here.",
            ),
            NotificationFilterTab::Unread => (
                Icon::Check,
                "All caught up",
                "Every notification has been marked as read.",
            ),
        };
        div()
            .id("notifications-empty-state")
            .flex_1()
            .min_h_0()
            .w_full()
            .flex()
            .flex_col()
            .items_center()
            .justify_center()
            .gap_2()
            .px_6()
            .child(render_icon(empty_icon, px(24.0), theme.text.muted))
            .child(
                apply_text_step(div(), theme.typography.size_sm)
                    .font_weight(FontWeight::SEMIBOLD)
                    .text_color(theme.text.primary)
                    .child(empty_title),
            )
            .child(
                apply_text_step(div(), theme.typography.size_xs)
                    .text_color(theme.text.muted)
                    .child(empty_subtitle),
            )
            .into_any_element()
    } else {
        div()
            .id("notifications-list")
            .flex_1()
            .min_h_0()
            .w_full()
            .overflow_y_scroll()
            .p_3()
            .flex()
            .flex_col()
            .gap_2()
            .children(visible.into_iter().map(|item| {
                let (sev_icon, sev_color, sev_bg) = match item.severity {
                    NotificationSeverity::Info => (
                        Icon::Info,
                        theme.semantic.info.solid,
                        theme.semantic.info.soft,
                    ),
                    NotificationSeverity::Success => (
                        Icon::CheckCircle2,
                        theme.semantic.success.solid,
                        theme.semantic.success.soft,
                    ),
                    NotificationSeverity::Warning => (
                        Icon::AlertTriangle,
                        theme.semantic.warning.solid,
                        theme.semantic.warning.soft,
                    ),
                    NotificationSeverity::Error => (
                        Icon::AlertCircle,
                        theme.semantic.danger.solid,
                        theme.semantic.danger.soft,
                    ),
                };

                let age_secs = now_secs.saturating_sub(item.timestamp_secs);
                let rel_time = format_relative_time(age_secs);
                let item_id = item.id;
                let copy_payload = format!("{}: {}", item.title, item.message);
                let on_mark_read = callbacks.on_mark_read.clone();
                let on_remove = callbacks.on_remove.clone();
                let on_copy = callbacks.on_copy.clone();

                div()
                    .id(("notification-item", item_id))
                    .w_full()
                    .p_3()
                    .rounded(theme.style.radius_control)
                    .border_1()
                    .border_color(if item.read {
                        theme.border.default
                    } else {
                        theme.border.strong
                    })
                    .bg(if item.read {
                        theme.surface.bg_panel
                    } else {
                        theme.surface.bg_hover
                    })
                    .cursor_pointer()
                    .hover(|s| s.bg(theme.surface.bg_hover))
                    .on_mouse_down(MouseButton::Left, move |_, window, cx| {
                        cx.stop_propagation();
                        on_mark_read(item_id, window, cx);
                    })
                    .flex()
                    .items_start()
                    .gap_2p5()
                    .child(
                        div()
                            .w(px(24.0))
                            .h(px(24.0))
                            .rounded(theme.style.radius_control)
                            .bg(sev_bg)
                            .flex()
                            .items_center()
                            .justify_center()
                            .flex_shrink_0()
                            .child(render_icon(sev_icon, px(13.0), sev_color)),
                    )
                    .child(
                        div()
                            .flex_1()
                            .min_w_0()
                            .flex()
                            .flex_col()
                            .gap_1()
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
                                            .gap_1p5()
                                            .min_w_0()
                                            .when(!item.read, |el| {
                                                el.child(
                                                    div()
                                                        .w(px(6.0))
                                                        .h(px(6.0))
                                                        .rounded_full()
                                                        .flex_shrink_0()
                                                        .bg(theme.accent.accent),
                                                )
                                            })
                                            .child(
                                                apply_text_step(div(), theme.typography.size_xs)
                                                    .font_weight(FontWeight::SEMIBOLD)
                                                    .text_color(theme.text.primary)
                                                    .truncate()
                                                    .child(item.title.clone()),
                                            ),
                                    )
                                    .child(
                                        apply_text_step(div(), theme.typography.size_3xs)
                                            .text_color(theme.text.muted)
                                            .flex_shrink_0()
                                            .child(rel_time),
                                    ),
                            )
                            .child(
                                apply_text_step(div(), theme.typography.size_xs)
                                    .text_color(theme.text.secondary)
                                    .child(item.message.clone()),
                            ),
                    )
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_1()
                            .flex_shrink_0()
                            .child(
                                div()
                                    .id(("notification-copy-btn", item_id))
                                    .w(px(22.0))
                                    .h(px(22.0))
                                    .rounded(theme.style.radius_control)
                                    .flex()
                                    .items_center()
                                    .justify_center()
                                    .cursor_pointer()
                                    .hover(|s| s.bg(theme.surface.control_bg))
                                    .tooltip(|_, cx| cx.new(|_| TextTooltip::new("Copy")).into())
                                    .on_mouse_down(MouseButton::Left, move |_, window, cx| {
                                        cx.stop_propagation();
                                        on_copy(copy_payload.clone(), window, cx);
                                    })
                                    .child(render_icon(Icon::Copy, px(12.0), theme.text.muted)),
                            )
                            .child(
                                div()
                                    .id(("notification-remove-btn", item_id))
                                    .w(px(22.0))
                                    .h(px(22.0))
                                    .rounded(theme.style.radius_control)
                                    .flex()
                                    .items_center()
                                    .justify_center()
                                    .cursor_pointer()
                                    .hover(|s| s.bg(theme.surface.control_bg))
                                    .tooltip(|_, cx| cx.new(|_| TextTooltip::new("Remove")).into())
                                    .on_mouse_down(MouseButton::Left, move |_, window, cx| {
                                        cx.stop_propagation();
                                        on_remove(item_id, window, cx);
                                    })
                                    .child(render_icon(Icon::X, px(12.0), theme.text.muted)),
                            ),
                    )
            }))
            .into_any_element()
    };

    render_shell_drawer_backdrop(
        "notifications-drawer-backdrop",
        "notifications-drawer-panel",
        window_width,
        theme,
        callbacks.on_close,
        vec![header, filter_bar, body],
    )
}

fn render_profile_field(
    label: &'static str,
    value: String,
    monospace: bool,
    theme: &OrbitTheme,
) -> AnyElement {
    div()
        .w_full()
        .flex()
        .items_center()
        .justify_between()
        .gap_3()
        .child(
            apply_text_step(div(), theme.typography.size_xs)
                .text_color(theme.text.muted)
                .flex_shrink_0()
                .child(label),
        )
        .child(
            apply_text_step(div(), theme.typography.size_xs)
                .text_color(theme.text.primary)
                .when(monospace, |el| {
                    el.font(theme.typography.mono_font(FontWeight::NORMAL))
                })
                .truncate()
                .child(value),
        )
        .into_any_element()
}

pub fn render_profile_drawer(
    profile: Option<&UserProfileInfo>,
    clusters: &[ClusterInfo],
    active_cluster_id: Option<&str>,
    window_width: f32,
    theme: &OrbitTheme,
    callbacks: ProfileDrawerCallbacks,
) -> AnyElement {
    let k8s_version = profile
        .and_then(|p| p.k8s_version.as_ref())
        .filter(|v| !v.trim().is_empty())
        .cloned();

    let header = div()
        .w_full()
        .px_4()
        .py_3()
        .border_b_1()
        .border_color(theme.border.default)
        .flex()
        .items_center()
        .justify_between()
        .gap_2()
        .child(
            div()
                .flex()
                .items_center()
                .gap_2()
                .child(
                    apply_text_step(div(), theme.typography.size_lg)
                        .font_weight(FontWeight::BOLD)
                        .text_color(theme.text.primary)
                        .child("User Profile"),
                )
                .when_some(k8s_version, |el, version| {
                    el.child(
                        apply_text_step(div(), theme.typography.size_3xs)
                            .px_1p5()
                            .py(px(1.0))
                            .rounded(theme.style.radius_control)
                            .bg(theme.surface.control_bg)
                            .text_color(theme.text.secondary)
                            .font_weight(FontWeight::SEMIBOLD)
                            .child(version),
                    )
                }),
        )
        .child(render_close_button(
            "profile-drawer-close-btn",
            theme,
            callbacks.on_close.clone(),
        ))
        .into_any_element();

    let identity_section = {
        let card = div()
            .w_full()
            .p_3()
            .rounded(theme.style.radius_control)
            .bg(theme.surface.bg_panel)
            .border_1()
            .border_color(theme.border.default)
            .flex()
            .flex_col()
            .gap_2();

        let card_content = if let Some(p) = profile {
            let user_name = p
                .user_name
                .as_deref()
                .filter(|s| !s.is_empty())
                .unwrap_or("Default Identity")
                .to_string();
            let active_context = p
                .active_context
                .as_deref()
                .filter(|s| !s.is_empty())
                .unwrap_or("—")
                .to_string();
            let cluster_name = p
                .cluster_name
                .as_deref()
                .filter(|s| !s.is_empty())
                .unwrap_or("—")
                .to_string();
            let server_url = p
                .server_url
                .as_deref()
                .filter(|s| !s.is_empty())
                .unwrap_or("—")
                .to_string();

            card.child(
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
                            .min_w_0()
                            .child(render_icon(Icon::User, px(15.0), theme.text.secondary))
                            .child(
                                apply_text_step(div(), theme.typography.size_sm)
                                    .font_weight(FontWeight::SEMIBOLD)
                                    .text_color(theme.text.primary)
                                    .truncate()
                                    .child(user_name),
                            ),
                    )
                    .child(
                        apply_text_step(div(), theme.typography.size_3xs)
                            .px_1p5()
                            .py(px(1.0))
                            .rounded(theme.style.radius_control)
                            .bg(theme.accent.soft)
                            .text_color(theme.accent.accent)
                            .font_weight(FontWeight::SEMIBOLD)
                            .child(p.auth_type.clone()),
                    ),
            )
            .child(render_profile_field("Context", active_context, true, theme))
            .child(render_profile_field("Cluster", cluster_name, false, theme))
            .child(render_profile_field("Server", server_url, true, theme))
        } else {
            card.child(
                apply_text_step(div(), theme.typography.size_xs)
                    .text_color(theme.text.muted)
                    .child("Not connected to a cluster. Select a cluster below to load identity details."),
            )
        };

        div()
            .w_full()
            .flex()
            .flex_col()
            .gap_2()
            .child(
                apply_text_step(div(), theme.typography.size_2xs)
                    .font_weight(FontWeight::SEMIBOLD)
                    .text_color(theme.text.muted)
                    .child("ACTIVE IDENTITY"),
            )
            .child(card_content)
    };

    let kubeconfig_paths = profile
        .map(|p| p.kubeconfig_paths.clone())
        .unwrap_or_default();

    let kubeconfig_section = div()
        .w_full()
        .flex()
        .flex_col()
        .gap_2()
        .child(
            apply_text_step(div(), theme.typography.size_2xs)
                .font_weight(FontWeight::SEMIBOLD)
                .text_color(theme.text.muted)
                .child("KUBECONFIG PATHS"),
        )
        .child(if kubeconfig_paths.is_empty() {
            apply_text_step(div(), theme.typography.size_xs)
                .text_color(theme.text.muted)
                .child("No kubeconfig paths loaded.")
                .into_any_element()
        } else {
            div()
                .w_full()
                .flex()
                .flex_col()
                .gap_1p5()
                .children(kubeconfig_paths.into_iter().map(|path| {
                    div()
                        .w_full()
                        .px_2p5()
                        .py_1p5()
                        .rounded(theme.style.radius_control)
                        .bg(theme.surface.bg_panel)
                        .flex()
                        .items_center()
                        .gap_2()
                        .child(render_icon(Icon::FileText, px(13.0), theme.text.muted))
                        .child(
                            apply_text_step(div(), theme.typography.size_xs)
                                .font(theme.typography.mono_font(FontWeight::NORMAL))
                                .text_color(theme.text.secondary)
                                .truncate()
                                .child(path),
                        )
                }))
                .into_any_element()
        });

    let clusters_section = div()
        .w_full()
        .flex()
        .flex_col()
        .gap_2()
        .child(
            div()
                .w_full()
                .flex()
                .items_center()
                .justify_between()
                .child(
                    apply_text_step(div(), theme.typography.size_2xs)
                        .font_weight(FontWeight::SEMIBOLD)
                        .text_color(theme.text.muted)
                        .child(format!("AVAILABLE CLUSTERS ({})", clusters.len())),
                )
                .child(render_small_action_button(
                    "profile-add-cluster-btn",
                    "Add Cluster",
                    false,
                    theme,
                    callbacks.on_add_cluster.clone(),
                )),
        )
        .child(if clusters.is_empty() {
            apply_text_step(div(), theme.typography.size_xs)
                .text_color(theme.text.muted)
                .child("No kubeconfig contexts available.")
                .into_any_element()
        } else {
            div()
                .w_full()
                .flex()
                .flex_col()
                .gap_1()
                .children(clusters.iter().enumerate().map(|(idx, cluster)| {
                    let is_active = active_cluster_id == Some(cluster.id.as_str());
                    let display_name = cluster
                        .alias
                        .as_deref()
                        .filter(|s| !s.is_empty())
                        .unwrap_or(&cluster.name)
                        .to_string();
                    let cluster_id = cluster.id.clone();
                    let on_switch = callbacks.on_switch_cluster.clone();

                    div()
                        .id(("profile-cluster-row", idx))
                        .w_full()
                        .px_2p5()
                        .py_2()
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
                        .cursor_pointer()
                        .hover(|s| s.bg(theme.surface.bg_hover))
                        .on_mouse_down(MouseButton::Left, move |_, window, cx| {
                            cx.stop_propagation();
                            on_switch(cluster_id.clone(), window, cx);
                        })
                        .flex()
                        .items_center()
                        .justify_between()
                        .gap_2()
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .gap_2()
                                .min_w_0()
                                .child(render_icon(
                                    Icon::Server,
                                    px(13.0),
                                    if is_active {
                                        theme.accent.accent
                                    } else {
                                        theme.text.muted
                                    },
                                ))
                                .child(
                                    apply_text_step(div(), theme.typography.size_xs)
                                        .font_weight(if is_active {
                                            FontWeight::SEMIBOLD
                                        } else {
                                            FontWeight::MEDIUM
                                        })
                                        .text_color(theme.text.primary)
                                        .truncate()
                                        .child(display_name),
                                ),
                        )
                        .when(is_active, |el| {
                            el.child(
                                apply_text_step(div(), theme.typography.size_3xs)
                                    .text_color(theme.accent.accent)
                                    .font_weight(FontWeight::SEMIBOLD)
                                    .child("Active"),
                            )
                        })
                }))
                .into_any_element()
        });

    let body = div()
        .id("profile-drawer-body")
        .flex_1()
        .min_h_0()
        .w_full()
        .overflow_y_scroll()
        .p_4()
        .flex()
        .flex_col()
        .gap_5()
        .child(identity_section)
        .child(kubeconfig_section)
        .child(clusters_section)
        .into_any_element();

    let footer = div()
        .w_full()
        .px_4()
        .py_3()
        .border_t_1()
        .border_color(theme.border.default)
        .bg(theme.surface.bg_panel)
        .flex()
        .items_center()
        .justify_end()
        .gap_2()
        .child(render_small_action_button(
            "profile-refresh-btn",
            "Refresh",
            false,
            theme,
            callbacks.on_refresh,
        ))
        .child(render_small_action_button(
            "profile-settings-btn",
            "Settings",
            false,
            theme,
            callbacks.on_open_settings,
        ))
        .into_any_element();

    render_shell_drawer_backdrop(
        "profile-drawer-backdrop",
        "profile-drawer-panel",
        window_width,
        theme,
        callbacks.on_close,
        vec![header, body, footer],
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_active_shell_drawer_toggle_and_mutual_exclusion() {
        let mut drawer = ActiveShellDrawer::default();
        assert_eq!(drawer, ActiveShellDrawer::None);
        assert!(!drawer.close());

        drawer.toggle(ActiveShellDrawer::Notifications);
        assert_eq!(drawer, ActiveShellDrawer::Notifications);

        drawer.toggle(ActiveShellDrawer::UserProfile);
        assert_eq!(drawer, ActiveShellDrawer::UserProfile);

        drawer.toggle(ActiveShellDrawer::UserProfile);
        assert_eq!(drawer, ActiveShellDrawer::None);

        drawer.toggle(ActiveShellDrawer::Notifications);
        assert!(drawer.close());
        assert_eq!(drawer, ActiveShellDrawer::None);
        assert!(!drawer.close());
    }

    #[test]
    fn test_notification_center_push_unread_mark_read_remove_and_cap() {
        let mut state = NotificationCenterState::default();
        assert_eq!(state.unread_count(), 0);
        assert!(state.visible_items().is_empty());

        let first_id = state.push(NotificationSeverity::Info, "First", "Msg 1", 100);
        let second_id = state.push(NotificationSeverity::Error, "Second", "Msg 2", 200);
        assert_eq!(state.items.len(), 2);
        assert_eq!(state.items[0].id, second_id);
        assert_eq!(state.items[1].id, first_id);
        assert_eq!(state.unread_count(), 2);

        assert!(state.mark_read(first_id));
        assert!(!state.mark_read(first_id));
        assert_eq!(state.unread_count(), 1);

        state.filter = NotificationFilterTab::Unread;
        let unread_visible = state.visible_items();
        assert_eq!(unread_visible.len(), 1);
        assert_eq!(unread_visible[0].id, second_id);

        assert!(state.mark_all_read());
        assert_eq!(state.unread_count(), 0);
        assert!(!state.mark_all_read());

        assert!(state.remove(first_id));
        assert!(!state.remove(first_id));
        assert_eq!(state.items.len(), 1);

        assert!(state.clear_all());
        assert!(state.items.is_empty());
        assert!(!state.clear_all());

        for idx in 0..(MAX_NOTIFICATIONS + 15) {
            state.push(
                NotificationSeverity::Success,
                format!("Item {idx}"),
                "body",
                idx as u64,
            );
        }
        assert_eq!(state.items.len(), MAX_NOTIFICATIONS);
        assert_eq!(
            state.items[0].title,
            format!("Item {}", MAX_NOTIFICATIONS + 14)
        );
    }

    #[test]
    fn test_format_relative_time_buckets() {
        assert_eq!(format_relative_time(0), "just now");
        assert_eq!(format_relative_time(59), "just now");
        assert_eq!(format_relative_time(120), "2m ago");
        assert_eq!(format_relative_time(7_200), "2h ago");
        assert_eq!(format_relative_time(172_800), "2d ago");
    }

    #[test]
    fn test_notification_button_tooltip_reflects_unread_count() {
        assert_eq!(notifications_tooltip(0), "Notifications");
        assert_eq!(notifications_tooltip(3), "Notifications (3 unread)");
    }
}
