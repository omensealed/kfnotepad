//! Typed-character insertion and overwrite behavior.

use super::*;

pub(crate) fn insert_typed_character(
    document: &mut TextDocument,
    cursor: &mut Cursor,
    runtime: &mut EditorRuntime,
    value: char,
) {
    insert_typed_character_internal(document, cursor, runtime, value);
}

fn insert_typed_character_internal(
    document: &mut TextDocument,
    cursor: &mut Cursor,
    runtime: &mut EditorRuntime,
    value: char,
) -> bool {
    runtime.quit_confirmation_pending = false;
    let result = if runtime.overwrite_mode {
        document
            .buffer
            .replace_char(cursor.row, cursor.column, value)
    } else {
        document
            .buffer
            .insert_char(cursor.row, cursor.column, value)
    };
    match result {
        Ok(()) => {
            let inserted_end = cursor.column.saturating_add(1);
            cursor.column = document
                .buffer
                .grapheme_range_end_boundary_column(cursor.row, inserted_end)
                .unwrap_or(inserted_end);
            stop_reader_mode_for_edit(runtime);
            runtime.status = if runtime.overwrite_mode {
                String::from("Modified overwrite")
            } else {
                String::from("Modified")
            };
            true
        }
        Err(BufferError::TooLarge { limit, .. }) => {
            runtime.status = format!("Document reached {limit} byte limit");
            false
        }
        Err(_) => false,
    }
}
