use std::rc::Rc;

use gpui::{AnyElement, App, FontWeight, Rgba, SharedString, Window, div, prelude::*, px};
use lucide_icons::Icon;
use orbit_core::kubernetes::storage_overview::{
    StorageOverviewSnapshot, format_storage_gib,
};

use crate::components::{SemanticStatus, StatusBadgeVariant, render_status_badge};
use crate::icons::render_icon;
use crate::navigation::WorkspaceView;
use crate::state::{NamespaceFilterState, StorageOverviewLoadStatus, StorageOverviewState};
use crate::theme::{OrbitTheme, apply_text_step};
use crate::views::overview::NavigateCallback;
use crate::views::render_action_button;

type RefreshCallback = Rc<dyn Fn(&mut Window, &mut App)>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StorageOverviewPresentationMode {
    Loading,
    Error,
    Empty,
    Summary,
}

#[derive(Debug, Clone, PartialEq)]
pub struct StorageMetricTile {
    pub label: &'static str,
    pub value: String,
    pub subtitle: String,
    pub accent_color: Option<Rgba>,
}

#[derive(Debug, Clone)]
pub struct StorageResourceCard {
    pub title: &'static str,
    pub target_view: WorkspaceView,
    pub icon: Icon,
    pub total: usize,
    pub status: SemanticStatus,
    pub status_label: &'static str,
    pub primary_label: &'static str,
    pub primary_count: usize,
    pub warning_label: &'static str,
    pub warning_count: usize,
    pub danger_label: &'static str,
    pub danger_count: usize,
    pub secondary_label: Option<(&'static str, usize)>,
    pub detail_subtitle: String,
    pub error: Option<String>,
}

pub fn resolve_storage_overview_mode(
    state: &StorageOverviewState,
    namespaces: &NamespaceFilterState,
) -> StorageOverviewPresentationMode {
    match state.snapshot.as_ref() {
        Some(snapshot) => {
            if snapshot.all_failed() {
                return StorageOverviewPresentationMode::Error;
            }
            let pv_total = snapshot.persistent_volumes.items.len();
            let pvc_total = snapshot
                .pvc_status_counts_for_namespaces(namespaces.active_selection())
                .total;
            let sc_total = snapshot.storage_classes.items.len();

            if pv_total == 0 && pvc_total == 0 && sc_total == 0 && !snapshot.has_any_error() {
                StorageOverviewPresentationMode::Empty
            } else {
                StorageOverviewPresentationMode::Summary
            }
        }
        None => match state.status {
            StorageOverviewLoadStatus::Error => StorageOverviewPresentationMode::Error,
            StorageOverviewLoadStatus::Idle
            | StorageOverviewLoadStatus::Loading
            | StorageOverviewLoadStatus::Loaded => StorageOverviewPresentationMode::Loading,
        },
    }
}

pub fn aggregate_storage_status(
    snapshot: &StorageOverviewSnapshot,
    namespaces: &NamespaceFilterState,
) -> (&'static str, SemanticStatus) {
    let pv = snapshot.pv_status_counts();
    let pvc = snapshot.pvc_status_counts_for_namespaces(namespaces.active_selection());
    let sc_total = snapshot.storage_classes.items.len();

    if pv.failed > 0 || pvc.lost > 0 {
        ("Failed", SemanticStatus::Danger)
    } else if pv.released > 0 || pvc.pending > 0 || snapshot.has_any_error() {
        ("Degraded", SemanticStatus::Warning)
    } else if pv.total > 0 || pvc.total > 0 || sc_total > 0 {
        ("Healthy", SemanticStatus::Success)
    } else {
        ("No Storage", SemanticStatus::Neutral)
    }
}

pub fn storage_metric_tiles(
    snapshot: &StorageOverviewSnapshot,
    namespaces: &NamespaceFilterState,
    theme: &OrbitTheme,
) -> [StorageMetricTile; 5] {
    let pv = snapshot.pv_status_counts();
    let pvc = snapshot.pvc_status_counts_for_namespaces(namespaces.active_selection());
    let cap = snapshot.capacity_summary();
    let sc_total = snapshot.storage_classes.items.len();
    let sc_expandable = snapshot.storage_classes_expandable_count();

    [
        StorageMetricTile {
            label: "Total Capacity",
            value: format_storage_gib(cap.total_gib),
            subtitle: format!("{} Total Volumes", pv.total),
            accent_color: None,
        },
        StorageMetricTile {
            label: "Bound Capacity",
            value: format_storage_gib(cap.bound_gib),
            subtitle: format!("{}% of cluster total", cap.bound_pct),
            accent_color: Some(theme.semantic.success.solid),
        },
        StorageMetricTile {
            label: "Available Capacity",
            value: format_storage_gib(cap.available_gib),
            subtitle: format!("{}% remaining", cap.available_pct),
            accent_color: Some(theme.accent.accent),
        },
        StorageMetricTile {
            label: "Volume Claims",
            value: pvc.total.to_string(),
            subtitle: format!("{} Bound · {} Pending", pvc.bound, pvc.pending),
            accent_color: if pvc.lost > 0 {
                Some(theme.semantic.danger.solid)
            } else if pvc.pending > 0 {
                Some(theme.semantic.warning.solid)
            } else {
                None
            },
        },
        StorageMetricTile {
            label: "Storage Classes",
            value: sc_total.to_string(),
            subtitle: format!("{sc_expandable} expandable"),
            accent_color: None,
        },
    ]
}

pub fn storage_resource_cards(
    snapshot: &StorageOverviewSnapshot,
    namespaces: &NamespaceFilterState,
) -> [StorageResourceCard; 3] {
    let pv = snapshot.pv_status_counts();
    let pvc = snapshot.pvc_status_counts_for_namespaces(namespaces.active_selection());
    let sc_total = snapshot.storage_classes.items.len();
    let sc_expandable = snapshot.storage_classes_expandable_count();

    let (pv_label, pv_status) = if pv.failed > 0 {
        ("Failed", SemanticStatus::Danger)
    } else if pv.released > 0 || snapshot.persistent_volumes.error.is_some() {
        ("Degraded", SemanticStatus::Warning)
    } else if pv.total > 0 {
        ("Healthy", SemanticStatus::Success)
    } else {
        ("Empty", SemanticStatus::Neutral)
    };

    let (pvc_label, pvc_status) = if pvc.lost > 0 {
        ("Failed", SemanticStatus::Danger)
    } else if pvc.pending > 0 || snapshot.persistent_volume_claims.error.is_some() {
        ("Degraded", SemanticStatus::Warning)
    } else if pvc.total > 0 {
        ("Healthy", SemanticStatus::Success)
    } else {
        ("Empty", SemanticStatus::Neutral)
    };

    let (sc_label, sc_status) = if snapshot.storage_classes.error.is_some() {
        ("Restricted", SemanticStatus::Warning)
    } else if sc_total > 0 {
        ("Configured", SemanticStatus::Success)
    } else {
        ("Empty", SemanticStatus::Neutral)
    };

    let unique_provisioners = {
        let mut set = std::collections::BTreeSet::new();
        for sc in &snapshot.storage_classes.items {
            if !sc.provisioner.trim().is_empty() {
                set.insert(sc.provisioner.as_str());
            }
        }
        set.len()
    };

    [
        StorageResourceCard {
            title: WorkspaceView::PersistentVolumes.label(),
            target_view: WorkspaceView::PersistentVolumes,
            icon: WorkspaceView::PersistentVolumes.icon(),
            total: pv.total,
            status: pv_status,
            status_label: pv_label,
            primary_label: "Bound",
            primary_count: pv.bound,
            warning_label: "Released",
            warning_count: pv.released,
            danger_label: "Failed",
            danger_count: pv.failed,
            secondary_label: Some(("Available", pv.available)),
            detail_subtitle: "Cluster-scoped persistent volumes".to_string(),
            error: snapshot.persistent_volumes.error.clone(),
        },
        StorageResourceCard {
            title: WorkspaceView::PersistentVolumeClaims.label(),
            target_view: WorkspaceView::PersistentVolumeClaims,
            icon: WorkspaceView::PersistentVolumeClaims.icon(),
            total: pvc.total,
            status: pvc_status,
            status_label: pvc_label,
            primary_label: "Bound",
            primary_count: pvc.bound,
            warning_label: "Pending",
            warning_count: pvc.pending,
            danger_label: "Lost",
            danger_count: pvc.lost,
            secondary_label: None,
            detail_subtitle: if namespaces.is_all_namespaces() {
                "All namespaces".to_string()
            } else {
                format!("Namespace: {}", namespaces.summary_label())
            },
            error: snapshot.persistent_volume_claims.error.clone(),
        },
        StorageResourceCard {
            title: WorkspaceView::StorageClasses.label(),
            target_view: WorkspaceView::StorageClasses,
            icon: WorkspaceView::StorageClasses.icon(),
            total: sc_total,
            status: sc_status,
            status_label: sc_label,
            primary_label: "Expandable",
            primary_count: sc_expandable,
            warning_label: "Fixed",
            warning_count: sc_total.saturating_sub(sc_expandable),
            danger_label: "Failed",
            danger_count: 0,
            secondary_label: None,
            detail_subtitle: format!("{unique_provisioners} provisioner(s)"),
            error: snapshot.storage_classes.error.clone(),
        },
    ]
}

pub fn render_storage_overview_view(
    state: &StorageOverviewState,
    namespaces: &NamespaceFilterState,
    theme: &OrbitTheme,
    on_navigate: NavigateCallback,
    on_refresh: impl Fn(&mut Window, &mut App) + 'static,
) -> AnyElement {
    let refresh_cb: RefreshCallback = Rc::new(on_refresh);
    let mode = resolve_storage_overview_mode(state, namespaces);
    let is_loading = state.status == StorageOverviewLoadStatus::Loading;

    let mut root = div().flex().flex_col().gap_5().child(render_summary_toolbar(
        state,
        namespaces,
        is_loading,
        theme,
    ));

    if let Some(ref err) = state.error_message
        && state.snapshot.is_some()
        && mode != StorageOverviewPresentationMode::Error
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
        StorageOverviewPresentationMode::Loading => render_loading_state(theme),
        StorageOverviewPresentationMode::Error => {
            let err_msg = state.error_message.as_deref().or_else(|| {
                state
                    .snapshot
                    .as_ref()
                    .and_then(|s| s.persistent_volumes.error.as_deref())
            });
            render_error_state(err_msg, is_loading, theme, refresh_cb)
        }
        StorageOverviewPresentationMode::Empty => render_empty_state(namespaces, theme),
        StorageOverviewPresentationMode::Summary => match state.snapshot.as_ref() {
            Some(snapshot) => render_summary_body(snapshot, namespaces, theme, on_navigate),
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

fn render_summary_toolbar(
    state: &StorageOverviewState,
    namespaces: &NamespaceFilterState,
    is_loading: bool,
    theme: &OrbitTheme,
) -> AnyElement {
    let (health_label, health_status, total_cap_text, bound_avail_el) =
        if let Some(snapshot) = state.snapshot.as_ref() {
            let (label, status) = aggregate_storage_status(snapshot, namespaces);
            let cap = snapshot.capacity_summary();
            let bound_avail = div()
                .flex()
                .items_center()
                .gap_3()
                .child(
                    apply_text_step(div(), theme.typography.size_xs)
                        .font(theme.typography.tabular_ui_font(FontWeight::MEDIUM))
                        .text_color(theme.semantic.success.solid)
                        .child(format!(
                            "Bound {} ({}%)",
                            format_storage_gib(cap.bound_gib),
                            cap.bound_pct
                        )),
                )
                .child(
                    apply_text_step(div(), theme.typography.size_xs)
                        .font(theme.typography.tabular_ui_font(FontWeight::MEDIUM))
                        .text_color(theme.text.secondary)
                        .child(format!(
                            "Available {} ({}%)",
                            format_storage_gib(cap.available_gib),
                            cap.available_pct
                        )),
                )
                .into_any_element();
            (label, status, format_storage_gib(cap.total_gib), bound_avail)
        } else if state.status == StorageOverviewLoadStatus::Error {
            (
                "Error",
                SemanticStatus::Danger,
                "-".to_string(),
                apply_text_step(div(), theme.typography.size_xs)
                    .text_color(theme.text.muted)
                    .child("-")
                    .into_any_element(),
            )
        } else {
            (
                "Loading",
                SemanticStatus::Info,
                "-".to_string(),
                apply_text_step(div(), theme.typography.size_xs)
                    .text_color(theme.text.muted)
                    .child("Collecting storage inventory...")
                    .into_any_element(),
            )
        };

    let scope_note = if namespaces.is_all_namespaces() {
        "All Namespaces (PVs & Classes cluster-scoped)".to_string()
    } else {
        format!(
            "{} (PVCs filtered; PVs & Classes cluster-scoped)",
            namespaces.summary_label()
        )
    };

    theme
        .style_card(div())
        .p_4()
        .bg(theme.surface.bg_panel)
        .flex()
        .flex_wrap()
        .items_center()
        .justify_between()
        .gap_4()
        .child(
            div()
                .flex()
                .flex_wrap()
                .items_center()
                .gap_6()
                .child(render_summary_metric(
                    "Storage Health",
                    render_status_badge(health_label, health_status, StatusBadgeVariant::Pill, theme),
                    theme,
                ))
                .child(render_summary_metric(
                    "Total Capacity",
                    apply_text_step(div(), theme.typography.size_sm)
                        .font(theme.typography.tabular_ui_font(FontWeight::SEMIBOLD))
                        .text_color(theme.text.primary)
                        .child(total_cap_text)
                        .into_any_element(),
                    theme,
                ))
                .child(render_summary_metric(
                    "Bound / Available",
                    bound_avail_el,
                    theme,
                )),
        )
        .child(
            div()
                .flex()
                .items_center()
                .gap_2()
                .when(is_loading, |el| {
                    el.child(render_icon(
                        Icon::LoaderCircle,
                        px(13.0),
                        theme.text.muted,
                    ))
                })
                .child(
                    apply_text_step(div(), theme.typography.size_xs)
                        .text_color(theme.text.muted)
                        .child(scope_note),
                ),
        )
        .into_any_element()
}

fn render_summary_body(
    snapshot: &StorageOverviewSnapshot,
    namespaces: &NamespaceFilterState,
    theme: &OrbitTheme,
    on_navigate: NavigateCallback,
) -> AnyElement {
    let tiles = storage_metric_tiles(snapshot, namespaces, theme);
    let cards = storage_resource_cards(snapshot, namespaces);
    let breakdown = snapshot.storage_class_capacity_breakdown();

    let kpi_strip = div()
        .w_full()
        .grid()
        .grid_cols(5)
        .gap_3()
        .children(tiles.into_iter().map(|tile| {
            theme
                .style_card(div())
                .p_3()
                .bg(theme.surface.bg_card)
                .flex()
                .flex_col()
                .gap_1()
                .child(
                    apply_text_step(div(), theme.typography.size_xs)
                        .text_color(theme.text.muted)
                        .child(tile.label),
                )
                .child(
                    apply_text_step(div(), theme.typography.size_lg)
                        .font(theme.typography.tabular_ui_font(FontWeight::BOLD))
                        .text_color(tile.accent_color.unwrap_or(theme.text.primary))
                        .child(tile.value),
                )
                .child(
                    apply_text_step(div(), theme.typography.size_2xs)
                        .text_color(theme.text.secondary)
                        .truncate()
                        .child(tile.subtitle),
                )
        }));

    let resource_cards_grid = div()
        .w_full()
        .grid()
        .grid_cols(3)
        .gap_4()
        .children(cards.into_iter().enumerate().map(|(idx, card)| {
            render_storage_resource_card(idx, &card, theme, on_navigate.clone())
        }));

    let breakdown_panel = render_storage_class_breakdown(&breakdown, theme);

    div()
        .w_full()
        .flex()
        .flex_col()
        .gap_5()
        .child(kpi_strip)
        .child(resource_cards_grid)
        .child(breakdown_panel)
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

fn render_segmented_health_bar(card: &StorageResourceCard, theme: &OrbitTheme) -> AnyElement {
    let total = card.total;
    if total == 0 {
        return div()
            .w_full()
            .h(px(5.0))
            .rounded_full()
            .bg(theme.surface.control_bg)
            .into_any_element();
    }

    let secondary = card.secondary_label.map(|(_, c)| c).unwrap_or(0);
    div()
        .w_full()
        .h(px(5.0))
        .rounded_full()
        .overflow_hidden()
        .bg(theme.surface.control_bg)
        .flex()
        .when(card.primary_count > 0, |el| {
            el.child(
                div()
                    .h_full()
                    .w(gpui::relative(card.primary_count as f32 / total as f32))
                    .bg(theme.semantic.success.solid),
            )
        })
        .when(secondary > 0, |el| {
            el.child(
                div()
                    .h_full()
                    .w(gpui::relative(secondary as f32 / total as f32))
                    .bg(theme.accent.accent),
            )
        })
        .when(card.warning_count > 0, |el| {
            el.child(
                div()
                    .h_full()
                    .w(gpui::relative(card.warning_count as f32 / total as f32))
                    .bg(theme.semantic.warning.solid),
            )
        })
        .when(card.danger_count > 0, |el| {
            el.child(
                div()
                    .h_full()
                    .w(gpui::relative(card.danger_count as f32 / total as f32))
                    .bg(theme.semantic.danger.solid),
            )
        })
        .into_any_element()
}

fn render_storage_resource_card(
    idx: usize,
    card: &StorageResourceCard,
    theme: &OrbitTheme,
    on_navigate: NavigateCallback,
) -> AnyElement {
    let target_view = card.target_view;
    let hover_bg = theme.surface.bg_hover;

    let mut counters = div()
        .flex()
        .flex_wrap()
        .items_center()
        .gap_3()
        .child(render_semantic_counter(
            card.primary_label,
            card.primary_count,
            theme.semantic.success.solid,
            theme,
        ));
    if let Some((sec_label, sec_count)) = card.secondary_label {
        counters = counters.child(render_semantic_counter(
            sec_label,
            sec_count,
            theme.accent.accent,
            theme,
        ));
    }
    counters = counters.child(render_semantic_counter(
        card.warning_label,
        card.warning_count,
        theme.semantic.warning.solid,
        theme,
    ));
    if card.danger_count > 0 || card.target_view != WorkspaceView::StorageClasses {
        counters = counters.child(render_semantic_counter(
            card.danger_label,
            card.danger_count,
            theme.semantic.danger.solid,
            theme,
        ));
    }

    theme
        .style_card(div())
        .id(SharedString::from(format!("storage-overview-card-{idx}")))
        .p_4()
        .bg(theme.surface.bg_card)
        .cursor_pointer()
        .hover(move |style| style.bg(hover_bg))
        .on_click(move |_, window, cx| on_navigate(target_view, window, cx))
        .flex()
        .flex_col()
        .gap_3()
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
                        .child(render_icon(card.icon, px(15.0), theme.text.secondary))
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
                .justify_between()
                .gap_2()
                .child(
                    apply_text_step(div(), theme.typography.size_xl)
                        .font(theme.typography.tabular_ui_font(FontWeight::BOLD))
                        .text_color(theme.text.primary)
                        .child(card.total.to_string()),
                )
                .child(
                    apply_text_step(div(), theme.typography.size_2xs)
                        .text_color(theme.text.muted)
                        .truncate()
                        .child(card.detail_subtitle.clone()),
                ),
        )
        .child(render_segmented_health_bar(card, theme))
        .child(counters)
        .when_some(card.error.as_ref(), |el, err| {
            el.child(
                apply_text_step(div(), theme.typography.size_2xs)
                    .px_2()
                    .py_1()
                    .rounded(theme.style.radius_control)
                    .bg(theme.semantic.warning.soft)
                    .text_color(theme.semantic.warning.solid)
                    .truncate()
                    .child(format!("Partial access: {err}")),
            )
        })
        .into_any_element()
}

fn render_storage_class_breakdown(
    entries: &[orbit_core::kubernetes::storage_overview::StorageClassCapacityEntry],
    theme: &OrbitTheme,
) -> AnyElement {
    let body = if entries.is_empty() {
        apply_text_step(div(), theme.typography.size_xs)
            .text_color(theme.text.muted)
            .child("No StorageClasses or PersistentVolumes found in this cluster.")
            .into_any_element()
    } else {
        div()
            .w_full()
            .flex()
            .flex_col()
            .gap_3()
            .children(entries.iter().map(|entry| {
                let fill_pct = entry.share_pct as f32 / 100.0;
                let meta_label = match (&entry.provisioner, entry.allow_volume_expansion) {
                    (Some(prov), Some(true)) => format!("{prov} · Expandable"),
                    (Some(prov), _) => prov.clone(),
                    (None, _) => "No StorageClass assigned".to_string(),
                };
                div()
                    .w_full()
                    .flex()
                    .flex_col()
                    .gap_1p5()
                    .child(
                        div()
                            .w_full()
                            .flex()
                            .items_center()
                            .justify_between()
                            .gap_3()
                            .child(
                                div()
                                    .flex()
                                    .items_center()
                                    .gap_2()
                                    .min_w(px(0.0))
                                    .child(
                                        apply_text_step(div(), theme.typography.size_xs)
                                            .font_weight(FontWeight::SEMIBOLD)
                                            .text_color(theme.text.primary)
                                            .child(entry.name.clone()),
                                    )
                                    .child(
                                        apply_text_step(div(), theme.typography.size_2xs)
                                            .text_color(theme.text.muted)
                                            .truncate()
                                            .child(meta_label),
                                    ),
                            )
                            .child(
                                div()
                                    .flex()
                                    .items_center()
                                    .gap_3()
                                    .flex_shrink_0()
                                    .child(
                                        apply_text_step(div(), theme.typography.size_2xs)
                                            .font(theme.typography.tabular_ui_font(FontWeight::NORMAL))
                                            .text_color(theme.text.muted)
                                            .child(format!("{} vol(s)", entry.volume_count)),
                                    )
                                    .child(
                                        apply_text_step(div(), theme.typography.size_xs)
                                            .font(theme.typography.mono_font(FontWeight::MEDIUM))
                                            .text_color(theme.text.primary)
                                            .child(format!(
                                                "{} ({}%)",
                                                format_storage_gib(entry.total_gib),
                                                entry.share_pct
                                            )),
                                    ),
                            ),
                    )
                    .child(
                        div()
                            .w_full()
                            .h(px(5.0))
                            .rounded_full()
                            .overflow_hidden()
                            .bg(theme.surface.control_bg)
                            .when(entry.share_pct > 0, |el| {
                                el.child(
                                    div()
                                        .h_full()
                                        .w(gpui::relative(fill_pct))
                                        .rounded_full()
                                        .bg(theme.accent.accent),
                                )
                            }),
                    )
            }))
            .into_any_element()
    };

    theme
        .style_card(div())
        .p_4()
        .bg(theme.surface.bg_card)
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
                        .child("Storage Class Capacity Distribution"),
                )
                .child(
                    apply_text_step(div(), theme.typography.size_2xs)
                        .text_color(theme.text.muted)
                        .child(format!("{} class(es)", entries.len())),
                ),
        )
        .child(body)
        .into_any_element()
}

fn render_loading_state(theme: &OrbitTheme) -> AnyElement {
    theme
        .style_card(div())
        .p_8()
        .bg(theme.surface.bg_panel)
        .flex()
        .items_center()
        .justify_center()
        .gap_2()
        .child(render_icon(Icon::LoaderCircle, px(16.0), theme.text.muted))
        .child(
            apply_text_step(div(), theme.typography.size_sm)
                .text_color(theme.text.secondary)
                .child("Loading storage overview..."),
        )
        .into_any_element()
}

fn render_empty_state(namespaces: &NamespaceFilterState, theme: &OrbitTheme) -> AnyElement {
    let subtitle = if namespaces.is_all_namespaces() {
        "No PersistentVolumes, PersistentVolumeClaims, or StorageClasses were found in this cluster."
    } else {
        "No storage resources were found for the active namespace filter."
    };

    theme
        .style_card(div())
        .p_8()
        .bg(theme.surface.bg_panel)
        .flex()
        .flex_col()
        .items_center()
        .justify_center()
        .gap_2()
        .child(render_icon(Icon::HardDrive, px(22.0), theme.text.muted))
        .child(
            apply_text_step(div(), theme.typography.size_sm)
                .font_weight(FontWeight::SEMIBOLD)
                .text_color(theme.text.primary)
                .child("No Storage Resources"),
        )
        .child(
            apply_text_step(div(), theme.typography.size_xs)
                .text_color(theme.text.secondary)
                .child(subtitle),
        )
        .into_any_element()
}

fn render_error_state(
    message: Option<&str>,
    is_loading: bool,
    theme: &OrbitTheme,
    on_refresh: RefreshCallback,
) -> AnyElement {
    theme
        .style_card(div())
        .p_6()
        .bg(theme.surface.bg_panel)
        .flex()
        .flex_col()
        .items_start()
        .gap_3()
        .child(
            div()
                .flex()
                .items_center()
                .gap_2()
                .child(render_icon(
                    Icon::AlertTriangle,
                    px(16.0),
                    theme.semantic.danger.solid,
                ))
                .child(
                    apply_text_step(div(), theme.typography.size_sm)
                        .font_weight(FontWeight::SEMIBOLD)
                        .text_color(theme.text.primary)
                        .child("Unable to Load Storage Overview"),
                ),
        )
        .child(
            apply_text_step(div(), theme.typography.size_xs)
                .text_color(theme.text.secondary)
                .child(message.unwrap_or("Unknown error").to_string()),
        )
        .child(render_action_button(
            "storage-overview-retry-btn",
            Icon::RotateCcw,
            "Retry",
            is_loading,
            theme,
            on_refresh,
        ))
        .into_any_element()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::theme::{AccentColor, SemanticColorsPreference, ThemeMode, UiStyle};
    use orbit_core::kubernetes::models::{
        PersistentVolumeClaimInfo, PersistentVolumeInfo, StorageClassInfo,
    };
    use orbit_core::kubernetes::storage_overview::build_storage_overview_snapshot;

    fn sample_pv(name: &str, capacity: &str, status: &str, sc: &str) -> PersistentVolumeInfo {
        PersistentVolumeInfo {
            name: name.to_string(),
            capacity: capacity.to_string(),
            access_mode: "ReadWriteOnce".to_string(),
            reclaim_policy: "Delete".to_string(),
            status: status.to_string(),
            storage_class: sc.to_string(),
            age: "2d".to_string(),
            volume_mode: "Filesystem".to_string(),
            reason: None,
        }
    }

    fn sample_pvc(name: &str, namespace: &str, status: &str) -> PersistentVolumeClaimInfo {
        PersistentVolumeClaimInfo {
            name: name.to_string(),
            namespace: namespace.to_string(),
            status: status.to_string(),
            volume: "pv-1".to_string(),
            capacity: "10Gi".to_string(),
            access_mode: "ReadWriteOnce".to_string(),
            storage_class: "gp3".to_string(),
            age: "1d".to_string(),
        }
    }

    fn sample_sc(name: &str, provisioner: &str, expandable: bool) -> StorageClassInfo {
        StorageClassInfo {
            name: name.to_string(),
            provisioner: provisioner.to_string(),
            reclaim_policy: "Delete".to_string(),
            volume_binding_mode: "WaitForFirstConsumer".to_string(),
            allow_volume_expansion: expandable,
            age: "10d".to_string(),
        }
    }

    #[test]
    fn test_resolve_storage_overview_presentation_modes() {
        let ns = NamespaceFilterState::default();
        let mut state = StorageOverviewState::default();
        assert_eq!(
            resolve_storage_overview_mode(&state, &ns),
            StorageOverviewPresentationMode::Loading
        );

        state.status = StorageOverviewLoadStatus::Error;
        assert_eq!(
            resolve_storage_overview_mode(&state, &ns),
            StorageOverviewPresentationMode::Error
        );

        // Snapshot where all 3 sources failed -> Error
        state.snapshot = Some(build_storage_overview_snapshot(
            Err("pv err".to_string()),
            Err("pvc err".to_string()),
            Err("sc err".to_string()),
        ));
        state.status = StorageOverviewLoadStatus::Loaded;
        assert_eq!(
            resolve_storage_overview_mode(&state, &ns),
            StorageOverviewPresentationMode::Error
        );

        // Empty snapshot without errors -> Empty
        state.snapshot = Some(build_storage_overview_snapshot(
            Ok(vec![]),
            Ok(vec![]),
            Ok(vec![]),
        ));
        assert_eq!(
            resolve_storage_overview_mode(&state, &ns),
            StorageOverviewPresentationMode::Empty
        );

        // Partial RBAC error -> Summary
        state.snapshot = Some(build_storage_overview_snapshot(
            Err("forbidden".to_string()),
            Ok(vec![]),
            Ok(vec![]),
        ));
        assert_eq!(
            resolve_storage_overview_mode(&state, &ns),
            StorageOverviewPresentationMode::Summary
        );
    }

    #[test]
    fn test_storage_summary_cards_cover_three_resources_with_navigation_and_semantic_status() {
        let ns = NamespaceFilterState::default();

        // Healthy cluster
        let healthy_snap = build_storage_overview_snapshot(
            Ok(vec![sample_pv("pv-1", "20Gi", "Bound", "gp3")]),
            Ok(vec![sample_pvc("pvc-1", "prod", "Bound")]),
            Ok(vec![sample_sc("gp3", "ebs.csi.aws.com", true)]),
        );
        let cards = storage_resource_cards(&healthy_snap, &ns);
        assert_eq!(cards[0].target_view, WorkspaceView::PersistentVolumes);
        assert_eq!(cards[0].status, SemanticStatus::Success);
        assert_eq!(cards[1].target_view, WorkspaceView::PersistentVolumeClaims);
        assert_eq!(cards[1].status, SemanticStatus::Success);
        assert_eq!(cards[2].target_view, WorkspaceView::StorageClasses);
        assert_eq!(cards[2].status, SemanticStatus::Success);

        // Failed PV + Pending PVC + RBAC restricted SC
        let mixed_snap = build_storage_overview_snapshot(
            Ok(vec![sample_pv("pv-fail", "10Gi", "Failed", "gp3")]),
            Ok(vec![sample_pvc("pvc-pend", "prod", "Pending")]),
            Err("forbidden".to_string()),
        );
        let mixed_cards = storage_resource_cards(&mixed_snap, &ns);
        assert_eq!(mixed_cards[0].status, SemanticStatus::Danger);
        assert_eq!(mixed_cards[1].status, SemanticStatus::Warning);
        assert_eq!(mixed_cards[2].status, SemanticStatus::Warning);
        assert_eq!(mixed_cards[2].status_label, "Restricted");

        // Empty -> Neutral
        let empty_snap = build_storage_overview_snapshot(Ok(vec![]), Ok(vec![]), Ok(vec![]));
        let empty_cards = storage_resource_cards(&empty_snap, &ns);
        assert!(empty_cards.iter().all(|c| c.status == SemanticStatus::Neutral));
    }

    #[test]
    fn test_storage_overview_respects_namespace_filter_for_pvcs_only() {
        let snap = build_storage_overview_snapshot(
            Ok(vec![
                sample_pv("pv-1", "10Gi", "Bound", "gp3"),
                sample_pv("pv-2", "20Gi", "Available", "gp3"),
            ]),
            Ok(vec![
                sample_pvc("pvc-1", "prod", "Bound"),
                sample_pvc("pvc-2", "staging", "Pending"),
            ]),
            Ok(vec![sample_sc("gp3", "ebs.csi.aws.com", true)]),
        );

        let mut ns = NamespaceFilterState {
            cluster_id: Some("ctx-a".to_string()),
            ..Default::default()
        };
        ns.toggle_namespace("prod");

        let cards = storage_resource_cards(&snap, &ns);
        assert_eq!(cards[0].total, 2, "PVs remain cluster-scoped");
        assert_eq!(cards[1].total, 1, "PVCs filtered to prod");
        assert_eq!(cards[2].total, 1, "StorageClasses remain cluster-scoped");
    }

    #[test]
    fn test_storage_metrics_strip_and_class_breakdown_values() {
        let theme = OrbitTheme::resolve(
            ThemeMode::Dark,
            UiStyle::Flat,
            AccentColor::Blue,
            SemanticColorsPreference::default(),
            gpui::WindowAppearance::Dark,
        );
        let ns = NamespaceFilterState::default();
        let snap = build_storage_overview_snapshot(
            Ok(vec![
                sample_pv("pv-1", "10Gi", "Bound", "gp3"),
                sample_pv("pv-2", "30Gi", "Available", "standard"),
            ]),
            Ok(vec![sample_pvc("pvc-1", "prod", "Bound")]),
            Ok(vec![
                sample_sc("gp3", "ebs.csi.aws.com", true),
                sample_sc("standard", "kubernetes.io/aws-ebs", false),
            ]),
        );

        let tiles = storage_metric_tiles(&snap, &ns, &theme);
        assert_eq!(tiles[0].value, "40.0 GiB");
        assert_eq!(tiles[1].value, "10.0 GiB");
        assert_eq!(tiles[1].subtitle, "25% of cluster total");
        assert_eq!(tiles[2].value, "30.0 GiB");
        assert_eq!(tiles[2].subtitle, "75% remaining");
        assert_eq!(tiles[3].value, "1");
        assert_eq!(tiles[4].value, "2");
        assert_eq!(tiles[4].subtitle, "1 expandable");
    }
}
