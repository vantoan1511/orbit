use gpui::{Font, FontFallbacks, FontFeatures, FontStyle, FontWeight, Pixels, SharedString, Styled, px};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TypographyScaleStep {
    pub font_size: Pixels,
    pub line_height: Pixels,
}

#[derive(Debug, Clone)]
pub struct TypographyTokens {
    pub ui_family: SharedString,
    pub ui_fallbacks: FontFallbacks,
    pub mono_family: SharedString,
    pub mono_fallbacks: FontFallbacks,
    pub size_3xs: TypographyScaleStep,
    pub size_2xs: TypographyScaleStep,
    pub size_xs: TypographyScaleStep,
    pub size_sm: TypographyScaleStep,
    pub size_base: TypographyScaleStep,
    pub size_lg: TypographyScaleStep,
    pub size_xl: TypographyScaleStep,
    pub control_height_sm: Pixels,
    pub control_height_md: Pixels,
    pub control_padding_x_sm: Pixels,
    pub control_padding_x_md: Pixels,
}

impl Default for TypographyTokens {
    fn default() -> Self {
        Self {
            ui_family: SharedString::from("Inter"),
            ui_fallbacks: FontFallbacks::from_fonts(vec![
                "Manrope".to_string(),
                ".SystemUIFont".to_string(),
            ]),
            mono_family: SharedString::from("Geist Mono"),
            mono_fallbacks: FontFallbacks::from_fonts(vec![
                "JetBrains Mono".to_string(),
                "Cascadia Code".to_string(),
                "Fira Code".to_string(),
                "Consolas".to_string(),
            ]),
            size_3xs: TypographyScaleStep {
                font_size: px(10.0),
                line_height: px(12.0),
            },
            size_2xs: TypographyScaleStep {
                font_size: px(11.0),
                line_height: px(14.0),
            },
            size_xs: TypographyScaleStep {
                font_size: px(12.0),
                line_height: px(16.0),
            },
            size_sm: TypographyScaleStep {
                font_size: px(13.0),
                line_height: px(18.0),
            },
            size_base: TypographyScaleStep {
                font_size: px(14.0),
                line_height: px(20.0),
            },
            size_lg: TypographyScaleStep {
                font_size: px(16.0),
                line_height: px(24.0),
            },
            size_xl: TypographyScaleStep {
                font_size: px(18.0),
                line_height: px(24.0),
            },
            control_height_sm: px(32.0),
            control_height_md: px(36.0),
            control_padding_x_sm: px(10.0),
            control_padding_x_md: px(12.0),
        }
    }
}

impl TypographyTokens {
    pub fn ui_font(&self, weight: FontWeight) -> Font {
        Font {
            family: self.ui_family.clone(),
            features: FontFeatures::default(),
            fallbacks: Some(self.ui_fallbacks.clone()),
            weight,
            style: FontStyle::Normal,
        }
    }

    pub fn mono_font(&self, weight: FontWeight) -> Font {
        Font {
            family: self.mono_family.clone(),
            features: FontFeatures::default(),
            fallbacks: Some(self.mono_fallbacks.clone()),
            weight,
            style: FontStyle::Normal,
        }
    }
}

pub fn apply_text_step<E: Styled>(el: E, step: TypographyScaleStep) -> E {
    el.text_size(step.font_size).line_height(step.line_height)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_typography_scale_and_font_fallback_stacks() {
        let t = TypographyTokens::default();
        assert_eq!(t.size_3xs, TypographyScaleStep { font_size: px(10.0), line_height: px(12.0) });
        assert_eq!(t.size_2xs, TypographyScaleStep { font_size: px(11.0), line_height: px(14.0) });
        assert_eq!(t.size_xs, TypographyScaleStep { font_size: px(12.0), line_height: px(16.0) });
        assert_eq!(t.size_sm, TypographyScaleStep { font_size: px(13.0), line_height: px(18.0) });
        assert_eq!(t.size_base, TypographyScaleStep { font_size: px(14.0), line_height: px(20.0) });
        assert_eq!(t.size_lg, TypographyScaleStep { font_size: px(16.0), line_height: px(24.0) });
        assert_eq!(t.size_xl, TypographyScaleStep { font_size: px(18.0), line_height: px(24.0) });

        assert_eq!(t.control_height_sm, px(32.0));
        assert_eq!(t.control_height_md, px(36.0));
        assert_eq!(t.control_padding_x_sm, px(10.0));
        assert_eq!(t.control_padding_x_md, px(12.0));

        assert_eq!(t.ui_family.as_ref(), "Inter");
        assert_eq!(t.mono_family.as_ref(), "Geist Mono");

        let ui = t.ui_font(FontWeight::MEDIUM);
        assert_eq!(ui.family.as_ref(), "Inter");
        assert_eq!(ui.weight, FontWeight::MEDIUM);
        let ui_fallbacks = ui.fallbacks.as_ref().map(|f| f.fallback_list()).unwrap_or_default();
        assert!(ui_fallbacks.iter().any(|f| f == "Manrope"));
        assert!(ui_fallbacks.iter().any(|f| f == ".SystemUIFont"));

        let mono = t.mono_font(FontWeight::NORMAL);
        assert_eq!(mono.family.as_ref(), "Geist Mono");
        let mono_fallbacks = mono.fallbacks.as_ref().map(|f| f.fallback_list()).unwrap_or_default();
        assert!(mono_fallbacks.iter().any(|f| f == "JetBrains Mono"));
        assert!(mono_fallbacks.iter().any(|f| f == "Consolas"));
    }
}
