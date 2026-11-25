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
use crate::state::{NamespaceFilterState, NetworkPoliciesLoadStatus, NetworkPoliciesState};
use crate::theme::{OrbitTheme, apply_text_step};
use crate::views::{
    parse_age_seconds, render_drawer_events_list, render_drawer_key_values, render_drawer_section,
    render_resource_error_banner, render_resource_error_state, render_resource_view_header,
    render_truncated_cell,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NetworkPoliciesPresentationMode {
    Table,
    ErrorCard,
}

pub fn network_policy_columns() -> Vec<TableColumn> {
    vec![
        TableColumn::new("name", "Name").flex_width(1.5, px(150.0)),
        TableColumn::new("namespace", "Namespace").flex_width(1.0, px(105.0)),
        TableColumn::new("scope", "Scope").fixed_width(px(100.0)),
        TableColumn::new("mode", "Mode")
            .fixed_width(px(124.0))
            .monospace(true),
        TableColumn::new("rules_summary", "Rules Summary").flex_width(1.8, px(180.0)),
        TableColumn::new("status", "Status").fixed_width(px(104.0)),
        TableColumn::new("age", "Age")
            .fixed_width(px(76.0))
            .right_numeric(),
    ]
}

pub fn is_network_policy(policy: &PolicyInfo) -> bool {
    policy.kind.eq_ignore_ascii_case("NetworkPolicy")
        || policy.r#type.eq_ignore_ascii_case("Network Policy")
}

pub fn compare_network_policies_by_col(a: &PolicyInfo, b: &PolicyInfo, col_id: &str) -> Ordering {
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
        "name" => tie_break(),
        "namespace" => a
            .namespace
            .to_ascii_lowercase()
            .cmp(&b.namespace.to_ascii_lowercase())
            .then_with(|| {
                a.name
                    .to_ascii_lowercase()
                    .cmp(&b.name.to_ascii_lowercase())
            }),
        "scope" => a
            .scope
            .to_ascii_lowercase()
            .cmp(&b.scope.to_ascii_lowercase())
            .then_with(tie_break),
        "mode" => a
            .mode
            .to_ascii_lowercase()
            .cmp(&b.mode.to_ascii_lowercase())
            .then_with(tie_break),
        "rules_summary" => a
            .description
            .to_ascii_lowercase()
            .cmp(&b.description.to_ascii_lowercase())
            .then_with(tie_break),
        "status" => a
            .status
            .to_ascii_lowercase()
            .cmp(&b.status.to_ascii_lowercase())
            .then_with(tie_break),
        "age" => parse_age_seconds(&a.last_updated)
            .cmp(&parse_age_seconds(&b.last_updated))
            .then_with(tie_break),
        _ => tie_break(),
    }
}

pub fn filter_and_sort_network_policies<'a>(
    policies: &'a [PolicyInfo],
    namespaces: &NamespaceFilterState,
    sort: Option<&SortState>,
) -> Vec<&'a PolicyInfo> {
    let mut rows: Vec<&'a PolicyInfo> = policies
        .iter()
        .filter(|policy| {
            is_network_policy(policy)
                && namespaces.matches_resource_namespace(Some(&policy.namespace))
        })
        .collect();
    if sort.is_some() {
        sort_rows_stable(&mut rows, sort, |a, b, col_id| {
            compare_network_policies_by_col(a, b, col_id)
        });
    } else {
        rows.sort_by(|a, b| compare_network_policies_by_col(a, b, "name"));
    }
    rows
}

pub fn network_policies_empty_message(state: &NetworkPoliciesState) -> &'static str {
    let has_network_policies = state.network_policies.iter().any(is_network_policy);
    if !has_network_policies {
        "No network policies found in this cluster."
    } else {
        "No network policies found in the selected namespace(s)."
    }
}

pub fn network_policies_presentation_mode(
    state: &NetworkPoliciesState,
) -> NetworkPoliciesPresentationMode {
    if state.status == NetworkPoliciesLoadStatus::Error && state.network_policies.is_empty() {
        NetworkPoliciesPresentationMode::ErrorCard
    } else {
        NetworkPoliciesPresentationMode::Table
    }
}

fn render_network_policy_cell(
    row_idx: usize,
    policy: &PolicyInfo,
    col: &TableColumn,
    theme: &OrbitTheme,
) -> AnyElement {
    match col.id.as_ref() {
        "name" => render_truncated_cell(
            ElementId::NamedInteger(SharedString::from("np-name"), row_idx as u64),
            policy.name.clone(),
            Some(SharedString::from(policy.name.clone())),
            theme.text.primary,
            FontWeight::MEDIUM,
            false,
            theme,
        ),
        "namespace" => render_truncated_cell(
            ElementId::NamedInteger(SharedString::from("np-ns"), row_idx as u64),
            policy.namespace.clone(),
            None,
            theme.text.secondary,
            FontWeight::NORMAL,
            false,
            theme,
        ),
        "scope" => render_truncated_cell(
            ElementId::NamedInteger(SharedString::from("np-scope"), row_idx as u64),
            policy.scope.to_ascii_uppercase(),
            None,
            theme.text.muted,
            FontWeight::MEDIUM,
            false,
            theme,
        ),
        "mode" => render_truncated_cell(
            ElementId::NamedInteger(SharedString::from("np-mode"), row_idx as u64),
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
        "rules_summary" => render_truncated_cell(
            ElementId::NamedInteger(SharedString::from("np-rules"), row_idx as u64),
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
        "status" => render_status_badge_from_str(&policy.status, StatusBadgeVariant::Inline, theme),
        "age" => apply_text_step(div(), theme.typography.size_xs)
            .font(theme.typography.mono_font(FontWeight::NORMAL))
            .text_color(theme.text.muted)
            .child(policy.last_updated.clone())
            .into_any_element(),
        _ => div().into_any_element(),
    }
}

pub fn build_network_policy_context_menu_entries(
    policy: &PolicyInfo,
    on_copy_name: impl 'static + Fn(&mut Window, &mut App),
    on_filter_namespace: impl 'static + Fn(&mut Window, &mut App),
    on_refresh: impl 'static + Fn(&mut Window, &mut App),
    on_view_details: impl 'static + Fn(&mut Window, &mut App),
    on_view_yaml: impl 'static + Fn(&mut Window, &mut App),
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
            ContextMenuItem::new("delete-network-policy", "Delete NetworkPolicy")
                .icon(Icon::Trash2)
                .danger(true)
                .on_select(on_delete),
        ),
    ]
}

#[allow(clippy::too_many_arguments)]
pub fn render_network_policies_view(
    state: &NetworkPoliciesState,
    namespaces: &NamespaceFilterState,
    theme: &OrbitTheme,
    on_sort: impl 'static + Fn(SharedString, &ClickEvent, &mut Window, &mut App),
    on_row_click: impl 'static + Fn(usize, SharedString, &ClickEvent, &mut Window, &mut App),
    on_row_quick_yaml: impl 'static + Fn(usize, SharedString, &ClickEvent, &mut Window, &mut App),
    on_row_context_menu: impl 'static
        + Fn(SharedString, &PolicyInfo, &MouseDownEvent, &mut Window, &mut App),
    on_refresh: impl 'static + Fn(&mut Window, &mut App),
) -> AnyElement {
    let columns = network_policy_columns();
    let filtered_rows: Rc<Vec<PolicyInfo>> = Rc::new(
        filter_and_sort_network_policies(
            &state.network_policies,
            namespaces,
            state.table.sort.as_ref(),
        )
        .into_iter()
        .cloned()
        .collect(),
    );
    let total_count = state
        .network_policies
        .iter()
        .filter(|p| is_network_policy(p))
        .count();
    let visible_count = filtered_rows.len();
    let is_loading = state.status == NetworkPoliciesLoadStatus::Loading;
    let presentation_mode = network_policies_presentation_mode(state);
    let on_refresh_rc: super::ActionCallback = Rc::new(on_refresh);

    let count_label = if namespaces.is_all_namespaces() {
        format!("{total_count} network policies")
    } else {
        format!("{visible_count} of {total_count} network policies")
    };

    let header = render_resource_view_header(
        WorkspaceView::NetworkPolicies.label(),
        WorkspaceView::NetworkPolicies.description(),
        count_label,
        Some(namespaces),
        None,
        theme,
    );

    let error_banner = if presentation_mode == NetworkPoliciesPresentationMode::Table {
        state.error_message.as_ref().map(|message| {
            render_resource_error_banner(
                "network-policies-retry-btn",
                format!("Failed to load network policies: {message}"),
                is_loading,
                theme,
                on_refresh_rc.clone(),
            )
        })
    } else {
        None
    };

    let body = match presentation_mode {
        NetworkPoliciesPresentationMode::ErrorCard => {
            let message = state
                .error_message
                .clone()
                .unwrap_or_else(|| "Unknown error".to_string());
            render_resource_error_state(
                "network-policies-retry-btn",
                "Unable to Load Network Policies",
                message,
                is_loading,
                theme,
                on_refresh_rc,
            )
        }
        NetworkPoliciesPresentationMode::Table => {
            let mut table_state = state.table.clone();
            table_state.loading = is_loading && state.network_policies.is_empty();
            table_state.empty_message = SharedString::from(network_policies_empty_message(state));

            let key_rows = filtered_rows.clone();
            let cell_rows = filtered_rows.clone();
            let ctx_rows = filtered_rows.clone();
            let callbacks = DataTableCallbacks::new(
                move |idx| {
                    key_rows
                        .get(idx)
                        .map(|policy| format!("{}/{}", policy.namespace, policy.name))
                        .unwrap_or_default()
                },
                move |idx, col, col_theme| {
                    cell_rows
                        .get(idx)
                        .map(|policy| render_network_policy_cell(idx, policy, col, col_theme))
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
                "network-policies-table",
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

pub fn filter_events_for_network_policy<'a>(
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
                && ((ev.object_kind == "NetworkPolicy" && ev.object_name == policy.name)
                    || ev.message.contains(&policy.name))
        })
        .collect()
}

pub fn render_network_policy_drawer_overview(
    policy: &PolicyInfo,
    theme: &OrbitTheme,
) -> AnyElement {
    let description = if policy.description.trim().is_empty() {
        "Kubernetes NetworkPolicy".to_string()
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
        ("Scope", policy.scope.clone(), false),
        ("Status", policy.status.clone(), false),
        ("Mode", policy.mode.clone(), true),
        ("UID", uid, true),
        ("Age", policy.last_updated.clone(), true),
    ];

    let rules_preview = if policy.rules.trim().is_empty() {
        None
    } else {
        let rules_block = div()
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
        Some(render_drawer_section("RULES SPEC (YAML)", rules_block, theme))
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

pub fn render_network_policy_drawer_events(
    events: &[&EventInfo],
    loading: bool,
    theme: &OrbitTheme,
) -> AnyElement {
    render_drawer_events_list(
        events,
        loading,
        "No recent events for this network policy.",
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
        scope: &str,
        mode: &str,
        description: &str,
        status: &str,
        age: &str,
        kind: &str,
        policy_type: &str,
    ) -> PolicyInfo {
        PolicyInfo {
            uid: format!("uid-{name}"),
            name: name.to_string(),
            r#type: policy_type.to_string(),
            scope: scope.to_string(),
            namespace: namespace.to_string(),
            status: status.to_string(),
            mode: mode.to_string(),
            violations: 0,
            last_updated: age.to_string(),
            description: description.to_string(),
            rules: "policyTypes:\n- Ingress\n".to_string(),
            kind: kind.to_string(),
        }
    }

    #[test]
    fn test_network_policy_columns_cover_required_fields_and_alignment() {
        let cols = network_policy_columns();
        let ids: Vec<&str> = cols.iter().map(|c| c.id.as_ref()).collect();
        assert_eq!(
            ids,
            vec![
                "name",
                "namespace",
                "scope",
                "mode",
                "rules_summary",
                "status",
                "age",
            ]
        );

        let Some(mode_col) = cols.iter().find(|c| c.id.as_ref() == "mode") else {
            panic!("expected mode column");
        };
        assert!(mode_col.monospace);

        let Some(age_col) = cols.iter().find(|c| c.id.as_ref() == "age") else {
            panic!("expected age column");
        };
        assert_eq!(age_col.align, ColumnAlign::Right);
        assert!(age_col.monospace);
    }

    #[test]
    fn test_filter_and_sort_network_policies_filters_non_network_policies_and_respects_namespace_filter()
    {
        let policies = vec![
            sample_policy(
                "zeta-deny",
                "prod",
                "Namespaced",
                "Ingress, Egress",
                "All pods in namespace",
                "Enforced",
                "2d",
                "NetworkPolicy",
                "Network Policy",
            ),
            sample_policy(
                "alpha-allow",
                "default",
                "Namespaced",
                "Ingress",
                "Pods matching: app=api",
                "Audit",
                "45m",
                "NetworkPolicy",
                "Network Policy",
            ),
            sample_policy(
                "beta-egress",
                "prod",
                "Namespaced",
                "Egress",
                "Pods matching: tier=worker",
                "Enforced",
                "3h",
                "NetworkPolicy",
                "Network Policy",
            ),
            sample_policy(
                "compute-quota",
                "prod",
                "Namespaced",
                "enforce",
                "Resource limits (1/3 tracked)",
                "Enforced",
                "1d",
                "ResourceQuota",
                "Resource Quota",
            ),
        ];

        // All namespaces -> filters out ResourceQuota, defaults to name ascending
        let mut ns_filter = NamespaceFilterState::default();
        let _ = ns_filter.begin_load_for_cluster("ctx-a");
        let all = filter_and_sort_network_policies(&policies, &ns_filter, None);
        let all_names: Vec<&str> = all.iter().map(|p| p.name.as_str()).collect();
        assert_eq!(all_names, vec!["alpha-allow", "beta-egress", "zeta-deny"]);

        // Single namespace filter
        ns_filter.toggle_namespace("prod");
        let prod_only = filter_and_sort_network_policies(&policies, &ns_filter, None);
        let prod_names: Vec<&str> = prod_only.iter().map(|p| p.name.as_str()).collect();
        assert_eq!(prod_names, vec!["beta-egress", "zeta-deny"]);

        // Multi-namespace filter
        ns_filter.toggle_namespace("default");
        let multi = filter_and_sort_network_policies(&policies, &ns_filter, None);
        assert_eq!(multi.len(), 3);

        // Reset to all namespaces for column sort checks
        ns_filter.select_all_namespaces();

        // Sort by age ascending: 45m < 3h < 2d
        let age_asc = filter_and_sort_network_policies(
            &policies,
            &ns_filter,
            Some(&SortState {
                column_id: "age".into(),
                direction: SortDirection::Asc,
            }),
        );
        let age_asc_names: Vec<&str> = age_asc.iter().map(|p| p.name.as_str()).collect();
        assert_eq!(
            age_asc_names,
            vec!["alpha-allow", "beta-egress", "zeta-deny"]
        );

        // Sort by age descending: 2d > 3h > 45m
        let age_desc = filter_and_sort_network_policies(
            &policies,
            &ns_filter,
            Some(&SortState {
                column_id: "age".into(),
                direction: SortDirection::Desc,
            }),
        );
        let age_desc_names: Vec<&str> = age_desc.iter().map(|p| p.name.as_str()).collect();
        assert_eq!(
            age_desc_names,
            vec!["zeta-deny", "beta-egress", "alpha-allow"]
        );

        // Sort by mode ascending: Egress < Ingress < Ingress, Egress
        let mode_asc = filter_and_sort_network_policies(
            &policies,
            &ns_filter,
            Some(&SortState {
                column_id: "mode".into(),
                direction: SortDirection::Asc,
            }),
        );
        let mode_asc_names: Vec<&str> = mode_asc.iter().map(|p| p.name.as_str()).collect();
        assert_eq!(
            mode_asc_names,
            vec!["beta-egress", "alpha-allow", "zeta-deny"]
        );

        // Sort by namespace ascending: default < prod (beta < zeta)
        let ns_asc = filter_and_sort_network_policies(
            &policies,
            &ns_filter,
            Some(&SortState {
                column_id: "namespace".into(),
                direction: SortDirection::Asc,
            }),
        );
        let ns_asc_names: Vec<&str> = ns_asc.iter().map(|p| p.name.as_str()).collect();
        assert_eq!(
            ns_asc_names,
            vec!["alpha-allow", "beta-egress", "zeta-deny"]
        );

        // Sort by rules_summary ascending: "All pods..." < "Pods matching: app=api" < "Pods matching: tier=worker"
        let rules_asc = filter_and_sort_network_policies(
            &policies,
            &ns_filter,
            Some(&SortState {
                column_id: "rules_summary".into(),
                direction: SortDirection::Asc,
            }),
        );
        let rules_asc_names: Vec<&str> = rules_asc.iter().map(|p| p.name.as_str()).collect();
        assert_eq!(
            rules_asc_names,
            vec!["zeta-deny", "alpha-allow", "beta-egress"]
        );

        // Sort by status ascending: Audit < Enforced
        let status_asc = filter_and_sort_network_policies(
            &policies,
            &ns_filter,
            Some(&SortState {
                column_id: "status".into(),
                direction: SortDirection::Asc,
            }),
        );
        let status_asc_names: Vec<&str> = status_asc.iter().map(|p| p.name.as_str()).collect();
        assert_eq!(
            status_asc_names,
            vec!["alpha-allow", "beta-egress", "zeta-deny"]
        );

        // Sort by scope ascending (tie-breaks by name)
        let scope_asc = filter_and_sort_network_policies(
            &policies,
            &ns_filter,
            Some(&SortState {
                column_id: "scope".into(),
                direction: SortDirection::Asc,
            }),
        );
        let scope_asc_names: Vec<&str> = scope_asc.iter().map(|p| p.name.as_str()).collect();
        assert_eq!(
            scope_asc_names,
            vec!["alpha-allow", "beta-egress", "zeta-deny"]
        );
    }

    #[test]
    fn test_network_policies_empty_message_and_presentation_mode() {
        let mut state = NetworkPoliciesState::default();
        assert_eq!(
            network_policies_empty_message(&state),
            "No network policies found in this cluster."
        );
        assert_eq!(
            network_policies_presentation_mode(&state),
            NetworkPoliciesPresentationMode::Table
        );

        // Error + empty -> ErrorCard
        state.status = NetworkPoliciesLoadStatus::Error;
        state.error_message = Some("Forbidden".to_string());
        assert_eq!(
            network_policies_presentation_mode(&state),
            NetworkPoliciesPresentationMode::ErrorCard
        );

        // Error + cached rows -> Table (renders inline banner above cached rows)
        state.network_policies.push(sample_policy(
            "allow-api",
            "prod",
            "Namespaced",
            "Ingress",
            "Pods matching: app=api",
            "Enforced",
            "1h",
            "NetworkPolicy",
            "Network Policy",
        ));
        assert_eq!(
            network_policies_presentation_mode(&state),
            NetworkPoliciesPresentationMode::Table
        );
        assert_eq!(
            network_policies_empty_message(&state),
            "No network policies found in the selected namespace(s)."
        );
    }

    #[test]
    fn test_filter_events_for_network_policy_matches_kind_and_name_in_namespace() {
        let policy = sample_policy(
            "allow-api",
            "prod",
            "Namespaced",
            "Ingress",
            "Pods matching: app=api",
            "Enforced",
            "1h",
            "NetworkPolicy",
            "Network Policy",
        );

        let events = vec![
            EventInfo {
                uid: "ev-1".to_string(),
                name: String::new(),
                r#type: "Normal".to_string(),
                reason: "Updated".to_string(),
                message: "Applied policy rules".to_string(),
                object_kind: "NetworkPolicy".to_string(),
                object_name: "allow-api".to_string(),
                namespace: "prod".to_string(),
                source: "cilium".to_string(),
                count: 1,
                time: "1m".to_string(),
                first_seen: "1m ago".to_string(),
                last_seen: "1m ago".to_string(),
                labels: Default::default(),
                annotations: Default::default(),
            },
            EventInfo {
                uid: "ev-2".to_string(),
                name: String::new(),
                r#type: "Warning".to_string(),
                reason: "PolicyMismatch".to_string(),
                message: "Pod blocked by allow-api selector".to_string(),
                object_kind: "Pod".to_string(),
                object_name: "web-0".to_string(),
                namespace: "prod".to_string(),
                source: "calico".to_string(),
                count: 1,
                time: "2m".to_string(),
                first_seen: "2m ago".to_string(),
                last_seen: "2m ago".to_string(),
                labels: Default::default(),
                annotations: Default::default(),
            },
            EventInfo {
                uid: "ev-3".to_string(),
                name: String::new(),
                r#type: "Normal".to_string(),
                reason: "Updated".to_string(),
                message: "Applied policy rules".to_string(),
                object_kind: "NetworkPolicy".to_string(),
                object_name: "allow-api".to_string(),
                namespace: "staging".to_string(),
                source: "cilium".to_string(),
                count: 1,
                time: "3m".to_string(),
                first_seen: "3m ago".to_string(),
                last_seen: "3m ago".to_string(),
                labels: Default::default(),
                annotations: Default::default(),
            },
            EventInfo {
                uid: "ev-4".to_string(),
                name: String::new(),
                r#type: "Normal".to_string(),
                reason: "Scheduled".to_string(),
                message: "Successfully assigned pod".to_string(),
                object_kind: "Pod".to_string(),
                object_name: "other-pod".to_string(),
                namespace: "prod".to_string(),
                source: "scheduler".to_string(),
                count: 1,
                time: "4m".to_string(),
                first_seen: "4m ago".to_string(),
                last_seen: "4m ago".to_string(),
                labels: Default::default(),
                annotations: Default::default(),
            },
        ];

        let matched = filter_events_for_network_policy(&events, &policy);
        let ids: Vec<&str> = matched.iter().map(|e| e.uid.as_str()).collect();
        assert_eq!(ids, vec!["ev-1", "ev-2"]);
    }

    #[test]
    fn test_build_network_policy_context_menu_entries_contains_expected_actions() {
        let policy = sample_policy(
            "allow-api",
            "prod",
            "Namespaced",
            "Ingress",
            "Pods matching: app=api",
            "Enforced",
            "1h",
            "NetworkPolicy",
            "Network Policy",
        );

        let entries = build_network_policy_context_menu_entries(
            &policy,
            |_, _| {},
            |_, _| {},
            |_, _| {},
            |_, _| {},
            |_, _| {},
            |_, _| {},
        );

        let item_ids: Vec<&str> = entries
            .iter()
            .filter_map(|entry| match entry {
                ContextMenuEntry::Item(item) => Some(item.id.as_ref()),
                ContextMenuEntry::Separator => None,
            })
            .collect();
        assert_eq!(
            item_ids,
            vec![
                "copy-name",
                "filter-namespace",
                "refresh",
                "view-details",
                "view-yaml",
                "delete-network-policy",
            ]
        );

        let filter_item = entries.iter().find_map(|entry| match entry {
            ContextMenuEntry::Item(item) if item.id.as_ref() == "filter-namespace" => Some(item),
            _ => None,
        });
        assert_eq!(
            filter_item.map(|item| item.label.as_ref()),
            Some("Filter by \"prod\"")
        );

        let delete_item = entries.iter().find_map(|entry| match entry {
            ContextMenuEntry::Item(item) if item.id.as_ref() == "delete-network-policy" => {
                Some(item)
            }
            _ => None,
        });
        assert!(delete_item.is_some_and(|item| item.danger && !item.disabled && item.on_select.is_some()));
    }
}
