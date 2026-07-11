pub(crate) fn handle_sidebar_key_event(
    document: &mut TextDocument,
    cursor: &mut Cursor,
    runtime: &mut EditorRuntime,
    event: KeyEvent,
) -> bool {
    match event.code {
        KeyCode::Esc => {
            close_file_sidebar(runtime);
            runtime.status = String::from("Files closed");
        }
        KeyCode::Up => select_previous_sidebar_entry(runtime),
        KeyCode::Down => select_next_sidebar_entry(runtime),
        KeyCode::Enter => activate_selected_sidebar_entry(document, cursor, runtime),
        KeyCode::Char('b') if event.modifiers.contains(KeyModifiers::CONTROL) => {
            close_file_sidebar(runtime);
            runtime.status = String::from("Files closed");
        }
        _ => {}
    }
    false
}

pub(crate) fn handle_workspace_sidebar_key_event(
    workspace: &mut EditorWorkspace<'_>,
    runtime: &mut EditorRuntime,
    event: KeyEvent,
) {
    if runtime.sidebar_prompt.is_some() {
        handle_sidebar_prompt_key_event(workspace, runtime, event);
        return;
    }

    match event.code {
        KeyCode::Esc => {
            close_file_sidebar(runtime);
            runtime.status = String::from("Files closed");
        }
        KeyCode::Up => select_previous_sidebar_entry(runtime),
        KeyCode::Down => select_next_sidebar_entry(runtime),
        KeyCode::Enter => {
            activate_selected_sidebar_entry_for_workspace(workspace, runtime);
        }
        KeyCode::Char('b') if event.modifiers.contains(KeyModifiers::CONTROL) => {
            close_file_sidebar(runtime);
            runtime.status = String::from("Files closed");
        }
        KeyCode::Char('n') if event.modifiers.contains(KeyModifiers::CONTROL) => {
            start_sidebar_create_file(runtime);
        }
        KeyCode::Char('d') if event.modifiers.contains(KeyModifiers::CONTROL) => {
            start_sidebar_create_directory(runtime);
        }
        KeyCode::Delete => {
            start_sidebar_delete(runtime);
        }
        _ => {}
    }
}

pub(crate) fn handle_sidebar_prompt_key_event(
    workspace: &mut EditorWorkspace<'_>,
    runtime: &mut EditorRuntime,
    event: KeyEvent,
) {
    match event.code {
        KeyCode::Esc => {
            runtime.sidebar_prompt = None;
            runtime.sidebar_query.clear();
            runtime.status = String::from("Files prompt cancelled");
        }
        KeyCode::Backspace => {
            runtime.sidebar_query.pop();
            refresh_sidebar_prompt_status(runtime);
        }
        KeyCode::Enter => {
            apply_sidebar_prompt(workspace, runtime);
        }
        KeyCode::Char(value)
            if event.modifiers.is_empty() || event.modifiers == KeyModifiers::SHIFT =>
        {
            runtime.sidebar_query.push(value);
            refresh_sidebar_prompt_status(runtime);
        }
        _ => {}
    }
}
