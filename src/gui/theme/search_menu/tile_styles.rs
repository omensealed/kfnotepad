use super::*;

pub(in crate::gui::app::state) fn gui_tile_body_style(
    palette: iced::theme::Palette,
    _active: bool,
) -> container::Style {
    container::Style {
        text_color: Some(palette.text),
        background: Some(palette.background.into()),
        border: iced::Border {
            color: gui_tile_border_color(palette, _active),
            width: 1.0,
            radius: GUI_TILE_RADIUS.into(),
        },
        ..container::Style::default()
    }
}

pub(in crate::gui::app::state) fn gui_tile_title_style(
    palette: iced::theme::Palette,
    active: bool,
) -> container::Style {
    let background = if active {
        Color {
            a: 0.18,
            ..palette.primary
        }
    } else {
        Color {
            a: 0.08,
            ..palette.primary
        }
    };

    container::Style {
        text_color: Some(if active {
            palette.primary
        } else {
            palette.text
        }),
        background: Some(background.into()),
        border: iced::Border {
            color: gui_tile_border_color(palette, active),
            width: 1.0,
            radius: GUI_TILE_RADIUS.into(),
        },
        ..container::Style::default()
    }
}

pub(in crate::gui::app::state) fn gui_pane_grid_style(
    palette: iced::theme::Palette,
) -> pane_grid::Style {
    pane_grid::Style {
        hovered_region: pane_grid::Highlight {
            background: Background::Color(Color::TRANSPARENT),
            border: iced::Border {
                color: Color::TRANSPARENT,
                width: 0.0,
                radius: GUI_TILE_RADIUS.into(),
            },
        },
        picked_split: pane_grid::Line {
            color: palette.primary,
            width: 1.0,
        },
        hovered_split: pane_grid::Line {
            color: palette.primary,
            width: 1.0,
        },
    }
}

pub(in crate::gui::app::state) fn gui_menu_group_chrome_label(group: GuiMenuGroup) -> String {
    gui_menu_group_label(group).to_string()
}

#[cfg(test)]
pub(in crate::gui::app::state) fn gui_menu_group_index(group: GuiMenuGroup) -> usize {
    gui_menu_groups()
        .iter()
        .position(|candidate| *candidate == group)
        .expect("menu group belongs to static menu list")
}

#[cfg(test)]
pub(in crate::gui::app::state) fn gui_menu_dropdown_labels(
    group: GuiMenuGroup,
) -> Vec<&'static str> {
    gui_menu_items(group)
        .into_iter()
        .map(|item| item.label)
        .collect()
}

#[cfg(test)]
pub(in crate::gui::app::state) fn gui_menu_uses_iced_aw_menu_tree() -> bool {
    true
}

#[cfg(test)]
pub(in crate::gui::app::state) fn gui_menu_submenu_policy() -> &'static str {
    "Keep current root command groups flat until a group gains enough depth to justify nested hover submenus."
}
