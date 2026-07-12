//! Tab navigation and lifecycle menu items.

use crate::tui::menu::{MenuCommand, MenuItem};

pub(super) const TABS_MENU_ITEMS: &[MenuItem] = &[
    MenuItem {
        label: "Previous tab",
        shortcut: Some("Ctrl-PageUp"),
        command: MenuCommand::PreviousTab,
    },
    MenuItem {
        label: "Next tab",
        shortcut: Some("Ctrl-PageDown"),
        command: MenuCommand::NextTab,
    },
    MenuItem {
        label: "Close tab",
        shortcut: Some("Ctrl-F4"),
        command: MenuCommand::CloseTab,
    },
    MenuItem {
        label: "Open sidebar file as tab",
        shortcut: Some("Ctrl-B, Ctrl-Enter"),
        command: MenuCommand::HelpOnly,
    },
];
