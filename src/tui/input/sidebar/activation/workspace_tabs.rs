pub(crate) fn activate_sidebar_entry_for_workspace(
    workspace: &mut EditorWorkspace<'_>,
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
            focus_or_open_file_tab(workspace, runtime, &entry.path, &entry.label);
        }
    }
}

pub(crate) fn focus_or_open_file_tab(
    workspace: &mut EditorWorkspace<'_>,
    runtime: &mut EditorRuntime,
    path: &Path,
    label: &str,
) {
    if let Some(index) = workspace
        .tabs
        .iter()
        .position(|tab| tab.document.as_ref().path == path)
    {
        workspace.active = index;
        close_file_sidebar(runtime);
        runtime.search_active = false;
        runtime.goto_line_active = false;
        runtime.quit_confirmation_pending = false;
        runtime.close_tab_confirmation_pending = false;
        stop_reader_mode(runtime, "Reader mode stopped for tab focus");
        runtime.status = format!("Focused tab {label}");
        autosave_tui_current_workspace(workspace, runtime);
        return;
    }

    match open_text_file(path) {
        Ok(document) => {
            workspace.push_owned_tab(document);
            close_file_sidebar(runtime);
            runtime.search_active = false;
            runtime.goto_line_active = false;
            runtime.quit_confirmation_pending = false;
            runtime.close_tab_confirmation_pending = false;
            stop_reader_mode(runtime, "Reader mode stopped for file open");
            runtime.status = format!("Opened tab {label}");
            autosave_tui_current_workspace(workspace, runtime);
        }
        Err(error) => runtime.status = format!("Open failed: {error}"),
    }
}

pub(crate) fn open_selected_sidebar_entry_in_new_tab(
    workspace: &mut EditorWorkspace<'_>,
    runtime: &mut EditorRuntime,
) {
    let Some(entry) = runtime
        .sidebar
        .as_ref()
        .and_then(FileSidebarState::selected_entry)
        .cloned()
    else {
        return;
    };

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
            focus_or_open_file_tab(workspace, runtime, &entry.path, &entry.label);
        }
    }
}
