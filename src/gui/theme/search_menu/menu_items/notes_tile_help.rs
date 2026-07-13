use super::*;

pub(in crate::gui::app::state) fn notes_menu_items() -> Vec<GuiMenuItem> {
    vec![
        GuiMenuItem {
            label: LABEL_OPEN_NOTE,
            command: GuiMenuCommand::OpenManagedNote,
        },
        GuiMenuItem {
            label: LABEL_LIST_NOTES,
            command: GuiMenuCommand::ListManagedNotes,
        },
    ]
}

pub(in crate::gui::app::state) fn tile_menu_items() -> Vec<GuiMenuItem> {
    vec![
        GuiMenuItem {
            label: LABEL_MINIMIZE,
            command: GuiMenuCommand::ToggleMinimize,
        },
        GuiMenuItem {
            label: LABEL_MAXIMIZE,
            command: GuiMenuCommand::ToggleMaximize,
        },
        GuiMenuItem {
            label: LABEL_EQUALIZE_TILES,
            command: GuiMenuCommand::EqualizeTiles,
        },
        GuiMenuItem {
            label: LABEL_MOVE_LEFT,
            command: GuiMenuCommand::MoveLeft,
        },
        GuiMenuItem {
            label: LABEL_MOVE_RIGHT,
            command: GuiMenuCommand::MoveRight,
        },
        GuiMenuItem {
            label: LABEL_MOVE_UP,
            command: GuiMenuCommand::MoveUp,
        },
        GuiMenuItem {
            label: LABEL_MOVE_DOWN,
            command: GuiMenuCommand::MoveDown,
        },
    ]
}

pub(in crate::gui::app::state) fn help_menu_items() -> Vec<GuiMenuItem> {
    vec![GuiMenuItem {
        label: LABEL_OPEN_HELP,
        command: GuiMenuCommand::OpenHelp,
    }]
}
