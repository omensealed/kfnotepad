//! Paste normalization, bulk insertion, and prompt-aware paste behavior.

use super::*;

pub(crate) fn insert_paste(
    document: &mut TextDocument,
    cursor: &mut Cursor,
    runtime: &mut EditorRuntime,
    text: &str,
) -> bool {
    let normalized = text.replace("\r\n", "\n");
    let normalized = normalized.replace('\r', "\n");
    if normalized.is_empty() {
        return false;
    }
    if can_bulk_overwrite_paste(runtime, &normalized) {
        runtime.quit_confirmation_pending = false;
        match document.buffer.overwrite_text(*cursor, &normalized) {
            Ok(next_cursor) => {
                *cursor = next_cursor;
                stop_reader_mode_for_edit(runtime);
                runtime.status = String::from("Modified overwrite");
            }
            Err(BufferError::TooLarge { limit, .. }) => {
                runtime.status = format!("Paste exceeds {limit} byte limit");
            }
            Err(_) => {}
        }
        return false;
    }
    if can_bulk_insert_paste(runtime) {
        runtime.quit_confirmation_pending = false;
        match document.buffer.insert_text(*cursor, &normalized) {
            Ok(next_cursor) => {
                *cursor = next_cursor;
                stop_reader_mode_for_edit(runtime);
                runtime.status = String::from("Modified");
            }
            Err(BufferError::TooLarge { limit, .. }) => {
                runtime.status = format!("Paste exceeds {limit} byte limit");
            }
            Err(_) => {}
        }
        return false;
    }

    let mut handled = false;
    document.with_compound_edit(|document| {
        for value in normalized.chars() {
            match value {
                '\n' => {
                    handled = handle_key_event(
                        document,
                        cursor,
                        runtime,
                        KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE),
                    );
                }
                value => {
                    let event = KeyEvent::new(KeyCode::Char(value), KeyModifiers::NONE);
                    handled = handle_key_event(document, cursor, runtime, event);
                }
            }
            if handled {
                break;
            }
        }
    });
    handled
}

fn can_bulk_insert_paste(runtime: &EditorRuntime) -> bool {
    !runtime.overwrite_mode
        && !runtime.search_active
        && !runtime.goto_line_active
        && runtime.menu.is_none()
        && runtime.sidebar.is_none()
        && runtime.sidebar_prompt.is_none()
        && runtime.workspace_prompt.is_none()
        && runtime.workspace_manager.is_none()
        && runtime.command_palette.is_none()
}

fn can_bulk_overwrite_paste(runtime: &EditorRuntime, text: &str) -> bool {
    runtime.overwrite_mode
        && text.is_ascii()
        && !text.contains('\n')
        && !runtime.search_active
        && !runtime.goto_line_active
        && runtime.menu.is_none()
        && runtime.sidebar.is_none()
        && runtime.sidebar_prompt.is_none()
        && runtime.workspace_prompt.is_none()
        && runtime.workspace_manager.is_none()
        && runtime.command_palette.is_none()
}
