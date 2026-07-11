struct StatusLineView<'a> {
    document: &'a TextDocument,
    cursor: Cursor,
    status: &'a str,
    settings: EditorSettings,
    screen_row: u16,
    horizontal_offset: usize,
}

fn write_status_line(
    writer: &mut impl Write,
    view: StatusLineView<'_>,
    frame: RenderFrame,
) -> io::Result<Option<u16>> {
    let line_numbers = if view.settings.show_line_numbers {
        "num:on"
    } else {
        "num:off"
    };
    let wrap = if view.settings.wrap_lines {
        "wrap:on"
    } else {
        "wrap:off"
    };
    let scroll = if view.settings.wrap_lines || view.horizontal_offset == 0 {
        "x:0".to_string()
    } else {
        format!("x:{}", view.horizontal_offset + 1)
    };
    let dirty = if view.document.buffer.is_dirty() {
        "modified"
    } else {
        "saved"
    };
    let right = format!(
        " Ln {}, Col {} | {} | {} | {} | {} | {} ",
        view.cursor.row + 1,
        view.cursor.column + 1,
        line_numbers,
        wrap,
        scroll,
        view.settings.theme_id.label(),
        dirty
    );
    let left = format!(" {} ", view.status);
    let mut remaining = frame.terminal_width;
    queue!(
        writer,
        frame.move_to(0, view.screen_row),
        Clear(ClearType::CurrentLine),
    )?;
    queue_set_foreground_color(writer, &frame, frame.theme.status_fg)?;
    queue_set_background_color(writer, &frame, frame.theme.status_bg)?;
    queue!(writer, SetAttribute(Attribute::Bold))?;
    let rendered = if let Some(query) = view.status.strip_prefix("Search: ") {
        compose_prompt_status_line("Search: ", query, &right, frame.terminal_width)
    } else if let Some(query) = view.status.strip_prefix("Go to line: ") {
        compose_prompt_status_line("Go to line: ", query, &right, frame.terminal_width)
    } else {
        StatusLineRender {
            text: compose_status_line(&left, &right, frame.terminal_width),
            cursor_column: None,
        }
    };
    print_truncated(writer, &rendered.text, &mut remaining)?;
    queue!(writer, SetAttribute(Attribute::Reset), ResetColor)?;
    Ok(rendered.cursor_column)
}
