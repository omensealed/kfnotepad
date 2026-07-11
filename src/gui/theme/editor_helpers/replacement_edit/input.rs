pub(super) fn apply_gui_editor_replacement_input(
    document: &mut TextDocument,
    cursor: &mut DocumentCursor,
    viewport: &mut GuiEditorViewportState,
    selection: &mut Option<GuiEditorReplacementSelection>,
    input: GuiEditorReplacementInput,
) {
    apply_gui_editor_replacement_input_with_mode(
        document, cursor, viewport, selection, false, input,
    );
}

pub(super) fn apply_gui_editor_replacement_input_with_mode(
    document: &mut TextDocument,
    cursor: &mut DocumentCursor,
    viewport: &mut GuiEditorViewportState,
    selection: &mut Option<GuiEditorReplacementSelection>,
    overwrite_mode: bool,
    input: GuiEditorReplacementInput,
) {
    match input {
        GuiEditorReplacementInput::InsertChar(value) => {
            let deleted_selection =
                delete_gui_editor_replacement_selection(document, cursor, selection);
            if overwrite_mode && !deleted_selection {
                let _ = document.buffer.delete_char(cursor.row, cursor.column);
            }
            if document
                .buffer
                .insert_char(cursor.row, cursor.column, value)
                .is_ok()
            {
                let inserted_end = cursor.column.saturating_add(1);
                cursor.column = document
                    .buffer
                    .grapheme_range_end_boundary_column(cursor.row, inserted_end)
                    .unwrap_or(inserted_end);
            }
        }
        GuiEditorReplacementInput::InsertNewline => {
            delete_gui_editor_replacement_selection(document, cursor, selection);
            if document
                .buffer
                .insert_newline(cursor.row, cursor.column)
                .is_ok()
            {
                cursor.row = cursor.row.saturating_add(1);
                cursor.column = 0;
            }
        }
        GuiEditorReplacementInput::DeleteBackward => {
            if delete_gui_editor_replacement_selection(document, cursor, selection) {
                // Selection deletion already positioned the cursor at the start of the range.
            } else if let Ok(next_cursor) = document.buffer.delete_before_cursor(*cursor) {
                *cursor = next_cursor;
            }
        }
        GuiEditorReplacementInput::DeleteForward => {
            if !delete_gui_editor_replacement_selection(document, cursor, selection) {
                let _ = document.buffer.delete_char(cursor.row, cursor.column);
            }
        }
        GuiEditorReplacementInput::DeletePreviousWord => {
            if !delete_gui_editor_replacement_selection(document, cursor, selection) {
                let _ = delete_previous_word(document, cursor);
            }
        }
        GuiEditorReplacementInput::DeleteNextWord => {
            if !delete_gui_editor_replacement_selection(document, cursor, selection) {
                let _ = delete_next_word(document, cursor);
            }
        }
        GuiEditorReplacementInput::DeleteToLineEnd => {
            if !delete_gui_editor_replacement_selection(document, cursor, selection) {
                let _ = delete_to_line_end(document, cursor);
            }
        }
        GuiEditorReplacementInput::Move(direction) => {
            *selection = None;
            if let Ok(next_cursor) = document.buffer.move_cursor(*cursor, direction) {
                *cursor = next_cursor;
            }
        }
        GuiEditorReplacementInput::MoveLineStart => {
            *selection = None;
            cursor.column = 0;
        }
        GuiEditorReplacementInput::MoveLineEnd => {
            *selection = None;
            cursor.column = document
                .buffer
                .line_char_count(cursor.row)
                .unwrap_or(cursor.column);
        }
        GuiEditorReplacementInput::ScrollViewportLines(delta) => {
            viewport.scroll_by(delta, document.buffer.line_count());
            let visible_cursor =
                viewport.clamp_cursor_to_visible(*cursor, document.buffer.line_count());
            *cursor = visible_cursor;
        }
        GuiEditorReplacementInput::SelectAll => {
            let start = DocumentCursor { row: 0, column: 0 };
            let end = gui_editor_replacement_document_end_cursor(&document.buffer);
            *cursor = end;
            *selection = GuiEditorReplacementSelection::new(start, end);
        }
        GuiEditorReplacementInput::SelectRange { anchor, focus } => {
            if gui_editor_replacement_cursor_is_valid(&document.buffer, anchor)
                && gui_editor_replacement_cursor_is_valid(&document.buffer, focus)
            {
                *cursor = focus;
                *selection = GuiEditorReplacementSelection::new(anchor, focus);
            }
        }
        GuiEditorReplacementInput::ClearSelection => {
            *selection = None;
        }
    }
    viewport.keep_cursor_visible(*cursor, document.buffer.line_count());
}
