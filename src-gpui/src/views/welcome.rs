use std::rc::Rc;

use gpui::{
    AnyElement, App, FontWeight, InteractiveElement, MouseButton, SharedString,
    StatefulInteractiveElement, Window, div, prelude::*, px,
};
use lucide_icons::Icon;
use orbit_core::kubernetes::models::ClusterInfo;

use crate::icons::render_icon;
use crate::navigation::WorkspaceView;
use crate::theme::{OrbitTheme, apply_text_step};

#[derive(Debug, Clone)]
pub struct WelcomeWalkthroughStep {
    pub icon: Icon,
    pub title: &'static str,
    pub description: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WelcomeClusterPromptState {
    pub is_empty: bool,
    pub heading: String,
    pub subtitle: String,
    pub cluster_count: usize,
}

pub fn welcome_walkthrough_steps() -> [WelcomeWalkthroughStep; 4] {
    [
        WelcomeWalkthroughStep {
            icon: Icon::Server,
            title: "Connect to a Cluster",
            description: "Pick any kubeconfig context from the header switcher or below.",
        },
        WelcomeWalkthroughStep {
            icon: Icon::LayoutDashboard,
            title: "Inspect Cluster Health",
            description: "Monitor nodes, pods, deployments, and services at a glance.",
        },
        WelcomeWalkthroughStep {
            icon: Icon::Boxes,
            title: "Explore Workloads & Network",
            description: "Browse workloads, services, ingresses, and config resources.",
        },
        WelcomeWalkthroughStep {
            icon: Icon::FileText,
            title: "Stream Logs & Events",
            description: "Tail container logs, inspect events, and forward ports.",
        },
    ]
}

pub fn welcome_cluster_prompt_state(cluster_count: usize) -> WelcomeClusterPromptState {
    if cluster_count == 0 {
        WelcomeClusterPromptState {
            is_empty: true,
            heading: "No Kubernetes Contexts Found".to_string(),
            subtitle: "Add a kubeconfig file to ~/.kube/config or refresh contexts to get started."
                .to_string(),
            cluster_count: 0,
        }
    } else {
        let suffix = if cluster_count == 1 { "" } else { "s" };
        WelcomeClusterPromptState {
            is_empty: false,
            heading: format!("{cluster_count} Available Cluster Context{suffix}"),
            subtitle: "Select a cluster context below or use the header switcher to connect."
                .to_string(),
            cluster_count,
        }
    }
}

pub type SelectClusterCallback = Rc<dyn Fn(String, &mut Window, &mut App)>;

pub fn render_welcome_view(
    clusters: &[ClusterInfo],
    kubeconfig_error: Option<&str>,
    theme: &OrbitTheme,
    on_select_cluster: SelectClusterCallback,
    on_refresh_contexts: impl Fn(&mut Window, &mut App) + 'static,
) -> AnyElement {
    div()
        .flex()
        .flex_col()
        .gap_6()
        .child(render_welcome_header(theme))
        .child(render_cluster_section(
            clusters,
            kubeconfig_error,
            theme,
            on_select_cluster,
            on_refresh_contexts,
        ))
        .child(render_walkthrough_grid(theme))
        .into_any_element()
}

fn render_welcome_header(theme: &OrbitTheme) -> AnyElement {
    theme
        .style_card(div())
        .p_6()
        .bg(theme.surface.bg_panel)
        .flex()
        .flex_col()
        .gap_2()
        .child(
            div()
                .flex()
                .items_center()
                .gap_2p5()
                .child(render_icon(Icon::Orbit, px(22.0), theme.accent.accent))
                .child(
                    apply_text_step(div(), theme.typography.size_xl)
                        .font_weight(FontWeight::BOLD)
                        .text_color(theme.text.primary)
                        .child("Welcome to Orbit"),
                ),
        )
        .child(
            apply_text_step(div(), theme.typography.size_sm)
                .text_color(theme.text.secondary)
                .child("Select a Kubernetes cluster context to inspect workloads, nodes, pods, and live cluster telemetry."),
        )
        .into_any_element()
}

fn render_cluster_section(
    clusters: &[ClusterInfo],
    kubeconfig_error: Option<&str>,
    theme: &OrbitTheme,
    on_select_cluster: SelectClusterCallback,
    on_refresh_contexts: impl Fn(&mut Window, &mut App) + 'static,
) -> AnyElement {
    let prompt = welcome_cluster_prompt_state(clusters.len());
    let control_hover = theme.surface.control_hover;

    let refresh_btn = theme
        .style_control(div().id("welcome-refresh-contexts-btn"))
        .px_3()
        .py_1p5()
        .text_color(theme.text.primary)
        .font_weight(FontWeight::MEDIUM)
        .cursor_pointer()
        .hover(move |s| s.bg(control_hover))
        .flex()
        .items_center()
        .gap_1p5()
        .child(render_icon(Icon::RefreshCw, px(12.0), theme.text.secondary))
        .child(apply_text_step(div(), theme.typography.size_xs).child("Refresh Contexts"))
        .on_click(move |_, window, cx| {
            on_refresh_contexts(window, cx);
        });

    let mut section = theme
        .style_card(div())
        .p_5()
        .flex()
        .flex_col()
        .gap_3()
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
                            apply_text_step(div(), theme.typography.size_sm)
                                .font_weight(FontWeight::SEMIBOLD)
                                .text_color(theme.text.primary)
                                .child(prompt.heading),
                        )
                        .child(
                            apply_text_step(div(), theme.typography.size_xs)
                                .text_color(theme.text.muted)
                                .child(prompt.subtitle),
                        ),
                )
                .child(refresh_btn),
        );

    if let Some(err) = kubeconfig_error {
        section = section.child(
            apply_text_step(div(), theme.typography.size_xs)
                .px_3()
                .py_2()
                .rounded(theme.style.radius_control)
                .bg(theme.semantic.danger.soft)
                .border_1()
                .border_color(theme.semantic.danger.solid)
                .text_color(theme.semantic.danger.solid)
                .child(err.to_string()),
        );
    }

    if !prompt.is_empty {
        section = section.child(render_cluster_list(clusters, theme, on_select_cluster));
    }

    section.into_any_element()
}

fn render_cluster_list(
    clusters: &[ClusterInfo],
    theme: &OrbitTheme,
    on_select_cluster: SelectClusterCallback,
) -> AnyElement {
    let hover_bg = theme.surface.bg_hover;
    let mut list = div().flex().flex_col().gap_2();

    for cluster in clusters {
        let cid = cluster.id.clone();
        let display_name = cluster
            .alias
            .as_deref()
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .unwrap_or(&cluster.name)
            .to_string();
        let server_label = cluster
            .server_url
            .as_deref()
            .unwrap_or("No server URL")
            .to_string();
        let select_cb = Rc::clone(&on_select_cluster);

        list = list.child(
            theme
                .style_control(div().id(SharedString::from(format!("welcome-cluster-{cid}"))))
                .px_3()
                .py_2p5()
                .bg(theme.surface.bg_panel)
                .hover(move |s| s.bg(hover_bg))
                .cursor_pointer()
                .flex()
                .items_center()
                .justify_between()
                .gap_3()
                .on_click(move |_, window, cx| {
                    select_cb(cid.clone(), window, cx);
                })
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap_2p5()
                        .min_w_0()
                        .child(render_icon(Icon::Server, px(14.0), theme.accent.accent))
                        .child(
                            div()
                                .flex()
                                .flex_col()
                                .min_w_0()
                                .child(
                                    apply_text_step(div(), theme.typography.size_sm)
                                        .font_weight(FontWeight::MEDIUM)
                                        .text_color(theme.text.primary)
                                        .truncate()
                                        .child(display_name),
                                )
                                .child(
                                    apply_text_step(div(), theme.typography.size_xs)
                                        .text_color(theme.text.muted)
                                        .truncate()
                                        .child(server_label),
                                ),
                        ),
                )
                .child(render_icon(Icon::ArrowRight, px(14.0), theme.text.muted)),
        );
    }

    list.into_any_element()
}

fn render_walkthrough_grid(theme: &OrbitTheme) -> AnyElement {
    let mut grid = div().flex().flex_wrap().gap_3();
    for step in welcome_walkthrough_steps() {
        grid = grid.child(
            theme
                .style_card(div())
                .flex_1()
                .min_w(px(220.0))
                .p_4()
                .flex()
                .flex_col()
                .gap_1p5()
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap_2()
                        .child(render_icon(step.icon, px(15.0), theme.accent.accent))
                        .child(
                            apply_text_step(div(), theme.typography.size_sm)
                                .font_weight(FontWeight::SEMIBOLD)
                                .text_color(theme.text.primary)
                                .child(step.title),
                        ),
                )
                .child(
                    apply_text_step(div(), theme.typography.size_xs)
                        .text_color(theme.text.secondary)
                        .child(step.description),
                ),
        );
    }
    grid.into_any_element()
}

#[derive(Debug, Clone, Copy)]
pub struct QuickStartTabItem {
    pub view: WorkspaceView,
    pub title: &'static str,
    pub description: &'static str,
}

pub fn empty_tabs_quick_start_items() -> [QuickStartTabItem; 6] {
    [
        QuickStartTabItem {
            view: WorkspaceView::Overview,
            title: "Cluster Overview",
            description: "Inspect cluster summary, node readiness, and resource health.",
        },
        QuickStartTabItem {
            view: WorkspaceView::Pods,
            title: "Pods",
            description: "Browse running pods, restarts, and container status.",
        },
        QuickStartTabItem {
            view: WorkspaceView::Deployments,
            title: "Deployments",
            description: "Monitor rollout replicas, images, and availability.",
        },
        QuickStartTabItem {
            view: WorkspaceView::Nodes,
            title: "Nodes",
            description: "Check cluster node capacity, conditions, and kubelet info.",
        },
        QuickStartTabItem {
            view: WorkspaceView::Services,
            title: "Services",
            description: "Inspect ClusterIP, NodePort, and LoadBalancer endpoints.",
        },
        QuickStartTabItem {
            view: WorkspaceView::Events,
            title: "Events",
            description: "Track recent cluster warnings and scheduling events.",
        },
    ]
}

pub fn render_empty_tabs_workspace(
    cluster_name: &str,
    theme: &OrbitTheme,
    on_open_view: super::ValueCallback<WorkspaceView>,
    on_open_picker: impl Fn(&mut Window, &mut App) + 'static,
) -> AnyElement {
    let mut grid = div().flex().flex_wrap().gap_3();
    for item in empty_tabs_quick_start_items() {
        let on_pick = on_open_view.clone();
        let view = item.view;
        let icon_color = view
            .resource_kind()
            .map(|k| theme.k8s_color(k))
            .unwrap_or(theme.accent.accent);

        grid = grid.child(
            theme
                .style_card(div())
                .id(SharedString::from(format!("empty-tab-card-{}", view.id())))
                .flex_1()
                .min_w(px(220.0))
                .p_4()
                .flex()
                .flex_col()
                .gap_1p5()
                .cursor_pointer()
                .hover(|s| s.bg(theme.surface.bg_hover))
                .on_mouse_down(MouseButton::Left, move |_event, window, cx| {
                    on_pick(view, window, cx);
                })
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap_2()
                        .child(render_icon(view.icon(), px(15.0), icon_color))
                        .child(
                            apply_text_step(div(), theme.typography.size_sm)
                                .font_weight(FontWeight::SEMIBOLD)
                                .text_color(theme.text.primary)
                                .child(item.title),
                        ),
                )
                .child(
                    apply_text_step(div(), theme.typography.size_xs)
                        .text_color(theme.text.secondary)
                        .child(item.description),
                ),
        );
    }

    div()
        .flex()
        .flex_col()
        .gap_6()
        .child(
            theme
                .style_card(div())
                .p_6()
                .bg(theme.surface.bg_panel)
                .flex()
                .items_center()
                .justify_between()
                .gap_4()
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .gap_1p5()
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .gap_2()
                                .child(render_icon(Icon::Layers, px(18.0), theme.accent.accent))
                                .child(
                                    apply_text_step(div(), theme.typography.size_lg)
                                        .font_weight(FontWeight::BOLD)
                                        .text_color(theme.text.primary)
                                        .child("No Open Tabs"),
                                ),
                        )
                        .child(
                            apply_text_step(div(), theme.typography.size_xs)
                                .text_color(theme.text.secondary)
                                .child(format!(
                                    "Connected to {cluster_name}. Open a resource view below or use the sidebar to start."
                                )),
                        ),
                )
                .child(
                    div()
                        .id("empty-tabs-open-new-btn")
                        .px_3()
                        .py_1p5()
                        .rounded(theme.style.radius_control)
                        .bg(theme.accent.accent)
                        .flex()
                        .items_center()
                        .gap_1p5()
                        .cursor_pointer()
                        .hover(|s| s.bg(theme.accent.hover))
                        .on_mouse_down(MouseButton::Left, move |_event, window, cx| {
                            window.prevent_default();
                            cx.stop_propagation();
                            on_open_picker(window, cx);
                        })
                        .child(render_icon(Icon::Plus, px(14.0), theme.surface.bg_app))
                        .child(
                            apply_text_step(div(), theme.typography.size_xs)
                                .font_weight(FontWeight::SEMIBOLD)
                                .text_color(theme.surface.bg_app)
                                .child("Open New Tab"),
                        ),
                ),
        )
        .child(grid)
        .into_any_element()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::icons::icon_char;

    #[test]
    fn test_welcome_walkthrough_and_empty_vs_populated_state_copy() {
        let steps = welcome_walkthrough_steps();
        assert_eq!(steps.len(), 4);
        for step in &steps {
            assert!(!step.title.trim().is_empty());
            assert!(!step.description.trim().is_empty());
            assert_ne!(icon_char(step.icon), '\0');
        }

        let empty_prompt = welcome_cluster_prompt_state(0);
        assert!(empty_prompt.is_empty);
        assert_eq!(empty_prompt.cluster_count, 0);
        assert!(
            empty_prompt.heading.to_lowercase().contains("no ")
                || empty_prompt.subtitle.to_lowercase().contains("kubeconfig")
        );

        let populated_prompt = welcome_cluster_prompt_state(2);
        assert!(!populated_prompt.is_empty);
        assert_eq!(populated_prompt.cluster_count, 2);
        assert!(populated_prompt.heading.contains('2'));
    }

    #[test]
    fn test_empty_tabs_quick_start_items_cover_core_workflows() {
        let items = empty_tabs_quick_start_items();
        let views: Vec<_> = items.iter().map(|item| item.view).collect();
        assert!(views.contains(&crate::navigation::WorkspaceView::Overview));
        assert!(views.contains(&crate::navigation::WorkspaceView::Pods));
        assert!(views.contains(&crate::navigation::WorkspaceView::Deployments));
        assert!(views.contains(&crate::navigation::WorkspaceView::Nodes));
        assert!(views.contains(&crate::navigation::WorkspaceView::Services));
        assert!(views.contains(&crate::navigation::WorkspaceView::Events));
        for item in items {
            assert!(!item.title.trim().is_empty());
            assert!(!item.description.trim().is_empty());
        }
    }
}
