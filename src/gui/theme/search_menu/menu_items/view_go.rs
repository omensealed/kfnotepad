pub(super) fn view_menu_items() -> Vec<GuiMenuItem> {
    vec![
        GuiMenuItem {
            label: LABEL_FILES,
            command: GuiMenuCommand::ToggleBrowser,
        },
        GuiMenuItem {
            label: LABEL_THEME,
            command: GuiMenuCommand::CycleTheme,
        },
        GuiMenuItem {
            label: LABEL_SYNTAX_THEME,
            command: GuiMenuCommand::CycleSyntaxTheme,
        },
        GuiMenuItem {
            label: LABEL_READER_MODE,
            command: GuiMenuCommand::ToggleReaderMode,
        },
    ]
}

pub(super) fn go_menu_items() -> Vec<GuiMenuItem> {
    vec![
        GuiMenuItem {
            label: LABEL_GO_TO_LINE,
            command: GuiMenuCommand::GoToLine,
        },
        GuiMenuItem {
            label: LABEL_DOCUMENT_START,
            command: GuiMenuCommand::GoDocumentStart,
        },
        GuiMenuItem {
            label: LABEL_DOCUMENT_END,
            command: GuiMenuCommand::GoDocumentEnd,
        },
    ]
}
