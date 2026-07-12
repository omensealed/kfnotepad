//! Edit and search menu items.

use crate::tui::menu::{MenuCommand, MenuItem};

pub(super) const EDIT_MENU_ITEMS: &[MenuItem] = &[
    MenuItem {
        label: "Find",
        shortcut: Some("Ctrl-F"),
        command: MenuCommand::Find,
    },
    MenuItem {
        label: "Exact case",
        shortcut: Some("Ctrl-Shift-F"),
        command: MenuCommand::ToggleSearchCase,
    },
    MenuItem {
        label: "Undo",
        shortcut: Some("Ctrl-Z"),
        command: MenuCommand::Undo,
    },
    MenuItem {
        label: "Redo",
        shortcut: Some("Ctrl-Y"),
        command: MenuCommand::Redo,
    },
    MenuItem {
        label: "Delete previous word",
        shortcut: Some("Ctrl-Backspace"),
        command: MenuCommand::DeletePreviousWord,
    },
    MenuItem {
        label: "Delete next word",
        shortcut: Some("Ctrl-Delete"),
        command: MenuCommand::DeleteNextWord,
    },
    MenuItem {
        label: "Delete to line end",
        shortcut: Some("Ctrl-K"),
        command: MenuCommand::DeleteToLineEnd,
    },
    MenuItem {
        label: "Find next",
        shortcut: Some("F3"),
        command: MenuCommand::FindNext,
    },
    MenuItem {
        label: "Find previous",
        shortcut: Some("Shift-F3"),
        command: MenuCommand::FindPrevious,
    },
];
