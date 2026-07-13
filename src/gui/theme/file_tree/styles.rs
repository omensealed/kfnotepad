use super::*;

pub(in crate::gui::app::state) fn gui_file_tree_row_text_color(
    palette: iced::theme::Palette,
    selected: bool,
    error: bool,
) -> Color {
    if error {
        Color::from_rgb(0.55, 0.55, 0.55)
    } else if selected {
        palette.background
    } else {
        palette.text
    }
}

pub(in crate::gui::app::state) fn gui_file_tree_row_style(
    palette: iced::theme::Palette,
    selected: bool,
) -> container::Style {
    container::Style {
        text_color: Some(if selected {
            palette.background
        } else {
            palette.text
        }),
        background: selected.then_some(Background::Color(palette.primary)),
        border: Border {
            radius: 3.0.into(),
            ..Border::default()
        },
        ..container::Style::default()
    }
}

pub(in crate::gui::app::state) fn gui_file_tree_button_style(
    palette: iced::theme::Palette,
    selected: bool,
    status: iced::widget::button::Status,
) -> iced::widget::button::Style {
    let active_background = if selected {
        Some(Background::Color(palette.primary))
    } else {
        None
    };
    let mut style = iced::widget::button::Style {
        background: active_background,
        text_color: if selected {
            palette.background
        } else {
            palette.text
        },
        border: Border {
            radius: 3.0.into(),
            ..Border::default()
        },
        ..iced::widget::button::Style::default()
    };

    if matches!(
        status,
        iced::widget::button::Status::Hovered | iced::widget::button::Status::Pressed
    ) && !selected
    {
        style.background = Some(Background::Color(Color {
            a: 0.16,
            ..palette.primary
        }));
    }

    style
}
