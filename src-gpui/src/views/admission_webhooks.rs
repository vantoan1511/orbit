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
use crate::state::{AdmissionWebhooksLoadStatus, AdmissionWebhooksState};
use crate::theme::{OrbitTheme, apply_text_step};
use crate::views::{
    parse_age_seconds, render_drawer_events_list, render_drawer_key_values, render_drawer_section,
    render_resource_error_banner, render_resource_error_state, render_resource_view_header,
    render_truncated_cell, table_numeric_font,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AdmissionWebhooksPresentationMode {
    Table,
    ErrorCard,
}

pub fn admission_webhook_columns() -> Vec<TableColumn> {
    vec![
        TableColumn::new("name", "Name").flex_width(1.6, px(160.0)),
        TableColumn::new("kind", "Kind").fixed_width(px(108.0)),
        TableColumn::new("mode", "Failure Policy")
            .fixed_width(px(120.0))
            .monospace(true),
        TableColumn::new("status", "Status").fixed_width(px(104.0)),
        TableColumn::new("rules", "Rules").flex_width(1.4, px(140.0)),
        TableColumn::new("age", "Age")
            .fixed_width(px(76.0))
            .right_numeric(),
    ]
}

pub fn admission_webhook_kind_label(policy: &PolicyInfo) -> &'static str {
    if policy
        .kind
        .eq_ignore_ascii_case("MutatingWebhookConfiguration")
    {
        "Mutating"
    } else if policy
        .kind
        .eq_ignore_ascii_case("ValidatingWebhookConfiguration")
    {
        "Validating"
    } else {
        "-"
    }
}

pub fn is_admission_webhook(policy: &PolicyInfo) -> bool {
    policy
        .kind
        .eq_ignore_ascii_case("MutatingWebhookConfiguration")
        || policy
            .kind
            .eq_ignore_ascii_case("ValidatingWebhookConfiguration")
}

pub fn compare_admission_webhooks_by_col(a: &PolicyInfo, b: &PolicyInfo, col_id: &str) -> Ordering {
    let tie_break = || {
        a.name
            .to_ascii_lowercase()
            .cmp(&b.name.to_ascii_lowercase())
            .then_with(|| a.kind.cmp(&b.kind))
    };

    match col_id {
        "name" => tie_break(),
        "kind" => admission_webhook_kind_label(a)
            .cmp(admission_webhook_kind_label(b))
            .then_with(tie_break),
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
        "rules" => a
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

pub fn filter_and_sort_admission_webhooks<'a>(
    policies: &'a [PolicyInfo],
    sort: Option<&SortState>,
) -> Vec<&'a PolicyInfo> {
    let mut rows: Vec<&'a PolicyInfo> = policies
        .iter()
        .filter(|policy| is_admission_webhook(policy))
        .collect();

    if sort.is_some() {
        sort_rows_stable(&mut rows, sort, |a, b, col_id| {
            compare_admission_webhooks_by_col(a, b, col_id)
        });
    } else {
        rows.sort_by(|a, b| compare_admission_webhooks_by_col(a, b, "name"));
    }
    rows
}

pub fn admission_webhooks_empty_message(state: &AdmissionWebhooksState) -> &'static str {
    let has_webhooks = state.policies.iter().any(is_admission_webhook);
    if !has_webhooks {
        "No admission webhooks found in this cluster."
    } else {
        "No admission webhooks match the current filter."
    }
}

pub fn admission_webhooks_presentation_mode(
    state: &AdmissionWebhooksState,
) -> AdmissionWebhooksPresentationMode {
    if state.status == AdmissionWebhooksLoadStatus::Error && state.policies.is_empty() {
        AdmissionWebhooksPresentationMode::ErrorCard
    } else {
        AdmissionWebhooksPresentationMode::Table
    }
}

fn render_admission_webhook_cell(
    row_idx: usize,
    policy: &PolicyInfo,
    col: &TableColumn,
    theme: &OrbitTheme,
) -> AnyElement {
    match col.id.as_ref() {
        "name" => render_truncated_cell(
            ElementId::NamedInteger(SharedString::from("aw-name"), row_idx as u64),
            policy.name.clone(),
            Some(SharedString::from(policy.name.clone())),
            theme.text.primary,
            FontWeight::MEDIUM,
            false,
            theme,
        ),
        "kind" => {
            let label = admission_webhook_kind_label(policy);
            apply_text_step(div(), theme.typography.size_xs)
                .font_weight(FontWeight::MEDIUM)
                .text_color(theme.text.secondary)
                .child(label)
                .into_any_element()
        }
        "mode" => render_truncated_cell(
            ElementId::NamedInteger(SharedString::from("aw-mode"), row_idx as u64),
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
        "rules" => render_truncated_cell(
            ElementId::NamedInteger(SharedString::from("aw-rules"), row_idx as u64),
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

pub fn build_admission_webhook_context_menu_entries(
    on_copy_name: impl 'static + Fn(&mut Window, &mut App),
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
pub fn render_admission_webhooks_view(
    state: &AdmissionWebhooksState,
    theme: &OrbitTheme,
    on_sort: impl 'static + Fn(SharedString, &ClickEvent, &mut Window, &mut App),
    on_row_click: impl 'static + Fn(usize, SharedString, &ClickEvent, &mut Window, &mut App),
    on_row_quick_yaml: impl 'static + Fn(usize, SharedString, &ClickEvent, &mut Window, &mut App),
    on_row_context_menu: impl 'static
    + Fn(SharedString, &PolicyInfo, &MouseDownEvent, &mut Window, &mut App),
    on_refresh: impl 'static + Fn(&mut Window, &mut App),
) -> AnyElement {
    let columns = admission_webhook_columns();
    let filtered_rows: Rc<Vec<PolicyInfo>> = Rc::new(
        filter_and_sort_admission_webhooks(&state.policies, state.table.sort.as_ref())
            .into_iter()
            .cloned()
            .collect(),
    );
    let total_count = filtered_rows.len();
    let is_loading = state.status == AdmissionWebhooksLoadStatus::Loading;
    let presentation_mode = admission_webhooks_presentation_mode(state);
    let on_refresh_rc: super::ActionCallback = Rc::new(on_refresh);

    let count_label = format!("{total_count} admission webhooks");
    let header = render_resource_view_header(count_label, None, None, theme);

    let error_banner = if presentation_mode == AdmissionWebhooksPresentationMode::Table {
        state.error_message.as_ref().map(|message| {
            render_resource_error_banner(
                "admission-webhooks-retry-btn",
                format!("Failed to load admission webhooks: {message}"),
                is_loading,
                theme,
                on_refresh_rc.clone(),
            )
        })
    } else {
        None
    };

    let body = match presentation_mode {
        AdmissionWebhooksPresentationMode::ErrorCard => {
            let message = state
                .error_message
                .clone()
                .unwrap_or_else(|| "Unknown error".to_string());
            render_resource_error_state(
                "admission-webhooks-retry-btn",
                "Unable to Load Admission Webhooks",
                message,
                is_loading,
                theme,
                on_refresh_rc,
            )
        }
        AdmissionWebhooksPresentationMode::Table => {
            let mut table_state = state.table.clone();
            table_state.loading = is_loading && state.policies.is_empty();
            table_state.refreshing = is_loading && !state.policies.is_empty();
            table_state.empty_message = SharedString::from(admission_webhooks_empty_message(state));

            let key_rows = filtered_rows.clone();
            let cell_rows = filtered_rows.clone();
            let ctx_rows = filtered_rows.clone();
            let callbacks = DataTableCallbacks::new(
                move |idx| {
                    key_rows
                        .get(idx)
                        .map(|policy| format!("{}/{}", policy.kind, policy.name))
                        .unwrap_or_default()
                },
                move |idx, col, col_theme| {
                    cell_rows
                        .get(idx)
                        .map(|policy| render_admission_webhook_cell(idx, policy, col, col_theme))
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
                "admission-webhooks-table",
                &columns,
                total_count,
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

pub fn filter_events_for_admission_webhook<'a>(
    events: &'a [EventInfo],
    policy: &PolicyInfo,
) -> Vec<&'a EventInfo> {
    events
        .iter()
        .filter(|ev| ev.object_kind == policy.kind && ev.object_name == policy.name)
        .collect()
}

pub fn render_admission_webhook_drawer_overview(
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
        (
            "Kind",
            format!("{} ({})", admission_webhook_kind_label(policy), policy.kind),
            false,
        ),
        ("Failure Policy", policy.mode.clone(), true),
        ("Status", policy.status.clone(), false),
        ("Webhooks", policy.description.clone(), false),
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
        Some(render_drawer_section("WEBHOOK RULES (YAML)", block, theme))
    };

    div()
        .w_full()
        .flex()
        .flex_col()
        .gap_4()
        .child(render_drawer_section(
            "WEBHOOK INFORMATION",
            render_drawer_key_values(info_rows, theme),
            theme,
        ))
        .when_some(rules_preview, |el, section| el.child(section))
        .into_any_element()
}

pub fn render_admission_webhook_drawer_events(
    events: &[&EventInfo],
    loading: bool,
    theme: &OrbitTheme,
) -> AnyElement {
    render_drawer_events_list(
        events,
        loading,
        "No recent events for this admission webhook.",
        theme,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::components::data_table::{ColumnAlign, SortDirection};

    fn sample_webhook(name: &str, kind: &str, mode: &str, status: &str, age: &str) -> PolicyInfo {
        PolicyInfo {
            uid: format!("uid-{name}"),
            name: name.to_string(),
            r#type: "Admission Policy".to_string(),
            scope: "Cluster".to_string(),
            namespace: "-".to_string(),
            status: status.to_string(),
            mode: mode.to_string(),
            violations: 0,
            last_updated: age.to_string(),
            description: "2 webhook(s)".to_string(),
            rules: "[]\n".to_string(),
            kind: kind.to_string(),
        }
    }

    #[test]
    fn test_admission_webhook_columns_cover_required_fields() {
        let cols = admission_webhook_columns();
        let ids: Vec<&str> = cols.iter().map(|c| c.id.as_ref()).collect();
        assert_eq!(ids, vec!["name", "kind", "mode", "status", "rules", "age"]);
        let age = cols.iter().find(|c| c.id.as_ref() == "age");
        assert!(age.is_some_and(|c| c.align == ColumnAlign::Right && c.monospace));
    }

    #[test]
    fn test_admission_webhook_kind_label() {
        let mutating = sample_webhook(
            "mut-1",
            "MutatingWebhookConfiguration",
            "Fail",
            "Enforced",
            "1d",
        );
        let validating = sample_webhook(
            "val-1",
            "ValidatingWebhookConfiguration",
            "Ignore",
            "Audit",
            "2d",
        );
        let other = sample_webhook("net-1", "NetworkPolicy", "Ingress", "Enforced", "3d");

        assert_eq!(admission_webhook_kind_label(&mutating), "Mutating");
        assert_eq!(admission_webhook_kind_label(&validating), "Validating");
        assert_eq!(admission_webhook_kind_label(&other), "-");
    }

    #[test]
    fn test_filter_and_sort_admission_webhooks() {
        let policies = vec![
            sample_webhook(
                "z-val",
                "ValidatingWebhookConfiguration",
                "Fail",
                "Enforced",
                "10d",
            ),
            sample_webhook(
                "a-mut",
                "MutatingWebhookConfiguration",
                "Ignore",
                "Audit",
                "2h",
            ),
            sample_webhook("net-decoy", "NetworkPolicy", "Ingress", "Enforced", "1m"),
        ];

        let default_rows = filter_and_sort_admission_webhooks(&policies, None);
        let names: Vec<&str> = default_rows.iter().map(|p| p.name.as_str()).collect();
        assert_eq!(names, vec!["a-mut", "z-val"]);

        let by_kind_desc = filter_and_sort_admission_webhooks(
            &policies,
            Some(&SortState {
                column_id: "kind".to_string(),
                direction: SortDirection::Desc,
            }),
        );
        assert_eq!(admission_webhook_kind_label(by_kind_desc[0]), "Validating");
        assert_eq!(admission_webhook_kind_label(by_kind_desc[1]), "Mutating");

        let by_age = filter_and_sort_admission_webhooks(
            &policies,
            Some(&SortState {
                column_id: "age".to_string(),
                direction: SortDirection::Asc,
            }),
        );
        assert_eq!(by_age[0].name, "a-mut");
    }

    #[test]
    fn test_admission_webhooks_empty_message() {
        let mut state = AdmissionWebhooksState::default();
        assert_eq!(
            admission_webhooks_empty_message(&state),
            "No admission webhooks found in this cluster."
        );
        assert_eq!(
            admission_webhooks_presentation_mode(&state),
            AdmissionWebhooksPresentationMode::Table
        );

        state.status = AdmissionWebhooksLoadStatus::Error;
        assert_eq!(
            admission_webhooks_presentation_mode(&state),
            AdmissionWebhooksPresentationMode::ErrorCard
        );

        state.policies.push(sample_webhook(
            "val-1",
            "ValidatingWebhookConfiguration",
            "Fail",
            "Enforced",
            "1d",
        ));
        assert_eq!(
            admission_webhooks_presentation_mode(&state),
            AdmissionWebhooksPresentationMode::Table
        );
        assert_eq!(
            admission_webhooks_empty_message(&state),
            "No admission webhooks match the current filter."
        );
    }
}
