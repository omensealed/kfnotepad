//! Menu-group, menu-command, and tab-strip mouse hit testing.

use super::*;

pub(crate) fn menu_group_at_mouse(column: u16, frame: RenderFrame) -> Option<MenuGroup> {
    if !show_menu_bar(frame) {
        return None;
    }

    let column = (column as usize).checked_sub(frame.origin_column as usize)?;
    let mut start = text_display_width(" kfnotepad ");
    for group in MENU_GROUPS {
        let end = start + text_display_width(&format!(" {} ", group.label()));
        if (start..end).contains(&column) {
            return Some(group);
        }
        start = end;
    }
    None
}

pub(crate) fn menu_command_at_mouse(
    column: u16,
    row: u16,
    menu: MenuState,
    frame: RenderFrame,
) -> Option<MenuCommand> {
    if row == 0 {
        return None;
    }

    let column_start = frame
        .origin_column
        .saturating_add(menu_dropdown_column(menu.group, frame));
    let available_width = frame.terminal_width.saturating_sub(column_start as usize);
    let width = menu
        .group
        .items()
        .iter()
        .map(menu_item_display_width)
        .max()
        .unwrap_or(4)
        + 4;
    let width = width.min(available_width);
    let item_index = row as usize - 1;

    if width == 0
        || column < column_start
        || column as usize >= column_start as usize + width
        || item_index >= menu.group.items().len()
    {
        return None;
    }

    menu.group.items().get(item_index).map(|item| item.command)
}

pub(crate) fn tab_index_at_mouse(
    column: u16,
    row: u16,
    tab_strip: &[TabStripItem],
    frame: RenderFrame,
) -> Option<usize> {
    if tab_strip.len() <= 1 || row == 0 || row >= frame.body_top {
        return None;
    }

    let column = (column as usize).checked_sub(frame.origin_column as usize)?;
    let target_row = row - 1;
    let mut current_row = 0u16;
    let mut start = 0usize;
    for (index, item) in tab_strip.iter().enumerate() {
        let width = text_display_width(&compose_tab_label(index, item));
        if width > frame.terminal_width.saturating_sub(start) && start > 0 {
            current_row += 1;
            start = 0;
        }
        if current_row > target_row {
            return None;
        }
        let end = start.saturating_add(width);
        if current_row == target_row && (start..end).contains(&column) {
            return Some(index);
        }
        start = end;
    }
    None
}
