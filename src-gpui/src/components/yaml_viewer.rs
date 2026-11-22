use std::rc::Rc;

use gpui::{
    AnyElement, App, FontWeight, MouseButton, Rgba, SharedString, StatefulInteractiveElement,
    Styled, Window, div, prelude::*, px,
};
use lucide_icons::Icon;

use crate::icons::render_icon;
use crate::theme::{OrbitTheme, apply_text_step};

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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum YamlLoadStatus {
    #[default]
    Idle,
    Loading,
    Loaded,
    Error,
}

pub type YamlActionCallback = Rc<dyn Fn(&mut Window, &mut App)>;

fn push_token(tokens: &mut Vec<YamlToken>, kind: YamlTokenKind, text: impl Into<String>) {
    let text = text.into();
    if !text.is_empty() {
        tokens.push(YamlToken { kind, text });
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

fn tokenize_yaml_value(mut rest: &str, tokens: &mut Vec<YamlToken>) {
    while !rest.is_empty() {
        let ws_len = rest
            .char_indices()
            .find(|(_, c)| !c.is_whitespace())
            .map(|(i, _)| i)
            .unwrap_or(rest.len());
        if ws_len > 0 {
            push_token(tokens, YamlTokenKind::Whitespace, &rest[..ws_len]);
            rest = &rest[ws_len..];
            continue;
        }

        if rest.starts_with('#') {
            push_token(tokens, YamlTokenKind::Comment, rest);
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
            push_token(tokens, YamlTokenKind::String, &rest[..end]);
            rest = &rest[end..];
            continue;
        }

        if matches!(first, '{' | '}' | '[' | ']' | ',') {
            let len = first.len_utf8();
            push_token(tokens, YamlTokenKind::Punctuation, &rest[..len]);
            rest = &rest[len..];
            continue;
        }

        let word_end = rest
            .char_indices()
            .find(|(_, c)| c.is_whitespace() || matches!(c, '{' | '}' | '[' | ']' | ','))
            .map(|(i, _)| i)
            .unwrap_or(rest.len());
        let word = &rest[..word_end];
        push_token(tokens, classify_scalar(word), word);
        rest = &rest[word_end..];
    }
}

pub fn tokenize_yaml_line(line: &str) -> Vec<YamlToken> {
    let mut tokens = Vec::new();
    if line.is_empty() {
        return tokens;
    }

    let indent_len = line
        .char_indices()
        .find(|(_, c)| !c.is_whitespace())
        .map(|(i, _)| i)
        .unwrap_or(line.len());
    if indent_len > 0 {
        push_token(&mut tokens, YamlTokenKind::Whitespace, &line[..indent_len]);
    }

    let mut rest = &line[indent_len..];
    if rest.is_empty() {
        return tokens;
    }

    if rest.starts_with('#') {
        push_token(&mut tokens, YamlTokenKind::Comment, rest);
        return tokens;
    }

    if let Some(after_dash) = rest.strip_prefix('-') {
        if after_dash.is_empty() || after_dash.starts_with(char::is_whitespace) {
            push_token(&mut tokens, YamlTokenKind::Punctuation, "-");
            let ws_len = after_dash
                .char_indices()
                .find(|(_, c)| !c.is_whitespace())
                .map(|(i, _)| i)
                .unwrap_or(after_dash.len());
            if ws_len > 0 {
                push_token(&mut tokens, YamlTokenKind::Whitespace, &after_dash[..ws_len]);
            }
            rest = &after_dash[ws_len..];
            if rest.is_empty() {
                return tokens;
            }
        }
    }

    if let Some(colon_idx) = find_key_colon_index(rest) {
        push_token(&mut tokens, YamlTokenKind::Key, &rest[..colon_idx]);
        push_token(&mut tokens, YamlTokenKind::Punctuation, ":");
        tokenize_yaml_value(&rest[colon_idx + 1..], &mut tokens);
    } else {
        tokenize_yaml_value(rest, &mut tokens);
    }

    tokens
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

pub struct YamlViewerProps<'a> {
    pub status: YamlLoadStatus,
    pub yaml_content: Option<&'a str>,
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
        error_message,
        copied,
        subtitle,
        on_copy,
        on_retry,
    } = props;
    let can_copy = status == YamlLoadStatus::Loaded
        && yaml_content.is_some_and(|content| !content.is_empty())
        && on_copy.is_some();

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
                el.cursor_pointer()
                    .hover(|s| s.bg(theme.surface.bg_hover))
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
            let content = yaml_content.unwrap_or_default();
            let mono = theme.typography.mono_font(FontWeight::NORMAL);
            let lines = content.lines().enumerate().map(|(idx, line)| {
                let line_number = format!("{:>4}", idx + 1);
                let tokens = tokenize_yaml_line(line);
                div()
                    .w_full()
                    .flex()
                    .items_start()
                    .gap_3()
                    .py(px(1.0))
                    .child(
                        apply_text_step(div(), theme.typography.size_xs)
                            .w(px(36.0))
                            .flex_shrink_0()
                            .font(mono.clone())
                            .text_color(theme.text.muted)
                            .child(line_number),
                    )
                    .child(
                        apply_text_step(div(), theme.typography.size_xs)
                            .flex()
                            .flex_wrap()
                            .min_w_0()
                            .font(mono.clone())
                            .children(tokens.into_iter().map(|tok| {
                                div()
                                    .text_color(yaml_token_color(tok.kind, theme))
                                    .child(tok.text)
                            })),
                    )
            });

            div()
                .id("yaml-viewer-scroll")
                .flex_1()
                .min_h_0()
                .overflow_y_scroll()
                .p_3()
                .bg(theme.surface.bg_app)
                .flex()
                .flex_col()
                .children(lines)
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
}
