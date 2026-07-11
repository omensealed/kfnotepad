pub(super) fn gui_menu_items(group: GuiMenuGroup) -> Vec<GuiMenuItem> {
    match group {
        GuiMenuGroup::File => file_menu_items(),
        GuiMenuGroup::Edit => edit_menu_items(),
        GuiMenuGroup::View => view_menu_items(),
        GuiMenuGroup::Go => go_menu_items(),
        GuiMenuGroup::Notes => notes_menu_items(),
        GuiMenuGroup::Tile => tile_menu_items(),
        GuiMenuGroup::Help => help_menu_items(),
    }
}
