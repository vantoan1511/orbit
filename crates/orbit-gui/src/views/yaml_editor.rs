use iced::{
    widget::{button, column, container, row, text},
    Element, Length,
};

use crate::state::yaml_editor::YamlEditorState;
use crate::theme::{FONT_MONO, FONT_UI_SEMIBOLD, OrbitPalette, Typography};

#[derive(Debug, Clone)]
pub enum YamlEditorAction {
    Apply,
    Discard,
    Format,
}

pub fn yaml_editor_view<'a, Message: 'a + Clone>(
    state: &'a YamlEditorState,
    palette: &'a OrbitPalette,
    on_apply: Message,
    on_discard: Message,
) -> Element<'a, Message> {
    let header = row![
        text(format!("Manifest: {}/{}", state.target_kind, state.target_name))
            .size(Typography::SUBHEADING)
            .font(FONT_UI_SEMIBOLD)
            .color(palette.text_primary),
        button(text("Apply Changes").size(Typography::SECONDARY)).on_press(on_apply),
        button(text("Discard").size(Typography::SECONDARY)).on_press(on_discard),
        text(if state.is_dirty { "● Modified" } else { "Clean" })
            .size(Typography::CAPTION)
            .color(if state.is_dirty { palette.warning } else { palette.text_muted }),
    ]
    .spacing(12)
    .padding(8);

    let status_bar = if let Some(ref err) = state.validation.error_message {
        container(text(format!("Syntax Error: {}", err)).size(Typography::CAPTION).color(palette.danger))
            .padding(6)
    } else {
        container(text("Valid YAML Manifest").size(Typography::CAPTION).color(palette.success))
            .padding(6)
    };

    let content_preview = container(
        text(&state.content)
            .size(Typography::SECONDARY)
            .font(FONT_MONO)
            .color(palette.text_primary),
    )
    .padding(12)
    .width(Length::Fill)
    .height(Length::Fill);

    container(
        column![
            header,
            status_bar,
            content_preview,
        ]
        .spacing(4)
        .padding(8),
    )
    .width(Length::Fill)
    .height(Length::Fill)
    .into()
}
