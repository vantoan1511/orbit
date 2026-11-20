use std::rc::Rc;

use gpui::{AnyElement, App, FontWeight, Rgba, SharedString, Window, div, prelude::*, px};
use lucide_icons::Icon;
use orbit_core::kubernetes::models::{ClusterInfo, UserProfileInfo};
use orbit_core::kubernetes::overview::{ClusterOverallHealth, ClusterOverviewSummary};

use crate::components::{SemanticStatus, StatusBadgeVariant, render_status_badge};
use crate::navigation::WorkspaceView;
use crate::state::{ClusterOverviewState, OverviewLoadStatus};
use crate::theme::{K8sResourceKind, OrbitTheme, apply_text_step};
use crate::views::{render_action_button, render_detail_row};

#[derive(Debug, Clone)]
pub struct OverviewMetricCard {
    pub title: &'static str,
    pub target_view: WorkspaceView,
    pub kind: K8sResourceKind,
    pub color: Rgba,
    pub total: usize,
    pub primary_value: String,
    pub subtitle: String,
    pub error: Option<String>,
}

#[derive(Debug, Clone)]
pub struct HealthBreakdownSegment {
    pub label: &'static str,
    pub count: usize,
    pub color: Rgba,
    pub include_in_bar: bool,
}

#[derive(Debug, Clone)]
pub struct HealthBreakdownSection {
    pub title: &'static str,
    pub total: usize,
    pub segments: Vec<HealthBreakdownSegment>,
    pub error: Option<String>,
}

pub type NavigateCallback = Rc<dyn Fn(WorkspaceView, &mut Window, &mut App)>;

pub fn health_percentage(healthy: usize, total: usize) -> u8 {
    (healthy.min(total) * 100).checked_div(total).unwrap_or(0) as u8
}

pub fn cluster_health_presentation(
    health: ClusterOverallHealth,
    theme: &OrbitTheme,
) -> (&'static str, SemanticStatus, Rgba) {
    match health {
        ClusterOverallHealth::Healthy => {
            ("Healthy", SemanticStatus::Success, theme.semantic.success.solid)
        }
        ClusterOverallHealth::Degraded => {
            ("Degraded", SemanticStatus::Warning, theme.semantic.warning.solid)
        }
        ClusterOverallHealth::Unknown => {
            ("Unknown", SemanticStatus::Neutral, theme.text.muted)
        }
    }
}

pub fn overview_metric_cards(
    summary: &ClusterOverviewSummary,
    theme: &OrbitTheme,
) -> [OverviewMetricCard; 4] {
    [
        OverviewMetricCard {
            title: "Nodes",
            target_view: WorkspaceView::Nodes,
            kind: K8sResourceKind::Node,
            color: theme.k8s_color(K8sResourceKind::Node),
            total: summary.nodes.total,
            primary_value: summary.nodes.total.to_string(),
            subtitle: format!(
                "{} Ready • {}% healthy",
                summary.nodes.ready,
                health_percentage(summary.nodes.ready, summary.nodes.total)
            ),
            error: summary.nodes.error.clone(),
        },
        OverviewMetricCard {
            title: "Pods",
            target_view: WorkspaceView::Pods,
            kind: K8sResourceKind::Pod,
            color: theme.k8s_color(K8sResourceKind::Pod),
            total: summary.pods.total,
            primary_value: summary.pods.total.to_string(),
            subtitle: format!(
                "{} Running • {}% healthy",
                summary.pods.running,
                health_percentage(summary.pods.running, summary.pods.total)
            ),
            error: summary.pods.error.clone(),
        },
        OverviewMetricCard {
            title: "Deployments",
            target_view: WorkspaceView::Deployments,
            kind: K8sResourceKind::Deployment,
            color: theme.k8s_color(K8sResourceKind::Deployment),
            total: summary.deployments.total,
            primary_value: summary.deployments.total.to_string(),
            subtitle: format!(
                "{}/{} replicas available",
                summary.deployments.available_replicas, summary.deployments.desired_replicas
            ),
            error: summary.deployments.error.clone(),
        },
        OverviewMetricCard {
            title: "Services",
            target_view: WorkspaceView::Services,
            kind: K8sResourceKind::Service,
            color: theme.k8s_color(K8sResourceKind::Service),
            total: summary.services.total,
            primary_value: summary.services.total.to_string(),
            subtitle: format!(
                "{} ClusterIP • {} LB",
                summary.services.cluster_ip, summary.services.load_balancer
            ),
            error: summary.services.error.clone(),
        },
    ]
}

pub fn health_breakdown_sections(
    summary: &ClusterOverviewSummary,
    theme: &OrbitTheme,
) -> [HealthBreakdownSection; 2] {
    [
        HealthBreakdownSection {
            title: "Node Health",
            total: summary.nodes.total,
            segments: vec![
                HealthBreakdownSegment {
                    label: "Ready",
                    count: summary.nodes.ready,
                    color: theme.semantic.success.solid,
                    include_in_bar: true,
                },
                HealthBreakdownSegment {
                    label: "NotReady",
                    count: summary.nodes.not_ready,
                    color: theme.semantic.danger.solid,
                    include_in_bar: true,
                },
                HealthBreakdownSegment {
                    label: "Cordoned",
                    count: summary.nodes.cordoned,
                    color: theme.semantic.warning.solid,
                    include_in_bar: false,
                },
                HealthBreakdownSegment {
                    label: "Unknown",
                    count: summary.nodes.unknown,
                    color: theme.text.muted,
                    include_in_bar: true,
                },
            ],
            error: summary.nodes.error.clone(),
        },
        HealthBreakdownSection {
            title: "Pod Health",
            total: summary.pods.total,
            segments: vec![
                HealthBreakdownSegment {
                    label: "Running",
                    count: summary.pods.running,
                    color: theme.semantic.success.solid,
                    include_in_bar: true,
                },
                HealthBreakdownSegment {
                    label: "Pending",
                    count: summary.pods.pending,
                    color: theme.semantic.warning.solid,
                    include_in_bar: true,
                },
                HealthBreakdownSegment {
                    label: "Failed",
                    count: summary.pods.failed,
                    color: theme.semantic.danger.solid,
                    include_in_bar: true,
                },
                HealthBreakdownSegment {
                    label: "CrashLoop",
                    count: summary.pods.crash_loop,
                    color: theme.semantic.danger.solid,
                    include_in_bar: false,
                },
                HealthBreakdownSegment {
                    label: "Succeeded",
                    count: summary.pods.succeeded,
                    color: theme.semantic.info.solid,
                    include_in_bar: true,
                },
            ],
            error: summary.pods.error.clone(),
        },
    ]
}

#[allow(clippy::too_many_arguments)]
pub fn render_overview_view(
    cluster_id: &str,
    display_name: &str,
    k8s_version: Option<&str>,
    profile: Option<&UserProfileInfo>,
    active_cluster: Option<&ClusterInfo>,
    overview: &ClusterOverviewState,
    theme: &OrbitTheme,
    on_navigate: NavigateCallback,
    on_refresh: impl Fn(&mut Window, &mut App) + 'static,
) -> AnyElement {
    let version_label = overview
        .summary
        .as_ref()
        .and_then(|s| s.k8s_version.as_deref())
        .or(k8s_version)
        .unwrap_or("Connected")
        .to_string();

    let server_url = profile
        .and_then(|p| p.server_url.clone())
        .or_else(|| active_cluster.and_then(|c| c.server_url.clone()))
        .unwrap_or_else(|| "Unknown".to_string());

    let mut root = div().flex().flex_col().gap_5().child(render_status_bar(
        cluster_id,
        &version_label,
        &server_url,
        overview,
        theme,
    ));

    if let Some(ref err) = overview.error_message {
        let is_loading = overview.status == OverviewLoadStatus::Loading;
        root = root.child(
            div()
                .p_3()
                .rounded(theme.style.radius_control)
                .bg(theme.semantic.danger.soft)
                .border_1()
                .border_color(theme.semantic.danger.solid)
                .flex()
                .items_center()
                .justify_between()
                .gap_3()
                .child(
                    apply_text_step(div(), theme.typography.size_xs)
                        .text_color(theme.semantic.danger.solid)
                        .child(err.clone()),
                )
                .child(render_action_button(
                    "overview-error-retry-btn",
                    Icon::RefreshCw,
                    "Retry",
                    is_loading,
                    theme,
                    Rc::new(on_refresh),
                )),
        );
    }

    if let Some(ref summary) = overview.summary {
        root = root
            .child(render_metric_cards(summary, theme, on_navigate))
            .child(render_health_breakdowns(summary, theme));
    } else if overview.status == OverviewLoadStatus::Loading {
        root = root.child(
            theme.style_card(div()).p_5().child(
                apply_text_step(div(), theme.typography.size_sm)
                    .text_color(theme.text.muted)
                    .child("Loading cluster overview metrics..."),
            ),
        );
    }

    root.child(render_cluster_profile(
        cluster_id,
        display_name,
        &version_label,
        &server_url,
        profile,
        active_cluster,
        theme,
    ))
    .into_any_element()
}

fn render_status_item(
    label: &str,
    value: AnyElement,
    theme: &OrbitTheme,
) -> AnyElement {
    div()
        .flex()
        .flex_col()
        .gap_0p5()
        .child(
            apply_text_step(div(), theme.typography.size_xs)
                .text_color(theme.text.muted)
                .child(label.to_string()),
        )
        .child(value)
        .into_any_element()
}

fn render_status_bar(
    cluster_id: &str,
    version_label: &str,
    server_url: &str,
    overview: &ClusterOverviewState,
    theme: &OrbitTheme,
) -> AnyElement {
    let mono_font = theme.typography.mono_font(FontWeight::NORMAL);
    let (health_label, health_status, _) = match overview.summary.as_ref() {
        Some(summary) => cluster_health_presentation(summary.overall_health, theme),
        None => ("Loading", SemanticStatus::Info, theme.semantic.info.solid),
    };

    theme
        .style_card(div())
        .p_4()
        .flex()
        .items_center()
        .justify_between()
        .gap_4()
        .child(
            div()
                .flex()
                .items_center()
                .gap_4()
                .flex_wrap()
                .child(render_status_item(
                    "Cluster Health",
                    render_status_badge(
                        health_label,
                        health_status,
                        StatusBadgeVariant::Pill,
                        theme,
                    ),
                    theme,
                ))
                .child(render_status_item(
                    "Kubernetes Version",
                    apply_text_step(div(), theme.typography.size_sm)
                        .font(mono_font.clone())
                        .font_weight(FontWeight::SEMIBOLD)
                        .text_color(theme.text.primary)
                        .child(version_label.to_string())
                        .into_any_element(),
                    theme,
                ))
                .child(render_status_item(
                    "Active Context",
                    apply_text_step(div(), theme.typography.size_sm)
                        .font(mono_font.clone())
                        .font_weight(FontWeight::MEDIUM)
                        .text_color(theme.text.primary)
                        .child(cluster_id.to_string())
                        .into_any_element(),
                    theme,
                ))
                .child(render_status_item(
                    "API Server",
                    apply_text_step(div(), theme.typography.size_xs)
                        .font(mono_font)
                        .text_color(theme.text.secondary)
                        .child(server_url.to_string())
                        .into_any_element(),
                    theme,
                )),
        )
        .into_any_element()
}

fn render_metric_cards(
    summary: &ClusterOverviewSummary,
    theme: &OrbitTheme,
    on_navigate: NavigateCallback,
) -> AnyElement {
    let hover_bg = theme.surface.bg_hover;
    let mut row = div().flex().flex_wrap().gap_3();

    for card in overview_metric_cards(summary, theme) {
        let nav_cb = Rc::clone(&on_navigate);
        let target = card.target_view;
        let mut el = theme
            .style_card(div().id(SharedString::from(format!(
                "overview-card-{}",
                card.title
            ))))
            .flex_1()
            .min_w(px(200.0))
            .p_4()
            .hover(move |s| s.bg(hover_bg))
            .cursor_pointer()
            .flex()
            .flex_col()
            .gap_2()
            .on_click(move |_, window, cx| {
                nav_cb(target, window, cx);
            })
            .child(
                div()
                    .flex()
                    .items_center()
                    .justify_between()
                    .child(
                        apply_text_step(div(), theme.typography.size_xs)
                            .font_weight(FontWeight::SEMIBOLD)
                            .text_color(theme.text.secondary)
                            .child(card.title),
                    )
                    .child(div().w(px(8.0)).h(px(8.0)).rounded_full().bg(card.color)),
            )
            .child(
                apply_text_step(div(), theme.typography.size_xl)
                    .font_weight(FontWeight::BOLD)
                    .text_color(theme.text.primary)
                    .child(card.primary_value),
            )
            .child(
                apply_text_step(div(), theme.typography.size_xs)
                    .text_color(theme.text.muted)
                    .child(card.subtitle),
            );

        if let Some(err) = card.error {
            el = el.child(
                apply_text_step(div(), theme.typography.size_xs)
                    .px_2()
                    .py_1()
                    .rounded(theme.style.radius_control)
                    .bg(theme.semantic.warning.soft)
                    .text_color(theme.semantic.warning.solid)
                    .truncate()
                    .child(err),
            );
        }

        row = row.child(el);
    }

    row.into_any_element()
}

fn render_health_breakdowns(
    summary: &ClusterOverviewSummary,
    theme: &OrbitTheme,
) -> AnyElement {
    let mut row = div().flex().flex_wrap().gap_3();

    for section in health_breakdown_sections(summary, theme) {
        let mut seg_bar = div()
            .w_full()
            .h(px(6.0))
            .rounded_full()
            .overflow_hidden()
            .bg(theme.surface.control_bg)
            .flex();

        if section.total > 0 {
            for seg in &section.segments {
                if seg.include_in_bar && seg.count > 0 {
                    let pct = (seg.count as f32 / section.total as f32).max(0.02);
                    seg_bar = seg_bar.child(
                        div()
                            .h_full()
                            .flex_basis(px(pct * 100.0))
                            .flex_grow()
                            .bg(seg.color),
                    );
                }
            }
        }

        let mut legend = div().flex().flex_wrap().gap_3();
        for seg in &section.segments {
            let pct = health_percentage(seg.count, section.total);
            legend = legend.child(
                div()
                    .flex()
                    .items_center()
                    .gap_1p5()
                    .child(div().w(px(6.0)).h(px(6.0)).rounded_full().bg(seg.color))
                    .child(
                        apply_text_step(div(), theme.typography.size_xs)
                            .text_color(theme.text.secondary)
                            .child(format!("{}: {} ({pct}%)", seg.label, seg.count)),
                    ),
            );
        }

        let mut section_el = theme
            .style_card(div())
            .flex_1()
            .min_w(px(280.0))
            .p_4()
            .flex()
            .flex_col()
            .gap_3()
            .child(
                div()
                    .flex()
                    .items_center()
                    .justify_between()
                    .child(
                        apply_text_step(div(), theme.typography.size_sm)
                            .font_weight(FontWeight::SEMIBOLD)
                            .text_color(theme.text.primary)
                            .child(section.title),
                    )
                    .child(
                        apply_text_step(div(), theme.typography.size_xs)
                            .text_color(theme.text.muted)
                            .child(format!("{} total", section.total)),
                    ),
            )
            .child(seg_bar)
            .child(legend);

        if let Some(err) = section.error {
            section_el = section_el.child(
                apply_text_step(div(), theme.typography.size_xs)
                    .text_color(theme.semantic.warning.solid)
                    .child(err),
            );
        }

        row = row.child(section_el);
    }

    row.into_any_element()
}

fn render_cluster_profile(
    cluster_id: &str,
    display_name: &str,
    version_label: &str,
    server_url: &str,
    profile: Option<&UserProfileInfo>,
    active_cluster: Option<&ClusterInfo>,
    theme: &OrbitTheme,
) -> AnyElement {
    let user_name = profile
        .and_then(|p| p.user_name.clone())
        .unwrap_or_else(|| "None".to_string());
    let auth_type = profile
        .map(|p| p.auth_type.clone())
        .unwrap_or_else(|| "None".to_string());
    let kubeconfigs = profile
        .map(|p| p.kubeconfig_paths.join(", "))
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "None".to_string());
    let description = active_cluster.and_then(|c| c.description.clone());
    let tags = active_cluster
        .map(|c| c.tags.join(", "))
        .filter(|s| !s.is_empty());

    let mut card = theme
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
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .gap_0p5()
                        .child(
                            apply_text_step(div(), theme.typography.size_base)
                                .font_weight(FontWeight::BOLD)
                                .text_color(theme.text.primary)
                                .child(display_name.to_string()),
                        )
                        .when_some(description, |el, desc| {
                            el.child(
                                apply_text_step(div(), theme.typography.size_xs)
                                    .text_color(theme.text.secondary)
                                    .child(desc),
                            )
                        }),
                )
                .child(render_status_badge(
                    format!("Connected • {version_label}"),
                    SemanticStatus::Success,
                    StatusBadgeVariant::Pill,
                    theme,
                )),
        )
        .child(render_detail_row(
            "Context",
            cluster_id.to_string(),
            true,
            theme,
        ))
        .child(render_detail_row(
            "Kubernetes Version",
            version_label.to_string(),
            true,
            theme,
        ))
        .child(render_detail_row(
            "API Server",
            server_url.to_string(),
            true,
            theme,
        ))
        .child(render_detail_row("User", user_name, false, theme))
        .child(render_detail_row("Auth Type", auth_type, false, theme))
        .child(render_detail_row(
            "Kubeconfig Paths",
            kubeconfigs,
            true,
            theme,
        ));

    if let Some(tag_str) = tags {
        card = card.child(render_detail_row("Tags", tag_str, false, theme));
    }

    card.into_any_element()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::theme::{AccentColor, SemanticColorsPreference, ThemeMode, UiStyle};
    use gpui::WindowAppearance;
    use orbit_core::kubernetes::overview::{
        DeploymentHealthSummary, NodeHealthSummary, PodHealthSummary, ServiceSummary,
    };

    #[test]
    fn test_overview_metric_cards_health_percentages_and_partial_errors() {
        assert_eq!(health_percentage(3, 4), 75);
        assert_eq!(health_percentage(0, 0), 0);

        let dark_theme = OrbitTheme::resolve(
            ThemeMode::Dark,
            UiStyle::Flat,
            AccentColor::Blue,
            SemanticColorsPreference::default(),
            WindowAppearance::Dark,
        );
        let light_theme = OrbitTheme::resolve(
            ThemeMode::Light,
            UiStyle::Flat,
            AccentColor::Blue,
            SemanticColorsPreference::default(),
            WindowAppearance::Light,
        );

        let summary = ClusterOverviewSummary {
            k8s_version: Some("v1.31.1".to_string()),
            overall_health: ClusterOverallHealth::Degraded,
            nodes: NodeHealthSummary {
                total: 4,
                ready: 3,
                not_ready: 1,
                cordoned: 1,
                unknown: 0,
                error: None,
            },
            pods: PodHealthSummary {
                total: 10,
                running: 7,
                pending: 1,
                failed: 1,
                succeeded: 1,
                crash_loop: 1,
                other: 0,
                error: None,
            },
            deployments: DeploymentHealthSummary {
                total: 3,
                available: 2,
                progressing: 1,
                failed: 0,
                desired_replicas: 6,
                available_replicas: 5,
                error: None,
            },
            services: ServiceSummary {
                total: 0,
                cluster_ip: 0,
                node_port: 0,
                load_balancer: 0,
                external_name: 0,
                error: Some("Forbidden".to_string()),
            },
        };

        for theme in [&dark_theme, &light_theme] {
            let cards = overview_metric_cards(&summary, theme);
            assert_eq!(cards.len(), 4);
            assert_eq!(cards[0].title, "Nodes");
            assert_eq!(cards[0].target_view, WorkspaceView::Nodes);
            assert_eq!(cards[0].color, theme.k8s_color(K8sResourceKind::Node));
            assert!(cards[0].error.is_none());

            assert_eq!(cards[1].title, "Pods");
            assert_eq!(cards[1].target_view, WorkspaceView::Pods);
            assert_eq!(cards[1].color, theme.k8s_color(K8sResourceKind::Pod));
            assert!(cards[1].error.is_none());

            assert_eq!(cards[2].title, "Deployments");
            assert_eq!(cards[2].target_view, WorkspaceView::Deployments);
            assert_eq!(cards[2].color, theme.k8s_color(K8sResourceKind::Deployment));
            assert!(cards[2].error.is_none());

            assert_eq!(cards[3].title, "Services");
            assert_eq!(cards[3].target_view, WorkspaceView::Services);
            assert_eq!(cards[3].color, theme.k8s_color(K8sResourceKind::Service));
            assert_eq!(cards[3].error.as_deref(), Some("Forbidden"));

            let sections = health_breakdown_sections(&summary, theme);
            assert_eq!(sections.len(), 2);
            assert_eq!(sections[0].title, "Node Health");
            let node_labels: Vec<&str> = sections[0].segments.iter().map(|s| s.label).collect();
            assert_eq!(
                node_labels,
                vec!["Ready", "NotReady", "Cordoned", "Unknown"]
            );
            assert_eq!(sections[0].segments[0].color, theme.semantic.success.solid);
            assert_eq!(sections[0].segments[1].color, theme.semantic.danger.solid);
            assert_eq!(sections[0].segments[2].color, theme.semantic.warning.solid);

            assert_eq!(sections[1].title, "Pod Health");
            let pod_labels: Vec<&str> = sections[1].segments.iter().map(|s| s.label).collect();
            assert_eq!(
                pod_labels,
                vec!["Running", "Pending", "Failed", "CrashLoop", "Succeeded"]
            );
            assert_eq!(sections[1].segments[0].color, theme.semantic.success.solid);
            assert_eq!(sections[1].segments[1].color, theme.semantic.warning.solid);
            assert_eq!(sections[1].segments[2].color, theme.semantic.danger.solid);
        }
    }
}
