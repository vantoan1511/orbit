use std::cmp::Ordering;
use std::rc::Rc;

use gpui::{AnyElement, App, ClickEvent, FontWeight, SharedString, Window, div, prelude::*, px};
use lucide_icons::Icon;
use orbit_core::kubernetes::models::DeploymentInfo;

use crate::components::data_table::{
    DataTableCallbacks, SortState, TableColumn, render_data_table, sort_rows_stable,
};
use crate::components::{StatusBadgeVariant, render_status_badge_from_str, resource_kind_meta};
use crate::icons::render_icon;
use crate::state::{DeploymentsLoadStatus, DeploymentsState, NamespaceFilterState};
use crate::theme::{K8sResourceKind, OrbitTheme, apply_text_step};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeploymentsPresentationMode {
    Table,
    ErrorCard,
}

pub fn deployment_columns() -> Vec<TableColumn> {
    vec![
        TableColumn::new("name", "Name").flex_width(2.0, px(160.0)),
        TableColumn::new("namespace", "Namespace").flex_width(1.1, px(110.0)),
        TableColumn::new("status", "Status").fixed_width(px(115.0)),
        TableColumn::new("replicas", "Replicas")
            .fixed_width(px(95.0))
            .right_numeric(),
        TableColumn::new("up_to_date", "Up-To-Date")
            .fixed_width(px(100.0))
            .right_numeric(),
        TableColumn::new("available", "Available")
            .fixed_width(px(95.0))
            .right_numeric(),
        TableColumn::new("strategy", "Strategy").fixed_width(px(120.0)),
        TableColumn::new("images", "Images")
            .flex_width(1.8, px(160.0))
            .monospace(true),
        TableColumn::new("age", "Age")
            .fixed_width(px(76.0))
            .right_numeric(),
    ]
}

pub fn format_short_image(image: &str) -> &str {
    image.rsplit('/').next().unwrap_or(image)
}

pub fn format_images_summary(images: &[String]) -> (String, usize) {
    let Some(first) = images.first() else {
        return ("-".to_string(), 0);
    };
    let short = format_short_image(first).to_string();
    let extra = images.len().saturating_sub(1);
    (short, extra)
}

pub fn parse_age_seconds(age: &str) -> u64 {
    let trimmed = age.trim();
    if trimmed.is_empty() || trimmed.eq_ignore_ascii_case("unknown") {
        return u64::MAX;
    }

    let (num_str, multiplier) = if let Some(n) = trimmed.strip_suffix('s') {
        (n, 1_u64)
    } else if let Some(n) = trimmed.strip_suffix('m') {
        (n, 60_u64)
    } else if let Some(n) = trimmed.strip_suffix('h') {
        (n, 3_600_u64)
    } else if let Some(n) = trimmed.strip_suffix('d') {
        (n, 86_400_u64)
    } else {
        return u64::MAX;
    };

    num_str
        .trim()
        .parse::<u64>()
        .map(|v| v.saturating_mul(multiplier))
        .unwrap_or(u64::MAX)
}

fn compare_deployments_by_col(a: &DeploymentInfo, b: &DeploymentInfo, col_id: &str) -> Ordering {
    let tie_break = || {
        a.name
            .to_ascii_lowercase()
            .cmp(&b.name.to_ascii_lowercase())
            .then_with(|| {
                a.namespace
                    .to_ascii_lowercase()
                    .cmp(&b.namespace.to_ascii_lowercase())
            })
    };

    match col_id {
        "name" => a
            .name
            .to_ascii_lowercase()
            .cmp(&b.name.to_ascii_lowercase())
            .then_with(|| {
                a.namespace
                    .to_ascii_lowercase()
                    .cmp(&b.namespace.to_ascii_lowercase())
            }),
        "namespace" => a
            .namespace
            .to_ascii_lowercase()
            .cmp(&b.namespace.to_ascii_lowercase())
            .then_with(|| {
                a.name
                    .to_ascii_lowercase()
                    .cmp(&b.name.to_ascii_lowercase())
            }),
        "status" => a
            .status
            .to_ascii_lowercase()
            .cmp(&b.status.to_ascii_lowercase())
            .then_with(tie_break),
        "replicas" => (a.replicas.desired, a.replicas.current)
            .cmp(&(b.replicas.desired, b.replicas.current))
            .then_with(tie_break),
        "up_to_date" => a.up_to_date.cmp(&b.up_to_date).then_with(tie_break),
        "available" => a.available.cmp(&b.available).then_with(tie_break),
        "strategy" => a
            .strategy
            .as_deref()
            .unwrap_or("-")
            .to_ascii_lowercase()
            .cmp(&b.strategy.as_deref().unwrap_or("-").to_ascii_lowercase())
            .then_with(tie_break),
        "images" => {
            let a_imgs = a.images.join(",");
            let b_imgs = b.images.join(",");
            a_imgs
                .to_ascii_lowercase()
                .cmp(&b_imgs.to_ascii_lowercase())
                .then_with(tie_break)
        }
        "age" => parse_age_seconds(&a.age)
            .cmp(&parse_age_seconds(&b.age))
            .then_with(tie_break),
        _ => tie_break(),
    }
}

pub fn filter_and_sort_deployments<'a>(
    deployments: &'a [DeploymentInfo],
    namespaces: &NamespaceFilterState,
    sort: Option<&SortState>,
) -> Vec<&'a DeploymentInfo> {
    let mut rows: Vec<&'a DeploymentInfo> = deployments
        .iter()
        .filter(|dep| namespaces.matches_resource_namespace(Some(&dep.namespace)))
        .collect();
    sort_rows_stable(&mut rows, sort, |a, b, col_id| {
        compare_deployments_by_col(a, b, col_id)
    });
    rows
}

pub fn deployments_empty_message(state: &DeploymentsState) -> &'static str {
    if state.deployments.is_empty() {
        "No deployments found in this cluster."
    } else {
        "No deployments found in the selected namespace(s)."
    }
}

pub fn deployments_presentation_mode(state: &DeploymentsState) -> DeploymentsPresentationMode {
    if state.status == DeploymentsLoadStatus::Error && state.deployments.is_empty() {
        DeploymentsPresentationMode::ErrorCard
    } else {
        DeploymentsPresentationMode::Table
    }
}

fn render_deployment_cell(
    dep: &DeploymentInfo,
    col: &TableColumn,
    theme: &OrbitTheme,
) -> AnyElement {
    match col.id.as_ref() {
        "name" => {
            let meta = resource_kind_meta(K8sResourceKind::Deployment, theme);
            div()
                .flex()
                .items_center()
                .gap_2()
                .min_w(px(0.0))
                .child(render_icon(meta.icon, px(14.0), meta.color))
                .child(
                    apply_text_step(div(), theme.typography.size_xs)
                        .font_weight(FontWeight::MEDIUM)
                        .text_color(theme.text.primary)
                        .truncate()
                        .child(dep.name.clone()),
                )
                .into_any_element()
        }
        "namespace" => apply_text_step(div(), theme.typography.size_xs)
            .text_color(theme.text.secondary)
            .truncate()
            .child(dep.namespace.clone())
            .into_any_element(),
        "status" => render_status_badge_from_str(&dep.status, StatusBadgeVariant::Inline, theme),
        "replicas" => div()
            .flex()
            .items_center()
            .justify_end()
            .child(
                apply_text_step(div(), theme.typography.size_xs)
                    .font(theme.typography.mono_font(FontWeight::SEMIBOLD))
                    .text_color(theme.text.primary)
                    .child(dep.replicas.current.to_string()),
            )
            .child(
                apply_text_step(div(), theme.typography.size_xs)
                    .font(theme.typography.mono_font(FontWeight::NORMAL))
                    .text_color(theme.text.muted)
                    .child(format!("/{}", dep.replicas.desired)),
            )
            .into_any_element(),
        "up_to_date" => apply_text_step(div(), theme.typography.size_xs)
            .font(theme.typography.mono_font(FontWeight::NORMAL))
            .text_color(theme.text.primary)
            .child(dep.up_to_date.to_string())
            .into_any_element(),
        "available" => {
            let color = if dep.available == dep.replicas.desired {
                theme.semantic.success.solid
            } else {
                theme.semantic.warning.solid
            };
            apply_text_step(div(), theme.typography.size_xs)
                .font(theme.typography.mono_font(FontWeight::MEDIUM))
                .text_color(color)
                .child(dep.available.to_string())
                .into_any_element()
        }
        "strategy" => apply_text_step(div(), theme.typography.size_xs)
            .text_color(theme.text.secondary)
            .truncate()
            .child(dep.strategy.as_deref().unwrap_or("-").to_string())
            .into_any_element(),
        "images" => {
            let (primary, extra) = format_images_summary(&dep.images);
            div()
                .flex()
                .items_center()
                .gap_1p5()
                .min_w(px(0.0))
                .child(
                    apply_text_step(div(), theme.typography.size_xs)
                        .font(theme.typography.mono_font(FontWeight::NORMAL))
                        .text_color(theme.text.secondary)
                        .truncate()
                        .child(primary),
                )
                .when(extra > 0, |el| {
                    el.child(
                        apply_text_step(div(), theme.typography.size_xs)
                            .px_1p5()
                            .py_0p5()
                            .flex_shrink_0()
                            .rounded(theme.style.radius_control)
                            .bg(theme.surface.bg_hover)
                            .font(theme.typography.mono_font(FontWeight::MEDIUM))
                            .text_color(theme.text.muted)
                            .child(format!("+{extra}")),
                    )
                })
                .into_any_element()
        }
        "age" => apply_text_step(div(), theme.typography.size_xs)
            .font(theme.typography.mono_font(FontWeight::NORMAL))
            .text_color(theme.text.muted)
            .child(dep.age.clone())
            .into_any_element(),
        _ => div().into_any_element(),
    }
}

fn render_action_button(
    id: &'static str,
    icon: Icon,
    label: &'static str,
    disabled: bool,
    theme: &OrbitTheme,
    on_click: Rc<dyn Fn(&mut Window, &mut App)>,
) -> AnyElement {
    let hover_bg = theme.surface.control_hover;
    div()
        .id(id)
        .h(px(28.0))
        .px_2p5()
        .rounded(theme.style.radius_control)
        .border_1()
        .border_color(theme.border.default)
        .bg(theme.surface.bg_card)
        .flex()
        .items_center()
        .gap_1p5()
        .when(disabled, |el| el.opacity(0.6))
        .when(!disabled, |el| {
            el.cursor_pointer()
                .hover(move |style| style.bg(hover_bg))
                .on_click(move |_, window, cx| on_click(window, cx))
        })
        .child(render_icon(icon, px(13.0), theme.text.secondary))
        .child(
            apply_text_step(div(), theme.typography.size_xs)
                .font_weight(FontWeight::MEDIUM)
                .text_color(theme.text.primary)
                .child(label),
        )
        .into_any_element()
}

#[allow(clippy::too_many_arguments)]
pub fn render_deployments_view(
    state: &DeploymentsState,
    namespaces: &NamespaceFilterState,
    theme: &OrbitTheme,
    on_sort: impl 'static + Fn(SharedString, &ClickEvent, &mut Window, &mut App),
    on_row_click: impl 'static + Fn(usize, SharedString, &ClickEvent, &mut Window, &mut App),
    on_refresh: impl 'static + Fn(&mut Window, &mut App),
) -> AnyElement {
    let columns = deployment_columns();
    let filtered_rows: Rc<Vec<DeploymentInfo>> = Rc::new(
        filter_and_sort_deployments(&state.deployments, namespaces, state.table.sort.as_ref())
            .into_iter()
            .cloned()
            .collect(),
    );
    let total_count = state.deployments.len();
    let visible_count = filtered_rows.len();
    let is_loading = state.status == DeploymentsLoadStatus::Loading;
    let on_refresh_rc: Rc<dyn Fn(&mut Window, &mut App)> = Rc::new(on_refresh);

    let count_label = if namespaces.is_all_namespaces() {
        format!("{total_count} deployments")
    } else {
        format!("{visible_count} of {total_count} deployments")
    };

    let toolbar = div()
        .flex()
        .items_center()
        .justify_between()
        .gap_3()
        .flex_shrink_0()
        .child(
            div()
                .flex()
                .items_center()
                .gap_2()
                .child(
                    apply_text_step(div(), theme.typography.size_xs)
                        .font_weight(FontWeight::MEDIUM)
                        .text_color(theme.text.secondary)
                        .child(count_label),
                )
                .when(!namespaces.is_all_namespaces(), |el| {
                    el.child(
                        apply_text_step(div(), theme.typography.size_xs)
                            .px_2()
                            .py_0p5()
                            .rounded(theme.style.radius_control)
                            .bg(theme.accent.soft)
                            .text_color(theme.accent.accent)
                            .font_weight(FontWeight::MEDIUM)
                            .child(format!("Namespace: {}", namespaces.summary_label())),
                    )
                }),
        )
        .child(render_action_button(
            "deployments-refresh-btn",
            Icon::RefreshCw,
            if is_loading { "Refreshing..." } else { "Refresh" },
            is_loading,
            theme,
            on_refresh_rc.clone(),
        ));

    let error_banner = state.error_message.as_ref().map(|message| {
        div()
            .flex_shrink_0()
            .px_3()
            .py_2()
            .rounded(theme.style.radius_control)
            .border_1()
            .border_color(theme.semantic.danger.solid)
            .bg(theme.semantic.danger.soft)
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
                    .child(render_icon(
                        Icon::AlertTriangle,
                        px(14.0),
                        theme.semantic.danger.solid,
                    ))
                    .child(
                        apply_text_step(div(), theme.typography.size_xs)
                            .text_color(theme.text.primary)
                            .truncate()
                            .child(format!("Failed to load deployments: {message}")),
                    ),
            )
            .child(render_action_button(
                "deployments-retry-btn",
                Icon::RotateCcw,
                "Retry",
                is_loading,
                theme,
                on_refresh_rc.clone(),
            ))
    });

    let body = match deployments_presentation_mode(state) {
        DeploymentsPresentationMode::ErrorCard => {
            let message = state
                .error_message
                .clone()
                .unwrap_or_else(|| "Unknown error".to_string());
            theme
                .style_card(div())
                .flex()
                .flex_col()
                .items_start()
                .gap_3()
                .p_6()
                .bg(theme.surface.bg_panel)
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
                                .child("Unable to Load Deployments"),
                        ),
                )
                .child(
                    apply_text_step(div(), theme.typography.size_xs)
                        .text_color(theme.text.secondary)
                        .child(message),
                )
                .child(render_action_button(
                    "deployments-retry-btn",
                    Icon::RotateCcw,
                    "Retry",
                    is_loading,
                    theme,
                    on_refresh_rc,
                ))
                .into_any_element()
        }
        DeploymentsPresentationMode::Table => {
            let mut table_state = state.table.clone();
            table_state.loading = is_loading && state.deployments.is_empty();
            table_state.empty_message = SharedString::from(deployments_empty_message(state));

            let key_rows = filtered_rows.clone();
            let cell_rows = filtered_rows.clone();
            let callbacks = DataTableCallbacks::new(
                move |idx| {
                    key_rows
                        .get(idx)
                        .map(|d| format!("{}/{}", d.namespace, d.name))
                        .unwrap_or_default()
                },
                move |idx, col, col_theme| {
                    cell_rows
                        .get(idx)
                        .map(|d| render_deployment_cell(d, col, col_theme))
                        .unwrap_or_else(|| div().into_any_element())
                },
            )
            .on_sort(on_sort)
            .on_row_click(on_row_click);

            render_data_table(
                "deployments-table",
                &columns,
                visible_count,
                &table_state,
                theme,
                callbacks,
            )
        }
    };

    div()
        .flex()
        .flex_col()
        .flex_1()
        .min_h(px(0.0))
        .gap_4()
        .child(toolbar)
        .when(
            deployments_presentation_mode(state) == DeploymentsPresentationMode::Table,
            |el| el.when_some(error_banner, |el, banner| el.child(banner)),
        )
        .child(body)
        .into_any_element()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::components::data_table::{ColumnAlign, SortDirection};
    use orbit_core::kubernetes::models::{NamespaceInfo, Replicas};

    fn make_deployment(
        name: &str,
        namespace: &str,
        status: &str,
        current: i32,
        desired: i32,
        up_to_date: i32,
        available: i32,
        strategy: Option<&str>,
        images: Vec<&str>,
        age: &str,
    ) -> DeploymentInfo {
        DeploymentInfo {
            kind: "Deployment".to_string(),
            name: name.to_string(),
            namespace: namespace.to_string(),
            status: status.to_string(),
            conditions: None,
            replicas: Replicas { current, desired },
            available,
            up_to_date,
            age: age.to_string(),
            images: images.into_iter().map(ToString::to_string).collect(),
            containers: Vec::new(),
            strategy: strategy.map(ToString::to_string),
            min_ready_seconds: 0,
            revision_history: Some(10),
            labels: Default::default(),
            annotations: Default::default(),
            ports: Vec::new(),
        }
    }

    fn make_ns(name: &str) -> NamespaceInfo {
        NamespaceInfo {
            name: name.to_string(),
            status: "Active".to_string(),
            is_system: name == "kube-system" || name == "default",
            age: "10d".to_string(),
            labels: Default::default(),
            annotations: Default::default(),
            uid: format!("uid-{name}"),
            created: String::new(),
        }
    }

    #[test]
    fn test_deployment_columns_cover_required_fields_and_numeric_alignment() {
        let cols = deployment_columns();
        let ids: Vec<&str> = cols.iter().map(|c| c.id.as_ref()).collect();
        assert_eq!(
            ids,
            vec![
                "name",
                "namespace",
                "status",
                "replicas",
                "up_to_date",
                "available",
                "strategy",
                "images",
                "age",
            ]
        );

        for numeric_id in ["replicas", "up_to_date", "available", "age"] {
            let col = cols
                .iter()
                .find(|c| c.id.as_ref() == numeric_id)
                .expect("numeric column present");
            assert_eq!(col.align, ColumnAlign::Right, "{numeric_id} right-aligned");
            assert!(col.monospace, "{numeric_id} monospace");
        }
    }

    #[test]
    fn test_filter_and_sort_deployments_respects_namespace_filter_and_sort_keys() {
        let rows = vec![
            make_deployment(
                "zeta",
                "payments",
                "Running",
                2,
                2,
                2,
                2,
                Some("RollingUpdate"),
                vec!["ghcr.io/org/zeta:1.0"],
                "5d",
            ),
            make_deployment(
                "alpha",
                "default",
                "Progressing",
                1,
                3,
                1,
                1,
                Some("Recreate"),
                vec!["nginx:1.25"],
                "45s",
            ),
            make_deployment(
                "coredns",
                "kube-system",
                "Failed",
                0,
                2,
                0,
                0,
                None,
                vec!["registry.k8s.io/coredns/coredns:v1.11.1"],
                "Unknown",
            ),
        ];

        let mut ns_state = NamespaceFilterState::default();
        ns_state.begin_load_for_cluster("ctx-a");
        ns_state.apply_loaded(
            "ctx-a",
            1,
            vec![
                make_ns("default"),
                make_ns("kube-system"),
                make_ns("payments"),
            ],
        );

        // All namespaces
        let all_by_name = filter_and_sort_deployments(
            &rows,
            &ns_state,
            Some(&SortState {
                column_id: "name".to_string(),
                direction: SortDirection::Asc,
            }),
        );
        let names: Vec<&str> = all_by_name.iter().map(|d| d.name.as_str()).collect();
        assert_eq!(names, vec!["alpha", "coredns", "zeta"]);

        // Age descending: Unknown (u64::MAX), 5d, 45s
        let by_age_desc = filter_and_sort_deployments(
            &rows,
            &ns_state,
            Some(&SortState {
                column_id: "age".to_string(),
                direction: SortDirection::Desc,
            }),
        );
        let age_names: Vec<&str> = by_age_desc.iter().map(|d| d.name.as_str()).collect();
        assert_eq!(age_names, vec!["coredns", "zeta", "alpha"]);

        // Replicas ascending: (2, 0), (2, 2), (3, 1)
        let by_replicas = filter_and_sort_deployments(
            &rows,
            &ns_state,
            Some(&SortState {
                column_id: "replicas".to_string(),
                direction: SortDirection::Asc,
            }),
        );
        let rep_names: Vec<&str> = by_replicas.iter().map(|d| d.name.as_str()).collect();
        assert_eq!(rep_names, vec!["coredns", "zeta", "alpha"]);

        // Filter to payments + default
        ns_state.toggle_namespace("payments");
        ns_state.toggle_namespace("default");
        let filtered = filter_and_sort_deployments(&rows, &ns_state, None);
        assert_eq!(filtered.len(), 2);
        assert!(filtered.iter().all(|d| d.namespace != "kube-system"));
    }

    #[test]
    fn test_format_short_image_and_parse_age_seconds() {
        assert_eq!(format_short_image("ghcr.io/org/api:v1.2.3"), "api:v1.2.3");
        assert_eq!(format_short_image("nginx:latest"), "nginx:latest");
        assert_eq!(
            format_images_summary(&["ghcr.io/org/api:v1.2.3".to_string()]),
            ("api:v1.2.3".to_string(), 0)
        );
        assert_eq!(
            format_images_summary(&[
                "ghcr.io/org/api:v1.2.3".to_string(),
                "busybox:1.36".to_string(),
                "envoy:v1".to_string(),
            ]),
            ("api:v1.2.3".to_string(), 2)
        );
        assert_eq!(format_images_summary(&[]), ("-".to_string(), 0));

        assert_eq!(parse_age_seconds("45s"), 45);
        assert_eq!(parse_age_seconds("12m"), 720);
        assert_eq!(parse_age_seconds("3h"), 10_800);
        assert_eq!(parse_age_seconds("5d"), 432_000);
        assert_eq!(parse_age_seconds("Unknown"), u64::MAX);
    }

    #[test]
    fn test_deployments_view_empty_and_error_presentation_modes() {
        let mut state = DeploymentsState::default();
        assert_eq!(
            deployments_presentation_mode(&state),
            DeploymentsPresentationMode::Table
        );
        assert_eq!(
            deployments_empty_message(&state),
            "No deployments found in this cluster."
        );

        state.begin_load_for_cluster("ctx-a");
        state.apply_loaded(
            "ctx-a",
            1,
            vec![make_deployment(
                "api",
                "default",
                "Running",
                1,
                1,
                1,
                1,
                None,
                vec![],
                "1m",
            )],
        );
        assert_eq!(
            deployments_empty_message(&state),
            "No deployments found in the selected namespace(s)."
        );

        state.begin_refresh();
        state.apply_failed("ctx-a", 2, "Forbidden".to_string());
        // With cached rows, keep showing table + error banner.
        assert_eq!(
            deployments_presentation_mode(&state),
            DeploymentsPresentationMode::Table
        );

        // When empty and in error, show dedicated error card.
        state.deployments.clear();
        assert_eq!(
            deployments_presentation_mode(&state),
            DeploymentsPresentationMode::ErrorCard
        );
    }
}
