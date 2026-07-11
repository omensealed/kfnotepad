const GO_MENU_ITEMS: &[MenuItem] = &[
    MenuItem {
        label: "Page up",
        shortcut: Some("PageUp"),
        command: MenuCommand::PageUp,
    },
    MenuItem {
        label: "Page down",
        shortcut: Some("PageDown"),
        command: MenuCommand::PageDown,
    },
    MenuItem {
        label: "Top",
        shortcut: Some("Ctrl-Home"),
        command: MenuCommand::DocumentStart,
    },
    MenuItem {
        label: "Bottom",
        shortcut: Some("Ctrl-End"),
        command: MenuCommand::DocumentEnd,
    },
    MenuItem {
        label: "Go to line",
        shortcut: Some("Ctrl-G"),
        command: MenuCommand::GoToLine,
    },
    MenuItem {
        label: "Previous word",
        shortcut: Some("Ctrl-Left"),
        command: MenuCommand::PreviousWord,
    },
    MenuItem {
        label: "Next word",
        shortcut: Some("Ctrl-Right"),
        command: MenuCommand::NextWord,
    },
];
