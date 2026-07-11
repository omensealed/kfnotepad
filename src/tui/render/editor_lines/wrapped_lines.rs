fn write_wrapped_editor_lines(
    writer: &mut impl Write,
    document: &TextDocument,
    view: EditorView<'_>,
    highlighter: &SyntaxHighlighter,
    syntax_cache: Option<&mut TuiSyntaxHighlightCache>,
    frame: RenderFrame,
) -> io::Result<()> {
    let body_width = visible_text_columns(view.settings, frame.gutter_width, frame.terminal_width);
    let mut screen_row = frame.body_top;
    let highlighted_lines = highlighter_lines_for_wrapped_view(
        document,
        view,
        highlighter,
        syntax_cache,
        body_width.max(1),
    );

    for (index, line) in document
        .buffer
        .lines()
        .iter()
        .enumerate()
        .skip(view.viewport_start)
    {
        let highlighted_line = highlighted_lines
            .get(index.saturating_sub(view.viewport_start))
            .cloned()
            .flatten();
        for (chunk_index, chunk) in wrapped_line_chunks(line, body_width)
            .into_iter()
            .enumerate()
        {
            if screen_row >= frame.body_top + view.visible_rows as u16 {
                return clear_remaining_editor_rows(writer, screen_row, view.visible_rows, frame);
            }
            write_wrapped_editor_chunk(
                writer,
                WrappedEditorChunkView {
                    screen_row,
                    document_row: index,
                    chunk_index,
                    line,
                    chunk: chunk.text,
                    chunk_start_column: chunk.start_column,
                    highlighted_line: highlighted_line.clone(),
                    settings: view.settings,
                    search_highlight: view.search_highlight,
                },
                frame,
            )?;
            screen_row += 1;
        }
    }

    clear_remaining_editor_rows(writer, screen_row, view.visible_rows, frame)
}
