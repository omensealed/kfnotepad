pub(crate) fn activate_sidebar_entry(
    document: &mut TextDocument,
    cursor: &mut Cursor,
    runtime: &mut EditorRuntime,
    entry: FileSidebarEntry,
) {
    match entry.kind {
        FileSidebarEntryKind::Parent | FileSidebarEntryKind::Directory => {
            match FileSidebarState::load(entry.path) {
                Ok(sidebar) => {
                    runtime.last_sidebar_dir = Some(sidebar.current_dir.clone());
                    runtime.sidebar = Some(sidebar);
                    runtime.status = String::from("Files");
                }
                Err(error) => runtime.status = format!("Files unavailable: {error}"),
            }
        }
        FileSidebarEntryKind::File => {
            if document.buffer.is_dirty() {
                runtime.status = String::from("Save before opening another file");
                return;
            }
            match open_text_file(&entry.path) {
                Ok(next_document) => {
                    *document = next_document;
                    *cursor = Cursor { row: 0, column: 0 };
                    close_file_sidebar(runtime);
                    runtime.search_active = false;
                    runtime.goto_line_active = false;
                    stop_reader_mode(runtime, "Reader mode stopped for file open");
                    runtime.status = format!("Opened {}", entry.label);
                }
                Err(error) => runtime.status = format!("Open failed: {error}"),
            }
        }
    }
}
