//! Editor command keyboard shortcuts.

use super::*;

pub(super) fn handle_editor_command_shortcut(
    document: &mut TextDocument,
    cursor: &mut Cursor,
    runtime: &mut EditorRuntime,
    event: KeyEvent,
) -> bool {
    match (event.modifiers, event.code) {
        (_, KeyCode::F(10)) => open_menu(runtime),
        (KeyModifiers::SHIFT, KeyCode::F(3)) => repeat_search_previous(document, cursor, runtime),
        (_, KeyCode::F(3)) => repeat_search(document, cursor, runtime),
        (KeyModifiers::CONTROL, KeyCode::Char('s')) => save_document(document, runtime),
        (KeyModifiers::CONTROL, KeyCode::Char('b')) => {
            toggle_file_sidebar(runtime);
        }
        (KeyModifiers::CONTROL, KeyCode::Char('z')) => {
            undo_document(document, cursor, runtime);
        }
        (KeyModifiers::CONTROL, KeyCode::Char('y')) => {
            redo_document(document, cursor, runtime);
        }
        (KeyModifiers::CONTROL, KeyCode::Char('f')) => {
            start_search(runtime);
        }
        (modifiers, KeyCode::Char('f') | KeyCode::Char('F'))
            if modifiers.contains(KeyModifiers::CONTROL)
                && modifiers.contains(KeyModifiers::SHIFT) =>
        {
            toggle_search_case(runtime);
        }
        (KeyModifiers::CONTROL, KeyCode::Char('g')) => {
            start_goto_line(runtime);
        }
        (KeyModifiers::CONTROL, KeyCode::Char('l')) => {
            toggle_line_numbers(runtime);
        }
        (KeyModifiers::CONTROL, KeyCode::Char('t')) => {
            cycle_theme(runtime);
        }
        (modifiers, KeyCode::Char('t') | KeyCode::Char('T'))
            if modifiers.contains(KeyModifiers::CONTROL)
                && modifiers.contains(KeyModifiers::SHIFT) =>
        {
            cycle_syntax_theme(runtime);
        }
        (KeyModifiers::CONTROL, KeyCode::Char('w')) => {
            toggle_wrap(runtime);
        }
        (KeyModifiers::CONTROL, KeyCode::Char('r')) => {
            toggle_reader_mode(runtime);
        }
        (KeyModifiers::CONTROL, KeyCode::Char('k')) => {
            delete_to_line_end(document, cursor, runtime);
        }
        (_, KeyCode::Insert) => {
            toggle_overwrite_mode(runtime);
        }
        _ => return false,
    }
    true
}
