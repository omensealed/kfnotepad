pub(super) fn gui_editor_replacement_cursor_from_mouse_point(
    buffer: &TextBuffer,
    viewport: GuiEditorViewportState,
    point: GuiEditorReplacementMousePoint,
) -> DocumentCursor {
    let total = buffer.line_count().max(1);
    let row = viewport
        .first_line
        .saturating_sub(1)
        .saturating_add(point.viewport_row)
        .min(total.saturating_sub(1));
    let column = buffer
        .line_char_count(row)
        .and_then(|columns| buffer.grapheme_boundary_column(row, point.column.min(columns)))
        .unwrap_or_default();
    DocumentCursor { row, column }
}
