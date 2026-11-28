use std::ops::Range;
use std::rc::Rc;

use gpui::{
    AnyElement, App, ElementId, FontWeight, HighlightStyle, MouseButton, Rgba, SharedString,
    Styled, StyledText, UniformListScrollHandle, Window, div, prelude::*, px, uniform_list,
};
use lucide_icons::Icon;

use crate::icons::render_icon;
use crate::theme::{OrbitTheme, apply_text_step};

pub const YAML_LINE_HEIGHT: f32 = 20.0;
pub const YAML_MIN_GUTTER_DIGITS: usize = 4;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum YamlTokenKind {
    Whitespace,
    Comment,
    Key,
    Punctuation,
    String,
    Number,
    BooleanOrNull,
    Plain,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct YamlToken {
    pub kind: YamlTokenKind,
    pub text: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct YamlHighlightSpan {
    pub range: Range<usize>,
    pub kind: YamlTokenKind,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CachedYamlLine {
    pub line_number: SharedString,
    pub text: SharedString,
    pub spans: Vec<YamlHighlightSpan>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CachedYamlDocument {
    revision: u64,
    gutter_digits: usize,
    lines: Vec<CachedYamlLine>,
}

impl CachedYamlDocument {
    pub fn from_yaml(revision: u64, content: &str) -> Self {
        let raw_lines: Vec<&str> = content.lines().collect();
        let gutter_digits = raw_lines
            .len()
            .max(1)
            .to_string()
            .len()
            .max(YAML_MIN_GUTTER_DIGITS);

        let lines = raw_lines
            .into_iter()
            .enumerate()
            .map(|(idx, line)| CachedYamlLine {
                line_number: SharedString::from(format!(
                    "{:>width$}",
                    idx + 1,
                    width = gutter_digits
                )),
                text: SharedString::from(line.to_string()),
                spans: tokenize_yaml_line_spans(line),
            })
            .collect();

        Self {
            revision,
            gutter_digits,
            lines,
        }
    }

    pub fn revision(&self) -> u64 {
        self.revision
    }

    pub fn line_count(&self) -> usize {
        self.lines.len()
    }

    pub fn is_empty(&self) -> bool {
        self.lines.is_empty()
    }

    pub fn gutter_digits(&self) -> usize {
        self.gutter_digits
    }

    pub fn gutter_width_px(&self) -> f32 {
        self.gutter_digits as f32 * 8.0 + 4.0
    }

    pub fn lines(&self) -> &[CachedYamlLine] {
        &self.lines
    }
}

#[derive(Debug, Default, Clone)]
pub struct YamlDocumentCache {
    cached: Option<Rc<CachedYamlDocument>>,
}

impl YamlDocumentCache {
    pub fn get_or_build(&mut self, revision: u64, content: &str) -> Rc<CachedYamlDocument> {
        if let Some(doc) = self.cached.as_ref()
            && doc.revision() == revision
        {
            return Rc::clone(doc);
        }
        let doc = Rc::new(CachedYamlDocument::from_yaml(revision, content));
        self.cached = Some(Rc::clone(&doc));
        doc
    }

    pub fn current(&self) -> Option<Rc<CachedYamlDocument>> {
        self.cached.as_ref().map(Rc::clone)
    }

    pub fn revision(&self) -> Option<u64> {
        self.cached.as_ref().map(|doc| doc.revision())
    }

    pub fn clear(&mut self) {
        self.cached = None;
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum YamlLoadStatus {
    #[default]
    Idle,
    Loading,
    Loaded,
    Error,
}

pub type YamlActionCallback = Rc<dyn Fn(&mut Window, &mut App)>;

fn push_span(spans: &mut Vec<YamlHighlightSpan>, kind: YamlTokenKind, start: usize, end: usize) {
    if start < end {
        spans.push(YamlHighlightSpan {
            range: start..end,
            kind,
        });
    }
}

fn is_yaml_number(s: &str) -> bool {
    let s = s.strip_prefix('-').unwrap_or(s);
    if s.is_empty() {
        return false;
    }
    let mut has_dot = false;
    let mut has_digit = false;
    for (idx, ch) in s.char_indices() {
        if ch.is_ascii_digit() {
            has_digit = true;
        } else if ch == '.' && !has_dot && idx > 0 && idx + 1 < s.len() {
            has_dot = true;
        } else {
            return false;
        }
    }
    has_digit
}

fn classify_scalar(word: &str) -> YamlTokenKind {
    match word {
        "true" | "false" | "null" | "~" => YamlTokenKind::BooleanOrNull,
        _ if is_yaml_number(word) => YamlTokenKind::Number,
        _ => YamlTokenKind::String,
    }
}

fn find_key_colon_index(s: &str) -> Option<usize> {
    if s.is_empty() || s.starts_with('#') || s.starts_with('{') || s.starts_with('[') {
        return None;
    }
    let mut in_single = false;
    let mut in_double = false;
    let mut escaped = false;
    let bytes = s.as_bytes();
    for (idx, ch) in s.char_indices() {
        if escaped {
            escaped = false;
            continue;
        }
        if in_double {
            if ch == '\\' {
                escaped = true;
            } else if ch == '"' {
                in_double = false;
            }
            continue;
        }
        if in_single {
            if ch == '\'' {
                in_single = false;
            }
            continue;
        }
        match ch {
            '"' if idx == 0 => in_double = true,
            '\'' if idx == 0 => in_single = true,
            '#' => return None,
            ':' => {
                let next_is_boundary =
                    idx + 1 == bytes.len() || bytes[idx + 1].is_ascii_whitespace();
                if next_is_boundary && idx > 0 {
                    return Some(idx);
                }
            }
            _ => {}
        }
    }
    None
}

fn tokenize_yaml_value_spans(line: &str, mut offset: usize, spans: &mut Vec<YamlHighlightSpan>) {
    while offset < line.len() {
        let rest = &line[offset..];
        let ws_len = rest
            .char_indices()
            .find(|(_, c)| !c.is_whitespace())
            .map(|(i, _)| i)
            .unwrap_or(rest.len());
        if ws_len > 0 {
            push_span(spans, YamlTokenKind::Whitespace, offset, offset + ws_len);
            offset += ws_len;
            continue;
        }

        if rest.starts_with('#') {
            push_span(spans, YamlTokenKind::Comment, offset, line.len());
            break;
        }

        let first = rest.chars().next().unwrap_or_default();
        if first == '"' || first == '\'' {
            let quote = first;
            let mut end = quote.len_utf8();
            let mut escaped = false;
            for (idx, ch) in rest[end..].char_indices() {
                if escaped {
                    escaped = false;
                    continue;
                }
                if quote == '"' && ch == '\\' {
                    escaped = true;
                    continue;
                }
                if ch == quote {
                    end += idx + ch.len_utf8();
                    break;
                }
            }
            if end == quote.len_utf8() {
                end = rest.len();
            }
            push_span(spans, YamlTokenKind::String, offset, offset + end);
            offset += end;
            continue;
        }

        if matches!(first, '{' | '}' | '[' | ']' | ',') {
            let len = first.len_utf8();
            push_span(spans, YamlTokenKind::Punctuation, offset, offset + len);
            offset += len;
            continue;
        }

        let word_end = rest
            .char_indices()
            .find(|(_, c)| c.is_whitespace() || matches!(c, '{' | '}' | '[' | ']' | ','))
            .map(|(i, _)| i)
            .unwrap_or(rest.len());
        let word = &rest[..word_end];
        push_span(spans, classify_scalar(word), offset, offset + word_end);
        offset += word_end;
    }
}

pub fn tokenize_yaml_line_spans(line: &str) -> Vec<YamlHighlightSpan> {
    let mut spans = Vec::new();
    if line.is_empty() {
        return spans;
    }

    let indent_len = line
        .char_indices()
        .find(|(_, c)| !c.is_whitespace())
        .map(|(i, _)| i)
        .unwrap_or(line.len());
    if indent_len > 0 {
        push_span(&mut spans, YamlTokenKind::Whitespace, 0, indent_len);
    }

    let mut offset = indent_len;
    if offset >= line.len() {
        return spans;
    }

    let mut rest = &line[offset..];
    if rest.starts_with('#') {
        push_span(&mut spans, YamlTokenKind::Comment, offset, line.len());
        return spans;
    }

    if let Some(after_dash) = rest.strip_prefix('-')
        && (after_dash.is_empty() || after_dash.starts_with(char::is_whitespace))
    {
        push_span(&mut spans, YamlTokenKind::Punctuation, offset, offset + 1);
        offset += 1;
        let ws_len = after_dash
            .char_indices()
            .find(|(_, c)| !c.is_whitespace())
            .map(|(i, _)| i)
            .unwrap_or(after_dash.len());
        if ws_len > 0 {
            push_span(
                &mut spans,
                YamlTokenKind::Whitespace,
                offset,
                offset + ws_len,
            );
            offset += ws_len;
        }
        if offset >= line.len() {
            return spans;
        }
        rest = &line[offset..];
    }

    if let Some(colon_idx) = find_key_colon_index(rest) {
        push_span(&mut spans, YamlTokenKind::Key, offset, offset + colon_idx);
        push_span(
            &mut spans,
            YamlTokenKind::Punctuation,
            offset + colon_idx,
            offset + colon_idx + 1,
        );
        tokenize_yaml_value_spans(line, offset + colon_idx + 1, &mut spans);
    } else {
        tokenize_yaml_value_spans(line, offset, &mut spans);
    }

    spans
}

pub fn tokenize_yaml_line(line: &str) -> Vec<YamlToken> {
    tokenize_yaml_line_spans(line)
        .into_iter()
        .map(|span| YamlToken {
            kind: span.kind,
            text: line[span.range].to_string(),
        })
        .collect()
}

pub fn yaml_token_color(kind: YamlTokenKind, theme: &OrbitTheme) -> Rgba {
    match kind {
        YamlTokenKind::Key => theme.accent.accent,
        YamlTokenKind::String => theme.semantic.success.solid,
        YamlTokenKind::Number => theme.semantic.warning.solid,
        YamlTokenKind::BooleanOrNull => theme.semantic.info.solid,
        YamlTokenKind::Comment | YamlTokenKind::Punctuation => theme.text.muted,
        YamlTokenKind::Whitespace | YamlTokenKind::Plain => theme.text.primary,
    }
}

pub fn build_line_highlights(
    spans: &[YamlHighlightSpan],
    theme: &OrbitTheme,
) -> Vec<(Range<usize>, HighlightStyle)> {
    spans
        .iter()
        .filter(|span| span.kind != YamlTokenKind::Whitespace && span.range.start < span.range.end)
        .map(|span| {
            (
                span.range.clone(),
                HighlightStyle {
                    color: Some(yaml_token_color(span.kind, theme).into()),
                    ..Default::default()
                },
            )
        })
        .collect()
}

pub struct YamlViewerProps<'a> {
    pub status: YamlLoadStatus,
    pub yaml_content: Option<&'a str>,
    pub cached_document: Option<Rc<CachedYamlDocument>>,
    pub scroll_handle: Option<UniformListScrollHandle>,
    pub error_message: Option<&'a str>,
    pub copied: bool,
    pub subtitle: SharedString,
    pub on_copy: Option<YamlActionCallback>,
    pub on_retry: Option<YamlActionCallback>,
}

pub fn render_yaml_viewer(props: YamlViewerProps<'_>, theme: &OrbitTheme) -> AnyElement {
    let YamlViewerProps {
        status,
        yaml_content,
        cached_document,
        scroll_handle,
        error_message,
        copied,
        subtitle,
        on_copy,
        on_retry,
    } = props;
    let has_content = yaml_content.is_some_and(|content| !content.is_empty())
        || cached_document.as_ref().is_some_and(|doc| !doc.is_empty());
    let can_copy = status == YamlLoadStatus::Loaded && has_content && on_copy.is_some();

    let copy_button = {
        let btn = theme
            .style_control(div())
            .id("yaml-viewer-copy-btn")
            .h(px(28.0))
            .px_2p5()
            .flex()
            .items_center()
            .gap_1p5()
            .when(!can_copy, |el| el.opacity(0.5))
            .when(can_copy, |el| {
                el.cursor_pointer().hover(|s| s.bg(theme.surface.bg_hover))
            })
            .child(render_icon(
                if copied { Icon::Check } else { Icon::Copy },
                px(13.0),
                if copied {
                    theme.semantic.success.solid
                } else {
                    theme.text.secondary
                },
            ))
            .child(
                apply_text_step(div(), theme.typography.size_xs)
                    .font_weight(FontWeight::MEDIUM)
                    .text_color(if copied {
                        theme.semantic.success.solid
                    } else {
                        theme.text.primary
                    })
                    .child(if copied { "Copied!" } else { "Copy YAML" }),
            );

        if let (true, Some(cb)) = (can_copy, on_copy) {
            btn.on_mouse_down(MouseButton::Left, move |_, window, cx| {
                cx.stop_propagation();
                cb(window, cx);
            })
        } else {
            btn
        }
    };

    let toolbar = div()
        .w_full()
        .px_4()
        .py_2p5()
        .border_b_1()
        .border_color(theme.border.default)
        .bg(theme.surface.bg_panel)
        .flex()
        .items_center()
        .justify_between()
        .gap_3()
        .child(
            div()
                .min_w_0()
                .flex()
                .flex_col()
                .gap_0p5()
                .child(
                    apply_text_step(div(), theme.typography.size_3xs)
                        .font_weight(FontWeight::SEMIBOLD)
                        .text_color(theme.text.muted)
                        .child("LIVE KUBERNETES MANIFEST"),
                )
                .child(
                    apply_text_step(div(), theme.typography.size_xs)
                        .text_color(theme.text.secondary)
                        .truncate()
                        .child(subtitle),
                ),
        )
        .child(copy_button);

    let body: AnyElement = match status {
        YamlLoadStatus::Loading => div()
            .flex_1()
            .p_6()
            .flex()
            .items_center()
            .justify_center()
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_2()
                    .child(render_icon(Icon::LoaderCircle, px(16.0), theme.text.muted))
                    .child(
                        apply_text_step(div(), theme.typography.size_sm)
                            .text_color(theme.text.secondary)
                            .child("Loading live YAML manifest..."),
                    ),
            )
            .into_any_element(),
        YamlLoadStatus::Error => {
            let msg = error_message
                .filter(|s| !s.trim().is_empty())
                .unwrap_or("Unable to load resource YAML.")
                .to_string();
            let retry_btn = {
                let btn = theme
                    .style_control(div())
                    .id("yaml-viewer-retry-btn")
                    .h(px(28.0))
                    .px_3()
                    .flex()
                    .items_center()
                    .gap_1p5()
                    .cursor_pointer()
                    .hover(|s| s.bg(theme.surface.bg_hover))
                    .child(render_icon(Icon::RefreshCw, px(13.0), theme.text.primary))
                    .child(
                        apply_text_step(div(), theme.typography.size_xs)
                            .font_weight(FontWeight::MEDIUM)
                            .text_color(theme.text.primary)
                            .child("Retry"),
                    );
                if let Some(cb) = on_retry {
                    btn.on_mouse_down(MouseButton::Left, move |_, window, cx| {
                        cx.stop_propagation();
                        cb(window, cx);
                    })
                } else {
                    btn
                }
            };

            div()
                .flex_1()
                .p_4()
                .child(
                    div()
                        .w_full()
                        .p_4()
                        .rounded(theme.style.radius_control)
                        .bg(theme.semantic.danger.soft)
                        .border_1()
                        .border_color(theme.semantic.danger.solid)
                        .flex()
                        .flex_col()
                        .gap_3()
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .gap_2()
                                .child(render_icon(
                                    Icon::TriangleAlert,
                                    px(15.0),
                                    theme.semantic.danger.solid,
                                ))
                                .child(
                                    apply_text_step(div(), theme.typography.size_sm)
                                        .font_weight(FontWeight::SEMIBOLD)
                                        .text_color(theme.semantic.danger.solid)
                                        .child("Failed to load YAML"),
                                ),
                        )
                        .child(
                            apply_text_step(div(), theme.typography.size_xs)
                                .text_color(theme.text.primary)
                                .child(msg),
                        )
                        .child(div().flex().justify_start().child(retry_btn)),
                )
                .into_any_element()
        }
        YamlLoadStatus::Idle => div()
            .flex_1()
            .p_6()
            .flex()
            .items_center()
            .justify_center()
            .child(
                apply_text_step(div(), theme.typography.size_sm)
                    .text_color(theme.text.muted)
                    .child("No YAML manifest loaded."),
            )
            .into_any_element(),
        YamlLoadStatus::Loaded => {
            let doc = cached_document.unwrap_or_else(|| {
                Rc::new(CachedYamlDocument::from_yaml(
                    0,
                    yaml_content.unwrap_or_default(),
                ))
            });
            let mono = theme.typography.mono_font(FontWeight::NORMAL);
            let gutter_width = doc.gutter_width_px();
            let line_count = doc.line_count();
            let theme_snapshot = theme.clone();

            uniform_list(
                ElementId::Name(SharedString::from("yaml-viewer-lines")),
                line_count,
                move |range: Range<usize>, _window: &mut Window, _cx: &mut App| {
                    let mut items = Vec::with_capacity(range.len());
                    let lines = doc.lines();
                    for idx in range {
                        let Some(line) = lines.get(idx) else {
                            continue;
                        };
                        let highlights = build_line_highlights(&line.spans, &theme_snapshot);
                        let row = div()
                            .h(px(YAML_LINE_HEIGHT))
                            .w_full()
                            .px_3()
                            .flex()
                            .items_center()
                            .gap_3()
                            .whitespace_nowrap()
                            .child(
                                apply_text_step(div(), theme_snapshot.typography.size_xs)
                                    .w(px(gutter_width))
                                    .flex_shrink_0()
                                    .font(mono.clone())
                                    .text_color(theme_snapshot.text.muted)
                                    .child(line.line_number.clone()),
                            )
                            .child(
                                apply_text_step(div(), theme_snapshot.typography.size_xs)
                                    .flex_1()
                                    .min_w_0()
                                    .overflow_x_hidden()
                                    .font(mono.clone())
                                    .text_color(theme_snapshot.text.primary)
                                    .child(
                                        StyledText::new(line.text.clone())
                                            .with_highlights(highlights),
                                    ),
                            );
                        items.push(row);
                    }
                    items
                },
            )
            .when_some(scroll_handle, |el, handle| el.track_scroll(&handle))
            .flex_1()
            .w_full()
            .min_h(px(0.0))
            .py_2()
            .bg(theme.surface.bg_app)
            .into_any_element()
        }
    };

    div()
        .w_full()
        .h_full()
        .min_h_0()
        .flex()
        .flex_col()
        .child(toolbar)
        .child(body)
        .into_any_element()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::theme::{AccentColor, SemanticColorsPreference, ThemeMode, UiStyle};
    use gpui::WindowAppearance;

    #[test]
    fn test_tokenize_yaml_line_classifies_keys_strings_numbers_booleans_and_comments() {
        let num_line = tokenize_yaml_line("  replicas: 3");
        assert_eq!(
            num_line,
            vec![
                YamlToken {
                    kind: YamlTokenKind::Whitespace,
                    text: "  ".to_string()
                },
                YamlToken {
                    kind: YamlTokenKind::Key,
                    text: "replicas".to_string()
                },
                YamlToken {
                    kind: YamlTokenKind::Punctuation,
                    text: ":".to_string()
                },
                YamlToken {
                    kind: YamlTokenKind::Whitespace,
                    text: " ".to_string()
                },
                YamlToken {
                    kind: YamlTokenKind::Number,
                    text: "3".to_string()
                },
            ]
        );

        let str_line = tokenize_yaml_line("  image: \"nginx:1.25\"");
        assert!(str_line.contains(&YamlToken {
            kind: YamlTokenKind::Key,
            text: "image".to_string()
        }));
        assert!(str_line.contains(&YamlToken {
            kind: YamlTokenKind::String,
            text: "\"nginx:1.25\"".to_string()
        }));

        let bool_line = tokenize_yaml_line("  suspend: false");
        assert!(bool_line.contains(&YamlToken {
            kind: YamlTokenKind::BooleanOrNull,
            text: "false".to_string()
        }));

        let list_comment_line = tokenize_yaml_line("  - name: web # sidecar");
        assert_eq!(
            list_comment_line,
            vec![
                YamlToken {
                    kind: YamlTokenKind::Whitespace,
                    text: "  ".to_string()
                },
                YamlToken {
                    kind: YamlTokenKind::Punctuation,
                    text: "-".to_string()
                },
                YamlToken {
                    kind: YamlTokenKind::Whitespace,
                    text: " ".to_string()
                },
                YamlToken {
                    kind: YamlTokenKind::Key,
                    text: "name".to_string()
                },
                YamlToken {
                    kind: YamlTokenKind::Punctuation,
                    text: ":".to_string()
                },
                YamlToken {
                    kind: YamlTokenKind::Whitespace,
                    text: " ".to_string()
                },
                YamlToken {
                    kind: YamlTokenKind::String,
                    text: "web".to_string()
                },
                YamlToken {
                    kind: YamlTokenKind::Whitespace,
                    text: " ".to_string()
                },
                YamlToken {
                    kind: YamlTokenKind::Comment,
                    text: "# sidecar".to_string()
                },
            ]
        );

        let comment_only = tokenize_yaml_line("  # full line comment");
        assert_eq!(
            comment_only,
            vec![
                YamlToken {
                    kind: YamlTokenKind::Whitespace,
                    text: "  ".to_string()
                },
                YamlToken {
                    kind: YamlTokenKind::Comment,
                    text: "# full line comment".to_string()
                },
            ]
        );
    }

    #[test]
    fn test_yaml_token_colors_map_to_orbit_theme_tokens() {
        for mode in [ThemeMode::Dark, ThemeMode::Light] {
            let appearance = match mode {
                ThemeMode::Dark => WindowAppearance::Dark,
                ThemeMode::Light | ThemeMode::System => WindowAppearance::Light,
            };
            let theme = OrbitTheme::resolve(
                mode,
                UiStyle::Flat,
                AccentColor::Blue,
                SemanticColorsPreference::default(),
                appearance,
            );

            assert_eq!(
                yaml_token_color(YamlTokenKind::Key, &theme),
                theme.accent.accent
            );
            assert_eq!(
                yaml_token_color(YamlTokenKind::String, &theme),
                theme.semantic.success.solid
            );
            assert_eq!(
                yaml_token_color(YamlTokenKind::Number, &theme),
                theme.semantic.warning.solid
            );
            assert_eq!(
                yaml_token_color(YamlTokenKind::BooleanOrNull, &theme),
                theme.semantic.info.solid
            );
            assert_eq!(
                yaml_token_color(YamlTokenKind::Comment, &theme),
                theme.text.muted
            );
            assert_eq!(
                yaml_token_color(YamlTokenKind::Punctuation, &theme),
                theme.text.muted
            );
            assert_eq!(
                yaml_token_color(YamlTokenKind::Plain, &theme),
                theme.text.primary
            );
        }
    }

    #[test]
    fn test_tokenize_yaml_line_spans_preserves_utf8_byte_ranges_and_reconstructs_line() {
        let samples = [
            "  replicas: 3",
            "  image: \"nginx:1.25\"",
            "  - name: web # sidecar",
            "  label: \"héllo-🚀\" # note",
            "  # full line comment with ünicode",
            "",
        ];

        for line in samples {
            let spans = tokenize_yaml_line_spans(line);
            let mut reconstructed = String::new();
            let mut prev_end = 0usize;

            for span in &spans {
                assert!(
                    span.range.start <= span.range.end,
                    "inverted range {:?} in {line:?}",
                    span.range
                );
                assert!(
                    span.range.start >= prev_end,
                    "overlapping range {:?} after {prev_end} in {line:?}",
                    span.range
                );
                assert!(
                    span.range.end <= line.len(),
                    "out-of-bounds range {:?} for len {} in {line:?}",
                    span.range,
                    line.len()
                );
                assert!(
                    line.is_char_boundary(span.range.start)
                        && line.is_char_boundary(span.range.end),
                    "non-UTF8 boundary {:?} in {line:?}",
                    span.range
                );
                reconstructed.push_str(&line[span.range.clone()]);
                prev_end = span.range.end;
            }

            assert_eq!(reconstructed, line);
        }

        let utf8_line = "  label: \"héllo-🚀\" # note";
        let utf8_spans = tokenize_yaml_line_spans(utf8_line);
        let extracted: Vec<(YamlTokenKind, &str)> = utf8_spans
            .iter()
            .map(|span| (span.kind, &utf8_line[span.range.clone()]))
            .collect();
        assert_eq!(
            extracted,
            vec![
                (YamlTokenKind::Whitespace, "  "),
                (YamlTokenKind::Key, "label"),
                (YamlTokenKind::Punctuation, ":"),
                (YamlTokenKind::Whitespace, " "),
                (YamlTokenKind::String, "\"héllo-🚀\""),
                (YamlTokenKind::Whitespace, " "),
                (YamlTokenKind::Comment, "# note"),
            ]
        );
    }

    #[test]
    fn test_cached_yaml_document_handles_large_manifests_and_dynamic_gutter() {
        let mut yaml = String::new();
        for idx in 0..1500 {
            if idx % 50 == 0 {
                yaml.push('\n');
            } else {
                yaml.push_str(&format!("  item_{idx}: \"value-{idx}\" # line {idx}\n"));
            }
        }

        let doc = CachedYamlDocument::from_yaml(7, &yaml);
        assert_eq!(doc.revision(), 7);
        assert_eq!(doc.line_count(), 1500);
        assert!(!doc.is_empty());
        assert!(doc.gutter_digits() >= YAML_MIN_GUTTER_DIGITS);
        assert_eq!(doc.gutter_digits(), 4);
        assert!(doc.gutter_width_px() >= 36.0);
        assert_eq!(doc.lines()[0].text.as_ref(), "");
        assert!(doc.lines()[0].spans.is_empty());
        assert_eq!(doc.lines()[0].line_number.as_ref(), "   1");
        assert_eq!(doc.lines()[1499].line_number.as_ref(), "1500");

        for line in doc.lines() {
            for span in &line.spans {
                assert!(span.range.end <= line.text.len());
            }
        }

        let large_yaml = "key: value\n".repeat(10_000);
        let large_doc = CachedYamlDocument::from_yaml(8, &large_yaml);
        assert_eq!(large_doc.line_count(), 10_000);
        assert_eq!(large_doc.gutter_digits(), 5);
        assert!(large_doc.gutter_width_px() > doc.gutter_width_px());
        assert_eq!(large_doc.lines()[9999].line_number.as_ref(), "10000");
    }

    #[test]
    fn test_yaml_document_cache_reuses_same_revision_and_invalidates_on_change() {
        let mut cache = YamlDocumentCache::default();
        assert!(cache.current().is_none());
        assert_eq!(cache.revision(), None);

        let first = cache.get_or_build(1, "kind: Deployment\nmetadata:\n  name: api\n");
        let second = cache.get_or_build(1, "kind: Deployment\nmetadata:\n  name: api\n");
        assert!(Rc::ptr_eq(&first, &second));
        assert_eq!(cache.revision(), Some(1));
        assert_eq!(first.line_count(), 3);

        let third = cache.get_or_build(2, "kind: Service\n");
        assert!(!Rc::ptr_eq(&first, &third));
        assert_eq!(third.revision(), 2);
        assert_eq!(cache.revision(), Some(2));
        assert_eq!(third.line_count(), 1);

        cache.clear();
        assert!(cache.current().is_none());
        assert_eq!(cache.revision(), None);
    }

    #[test]
    fn test_build_line_highlights_maps_spans_to_theme_colors() {
        let theme = OrbitTheme::resolve(
            ThemeMode::Dark,
            UiStyle::Flat,
            AccentColor::Blue,
            SemanticColorsPreference::default(),
            WindowAppearance::Dark,
        );
        let line = "  replicas: 3 # count";
        let spans = tokenize_yaml_line_spans(line);
        let highlights = build_line_highlights(&spans, &theme);

        // Whitespace spans should be skipped; non-whitespace spans should map to theme colors.
        assert_eq!(highlights.len(), 4);
        assert_eq!(
            highlights[0],
            (
                2..10,
                HighlightStyle {
                    color: Some(yaml_token_color(YamlTokenKind::Key, &theme).into()),
                    ..Default::default()
                }
            )
        );
        assert_eq!(
            highlights[1],
            (
                10..11,
                HighlightStyle {
                    color: Some(yaml_token_color(YamlTokenKind::Punctuation, &theme).into()),
                    ..Default::default()
                }
            )
        );
        assert_eq!(
            highlights[2],
            (
                12..13,
                HighlightStyle {
                    color: Some(yaml_token_color(YamlTokenKind::Number, &theme).into()),
                    ..Default::default()
                }
            )
        );
        assert_eq!(
            highlights[3],
            (
                14..21,
                HighlightStyle {
                    color: Some(yaml_token_color(YamlTokenKind::Comment, &theme).into()),
                    ..Default::default()
                }
            )
        );

        let mut prev_end = 0usize;
        for (range, _) in &highlights {
            assert!(range.start < range.end);
            assert!(range.start >= prev_end);
            prev_end = range.end;
        }
    }
}
