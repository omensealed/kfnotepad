pub(crate) fn write_menu_dropdown(
    writer: &mut impl Write,
    menu: MenuState,
    frame: RenderFrame,
) -> io::Result<()> {
    let column = menu_dropdown_column(menu.group, frame);
    let available_width = frame.terminal_width.saturating_sub(column as usize);
    if available_width == 0 {
        return Ok(());
    }
    let width = menu
        .group
        .items()
        .iter()
        .map(menu_item_display_width)
        .max()
        .unwrap_or(4)
        + 4;
    let width = width.min(available_width);
    for (index, item) in menu.group.items().iter().enumerate() {
        let mut remaining = width;
        queue!(
            writer,
            ResetColor,
            SetAttribute(Attribute::Reset),
            frame.move_to(column, (index + 1) as u16)
        )?;
        if index == menu.selected {
            queue_set_foreground_color(writer, &frame, frame.theme.status_fg)?;
            queue_set_background_color(writer, &frame, frame.theme.status_bg)?;
            queue!(writer, SetAttribute(Attribute::Bold))?;
        } else {
            queue_set_foreground_color(writer, &frame, frame.theme.header_fg)?;
            queue_set_background_color(writer, &frame, frame.theme.header_bg)?;
            queue!(writer, SetAttribute(Attribute::Reset))?;
        }
        print_truncated(
            writer,
            &format_menu_item(item, width.saturating_sub(2)),
            &mut remaining,
        )?;
    }
    queue!(writer, ResetColor, SetAttribute(Attribute::Reset))
}
