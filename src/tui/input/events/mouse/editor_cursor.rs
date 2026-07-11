pub(crate) fn cursor_at_mouse(
    document: &TextDocument,
    column: u16,
    row: u16,
    runtime: &EditorRuntime,
    context: MouseContext,
) -> Option<Cursor> {
    let body_row = row.checked_sub(context.body_top)? as usize;
    if body_row >= context.visible_rows {
        return None;
    }

    let gutter_columns = if runtime.settings.show_line_numbers {
        context.gutter_width + 1
    } else {
        0
    };
    let body_column = (column as usize).saturating_sub(gutter_columns);
    let body_column = body_column.saturating_sub(context.sidebar_width);
    let body_column = body_column.saturating_sub(EDITOR_BODY_PADDING);

    if runtime.settings.wrap_lines {
        let (document_row, wrapped_row_offset) =
            wrapped_document_row_at_screen_row(document, runtime.settings, context, body_row)?;
        let line = document.buffer.lines().get(document_row)?;
        let body_width = visible_text_columns(
            runtime.settings,
            context.gutter_width,
            context.terminal_width,
        );
        let target_display_column = wrapped_row_offset
            .saturating_mul(body_width)
            .saturating_add(body_column);
        let column = char_column_for_display_column(line, target_display_column);
        return Some(Cursor {
            row: document_row,
            column: document
                .buffer
                .grapheme_boundary_column(document_row, column)
                .unwrap_or(column),
        });
    }

    let document_row = context.viewport_start + body_row;
    let line = document.buffer.lines().get(document_row)?;
    let column = char_column_for_display_column(
        line,
        context.horizontal_offset.saturating_add(body_column),
    );
    Some(Cursor {
        row: document_row,
        column: document
            .buffer
            .grapheme_boundary_column(document_row, column)
            .unwrap_or(column),
    })
}

pub(crate) fn wrapped_document_row_at_screen_row(
    document: &TextDocument,
    settings: EditorSettings,
    context: MouseContext,
    body_row: usize,
) -> Option<(usize, usize)> {
    let body_width = visible_text_columns(settings, context.gutter_width, context.terminal_width);
    let mut screen_row = 0usize;

    for (document_row, line) in document
        .buffer
        .lines()
        .iter()
        .enumerate()
        .skip(context.viewport_start)
    {
        let chunk_count = wrapped_line_chunks(line, body_width).len().max(1);
        if body_row < screen_row + chunk_count {
            return Some((document_row, body_row - screen_row));
        }
        screen_row += chunk_count;
        if screen_row > context.visible_rows {
            break;
        }
    }

    None
}
