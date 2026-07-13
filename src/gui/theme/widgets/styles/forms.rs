use super::*;

pub(in crate::gui::app::state) fn gui_text_input_style(
    palette: iced::theme::Palette,
    _status: iced::widget::text_input::Status,
) -> iced::widget::text_input::Style {
    iced::widget::text_input::Style {
        background: palette.background.into(),
        border: iced::Border {
            color: Color {
                a: 0.65,
                ..palette.primary
            },
            width: 1.0,
            radius: 2.0.into(),
        },
        icon: palette.primary,
        placeholder: Color {
            a: 0.58,
            ..palette.text
        },
        value: palette.text,
        selection: Color {
            a: 0.7,
            ..palette.primary
        },
    }
}

pub(in crate::gui::app::state) fn gui_checkbox_style(
    palette: iced::theme::Palette,
    status: iced::widget::checkbox::Status,
) -> iced::widget::checkbox::Style {
    let checked = match status {
        iced::widget::checkbox::Status::Active { is_checked }
        | iced::widget::checkbox::Status::Hovered { is_checked }
        | iced::widget::checkbox::Status::Disabled { is_checked } => is_checked,
    };

    let background = if checked {
        palette.primary.into()
    } else {
        palette.background.into()
    };

    iced::widget::checkbox::Style {
        background,
        icon_color: palette.background,
        border: iced::Border {
            color: palette.primary,
            width: 1.0,
            radius: 2.0.into(),
        },
        text_color: Some(palette.text),
    }
}
