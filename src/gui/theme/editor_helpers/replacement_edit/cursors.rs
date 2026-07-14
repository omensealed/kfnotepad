use super::*;

pub(in crate::gui::app::state) fn delete_gui_editor_replacement_selection(
    document: &mut TextDocument,
    cursor: &mut DocumentCursor,
    selection: &mut Option<GuiEditorReplacementSelection>,
) -> bool {
    let Some(active_selection) = selection.take() else {
        return false;
    };
    let (start, end) = active_selection.normalized();
    let Ok((range_start, range_end)) =
        gui_editor_replacement_grapheme_range(&document.buffer, start, end)
    else {
        return false;
    };
    if gui_editor_replacement_delete_range(&mut document.buffer, range_start, range_end).is_ok() {
        *cursor = range_start;
        true
    } else {
        false
    }
}

pub(in crate::gui::app::state) fn gui_editor_replacement_document_end_cursor(
    buffer: &TextBuffer,
) -> DocumentCursor {
    let row = buffer.line_count().saturating_sub(1);
    DocumentCursor {
        row,
        column: buffer.line_char_count(row).unwrap_or_default(),
    }
}

pub(in crate::gui::app::state) fn gui_editor_replacement_selection_covers_full_text(
    document: &TextDocument,
    start: DocumentCursor,
    end: DocumentCursor,
) -> bool {
    start == (DocumentCursor { row: 0, column: 0 })
        && end == gui_editor_replacement_document_end_cursor(&document.buffer)
}

pub(in crate::gui::app::state) fn gui_editor_replacement_cursor_is_valid(
    buffer: &TextBuffer,
    cursor: DocumentCursor,
) -> bool {
    buffer
        .line_char_count(cursor.row)
        .is_ok_and(|columns| cursor.column <= columns)
}

pub(in crate::gui::app::state) fn validate_gui_editor_replacement_cursor(
    buffer: &TextBuffer,
    cursor: DocumentCursor,
) -> Result<(), kfnotepad::BufferError> {
    let columns = buffer.line_char_count(cursor.row)?;
    if cursor.column <= columns {
        Ok(())
    } else {
        Err(kfnotepad::BufferError::ColumnOutOfBounds {
            column: cursor.column,
            columns,
        })
    }
}

pub(in crate::gui::app::state) fn document_cursor_is_before_or_equal(
    left: DocumentCursor,
    right: DocumentCursor,
) -> bool {
    (left.row, left.column) <= (right.row, right.column)
}
