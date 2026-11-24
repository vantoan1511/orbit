use std::cmp::Ordering;
use std::collections::BTreeSet;
use std::rc::Rc;

use gpui::{
    AnyElement, App, ClickEvent, ElementId, FontWeight, MouseButton, MouseDownEvent, Rgba,
    SharedString, Window, div, prelude::*, px,
};
use lucide_icons::Icon;
use orbit_core::kubernetes::configmaps::{format_size, parse_size_bytes};
use orbit_core::kubernetes::models::{EventInfo, SecretInfo};
use orbit_core::kubernetes::secrets::{DecodedSecretValue, decode_secret_value};

use crate::components::data_table::{
    DataTableCallbacks, SortState, TableColumn, render_data_table, sort_rows_stable,
};
use crate::components::{
    ContextMenuEntry, ContextMenuItem, SemanticStatus, StatusBadgeVariant, render_status_badge,
    render_status_badge_from_str,
};
use crate::icons::render_icon;
use crate::navigation::WorkspaceView;
use crate::state::{NamespaceFilterState, SecretsLoadStatus, SecretsState};
use crate::theme::{K8sResourceKind, OrbitTheme, apply_text_step};
pub use crate::views::{
    nonzero_metric_text_color, parse_age_seconds, render_action_button,
    render_resource_error_banner, render_resource_error_state, render_resource_view_header,
    render_truncated_cell,
};

pub const MASKED_SECRET_PLACEHOLDER: &str = "••••••••";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SecretsPresentationMode {
    Table,
    ErrorCard,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct SecretSummary {
    pub total: usize,
    pub total_bytes: u64,
    pub mounted: usize,
    pub orphaned: usize,
}

#[derive(Debug, Clone)]
pub struct SecretMetricCard {
    pub title: &'static str,
    pub primary_value: String,
    pub subtitle: String,
    pub accent_color: Rgba,
}

pub fn secret_columns() -> Vec<TableColumn> {
    vec![
        TableColumn::new("name", "Name").flex_width(1.6, px(150.0)),
        TableColumn::new("namespace", "Namespace").flex_width(1.0, px(105.0)),
        TableColumn::new("type", "Type").flex_width(1.2, px(130.0)),
        TableColumn::new("keys_count", "Keys")
            .fixed_width(px(72.0))
            .right_numeric(),
        TableColumn::new("size", "Size")
            .fixed_width(px(90.0))
            .right_numeric(),
        TableColumn::new("mounted_pods", "Mounted Pods")
            .fixed_width(px(112.0))
            .right_numeric(),
        TableColumn::new("immutable", "Immutable").fixed_width(px(100.0)),
        TableColumn::new("age", "Age")
            .fixed_width(px(72.0))
            .right_numeric(),
    ]
}

pub fn compare_secrets_by_col(a: &SecretInfo, b: &SecretInfo, col_id: &str) -> Ordering {
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
        "type" => a
            .r#type
            .to_ascii_lowercase()
            .cmp(&b.r#type.to_ascii_lowercase())
            .then_with(tie_break),
        "keys_count" => a.keys_count.cmp(&b.keys_count).then_with(tie_break),
        "size" => parse_size_bytes(&a.size)
            .cmp(&parse_size_bytes(&b.size))
            .then_with(tie_break),
        "mounted_pods" => a.mounted_pods.cmp(&b.mounted_pods).then_with(tie_break),
        "immutable" => a.immutable.cmp(&b.immutable).then_with(tie_break),
        "age" => parse_age_seconds(&a.age)
            .cmp(&parse_age_seconds(&b.age))
            .then_with(tie_break),
        _ => tie_break(),
    }
}

pub fn filter_and_sort_secrets<'a>(
    secrets: &'a [SecretInfo],
    namespaces: &NamespaceFilterState,
    sort: Option<&SortState>,
) -> Vec<&'a SecretInfo> {
    let mut rows: Vec<&'a SecretInfo> = secrets
        .iter()
        .filter(|sec| namespaces.matches_resource_namespace(Some(&sec.namespace)))
        .collect();
    sort_rows_stable(&mut rows, sort, |a, b, col_id| {
        compare_secrets_by_col(a, b, col_id)
    });
    rows
}

pub fn compute_secret_summary(secrets: &[&SecretInfo]) -> SecretSummary {
    let mut summary = SecretSummary {
        total: secrets.len(),
        ..Default::default()
    };

    for sec in secrets {
        summary.total_bytes = summary.total_bytes.saturating_add(parse_size_bytes(&sec.size));
        if sec.mounted_pods > 0 {
            summary.mounted += 1;
        } else {
            summary.orphaned += 1;
        }
    }

    summary
}

pub fn secret_summary_cards(
    secrets: &[&SecretInfo],
    namespaces: &NamespaceFilterState,
    theme: &OrbitTheme,
) -> Vec<SecretMetricCard> {
    let summary = compute_secret_summary(secrets);
    let pct = |count: usize, total: usize| -> f64 {
        if total == 0 {
            0.0
        } else {
            (count as f64 / total as f64) * 100.0
        }
    };

    let total_subtitle = if !namespaces.is_all_namespaces() {
        format!("Scope: {}", namespaces.summary_label())
    } else {
        "All namespaces".to_string()
    };

    vec![
        SecretMetricCard {
            title: "Total Secrets",
            primary_value: summary.total.to_string(),
            subtitle: total_subtitle,
            accent_color: theme.k8s_color(K8sResourceKind::Secret),
        },
        SecretMetricCard {
            title: "Total Size",
            primary_value: format_size(summary.total_bytes as usize),
            subtitle: "Total data footprint".to_string(),
            accent_color: theme.semantic.info.solid,
        },
        SecretMetricCard {
            title: "Mounted in Pods",
            primary_value: summary.mounted.to_string(),
            subtitle: format!("{:.0}% of secrets", pct(summary.mounted, summary.total)),
            accent_color: theme.semantic.success.solid,
        },
        SecretMetricCard {
            title: "Orphaned",
            primary_value: summary.orphaned.to_string(),
            subtitle: "Not referenced".to_string(),
            accent_color: theme.semantic.warning.solid,
        },
    ]
}

pub fn secrets_empty_message(state: &SecretsState) -> &'static str {
    if state.secrets.is_empty() {
        "No secrets found in this cluster."
    } else {
        "No secrets found in the selected namespace(s)."
    }
}

pub fn secrets_presentation_mode(state: &SecretsState) -> SecretsPresentationMode {
    if state.status == SecretsLoadStatus::Error && state.secrets.is_empty() {
        SecretsPresentationMode::ErrorCard
    } else {
        SecretsPresentationMode::Table
    }
}

/// Formats a Secret value for display in the drawer, keeping it masked unless `revealed` is true.
pub fn format_secret_entry_display(encoded: &str, revealed: bool) -> (String, bool) {
    if !revealed {
        return (MASKED_SECRET_PLACEHOLDER.to_string(), false);
    }
    match decode_secret_value(encoded) {
        DecodedSecretValue::Text(text) => (text, false),
        DecodedSecretValue::Binary { byte_len } => (format!("Binary data ({byte_len} B)"), true),
    }
}

fn render_secret_cell(
    row_idx: usize,
    sec: &SecretInfo,
    col: &TableColumn,
    theme: &OrbitTheme,
) -> AnyElement {
    match col.id.as_ref() {
        "name" => render_truncated_cell(
            ElementId::NamedInteger(SharedString::from("sec-name"), row_idx as u64),
            sec.name.clone(),
            Some(SharedString::from(sec.name.clone())),
            theme.text.primary,
            FontWeight::MEDIUM,
            false,
            theme,
        ),
        "namespace" => render_truncated_cell(
            ElementId::NamedInteger(SharedString::from("sec-ns"), row_idx as u64),
            sec.namespace.clone(),
            None,
            theme.text.secondary,
            FontWeight::NORMAL,
            false,
            theme,
        ),
        "type" => render_truncated_cell(
            ElementId::NamedInteger(SharedString::from("sec-type"), row_idx as u64),
            sec.r#type.clone(),
            Some(SharedString::from(sec.r#type.clone())),
            theme.text.secondary,
            FontWeight::NORMAL,
            true,
            theme,
        ),
        "keys_count" => apply_text_step(div(), theme.typography.size_xs)
            .font(theme.typography.mono_font(FontWeight::NORMAL))
            .text_color(if sec.keys_count == 0 {
                theme.text.muted
            } else {
                theme.text.primary
            })
            .child(sec.keys_count.to_string())
            .into_any_element(),
        "size" => apply_text_step(div(), theme.typography.size_xs)
            .font(theme.typography.mono_font(FontWeight::NORMAL))
            .text_color(theme.text.secondary)
            .child(sec.size.clone())
            .into_any_element(),
        "mounted_pods" => {
            let label = if sec.mounted_pods == 1 {
                "1 pod".to_string()
            } else {
                format!("{} pods", sec.mounted_pods)
            };
            apply_text_step(div(), theme.typography.size_xs)
                .font(theme.typography.mono_font(FontWeight::NORMAL))
                .text_color(nonzero_metric_text_color(sec.mounted_pods, theme))
                .child(label)
                .into_any_element()
        }
        "immutable" => {
            if sec.immutable {
                render_status_badge(
                    "Immutable",
                    SemanticStatus::Info,
                    StatusBadgeVariant::Inline,
                    theme,
                )
            } else {
                apply_text_step(div(), theme.typography.size_xs)
                    .text_color(theme.text.muted)
                    .child("Mutable")
                    .into_any_element()
            }
        }
        "age" => apply_text_step(div(), theme.typography.size_xs)
            .font(theme.typography.mono_font(FontWeight::NORMAL))
            .text_color(theme.text.muted)
            .child(sec.age.clone())
            .into_any_element(),
        _ => div().into_any_element(),
    }
}

fn render_summary_cards_row(
    secrets: &[&SecretInfo],
    namespaces: &NamespaceFilterState,
    theme: &OrbitTheme,
) -> AnyElement {
    let cards = secret_summary_cards(secrets, namespaces, theme);
    div()
        .grid()
        .grid_cols(4)
        .gap_3()
        .flex_shrink_0()
        .children(cards.into_iter().map(|card| {
            theme
                .style_card(div())
                .p_3()
                .flex()
                .flex_col()
                .gap_1()
                .child(
                    div()
                        .flex()
                        .items_center()
                        .justify_between()
                        .child(
                            apply_text_step(div(), theme.typography.size_xs)
                                .font_weight(FontWeight::MEDIUM)
                                .text_color(theme.text.muted)
                                .child(card.title),
                        )
                        .child(div().w(px(6.0)).h(px(6.0)).rounded_full().bg(card.accent_color)),
                )
                .child(
                    apply_text_step(div(), theme.typography.size_lg)
                        .font(theme.typography.tabular_ui_font(FontWeight::SEMIBOLD))
                        .text_color(theme.text.primary)
                        .child(card.primary_value),
                )
                .child(
                    apply_text_step(div(), theme.typography.size_xs)
                        .text_color(theme.text.secondary)
                        .truncate()
                        .child(card.subtitle),
                )
        }))
        .into_any_element()
}

pub fn build_secret_context_menu_entries(
    secret: &SecretInfo,
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
                format!("Filter by \"{}\"", secret.namespace),
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
            ContextMenuItem::new("delete-secret", "Delete Secret")
                .icon(Icon::Trash2)
                .danger(true)
                .disabled(true),
        ),
    ]
}

#[allow(clippy::too_many_arguments)]
pub fn render_secrets_view(
    state: &SecretsState,
    namespaces: &NamespaceFilterState,
    theme: &OrbitTheme,
    on_sort: impl 'static + Fn(SharedString, &ClickEvent, &mut Window, &mut App),
    on_row_click: impl 'static + Fn(usize, SharedString, &ClickEvent, &mut Window, &mut App),
    on_row_quick_yaml: impl 'static + Fn(usize, SharedString, &ClickEvent, &mut Window, &mut App),
    on_row_context_menu: impl 'static + Fn(SharedString, &SecretInfo, &MouseDownEvent, &mut Window, &mut App),
    on_refresh: impl 'static + Fn(&mut Window, &mut App),
) -> AnyElement {
    let columns = secret_columns();
    let filtered_refs =
        filter_and_sort_secrets(&state.secrets, namespaces, state.table.sort.as_ref());
    let summary_row = div()
        .w_full()
        .px_4()
        .pb_3()
        .flex_shrink_0()
        .child(render_summary_cards_row(&filtered_refs, namespaces, theme));
    let filtered_rows: Rc<Vec<SecretInfo>> = Rc::new(filtered_refs.into_iter().cloned().collect());
    let total_count = state.secrets.len();
    let visible_count = filtered_rows.len();
    let is_loading = state.status == SecretsLoadStatus::Loading;
    let on_refresh_rc: super::ActionCallback = Rc::new(on_refresh);

    let count_label = if namespaces.is_all_namespaces() {
        format!("{total_count} secrets")
    } else {
        format!("{visible_count} of {total_count} secrets")
    };

    let header = render_resource_view_header(
        WorkspaceView::Secrets.label(),
        WorkspaceView::Secrets.description(),
        count_label,
        Some(namespaces),
        None,
        theme,
    );

    let error_banner = state.error_message.as_ref().map(|message| {
        render_resource_error_banner(
            "secrets-retry-btn",
            format!("Failed to load secrets: {message}"),
            is_loading,
            theme,
            on_refresh_rc.clone(),
        )
    });

    let mode = secrets_presentation_mode(state);
    let body = match mode {
        SecretsPresentationMode::ErrorCard => {
            let message = state
                .error_message
                .clone()
                .unwrap_or_else(|| "Unknown error".to_string());
            render_resource_error_state(
                "secrets-retry-btn",
                "Unable to Load Secrets",
                message,
                is_loading,
                theme,
                on_refresh_rc,
            )
        }
        SecretsPresentationMode::Table => {
            let mut table_state = state.table.clone();
            table_state.loading = is_loading && state.secrets.is_empty();
            table_state.empty_message = SharedString::from(secrets_empty_message(state));

            let key_rows = filtered_rows.clone();
            let cell_rows = filtered_rows.clone();
            let ctx_rows = filtered_rows.clone();
            let callbacks = DataTableCallbacks::new(
                move |idx| {
                    key_rows
                        .get(idx)
                        .map(|s| format!("{}/{}", s.namespace, s.name))
                        .unwrap_or_default()
                },
                move |idx, col, col_theme| {
                    cell_rows
                        .get(idx)
                        .map(|s| render_secret_cell(idx, s, col, col_theme))
                        .unwrap_or_else(|| div().into_any_element())
                },
            )
            .on_sort(on_sort)
            .on_row_click(on_row_click)
            .on_row_quick_yaml(on_row_quick_yaml)
            .on_row_context_menu(move |idx, row_key, event, window, cx| {
                if let Some(sec) = ctx_rows.get(idx) {
                    on_row_context_menu(row_key, sec, event, window, cx);
                }
            });

            render_data_table(
                "secrets-table",
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
        .child(summary_row)
        .when(mode == SecretsPresentationMode::Table, |el| {
            el.when_some(error_banner, |el, banner| el.child(banner))
        })
        .child(body)
        .into_any_element()
}

pub fn filter_events_for_secret<'a>(
    events: &'a [EventInfo],
    secret: &SecretInfo,
) -> Vec<&'a EventInfo> {
    events
        .iter()
        .filter(|ev| {
            ev.namespace == secret.namespace
                && ((ev.object_kind == "Secret" && ev.object_name == secret.name)
                    || ev.message.contains(&secret.name))
        })
        .collect()
}

pub fn render_secret_drawer_overview(secret: &SecretInfo, theme: &OrbitTheme) -> AnyElement {
    let summary = crate::views::render_drawer_key_values(
        vec![
            ("Name", secret.name.clone(), true),
            ("Namespace", secret.namespace.clone(), true),
            ("Type", secret.r#type.clone(), true),
            ("Keys", secret.keys_count.to_string(), true),
            ("Size", secret.size.clone(), true),
            (
                "Immutable",
                if secret.immutable { "Yes" } else { "No" }.to_string(),
                false,
            ),
            ("Mounted Pods", secret.mounted_pods.to_string(), true),
            (
                "Resource Version",
                if secret.resource_version.is_empty() {
                    "-".to_string()
                } else {
                    secret.resource_version.clone()
                },
                true,
            ),
            ("Age", secret.age.clone(), true),
        ],
        theme,
    );

    let mono = theme.typography.mono_font(FontWeight::NORMAL);
    let used_by_section = {
        let content = if secret.used_by.is_empty() {
            apply_text_step(div(), theme.typography.size_xs)
                .text_color(theme.text.muted)
                .child("Not mounted by any pods.")
                .into_any_element()
        } else {
            div()
                .w_full()
                .flex()
                .flex_col()
                .gap_1p5()
                .children(secret.used_by.iter().map(|pod| {
                    div()
                        .w_full()
                        .px_2p5()
                        .py_1p5()
                        .rounded(theme.style.radius_control)
                        .bg(theme.surface.bg_panel)
                        .border_1()
                        .border_color(theme.border.default)
                        .flex()
                        .items_center()
                        .justify_between()
                        .gap_2()
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .gap_2()
                                .min_w(px(0.0))
                                .child(
                                    apply_text_step(div(), theme.typography.size_xs)
                                        .font(mono.clone())
                                        .font_weight(FontWeight::MEDIUM)
                                        .text_color(theme.text.primary)
                                        .truncate()
                                        .child(pod.name.clone()),
                                )
                                .child(render_status_badge_from_str(
                                    &pod.status,
                                    StatusBadgeVariant::Inline,
                                    theme,
                                )),
                        )
                        .child(
                            apply_text_step(div(), theme.typography.size_2xs)
                                .px_1p5()
                                .py_0p5()
                                .rounded(theme.style.radius_control)
                                .bg(theme.surface.control_bg)
                                .border_1()
                                .border_color(theme.border.default)
                                .font(mono.clone())
                                .text_color(theme.text.secondary)
                                .flex_shrink_0()
                                .child(pod.mount_type.clone()),
                        )
                }))
                .into_any_element()
        };
        crate::views::render_drawer_section("USED BY PODS", content, theme)
    };

    div()
        .id("secret-drawer-overview")
        .flex_1()
        .min_h_0()
        .overflow_y_scroll()
        .p_4()
        .flex()
        .flex_col()
        .gap_4()
        .child(crate::views::render_drawer_section("SUMMARY", summary, theme))
        .child(used_by_section)
        .child(crate::views::render_drawer_kv_badges(
            "LABELS",
            &secret.labels,
            "No labels",
            theme,
        ))
        .into_any_element()
}

fn render_secret_inline_button(
    id: SharedString,
    icon: Icon,
    label: &'static str,
    theme: &OrbitTheme,
    on_click: impl 'static + Fn(&mut Window, &mut App),
) -> AnyElement {
    let hover_bg = theme.surface.bg_hover;
    div()
        .id(id)
        .px_2()
        .py_0p5()
        .rounded(theme.style.radius_control)
        .bg(theme.surface.bg_panel)
        .border_1()
        .border_color(theme.border.default)
        .flex()
        .items_center()
        .gap_1()
        .cursor_pointer()
        .hover(move |s| s.bg(hover_bg))
        .on_mouse_down(MouseButton::Left, move |_, window, cx| {
            on_click(window, cx);
        })
        .child(render_icon(icon, px(12.0), theme.text.secondary))
        .child(
            apply_text_step(div(), theme.typography.size_2xs)
                .font_weight(FontWeight::MEDIUM)
                .text_color(theme.text.secondary)
                .child(label),
        )
        .into_any_element()
}

pub fn render_secret_drawer_data(
    secret: &SecretInfo,
    revealed_keys: &BTreeSet<String>,
    theme: &OrbitTheme,
    on_toggle_reveal: impl 'static + Fn(String, &mut Window, &mut App),
    on_copy_value: impl 'static + Fn(String, &mut Window, &mut App),
) -> AnyElement {
    let mono = theme.typography.mono_font(FontWeight::NORMAL);
    let on_toggle_rc: super::ValueCallback<String> = Rc::new(on_toggle_reveal);
    let on_copy_rc: super::ValueCallback<String> = Rc::new(on_copy_value);

    let content = if secret.data.is_empty() {
        apply_text_step(div(), theme.typography.size_xs)
            .text_color(theme.text.muted)
            .child("No data entries in this Secret.")
            .into_any_element()
    } else {
        div()
            .w_full()
            .flex()
            .flex_col()
            .gap_3()
            .children(secret.data.iter().enumerate().map(|(idx, (key, encoded))| {
                let is_revealed = revealed_keys.contains(key);
                let (display_value, is_binary) = format_secret_entry_display(encoded, is_revealed);
                let key_for_toggle = key.clone();
                let toggle_cb = on_toggle_rc.clone();
                let value_for_copy = if is_binary {
                    encoded.clone()
                } else {
                    display_value.clone()
                };
                let copy_cb = on_copy_rc.clone();

                div()
                    .w_full()
                    .rounded(theme.style.radius_control)
                    .bg(theme.surface.bg_panel)
                    .border_1()
                    .border_color(theme.border.default)
                    .flex()
                    .flex_col()
                    .overflow_hidden()
                    .child(
                        div()
                            .px_3()
                            .py_1p5()
                            .bg(theme.surface.control_bg)
                            .border_b_1()
                            .border_color(theme.border.default)
                            .flex()
                            .items_center()
                            .justify_between()
                            .gap_2()
                            .child(
                                apply_text_step(div(), theme.typography.size_xs)
                                    .font(mono.clone())
                                    .font_weight(FontWeight::SEMIBOLD)
                                    .text_color(theme.text.primary)
                                    .truncate()
                                    .child(key.clone()),
                            )
                            .child(
                                div()
                                    .flex()
                                    .items_center()
                                    .gap_2()
                                    .flex_shrink_0()
                                    .child(
                                        apply_text_step(div(), theme.typography.size_2xs)
                                            .font(mono.clone())
                                            .text_color(theme.text.muted)
                                            .child(format_size(encoded.len())),
                                    )
                                    .child(render_secret_inline_button(
                                        SharedString::from(format!("secret-reveal-{idx}")),
                                        if is_revealed {
                                            Icon::EyeOff
                                        } else {
                                            Icon::Eye
                                        },
                                        if is_revealed { "Hide" } else { "Reveal" },
                                        theme,
                                        move |window, cx| {
                                            toggle_cb(key_for_toggle.clone(), window, cx);
                                        },
                                    ))
                                    .when(is_revealed, |el| {
                                        el.child(render_secret_inline_button(
                                            SharedString::from(format!("secret-copy-{idx}")),
                                            Icon::Copy,
                                            "Copy",
                                            theme,
                                            move |window, cx| {
                                                copy_cb(value_for_copy.clone(), window, cx);
                                            },
                                        ))
                                    }),
                            ),
                    )
                    .child(
                        div().p_3().child(
                            apply_text_step(div(), theme.typography.size_xs)
                                .font(mono.clone())
                                .text_color(if !is_revealed || is_binary {
                                    theme.text.muted
                                } else {
                                    theme.text.secondary
                                })
                                .child(display_value),
                        ),
                    )
            }))
            .into_any_element()
    };

    div()
        .id("secret-drawer-data")
        .flex_1()
        .min_h_0()
        .overflow_y_scroll()
        .p_4()
        .child(crate::views::render_drawer_section(
            "DATA ENTRIES",
            content,
            theme,
        ))
        .into_any_element()
}

pub fn render_secret_drawer_events(
    events: &[EventInfo],
    loading: bool,
    secret: &SecretInfo,
    theme: &OrbitTheme,
) -> AnyElement {
    let matched = filter_events_for_secret(events, secret);
    div()
        .id("secret-drawer-events")
        .flex_1()
        .min_h_0()
        .overflow_y_scroll()
        .p_4()
        .child(crate::views::render_drawer_events_list(
            &matched,
            loading,
            "No events found for this secret.",
            theme,
        ))
        .into_any_element()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::components::data_table::{ColumnAlign, SortDirection};
    use std::collections::BTreeMap;

    fn make_namespace_filter_prod() -> NamespaceFilterState {
        let mut ns = NamespaceFilterState::default();
        ns.begin_load_for_cluster("ctx-a");
        ns.apply_loaded(
            "ctx-a",
            1,
            vec![
                orbit_core::kubernetes::models::NamespaceInfo {
                    name: "prod".to_string(),
                    status: "Active".to_string(),
                    age: "1d".to_string(),
                    is_system: false,
                    labels: BTreeMap::new(),
                    annotations: BTreeMap::new(),
                    uid: "ns-prod".to_string(),
                    created: String::new(),
                },
                orbit_core::kubernetes::models::NamespaceInfo {
                    name: "dev".to_string(),
                    status: "Active".to_string(),
                    age: "1d".to_string(),
                    is_system: false,
                    labels: BTreeMap::new(),
                    annotations: BTreeMap::new(),
                    uid: "ns-dev".to_string(),
                    created: String::new(),
                },
            ],
        );
        ns.toggle_namespace("prod");
        ns
    }

    #[allow(clippy::too_many_arguments)]
    fn sample_secret(
        name: &str,
        namespace: &str,
        secret_type: &str,
        keys_count: i32,
        size: &str,
        mounted_pods: i32,
        immutable: bool,
        age: &str,
    ) -> SecretInfo {
        SecretInfo {
            name: name.to_string(),
            namespace: namespace.to_string(),
            labels: BTreeMap::new(),
            annotations: 0,
            created: String::new(),
            age: age.to_string(),
            resource_version: "1".to_string(),
            r#type: secret_type.to_string(),
            immutable,
            keys_count,
            size: size.to_string(),
            mounted_pods,
            used_by: Vec::new(),
            data: BTreeMap::new(),
        }
    }

    #[test]
    fn test_secret_columns_cover_required_fields_and_alignment() {
        let cols = secret_columns();
        let ids: Vec<&str> = cols.iter().map(|c| c.id.as_ref()).collect();
        assert_eq!(
            ids,
            vec![
                "name",
                "namespace",
                "type",
                "keys_count",
                "size",
                "mounted_pods",
                "immutable",
                "age",
            ]
        );

        for right_col in ["keys_count", "size", "mounted_pods", "age"] {
            let col = cols.iter().find(|c| c.id.as_ref() == right_col).unwrap();
            assert_eq!(col.align, ColumnAlign::Right, "{right_col} must be right-aligned");
            assert!(col.monospace, "{right_col} must be monospace");
        }
    }

    #[test]
    fn test_filter_and_sort_secrets_respects_namespaces_type_and_numeric_columns() {
        let secrets = vec![
            sample_secret("tls-cert", "prod", "kubernetes.io/tls", 2, "4.0 KiB", 2, true, "5d"),
            sample_secret("db-pass", "prod", "Opaque", 3, "512 B", 0, false, "12m"),
            sample_secret("reg-cred", "staging", "kubernetes.io/dockerconfigjson", 1, "2.0 KiB", 1, false, "2h"),
        ];

        let ns_filter = make_namespace_filter_prod();

        let sort_size_desc = SortState {
            column_id: "size".to_string(),
            direction: SortDirection::Desc,
        };
        let filtered = filter_and_sort_secrets(&secrets, &ns_filter, Some(&sort_size_desc));
        assert_eq!(filtered.len(), 2);
        assert_eq!(filtered[0].name, "tls-cert");
        assert_eq!(filtered[1].name, "db-pass");

        let sort_type_asc = SortState {
            column_id: "type".to_string(),
            direction: SortDirection::Asc,
        };
        let all_ns = NamespaceFilterState::default();
        let by_type = filter_and_sort_secrets(&secrets, &all_ns, Some(&sort_type_asc));
        assert_eq!(by_type[0].name, "reg-cred");
        assert_eq!(by_type[1].name, "tls-cert");
        assert_eq!(by_type[2].name, "db-pass");

        let sort_keys = SortState {
            column_id: "keys_count".to_string(),
            direction: SortDirection::Desc,
        };
        let by_keys = filter_and_sort_secrets(&secrets, &all_ns, Some(&sort_keys));
        assert_eq!(by_keys[0].name, "db-pass");
        assert_eq!(by_keys[2].name, "reg-cred");

        let sort_pods = SortState {
            column_id: "mounted_pods".to_string(),
            direction: SortDirection::Desc,
        };
        let by_pods = filter_and_sort_secrets(&secrets, &all_ns, Some(&sort_pods));
        assert_eq!(by_pods[0].name, "tls-cert");
        assert_eq!(by_pods[2].name, "db-pass");

        let sort_imm = SortState {
            column_id: "immutable".to_string(),
            direction: SortDirection::Desc,
        };
        let by_imm = filter_and_sort_secrets(&secrets, &all_ns, Some(&sort_imm));
        assert_eq!(by_imm[0].name, "tls-cert");

        let sort_age = SortState {
            column_id: "age".to_string(),
            direction: SortDirection::Asc,
        };
        let by_age = filter_and_sort_secrets(&secrets, &all_ns, Some(&sort_age));
        assert_eq!(by_age[0].name, "db-pass");
        assert_eq!(by_age[2].name, "tls-cert");
    }

    #[test]
    fn test_secret_summary_cards_scope_to_filtered_namespace() {
        let theme = OrbitTheme::from_config(
            &orbit_core::config::OrbitConfig::default(),
            gpui::WindowAppearance::Dark,
        );
        let secrets = vec![
            sample_secret("s1", "prod", "Opaque", 1, "1.0 KiB", 2, false, "1h"),
            sample_secret("s2", "prod", "Opaque", 2, "512 B", 0, true, "2h"),
            sample_secret("s3", "dev", "Opaque", 1, "256 B", 1, false, "3h"),
        ];

        let ns = make_namespace_filter_prod();
        let scoped = filter_and_sort_secrets(&secrets, &ns, None);
        let cards = secret_summary_cards(&scoped, &ns, &theme);

        assert_eq!(cards.len(), 4);
        assert_eq!(cards[0].title, "Total Secrets");
        assert_eq!(cards[0].primary_value, "2");
        assert_eq!(cards[0].subtitle, "Scope: prod");
        assert_eq!(cards[1].title, "Total Size");
        assert_eq!(cards[1].primary_value, "1.5 KiB");
        assert_eq!(cards[2].title, "Mounted in Pods");
        assert_eq!(cards[2].primary_value, "1");
        assert_eq!(cards[2].subtitle, "50% of secrets");
        assert_eq!(cards[3].title, "Orphaned");
        assert_eq!(cards[3].primary_value, "1");
    }

    #[test]
    fn test_secrets_presentation_mode_and_empty_messages() {
        let mut state = SecretsState::default();
        assert_eq!(secrets_presentation_mode(&state), SecretsPresentationMode::Table);
        assert_eq!(
            secrets_empty_message(&state),
            "No secrets found in this cluster."
        );

        state.status = SecretsLoadStatus::Error;
        state.error_message = Some("forbidden".to_string());
        assert_eq!(
            secrets_presentation_mode(&state),
            SecretsPresentationMode::ErrorCard
        );

        // Stale rows present -> keep Table mode (with banner)
        state
            .secrets
            .push(sample_secret("s1", "default", "Opaque", 1, "10 B", 0, false, "1m"));
        assert_eq!(secrets_presentation_mode(&state), SecretsPresentationMode::Table);
        assert_eq!(
            secrets_empty_message(&state),
            "No secrets found in the selected namespace(s)."
        );
    }

    #[test]
    fn test_secret_data_display_masks_by_default_and_decodes_when_revealed() {
        let (masked, is_bin_masked) = format_secret_entry_display("c2VjcmV0", false);
        assert_eq!(masked, MASKED_SECRET_PLACEHOLDER);
        assert!(!is_bin_masked);

        let (decoded, is_bin_text) = format_secret_entry_display("c2VjcmV0", true);
        assert_eq!(decoded, "secret");
        assert!(!is_bin_text);

        // Base64 of [0xff, 0xef, 0x00, 0x01] (non-UTF-8 4 bytes)
        let binary_b64 = "/+8AAQ==";
        let (bin_masked, _) = format_secret_entry_display(binary_b64, false);
        assert_eq!(bin_masked, MASKED_SECRET_PLACEHOLDER);

        let (bin_revealed, is_bin) = format_secret_entry_display(binary_b64, true);
        assert_eq!(bin_revealed, "Binary data (4 B)");
        assert!(is_bin);
    }

    #[test]
    fn test_build_secret_context_menu_entries_and_event_matching() {
        let sec = sample_secret("db-creds", "prod", "Opaque", 2, "64 B", 1, false, "1h");
        let entries = build_secret_context_menu_entries(
            &sec,
            |_, _| {},
            |_, _| {},
            |_, _| {},
            |_, _| {},
            |_, _| {},
        );
        let item_ids: Vec<&str> = entries
            .iter()
            .filter_map(|e| match e {
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
                "delete-secret",
            ]
        );
        let delete_item = entries
            .iter()
            .find_map(|e| match e {
                ContextMenuEntry::Item(item) if item.id.as_ref() == "delete-secret" => Some(item),
                _ => None,
            })
            .unwrap();
        assert!(delete_item.disabled);
        assert!(delete_item.danger);

        let events = vec![
            EventInfo {
                uid: "1".to_string(),
                r#type: "Normal".to_string(),
                reason: "Updated".to_string(),
                message: "Secret updated".to_string(),
                object_kind: "Secret".to_string(),
                object_name: "db-creds".to_string(),
                namespace: "prod".to_string(),
                count: 1,
                time: "1m".to_string(),
                first_seen: String::new(),
                last_seen: "1m".to_string(),
                source: String::new(),
                labels: BTreeMap::new(),
                annotations: BTreeMap::new(),
            },
            EventInfo {
                uid: "2".to_string(),
                r#type: "Normal".to_string(),
                reason: "Updated".to_string(),
                message: "Secret updated".to_string(),
                object_kind: "Secret".to_string(),
                object_name: "db-creds".to_string(),
                namespace: "staging".to_string(),
                count: 1,
                time: "1m".to_string(),
                first_seen: String::new(),
                last_seen: "1m".to_string(),
                source: String::new(),
                labels: BTreeMap::new(),
                annotations: BTreeMap::new(),
            },
        ];
        let matched = filter_events_for_secret(&events, &sec);
        assert_eq!(matched.len(), 1);
        assert_eq!(matched[0].uid, "1");
    }
}
