const VIEW_MENU_ITEMS: &[MenuItem] = &[
    MenuItem {
        label: "Line numbers",
        shortcut: Some("Ctrl-L"),
        command: MenuCommand::ToggleLineNumbers,
    },
    MenuItem {
        label: "Theme",
        shortcut: Some("Ctrl-T"),
        command: MenuCommand::CycleTheme,
    },
    MenuItem {
        label: "Syntax theme",
        shortcut: Some("Ctrl-Shift-T"),
        command: MenuCommand::CycleSyntaxTheme,
    },
    MenuItem {
        label: "Reader mode",
        shortcut: Some("Ctrl-R"),
        command: MenuCommand::ToggleReaderMode,
    },
    MenuItem {
        label: "Reader slower",
        shortcut: None,
        command: MenuCommand::DecreaseReaderSpeed,
    },
    MenuItem {
        label: "Reader faster",
        shortcut: None,
        command: MenuCommand::IncreaseReaderSpeed,
    },
    MenuItem {
        label: "Word wrap",
        shortcut: Some("Ctrl-W"),
        command: MenuCommand::ToggleWrap,
    },
];
