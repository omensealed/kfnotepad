#[cfg(test)]
fn render_editor_with_width_and_color(
    writer: &mut impl Write,
    document: &TextDocument,
    view: EditorView<'_>,
    highlighter: &SyntaxHighlighter,
    terminal_width: usize,
    no_color: bool,
) -> io::Result<()> {
    render_editor_with_width_color_and_cache(
        writer,
        document,
        view,
        highlighter,
        terminal_width,
        no_color,
        None,
    )
}

fn render_editor_with_width_color_and_cache(
    writer: &mut impl Write,
    document: &TextDocument,
    view: EditorView<'_>,
    highlighter: &SyntaxHighlighter,
    terminal_width: usize,
    no_color: bool,
    syntax_cache: Option<&mut TuiSyntaxHighlightCache>,
) -> io::Result<()> {
    let theme = EditorTheme::for_id(view.settings.theme_id);
    let sidebar_width = view.sidebar_width.min(terminal_width.saturating_sub(1));
    let terminal_width = terminal_width.saturating_sub(sidebar_width).max(1);
    let frame = RenderFrame {
        theme,
        gutter_width: line_number_width(document),
        terminal_width,
        origin_column: sidebar_width as u16,
        body_top: tab_strip_height_for_width(view.tab_strip, terminal_width),
        no_color,
    };
    write_header(writer, document, view.menu, frame)?;
    if !view.tab_strip.is_empty() {
        write_tab_strip(writer, view.tab_strip, frame)?;
    }

    if view.settings.wrap_lines {
        write_wrapped_editor_lines(writer, document, view, highlighter, syntax_cache, frame)?;
    } else {
        let highlighted_lines = highlight_lines_for_render(
            document,
            view.viewport_start,
            view.visible_rows,
            highlighter,
            syntax_cache,
        );
        for body_index in 0..view.visible_rows {
            let document_row = view.viewport_start + body_index;
            let screen_row = frame.body_top + body_index as u16;
            if let Some(line) = document.buffer.lines().get(document_row) {
                write_editor_line(
                    writer,
                    EditorLineView {
                        screen_row,
                        document_row,
                        line,
                        settings: view.settings,
                        horizontal_offset: view.horizontal_offset,
                        highlighted_line: highlighted_lines.get(body_index).cloned().flatten(),
                        search_highlight: view.search_highlight,
                    },
                    frame,
                )?;
            } else {
                clear_editor_body_row(writer, screen_row, frame)?;
            }
        }
    }

    if let Some(menu) = view.menu {
        write_menu_dropdown(writer, menu, frame)?;
    }

    let status_screen_row = frame.body_top + view.visible_rows as u16;
    let status_cursor_column = write_status_line(
        writer,
        StatusLineView {
            document,
            cursor: view.cursor,
            status: view.status,
            settings: view.settings,
            horizontal_offset: view.horizontal_offset,
            screen_row: status_screen_row,
        },
        frame,
    )?;
    write_help_line(writer, status_screen_row + 1, frame)?;
    place_terminal_cursor(
        writer,
        document,
        view,
        frame,
        status_screen_row,
        status_cursor_column,
    )?;
    writer.flush()
}
