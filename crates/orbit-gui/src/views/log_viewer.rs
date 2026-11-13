use iced::{
    widget::{button, column, container, row, scrollable, text, text_input},
    Element, Length,
};

use crate::state::log_session::LogSessionState;
use crate::theme::{FONT_MONO, OrbitPalette, Typography};

#[derive(Debug, Clone)]
pub struct LogViewerAction {
    pub pause_toggle: bool,
    pub clear: bool,
    pub copy: bool,
    pub save_file: bool,
}

pub fn format_log_line(line: &str) -> (&str, bool) {
    let is_err = line.contains("[ERROR]") || line.contains("level=error") || line.contains("status: 500");
    (line, is_err)
}

pub fn log_viewer_view<'a, Message: 'a + Clone>(
    session: &'a LogSessionState,
    palette: &'a OrbitPalette,
    on_search: impl Fn(String) -> Message + 'a,
    on_toggle_pause: Message,
) -> Element<'a, Message> {
    let container_name = session.container.as_deref().unwrap_or("all containers");
    let toolbar = row![
        text(format!("Pod: {} ({})", session.pod, container_name))
            .size(Typography::BODY)
            .color(palette.text_primary),
        button(text(if session.is_paused { "▶ Resume" } else { "⏸ Pause" }).size(Typography::CAPTION))
            .on_press(on_toggle_pause),
        text_input("Filter logs...", &session.search_query)
            .on_input(on_search)
            .size(Typography::SECONDARY)
            .width(Length::Fixed(240.0)),
        text(format!("{} lines", session.len()))
            .size(Typography::CAPTION)
            .font(FONT_MONO)
            .color(palette.text_muted),
    ]
    .spacing(12)
    .padding(8);

    let mut log_lines_col = column![].spacing(1);
    for idx in session.filtered_lines(&session.search_query) {
        if let Some(line) = session.lines.get(idx) {
            let (formatted, is_err) = format_log_line(line);
            let color = if is_err { palette.danger } else { palette.text_primary };
            log_lines_col = log_lines_col.push(
                text(formatted.to_string())
                    .size(Typography::SECONDARY)
                    .font(FONT_MONO)
                    .color(color),
            );
        }
    }

    container(
        column![
            toolbar,
            scrollable(log_lines_col).width(Length::Fill).height(Length::Fill),
        ]
        .spacing(4)
        .padding(8),
    )
    .width(Length::Fill)
    .height(Length::Fill)
    .into()
}
