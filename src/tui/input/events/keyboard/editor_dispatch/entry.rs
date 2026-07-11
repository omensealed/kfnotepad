pub(crate) fn handle_key_event(
    document: &mut TextDocument,
    cursor: &mut Cursor,
    runtime: &mut EditorRuntime,
    event: KeyEvent,
) -> bool {
    if key_requests_quit(event) {
        return request_quit(document, runtime);
    }

    if let Some(result) = handle_active_editor_mode(document, cursor, runtime, event) {
        return result;
    }

    if handle_editor_command_shortcut(document, cursor, runtime, event) {
        return false;
    }

    if handle_editor_movement_key(document, cursor, runtime, event) {
        return false;
    }

    handle_editor_edit_key(document, cursor, runtime, event);
    false
}

fn key_requests_quit(event: KeyEvent) -> bool {
    event.modifiers.contains(KeyModifiers::CONTROL)
        && matches!(event.code, KeyCode::Char('q') | KeyCode::Char('c'))
}
