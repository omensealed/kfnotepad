pub(super) fn gui_editor_replacement_delete_range(
    buffer: &mut TextBuffer,
    start: DocumentCursor,
    end: DocumentCursor,
) -> Result<(), kfnotepad::BufferError> {
    if !document_cursor_is_before_or_equal(start, end) {
        return gui_editor_replacement_delete_range(buffer, end, start);
    }
    if start == end {
        return Ok(());
    }
    validate_gui_editor_replacement_cursor(buffer, start)?;
    validate_gui_editor_replacement_cursor(buffer, end)?;
    let (start, end) = gui_editor_replacement_grapheme_range(buffer, start, end)?;

    buffer.delete_replacement_range(start, end)
}

pub(super) fn gui_editor_replacement_grapheme_range(
    buffer: &TextBuffer,
    start: DocumentCursor,
    end: DocumentCursor,
) -> Result<(DocumentCursor, DocumentCursor), kfnotepad::BufferError> {
    if start.row == end.row {
        let (start_column, end_column) =
            buffer.grapheme_range_boundary_columns(start.row, start.column, end.column)?;
        return Ok((
            DocumentCursor {
                row: start.row,
                column: start_column,
            },
            DocumentCursor {
                row: end.row,
                column: end_column,
            },
        ));
    }

    let start_column = buffer.grapheme_range_start_boundary_column(start.row, start.column)?;
    let end_column = buffer.grapheme_range_end_boundary_column(end.row, end.column)?;
    Ok((
        DocumentCursor {
            row: start.row,
            column: start_column,
        },
        DocumentCursor {
            row: end.row,
            column: end_column,
        },
    ))
}

pub(super) fn char_prefix(value: &str, end_column: usize) -> String {
    value.chars().take(end_column).collect()
}

pub(super) fn char_suffix(value: &str, start_column: usize) -> String {
    value.chars().skip(start_column).collect()
}

pub(super) fn char_slice(value: &str, start_column: usize, end_column: usize) -> String {
    value
        .chars()
        .skip(start_column)
        .take(end_column.saturating_sub(start_column))
        .collect()
}
