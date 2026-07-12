//! Static TUI menu item tables grouped by header menu.

#[path = "group_items/edit.rs"]
mod edit;
#[path = "group_items/file.rs"]
mod file;
#[path = "group_items/go.rs"]
mod go;
#[path = "group_items/help.rs"]
mod help;
#[path = "group_items/tabs.rs"]
mod tabs;
#[path = "group_items/view.rs"]
mod view;
#[path = "group_items/workspace.rs"]
mod workspace;

use super::{MenuGroup, MenuItem};
use edit::EDIT_MENU_ITEMS;
use file::FILE_MENU_ITEMS;
use go::GO_MENU_ITEMS;
use help::HELP_MENU_ITEMS;
use tabs::TABS_MENU_ITEMS;
use view::VIEW_MENU_ITEMS;
use workspace::WORKSPACE_MENU_ITEMS;

impl MenuGroup {
    pub(crate) fn items(self) -> &'static [MenuItem] {
        match self {
            Self::File => FILE_MENU_ITEMS,
            Self::Edit => EDIT_MENU_ITEMS,
            Self::View => VIEW_MENU_ITEMS,
            Self::Go => GO_MENU_ITEMS,
            Self::Tabs => TABS_MENU_ITEMS,
            Self::Workspace => WORKSPACE_MENU_ITEMS,
            Self::Help => HELP_MENU_ITEMS,
        }
    }
}
