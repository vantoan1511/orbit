pub mod configmaps;
pub mod cronjobs;
pub mod daemonsets;
pub mod deployments;
pub mod events;
pub mod ingresses;
pub mod jobs;
pub mod namespaces;
pub mod network_policies;
pub mod nodes;
pub mod offline;
pub mod overview;
pub mod persistent_volume_claims;
pub mod persistent_volumes;
pub mod pod_security;
pub mod pods;
pub mod replicasets;
pub mod resource_quotas;
pub mod secrets;
pub mod services;
pub mod settings;
pub mod statefulsets;
pub mod storage_overview;
pub mod welcome;
pub mod workloads_overview;

use std::collections::BTreeMap;
use std::rc::Rc;

use gpui::{AnyElement, App, FontWeight, Rgba, Window, div, prelude::*, px};
use lucide_icons::Icon;
use orbit_core::kubernetes::models::{ContainerImageInfo, EventInfo};

use crate::components::status_badge::{StatusBadgeVariant, render_status_badge_from_str};
use crate::icons::render_icon;
use crate::theme::{OrbitTheme, apply_text_step};

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

pub fn numeric_metric_text_color(theme: &OrbitTheme) -> Rgba {
    theme.text.primary
}

pub fn nonzero_metric_text_color(value: i32, theme: &OrbitTheme) -> Rgba {
    if value > 0 {
        theme.text.primary
    } else {
        theme.text.muted
    }
}

/// Canonical tabular monospace font for numeric/metric table cells (`FontWeight::NORMAL`, `tnum = 1`).
pub fn table_numeric_font(theme: &OrbitTheme) -> gpui::Font {
    theme.typography.mono_font(FontWeight::NORMAL)
}

/// Renders a right-aligned tabular numeric ratio cell (`current/total`) with primary numerator
/// ink and muted denominator ink.
pub fn render_ratio_metric_cell(
    current: impl Into<gpui::SharedString>,
    total: impl Into<gpui::SharedString>,
    theme: &OrbitTheme,
) -> AnyElement {
    let numeric_font = table_numeric_font(theme);
    let current = current.into();
    let total = total.into();

    apply_text_step(div(), theme.typography.size_xs)
        .w_full()
        .flex()
        .items_center()
        .justify_end()
        .child(
            div()
                .font(numeric_font.clone())
                .text_color(theme.text.primary)
                .child(current),
        )
        .child(
            div()
                .font(numeric_font)
                .text_color(theme.text.muted)
                .child(format!("/{total}")),
        )
        .into_any_element()
}

pub type ActionCallback = Rc<dyn Fn(&mut Window, &mut App)>;
pub type ValueCallback<T> = Rc<dyn Fn(T, &mut Window, &mut App)>;

pub fn render_action_button(
    id: &'static str,
    icon: Icon,
    label: &'static str,
    disabled: bool,
    theme: &OrbitTheme,
    on_click: ActionCallback,
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

pub fn render_detail_row(
    label: &'static str,
    value: String,
    mono: bool,
    theme: &OrbitTheme,
) -> AnyElement {
    let value_el = apply_text_step(div(), theme.typography.size_xs)
        .text_color(theme.text.primary)
        .when(mono, |el| {
            el.font(theme.typography.mono_font(FontWeight::NORMAL))
        })
        .when(!mono, |el| el.font_weight(FontWeight::MEDIUM));

    div()
        .flex()
        .items_center()
        .justify_between()
        .gap_4()
        .py_1p5()
        .border_b_1()
        .border_color(theme.border.default)
        .child(
            apply_text_step(div(), theme.typography.size_xs)
                .text_color(theme.text.muted)
                .child(label),
        )
        .child(value_el.child(value))
        .into_any_element()
}

pub fn render_drawer_section(
    title: &'static str,
    body: AnyElement,
    theme: &OrbitTheme,
) -> AnyElement {
    div()
        .w_full()
        .flex()
        .flex_col()
        .gap_2()
        .child(
            apply_text_step(div(), theme.typography.size_3xs)
                .font_weight(FontWeight::SEMIBOLD)
                .text_color(theme.text.muted)
                .child(title),
        )
        .child(body)
        .into_any_element()
}

pub fn render_drawer_key_values(
    rows: Vec<(&'static str, String, bool)>,
    theme: &OrbitTheme,
) -> AnyElement {
    div()
        .w_full()
        .px_3()
        .py_1()
        .rounded(theme.style.radius_control)
        .bg(theme.surface.bg_panel)
        .flex()
        .flex_col()
        .children(
            rows.into_iter()
                .map(|(label, value, mono)| render_detail_row(label, value, mono, theme)),
        )
        .into_any_element()
}

pub fn render_drawer_kv_badges(
    title: &'static str,
    map: &BTreeMap<String, String>,
    empty_text: &'static str,
    theme: &OrbitTheme,
) -> AnyElement {
    let mono = theme.typography.mono_font(FontWeight::NORMAL);
    let content = if map.is_empty() {
        apply_text_step(div(), theme.typography.size_xs)
            .text_color(theme.text.muted)
            .child(empty_text)
            .into_any_element()
    } else {
        div()
            .w_full()
            .flex()
            .flex_wrap()
            .gap_1p5()
            .children(map.iter().map(|(k, v)| {
                apply_text_step(div(), theme.typography.size_2xs)
                    .px_2()
                    .py_0p5()
                    .rounded(theme.style.radius_control)
                    .bg(theme.surface.control_bg)
                    .border_1()
                    .border_color(theme.border.default)
                    .font(mono.clone())
                    .text_color(theme.text.primary)
                    .child(format!("{k}={v}"))
            }))
            .into_any_element()
    };

    render_drawer_section(title, content, theme)
}

pub fn render_drawer_containers_section(
    containers: &[ContainerImageInfo],
    images: &[String],
    theme: &OrbitTheme,
) -> AnyElement {
    let mono = theme.typography.mono_font(FontWeight::NORMAL);
    let content = if containers.is_empty() && images.is_empty() {
        apply_text_step(div(), theme.typography.size_xs)
            .text_color(theme.text.muted)
            .child("No containers reported.")
            .into_any_element()
    } else if !containers.is_empty() {
        div()
            .w_full()
            .flex()
            .flex_col()
            .gap_2()
            .children(containers.iter().map(|c| {
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
            .children(images.iter().map(|img| {
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

    render_drawer_section("CONTAINERS & IMAGES", content, theme)
}

pub fn render_drawer_events_list(
    events: &[&EventInfo],
    loading: bool,
    empty_message: &'static str,
    theme: &OrbitTheme,
) -> AnyElement {
    if loading && events.is_empty() {
        return div()
            .w_full()
            .p_6()
            .flex()
            .items_center()
            .justify_center()
            .gap_2()
            .child(render_icon(Icon::LoaderCircle, px(15.0), theme.text.muted))
            .child(
                apply_text_step(div(), theme.typography.size_xs)
                    .text_color(theme.text.secondary)
                    .child("Loading cluster events..."),
            )
            .into_any_element();
    }

    if events.is_empty() {
        return div()
            .w_full()
            .p_6()
            .flex()
            .items_center()
            .justify_center()
            .child(
                apply_text_step(div(), theme.typography.size_xs)
                    .text_color(theme.text.muted)
                    .child(empty_message),
            )
            .into_any_element();
    }

    div()
        .w_full()
        .flex()
        .flex_col()
        .gap_2p5()
        .children(events.iter().map(|ev| {
            div()
                .w_full()
                .p_3()
                .rounded(theme.style.radius_control)
                .bg(theme.surface.bg_panel)
                .border_1()
                .border_color(theme.border.default)
                .flex()
                .flex_col()
                .gap_1p5()
                .child(
                    div()
                        .w_full()
                        .flex()
                        .items_center()
                        .justify_between()
                        .gap_2()
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .gap_2()
                                .child(render_status_badge_from_str(
                                    &ev.r#type,
                                    StatusBadgeVariant::Pill,
                                    theme,
                                ))
                                .child(
                                    apply_text_step(div(), theme.typography.size_xs)
                                        .font_weight(FontWeight::SEMIBOLD)
                                        .text_color(theme.text.primary)
                                        .child(ev.reason.clone()),
                                ),
                        )
                        .child(
                            apply_text_step(div(), theme.typography.size_2xs)
                                .text_color(theme.text.muted)
                                .child(format!("{} (x{})", ev.time, ev.count)),
                        ),
                )
                .child(
                    apply_text_step(div(), theme.typography.size_xs)
                        .text_color(theme.text.secondary)
                        .child(ev.message.clone()),
                )
                .child(
                    apply_text_step(div(), theme.typography.size_2xs)
                        .text_color(theme.text.muted)
                        .child(format!(
                            "{}/{} · Source: {}",
                            ev.object_kind, ev.object_name, ev.source
                        )),
                )
        }))
        .into_any_element()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::theme::{AccentColor, SemanticColorsPreference, ThemeMode, UiStyle};

    #[test]
    fn test_numeric_and_nonzero_metric_text_colors_are_neutral() {
        for (mode, appearance) in [
            (ThemeMode::Light, gpui::WindowAppearance::Light),
            (ThemeMode::Dark, gpui::WindowAppearance::Dark),
        ] {
            let theme = OrbitTheme::resolve(
                mode,
                UiStyle::Flat,
                AccentColor::Blue,
                SemanticColorsPreference::default(),
                appearance,
            );
            let primary = numeric_metric_text_color(&theme);
            assert_eq!(primary, theme.text.primary);
            assert_ne!(primary, theme.semantic.success.solid);
            assert_ne!(primary, theme.semantic.warning.solid);

            assert_eq!(nonzero_metric_text_color(0, &theme), theme.text.muted);
            assert_eq!(nonzero_metric_text_color(3, &theme), theme.text.primary);
            assert_ne!(
                nonzero_metric_text_color(3, &theme),
                theme.semantic.success.solid
            );
            assert_ne!(
                nonzero_metric_text_color(3, &theme),
                theme.semantic.warning.solid
            );
        }
    }

    #[test]
    fn test_table_numeric_font_uses_normal_weight_and_tabular_numerals() {
        for (mode, appearance) in [
            (ThemeMode::Light, gpui::WindowAppearance::Light),
            (ThemeMode::Dark, gpui::WindowAppearance::Dark),
        ] {
            let theme = OrbitTheme::resolve(
                mode,
                UiStyle::Flat,
                AccentColor::Blue,
                SemanticColorsPreference::default(),
                appearance,
            );
            let font = table_numeric_font(&theme);
            assert_eq!(font.weight, FontWeight::NORMAL);
            assert!(
                font.features
                    .tag_value_list()
                    .iter()
                    .any(|(tag, val)| tag == "tnum" && *val == 1)
            );
        }
    }

    #[test]
    fn test_all_resource_table_views_enforce_column_alignment_and_tabular_monospace_contract() {
        use crate::components::data_table::{ColumnAlign, resolve_table_cell_font};

        let theme = OrbitTheme::resolve(
            ThemeMode::Dark,
            UiStyle::Flat,
            AccentColor::Blue,
            SemanticColorsPreference::default(),
            gpui::WindowAppearance::Dark,
        );

        let numeric_or_temporal_ids = [
            "replicas",
            "desired_current",
            "ready",
            "up_to_date",
            "available",
            "restarts",
            "cpu",
            "memory",
            "pods",
            "keys_count",
            "size",
            "mounted_pods",
            "capacity",
            "completions",
            "duration",
            "active",
            "last_schedule",
            "count",
            "last_seen",
            "violations",
            "age",
        ];

        let all_views: [(&str, Vec<crate::components::data_table::TableColumn>); 19] = [
            ("configmaps", configmaps::configmap_columns()),
            ("cronjobs", cronjobs::cronjob_columns()),
            ("daemonsets", daemonsets::daemonset_columns()),
            ("deployments", deployments::deployment_columns()),
            ("events", events::event_columns()),
            ("ingresses", ingresses::ingress_columns()),
            ("jobs", jobs::job_columns()),
            ("namespaces", namespaces::namespace_columns()),
            ("network_policies", network_policies::network_policy_columns()),
            ("nodes", nodes::node_columns()),
            (
                "persistent_volume_claims",
                persistent_volume_claims::persistent_volume_claim_columns(),
            ),
            (
                "persistent_volumes",
                persistent_volumes::persistent_volume_columns(),
            ),
            ("pod_security", pod_security::pod_security_columns()),
            ("pods", pods::pod_columns()),
            ("replicasets", replicasets::replicaset_columns()),
            ("resource_quotas", resource_quotas::resource_quota_columns()),
            ("secrets", secrets::secret_columns()),
            ("services", services::service_columns()),
            ("statefulsets", statefulsets::statefulset_columns()),
        ];

        for (view_name, cols) in all_views {
            for col in &cols {
                let col_id = col.id.as_ref();
                if numeric_or_temporal_ids.contains(&col_id) {
                    assert_eq!(
                        col.align,
                        ColumnAlign::Right,
                        "{view_name}.{col_id} must be right-aligned"
                    );
                    assert!(
                        col.monospace,
                        "{view_name}.{col_id} must enable monospace tabular formatting"
                    );
                    let font = resolve_table_cell_font(col, &theme)
                        .expect("numeric/temporal column must resolve a cell font");
                    assert_eq!(font.weight, FontWeight::NORMAL);
                    assert!(
                        font.features
                            .tag_value_list()
                            .iter()
                            .any(|(tag, val)| tag == "tnum" && *val == 1),
                        "{view_name}.{col_id} must enable OpenType tnum=1"
                    );
                } else {
                    assert_eq!(
                        col.align,
                        ColumnAlign::Left,
                        "{view_name}.{col_id} must be left-aligned"
                    );
                }
            }
        }
    }
}
