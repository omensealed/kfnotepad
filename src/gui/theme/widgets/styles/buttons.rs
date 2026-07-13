use super::*;

pub(in crate::gui::app::state) fn gui_chrome_button_style(
    palette: iced::theme::Palette,
    status: iced::widget::button::Status,
) -> iced::widget::button::Style {
    let mut style = iced::widget::button::Style {
        background: Some(palette.primary.into()),
        text_color: palette.background,
        border: iced::Border {
            color: Color::TRANSPARENT,
            width: 0.0,
            radius: 4.0.into(),
        },
        shadow: Shadow::default(),
        snap: true,
    };

    if matches!(
        status,
        iced::widget::button::Status::Hovered | iced::widget::button::Status::Pressed
    ) {
        style.background = Some(
            Color {
                a: 0.86,
                ..palette.primary
            }
            .into(),
        );
    }

    if matches!(status, iced::widget::button::Status::Disabled) {
        style.background = Some(
            Color {
                a: 0.35,
                ..palette.primary
            }
            .into(),
        );
        style.text_color = Color {
            a: 0.55,
            ..palette.background
        };
    }

    style
}
