//! Terminal dimensions and available editor-body geometry.

use super::*;

pub(crate) fn visible_editor_rows(extra_reserved_rows: usize) -> usize {
    size()
        .map(|(_, rows)| rows.saturating_sub(3 + extra_reserved_rows as u16).max(1) as usize)
        .unwrap_or(20)
}

pub(crate) fn terminal_width() -> usize {
    size()
        .map(|(columns, _)| columns.max(1) as usize)
        .unwrap_or(80)
}

pub(crate) fn visible_text_columns(
    settings: EditorSettings,
    gutter_width: usize,
    terminal_width: usize,
) -> usize {
    let gutter_columns = if settings.show_line_numbers {
        gutter_width + 1
    } else {
        0
    };
    terminal_width
        .saturating_sub(gutter_columns)
        .saturating_sub(EDITOR_BODY_PADDING)
        .max(1)
}
