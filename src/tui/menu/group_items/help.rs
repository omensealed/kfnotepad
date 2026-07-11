const HELP_MENU_ITEMS: &[MenuItem] = &[
    MenuItem {
        label: "Open help document",
        shortcut: Some("F10"),
        command: MenuCommand::OpenHelp,
    },
    MenuItem {
        label: "Command palette",
        shortcut: Some("F2"),
        command: MenuCommand::OpenCommandPalette,
    },
    MenuItem {
        label: "Files and tabs",
        shortcut: Some("Ctrl-B / Ctrl-Enter / Ctrl-F4"),
        command: MenuCommand::HelpOnly,
    },
    MenuItem {
        label: "Search and go",
        shortcut: Some("Ctrl-F / F3 / Shift-F3 / Ctrl-G"),
        command: MenuCommand::HelpOnly,
    },
    MenuItem {
        label: "Editing",
        shortcut: Some("Ctrl-Z/Y / Ctrl-K / Insert"),
        command: MenuCommand::HelpOnly,
    },
    MenuItem {
        label: "View and reader",
        shortcut: Some("Ctrl-L/T/R/W"),
        command: MenuCommand::HelpOnly,
    },
    MenuItem {
        label: "Workspaces",
        shortcut: Some("F10 -> Workspace"),
        command: MenuCommand::HelpOnly,
    },
    MenuItem {
        label: "Save and quit",
        shortcut: Some("Ctrl-S / Ctrl-Q"),
        command: MenuCommand::HelpOnly,
    },
];
