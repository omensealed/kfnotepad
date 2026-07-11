pub(crate) fn select_previous_tab(
    workspace: &mut EditorWorkspace<'_>,
    runtime: &mut EditorRuntime,
) {
    runtime.quit_confirmation_pending = false;
    runtime.close_tab_confirmation_pending = false;
    if !workspace.select_previous_tab() {
        runtime.status = String::from("Only one tab open");
        return;
    }
    stop_reader_mode(runtime, "Reader mode stopped for tab switch");
    runtime.status = active_tab_status(workspace);
    autosave_tui_current_workspace(workspace, runtime);
}

pub(crate) fn select_next_tab(workspace: &mut EditorWorkspace<'_>, runtime: &mut EditorRuntime) {
    runtime.quit_confirmation_pending = false;
    runtime.close_tab_confirmation_pending = false;
    if !workspace.select_next_tab() {
        runtime.status = String::from("Only one tab open");
        return;
    }
    stop_reader_mode(runtime, "Reader mode stopped for tab switch");
    runtime.status = active_tab_status(workspace);
    autosave_tui_current_workspace(workspace, runtime);
}

pub(crate) fn close_active_tab(workspace: &mut EditorWorkspace<'_>, runtime: &mut EditorRuntime) {
    runtime.quit_confirmation_pending = false;
    match workspace.close_active_tab(runtime.close_tab_confirmation_pending) {
        CloseActiveTabResult::OnlyTab => {
            runtime.close_tab_confirmation_pending = false;
            runtime.status = String::from("Cannot close the only tab");
        }
        CloseActiveTabResult::Dirty => {
            runtime.close_tab_confirmation_pending = true;
            runtime.status = String::from("Unsaved changes. Press Ctrl-F4 again to close tab.");
        }
        CloseActiveTabResult::Closed { path } => {
            runtime.close_tab_confirmation_pending = false;
            stop_reader_mode(runtime, "Reader mode stopped for tab close");
            runtime.status = format!("Closed tab: {}", display_file_name(&path));
            autosave_tui_current_workspace(workspace, runtime);
        }
    }
}
