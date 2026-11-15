pub mod offline;
pub mod overview;
pub mod welcome;

use gpui::{AnyElement, FontWeight, div, prelude::*};

use crate::theme::{OrbitTheme, apply_text_step};

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
