use super::*;

pub(in crate::gui::app::state) fn gui_repeat_search(
    document: &TextDocument,
    cursor: &mut DocumentCursor,
    query: &str,
    backwards: bool,
    case_sensitive: bool,
) -> SearchRepeatResult {
    if query.is_empty() {
        return SearchRepeatResult::NoPreviousSearch;
    }
    if case_sensitive {
        return if backwards {
            repeat_search_previous(document, cursor, query)
        } else {
            repeat_search_next(document, cursor, query)
        };
    }

    let found = if backwards {
        gui_find_previous_case_insensitive(document, query, *cursor)
    } else {
        let start = gui_next_search_start(document, *cursor);
        gui_find_next_case_insensitive(document, query, start).or_else(|| {
            gui_find_next_case_insensitive(document, query, DocumentCursor { row: 0, column: 0 })
        })
    };

    if let Some(found) = found {
        *cursor = found;
        SearchRepeatResult::Found {
            query: query.to_string(),
        }
    } else {
        SearchRepeatResult::NoMatch {
            query: query.to_string(),
        }
    }
}

pub(in crate::gui::app::state) fn gui_next_search_start(
    document: &TextDocument,
    cursor: DocumentCursor,
) -> DocumentCursor {
    let columns = document.buffer.line_char_count(cursor.row).unwrap_or(0);
    if cursor.column < columns {
        let next_column = document
            .buffer
            .grapheme_range_end_boundary_column(cursor.row, cursor.column.saturating_add(1))
            .unwrap_or_else(|_| cursor.column.saturating_add(1));
        return DocumentCursor {
            row: cursor.row,
            column: next_column,
        };
    }
    if cursor.row + 1 < document.buffer.line_count() {
        return DocumentCursor {
            row: cursor.row + 1,
            column: 0,
        };
    }
    DocumentCursor { row: 0, column: 0 }
}
