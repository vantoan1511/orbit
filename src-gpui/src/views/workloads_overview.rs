use std::rc::Rc;

use gpui::{AnyElement, App, FontWeight, Rgba, SharedString, Window, div, prelude::*, px};
use lucide_icons::Icon;
use orbit_core::kubernetes::workloads_overview::{
    WorkloadControllerKind, WorkloadHealthCounts, WorkloadsOverviewSnapshot,
};

use crate::components::{SemanticStatus, StatusBadgeVariant, render_status_badge};
use crate::icons::render_icon;
use crate::navigation::WorkspaceView;
use crate::state::{NamespaceFilterState, WorkloadsOverviewLoadStatus, WorkloadsOverviewState};
use crate::theme::{K8sResourceKind, OrbitTheme, apply_text_step};
use crate::views::overview::{NavigateCallback, health_percentage};
use crate::views::render_action_button;

type RefreshCallback = Rc<dyn Fn(&mut Window, &mut App)>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WorkloadsOverviewPresentationMode {
    Loading,
    Error,
    Empty,
    Summary,
}

#[derive(Debug, Clone)]
pub struct WorkloadSummaryCard {
    pub kind: WorkloadControllerKind,
    pub title: &'static str,
    pub target_view: WorkspaceView,
    pub icon: Icon,
    pub kind_color: Rgba,
    pub counts: WorkloadHealthCounts,
    pub health_pct: u8,
    pub status: SemanticStatus,
    pub status_label: &'static str,
    pub error: Option<String>,
}

pub fn resolve_workloads_overview_mode(
    state: &WorkloadsOverviewState,
    namespaces: &NamespaceFilterState,
) -> WorkloadsOverviewPresentationMode {
    match state.snapshot.as_ref() {
        Some(snapshot) => {
            let aggregate = snapshot.aggregate_counts(namespaces.active_selection());
            if aggregate.total == 0 && !snapshot.has_any_error() {
                WorkloadsOverviewPresentationMode::Empty
            } else {
                WorkloadsOverviewPresentationMode::Summary
            }
        }
        None => match state.status {
            WorkloadsOverviewLoadStatus::Error => WorkloadsOverviewPresentationMode::Error,
            WorkloadsOverviewLoadStatus::Idle
            | WorkloadsOverviewLoadStatus::Loading
            | WorkloadsOverviewLoadStatus::Loaded => WorkloadsOverviewPresentationMode::Loading,
        },
    }
}

pub fn aggregate_workload_status(
    counts: WorkloadHealthCounts,
    has_errors: bool,
) -> (&'static str, SemanticStatus) {
    if counts.failed > 0 {
        ("Failed", SemanticStatus::Danger)
    } else if counts.degraded > 0 || has_errors {
        ("Degraded", SemanticStatus::Warning)
    } else if counts.total > 0 {
        ("Healthy", SemanticStatus::Success)
    } else {
        ("No Workloads", SemanticStatus::Neutral)
    }
}

const fn target_view_for_kind(kind: WorkloadControllerKind) -> WorkspaceView {
    match kind {
        WorkloadControllerKind::Pods => WorkspaceView::Pods,
        WorkloadControllerKind::Deployments => WorkspaceView::Deployments,
        WorkloadControllerKind::StatefulSets => WorkspaceView::StatefulSets,
        WorkloadControllerKind::DaemonSets => WorkspaceView::DaemonSets,
        WorkloadControllerKind::ReplicaSets => WorkspaceView::ReplicaSets,
        WorkloadControllerKind::Jobs => WorkspaceView::Jobs,
        WorkloadControllerKind::CronJobs => WorkspaceView::CronJobs,
        WorkloadControllerKind::Hpas => WorkspaceView::Hpas,
    }
}

const fn k8s_kind_for_workload(kind: WorkloadControllerKind) -> K8sResourceKind {
    match kind {
        WorkloadControllerKind::Pods => K8sResourceKind::Pod,
        WorkloadControllerKind::Deployments => K8sResourceKind::Deployment,
        WorkloadControllerKind::StatefulSets => K8sResourceKind::StatefulSet,
        WorkloadControllerKind::DaemonSets => K8sResourceKind::DaemonSet,
        WorkloadControllerKind::ReplicaSets => K8sResourceKind::ReplicaSet,
        WorkloadControllerKind::Jobs | WorkloadControllerKind::CronJobs => K8sResourceKind::Job,
        WorkloadControllerKind::Hpas => K8sResourceKind::Hpa,
    }
}

fn card_status(counts: WorkloadHealthCounts, has_error: bool) -> (&'static str, SemanticStatus) {
    if counts.failed > 0 {
        ("Failed", SemanticStatus::Danger)
    } else if counts.degraded > 0 {
        ("Degraded", SemanticStatus::Warning)
    } else if has_error {
        ("Restricted", SemanticStatus::Warning)
    } else if counts.total > 0 {
        ("Healthy", SemanticStatus::Success)
    } else {
        ("Empty", SemanticStatus::Neutral)
    }
}

fn build_card(
    kind: WorkloadControllerKind,
    snapshot: &WorkloadsOverviewSnapshot,
    selected_namespaces: &[String],
    theme: &OrbitTheme,
) -> WorkloadSummaryCard {
    let target_view = target_view_for_kind(kind);
    let kind_snapshot = snapshot.kind_snapshot(kind);
    let counts = kind_snapshot.counts_for_namespaces(selected_namespaces);
    let health_pct = health_percentage(counts.healthy, counts.total);
    let (status_label, status) = card_status(counts, kind_snapshot.error.is_some());

    WorkloadSummaryCard {
        kind,
        title: target_view.label(),
        target_view,
        icon: target_view.icon(),
        kind_color: theme.k8s_color(k8s_kind_for_workload(kind)),
        counts,
        health_pct,
        status,
        status_label,
        error: kind_snapshot.error.clone(),
    }
}

pub fn workload_summary_cards(
    snapshot: &WorkloadsOverviewSnapshot,
    namespaces: &NamespaceFilterState,
    theme: &OrbitTheme,
) -> [WorkloadSummaryCard; 8] {
    let selected = namespaces.active_selection();
    WorkloadControllerKind::ALL.map(|kind| build_card(kind, snapshot, selected, theme))
}

pub fn render_workloads_overview_view(
    state: &WorkloadsOverviewState,
    namespaces: &NamespaceFilterState,
    theme: &OrbitTheme,
    on_navigate: NavigateCallback,
    on_refresh: impl Fn(&mut Window, &mut App) + 'static,
) -> AnyElement {
    let refresh_cb: RefreshCallback = Rc::new(on_refresh);
    let mode = resolve_workloads_overview_mode(state, namespaces);
    let is_loading = state.status == WorkloadsOverviewLoadStatus::Loading;

    let mut root = div().flex().flex_col().gap_5().child(render_summary_toolbar(
        state,
        namespaces,
        is_loading,
        theme,
    ));

    if let Some(ref err) = state.error_message
        && state.snapshot.is_some()
    {
        root = root.child(
            apply_text_step(div(), theme.typography.size_xs)
                .p_3()
                .rounded(theme.style.radius_control)
                .bg(theme.semantic.danger.soft)
                .border_1()
                .border_color(theme.semantic.danger.solid)
                .text_color(theme.semantic.danger.solid)
                .child(err.clone()),
        );
    }

    let body = match mode {
        WorkloadsOverviewPresentationMode::Loading => render_loading_state(theme),
        WorkloadsOverviewPresentationMode::Error => {
            render_error_state(state.error_message.as_deref(), theme, refresh_cb)
        }
        WorkloadsOverviewPresentationMode::Empty => render_empty_state(namespaces, theme),
        WorkloadsOverviewPresentationMode::Summary => match state.snapshot.as_ref() {
            Some(snapshot) => render_summary_grid(snapshot, namespaces, theme, on_navigate),
            None => render_loading_state(theme),
        },
    };

    root.child(body).into_any_element()
}

fn render_summary_metric(
    label: &'static str,
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
                .child(label),
        )
        .child(value)
        .into_any_element()
}

fn render_semantic_counter(
    label: &'static str,
    count: usize,
    dot_color: Rgba,
    theme: &OrbitTheme,
) -> AnyElement {
    div()
        .flex()
        .items_center()
        .gap_1p5()
        .child(div().w(px(6.0)).h(px(6.0)).rounded_full().bg(dot_color))
        .child(
            apply_text_step(div(), theme.typography.size_xs)
                .font(theme.typography.tabular_ui_font(FontWeight::NORMAL))
                .text_color(theme.text.secondary)
                .child(format!("{label}: {count}")),
        )
        .into_any_element()
}

fn render_health_counters(counts: WorkloadHealthCounts, theme: &OrbitTheme) -> AnyElement {
    div()
        .flex()
        .flex_wrap()
        .items_center()
        .gap_3()
        .child(render_semantic_counter(
            "Healthy",
            counts.healthy,
            theme.semantic.success.solid,
            theme,
        ))
        .child(render_semantic_counter(
            "Degraded",
            counts.degraded,
            theme.semantic.warning.solid,
            theme,
        ))
        .child(render_semantic_counter(
            "Failed",
            counts.failed,
            theme.semantic.danger.solid,
            theme,
        ))
        .into_any_element()
}

fn render_summary_toolbar(
    state: &WorkloadsOverviewState,
    namespaces: &NamespaceFilterState,
    is_loading: bool,
    theme: &OrbitTheme,
) -> AnyElement {
    let counts = state
        .snapshot
        .as_ref()
        .map(|s| s.aggregate_counts(namespaces.active_selection()))
        .unwrap_or_default();
    let has_errors = state
        .snapshot
        .as_ref()
        .is_some_and(WorkloadsOverviewSnapshot::has_any_error);

    let (health_label, health_status) = if state.snapshot.is_none() && is_loading {
        ("Loading", SemanticStatus::Info)
    } else if state.snapshot.is_none() && state.status == WorkloadsOverviewLoadStatus::Error {
        ("Error", SemanticStatus::Danger)
    } else {
        aggregate_workload_status(counts, has_errors)
    };

    theme
        .style_card(div())
        .p_4()
        .flex()
        .items_center()
        .justify_between()
        .gap_4()
        .flex_wrap()
        .child(
            div()
                .flex()
                .items_center()
                .gap_5()
                .flex_wrap()
                .child(render_summary_metric(
                    "Workloads Health",
                    render_status_badge(
                        health_label,
                        health_status,
                        StatusBadgeVariant::Pill,
                        theme,
                    ),
                    theme,
                ))
                .child(render_summary_metric(
                    "Total Workloads",
                    apply_text_step(div(), theme.typography.size_sm)
                        .font(theme.typography.tabular_ui_font(FontWeight::SEMIBOLD))
                        .text_color(theme.text.primary)
                        .child(counts.total.to_string())
                        .into_any_element(),
                    theme,
                ))
                .child(render_summary_metric(
                    "Breakdown",
                    render_health_counters(counts, theme),
                    theme,
                ))
                .child(render_summary_metric(
                    "Namespace Scope",
                    apply_text_step(div(), theme.typography.size_xs)
                        .font_weight(FontWeight::MEDIUM)
                        .text_color(theme.text.primary)
                        .child(namespaces.summary_label())
                        .into_any_element(),
                    theme,
                )),
        )
        .into_any_element()
}

fn render_loading_state(theme: &OrbitTheme) -> AnyElement {
    theme
        .style_card(div())
        .p_6()
        .flex()
        .items_center()
        .justify_center()
        .gap_2p5()
        .child(render_icon(Icon::LoaderCircle, px(16.0), theme.text.muted))
        .child(
            apply_text_step(div(), theme.typography.size_sm)
                .text_color(theme.text.secondary)
                .child(
                    "Loading workload health across pods, controllers, batch jobs, and autoscalers...",
                ),
        )
        .into_any_element()
}

fn render_error_state(
    error_message: Option<&str>,
    theme: &OrbitTheme,
    on_retry: RefreshCallback,
) -> AnyElement {
    let message = error_message.unwrap_or("Failed to load workload health summary.");
    div()
        .w_full()
        .p_5()
        .rounded(theme.style.radius_control)
        .bg(theme.semantic.danger.soft)
        .border_1()
        .border_color(theme.semantic.danger.solid)
        .flex()
        .items_center()
        .justify_between()
        .gap_4()
        .child(
            div()
                .flex()
                .items_center()
                .gap_2p5()
                .child(render_icon(
                    Icon::TriangleAlert,
                    px(16.0),
                    theme.semantic.danger.solid,
                ))
                .child(
                    apply_text_step(div(), theme.typography.size_sm)
                        .text_color(theme.semantic.danger.solid)
                        .child(message.to_string()),
                ),
        )
        .child(render_action_button(
            "workloads-overview-retry-btn",
            Icon::RefreshCw,
            "Retry",
            false,
            theme,
            on_retry,
        ))
        .into_any_element()
}

fn render_empty_state(namespaces: &NamespaceFilterState, theme: &OrbitTheme) -> AnyElement {
    let message = if namespaces.is_all_namespaces() {
        "No workloads found in this cluster."
    } else {
        "No workloads found in the selected namespace filter."
    };

    theme
        .style_card(div())
        .p_6()
        .flex()
        .flex_col()
        .items_center()
        .justify_center()
        .gap_2()
        .child(render_icon(Icon::Boxes, px(20.0), theme.text.muted))
        .child(
            apply_text_step(div(), theme.typography.size_sm)
                .font_weight(FontWeight::MEDIUM)
                .text_color(theme.text.secondary)
                .child(message),
        )
        .into_any_element()
}

fn render_health_bar(counts: WorkloadHealthCounts, theme: &OrbitTheme) -> AnyElement {
    let mut bar = div()
        .w_full()
        .h(px(6.0))
        .rounded_full()
        .overflow_hidden()
        .bg(theme.surface.control_bg)
        .flex();

    if counts.total > 0 {
        let segments = [
            (counts.healthy, theme.semantic.success.solid),
            (counts.degraded, theme.semantic.warning.solid),
            (counts.failed, theme.semantic.danger.solid),
        ];
        for (count, color) in segments {
            if count > 0 {
                let pct = (count as f32 / counts.total as f32).max(0.02);
                bar = bar.child(div().h_full().w(gpui::relative(pct)).bg(color));
            }
        }
    }

    bar.into_any_element()
}

fn render_workload_card(
    card: WorkloadSummaryCard,
    hover_bg: Rgba,
    theme: &OrbitTheme,
    on_navigate: NavigateCallback,
) -> AnyElement {
    let target_view = card.target_view;
    let subtitle = if card.counts.total == 0 {
        "No resources".to_string()
    } else {
        format!("{}% healthy", card.health_pct)
    };

    let mut card_el = theme
        .style_card(div().id(SharedString::from(format!(
            "workloads-overview-card-{}",
            card.target_view.id()
        ))))
        .flex_1()
        .min_w(px(240.0))
        .p_4()
        .cursor_pointer()
        .hover(move |s| s.bg(hover_bg))
        .flex()
        .flex_col()
        .gap_3()
        .on_click(move |_, window, cx| {
            on_navigate(target_view, window, cx);
        })
        .child(
            div()
                .flex()
                .items_center()
                .justify_between()
                .gap_2()
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap_2()
                        .child(render_icon(card.icon, px(15.0), card.kind_color))
                        .child(
                            apply_text_step(div(), theme.typography.size_sm)
                                .font_weight(FontWeight::SEMIBOLD)
                                .text_color(theme.text.primary)
                                .child(card.title),
                        ),
                )
                .child(render_status_badge(
                    card.status_label,
                    card.status,
                    StatusBadgeVariant::Pill,
                    theme,
                )),
        )
        .child(
            div()
                .flex()
                .items_baseline()
                .gap_2()
                .child(
                    apply_text_step(div(), theme.typography.size_xl)
                        .font(theme.typography.tabular_ui_font(FontWeight::SEMIBOLD))
                        .text_color(theme.text.primary)
                        .child(card.counts.total.to_string()),
                )
                .child(
                    apply_text_step(div(), theme.typography.size_xs)
                        .text_color(theme.text.muted)
                        .child(subtitle),
                ),
        )
        .child(render_health_bar(card.counts, theme))
        .child(render_health_counters(card.counts, theme));

    if let Some(err) = card.error {
        card_el = card_el.child(
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

    card_el.into_any_element()
}

fn render_summary_grid(
    snapshot: &WorkloadsOverviewSnapshot,
    namespaces: &NamespaceFilterState,
    theme: &OrbitTheme,
    on_navigate: NavigateCallback,
) -> AnyElement {
    let hover_bg = theme.surface.bg_hover;
    let mut grid = div().flex().flex_wrap().gap_3();

    for card in workload_summary_cards(snapshot, namespaces, theme) {
        grid = grid.child(render_workload_card(
            card,
            hover_bg,
            theme,
            Rc::clone(&on_navigate),
        ));
    }

    grid.into_any_element()
}

#[cfg(test)]
mod tests {
    use super::*;
    use gpui::WindowAppearance;
    use orbit_core::kubernetes::workloads_overview::{
        WorkloadHealthItem, WorkloadHealthStatus, WorkloadKindSnapshot,
    };

    use crate::theme::{AccentColor, SemanticColorsPreference, ThemeMode, UiStyle};

    fn item(ns: &str, status: WorkloadHealthStatus) -> WorkloadHealthItem {
        WorkloadHealthItem {
            namespace: ns.to_string(),
            status,
        }
    }

    fn sample_snapshot() -> WorkloadsOverviewSnapshot {
        WorkloadsOverviewSnapshot {
            pods: WorkloadKindSnapshot {
                items: vec![
                    item("default", WorkloadHealthStatus::Healthy),
                    item("prod", WorkloadHealthStatus::Degraded),
                    item("staging", WorkloadHealthStatus::Failed),
                ],
                error: None,
            },
            deployments: WorkloadKindSnapshot {
                items: vec![
                    item("prod", WorkloadHealthStatus::Healthy),
                    item("prod", WorkloadHealthStatus::Healthy),
                ],
                error: None,
            },
            statefulsets: WorkloadKindSnapshot {
                items: vec![item("prod", WorkloadHealthStatus::Degraded)],
                error: None,
            },
            daemonsets: WorkloadKindSnapshot {
                items: vec![item("kube-system", WorkloadHealthStatus::Healthy)],
                error: None,
            },
            replicasets: WorkloadKindSnapshot {
                items: Vec::new(),
                error: None,
            },
            jobs: WorkloadKindSnapshot {
                items: vec![item("staging", WorkloadHealthStatus::Healthy)],
                error: None,
            },
            cronjobs: WorkloadKindSnapshot {
                items: vec![item("default", WorkloadHealthStatus::Degraded)],
                error: None,
            },
            hpas: WorkloadKindSnapshot {
                items: Vec::new(),
                error: Some("Forbidden: hpas is forbidden".to_string()),
            },
        }
    }

    #[test]
    fn test_workload_summary_cards_cover_all_eight_kinds_with_navigation_targets_and_semantic_tokens()
    {
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

        let snapshot = sample_snapshot();
        let namespaces = NamespaceFilterState::default();

        let expected_views = [
            (WorkspaceView::Pods, K8sResourceKind::Pod),
            (WorkspaceView::Deployments, K8sResourceKind::Deployment),
            (WorkspaceView::StatefulSets, K8sResourceKind::StatefulSet),
            (WorkspaceView::DaemonSets, K8sResourceKind::DaemonSet),
            (WorkspaceView::ReplicaSets, K8sResourceKind::ReplicaSet),
            (WorkspaceView::Jobs, K8sResourceKind::Job),
            (WorkspaceView::CronJobs, K8sResourceKind::Job),
            (WorkspaceView::Hpas, K8sResourceKind::Hpa),
        ];

        for theme in [&dark_theme, &light_theme] {
            let cards = workload_summary_cards(&snapshot, &namespaces, theme);
            assert_eq!(cards.len(), 8);

            for (card, (expected_view, expected_k8s_kind)) in
                cards.iter().zip(expected_views.iter())
            {
                assert_eq!(card.target_view, *expected_view);
                assert_eq!(card.icon.unicode(), expected_view.icon().unicode());
                assert_eq!(card.kind_color, theme.k8s_color(*expected_k8s_kind));
            }

            // Pods: 1 healthy, 1 degraded, 1 failed -> Danger, 33%
            assert_eq!(cards[0].status, SemanticStatus::Danger);
            assert_eq!(cards[0].health_pct, 33);

            // Deployments: 2 healthy -> Success, 100%
            assert_eq!(cards[1].status, SemanticStatus::Success);
            assert_eq!(cards[1].health_pct, 100);

            // StatefulSets: 1 degraded -> Warning, 0%
            assert_eq!(cards[2].status, SemanticStatus::Warning);
            assert_eq!(cards[2].health_pct, 0);

            // ReplicaSets: 0 total, no error -> Neutral
            assert_eq!(cards[4].status, SemanticStatus::Neutral);
            assert_eq!(cards[4].counts.total, 0);

            // HPAs: 0 total, RBAC error -> Warning + error propagated
            assert_eq!(cards[7].status, SemanticStatus::Warning);
            assert_eq!(
                cards[7].error.as_deref(),
                Some("Forbidden: hpas is forbidden")
            );
        }
    }

    #[test]
    fn test_workload_summary_cards_respect_active_namespace_filter() {
        let theme = OrbitTheme::resolve(
            ThemeMode::Dark,
            UiStyle::Flat,
            AccentColor::Blue,
            SemanticColorsPreference::default(),
            WindowAppearance::Dark,
        );
        let snapshot = sample_snapshot();
        let mut namespaces = NamespaceFilterState {
            cluster_id: Some("ctx-a".to_string()),
            ..Default::default()
        };

        // All namespaces: total = 9
        let all_cards = workload_summary_cards(&snapshot, &namespaces, &theme);
        let all_total: usize = all_cards.iter().map(|c| c.counts.total).sum();
        assert_eq!(all_total, 9);

        // Single namespace: "prod" -> 1 pod, 2 deployments, 1 statefulset = 4
        namespaces.toggle_namespace("prod");
        let prod_cards = workload_summary_cards(&snapshot, &namespaces, &theme);
        assert_eq!(prod_cards[0].counts.total, 1);
        assert_eq!(prod_cards[0].counts.degraded, 1);
        assert_eq!(prod_cards[1].counts.total, 2);
        assert_eq!(prod_cards[1].counts.healthy, 2);
        assert_eq!(prod_cards[2].counts.total, 1);
        assert_eq!(prod_cards[3].counts.total, 0);
        let prod_total: usize = prod_cards.iter().map(|c| c.counts.total).sum();
        assert_eq!(prod_total, 4);

        // Multi-select: "prod" + "staging" -> adds 1 failed pod and 1 healthy job = 6
        namespaces.toggle_namespace("staging");
        let multi_cards = workload_summary_cards(&snapshot, &namespaces, &theme);
        assert_eq!(multi_cards[0].counts.total, 2);
        assert_eq!(multi_cards[0].counts.failed, 1);
        assert_eq!(multi_cards[5].counts.total, 1);
        let multi_total: usize = multi_cards.iter().map(|c| c.counts.total).sum();
        assert_eq!(multi_total, 6);
    }

    #[test]
    fn test_resolve_workloads_overview_presentation_modes() {
        let mut namespaces = NamespaceFilterState {
            cluster_id: Some("ctx-a".to_string()),
            ..Default::default()
        };

        // Loading (no snapshot)
        let mut state = WorkloadsOverviewState {
            status: WorkloadsOverviewLoadStatus::Loading,
            ..Default::default()
        };
        assert_eq!(
            resolve_workloads_overview_mode(&state, &namespaces),
            WorkloadsOverviewPresentationMode::Loading
        );

        // Error (no snapshot)
        state.status = WorkloadsOverviewLoadStatus::Error;
        state.error_message = Some("cluster unreachable".to_string());
        assert_eq!(
            resolve_workloads_overview_mode(&state, &namespaces),
            WorkloadsOverviewPresentationMode::Error
        );

        // Empty (cluster-empty, no errors)
        state.status = WorkloadsOverviewLoadStatus::Loaded;
        state.snapshot = Some(WorkloadsOverviewSnapshot::default());
        assert_eq!(
            resolve_workloads_overview_mode(&state, &namespaces),
            WorkloadsOverviewPresentationMode::Empty
        );

        // Empty (namespace-filtered-empty, no errors)
        state.snapshot = Some(WorkloadsOverviewSnapshot {
            pods: WorkloadKindSnapshot {
                items: vec![item("default", WorkloadHealthStatus::Healthy)],
                error: None,
            },
            ..Default::default()
        });
        namespaces.toggle_namespace("prod");
        assert_eq!(
            resolve_workloads_overview_mode(&state, &namespaces),
            WorkloadsOverviewPresentationMode::Empty
        );

        // Summary (total is 0 in selected namespace, but RBAC error exists)
        state.snapshot = Some(WorkloadsOverviewSnapshot {
            hpas: WorkloadKindSnapshot {
                items: Vec::new(),
                error: Some("Forbidden".to_string()),
            },
            ..Default::default()
        });
        assert_eq!(
            resolve_workloads_overview_mode(&state, &namespaces),
            WorkloadsOverviewPresentationMode::Summary
        );
    }
}
