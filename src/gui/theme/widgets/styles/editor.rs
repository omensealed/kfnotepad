pub(super) fn gui_native_editor_style(
    palette: iced::theme::Palette,
    _status: text_editor::Status,
    search_highlight_active: bool,
) -> text_editor::Style {
    let selection = if search_highlight_active {
        Color {
            a: 0.95,
            ..palette.primary
        }
    } else {
        Color {
            a: 0.7,
            ..palette.primary
        }
    };

    text_editor::Style {
        background: palette.background.into(),
        border: iced::Border {
            color: Color::TRANSPARENT,
            width: 0.0,
            radius: 0.0.into(),
        },
        placeholder: Color {
            a: 0.42,
            ..palette.text
        },
        value: palette.text,
        selection,
    }
}
