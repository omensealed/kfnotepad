//! Indentation, paging, document-edge, and word navigation commands.

use super::*;

pub(crate) fn indent_at_cursor(
    document: &mut TextDocument,
    cursor: &mut Cursor,
    runtime: &mut EditorRuntime,
) {
    runtime.quit_confirmation_pending = false;
    for _ in 0..TAB_WIDTH {
        if document
            .buffer
            .insert_char(cursor.row, cursor.column, ' ')
            .is_err()
        {
            return;
        }
        cursor.column += 1;
    }
    stop_reader_mode_for_edit(runtime);
    runtime.status = String::from("Indented");
}

pub(crate) fn unindent_at_cursor(
    document: &mut TextDocument,
    cursor: &mut Cursor,
    runtime: &mut EditorRuntime,
) {
    runtime.quit_confirmation_pending = false;
    let Some(prefix) = document
        .buffer
        .line(cursor.row)
        .map(|line| line.chars().take(cursor.column).collect::<Vec<_>>())
    else {
        return;
    };
    let removable = prefix
        .iter()
        .rev()
        .take(TAB_WIDTH)
        .take_while(|character| **character == ' ')
        .count();

    if removable == 0 {
        runtime.status = String::from("No indentation to remove");
        return;
    }

    for _ in 0..removable {
        let delete_column = cursor.column.saturating_sub(1);
        if document
            .buffer
            .delete_char(cursor.row, delete_column)
            .is_err()
        {
            return;
        }
        cursor.column = delete_column;
    }
    stop_reader_mode_for_edit(runtime);
    runtime.status = String::from("Unindented");
}

pub(crate) fn page_up(document: &TextDocument, cursor: &mut Cursor, runtime: &mut EditorRuntime) {
    runtime.quit_confirmation_pending = false;
    shared_page_up(document, cursor, runtime.page_rows);
    runtime.status = String::from("Page up");
}

pub(crate) fn page_down(document: &TextDocument, cursor: &mut Cursor, runtime: &mut EditorRuntime) {
    runtime.quit_confirmation_pending = false;
    shared_page_down(document, cursor, runtime.page_rows);
    runtime.status = String::from("Page down");
}

pub(crate) fn go_to_document_start(cursor: &mut Cursor, runtime: &mut EditorRuntime) {
    runtime.quit_confirmation_pending = false;
    shared_go_to_document_start(cursor);
    runtime.status = String::from("Top");
}

pub(crate) fn go_to_document_end(
    document: &TextDocument,
    cursor: &mut Cursor,
    runtime: &mut EditorRuntime,
) {
    runtime.quit_confirmation_pending = false;
    shared_go_to_document_end(document, cursor);
    runtime.status = String::from("Bottom");
}

pub(crate) fn go_to_previous_word(
    document: &TextDocument,
    cursor: &mut Cursor,
    runtime: &mut EditorRuntime,
) {
    runtime.quit_confirmation_pending = false;
    move_document_cursor(document, cursor, CursorMove::WordLeft);
    runtime.status = String::from("Previous word");
}

pub(crate) fn go_to_next_word(
    document: &TextDocument,
    cursor: &mut Cursor,
    runtime: &mut EditorRuntime,
) {
    runtime.quit_confirmation_pending = false;
    move_document_cursor(document, cursor, CursorMove::WordRight);
    runtime.status = String::from("Next word");
}
