use std::cmp::Ordering;
use std::rc::Rc;

use gpui::{
    AnyElement, App, ClickEvent, ElementId, FontWeight, MouseDownEvent, SharedString, Window, div,
    prelude::*, px,
};
use lucide_icons::Icon;
use orbit_core::kubernetes::models::DeploymentInfo;

use crate::components::data_table::{
    DataTableCallbacks, SortState, TableColumn, render_data_table, sort_rows_stable,
};
use crate::components::{
    ContextMenuEntry, ContextMenuItem, StatusBadgeVariant, render_status_badge_from_str,
};
use crate::navigation::WorkspaceView;
use crate::state::{DeploymentsLoadStatus, DeploymentsState, NamespaceFilterState};
use crate::theme::{OrbitTheme, apply_text_step};
pub use crate::views::{
    format_images_summary, format_short_image, numeric_metric_text_color, parse_age_seconds,
    render_action_button, render_images_cell, render_ratio_metric_cell,
    render_resource_error_banner, render_resource_error_state, render_resource_view_header,
    render_truncated_cell, table_numeric_font,
};

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
    row_idx: usize,
    dep: &DeploymentInfo,
    col: &TableColumn,
    theme: &OrbitTheme,
) -> AnyElement {
    match col.id.as_ref() {
        "name" => render_truncated_cell(
            ElementId::NamedInteger(SharedString::from("dep-name"), row_idx as u64),
            dep.name.clone(),
            Some(SharedString::from(dep.name.clone())),
            theme.text.primary,
            FontWeight::MEDIUM,
            false,
            theme,
        ),
        "namespace" => render_truncated_cell(
            ElementId::NamedInteger(SharedString::from("dep-ns"), row_idx as u64),
            dep.namespace.clone(),
            None,
            theme.text.secondary,
            FontWeight::NORMAL,
            false,
            theme,
        ),
        "status" => render_status_badge_from_str(&dep.status, StatusBadgeVariant::Inline, theme),
        "replicas" => render_ratio_metric_cell(
            dep.replicas.current.to_string(),
            dep.replicas.desired.to_string(),
            theme,
        ),
        "up_to_date" => apply_text_step(div(), theme.typography.size_xs)
            .font(table_numeric_font(theme))
            .text_color(numeric_metric_text_color(theme))
            .child(dep.up_to_date.to_string())
            .into_any_element(),
        "available" => apply_text_step(div(), theme.typography.size_xs)
            .font(table_numeric_font(theme))
            .text_color(numeric_metric_text_color(theme))
            .child(dep.available.to_string())
            .into_any_element(),
        "strategy" => render_truncated_cell(
            ElementId::NamedInteger(SharedString::from("dep-strat"), row_idx as u64),
            dep.strategy.as_deref().unwrap_or("-").to_string(),
            None,
            theme.text.secondary,
            FontWeight::NORMAL,
            false,
            theme,
        ),
        "images" => render_images_cell(
            ElementId::NamedInteger(SharedString::from("dep-img"), row_idx as u64),
            &dep.images,
            theme,
        ),
        "age" => apply_text_step(div(), theme.typography.size_xs)
            .font(theme.typography.mono_font(FontWeight::NORMAL))
            .text_color(theme.text.muted)
            .child(dep.age.clone())
            .into_any_element(),
        _ => div().into_any_element(),
    }
}

#[allow(clippy::too_many_arguments)]
pub fn build_deployment_context_menu_entries(
    deployment: &DeploymentInfo,
    on_copy_name: impl 'static + Fn(&mut Window, &mut App),
    on_filter_namespace: impl 'static + Fn(&mut Window, &mut App),
    on_refresh: impl 'static + Fn(&mut Window, &mut App),
    on_view_details: impl 'static + Fn(&mut Window, &mut App),
    on_view_yaml: impl 'static + Fn(&mut Window, &mut App),
    on_scale: impl 'static + Fn(&mut Window, &mut App),
    on_restart: impl 'static + Fn(&mut Window, &mut App),
    on_delete: impl 'static + Fn(&mut Window, &mut App),
) -> Vec<ContextMenuEntry> {
    vec![
        ContextMenuEntry::Item(
            ContextMenuItem::new("copy-name", "Copy Name")
                .icon(Icon::Copy)
                .on_select(on_copy_name),
        ),
        ContextMenuEntry::Item(
            ContextMenuItem::new(
                "filter-namespace",
                format!("Filter by \"{}\"", deployment.namespace),
            )
            .icon(Icon::Filter)
            .on_select(on_filter_namespace),
        ),
        ContextMenuEntry::Item(
            ContextMenuItem::new("refresh", "Refresh")
                .icon(Icon::RefreshCw)
                .on_select(on_refresh),
        ),
        ContextMenuEntry::Separator,
        ContextMenuEntry::Item(
            ContextMenuItem::new("view-details", "View Details")
                .icon(Icon::Info)
                .on_select(on_view_details),
        ),
        ContextMenuEntry::Item(
            ContextMenuItem::new("view-yaml", "View YAML")
                .icon(Icon::FileCode)
                .on_select(on_view_yaml),
        ),
        ContextMenuEntry::Separator,
        ContextMenuEntry::Item(
            ContextMenuItem::new("scale", "Scale...")
                .icon(Icon::SlidersHorizontal)
                .on_select(on_scale),
        ),
        ContextMenuEntry::Item(
            ContextMenuItem::new("restart", "Restart")
                .icon(Icon::RotateCw)
                .on_select(on_restart),
        ),
        ContextMenuEntry::Separator,
        ContextMenuEntry::Item(
            ContextMenuItem::new("delete-deployment", "Delete Deployment")
                .icon(Icon::Trash2)
                .danger(true)
                .on_select(on_delete),
        ),
    ]
}

#[allow(clippy::too_many_arguments)]
pub fn render_deployments_view(
    state: &DeploymentsState,
    namespaces: &NamespaceFilterState,
    theme: &OrbitTheme,
    on_sort: impl 'static + Fn(SharedString, &ClickEvent, &mut Window, &mut App),
    on_row_click: impl 'static + Fn(usize, SharedString, &ClickEvent, &mut Window, &mut App),
    on_row_quick_yaml: impl 'static + Fn(usize, SharedString, &ClickEvent, &mut Window, &mut App),
    on_row_context_menu: impl 'static
    + Fn(SharedString, &DeploymentInfo, &MouseDownEvent, &mut Window, &mut App),
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
    let on_refresh_rc: super::ActionCallback = Rc::new(on_refresh);

    let count_label = if namespaces.is_all_namespaces() {
        format!("{total_count} deployments")
    } else {
        format!("{visible_count} of {total_count} deployments")
    };

    let header = render_resource_view_header(
        WorkspaceView::Deployments.label(),
        WorkspaceView::Deployments.description(),
        count_label,
        Some(namespaces),
        None,
        theme,
    );

    let error_banner = state.error_message.as_ref().map(|message| {
        render_resource_error_banner(
            "deployments-retry-btn",
            format!("Failed to load deployments: {message}"),
            is_loading,
            theme,
            on_refresh_rc.clone(),
        )
    });

    let body = match deployments_presentation_mode(state) {
        DeploymentsPresentationMode::ErrorCard => {
            let message = state
                .error_message
                .clone()
                .unwrap_or_else(|| "Unknown error".to_string());
            render_resource_error_state(
                "deployments-retry-btn",
                "Unable to Load Deployments",
                message,
                is_loading,
                theme,
                on_refresh_rc,
            )
        }
        DeploymentsPresentationMode::Table => {
            let mut table_state = state.table.clone();
            table_state.loading = is_loading && state.deployments.is_empty();
            table_state.empty_message = SharedString::from(deployments_empty_message(state));

            let key_rows = filtered_rows.clone();
            let cell_rows = filtered_rows.clone();
            let ctx_rows = filtered_rows.clone();
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
                        .map(|d| render_deployment_cell(idx, d, col, col_theme))
                        .unwrap_or_else(|| div().into_any_element())
                },
            )
            .on_sort(on_sort)
            .on_row_click(on_row_click)
            .on_row_quick_yaml(on_row_quick_yaml)
            .on_row_context_menu(move |idx, row_key, event, window, cx| {
                if let Some(deployment) = ctx_rows.get(idx) {
                    on_row_context_menu(row_key, deployment, event, window, cx);
                }
            });

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
        .gap_0()
        .child(header)
        .when(
            deployments_presentation_mode(state) == DeploymentsPresentationMode::Table,
            |el| el.when_some(error_banner, |el, banner| el.child(banner)),
        )
        .child(body)
        .into_any_element()
}

pub fn filter_events_for_deployment<'a>(
    events: &'a [orbit_core::kubernetes::models::EventInfo],
    deployment: &DeploymentInfo,
) -> Vec<&'a orbit_core::kubernetes::models::EventInfo> {
    let prefix = format!("{}-", deployment.name);
    events
        .iter()
        .filter(|ev| {
            ev.namespace == deployment.namespace
                && ((ev.object_kind == "Deployment" && ev.object_name == deployment.name)
                    || ((ev.object_kind == "ReplicaSet" || ev.object_kind == "Pod")
                        && ev.object_name.starts_with(&prefix))
                    || ev.message.contains(&deployment.name))
        })
        .collect()
}

pub fn render_deployment_drawer_overview(
    deployment: &DeploymentInfo,
    theme: &OrbitTheme,
) -> AnyElement {
    let summary = crate::views::render_drawer_key_values(
        vec![
            ("Name", deployment.name.clone(), true),
            ("Namespace", deployment.namespace.clone(), true),
            ("Status", deployment.status.clone(), false),
            (
                "Ready Replicas",
                format!(
                    "{}/{}",
                    deployment.replicas.current, deployment.replicas.desired
                ),
                true,
            ),
            ("Available", deployment.available.to_string(), true),
            ("Up-to-date", deployment.up_to_date.to_string(), true),
            (
                "Strategy",
                deployment
                    .strategy
                    .clone()
                    .unwrap_or_else(|| "RollingUpdate".to_string()),
                false,
            ),
            (
                "Min Ready Seconds",
                deployment.min_ready_seconds.to_string(),
                true,
            ),
            (
                "Revision History",
                deployment
                    .revision_history
                    .map(|v| v.to_string())
                    .unwrap_or_else(|| "-".to_string()),
                true,
            ),
            ("Age", deployment.age.clone(), true),
        ],
        theme,
    );

    let mono = theme.typography.mono_font(FontWeight::NORMAL);
    let containers_section = {
        let content = if deployment.containers.is_empty() && deployment.images.is_empty() {
            apply_text_step(div(), theme.typography.size_xs)
                .text_color(theme.text.muted)
                .child("No containers reported.")
                .into_any_element()
        } else if !deployment.containers.is_empty() {
            div()
                .w_full()
                .flex()
                .flex_col()
                .gap_2()
                .children(deployment.containers.iter().map(|c| {
                    div()
                        .w_full()
                        .p_2p5()
                        .rounded(theme.style.radius_control)
                        .bg(theme.surface.bg_panel)
                        .border_1()
                        .border_color(theme.border.default)
                        .flex()
                        .flex_col()
                        .gap_1()
                        .child(
                            apply_text_step(div(), theme.typography.size_xs)
                                .font_weight(FontWeight::SEMIBOLD)
                                .text_color(theme.text.primary)
                                .child(c.name.clone()),
                        )
                        .child(
                            apply_text_step(div(), theme.typography.size_2xs)
                                .font(mono.clone())
                                .text_color(theme.text.secondary)
                                .child(c.image.clone()),
                        )
                }))
                .into_any_element()
        } else {
            div()
                .w_full()
                .flex()
                .flex_col()
                .gap_1p5()
                .children(deployment.images.iter().map(|img| {
                    apply_text_step(div(), theme.typography.size_2xs)
                        .px_2p5()
                        .py_1p5()
                        .rounded(theme.style.radius_control)
                        .bg(theme.surface.bg_panel)
                        .font(mono.clone())
                        .text_color(theme.text.primary)
                        .child(img.clone())
                }))
                .into_any_element()
        };
        crate::views::render_drawer_section("CONTAINERS & IMAGES", content, theme)
    };

    let ports_section = if deployment.ports.is_empty() {
        None
    } else {
        Some(crate::views::render_drawer_section(
            "EXPOSED PORTS",
            div()
                .flex()
                .flex_wrap()
                .gap_1p5()
                .children(deployment.ports.iter().map(|port| {
                    apply_text_step(div(), theme.typography.size_2xs)
                        .px_2()
                        .py_0p5()
                        .rounded(theme.style.radius_control)
                        .bg(theme.surface.control_bg)
                        .border_1()
                        .border_color(theme.border.default)
                        .font(mono.clone())
                        .text_color(theme.text.primary)
                        .child(port.to_string())
                }))
                .into_any_element(),
            theme,
        ))
    };

    let conditions_section = deployment
        .conditions
        .as_ref()
        .filter(|c| !c.is_empty())
        .map(|conds| {
            crate::views::render_drawer_section(
                "CONDITIONS",
                div()
                    .w_full()
                    .flex()
                    .flex_col()
                    .gap_2()
                    .children(conds.iter().map(|cond| {
                        div()
                            .w_full()
                            .p_2p5()
                            .rounded(theme.style.radius_control)
                            .bg(theme.surface.bg_panel)
                            .border_1()
                            .border_color(theme.border.default)
                            .flex()
                            .flex_col()
                            .gap_1()
                            .child(
                                div()
                                    .flex()
                                    .items_center()
                                    .justify_between()
                                    .gap_2()
                                    .child(
                                        apply_text_step(div(), theme.typography.size_xs)
                                            .font_weight(FontWeight::SEMIBOLD)
                                            .text_color(theme.text.primary)
                                            .child(cond.type_.clone()),
                                    )
                                    .child(
                                        apply_text_step(div(), theme.typography.size_2xs)
                                            .text_color(theme.text.secondary)
                                            .child(cond.status.clone()),
                                    ),
                            )
                            .when_some(cond.message.as_ref(), |el, msg| {
                                el.child(
                                    apply_text_step(div(), theme.typography.size_2xs)
                                        .text_color(theme.text.muted)
                                        .child(msg.clone()),
                                )
                            })
                    }))
                    .into_any_element(),
                theme,
            )
        });

    div()
        .id("deployment-drawer-overview")
        .flex_1()
        .min_h_0()
        .overflow_y_scroll()
        .p_4()
        .flex()
        .flex_col()
        .gap_4()
        .child(crate::views::render_drawer_section(
            "SUMMARY", summary, theme,
        ))
        .child(containers_section)
        .when_some(ports_section, |el, sec| el.child(sec))
        .when_some(conditions_section, |el, sec| el.child(sec))
        .child(crate::views::render_drawer_kv_badges(
            "LABELS",
            &deployment.labels,
            "No labels",
            theme,
        ))
        .child(crate::views::render_drawer_kv_badges(
            "ANNOTATIONS",
            &deployment.annotations,
            "No annotations",
            theme,
        ))
        .into_any_element()
}

pub fn render_deployment_drawer_events(
    events: &[orbit_core::kubernetes::models::EventInfo],
    loading: bool,
    deployment: &DeploymentInfo,
    theme: &OrbitTheme,
) -> AnyElement {
    let matched = filter_events_for_deployment(events, deployment);
    div()
        .id("deployment-drawer-events")
        .flex_1()
        .min_h_0()
        .overflow_y_scroll()
        .p_4()
        .child(crate::views::render_drawer_events_list(
            &matched,
            loading,
            "No events found for this deployment.",
            theme,
        ))
        .into_any_element()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::components::data_table::{ColumnAlign, SortDirection};
    use orbit_core::kubernetes::models::{NamespaceInfo, Replicas};

    #[allow(clippy::too_many_arguments)]
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
        for text_id in ["name", "namespace", "status", "strategy", "images"] {
            let col = cols
                .iter()
                .find(|c| c.id.as_ref() == text_id)
                .expect("text column present");
            assert_eq!(col.align, ColumnAlign::Left, "{text_id} left-aligned");
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

    #[test]
    fn test_filter_events_for_deployment_matches_namespace_and_pod_prefix() {
        let deployment = make_deployment(
            "api",
            "default",
            "Healthy",
            2,
            2,
            2,
            2,
            Some("RollingUpdate"),
            vec!["nginx:1.25"],
            "1d",
        );
        let events = vec![
            orbit_core::kubernetes::models::EventInfo {
                uid: "1".to_string(),
                name: String::new(),
                time: "1m".to_string(),
                r#type: "Normal".to_string(),
                reason: "ScalingReplicaSet".to_string(),
                object_name: "api".to_string(),
                object_kind: "Deployment".to_string(),
                message: "Scaled up replica set api-7f9b".to_string(),
                namespace: "default".to_string(),
                source: "deployment-controller".to_string(),
                first_seen: String::new(),
                last_seen: String::new(),
                count: 1,
                labels: Default::default(),
                annotations: Default::default(),
            },
            orbit_core::kubernetes::models::EventInfo {
                uid: "2".to_string(),
                name: String::new(),
                time: "1m".to_string(),
                r#type: "Normal".to_string(),
                reason: "Started".to_string(),
                object_name: "api-7f9b-xyz".to_string(),
                object_kind: "Pod".to_string(),
                message: "Started container api".to_string(),
                namespace: "default".to_string(),
                source: "kubelet".to_string(),
                first_seen: String::new(),
                last_seen: String::new(),
                count: 1,
                labels: Default::default(),
                annotations: Default::default(),
            },
            orbit_core::kubernetes::models::EventInfo {
                uid: "3".to_string(),
                name: String::new(),
                time: "1m".to_string(),
                r#type: "Normal".to_string(),
                reason: "Started".to_string(),
                object_name: "api-7f9b-xyz".to_string(),
                object_kind: "Pod".to_string(),
                message: "Wrong namespace".to_string(),
                namespace: "other".to_string(),
                source: "kubelet".to_string(),
                first_seen: String::new(),
                last_seen: String::new(),
                count: 1,
                labels: Default::default(),
                annotations: Default::default(),
            },
        ];

        let filtered = filter_events_for_deployment(&events, &deployment);
        assert_eq!(filtered.len(), 2);
        assert_eq!(filtered[0].uid, "1");
        assert_eq!(filtered[1].uid, "2");
    }

    #[test]
    fn test_build_deployment_context_menu_entries_structure() {
        let dep = make_deployment(
            "web",
            "prod",
            "Running",
            3,
            3,
            3,
            3,
            Some("RollingUpdate"),
            vec!["nginx:1.25"],
            "1d",
        );
        let entries = build_deployment_context_menu_entries(
            &dep,
            |_, _| {},
            |_, _| {},
            |_, _| {},
            |_, _| {},
            |_, _| {},
            |_, _| {},
            |_, _| {},
            |_, _| {},
        );
        let ids: Vec<&str> = entries
            .iter()
            .filter_map(|e| match e {
                ContextMenuEntry::Item(item) => Some(item.id.as_ref()),
                ContextMenuEntry::Separator => None,
            })
            .collect();
        assert_eq!(
            ids,
            vec![
                "copy-name",
                "filter-namespace",
                "refresh",
                "view-details",
                "view-yaml",
                "scale",
                "restart",
                "delete-deployment",
            ]
        );
        let delete_item = entries
            .iter()
            .find_map(|e| match e {
                ContextMenuEntry::Item(item) if item.id.as_ref() == "delete-deployment" => Some(item),
                _ => None,
            })
            .expect("delete-deployment item");
        assert!(delete_item.danger);
        assert!(!delete_item.disabled);
        assert!(delete_item.on_select.is_some());
    }
}
