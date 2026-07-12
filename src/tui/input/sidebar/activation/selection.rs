//! Selected-entry activation routing for documents and workspaces.

use super::*;

pub(crate) fn activate_selected_sidebar_entry(
    document: &mut TextDocument,
    cursor: &mut Cursor,
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
    activate_sidebar_entry(document, cursor, runtime, entry);
}

pub(crate) fn activate_selected_sidebar_entry_for_workspace(
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
    activate_sidebar_entry_for_workspace(workspace, runtime, entry);
}

pub(crate) fn activate_sidebar_entry_at_mouse_for_workspace(
    workspace: &mut EditorWorkspace<'_>,
    runtime: &mut EditorRuntime,
    row: u16,
) {
    let Some(sidebar) = runtime.sidebar.as_mut() else {
        return;
    };
    if row == 0 {
        close_file_sidebar(runtime);
        runtime.status = String::from("Files closed");
        return;
    }
    let Some(entry) = sidebar.selected_entry_for_mouse_row(row) else {
        return;
    };
    activate_sidebar_entry_for_workspace(workspace, runtime, entry);
}
