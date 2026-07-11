fn handle_editor_movement_key(
    document: &TextDocument,
    cursor: &mut Cursor,
    runtime: &mut EditorRuntime,
    event: KeyEvent,
) -> bool {
    match (event.modifiers, event.code) {
        (KeyModifiers::CONTROL, KeyCode::Left) => {
            runtime.quit_confirmation_pending = false;
            move_cursor(document, cursor, CursorMove::WordLeft);
        }
        (KeyModifiers::CONTROL, KeyCode::Right) => {
            runtime.quit_confirmation_pending = false;
            move_cursor(document, cursor, CursorMove::WordRight);
        }
        (KeyModifiers::CONTROL, KeyCode::Char('a')) => {
            runtime.quit_confirmation_pending = false;
            cursor.column = 0;
        }
        (KeyModifiers::CONTROL, KeyCode::Char('e')) => {
            runtime.quit_confirmation_pending = false;
            if let Ok(columns) = document.buffer.line_char_count(cursor.row) {
                cursor.column = columns;
            }
        }
        (_, KeyCode::Left) => {
            runtime.quit_confirmation_pending = false;
            move_cursor(document, cursor, CursorMove::Left);
        }
        (_, KeyCode::Right) => {
            runtime.quit_confirmation_pending = false;
            move_cursor(document, cursor, CursorMove::Right);
        }
        (_, KeyCode::Up) => {
            runtime.quit_confirmation_pending = false;
            move_cursor(document, cursor, CursorMove::Up);
        }
        (_, KeyCode::Down) => {
            runtime.quit_confirmation_pending = false;
            move_cursor(document, cursor, CursorMove::Down);
        }
        (_, KeyCode::PageUp) => {
            page_up(document, cursor, runtime);
        }
        (_, KeyCode::PageDown) => {
            page_down(document, cursor, runtime);
        }
        (KeyModifiers::CONTROL, KeyCode::Home) => {
            go_to_document_start(cursor, runtime);
        }
        (KeyModifiers::CONTROL, KeyCode::End) => {
            go_to_document_end(document, cursor, runtime);
        }
        (_, KeyCode::Home) => {
            runtime.quit_confirmation_pending = false;
            cursor.column = 0;
        }
        (_, KeyCode::End) => {
            runtime.quit_confirmation_pending = false;
            if let Ok(columns) = document.buffer.line_char_count(cursor.row) {
                cursor.column = columns;
            }
        }
        _ => return false,
    }
    true
}
