use std::cmp::Ordering;
use std::rc::Rc;

use gpui::{
    AnyElement, App, ClickEvent, ElementId, FontWeight, MouseDownEvent, SharedString, Window, div,
    prelude::*, px,
};
use lucide_icons::Icon;
use orbit_core::kubernetes::models::{EventInfo, PolicyInfo};

use crate::components::data_table::{
    DataTableCallbacks, SortState, TableColumn, render_data_table, sort_rows_stable,
};
use crate::components::{
    ContextMenuEntry, ContextMenuItem, StatusBadgeVariant, render_status_badge_from_str,
};
use crate::navigation::WorkspaceView;
use crate::state::{
    NamespaceFilterState, ResourceQuotaKindFilter, ResourceQuotasLoadStatus, ResourceQuotasState,
};
use crate::theme::{OrbitTheme, apply_text_step};
use crate::views::{
    parse_age_seconds, render_drawer_events_list, render_drawer_key_values, render_drawer_section,
    render_resource_error_banner, render_resource_error_state, render_resource_view_header,
    render_truncated_cell, table_numeric_font,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResourceQuotasPresentationMode {
    Table,
    ErrorCard,
}

pub fn resource_quota_columns() -> Vec<TableColumn> {
    vec![
        TableColumn::new("name", "Name").flex_width(1.5, px(150.0)),
        TableColumn::new("namespace", "Namespace").flex_width(1.0, px(105.0)),
        TableColumn::new("kind", "Kind").fixed_width(px(128.0)),
        TableColumn::new("status", "Status").fixed_width(px(104.0)),
        TableColumn::new("violations", "Violations")
            .fixed_width(px(88.0))
            .right_numeric(),
        TableColumn::new("rules_summary", "Rules Summary").flex_width(1.8, px(180.0)),
        TableColumn::new("age", "Age")
            .fixed_width(px(76.0))
            .right_numeric(),
    ]
}

pub fn is_resource_policy(policy: &PolicyInfo) -> bool {
    policy.kind.eq_ignore_ascii_case("ResourceQuota")
        || policy.kind.eq_ignore_ascii_case("LimitRange")
        || policy.r#type.eq_ignore_ascii_case("Resource Quota")
        || policy.r#type.eq_ignore_ascii_case("Limit Range")
}

fn matches_kind_filter(policy: &PolicyInfo, filter: ResourceQuotaKindFilter) -> bool {
    match filter {
        ResourceQuotaKindFilter::All => is_resource_policy(policy),
        ResourceQuotaKindFilter::ResourceQuota => {
            policy.kind.eq_ignore_ascii_case("ResourceQuota")
                || policy.r#type.eq_ignore_ascii_case("Resource Quota")
        }
        ResourceQuotaKindFilter::LimitRange => {
            policy.kind.eq_ignore_ascii_case("LimitRange")
                || policy.r#type.eq_ignore_ascii_case("Limit Range")
        }
    }
}

pub fn compare_resource_policies_by_col(a: &PolicyInfo, b: &PolicyInfo, col_id: &str) -> Ordering {
    let tie_break = || {
        a.name
            .to_ascii_lowercase()
            .cmp(&b.name.to_ascii_lowercase())
            .then_with(|| {
                a.namespace
                    .to_ascii_lowercase()
                    .cmp(&b.namespace.to_ascii_lowercase())
            })
            .then_with(|| {
                a.kind
                    .to_ascii_lowercase()
                    .cmp(&b.kind.to_ascii_lowercase())
            })
    };

    match col_id {
        "name" => tie_break(),
        "namespace" => a
            .namespace
            .to_ascii_lowercase()
            .cmp(&b.namespace.to_ascii_lowercase())
            .then_with(tie_break),
        "kind" => a
            .kind
            .to_ascii_lowercase()
            .cmp(&b.kind.to_ascii_lowercase())
            .then_with(tie_break),
        "status" => a
            .status
            .to_ascii_lowercase()
            .cmp(&b.status.to_ascii_lowercase())
            .then_with(tie_break),
        "violations" => a.violations.cmp(&b.violations).then_with(tie_break),
        "rules_summary" => a
            .description
            .to_ascii_lowercase()
            .cmp(&b.description.to_ascii_lowercase())
            .then_with(tie_break),
        "age" => parse_age_seconds(&a.last_updated)
            .cmp(&parse_age_seconds(&b.last_updated))
            .then_with(tie_break),
        _ => tie_break(),
    }
}

pub fn filter_and_sort_resource_policies<'a>(
    policies: &'a [PolicyInfo],
    namespaces: &NamespaceFilterState,
    kind_filter: ResourceQuotaKindFilter,
    sort: Option<&SortState>,
) -> Vec<&'a PolicyInfo> {
    let mut rows: Vec<&'a PolicyInfo> = policies
        .iter()
        .filter(|policy| {
            matches_kind_filter(policy, kind_filter)
                && namespaces.matches_resource_namespace(Some(&policy.namespace))
        })
        .collect();
    if sort.is_some() {
        sort_rows_stable(&mut rows, sort, |a, b, col_id| {
            compare_resource_policies_by_col(a, b, col_id)
        });
    } else {
        rows.sort_by(|a, b| compare_resource_policies_by_col(a, b, "name"));
    }
    rows
}

pub fn resource_quotas_empty_message(
    state: &ResourceQuotasState,
    namespaces: &NamespaceFilterState,
) -> &'static str {
    let has_any = state.policies.iter().any(is_resource_policy);
    if !has_any {
        return "No resource quotas or limit ranges found in this cluster.";
    }
    let has_kind = state
        .policies
        .iter()
        .any(|p| matches_kind_filter(p, state.kind_filter));
    if !has_kind {
        return match state.kind_filter {
            ResourceQuotaKindFilter::ResourceQuota => "No ResourceQuotas found in this cluster.",
            ResourceQuotaKindFilter::LimitRange => "No LimitRanges found in this cluster.",
            ResourceQuotaKindFilter::All => {
                "No resource quotas or limit ranges found in this cluster."
            }
        };
    }
    if !namespaces.is_all_namespaces() {
        return "No resource policies found in the selected namespace(s).";
    }
    "No resource policies match the current filters."
}

pub fn resource_quotas_presentation_mode(
    state: &ResourceQuotasState,
) -> ResourceQuotasPresentationMode {
    if state.status == ResourceQuotasLoadStatus::Error && state.policies.is_empty() {
        ResourceQuotasPresentationMode::ErrorCard
    } else {
        ResourceQuotasPresentationMode::Table
    }
}

fn render_resource_policy_cell(
    row_idx: usize,
    policy: &PolicyInfo,
    col: &TableColumn,
    theme: &OrbitTheme,
) -> AnyElement {
    match col.id.as_ref() {
        "name" => render_truncated_cell(
            ElementId::NamedInteger(SharedString::from("rq-name"), row_idx as u64),
            policy.name.clone(),
            Some(SharedString::from(policy.name.clone())),
            theme.text.primary,
            FontWeight::MEDIUM,
            false,
            theme,
        ),
        "namespace" => render_truncated_cell(
            ElementId::NamedInteger(SharedString::from("rq-ns"), row_idx as u64),
            policy.namespace.clone(),
            None,
            theme.text.secondary,
            FontWeight::NORMAL,
            false,
            theme,
        ),
        "kind" => render_truncated_cell(
            ElementId::NamedInteger(SharedString::from("rq-kind"), row_idx as u64),
            policy.kind.clone(),
            None,
            theme.text.secondary,
            FontWeight::MEDIUM,
            false,
            theme,
        ),
        "status" => render_status_badge_from_str(&policy.status, StatusBadgeVariant::Inline, theme),
        "violations" => apply_text_step(div(), theme.typography.size_xs)
            .font(table_numeric_font(theme))
            .text_color(if policy.violations > 0 {
                theme.semantic.danger.solid
            } else {
                theme.text.muted
            })
            .child(policy.violations.to_string())
            .into_any_element(),
        "rules_summary" => render_truncated_cell(
            ElementId::NamedInteger(SharedString::from("rq-rules"), row_idx as u64),
            if policy.description.trim().is_empty() {
                "-".to_string()
            } else {
                policy.description.clone()
            },
            (!policy.description.trim().is_empty())
                .then(|| SharedString::from(policy.description.clone())),
            if policy.description.trim().is_empty() {
                theme.text.muted
            } else {
                theme.text.secondary
            },
            FontWeight::NORMAL,
            false,
            theme,
        ),
        "age" => apply_text_step(div(), theme.typography.size_xs)
            .font(table_numeric_font(theme))
            .text_color(theme.text.muted)
            .child(policy.last_updated.clone())
            .into_any_element(),
        _ => div().into_any_element(),
    }
}

pub fn build_resource_policy_context_menu_entries(
    policy: &PolicyInfo,
    on_copy_name: impl 'static + Fn(&mut Window, &mut App),
    on_filter_namespace: impl 'static + Fn(&mut Window, &mut App),
    on_refresh: impl 'static + Fn(&mut Window, &mut App),
    on_view_details: impl 'static + Fn(&mut Window, &mut App),
    on_view_yaml: impl 'static + Fn(&mut Window, &mut App),
) -> Vec<ContextMenuEntry> {
    let delete_label = if policy.kind.eq_ignore_ascii_case("LimitRange") {
        "Delete LimitRange"
    } else {
        "Delete ResourceQuota"
    };

    vec![
        ContextMenuEntry::Item(
            ContextMenuItem::new("copy-name", "Copy Name")
                .icon(Icon::Copy)
                .on_select(on_copy_name),
        ),
        ContextMenuEntry::Item(
            ContextMenuItem::new(
                "filter-namespace",
                format!("Filter by \"{}\"", policy.namespace),
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
            ContextMenuItem::new("delete-resource-policy", delete_label)
                .icon(Icon::Trash2)
                .danger(true)
                .disabled(true),
        ),
    ]
}

type KindFilterCallback = Rc<dyn Fn(ResourceQuotaKindFilter, &mut Window, &mut App)>;

#[allow(clippy::too_many_arguments)]
pub fn render_resource_quotas_view(
    state: &ResourceQuotasState,
    namespaces: &NamespaceFilterState,
    theme: &OrbitTheme,
    on_kind_filter: impl 'static + Fn(ResourceQuotaKindFilter, &mut Window, &mut App),
    on_sort: impl 'static + Fn(SharedString, &ClickEvent, &mut Window, &mut App),
    on_row_click: impl 'static + Fn(usize, SharedString, &ClickEvent, &mut Window, &mut App),
    on_row_quick_yaml: impl 'static + Fn(usize, SharedString, &ClickEvent, &mut Window, &mut App),
    on_row_context_menu: impl 'static
        + Fn(SharedString, &PolicyInfo, &MouseDownEvent, &mut Window, &mut App),
    on_refresh: impl 'static + Fn(&mut Window, &mut App),
) -> AnyElement {
    let columns = resource_quota_columns();
    let filtered_rows: Rc<Vec<PolicyInfo>> = Rc::new(
        filter_and_sort_resource_policies(
            &state.policies,
            namespaces,
            state.kind_filter,
            state.table.sort.as_ref(),
        )
        .into_iter()
        .cloned()
        .collect(),
    );
    let total_count = state
        .policies
        .iter()
        .filter(|p| is_resource_policy(p))
        .count();
    let visible_count = filtered_rows.len();
    let is_loading = state.status == ResourceQuotasLoadStatus::Loading;
    let presentation_mode = resource_quotas_presentation_mode(state);
    let on_refresh_rc: super::ActionCallback = Rc::new(on_refresh);
    let on_kind_filter_rc: KindFilterCallback = Rc::new(on_kind_filter);

    let count_label = if namespaces.is_all_namespaces()
        && state.kind_filter == ResourceQuotaKindFilter::All
    {
        format!("{total_count} resource policies")
    } else {
        format!("{visible_count} of {total_count} resource policies")
    };

    let kind_chips = div().flex().items_center().gap_1p5().children(
        ResourceQuotaKindFilter::ALL_OPTIONS.into_iter().map(|opt| {
            let active = state.kind_filter == opt;
            let cb = on_kind_filter_rc.clone();
            let hover_bg = theme.surface.control_hover;
            let chip_id = match opt {
                ResourceQuotaKindFilter::All => "rq-kind-chip-all",
                ResourceQuotaKindFilter::ResourceQuota => "rq-kind-chip-quota",
                ResourceQuotaKindFilter::LimitRange => "rq-kind-chip-limit",
            };
            apply_text_step(div(), theme.typography.size_xs)
                .id(chip_id)
                .px_2p5()
                .py_1()
                .rounded(theme.style.radius_control)
                .border_1()
                .border_color(if active {
                    theme.accent.accent
                } else {
                    theme.border.default
                })
                .bg(if active {
                    theme.accent.soft
                } else {
                    theme.surface.bg_card
                })
                .text_color(if active {
                    theme.accent.accent
                } else {
                    theme.text.secondary
                })
                .font_weight(if active {
                    FontWeight::SEMIBOLD
                } else {
                    FontWeight::MEDIUM
                })
                .cursor_pointer()
                .when(!active, |el| el.hover(move |style| style.bg(hover_bg)))
                .on_click(move |_, window, cx| cb(opt, window, cx))
                .child(opt.label())
        }),
    );

    let header = render_resource_view_header(
        WorkspaceView::ResourceQuotas.label(),
        WorkspaceView::ResourceQuotas.description(),
        count_label,
        Some(namespaces),
        Some(kind_chips.into_any_element()),
        theme,
    );

    let error_banner = if presentation_mode == ResourceQuotasPresentationMode::Table {
        state.error_message.as_ref().map(|message| {
            render_resource_error_banner(
                "resource-quotas-retry-btn",
                format!("Failed to load resource quotas and limits: {message}"),
                is_loading,
                theme,
                on_refresh_rc.clone(),
            )
        })
    } else {
        None
    };

    let body = match presentation_mode {
        ResourceQuotasPresentationMode::ErrorCard => {
            let message = state
                .error_message
                .clone()
                .unwrap_or_else(|| "Unknown error".to_string());
            render_resource_error_state(
                "resource-quotas-retry-btn",
                "Unable to Load Resource Quotas & Limits",
                message,
                is_loading,
                theme,
                on_refresh_rc,
            )
        }
        ResourceQuotasPresentationMode::Table => {
            let mut table_state = state.table.clone();
            table_state.loading = is_loading && state.policies.is_empty();
            table_state.empty_message =
                SharedString::from(resource_quotas_empty_message(state, namespaces));

            let key_rows = filtered_rows.clone();
            let cell_rows = filtered_rows.clone();
            let ctx_rows = filtered_rows.clone();
            let callbacks = DataTableCallbacks::new(
                move |idx| {
                    key_rows
                        .get(idx)
                        .map(|policy| {
                            format!("{}/{}/{}", policy.kind, policy.namespace, policy.name)
                        })
                        .unwrap_or_default()
                },
                move |idx, col, col_theme| {
                    cell_rows
                        .get(idx)
                        .map(|policy| render_resource_policy_cell(idx, policy, col, col_theme))
                        .unwrap_or_else(|| div().into_any_element())
                },
            )
            .on_sort(on_sort)
            .on_row_click(on_row_click)
            .on_row_quick_yaml(on_row_quick_yaml)
            .on_row_context_menu(move |idx, row_key, event, window, cx| {
                if let Some(policy) = ctx_rows.get(idx) {
                    on_row_context_menu(row_key, policy, event, window, cx);
                }
            });

            render_data_table(
                "resource-quotas-table",
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
        .when_some(error_banner, |el, banner| el.child(banner))
        .child(body)
        .into_any_element()
}

pub fn filter_events_for_resource_policy<'a>(
    events: &'a [EventInfo],
    policy: &PolicyInfo,
) -> Vec<&'a EventInfo> {
    if policy.name.is_empty() {
        return Vec::new();
    }
    events
        .iter()
        .filter(|ev| {
            ev.namespace == policy.namespace
                && ((ev.object_kind.eq_ignore_ascii_case(&policy.kind)
                    && ev.object_name == policy.name)
                    || ev.message.contains(&policy.name))
        })
        .collect()
}

pub fn render_resource_policy_drawer_overview(
    policy: &PolicyInfo,
    theme: &OrbitTheme,
) -> AnyElement {
    let description = if policy.description.trim().is_empty() {
        format!("Kubernetes {}", policy.kind)
    } else {
        policy.description.clone()
    };

    let uid = if policy.uid.trim().is_empty() {
        "-".to_string()
    } else {
        policy.uid.clone()
    };

    let info_rows = vec![
        ("Name", policy.name.clone(), true),
        ("Namespace", policy.namespace.clone(), false),
        ("Kind", policy.kind.clone(), false),
        ("Status", policy.status.clone(), false),
        ("Violations", policy.violations.to_string(), true),
        ("UID", uid, true),
        ("Age", policy.last_updated.clone(), true),
    ];

    let rules_preview = if policy.rules.trim().is_empty() {
        None
    } else {
        let block = div()
            .w_full()
            .p_3()
            .rounded(theme.style.radius_control)
            .border_1()
            .border_color(theme.border.default)
            .bg(theme.surface.bg_card)
            .child(
                apply_text_step(div(), theme.typography.size_xs)
                    .font(theme.typography.mono_font(FontWeight::NORMAL))
                    .text_color(theme.text.primary)
                    .child(policy.rules.trim_end().to_string()),
            )
            .into_any_element();
        Some(render_drawer_section("RULES SPEC (YAML)", block, theme))
    };

    div()
        .w_full()
        .flex()
        .flex_col()
        .gap_4()
        .child(render_drawer_section(
            "DESCRIPTION",
            apply_text_step(div(), theme.typography.size_xs)
                .text_color(theme.text.primary)
                .child(description)
                .into_any_element(),
            theme,
        ))
        .child(render_drawer_section(
            "POLICY INFORMATION",
            render_drawer_key_values(info_rows, theme),
            theme,
        ))
        .when_some(rules_preview, |el, section| el.child(section))
        .into_any_element()
}

pub fn render_resource_policy_drawer_events(
    events: &[&EventInfo],
    loading: bool,
    theme: &OrbitTheme,
) -> AnyElement {
    render_drawer_events_list(
        events,
        loading,
        "No recent events for this resource policy.",
        theme,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::components::data_table::{ColumnAlign, SortDirection};

    #[allow(clippy::too_many_arguments)]
    fn sample_policy(
        name: &str,
        namespace: &str,
        kind: &str,
        policy_type: &str,
        status: &str,
        violations: i32,
        description: &str,
        age: &str,
    ) -> PolicyInfo {
        PolicyInfo {
            uid: format!("uid-{kind}-{name}"),
            name: name.to_string(),
            r#type: policy_type.to_string(),
            scope: "Namespaced".to_string(),
            namespace: namespace.to_string(),
            status: status.to_string(),
            mode: "enforce".to_string(),
            violations,
            last_updated: age.to_string(),
            description: description.to_string(),
            rules: "hard:\n  cpu: '4'\n".to_string(),
            kind: kind.to_string(),
        }
    }

    #[test]
    fn test_resource_quota_columns_cover_required_fields_and_alignment() {
        let cols = resource_quota_columns();
        let ids: Vec<&str> = cols.iter().map(|c| c.id.as_ref()).collect();
        assert_eq!(
            ids,
            vec![
                "name",
                "namespace",
                "kind",
                "status",
                "violations",
                "rules_summary",
                "age",
            ]
        );

        for id in ["violations", "age"] {
            let col = cols.iter().find(|c| c.id.as_ref() == id);
            assert!(col.is_some_and(|c| c.align == ColumnAlign::Right && c.monospace));
        }
    }

    #[test]
    fn test_filter_and_sort_resource_quotas_respects_kind_filter_namespace_and_sorts_all_columns() {
        let policies = vec![
            sample_policy(
                "compute-quota",
                "prod",
                "ResourceQuota",
                "Resource Quota",
                "Enforced",
                2,
                "Resource limits (3/3 tracked)",
                "5d",
            ),
            sample_policy(
                "default-limits",
                "staging",
                "LimitRange",
                "Limit Range",
                "Enforced",
                0,
                "Limits for: Container, Pod",
                "1h",
            ),
            sample_policy(
                "empty-limits",
                "prod",
                "LimitRange",
                "Limit Range",
                "Disabled",
                0,
                "No limits defined",
                "10m",
            ),
            sample_policy(
                "allow-api",
                "prod",
                "NetworkPolicy",
                "Network Policy",
                "Enforced",
                0,
                "Pods matching: app=api",
                "2d",
            ),
        ];

        let mut ns = NamespaceFilterState {
            cluster_id: Some("ctx-a".to_string()),
            ..Default::default()
        };

        // All excludes NetworkPolicy decoy
        let all = filter_and_sort_resource_policies(
            &policies,
            &ns,
            ResourceQuotaKindFilter::All,
            None,
        );
        let names: Vec<&str> = all.iter().map(|p| p.name.as_str()).collect();
        assert_eq!(names, vec!["compute-quota", "default-limits", "empty-limits"]);

        // Kind filter: ResourceQuota
        let only_rq = filter_and_sort_resource_policies(
            &policies,
            &ns,
            ResourceQuotaKindFilter::ResourceQuota,
            None,
        );
        assert_eq!(only_rq.len(), 1);
        assert_eq!(only_rq[0].kind, "ResourceQuota");

        // Kind filter: LimitRange
        let only_lr = filter_and_sort_resource_policies(
            &policies,
            &ns,
            ResourceQuotaKindFilter::LimitRange,
            None,
        );
        assert_eq!(only_lr.len(), 2);
        assert!(only_lr.iter().all(|p| p.kind == "LimitRange"));

        // Namespace filter
        ns.toggle_namespace("prod");
        let prod_lr = filter_and_sort_resource_policies(
            &policies,
            &ns,
            ResourceQuotaKindFilter::LimitRange,
            None,
        );
        assert_eq!(prod_lr.len(), 1);
        assert_eq!(prod_lr[0].name, "empty-limits");

        ns.select_all_namespaces();

        // Sort by violations desc
        let by_viol = filter_and_sort_resource_policies(
            &policies,
            &ns,
            ResourceQuotaKindFilter::All,
            Some(&SortState {
                column_id: "violations".to_string(),
                direction: SortDirection::Desc,
            }),
        );
        assert_eq!(by_viol[0].name, "compute-quota");

        // Sort by age asc (10m < 1h < 5d)
        let by_age = filter_and_sort_resource_policies(
            &policies,
            &ns,
            ResourceQuotaKindFilter::All,
            Some(&SortState {
                column_id: "age".to_string(),
                direction: SortDirection::Asc,
            }),
        );
        let ages: Vec<&str> = by_age.iter().map(|p| p.last_updated.as_str()).collect();
        assert_eq!(ages, vec!["10m", "1h", "5d"]);
    }

    #[test]
    fn test_resource_quotas_empty_message_distinguishes_cluster_kind_and_namespace_empty() {
        let mut state = ResourceQuotasState::default();
        let mut ns = NamespaceFilterState {
            cluster_id: Some("ctx-a".to_string()),
            ..Default::default()
        };

        assert_eq!(
            resource_quotas_empty_message(&state, &ns),
            "No resource quotas or limit ranges found in this cluster."
        );

        state.policies.push(sample_policy(
            "compute-quota",
            "prod",
            "ResourceQuota",
            "Resource Quota",
            "Enforced",
            0,
            "Resource limits (1/1 tracked)",
            "1d",
        ));

        state.set_kind_filter(ResourceQuotaKindFilter::LimitRange);
        assert_eq!(
            resource_quotas_empty_message(&state, &ns),
            "No LimitRanges found in this cluster."
        );

        state.set_kind_filter(ResourceQuotaKindFilter::All);
        ns.toggle_namespace("staging");
        assert_eq!(
            resource_quotas_empty_message(&state, &ns),
            "No resource policies found in the selected namespace(s)."
        );
    }

    #[test]
    fn test_filter_events_for_resource_policy_matches_exact_kind_and_name() {
        let rq = sample_policy(
            "compute-quota",
            "prod",
            "ResourceQuota",
            "Resource Quota",
            "Enforced",
            1,
            "Resource limits (1/1 tracked)",
            "1d",
        );

        let events = vec![
            EventInfo {
                uid: "e1".to_string(),
                time: "1m".to_string(),
                r#type: "Warning".to_string(),
                reason: "ExceededQuota".to_string(),
                object_name: "compute-quota".to_string(),
                object_kind: "ResourceQuota".to_string(),
                message: "exceeded quota".to_string(),
                namespace: "prod".to_string(),
                source: "quota-controller".to_string(),
                first_seen: String::new(),
                last_seen: String::new(),
                count: 1,
                labels: Default::default(),
                annotations: Default::default(),
            },
            EventInfo {
                uid: "e2".to_string(),
                time: "2m".to_string(),
                r#type: "Warning".to_string(),
                reason: "ExceededQuota".to_string(),
                object_name: "compute-quota".to_string(),
                object_kind: "LimitRange".to_string(),
                message: "unrelated".to_string(),
                namespace: "prod".to_string(),
                source: "limit-ranger".to_string(),
                first_seen: String::new(),
                last_seen: String::new(),
                count: 1,
                labels: Default::default(),
                annotations: Default::default(),
            },
        ];

        let matched = filter_events_for_resource_policy(&events, &rq);
        assert_eq!(matched.len(), 1);
        assert_eq!(matched[0].uid, "e1");
    }
}
