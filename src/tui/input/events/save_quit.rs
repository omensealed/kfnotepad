pub(crate) fn request_quit(document: &TextDocument, runtime: &mut EditorRuntime) -> bool {
    if document.buffer.is_dirty() {
        if runtime.quit_confirmation_pending {
            return true;
        }
        runtime.quit_confirmation_pending = true;
        runtime.status = String::from("Unsaved changes. Press Ctrl-Q again to quit.");
        return false;
    }
    true
}

pub(crate) fn save_document(document: &mut TextDocument, runtime: &mut EditorRuntime) {
    match save_text_document(document) {
        Ok(()) => {
            runtime.quit_confirmation_pending = false;
            runtime.status = String::from("Saved");
        }
        Err(error) => runtime.status = format!("Save failed: {error}"),
    }
}
