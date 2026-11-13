use orbit_core::command::ResourceKind;
use crate::state::tabs::WorkspaceTab;
use crate::theme::modal_dialog_style;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AddClusterModalMode {
    FilePath { path: String },
    PasteYaml { yaml: String },
    ManualEndpoint {
        name: String,
        server_url: String,
        bearer_token: String,
        insecure_skip_tls_verify: bool,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ActiveModal {
    None,
    AddCluster {
        mode: AddClusterModalMode,
        error: Option<String>,
    },
    Scale {
        kind: ResourceKind,
        name: String,
        namespace: String,
        current_replicas: i32,
        target_replicas: i32,
    },
    PortForward {
        namespace: String,
        kind: String,
        name: String,
        local_port: u16,
        remote_port: u16,
    },
    DeleteConfirm {
        kind: ResourceKind,
        name: String,
        namespace: Option<String>,
    },
    SearchEverywhere {
        query: String,
        selected_index: usize,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SearchResultKind {
    Navigation,
    Resource(ResourceKind),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SearchResultItem {
    pub title: String,
    pub subtitle: String,
    pub kind: SearchResultKind,
    pub target_tab: Option<WorkspaceTab>,
}

pub fn modal_view<'a, Message: 'a + Clone>(
    modal: &'a ActiveModal,
    palette: &'a crate::theme::OrbitPalette,
    on_close: Message,
    on_search_input: impl Fn(String) -> Message + 'a,
    on_search_select: impl Fn(usize) -> Message + 'a,
    search_results: Vec<SearchResultItem>,
) -> Option<iced::Element<'a, Message>> {
    use iced::widget::{button, column, container, row, scrollable, text, text_input};
    use iced::Length;
    use crate::icons::{Icon, button_icon};
    use crate::theme::{FONT_UI_SEMIBOLD, Typography};

    match modal {
        ActiveModal::None => None,
        ActiveModal::SearchEverywhere { query, .. } => {
            let input = text_input("Search resources, tabs, commands (e.g. nginx, pods)...", query)
                .on_input(on_search_input)
                .size(Typography::SUBHEADING)
                .padding(10)
                .width(Length::Fill);

            let mut results_col = column![].spacing(4);
            for (idx, item) in search_results.into_iter().enumerate() {
                let row_item = button(
                    row![
                        text(item.title).size(Typography::BODY).color(palette.text_primary),
                        text(item.subtitle).size(Typography::CAPTION).color(palette.text_muted),
                    ]
                    .spacing(12)
                    .padding(6),
                )
                .on_press(on_search_select(idx))
                .width(Length::Fill);

                results_col = results_col.push(row_item);
            }

            let dialog = container(
                column![
                    row![
                        text("Search Everywhere (Ctrl+K)")
                            .size(Typography::SUBHEADING)
                            .font(FONT_UI_SEMIBOLD)
                            .color(palette.text_primary),
                        button(button_icon(Icon::X, 13)).on_press(on_close),
                    ]
                    .spacing(12),
                    input,
                    scrollable(results_col).height(Length::Fixed(280.0)),
                ]
                .spacing(12)
                .padding(16),
            )
            .style(modal_dialog_style(palette))
            .width(Length::Fixed(560.0));

            Some(
                container(dialog)
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .center_x(Length::Fill)
                    .center_y(Length::Fill)
                    .into(),
            )
        }
        ActiveModal::DeleteConfirm { kind, name, namespace } => {
            let desc = format!(
                "Are you sure you want to delete {:?} '{}'{}? This action cannot be undone.",
                kind,
                name,
                namespace.as_ref().map(|ns| format!(" in namespace '{}'", ns)).unwrap_or_default()
            );

            let dialog = container(
                column![
                    text("Confirm Resource Deletion")
                        .size(Typography::HEADING)
                        .font(FONT_UI_SEMIBOLD)
                        .color(palette.danger),
                    text(desc).size(Typography::BODY).color(palette.text_primary),
                    row![
                        button(text("Cancel").size(Typography::SECONDARY)).on_press(on_close),
                    ]
                    .spacing(12),
                ]
                .spacing(16)
                .padding(20),
            )
            .style(modal_dialog_style(palette))
            .width(Length::Fixed(440.0));

            Some(
                container(dialog)
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .center_x(Length::Fill)
                    .center_y(Length::Fill)
                    .into(),
            )
        }
        ActiveModal::Scale { kind, name, current_replicas, target_replicas, .. } => {
            let dialog = container(
                column![
                    text(format!("Scale {:?} '{}'", kind, name))
                        .size(Typography::HEADING)
                        .font(FONT_UI_SEMIBOLD)
                        .color(palette.text_primary),
                    text(format!("Current: {} replicas · Target: {} replicas", current_replicas, target_replicas))
                        .size(Typography::BODY)
                        .color(palette.text_secondary),
                    row![
                        button(text("Close").size(Typography::SECONDARY)).on_press(on_close),
                    ]
                    .spacing(12),
                ]
                .spacing(16)
                .padding(20),
            )
            .style(modal_dialog_style(palette))
            .width(Length::Fixed(440.0));

            Some(
                container(dialog)
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .center_x(Length::Fill)
                    .center_y(Length::Fill)
                    .into(),
            )
        }
        _ => {
            let dialog = container(
                column![
                    text("Action Modal")
                        .size(Typography::HEADING)
                        .font(FONT_UI_SEMIBOLD)
                        .color(palette.text_primary),
                    button(text("Close").size(Typography::SECONDARY)).on_press(on_close),
                ]
                .spacing(16)
                .padding(20),
            )
            .style(modal_dialog_style(palette))
            .width(Length::Fixed(400.0));

            Some(
                container(dialog)
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .center_x(Length::Fill)
                    .center_y(Length::Fill)
                    .into(),
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_modal_dialog_elevation_and_radius() {
        let dark = crate::theme::OrbitPalette::dark();
        let modal = modal_dialog_style(&dark)(&iced::Theme::Dark);
        assert_eq!(modal.border.radius, iced::border::Radius::from(8.0));
        assert_eq!(modal.shadow, dark.modal_shadow());
        assert_eq!(modal.background, Some(iced::Background::Color(dark.bg_panel)));
    }
}
