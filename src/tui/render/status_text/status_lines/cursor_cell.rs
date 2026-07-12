//! Cursor-cell highlighting and viewport visibility checks.

use super::*;

pub(crate) fn write_cursor_cell_highlight(
    writer: &mut impl Write,
    document: &TextDocument,
    view: EditorView<'_>,
    frame: RenderFrame,
) -> io::Result<()> {
    let character = document
        .buffer
        .lines()
        .get(view.cursor.row)
        .and_then(|line: &String| line.chars().nth(view.cursor.column))
        .map(cursor_cell_character)
        .unwrap_or(' ');
    queue!(
        writer,
        frame.move_to(
            cursor_screen_column(document, view.cursor, view, frame),
            cursor_screen_row(document, view, frame)
        ),
        SetAttribute(Attribute::Reverse),
        Print(character),
        SetAttribute(Attribute::NoReverse),
        ResetColor,
    )
}

pub(crate) fn cursor_row_is_visible(
    document: &TextDocument,
    view: EditorView<'_>,
    frame: RenderFrame,
) -> bool {
    if view.settings.wrap_lines {
        return cursor_visual_row_offset(document, view, frame)
            .is_some_and(|offset| offset < view.visible_rows);
    }

    let visible_end = view.viewport_start.saturating_add(view.visible_rows.max(1));
    view.cursor.row >= view.viewport_start && view.cursor.row < visible_end
}
