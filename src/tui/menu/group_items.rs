// Static TUI menu item tables.

include!("group_items/file.rs");
include!("group_items/edit.rs");
include!("group_items/view.rs");
include!("group_items/go.rs");
include!("group_items/tabs.rs");
include!("group_items/workspace.rs");
include!("group_items/help.rs");

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
