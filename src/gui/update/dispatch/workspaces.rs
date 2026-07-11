fn handle_workspace_project_clicked(state: &mut KfnotepadGui, index: usize) {
    state.open_workspace_project_in_current_window(index);
}

fn handle_workspace_project_new_window_clicked(state: &mut KfnotepadGui, index: usize) {
    state.open_workspace_project_in_new_window(index);
}

fn handle_workspace_project_delete_clicked(state: &mut KfnotepadGui, index: usize) {
    state.delete_workspace_project(index);
}

fn handle_workspace_project_name_changed(state: &mut KfnotepadGui, name: String) {
    state.workspace_project_name = name;
}

fn handle_quit_latest_window_missing(state: &mut KfnotepadGui) {
    state.status_message = "quit failed: no active window".to_string();
}
