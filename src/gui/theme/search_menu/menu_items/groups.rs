pub(super) fn gui_menu_groups() -> [GuiMenuGroup; 7] {
    [
        GuiMenuGroup::File,
        GuiMenuGroup::Edit,
        GuiMenuGroup::View,
        GuiMenuGroup::Go,
        GuiMenuGroup::Notes,
        GuiMenuGroup::Tile,
        GuiMenuGroup::Help,
    ]
}

pub(super) fn gui_menu_group_label(group: GuiMenuGroup) -> &'static str {
    match group {
        GuiMenuGroup::File => "File",
        GuiMenuGroup::Edit => "Edit",
        GuiMenuGroup::View => "View",
        GuiMenuGroup::Go => "Nav",
        GuiMenuGroup::Notes => "Notes",
        GuiMenuGroup::Tile => "Tile",
        GuiMenuGroup::Help => "Help",
    }
}
