const FILE_MENU_ITEMS: &[MenuItem] = &[
    MenuItem {
        label: "New file",
        shortcut: Some("Ctrl-N"),
        command: MenuCommand::NewFile,
    },
    MenuItem {
        label: "Save",
        shortcut: Some("Ctrl-S"),
        command: MenuCommand::Save,
    },
    MenuItem {
        label: "Files",
        shortcut: Some("Ctrl-B"),
        command: MenuCommand::ToggleSidebar,
    },
    MenuItem {
        label: "Quit",
        shortcut: Some("Ctrl-Q"),
        command: MenuCommand::Quit,
    },
];
