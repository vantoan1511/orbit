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
use crate::state::{NamespaceFilterState, PodSecurityLoadStatus, PodSecurityState};
use crate::theme::{OrbitTheme, apply_text_step};
use crate::views::{
    parse_age_seconds, render_drawer_events_list, render_drawer_key_values, render_drawer_section,
    render_resource_error_banner, render_resource_error_state, render_resource_view_header,
    render_truncated_cell, table_numeric_font,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PodSecurityPresentationMode {
    Table,
    ErrorCard,
}

pub fn pod_security_columns() -> Vec<TableColumn> {
    vec![
        TableColumn::new("name", "Name").flex_width(1.4, px(140.0)),
        TableColumn::new("namespace", "Namespace").flex_width(1.0, px(110.0)),
        TableColumn::new("mode", "Mode")
            .fixed_width(px(140.0))
            .monospace(true),
        TableColumn::new("status", "Status").fixed_width(px(104.0)),
        TableColumn::new("violations", "Violations")
            .fixed_width(px(88.0))
            .right_numeric(),
        TableColumn::new("rules_summary", "Description").flex_width(1.8, px(180.0)),
        TableColumn::new("age", "Age")
            .fixed_width(px(76.0))
            .right_numeric(),
    ]
}

pub fn is_pod_security_policy(policy: &PolicyInfo) -> bool {
    policy.r#type.eq_ignore_ascii_case("Pod Security")
        || policy.kind.eq_ignore_ascii_case("Namespace")
}

pub fn compare_pod_security_by_col(a: &PolicyInfo, b: &PolicyInfo, col_id: &str) -> Ordering {
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
        "mode" => a
            .mode
            .to_ascii_lowercase()
            .cmp(&b.mode.to_ascii_lowercase())
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

pub fn filter_and_sort_pod_security_policies<'a>(
    policies: &'a [PolicyInfo],
    namespaces: &NamespaceFilterState,
    sort: Option<&SortState>,
) -> Vec<&'a PolicyInfo> {
    let mut rows: Vec<&'a PolicyInfo> = policies
        .iter()
        .filter(|policy| {
            is_pod_security_policy(policy)
                && namespaces.matches_resource_namespace(Some(&policy.namespace))
        })
        .collect();
    if sort.is_some() {
        sort_rows_stable(&mut rows, sort, |a, b, col_id| {
            compare_pod_security_by_col(a, b, col_id)
        });
    } else {
        rows.sort_by(|a, b| compare_pod_security_by_col(a, b, "name"));
    }
    rows
}

pub fn pod_security_empty_message(state: &PodSecurityState) -> &'static str {
    let has_policies = state.policies.iter().any(is_pod_security_policy);
    if !has_policies {
        "No pod security policies found in this cluster."
    } else {
        "No pod security policies found in the selected namespace(s)."
    }
}

pub fn pod_security_presentation_mode(state: &PodSecurityState) -> PodSecurityPresentationMode {
    if state.status == PodSecurityLoadStatus::Error && state.policies.is_empty() {
        PodSecurityPresentationMode::ErrorCard
    } else {
        PodSecurityPresentationMode::Table
    }
}

fn render_pod_security_cell(
    row_idx: usize,
    policy: &PolicyInfo,
    col: &TableColumn,
    theme: &OrbitTheme,
) -> AnyElement {
    match col.id.as_ref() {
        "name" => render_truncated_cell(
            ElementId::NamedInteger(SharedString::from("psa-name"), row_idx as u64),
            policy.name.clone(),
            Some(SharedString::from(policy.name.clone())),
            theme.text.primary,
            FontWeight::MEDIUM,
            false,
            theme,
        ),
        "namespace" => render_truncated_cell(
            ElementId::NamedInteger(SharedString::from("psa-ns"), row_idx as u64),
            policy.namespace.clone(),
            None,
            theme.text.secondary,
            FontWeight::NORMAL,
            false,
            theme,
        ),
        "mode" => render_truncated_cell(
            ElementId::NamedInteger(SharedString::from("psa-mode"), row_idx as u64),
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
            ElementId::NamedInteger(SharedString::from("psa-rules"), row_idx as u64),
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

pub fn build_pod_security_context_menu_entries(
    policy: &PolicyInfo,
    on_copy_name: impl 'static + Fn(&mut Window, &mut App),
    on_filter_namespace: impl 'static + Fn(&mut Window, &mut App),
    on_refresh: impl 'static + Fn(&mut Window, &mut App),
    on_view_details: impl 'static + Fn(&mut Window, &mut App),
    on_view_yaml: impl 'static + Fn(&mut Window, &mut App),
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
    ]
}

#[allow(clippy::too_many_arguments)]
pub fn render_pod_security_view(
    state: &PodSecurityState,
    namespaces: &NamespaceFilterState,
    theme: &OrbitTheme,
    on_sort: impl 'static + Fn(SharedString, &ClickEvent, &mut Window, &mut App),
    on_row_click: impl 'static + Fn(usize, SharedString, &ClickEvent, &mut Window, &mut App),
    on_row_quick_yaml: impl 'static + Fn(usize, SharedString, &ClickEvent, &mut Window, &mut App),
    on_row_context_menu: impl 'static
        + Fn(SharedString, &PolicyInfo, &MouseDownEvent, &mut Window, &mut App),
    on_refresh: impl 'static + Fn(&mut Window, &mut App),
) -> AnyElement {
    let columns = pod_security_columns();
    let filtered_rows: Rc<Vec<PolicyInfo>> = Rc::new(
        filter_and_sort_pod_security_policies(&state.policies, namespaces, state.table.sort.as_ref())
            .into_iter()
            .cloned()
            .collect(),
    );
    let total_count = state
        .policies
        .iter()
        .filter(|p| is_pod_security_policy(p))
        .count();
    let visible_count = filtered_rows.len();
    let is_loading = state.status == PodSecurityLoadStatus::Loading;
    let presentation_mode = pod_security_presentation_mode(state);
    let on_refresh_rc: super::ActionCallback = Rc::new(on_refresh);

    let count_label = if namespaces.is_all_namespaces() {
        format!("{total_count} pod security policies")
    } else {
        format!("{visible_count} of {total_count} pod security policies")
    };

    let header = render_resource_view_header(
        WorkspaceView::PodSecurity.label(),
        WorkspaceView::PodSecurity.description(),
        count_label,
        Some(namespaces),
        None,
        theme,
    );

    let error_banner = if presentation_mode == PodSecurityPresentationMode::Table {
        state.error_message.as_ref().map(|message| {
            render_resource_error_banner(
                "pod-security-retry-btn",
                format!("Failed to load pod security policies: {message}"),
                is_loading,
                theme,
                on_refresh_rc.clone(),
            )
        })
    } else {
        None
    };

    let body = match presentation_mode {
        PodSecurityPresentationMode::ErrorCard => {
            let message = state
                .error_message
                .clone()
                .unwrap_or_else(|| "Unknown error".to_string());
            render_resource_error_state(
                "pod-security-retry-btn",
                "Unable to Load Pod Security Policies",
                message,
                is_loading,
                theme,
                on_refresh_rc,
            )
        }
        PodSecurityPresentationMode::Table => {
            let mut table_state = state.table.clone();
            table_state.loading = is_loading && state.policies.is_empty();
            table_state.empty_message = SharedString::from(pod_security_empty_message(state));

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
                        .map(|policy| render_pod_security_cell(idx, policy, col, col_theme))
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
                "pod-security-table",
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

pub fn filter_events_for_pod_security<'a>(
    events: &'a [EventInfo],
    policy: &PolicyInfo,
) -> Vec<&'a EventInfo> {
    if policy.namespace.is_empty() {
        return Vec::new();
    }
    events
        .iter()
        .filter(|ev| {
            (ev.object_kind == "Namespace" && ev.object_name == policy.namespace)
                || (ev.namespace == policy.namespace
                    && (ev.message.contains("pod-security")
                        || ev.reason.to_ascii_lowercase().contains("podsecurity")))
        })
        .collect()
}

pub fn render_pod_security_drawer_overview(
    policy: &PolicyInfo,
    theme: &OrbitTheme,
) -> AnyElement {
    let description = if policy.description.trim().is_empty() {
        "Namespace Pod Security Admission policy".to_string()
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
        ("Mode", policy.mode.clone(), true),
        ("Status", policy.status.clone(), false),
        ("Violations", policy.violations.to_string(), true),
        ("UID", uid, true),
        ("Age", policy.last_updated.clone(), true),
    ];

    let labels_preview = if policy.rules.trim().is_empty() {
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
        Some(render_drawer_section("POD SECURITY LABELS (YAML)", block, theme))
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
        .when_some(labels_preview, |el, section| el.child(section))
        .into_any_element()
}

pub fn render_pod_security_drawer_events(
    events: &[&EventInfo],
    loading: bool,
    theme: &OrbitTheme,
) -> AnyElement {
    render_drawer_events_list(
        events,
        loading,
        "No recent Pod Security events for this namespace.",
        theme,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::components::data_table::{ColumnAlign, SortDirection};

    #[allow(clippy::too_many_arguments)]
    fn sample_psa(
        name: &str,
        namespace: &str,
        mode: &str,
        status: &str,
        violations: i32,
        description: &str,
        age: &str,
    ) -> PolicyInfo {
        PolicyInfo {
            uid: format!("uid-{name}"),
            name: name.to_string(),
            r#type: "Pod Security".to_string(),
            scope: "Namespaced".to_string(),
            namespace: namespace.to_string(),
            status: status.to_string(),
            mode: mode.to_string(),
            violations,
            last_updated: age.to_string(),
            description: description.to_string(),
            rules: "pod-security.kubernetes.io/enforce: restricted\n".to_string(),
            kind: "Namespace".to_string(),
        }
    }

    #[test]
    fn test_pod_security_columns_cover_required_fields_and_alignment() {
        let cols = pod_security_columns();
        let ids: Vec<&str> = cols.iter().map(|c| c.id.as_ref()).collect();
        assert_eq!(
            ids,
            vec![
                "name",
                "namespace",
                "mode",
                "status",
                "violations",
                "rules_summary",
                "age",
            ]
        );

        let mode_col = cols.iter().find(|c| c.id.as_ref() == "mode");
        assert!(mode_col.is_some_and(|c| c.monospace));

        for id in ["violations", "age"] {
            let col = cols.iter().find(|c| c.id.as_ref() == id);
            assert!(col.is_some_and(|c| c.align == ColumnAlign::Right && c.monospace));
        }
    }

    #[test]
    fn test_filter_and_sort_pod_security_policies_filters_type_and_namespace_and_sorts_all_columns()
    {
        let mut decoy = sample_psa("allow-web", "prod", "Ingress", "Enforced", 0, "Pods", "1d");
        decoy.r#type = "Network Policy".to_string();
        decoy.kind = "NetworkPolicy".to_string();

        let policies = vec![
            sample_psa(
                "psa-staging",
                "staging",
                "audit: baseline",
                "Audit",
                0,
                "PSA: audit=baseline",
                "10d",
            ),
            sample_psa(
                "psa-prod",
                "prod",
                "enforce: restricted",
                "Enforced",
                3,
                "PSA: enforce=restricted",
                "2h",
            ),
            sample_psa(
                "psa-dev",
                "dev",
                "warn: baseline",
                "Audit",
                1,
                "PSA: warn=baseline",
                "30m",
            ),
            decoy,
        ];

        let mut ns = NamespaceFilterState {
            cluster_id: Some("ctx-a".to_string()),
            ..Default::default()
        };

        // Default sort (name asc) excludes NetworkPolicy decoy
        let all = filter_and_sort_pod_security_policies(&policies, &ns, None);
        let names: Vec<&str> = all.iter().map(|p| p.name.as_str()).collect();
        assert_eq!(names, vec!["psa-dev", "psa-prod", "psa-staging"]);

        // Single & multi-namespace filter
        ns.toggle_namespace("prod");
        let prod_only = filter_and_sort_pod_security_policies(&policies, &ns, None);
        assert_eq!(prod_only.len(), 1);
        assert_eq!(prod_only[0].namespace, "prod");

        ns.toggle_namespace("staging");
        let multi = filter_and_sort_pod_security_policies(&policies, &ns, None);
        let multi_ns: Vec<&str> = multi.iter().map(|p| p.namespace.as_str()).collect();
        assert_eq!(multi_ns, vec!["prod", "staging"]);

        ns.select_all_namespaces();

        // Sort by violations desc
        let by_violations = filter_and_sort_pod_security_policies(
            &policies,
            &ns,
            Some(&SortState {
                column_id: "violations".to_string(),
                direction: SortDirection::Desc,
            }),
        );
        let viol_counts: Vec<i32> = by_violations.iter().map(|p| p.violations).collect();
        assert_eq!(viol_counts, vec![3, 1, 0]);

        // Sort by age asc (30m < 2h < 10d)
        let by_age = filter_and_sort_pod_security_policies(
            &policies,
            &ns,
            Some(&SortState {
                column_id: "age".to_string(),
                direction: SortDirection::Asc,
            }),
        );
        let ages: Vec<&str> = by_age.iter().map(|p| p.last_updated.as_str()).collect();
        assert_eq!(ages, vec!["30m", "2h", "10d"]);
    }

    #[test]
    fn test_pod_security_empty_message_and_presentation_mode() {
        let mut state = PodSecurityState::default();
        assert_eq!(
            pod_security_empty_message(&state),
            "No pod security policies found in this cluster."
        );
        assert_eq!(
            pod_security_presentation_mode(&state),
            PodSecurityPresentationMode::Table
        );

        state.status = PodSecurityLoadStatus::Error;
        assert_eq!(
            pod_security_presentation_mode(&state),
            PodSecurityPresentationMode::ErrorCard
        );

        state.policies.push(sample_psa(
            "psa-prod",
            "prod",
            "enforce: restricted",
            "Enforced",
            0,
            "PSA: enforce=restricted",
            "1d",
        ));
        assert_eq!(
            pod_security_empty_message(&state),
            "No pod security policies found in the selected namespace(s)."
        );
        assert_eq!(
            pod_security_presentation_mode(&state),
            PodSecurityPresentationMode::Table
        );
    }

    #[test]
    fn test_filter_events_for_pod_security_matches_namespace_events() {
        let policy = sample_psa(
            "psa-prod",
            "prod",
            "enforce: restricted",
            "Enforced",
            0,
            "PSA: enforce=restricted",
            "1d",
        );

        let events = vec![
            EventInfo {
                uid: "e1".to_string(),
                name: String::new(),
                time: "1m".to_string(),
                r#type: "Normal".to_string(),
                reason: "Updated".to_string(),
                object_name: "prod".to_string(),
                object_kind: "Namespace".to_string(),
                message: "Namespace labels updated".to_string(),
                namespace: String::new(),
                source: "kube-apiserver".to_string(),
                first_seen: String::new(),
                last_seen: String::new(),
                count: 1,
                labels: Default::default(),
                annotations: Default::default(),
            },
            EventInfo {
                uid: "e2".to_string(),
                name: String::new(),
                time: "2m".to_string(),
                r#type: "Warning".to_string(),
                reason: "FailedCreate".to_string(),
                object_name: "api".to_string(),
                object_kind: "ReplicaSet".to_string(),
                message: "violates PodSecurity pod-security.kubernetes.io/enforce".to_string(),
                namespace: "prod".to_string(),
                source: "replicaset-controller".to_string(),
                first_seen: String::new(),
                last_seen: String::new(),
                count: 2,
                labels: Default::default(),
                annotations: Default::default(),
            },
            EventInfo {
                uid: "e3".to_string(),
                name: String::new(),
                time: "3m".to_string(),
                r#type: "Normal".to_string(),
                reason: "Scheduled".to_string(),
                object_name: "other".to_string(),
                object_kind: "Pod".to_string(),
                message: "Successfully assigned".to_string(),
                namespace: "staging".to_string(),
                source: "scheduler".to_string(),
                first_seen: String::new(),
                last_seen: String::new(),
                count: 1,
                labels: Default::default(),
                annotations: Default::default(),
            },
        ];

        let matched = filter_events_for_pod_security(&events, &policy);
        let uids: Vec<&str> = matched.iter().map(|e| e.uid.as_str()).collect();
        assert_eq!(uids, vec!["e1", "e2"]);
    }
}
