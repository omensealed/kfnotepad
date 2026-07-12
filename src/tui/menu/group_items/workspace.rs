//! Workspace persistence and restore menu items.

use crate::tui::menu::{MenuCommand, MenuItem};

pub(super) const WORKSPACE_MENU_ITEMS: &[MenuItem] = &[
    MenuItem {
        label: "Save current",
        shortcut: None,
        command: MenuCommand::SaveCurrentWorkspace,
    },
    MenuItem {
        label: "Save named",
        shortcut: None,
        command: MenuCommand::SaveNamedWorkspace,
    },
    MenuItem {
        label: "Manage projects",
        shortcut: None,
        command: MenuCommand::ListWorkspaces,
    },
    MenuItem {
        label: "Open project",
        shortcut: None,
        command: MenuCommand::OpenWorkspace,
    },
    MenuItem {
        label: "Delete project",
        shortcut: None,
        command: MenuCommand::DeleteWorkspace,
    },
    MenuItem {
        label: "Open current",
        shortcut: None,
        command: MenuCommand::OpenCurrentWorkspace,
    },
    MenuItem {
        label: "Restore last",
        shortcut: None,
        command: MenuCommand::ToggleRestoreLastWorkspace,
    },
];
