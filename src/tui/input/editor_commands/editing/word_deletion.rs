pub(crate) fn delete_previous_word(
    document: &mut TextDocument,
    cursor: &mut Cursor,
    runtime: &mut EditorRuntime,
) {
    runtime.quit_confirmation_pending = false;
    if shared_delete_previous_word(document, cursor) == EditResult::Modified {
        stop_reader_mode_for_edit(runtime);
        runtime.status = String::from("Modified");
    }
}

pub(crate) fn delete_next_word(
    document: &mut TextDocument,
    cursor: &mut Cursor,
    runtime: &mut EditorRuntime,
) {
    runtime.quit_confirmation_pending = false;
    if shared_delete_next_word(document, cursor) == EditResult::Modified {
        stop_reader_mode_for_edit(runtime);
        runtime.status = String::from("Modified");
    }
}

pub(crate) fn delete_to_line_end(
    document: &mut TextDocument,
    cursor: &mut Cursor,
    runtime: &mut EditorRuntime,
) {
    runtime.quit_confirmation_pending = false;
    if shared_delete_to_line_end(document, cursor) == EditResult::Modified {
        stop_reader_mode_for_edit(runtime);
        runtime.status = String::from("Modified");
    }
}
