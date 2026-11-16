pub mod deployments;
pub mod events;
pub mod namespaces;
pub mod nodes;
pub mod offline;
pub mod overview;
pub mod pods;
pub mod services;
pub mod welcome;

use std::collections::BTreeMap;
use std::rc::Rc;

use gpui::{AnyElement, App, FontWeight, Window, div, prelude::*, px};
use lucide_icons::Icon;
use orbit_core::kubernetes::models::EventInfo;

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

pub fn render_action_button(
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

pub fn render_detail_row(
    label: &'static str,
    value: String,
    mono: bool,
    theme: &OrbitTheme,
) -> AnyElement {
    let mut value_el = apply_text_step(div(), theme.typography.size_xs)
        .font_weight(FontWeight::MEDIUM)
        .text_color(theme.text.primary);
    if mono {
        value_el = value_el.font(theme.typography.mono_font(FontWeight::MEDIUM));
    }

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
