use iced::{
    widget::{column, container, row, text},
    Element, Length,
};

use crate::theme::{divider_style, OrbitPalette};

pub fn form_divider<'a, Message: 'a>(palette: &'a OrbitPalette) -> Element<'a, Message> {
    container(column![])
        .style(divider_style(palette))
        .width(Length::Fill)
        .height(Length::Fixed(1.0))
        .into()
}

pub fn form_row<'a, Message: 'a + Clone>(
    label: impl Into<String>,
    description: impl Into<String>,
    control: Element<'a, Message>,
    palette: &'a OrbitPalette,
) -> Element<'a, Message> {
    let desc_str = description.into();
    let mut text_col = column![
        text(label.into()).size(13).color(palette.text_primary),
    ]
    .spacing(2)
    .width(Length::FillPortion(3));

    if !desc_str.is_empty() {
        text_col = text_col.push(text(desc_str).size(11).color(palette.text_muted));
    }

    let control_col = container(control)
        .width(Length::FillPortion(2))
        .align_x(iced::alignment::Horizontal::Right);

    row![text_col, control_col]
        .spacing(12)
        .align_y(iced::Alignment::Center)
        .padding([10, 0])
        .into()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_form_row_layout_and_divider() {
        let palette = OrbitPalette::dark();
        let _divider: Element<'_, ()> = form_divider(&palette);
        let _row_with_desc: Element<'_, ()> = form_row(
            "Setting Name",
            "Helpful explanation of setting",
            text("control").into(),
            &palette,
        );
        let _row_no_desc: Element<'_, ()> = form_row(
            "Setting Name",
            "",
            text("control").into(),
            &palette,
        );
    }
}
