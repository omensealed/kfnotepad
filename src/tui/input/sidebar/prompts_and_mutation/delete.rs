pub(crate) fn delete_sidebar_entry(
    workspace: &EditorWorkspace<'_>,
    runtime: &mut EditorRuntime,
    entry: &FileSidebarEntry,
    recursive: bool,
) {
    if runtime.sidebar_query.trim() != "yes" {
        runtime.status = String::from("Delete cancelled; type yes to confirm");
        return;
    }

    if open_dirty_tab_uses_path(workspace, &entry.path) {
        runtime.status = String::from("Cannot delete an open modified file");
        return;
    }

    let metadata = match fs::symlink_metadata(&entry.path) {
        Ok(metadata) => metadata,
        Err(error) => {
            runtime.status = format!("Delete failed: {error}");
            return;
        }
    };
    if metadata.file_type().is_symlink() {
        runtime.status = String::from("Refusing to delete symlink");
        return;
    }

    let result = match entry.kind {
        FileSidebarEntryKind::File if metadata.is_file() => move_path_to_trash(&entry.path),
        FileSidebarEntryKind::Directory if metadata.is_dir() && recursive => {
            move_path_to_trash(&entry.path)
        }
        FileSidebarEntryKind::Parent => {
            runtime.status = String::from("Cannot delete parent entry");
            return;
        }
        _ => {
            runtime.status = String::from("Delete target changed type");
            return;
        }
    };

    match result {
        Ok(()) => {
            let deleted_label = entry.label.clone();
            refresh_sidebar_after_delete(runtime);
            runtime.sidebar_prompt = None;
            runtime.sidebar_query.clear();
            runtime.status = format!("Moved to trash {deleted_label}");
        }
        Err(error) => runtime.status = format!("Delete failed: {error}"),
    }
}
