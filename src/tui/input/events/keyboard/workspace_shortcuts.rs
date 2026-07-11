pub(crate) fn handle_workspace_key_event(
    workspace: &mut EditorWorkspace<'_>,
    runtime: &mut EditorRuntime,
    event: KeyEvent,
) -> bool {
    if runtime.sidebar.is_some()
        && event.modifiers.contains(KeyModifiers::CONTROL)
        && event.code == KeyCode::Enter
    {
        open_selected_sidebar_entry_in_new_tab(workspace, runtime);
        return true;
    }

    if runtime.sidebar.is_some()
        || runtime.menu.is_some()
        || runtime.command_palette.is_some()
        || runtime.goto_line_active
        || runtime.search_active
        || runtime.workspace_prompt.is_some()
        || runtime.workspace_manager.is_some()
    {
        return false;
    }

    match (event.modifiers, event.code) {
        (_, KeyCode::F(2)) => {
            open_command_palette(runtime);
            true
        }
        (KeyModifiers::CONTROL, KeyCode::Char('n')) => {
            create_new_file_tab(workspace, runtime);
            true
        }
        (KeyModifiers::CONTROL, KeyCode::F(4)) => {
            close_active_tab(workspace, runtime);
            true
        }
        (KeyModifiers::CONTROL, KeyCode::PageUp) => {
            select_previous_tab(workspace, runtime);
            true
        }
        (KeyModifiers::CONTROL, KeyCode::PageDown) => {
            select_next_tab(workspace, runtime);
            true
        }
        _ => false,
    }
}
