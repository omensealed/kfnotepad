use super::*;

pub(in crate::gui::app::state) fn gui_menu_panel_style(
    palette: iced::theme::Palette,
) -> iced_aw::style::menu_bar::Style {
    iced_aw::style::menu_bar::Style {
        bar_background: palette.background.into(),
        bar_border: iced::Border {
            color: palette.background,
            width: 0.0,
            radius: 0.0.into(),
        },
        bar_shadow: Shadow::default(),
        menu_background: palette.background.into(),
        menu_border: iced::Border {
            color: palette.primary,
            width: 1.0,
            radius: GUI_MENU_DROPDOWN_RADIUS.into(),
        },
        menu_shadow: Shadow {
            color: Color {
                a: 0.35,
                ..Color::BLACK
            },
            offset: Vector::new(0.0, 6.0),
            blur_radius: 16.0,
        },
        path: palette.primary.into(),
        path_border: iced::Border {
            color: palette.primary,
            width: 1.0,
            radius: GUI_MENU_ITEM_RADIUS.into(),
        },
    }
}

pub(in crate::gui::app::state) fn gui_menu_item_button_style(
    palette: iced::theme::Palette,
    status: iced::widget::button::Status,
) -> iced::widget::button::Style {
    let mut style = iced::widget::button::Style {
        background: Some(palette.background.into()),
        text_color: palette.text,
        border: iced::Border {
            color: palette.primary,
            width: 0.0,
            radius: GUI_MENU_ITEM_RADIUS.into(),
        },
        shadow: Shadow::default(),
        snap: true,
    };

    match status {
        iced::widget::button::Status::Active => style,
        iced::widget::button::Status::Hovered | iced::widget::button::Status::Pressed => {
            style.background = Some(palette.primary.into());
            style.text_color = palette.background;
            style.border.width = 1.0;
            style
        }
        iced::widget::button::Status::Disabled => {
            style.text_color = Color {
                a: 0.45,
                ..palette.text
            };
            style
        }
    }
}

pub(in crate::gui::app::state) fn gui_menu_root_style(
    palette: iced::theme::Palette,
) -> container::Style {
    container::Style {
        text_color: Some(palette.text),
        background: None,
        border: iced::Border {
            color: Color::TRANSPARENT,
            width: 0.0,
            radius: 0.0.into(),
        },
        ..container::Style::default()
    }
}
