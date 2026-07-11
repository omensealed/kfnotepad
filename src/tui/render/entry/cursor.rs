fn place_terminal_cursor(
    writer: &mut impl Write,
    document: &TextDocument,
    view: EditorView<'_>,
    frame: RenderFrame,
    status_screen_row: u16,
    status_cursor_column: Option<u16>,
) -> io::Result<()> {
    if let Some(status_cursor_column) = status_cursor_column {
        queue!(
            writer,
            Show,
            frame.move_to(status_cursor_column, status_screen_row)
        )?;
    } else if let Some(menu) = view.menu {
        queue!(
            writer,
            Show,
            frame.move_to(
                menu_dropdown_column(menu.group, frame).saturating_add(2),
                (menu.selected + 1) as u16
            )
        )?;
    } else if !cursor_row_is_visible(document, view, frame) {
        queue!(writer, Hide)?;
    } else {
        write_cursor_cell_highlight(writer, document, view, frame)?;
        queue!(
            writer,
            Show,
            frame.move_to(
                cursor_screen_column(document, view.cursor, view, frame),
                cursor_screen_row(document, view, frame)
            )
        )?;
    }
    Ok(())
}
