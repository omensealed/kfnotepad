pub(crate) fn start_sidebar_create_file(runtime: &mut EditorRuntime) {
    runtime.sidebar_prompt = Some(SidebarPrompt::CreateFile);
    runtime.sidebar_query.clear();
    runtime.status = String::from("New file name: ");
}

pub(crate) fn start_sidebar_create_directory(runtime: &mut EditorRuntime) {
    runtime.sidebar_prompt = Some(SidebarPrompt::CreateDirectory);
    runtime.sidebar_query.clear();
    runtime.status = String::from("New directory name: ");
}

pub(crate) fn start_sidebar_delete(runtime: &mut EditorRuntime) {
    let Some(entry) = runtime
        .sidebar
        .as_ref()
        .and_then(FileSidebarState::selected_entry)
        .cloned()
    else {
        runtime.status = String::from("No file selected");
        return;
    };

    if entry.kind == FileSidebarEntryKind::Parent {
        runtime.status = String::from("Cannot delete parent entry");
        return;
    }

    let recursive = entry.kind == FileSidebarEntryKind::Directory;
    runtime.sidebar_prompt = Some(SidebarPrompt::DeleteConfirm { entry, recursive });
    runtime.sidebar_query.clear();
    runtime.status = if recursive {
        String::from("Delete directory and all contents? type yes: ")
    } else {
        String::from("Delete file? type yes: ")
    };
}
