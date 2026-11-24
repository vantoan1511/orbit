use gpui::{
    Context, FontWeight, IntoElement, Render, SharedString, Styled, Window, div, prelude::*, px,
};

use crate::theme::{OrbitTheme, apply_text_step};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TextTooltip {
    pub text: SharedString,
    pub monospace: bool,
}

impl TextTooltip {
    pub fn new(text: impl Into<SharedString>) -> Self {
        Self {
            text: text.into(),
            monospace: false,
        }
    }

    pub fn monospace(mut self, monospace: bool) -> Self {
        self.monospace = monospace;
        self
    }
}

impl Render for TextTooltip {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = OrbitTheme::get(cx);
        let content = apply_text_step(div(), theme.typography.size_2xs)
            .text_color(theme.text.primary)
            .when(self.monospace, |el| {
                el.font(theme.typography.mono_font(FontWeight::NORMAL))
            })
            .child(self.text.clone());

        theme
            .style_overlay(div())
            .px_2()
            .py_1()
            .max_w(px(480.0))
            .child(content)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_text_tooltip_builder_sets_text_and_monospace_flag() {
        let plain = TextTooltip::new("View YAML");
        assert_eq!(plain.text.as_ref(), "View YAML");
        assert!(!plain.monospace);

        let full_refs = "registry.k8s.io/ingress-nginx/controller:v1.15.1@sha256:d94ceea78b0123456789abcdef\nregistry.k8s.io/pause@sha256:abcdef0123456789";
        let mono = TextTooltip::new(full_refs).monospace(true);
        assert_eq!(mono.text.as_ref(), full_refs);
        assert!(mono.monospace);
    }
}
