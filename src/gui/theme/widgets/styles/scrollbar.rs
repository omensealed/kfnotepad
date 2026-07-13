use super::*;

pub(in crate::gui::app::state) fn gui_scrollbar_track_style(
    palette: iced::theme::Palette,
    enabled: bool,
) -> container::Style {
    container::Style {
        background: Some(
            Color {
                a: if enabled { 0.12 } else { 0.04 },
                ..palette.primary
            }
            .into(),
        ),
        border: iced::Border {
            color: Color::TRANSPARENT,
            width: 0.0,
            radius: (GUI_EDITOR_SCROLLBAR_WIDTH / 2.0).into(),
        },
        ..container::Style::default()
    }
}

pub(in crate::gui::app::state) fn gui_scrollbar_thumb_style(
    palette: iced::theme::Palette,
    enabled: bool,
) -> container::Style {
    container::Style {
        background: Some(
            Color {
                a: if enabled { 0.78 } else { 0.24 },
                ..palette.primary
            }
            .into(),
        ),
        border: iced::Border {
            color: Color {
                a: if enabled { 0.82 } else { 0.28 },
                ..palette.primary
            },
            width: 1.0,
            radius: (GUI_EDITOR_SCROLLBAR_WIDTH / 2.0).into(),
        },
        ..container::Style::default()
    }
}
