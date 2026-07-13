use super::*;

pub(in crate::gui::app::state) fn gui_find_next_case_insensitive(
    document: &TextDocument,
    query: &str,
    start: DocumentCursor,
) -> Option<DocumentCursor> {
    if query.is_empty() || start.row >= document.buffer.line_count() {
        return None;
    }
    for row in start.row..document.buffer.line_count() {
        let column = if row == start.row { start.column } else { 0 };
        if let Some(cursor) = gui_find_in_line_case_insensitive(
            document.buffer.line(row).unwrap_or_default(),
            query,
            column,
            row,
        ) {
            return Some(cursor);
        }
    }
    None
}

pub(in crate::gui::app::state) fn gui_find_previous_case_insensitive(
    document: &TextDocument,
    query: &str,
    start: DocumentCursor,
) -> Option<DocumentCursor> {
    if query.is_empty() || document.buffer.line_count() == 0 {
        return None;
    }
    let start_row = start
        .row
        .min(document.buffer.line_count().saturating_sub(1));
    for row in (0..=start_row).rev() {
        let line = document.buffer.line(row).unwrap_or_default();
        let max_column = line.chars().count();
        let before_column = if row == start_row {
            start.column.min(max_column)
        } else {
            max_column
        };
        if let Some(cursor) =
            gui_find_last_in_line_case_insensitive(line, query, before_column, row)
        {
            return Some(cursor);
        }
    }
    None
}

pub(in crate::gui::app::state) fn gui_find_in_line_case_insensitive(
    line: &str,
    query: &str,
    start_column: usize,
    row: usize,
) -> Option<DocumentCursor> {
    let range = find_case_insensitive_range(line, query, start_column)?;
    Some(DocumentCursor {
        row,
        column: range.start,
    })
}

pub(in crate::gui::app::state) fn gui_find_last_in_line_case_insensitive(
    line: &str,
    query: &str,
    before_column: usize,
    row: usize,
) -> Option<DocumentCursor> {
    let range = find_last_case_insensitive_range(line, query, before_column)?;
    Some(DocumentCursor {
        row,
        column: range.start,
    })
}
