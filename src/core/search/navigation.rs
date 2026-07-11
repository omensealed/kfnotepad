pub(crate) fn next_search_start(
    document: &super::TextDocument,
    cursor: super::Cursor,
) -> super::Cursor {
    let columns = document.buffer.line_char_count(cursor.row).unwrap_or(0);
    if cursor.column < columns {
        let next_column = document
            .buffer
            .grapheme_range_end_boundary_column(cursor.row, cursor.column.saturating_add(1))
            .unwrap_or_else(|_| cursor.column.saturating_add(1));
        return super::Cursor {
            row: cursor.row,
            column: next_column,
        };
    }
    if cursor.row + 1 < document.buffer.line_count() {
        return super::Cursor {
            row: cursor.row + 1,
            column: 0,
        };
    }
    super::Cursor { row: 0, column: 0 }
}
