use std::rc::Rc;

use gpui::{
    AnyElement, App, FocusHandle, FontWeight, MouseButton, SharedString, Window, div, prelude::*,
    px,
};
use lucide_icons::Icon;
use orbit_core::kubernetes::models::ClusterInfo;

use crate::icons::render_icon;
use crate::navigation::WorkspaceView;
use crate::state::NamespaceFilterState;
use crate::theme::{OrbitTheme, apply_text_step};

pub type CommandPaletteActionCallback = Rc<dyn Fn(CommandPaletteAction, &mut Window, &mut App)>;
pub type CommandPaletteVoidCallback = Rc<dyn Fn(&mut Window, &mut App)>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommandPaletteCategory {
    View,
    Cluster,
    Namespace,
    Action,
}

impl CommandPaletteCategory {
    pub const fn label(self) -> &'static str {
        match self {
            Self::View => "Views",
            Self::Cluster => "Clusters",
            Self::Namespace => "Namespaces",
            Self::Action => "Actions",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CommandPaletteAction {
    NavigateView(WorkspaceView),
    SwitchCluster(String),
    SelectAllNamespaces,
    ToggleNamespace(String),
    RefreshWorkspace,
    ToggleSidebar,
    OpenSettings,
    AddCluster,
    DisconnectCluster,
    ClearNamespaceFilter,
}

#[derive(Debug, Clone)]
pub struct CommandPaletteItem {
    pub id: SharedString,
    pub category: CommandPaletteCategory,
    pub label: SharedString,
    pub subtitle: SharedString,
    pub keywords: Vec<String>,
    pub icon: Icon,
    pub shortcut: Option<&'static str>,
    pub is_active: bool,
    pub action: CommandPaletteAction,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommandPaletteKeyResult {
    Ignored,
    Handled,
    Close,
    Execute(usize),
}

#[derive(Debug, Clone, Default)]
pub struct CommandPaletteState {
    pub is_open: bool,
    pub query: String,
    pub selected_index: usize,
}

impl CommandPaletteState {
    pub fn open(&mut self) {
        self.is_open = true;
        self.query.clear();
        self.selected_index = 0;
    }

    pub fn close(&mut self) {
        self.is_open = false;
        self.query.clear();
        self.selected_index = 0;
    }

    pub fn toggle(&mut self) -> bool {
        if self.is_open {
            self.close();
            false
        } else {
            self.open();
            true
        }
    }

    pub fn clear_query(&mut self) {
        self.query.clear();
        self.selected_index = 0;
    }

    pub fn filter_items(&self, catalog: &[CommandPaletteItem]) -> Vec<CommandPaletteItem> {
        let trimmed = self.query.trim();
        if trimmed.is_empty() {
            return catalog.to_vec();
        }

        let mut scored: Vec<(i32, usize, CommandPaletteItem)> = catalog
            .iter()
            .enumerate()
            .filter_map(|(idx, item)| {
                let mut best = fuzzy_match_score(trimmed, item.label.as_ref()).map(|s| s + 40);
                if let Some(sub_score) = fuzzy_match_score(trimmed, item.subtitle.as_ref()) {
                    best = Some(best.map_or(sub_score, |cur| cur.max(sub_score)));
                }
                if let Some(cat_score) = fuzzy_match_score(trimmed, item.category.label()) {
                    let adjusted = cat_score - 20;
                    best = Some(best.map_or(adjusted, |cur| cur.max(adjusted)));
                }
                for kw in &item.keywords {
                    if let Some(kw_score) = fuzzy_match_score(trimmed, kw) {
                        let adjusted = kw_score + 10;
                        best = Some(best.map_or(adjusted, |cur| cur.max(adjusted)));
                    }
                }
                best.map(|score| (score, idx, item.clone()))
            })
            .collect();

        scored.sort_by(|a, b| b.0.cmp(&a.0).then_with(|| a.1.cmp(&b.1)));
        scored.into_iter().map(|(_, _, item)| item).collect()
    }

    pub fn handle_key_input(
        &mut self,
        key: &str,
        key_char: Option<&str>,
        filtered_len: usize,
    ) -> CommandPaletteKeyResult {
        if !self.is_open {
            return CommandPaletteKeyResult::Ignored;
        }
        match key {
            "escape" => {
                self.close();
                CommandPaletteKeyResult::Close
            }
            "enter" => {
                if filtered_len > 0 {
                    CommandPaletteKeyResult::Execute(self.selected_index.min(filtered_len - 1))
                } else {
                    CommandPaletteKeyResult::Handled
                }
            }
            "down" => {
                if filtered_len > 0 {
                    self.selected_index = (self.selected_index + 1) % filtered_len;
                }
                CommandPaletteKeyResult::Handled
            }
            "up" => {
                if filtered_len > 0 {
                    self.selected_index = if self.selected_index == 0 {
                        filtered_len - 1
                    } else {
                        self.selected_index - 1
                    };
                }
                CommandPaletteKeyResult::Handled
            }
            "backspace" => {
                self.query.pop();
                self.selected_index = 0;
                CommandPaletteKeyResult::Handled
            }
            _ => {
                if let Some(ch_str) = key_char
                    && !ch_str.is_empty()
                    && !ch_str.chars().any(char::is_control)
                {
                    self.query.push_str(ch_str);
                    self.selected_index = 0;
                }
                CommandPaletteKeyResult::Handled
            }
        }
    }
}

pub fn fuzzy_match_score(query: &str, candidate: &str) -> Option<i32> {
    let q = query.trim().to_ascii_lowercase();
    if q.is_empty() {
        return Some(0);
    }
    let c = candidate.to_ascii_lowercase();
    if c == q {
        return Some(1000);
    }
    if c.starts_with(&q) {
        let len_penalty = (c.len() as i32 - q.len() as i32).max(0);
        return Some(500 - len_penalty);
    }
    if let Some(pos) = c.find(&q) {
        let is_boundary = pos == 0
            || !c.as_bytes()[pos - 1].is_ascii_alphanumeric();
        let boundary_bonus = if is_boundary { 60 } else { 0 };
        let len_penalty = (c.len() as i32 - q.len() as i32).max(0);
        return Some(300 + boundary_bonus - (pos as i32) * 2 - len_penalty);
    }

    let q_chars: Vec<char> = q.chars().collect();
    let c_chars: Vec<char> = c.chars().collect();
    let mut qi = 0;
    let mut score: i32 = 100;
    let mut prev_match: Option<usize> = None;

    for (ci, &ch) in c_chars.iter().enumerate() {
        if qi < q_chars.len() && ch == q_chars[qi] {
            if let Some(prev) = prev_match {
                if ci == prev + 1 {
                    score += 25;
                } else {
                    score -= ((ci - prev - 1) as i32) * 2;
                }
            }
            if ci == 0 || !c_chars[ci - 1].is_ascii_alphanumeric() {
                score += 40;
            }
            prev_match = Some(ci);
            qi += 1;
        }
    }

    if qi == q_chars.len() {
        let len_penalty = (c_chars.len() as i32 - q_chars.len() as i32).max(0);
        Some(score - len_penalty)
    } else {
        None
    }
}

pub fn build_command_palette_items(
    clusters: &[ClusterInfo],
    active_cluster_id: Option<&str>,
    namespaces: &NamespaceFilterState,
) -> Vec<CommandPaletteItem> {
    let mut items = Vec::new();
    let sidebar_shortcut = if cfg!(target_os = "macos") {
        "⌘B"
    } else {
        "Ctrl+B"
    };

    // 1. Workspace Views
    for view in WorkspaceView::ALL {
        if view == WorkspaceView::Settings {
            continue;
        }
        let category_label = view
            .category()
            .map(|c| c.label())
            .unwrap_or("Application");
        let label = match view {
            WorkspaceView::Overview => "Cluster Overview".to_string(),
            WorkspaceView::WorkloadsOverview => "Workloads Overview".to_string(),
            WorkspaceView::StorageOverview => "Storage Overview".to_string(),
            WorkspaceView::SecurityOverview => "Security Overview".to_string(),
            _ => view.label().to_string(),
        };
        let mut keywords: Vec<String> = view
            .search_keywords()
            .iter()
            .map(|s| (*s).to_string())
            .collect();
        keywords.push(category_label.to_string());

        items.push(CommandPaletteItem {
            id: SharedString::from(format!("cmd-view-{}", view.id())),
            category: CommandPaletteCategory::View,
            label: SharedString::from(label),
            subtitle: SharedString::from(category_label),
            keywords,
            icon: view.icon(),
            shortcut: None,
            is_active: false,
            action: CommandPaletteAction::NavigateView(view),
        });
    }

    // 2. Clusters
    for cluster in clusters {
        let display_name = cluster
            .alias
            .as_deref()
            .filter(|a| !a.trim().is_empty())
            .unwrap_or(&cluster.name);
        let is_active = active_cluster_id == Some(cluster.id.as_str());
        let subtitle = if is_active {
            "Active cluster".to_string()
        } else {
            cluster.name.clone()
        };
        items.push(CommandPaletteItem {
            id: SharedString::from(format!("cmd-cluster-{}", cluster.id)),
            category: CommandPaletteCategory::Cluster,
            label: SharedString::from(display_name.to_string()),
            subtitle: SharedString::from(subtitle),
            keywords: vec![
                "cluster".to_string(),
                "context".to_string(),
                cluster.id.clone(),
                cluster.name.clone(),
            ],
            icon: Icon::Server,
            shortcut: None,
            is_active,
            action: CommandPaletteAction::SwitchCluster(cluster.id.clone()),
        });
    }

    // 3. Namespaces (when connected)
    if active_cluster_id.is_some() {
        items.push(CommandPaletteItem {
            id: SharedString::from("cmd-ns-all"),
            category: CommandPaletteCategory::Namespace,
            label: SharedString::from("All Namespaces"),
            subtitle: SharedString::from("Show resources across all namespaces"),
            keywords: vec!["ns".to_string(), "namespace".to_string(), "all".to_string()],
            icon: Icon::FolderOpen,
            shortcut: None,
            is_active: namespaces.is_all_namespaces(),
            action: CommandPaletteAction::SelectAllNamespaces,
        });

        let selected_ns = namespaces.active_selection();
        for ns in &namespaces.namespaces {
            let is_selected = selected_ns.iter().any(|s| s == &ns.name);
            items.push(CommandPaletteItem {
                id: SharedString::from(format!("cmd-ns-{}", ns.name)),
                category: CommandPaletteCategory::Namespace,
                label: SharedString::from(ns.name.clone()),
                subtitle: SharedString::from(if is_selected {
                    "Selected namespace"
                } else {
                    "Filter by namespace"
                }),
                keywords: vec!["ns".to_string(), "namespace".to_string()],
                icon: Icon::FolderOpen,
                shortcut: None,
                is_active: is_selected,
                action: CommandPaletteAction::ToggleNamespace(ns.name.clone()),
            });
        }
    }

    // 4. Global Actions
    items.push(CommandPaletteItem {
        id: SharedString::from("cmd-action-refresh"),
        category: CommandPaletteCategory::Action,
        label: SharedString::from("Refresh Workspace"),
        subtitle: SharedString::from("Reload active view data"),
        keywords: vec!["reload".to_string(), "sync".to_string(), "fetch".to_string()],
        icon: Icon::RefreshCw,
        shortcut: Some("R"),
        is_active: false,
        action: CommandPaletteAction::RefreshWorkspace,
    });
    items.push(CommandPaletteItem {
        id: SharedString::from("cmd-action-toggle-sidebar"),
        category: CommandPaletteCategory::Action,
        label: SharedString::from("Toggle Sidebar"),
        subtitle: SharedString::from("Show or collapse navigation panel"),
        keywords: vec!["panel".to_string(), "collapse".to_string(), "navigation".to_string()],
        icon: Icon::PanelLeft,
        shortcut: Some(sidebar_shortcut),
        is_active: false,
        action: CommandPaletteAction::ToggleSidebar,
    });
    items.push(CommandPaletteItem {
        id: SharedString::from("cmd-action-settings"),
        category: CommandPaletteCategory::Action,
        label: SharedString::from("Open Settings"),
        subtitle: SharedString::from("Appearance, preferences, and cluster config"),
        keywords: vec!["preferences".to_string(), "theme".to_string(), "config".to_string()],
        icon: Icon::Settings,
        shortcut: None,
        is_active: false,
        action: CommandPaletteAction::OpenSettings,
    });
    items.push(CommandPaletteItem {
        id: SharedString::from("cmd-action-add-cluster"),
        category: CommandPaletteCategory::Action,
        label: SharedString::from("Add Cluster"),
        subtitle: SharedString::from("Import a kubeconfig context"),
        keywords: vec!["import".to_string(), "kubeconfig".to_string(), "connect".to_string()],
        icon: Icon::Plus,
        shortcut: None,
        is_active: false,
        action: CommandPaletteAction::AddCluster,
    });
    if active_cluster_id.is_some() {
        items.push(CommandPaletteItem {
            id: SharedString::from("cmd-action-disconnect"),
            category: CommandPaletteCategory::Action,
            label: SharedString::from("Disconnect Cluster"),
            subtitle: SharedString::from("Disconnect from the active cluster"),
            keywords: vec!["logout".to_string(), "close".to_string()],
            icon: Icon::Unplug,
            shortcut: None,
            is_active: false,
            action: CommandPaletteAction::DisconnectCluster,
        });
    }
    if !namespaces.is_all_namespaces() {
        items.push(CommandPaletteItem {
            id: SharedString::from("cmd-action-clear-ns"),
            category: CommandPaletteCategory::Action,
            label: SharedString::from("Clear Namespace Filter"),
            subtitle: SharedString::from("Reset scope to All Namespaces"),
            keywords: vec!["reset".to_string(), "namespace".to_string()],
            icon: Icon::FilterX,
            shortcut: None,
            is_active: false,
            action: CommandPaletteAction::ClearNamespaceFilter,
        });
    }

    items
}

#[derive(Clone)]
pub struct CommandPaletteCallbacks {
    pub on_execute: CommandPaletteActionCallback,
    pub on_clear_query: CommandPaletteVoidCallback,
    pub on_close: CommandPaletteVoidCallback,
}

pub fn render_command_palette_overlay(
    state: &CommandPaletteState,
    filtered_items: &[CommandPaletteItem],
    theme: &OrbitTheme,
    focus_handle: &FocusHandle,
    is_focused: bool,
    callbacks: CommandPaletteCallbacks,
) -> AnyElement {
    let CommandPaletteCallbacks {
        on_execute,
        on_clear_query,
        on_close,
    } = callbacks;
    let is_empty_query = state.query.is_empty();
    let caret = || {
        div()
            .w(px(1.5))
            .h(px(14.0))
            .flex_shrink_0()
            .bg(theme.accent.accent)
    };

    let search_text = if is_empty_query {
        div()
            .flex()
            .items_center()
            .gap_0p5()
            .min_w_0()
            .when(is_focused, |el| el.child(caret()))
            .child(
                apply_text_step(div(), theme.typography.size_sm)
                    .text_color(theme.text.muted)
                    .truncate()
                    .child("Jump to views, clusters, namespaces, or actions..."),
            )
    } else {
        div()
            .flex()
            .items_center()
            .min_w_0()
            .child(
                apply_text_step(div(), theme.typography.size_sm)
                    .text_color(theme.text.primary)
                    .truncate()
                    .child(state.query.clone()),
            )
            .when(is_focused, |el| el.child(caret()))
    };

    let card_focus = focus_handle.clone();
    let input_focus = focus_handle.clone();
    let clamped_selected = if filtered_items.is_empty() {
        0
    } else {
        state.selected_index.min(filtered_items.len() - 1)
    };

    let mut list = div()
        .id("command-palette-results")
        .max_h(px(360.0))
        .overflow_y_scroll()
        .flex()
        .flex_col()
        .gap_0p5()
        .p_1p5();

    if filtered_items.is_empty() {
        list = list.child(
            div()
                .px_3()
                .py_8()
                .flex()
                .items_center()
                .justify_center()
                .child(
                    apply_text_step(div(), theme.typography.size_xs)
                        .text_color(theme.text.muted)
                        .child("No matching commands or resources."),
                ),
        );
    } else {
        let mut last_category: Option<CommandPaletteCategory> = None;
        for (idx, item) in filtered_items.iter().enumerate() {
            if last_category != Some(item.category) {
                last_category = Some(item.category);
                list = list.child(
                    apply_text_step(div(), theme.typography.size_3xs)
                        .px_2p5()
                        .pt_2()
                        .pb_1()
                        .font_weight(FontWeight::SEMIBOLD)
                        .text_color(theme.text.muted)
                        .child(item.category.label().to_ascii_uppercase()),
                );
            }

            let is_selected = idx == clamped_selected;
            let on_exec = on_execute.clone();
            let action = item.action.clone();
            let hover_bg = theme.surface.bg_hover;

            list = list.child(
                div()
                    .id(item.id.clone())
                    .px_2p5()
                    .py_1p5()
                    .rounded(theme.style.radius_control)
                    .flex()
                    .items_center()
                    .justify_between()
                    .gap_2()
                    .cursor_pointer()
                    .when(is_selected, |el| el.bg(theme.accent.soft))
                    .hover(move |s| s.bg(if is_selected { theme.accent.soft } else { hover_bg }))
                    .on_mouse_down(MouseButton::Left, move |_event, window, cx| {
                        window.prevent_default();
                        cx.stop_propagation();
                        on_exec(action.clone(), window, cx);
                    })
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_2()
                            .min_w_0()
                            .child(render_icon(
                                item.icon,
                                px(14.0),
                                if is_selected || item.is_active {
                                    theme.accent.accent
                                } else {
                                    theme.text.secondary
                                },
                            ))
                            .child(
                                apply_text_step(div(), theme.typography.size_xs)
                                    .font_weight(FontWeight::MEDIUM)
                                    .text_color(theme.text.primary)
                                    .truncate()
                                    .child(item.label.clone()),
                            )
                            .when(item.is_active, |el| {
                                el.child(
                                    apply_text_step(div(), theme.typography.size_3xs)
                                        .px_1p5()
                                        .py(px(1.0))
                                        .rounded(theme.style.radius_control)
                                        .bg(theme.accent.soft)
                                        .text_color(theme.accent.accent)
                                        .font_weight(FontWeight::SEMIBOLD)
                                        .child("Active"),
                                )
                            }),
                    )
                    .child(
                        div()
                            .flex_shrink_0()
                            .flex()
                            .items_center()
                            .gap_2()
                            .child(
                                apply_text_step(div(), theme.typography.size_xs)
                                    .text_color(theme.text.muted)
                                    .child(item.subtitle.clone()),
                            )
                            .when_some(item.shortcut, |el, shortcut| {
                                el.child(
                                    apply_text_step(div(), theme.typography.size_3xs)
                                        .px_1p5()
                                        .py(px(1.0))
                                        .rounded(theme.style.radius_control)
                                        .bg(theme.surface.control_bg)
                                        .border_1()
                                        .border_color(theme.border.default)
                                        .font(theme.typography.mono_font(FontWeight::MEDIUM))
                                        .text_color(theme.text.muted)
                                        .child(shortcut),
                                )
                            }),
                    ),
            );
        }
    }

    let card = theme
        .style_card(div())
        .id("command-palette-card")
        .w(px(560.0))
        .bg(theme.surface.bg_card)
        .border_1()
        .border_color(theme.border.strong)
        .shadow_lg()
        .flex()
        .flex_col()
        .overflow_hidden()
        .on_mouse_down(MouseButton::Left, move |_event, window, cx| {
            window.prevent_default();
            cx.stop_propagation();
            card_focus.focus(window, cx);
        })
        .child(
            div()
                .id("command-palette-input")
                .track_focus(focus_handle)
                .px_3p5()
                .py_3()
                .border_b_1()
                .border_color(theme.border.default)
                .flex()
                .items_center()
                .gap_2p5()
                .cursor_text()
                .on_mouse_down(MouseButton::Left, move |_event, window, cx| {
                    window.prevent_default();
                    cx.stop_propagation();
                    input_focus.focus(window, cx);
                })
                .child(render_icon(
                    Icon::Search,
                    px(15.0),
                    if is_focused {
                        theme.accent.accent
                    } else {
                        theme.text.muted
                    },
                ))
                .child(div().flex_1().min_w_0().child(search_text))
                .when(!is_empty_query, |el| {
                    el.child(
                        div()
                            .id("command-palette-clear-btn")
                            .w(px(20.0))
                            .h(px(20.0))
                            .rounded(theme.style.radius_control)
                            .flex()
                            .items_center()
                            .justify_center()
                            .cursor_pointer()
                            .hover(|s| s.bg(theme.surface.bg_hover))
                            .on_mouse_down(MouseButton::Left, move |_event, window, cx| {
                                window.prevent_default();
                                cx.stop_propagation();
                                on_clear_query(window, cx);
                            })
                            .child(render_icon(Icon::X, px(12.0), theme.text.muted)),
                    )
                }),
        )
        .child(list)
        .child(
            div()
                .px_3p5()
                .py_2()
                .border_t_1()
                .border_color(theme.border.default)
                .bg(theme.surface.bg_panel)
                .flex()
                .items_center()
                .justify_between()
                .child(
                    apply_text_step(div(), theme.typography.size_3xs)
                        .text_color(theme.text.muted)
                        .child("↑↓ Navigate  ·  ↵ Select  ·  Esc Close"),
                )
                .child(
                    apply_text_step(div(), theme.typography.size_3xs)
                        .text_color(theme.text.muted)
                        .child(format!("{} results", filtered_items.len())),
                ),
        );

    div()
        .id("command-palette-backdrop")
        .absolute()
        .inset_0()
        .bg(theme.surface.overlay_bg)
        .flex()
        .justify_center()
        .items_start()
        .pt(px(72.0))
        .on_mouse_down(MouseButton::Left, move |_event, window, cx| {
            window.prevent_default();
            cx.stop_propagation();
            on_close(window, cx);
        })
        .child(card)
        .into_any_element()
}

#[cfg(test)]
mod tests {
    use super::*;
    use orbit_core::kubernetes::models::NamespaceInfo;

    fn sample_cluster(id: &str, name: &str) -> ClusterInfo {
        ClusterInfo {
            id: id.to_string(),
            name: name.to_string(),
            status: "disconnected".to_string(),
            alias: None,
            tags: Vec::new(),
            server_url: Some("https://127.0.0.1:6443".to_string()),
            color: None,
            description: None,
        }
    }

    fn sample_namespace(name: &str) -> NamespaceInfo {
        NamespaceInfo {
            name: name.to_string(),
            status: "Active".to_string(),
            age: "1d".to_string(),
            labels: Default::default(),
            annotations: Default::default(),
            created: "2026-01-01T00:00:00Z".to_string(),
            uid: format!("uid-{name}"),
            is_system: false,
        }
    }

    #[test]
    fn test_fuzzy_match_score_ranks_prefix_and_word_boundary_above_loose_subsequence() {
        assert_eq!(fuzzy_match_score("", "Deployments"), Some(0));
        assert!(
            fuzzy_match_score("pod", "Pods").unwrap()
                > fuzzy_match_score("pod", "Pod Security").unwrap()
        );
        assert!(fuzzy_match_score("dp", "Deployments").is_some());
        assert!(fuzzy_match_score("xyz", "Deployments").is_none());
    }

    #[test]
    fn test_command_palette_catalog_includes_views_clusters_namespaces_and_actions() {
        let clusters = vec![
            sample_cluster("prod-ctx", "Production"),
            sample_cluster("dev-ctx", "Development"),
        ];
        let ns_state = NamespaceFilterState {
            namespaces: vec![
                sample_namespace("default"),
                sample_namespace("kube-system"),
                sample_namespace("payments"),
            ],
            ..Default::default()
        };

        let catalog = build_command_palette_items(&clusters, Some("prod-ctx"), &ns_state);
        assert!(
            catalog
                .iter()
                .any(|i| i.action == CommandPaletteAction::NavigateView(WorkspaceView::Pods))
        );
        assert!(
            catalog
                .iter()
                .any(|i| i.action == CommandPaletteAction::SwitchCluster("prod-ctx".to_string())
                    && i.is_active)
        );
        assert!(
            catalog
                .iter()
                .any(|i| i.action == CommandPaletteAction::SwitchCluster("dev-ctx".to_string()))
        );
        assert!(
            catalog
                .iter()
                .any(|i| i.action == CommandPaletteAction::SelectAllNamespaces)
        );
        assert!(
            catalog
                .iter()
                .any(|i| i.action == CommandPaletteAction::ToggleNamespace("payments".to_string()))
        );
        assert!(
            catalog
                .iter()
                .any(|i| i.action == CommandPaletteAction::RefreshWorkspace)
        );
        assert!(
            catalog
                .iter()
                .any(|i| i.action == CommandPaletteAction::ToggleSidebar)
        );
        assert!(
            catalog
                .iter()
                .any(|i| i.action == CommandPaletteAction::OpenSettings)
        );
    }

    #[test]
    fn test_command_palette_keyboard_navigation_and_filtering_transitions() {
        let catalog = build_command_palette_items(&[], None, &NamespaceFilterState::default());
        let mut state = CommandPaletteState::default();
        state.open();
        assert!(state.is_open);

        for ch in "dep".chars() {
            let s = ch.to_string();
            let filtered_len = state.filter_items(&catalog).len();
            assert_eq!(
                state.handle_key_input(&s, Some(&s), filtered_len),
                CommandPaletteKeyResult::Handled
            );
        }
        let filtered = state.filter_items(&catalog);
        assert!(!filtered.is_empty());
        assert_eq!(
            filtered[0].action,
            CommandPaletteAction::NavigateView(WorkspaceView::Deployments)
        );
        assert_eq!(state.selected_index, 0);

        let all_len = 3;
        assert_eq!(
            state.handle_key_input("down", None, all_len),
            CommandPaletteKeyResult::Handled
        );
        assert_eq!(state.selected_index, 1);
        assert_eq!(
            state.handle_key_input("up", None, all_len),
            CommandPaletteKeyResult::Handled
        );
        assert_eq!(state.selected_index, 0);
        assert_eq!(
            state.handle_key_input("up", None, all_len),
            CommandPaletteKeyResult::Handled
        );
        assert_eq!(state.selected_index, 2);

        assert_eq!(
            state.handle_key_input("enter", None, all_len),
            CommandPaletteKeyResult::Execute(2)
        );

        assert_eq!(
            state.handle_key_input("escape", None, all_len),
            CommandPaletteKeyResult::Close
        );
        assert!(!state.is_open);
        assert!(state.query.is_empty());
    }

    #[test]
    fn test_picker_and_command_palette_exclude_standalone_logs() {
        use crate::navigation::{LogsTabTarget, WorkspaceTab};
        use crate::state::WorkspaceTabsState;

        let mut tabs = WorkspaceTabsState::new();
        tabs.activate_cluster("prod-ctx");
        tabs.open_picker();
        tabs.picker_query = "logs".to_string();
        assert!(
            tabs.filtered_picker_views().is_empty(),
            "Tab picker must not return standalone Logs view entries"
        );

        let catalog = build_command_palette_items(&[], None, &NamespaceFilterState::default());
        let mut palette = CommandPaletteState::default();
        palette.open();
        palette.query = "logs".to_string();
        let filtered_views: Vec<_> = palette
            .filter_items(&catalog)
            .into_iter()
            .filter(|item| item.category == CommandPaletteCategory::View)
            .collect();
        assert!(
            filtered_views.is_empty(),
            "Command palette must not return standalone Logs view entries"
        );

        let target = LogsTabTarget {
            kind: "Pod".to_string(),
            namespace: "default".to_string(),
            name: "api-0".to_string(),
        };
        tabs.open_logs_tab(target.clone());
        let count_after_first = tabs.open_tabs().len();
        tabs.open_logs_tab(target.clone());
        assert_eq!(tabs.open_tabs().len(), count_after_first);
        assert_eq!(tabs.active_tab(), Some(WorkspaceTab::Logs(target)));
    }
}
