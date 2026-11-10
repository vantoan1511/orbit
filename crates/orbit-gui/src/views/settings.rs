use std::collections::HashMap;
use iced::{
    widget::{button, column, container, row, scrollable, text},
    Element, Length,
};
use crate::theme::OrbitPalette;

#[derive(Debug, Clone, PartialEq, Default)]
pub struct SettingsViewState {
    pub values: HashMap<String, serde_json::Value>,
    pub is_modified: bool,
}

pub fn settings_view<'a, Message: 'a + Clone>(
    values: &'a HashMap<String, serde_json::Value>,
    palette: &'a OrbitPalette,
    on_reset: Message,
) -> Element<'a, Message> {
    let header = row![
        text("Orbit Application Preferences").size(16).color(palette.text_primary),
        button(text("Reset All").size(11)).on_press(on_reset),
    ]
    .spacing(16);

    let mut settings_list = column![].spacing(8);
    for (k, v) in values {
        let val_str = serde_json::to_string_pretty(v).unwrap_or_else(|_| v.to_string());
        settings_list = settings_list.push(
            row![
                container(text(k.clone()).size(12).color(palette.text_secondary)).width(Length::Fixed(200.0)),
                container(text(val_str).size(12).color(palette.text_primary)).width(Length::Fill),
            ]
            .spacing(12)
            .padding(4),
        );
    }

    container(
        column![
            header,
            scrollable(settings_list).width(Length::Fill).height(Length::Fill),
        ]
        .spacing(16)
        .padding(20),
    )
    .width(Length::Fill)
    .height(Length::Fill)
    .into()
}
