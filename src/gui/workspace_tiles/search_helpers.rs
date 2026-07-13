//! Search match width helpers shared by workspace navigation operations.

use super::*;

pub(in crate::gui::app::state) fn gui_search_match_columns(
    document: &TextDocument,
    cursor: DocumentCursor,
    query: &str,
    case_sensitive: bool,
) -> Option<usize> {
    let line = document.buffer.line(cursor.row)?;
    if case_sensitive {
        let range = expand_range_to_grapheme_boundaries(
            line,
            cursor.column..cursor.column.saturating_add(query.chars().count()),
        );
        return (range.start == cursor.column).then_some(range.end.saturating_sub(range.start));
    }
    let range = find_case_insensitive_range(line, query, cursor.column)?;
    (range.start == cursor.column).then_some(range.end.saturating_sub(range.start))
}
