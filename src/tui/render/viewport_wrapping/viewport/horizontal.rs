//! Cursor-following horizontal viewport clamping.

use super::*;

pub(crate) fn clamp_horizontal_viewport(
    document: &TextDocument,
    cursor: Cursor,
    settings: EditorSettings,
    gutter_width: usize,
    terminal_width: usize,
    horizontal_offset: usize,
) -> usize {
    let visible_columns = visible_text_columns(settings, gutter_width, terminal_width);
    let cursor_display_column = document
        .buffer
        .lines()
        .get(cursor.row)
        .map(|line| line_display_width_until(line, cursor.column))
        .unwrap_or(0);
    if cursor_display_column < horizontal_offset {
        cursor_display_column
    } else if cursor_display_column >= horizontal_offset + visible_columns {
        cursor_display_column + 1 - visible_columns
    } else {
        horizontal_offset
    }
}
