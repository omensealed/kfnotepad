use super::*;

pub(in crate::gui::app::state) fn file_menu_items() -> Vec<GuiMenuItem> {
    vec![
        GuiMenuItem {
            label: LABEL_NEW_TILE,
            command: GuiMenuCommand::NewTile,
        },
        GuiMenuItem {
            label: LABEL_OPEN,
            command: GuiMenuCommand::Open,
        },
        GuiMenuItem {
            label: LABEL_OPEN_PATH,
            command: GuiMenuCommand::OpenPath,
        },
        GuiMenuItem {
            label: LABEL_SAVE,
            command: GuiMenuCommand::Save,
        },
        GuiMenuItem {
            label: LABEL_SAVE_AS,
            command: GuiMenuCommand::SaveAs,
        },
        GuiMenuItem {
            label: LABEL_SAVE_AS_PATH,
            command: GuiMenuCommand::SaveAsPath,
        },
        GuiMenuItem {
            label: LABEL_CLOSE_TILE,
            command: GuiMenuCommand::ClosePane,
        },
        GuiMenuItem {
            label: LABEL_QUIT,
            command: GuiMenuCommand::Quit,
        },
    ]
}

pub(in crate::gui::app::state) fn edit_menu_items() -> Vec<GuiMenuItem> {
    vec![
        GuiMenuItem {
            label: LABEL_UNDO,
            command: GuiMenuCommand::Undo,
        },
        GuiMenuItem {
            label: LABEL_REDO,
            command: GuiMenuCommand::Redo,
        },
        GuiMenuItem {
            label: LABEL_COPY,
            command: GuiMenuCommand::Copy,
        },
        GuiMenuItem {
            label: LABEL_CUT,
            command: GuiMenuCommand::Cut,
        },
        GuiMenuItem {
            label: LABEL_PASTE,
            command: GuiMenuCommand::Paste,
        },
        GuiMenuItem {
            label: LABEL_SELECT_ALL,
            command: GuiMenuCommand::SelectAll,
        },
        GuiMenuItem {
            label: LABEL_FIND_NEXT,
            command: GuiMenuCommand::FindNext,
        },
        GuiMenuItem {
            label: LABEL_FIND_PREVIOUS,
            command: GuiMenuCommand::FindPrevious,
        },
    ]
}
