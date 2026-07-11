fn handle_editor_edit_key(
    document: &mut TextDocument,
    cursor: &mut Cursor,
    runtime: &mut EditorRuntime,
    event: KeyEvent,
) -> bool {
    match (event.modifiers, event.code) {
        (KeyModifiers::CONTROL, KeyCode::Backspace) => {
            delete_previous_word(document, cursor, runtime);
        }
        (KeyModifiers::CONTROL, KeyCode::Delete) => {
            delete_next_word(document, cursor, runtime);
        }
        (_, KeyCode::Backspace) => {
            runtime.quit_confirmation_pending = false;
            if let Ok(moved) = document.buffer.delete_before_cursor(*cursor) {
                *cursor = moved;
                stop_reader_mode_for_edit(runtime);
                runtime.status = String::from("Modified");
            }
        }
        (_, KeyCode::Delete) => {
            runtime.quit_confirmation_pending = false;
            if document
                .buffer
                .delete_char(cursor.row, cursor.column)
                .is_ok()
            {
                stop_reader_mode_for_edit(runtime);
                runtime.status = String::from("Modified");
            }
        }
        (_, KeyCode::Enter) => {
            runtime.quit_confirmation_pending = false;
            if document
                .buffer
                .insert_newline(cursor.row, cursor.column)
                .is_ok()
            {
                cursor.row += 1;
                cursor.column = 0;
                stop_reader_mode_for_edit(runtime);
                runtime.status = String::from("Modified");
            }
        }
        (_, KeyCode::BackTab) | (KeyModifiers::SHIFT, KeyCode::Tab) => {
            unindent_at_cursor(document, cursor, runtime);
        }
        (_, KeyCode::Tab) => {
            indent_at_cursor(document, cursor, runtime);
        }
        (KeyModifiers::NONE | KeyModifiers::SHIFT, KeyCode::Char(value)) => {
            insert_typed_character(document, cursor, runtime, value);
        }
        _ => return false,
    }
    true
}
