pub(crate) fn menu_item_display_width(item: &MenuItem) -> usize {
    text_display_width(&format_menu_item(item, 0))
}

fn format_menu_item(item: &MenuItem, inner_width: usize) -> String {
    match item.shortcut {
        Some(shortcut) => {
            let label_width = text_display_width(item.label);
            let shortcut_width = text_display_width(shortcut);
            let gap = inner_width
                .saturating_sub(label_width + shortcut_width)
                .max(2);
            format!("  {}{}{}", item.label, " ".repeat(gap), shortcut)
        }
        None => format!("  {}", item.label),
    }
}

pub(crate) fn menu_dropdown_column(group: MenuGroup, frame: RenderFrame) -> u16 {
    let mut column = text_display_width(" kfnotepad ");
    for menu_group in MENU_GROUPS {
        if menu_group == group {
            break;
        }
        let menu_group_label = format!(" {} ", menu_group.label());
        column += text_display_width(&menu_group_label);
    }
    column.min(frame.terminal_width.saturating_sub(1)) as u16
}

fn menu_bar_text() -> &'static str {
    " File  Edit  View  Go  Tabs  Workspace  Help |"
}
