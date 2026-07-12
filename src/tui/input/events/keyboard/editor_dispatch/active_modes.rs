//! Routing for active sidebar, menu, go-to-line, and search modes.

use super::*;

pub(super) fn handle_active_editor_mode(
    document: &mut TextDocument,
    cursor: &mut Cursor,
    runtime: &mut EditorRuntime,
    event: KeyEvent,
) -> Option<bool> {
    if runtime.sidebar.is_some() {
        return Some(handle_sidebar_key_event(document, cursor, runtime, event));
    }

    if runtime.menu.is_some() {
        return Some(handle_menu_key_event(document, cursor, runtime, event));
    }

    if runtime.goto_line_active {
        handle_goto_line_key_event(document, cursor, runtime, event);
        return Some(false);
    }

    if runtime.search_active {
        handle_search_key_event(document, cursor, runtime, event);
        return Some(false);
    }

    None
}
