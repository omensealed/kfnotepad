fn gui_pane_controls(
    state: &KfnotepadGui,
    pane: pane_grid::Pane,
    tile_id: GuiTileId,
    minimized: bool,
    is_maximized: bool,
) -> iced::widget::Row<'_, Message> {
    let minimize_icon = if minimized {
        ICON_RESTORE
    } else {
        ICON_MINIMIZE
    };
    let maximize_icon = if is_maximized {
        ICON_RESTORE
    } else {
        ICON_MAXIMIZE
    };
    let mut controls = row![
        gui_icon_tooltip_button(
            ICON_MOVE_LEFT,
            LABEL_MOVE_LEFT,
            Message::MovePane(pane, pane_grid::Direction::Left),
            LABEL_MOVE_LEFT,
            state.settings,
        ),
        gui_icon_tooltip_button(
            ICON_MOVE_RIGHT,
            LABEL_MOVE_RIGHT,
            Message::MovePane(pane, pane_grid::Direction::Right),
            LABEL_MOVE_RIGHT,
            state.settings,
        ),
        gui_icon_tooltip_button(
            ICON_MOVE_UP,
            LABEL_MOVE_UP,
            Message::MovePane(pane, pane_grid::Direction::Up),
            LABEL_MOVE_UP,
            state.settings,
        ),
        gui_icon_tooltip_button(
            ICON_MOVE_DOWN,
            LABEL_MOVE_DOWN,
            Message::MovePane(pane, pane_grid::Direction::Down),
            LABEL_MOVE_DOWN,
            state.settings,
        ),
        gui_icon_tooltip_button(
            minimize_icon,
            if minimized {
                LABEL_RESTORE
            } else {
                LABEL_MINIMIZE
            },
            Message::ToggleMinimizePane(pane),
            if minimized {
                LABEL_RESTORE
            } else {
                LABEL_MINIMIZE
            },
            state.settings,
        ),
        gui_icon_tooltip_button(
            maximize_icon,
            if is_maximized {
                LABEL_RESTORE
            } else {
                LABEL_MAXIMIZE
            },
            Message::ToggleMaximizePane(pane),
            if is_maximized {
                LABEL_RESTORE
            } else {
                LABEL_MAXIMIZE
            },
            state.settings,
        ),
        gui_icon_tooltip_button(
            ICON_CLOSE,
            LABEL_CLOSE_TILE,
            Message::ClosePane(pane),
            LABEL_CLOSE_TILE,
            state.settings,
        ),
    ]
    .spacing(GUI_TILE_CONTROL_SPACING);
    if state.is_external_edit_locked(tile_id) {
        controls = controls.push(gui_icon_tooltip_button(
            ICON_UNLOCK,
            LABEL_UNLOCK_EXTERNAL_EDIT,
            Message::UnlockExternalEdit(tile_id),
            LABEL_UNLOCK_EXTERNAL_EDIT,
            state.settings,
        ));
    }
    controls
}
