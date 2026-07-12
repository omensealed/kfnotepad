//! Undo and redo commands with cursor/runtime synchronization.

use super::*;

pub(crate) fn undo_document(
    document: &mut TextDocument,
    cursor: &mut Cursor,
    runtime: &mut EditorRuntime,
) {
    runtime.quit_confirmation_pending = false;
    runtime.status = match undo_document_edit(document, cursor) {
        UndoRedoResult::Applied => {
            stop_reader_mode_for_edit(runtime);
            String::from("Undone")
        }
        UndoRedoResult::NothingToApply => String::from("Nothing to undo"),
    };
}

pub(crate) fn redo_document(
    document: &mut TextDocument,
    cursor: &mut Cursor,
    runtime: &mut EditorRuntime,
) {
    runtime.quit_confirmation_pending = false;
    runtime.status = match redo_document_edit(document, cursor) {
        UndoRedoResult::Applied => {
            stop_reader_mode_for_edit(runtime);
            String::from("Redone")
        }
        UndoRedoResult::NothingToApply => String::from("Nothing to redo"),
    };
}
