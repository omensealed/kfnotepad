//! Cursor and line-number geometry for terminal rendering.

use super::*;

pub(crate) fn line_number_width(document: &TextDocument) -> usize {
    document.buffer.line_count().to_string().len().max(2)
}

pub(crate) fn cursor_screen_column(
    document: &TextDocument,
    cursor: Cursor,
    view: EditorView<'_>,
    frame: RenderFrame,
) -> u16 {
    let gutter_columns = if view.settings.show_line_numbers {
        frame.gutter_width + 1
    } else {
        0
    };
    let body_width = visible_text_columns(view.settings, frame.gutter_width, frame.terminal_width);
    let body_display_column = document
        .buffer
        .lines()
        .get(cursor.row)
        .map(|line| line_display_width_until(line, cursor.column))
        .unwrap_or(0);
    let body_column = if view.settings.wrap_lines {
        body_display_column % body_width
    } else {
        body_display_column.saturating_sub(view.horizontal_offset)
    };
    let max_column = frame.terminal_width.saturating_sub(1);
    body_column
        .saturating_add(gutter_columns)
        .saturating_add(EDITOR_BODY_PADDING)
        .min(max_column) as u16
}

pub(crate) fn cursor_screen_row(
    document: &TextDocument,
    view: EditorView<'_>,
    frame: RenderFrame,
) -> u16 {
    if !view.settings.wrap_lines {
        return frame
            .body_top
            .saturating_add(view.cursor.row.saturating_sub(view.viewport_start) as u16);
    }

    let visual_offset = cursor_visual_row_offset(document, view, frame).unwrap_or(0);
    let max_row = frame
        .body_top
        .saturating_add(view.visible_rows.saturating_sub(1) as u16);
    frame
        .body_top
        .saturating_add(visual_offset as u16)
        .min(max_row)
}

pub(super) fn cursor_visual_row_offset(
    document: &TextDocument,
    view: EditorView<'_>,
    frame: RenderFrame,
) -> Option<usize> {
    if view.cursor.row < view.viewport_start {
        return None;
    }

    let body_width = visible_text_columns(view.settings, frame.gutter_width, frame.terminal_width);
    let mut visual_offset = 0usize;
    for line in document
        .buffer
        .lines()
        .iter()
        .skip(view.viewport_start)
        .take(view.cursor.row.saturating_sub(view.viewport_start))
    {
        visual_offset = visual_offset.saturating_add(wrapped_line_chunk_count(line, body_width));
        if visual_offset >= view.visible_rows {
            return Some(visual_offset);
        }
    }

    let body_display_column = document
        .buffer
        .lines()
        .get(view.cursor.row)
        .map(|line| line_display_width_until(line, view.cursor.column))
        .unwrap_or(0);
    Some(visual_offset.saturating_add(body_display_column / body_width))
}
