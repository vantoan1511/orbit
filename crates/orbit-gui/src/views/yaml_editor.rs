use iced::{
    widget::{button, column, container, row, text},
    Element, Length,
};

use crate::state::yaml_editor::YamlEditorState;
use crate::theme::OrbitPalette;

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
            .size(15)
            .color(palette.text_primary),
        button(text("Apply Changes").size(12)).on_press(on_apply),
        button(text("Discard").size(12)).on_press(on_discard),
        text(if state.is_dirty { "● Modified" } else { "Clean" })
            .size(11)
            .color(if state.is_dirty { palette.warning } else { palette.text_muted }),
    ]
    .spacing(12)
    .padding(8);

    let status_bar = if let Some(ref err) = state.validation.error_message {
        container(text(format!("Syntax Error: {}", err)).size(11).color(palette.danger))
            .padding(6)
    } else {
        container(text("Valid YAML Manifest").size(11).color(palette.success))
            .padding(6)
    };

    let content_preview = container(
        text(&state.content)
            .size(12)
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
