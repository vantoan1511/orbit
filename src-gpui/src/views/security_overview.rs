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
use crate::state::{NamespaceFilterState, SecurityOverviewLoadStatus, SecurityOverviewState};
use crate::theme::{OrbitTheme, apply_text_step};
use crate::views::{
    parse_age_seconds, render_drawer_events_list, render_drawer_key_values, render_drawer_section,
    render_resource_error_banner, render_resource_error_state, render_resource_view_header,
    render_truncated_cell, table_numeric_font,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SecurityOverviewPresentationMode {
    Table,
    ErrorCard,
}

pub fn security_policy_columns() -> Vec<TableColumn> {
    vec![
        TableColumn::new("name", "Name").flex_width(1.4, px(140.0)),
        TableColumn::new("type", "Type").fixed_width(px(136.0)),
        TableColumn::new("namespace", "Namespace").flex_width(0.9, px(104.0)),
        TableColumn::new("status", "Status").fixed_width(px(100.0)),
        TableColumn::new("mode", "Mode")
            .fixed_width(px(132.0))
            .monospace(true),
        TableColumn::new("violations", "Violations")
            .fixed_width(px(88.0))
            .right_numeric(),
        TableColumn::new("last_updated", "Last Updated")
            .fixed_width(px(96.0))
            .right_numeric(),
    ]
}

pub fn compare_security_policies_by_col(a: &PolicyInfo, b: &PolicyInfo, col_id: &str) -> Ordering {
    let tie_break = || {
        a.name
            .to_ascii_lowercase()
            .cmp(&b.name.to_ascii_lowercase())
            .then_with(|| {
                a.namespace
                    .to_ascii_lowercase()
                    .cmp(&b.namespace.to_ascii_lowercase())
            })
            .then_with(|| a.kind.cmp(&b.kind))
    };

    match col_id {
        "name" => tie_break(),
        "type" => a
            .r#type
            .to_ascii_lowercase()
            .cmp(&b.r#type.to_ascii_lowercase())
            .then_with(tie_break),
        "namespace" => a
            .namespace
            .to_ascii_lowercase()
            .cmp(&b.namespace.to_ascii_lowercase())
            .then_with(tie_break),
        "status" => a
            .status
            .to_ascii_lowercase()
            .cmp(&b.status.to_ascii_lowercase())
            .then_with(tie_break),
        "mode" => a
            .mode
            .to_ascii_lowercase()
            .cmp(&b.mode.to_ascii_lowercase())
            .then_with(tie_break),
        "violations" => a.violations.cmp(&b.violations).then_with(tie_break),
        "last_updated" => parse_age_seconds(&a.last_updated)
            .cmp(&parse_age_seconds(&b.last_updated))
            .then_with(tie_break),
        _ => tie_break(),
    }
}

pub fn filter_and_sort_security_policies<'a>(
    policies: &'a [PolicyInfo],
    namespaces: &NamespaceFilterState,
    sort: Option<&SortState>,
) -> Vec<&'a PolicyInfo> {
    let mut rows: Vec<&'a PolicyInfo> = policies
        .iter()
        .filter(|policy| {
            let ns = if policy.namespace.is_empty() || policy.namespace == "-" {
                None
            } else {
                Some(policy.namespace.as_str())
            };
            namespaces.matches_resource_namespace(ns)
        })
        .collect();

    if sort.is_some() {
        sort_rows_stable(&mut rows, sort, |a, b, col_id| {
            compare_security_policies_by_col(a, b, col_id)
        });
    } else {
        rows.sort_by(|a, b| compare_security_policies_by_col(a, b, "name"));
    }
    rows
}

pub fn security_overview_empty_message(state: &SecurityOverviewState) -> &'static str {
    if state.policies.is_empty() {
        "No security policies found in this cluster."
    } else {
        "No security policies found in the selected namespace(s)."
    }
}

pub fn security_overview_presentation_mode(
    state: &SecurityOverviewState,
) -> SecurityOverviewPresentationMode {
    if state.status == SecurityOverviewLoadStatus::Error && state.policies.is_empty() {
        SecurityOverviewPresentationMode::ErrorCard
    } else {
        SecurityOverviewPresentationMode::Table
    }
}

fn render_security_policy_cell(
    row_idx: usize,
    policy: &PolicyInfo,
    col: &TableColumn,
    theme: &OrbitTheme,
) -> AnyElement {
    match col.id.as_ref() {
        "name" => render_truncated_cell(
            ElementId::NamedInteger(SharedString::from("sec-name"), row_idx as u64),
            policy.name.clone(),
            Some(SharedString::from(policy.name.clone())),
            theme.text.primary,
            FontWeight::MEDIUM,
            false,
            theme,
        ),
        "type" => render_truncated_cell(
            ElementId::NamedInteger(SharedString::from("sec-type"), row_idx as u64),
            policy.r#type.clone(),
            None,
            theme.text.secondary,
            FontWeight::NORMAL,
            false,
            theme,
        ),
        "namespace" => render_truncated_cell(
            ElementId::NamedInteger(SharedString::from("sec-ns"), row_idx as u64),
            if policy.namespace.is_empty() {
                "-".to_string()
            } else {
                policy.namespace.clone()
            },
            None,
            if policy.namespace.is_empty() || policy.namespace == "-" {
                theme.text.muted
            } else {
                theme.text.secondary
            },
            FontWeight::NORMAL,
            false,
            theme,
        ),
        "status" => render_status_badge_from_str(&policy.status, StatusBadgeVariant::Inline, theme),
        "mode" => render_truncated_cell(
            ElementId::NamedInteger(SharedString::from("sec-mode"), row_idx as u64),
            if policy.mode.trim().is_empty() {
                "-".to_string()
            } else {
                policy.mode.clone()
            },
            None,
            if policy.mode.trim().is_empty() || policy.mode == "-" {
                theme.text.muted
            } else {
                theme.text.secondary
            },
            FontWeight::NORMAL,
            true,
            theme,
        ),
        "violations" => apply_text_step(div(), theme.typography.size_xs)
            .font(table_numeric_font(theme))
            .text_color(if policy.violations > 0 {
                theme.semantic.danger.solid
            } else {
                theme.text.muted
            })
            .child(policy.violations.to_string())
            .into_any_element(),
        "last_updated" => apply_text_step(div(), theme.typography.size_xs)
            .font(table_numeric_font(theme))
            .text_color(theme.text.muted)
            .child(policy.last_updated.clone())
            .into_any_element(),
        _ => div().into_any_element(),
    }
}

pub fn build_security_policy_context_menu_entries(
    policy: &PolicyInfo,
    on_copy_name: impl 'static + Fn(&mut Window, &mut App),
    on_filter_namespace: Option<impl 'static + Fn(&mut Window, &mut App)>,
    on_refresh: impl 'static + Fn(&mut Window, &mut App),
    on_view_details: impl 'static + Fn(&mut Window, &mut App),
    on_view_yaml: impl 'static + Fn(&mut Window, &mut App),
    on_edit: impl 'static + Fn(&mut Window, &mut App),
) -> Vec<ContextMenuEntry> {
    let mut entries = vec![ContextMenuEntry::Item(
        ContextMenuItem::new("copy-name", "Copy Name")
            .icon(Icon::Copy)
            .on_select(on_copy_name),
    )];
    if let Some(cb) = on_filter_namespace
        && !policy.namespace.is_empty()
        && policy.namespace != "-"
    {
        entries.push(ContextMenuEntry::Item(
            ContextMenuItem::new(
                "filter-namespace",
                format!("Filter by \"{}\"", policy.namespace),
            )
            .icon(Icon::Filter)
            .on_select(cb),
        ));
    }
    entries.extend([
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
        ContextMenuEntry::Item(
            ContextMenuItem::new("edit", "Edit")
                .icon(Icon::Pencil)
                .on_select(on_edit),
        ),
    ]);
    entries
}

#[allow(clippy::too_many_arguments)]
pub fn render_security_overview_view(
    state: &SecurityOverviewState,
    namespaces: &NamespaceFilterState,
    theme: &OrbitTheme,
    on_sort: impl 'static + Fn(SharedString, &ClickEvent, &mut Window, &mut App),
    on_row_click: impl 'static + Fn(usize, SharedString, &ClickEvent, &mut Window, &mut App),
    on_row_quick_yaml: impl 'static + Fn(usize, SharedString, &ClickEvent, &mut Window, &mut App),
    on_row_context_menu: impl 'static
    + Fn(SharedString, &PolicyInfo, &MouseDownEvent, &mut Window, &mut App),
    on_refresh: impl 'static + Fn(&mut Window, &mut App),
) -> AnyElement {
    let columns = security_policy_columns();
    let cache_key = crate::state::RowCacheKey::from_table(
        state.data_revision,
        namespaces.selection_revision,
        &state.table,
        0,
    );
    let filtered_rows = state.row_cache.get_or_compute(cache_key, || {
        filter_and_sort_security_policies(&state.policies, namespaces, state.table.sort.as_ref())
            .into_iter()
            .cloned()
            .collect()
    });
    let total_count = state.policies.len();
    let visible_count = filtered_rows.len();
    let is_loading = state.status == SecurityOverviewLoadStatus::Loading;
    let presentation_mode = security_overview_presentation_mode(state);
    let on_refresh_rc: super::ActionCallback = Rc::new(on_refresh);

    let count_label = if namespaces.is_all_namespaces() {
        format!("{total_count} policies")
    } else {
        format!("{visible_count} of {total_count} policies")
    };

    let header = render_resource_view_header(count_label, Some(namespaces), None, theme);

    let error_banner = if presentation_mode == SecurityOverviewPresentationMode::Table {
        state.error_message.as_ref().map(|message| {
            render_resource_error_banner(
                "security-overview-retry-btn",
                format!("Failed to load security policies: {message}"),
                is_loading,
                theme,
                on_refresh_rc.clone(),
            )
        })
    } else {
        None
    };

    let body = match presentation_mode {
        SecurityOverviewPresentationMode::ErrorCard => {
            let message = state
                .error_message
                .clone()
                .unwrap_or_else(|| "Unknown error".to_string());
            render_resource_error_state(
                "security-overview-retry-btn",
                "Unable to Load Security Overview",
                message,
                is_loading,
                theme,
                on_refresh_rc,
            )
        }
        SecurityOverviewPresentationMode::Table => {
            let mut table_state = state.table.clone();
            table_state.loading = is_loading && state.policies.is_empty();
            table_state.refreshing = is_loading && !state.policies.is_empty();
            table_state.empty_message = SharedString::from(security_overview_empty_message(state));

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
                        .map(|policy| render_security_policy_cell(idx, policy, col, col_theme))
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
                "security-overview-table",
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

pub fn filter_events_for_security_policy<'a>(
    events: &'a [EventInfo],
    policy: &PolicyInfo,
) -> Vec<&'a EventInfo> {
    if policy.kind.eq_ignore_ascii_case("Namespace") {
        return crate::views::pod_security::filter_events_for_pod_security(events, policy);
    }
    let is_cluster_scoped = policy.namespace.is_empty() || policy.namespace == "-";
    events
        .iter()
        .filter(|ev| {
            ev.object_kind == policy.kind
                && ev.object_name == policy.name
                && (is_cluster_scoped || ev.namespace == policy.namespace)
        })
        .collect()
}

pub fn render_security_policy_drawer_overview(
    policy: &PolicyInfo,
    theme: &OrbitTheme,
) -> AnyElement {
    let uid = if policy.uid.trim().is_empty() {
        "-".to_string()
    } else {
        policy.uid.clone()
    };

    let info_rows = vec![
        ("Name", policy.name.clone(), true),
        ("Type", policy.r#type.clone(), false),
        ("Kind", policy.kind.clone(), true),
        ("Scope", policy.scope.clone(), false),
        ("Namespace", policy.namespace.clone(), false),
        ("Status", policy.status.clone(), false),
        ("Mode", policy.mode.clone(), true),
        ("Violations", policy.violations.to_string(), true),
        ("UID", uid, true),
        ("Last Updated", policy.last_updated.clone(), true),
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
        Some(render_drawer_section("POLICY RULES (YAML)", block, theme))
    };

    div()
        .w_full()
        .flex()
        .flex_col()
        .gap_4()
        .when(!policy.description.trim().is_empty(), |el| {
            el.child(render_drawer_section(
                "DESCRIPTION",
                apply_text_step(div(), theme.typography.size_xs)
                    .text_color(theme.text.primary)
                    .child(policy.description.clone())
                    .into_any_element(),
                theme,
            ))
        })
        .child(render_drawer_section(
            "POLICY INFORMATION",
            render_drawer_key_values(info_rows, theme),
            theme,
        ))
        .when_some(rules_preview, |el, section| el.child(section))
        .into_any_element()
}

pub fn render_security_policy_drawer_events(
    events: &[&EventInfo],
    loading: bool,
    theme: &OrbitTheme,
) -> AnyElement {
    render_drawer_events_list(
        events,
        loading,
        "No recent events for this security policy.",
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
        policy_type: &str,
        kind: &str,
        namespace: &str,
        status: &str,
        violations: i32,
        age: &str,
    ) -> PolicyInfo {
        PolicyInfo {
            uid: format!("uid-{name}"),
            name: name.to_string(),
            r#type: policy_type.to_string(),
            scope: if namespace == "-" {
                "Cluster".to_string()
            } else {
                "Namespaced".to_string()
            },
            namespace: namespace.to_string(),
            status: status.to_string(),
            mode: "enforce".to_string(),
            violations,
            last_updated: age.to_string(),
            description: "policy".to_string(),
            rules: String::new(),
            kind: kind.to_string(),
        }
    }

    #[test]
    fn test_security_policy_columns_cover_required_fields() {
        let cols = security_policy_columns();
        let ids: Vec<&str> = cols.iter().map(|c| c.id.as_ref()).collect();
        assert_eq!(
            ids,
            vec![
                "name",
                "type",
                "namespace",
                "status",
                "mode",
                "violations",
                "last_updated",
            ]
        );
        for id in ["violations", "last_updated"] {
            let col = cols.iter().find(|c| c.id.as_ref() == id);
            assert!(col.is_some_and(|c| c.align == ColumnAlign::Right && c.monospace));
        }
    }

    #[test]
    fn test_filter_and_sort_security_policies() {
        let policies = vec![
            sample_policy(
                "zeta",
                "Network Policy",
                "NetworkPolicy",
                "prod",
                "Enforced",
                3,
                "10d",
            ),
            sample_policy(
                "alpha",
                "Admission Policy",
                "ValidatingWebhookConfiguration",
                "-",
                "Audit",
                0,
                "2h",
            ),
            sample_policy(
                "beta",
                "Resource Quota",
                "ResourceQuota",
                "staging",
                "Enforced",
                1,
                "30m",
            ),
        ];

        let mut ns = NamespaceFilterState {
            cluster_id: Some("ctx-a".to_string()),
            ..Default::default()
        };

        let all = filter_and_sort_security_policies(&policies, &ns, None);
        let names: Vec<&str> = all.iter().map(|p| p.name.as_str()).collect();
        assert_eq!(names, vec!["alpha", "beta", "zeta"]);

        ns.toggle_namespace("prod");
        let prod = filter_and_sort_security_policies(&policies, &ns, None);
        assert_eq!(prod.len(), 1);
        assert_eq!(prod[0].name, "zeta");

        ns.select_all_namespaces();
        let by_violations = filter_and_sort_security_policies(
            &policies,
            &ns,
            Some(&SortState {
                column_id: "violations".to_string(),
                direction: SortDirection::Desc,
            }),
        );
        let counts: Vec<i32> = by_violations.iter().map(|p| p.violations).collect();
        assert_eq!(counts, vec![3, 1, 0]);

        let by_age = filter_and_sort_security_policies(
            &policies,
            &ns,
            Some(&SortState {
                column_id: "last_updated".to_string(),
                direction: SortDirection::Asc,
            }),
        );
        assert_eq!(by_age[0].name, "beta");
    }
}
