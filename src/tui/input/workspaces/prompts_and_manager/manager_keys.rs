pub(crate) fn handle_workspace_manager_key_event(
    workspace: &mut EditorWorkspace<'_>,
    runtime: &mut EditorRuntime,
    event: KeyEvent,
) {
    match event.code {
        KeyCode::Esc => {
            runtime.workspace_manager = None;
            runtime.status = String::from("Workspace manager closed");
        }
        KeyCode::Up => move_workspace_manager_selection(runtime, -1),
        KeyCode::Down => move_workspace_manager_selection(runtime, 1),
        KeyCode::PageUp => move_workspace_manager_selection(runtime, -5),
        KeyCode::PageDown => move_workspace_manager_selection(runtime, 5),
        KeyCode::Home => set_workspace_manager_selection(runtime, 0),
        KeyCode::End => {
            let last = runtime
                .workspace_manager
                .as_ref()
                .map(|manager| manager.entries.len().saturating_sub(1))
                .unwrap_or(0);
            set_workspace_manager_selection(runtime, last);
        }
        KeyCode::Enter => {
            if let Some(name) = selected_workspace_manager_name(runtime) {
                runtime.workspace_manager = None;
                open_workspace_project_named(workspace, runtime, &name);
            } else {
                runtime.status = String::from("No workspace selected");
            }
        }
        KeyCode::Delete | KeyCode::Char('d') | KeyCode::Char('D') => {
            if let Some(name) = selected_workspace_manager_name(runtime) {
                runtime.workspace_manager = None;
                prepare_delete_workspace_project(runtime, &name);
            } else {
                runtime.status = String::from("No workspace selected");
            }
        }
        KeyCode::Char('s') | KeyCode::Char('S') => {
            if let Some(name) = selected_workspace_manager_name(runtime) {
                runtime.workspace_manager = None;
                save_workspace_project_named(workspace, runtime, &name, &name);
            } else {
                runtime.status = String::from("No workspace selected");
            }
        }
        KeyCode::Char('n') | KeyCode::Char('N') => {
            runtime.workspace_manager = None;
            start_workspace_save_prompt(runtime);
        }
        _ => {}
    }
}

pub(crate) fn selected_workspace_manager_name(runtime: &EditorRuntime) -> Option<String> {
    runtime
        .workspace_manager
        .as_ref()
        .and_then(|manager| manager.entries.get(manager.selected))
        .map(|entry| entry.name.clone())
}

pub(crate) fn move_workspace_manager_selection(runtime: &mut EditorRuntime, delta: isize) {
    let Some(manager) = runtime.workspace_manager.as_ref() else {
        return;
    };
    if manager.entries.is_empty() {
        runtime.status = String::from("No workspace projects saved; press N to save a new project");
        return;
    }
    let len = manager.entries.len();
    let selected = manager.selected;
    let next = if delta.is_negative() {
        selected.saturating_sub(delta.unsigned_abs())
    } else {
        (selected + delta as usize).min(len.saturating_sub(1))
    };
    set_workspace_manager_selection(runtime, next);
}

pub(crate) fn set_workspace_manager_selection(runtime: &mut EditorRuntime, selected: usize) {
    let Some(manager) = runtime.workspace_manager.as_mut() else {
        return;
    };
    if manager.entries.is_empty() {
        manager.selected = 0;
        manager.scroll = 0;
        runtime.status = String::from("No workspace projects saved; press N to save a new project");
        return;
    }
    manager.selected = selected.min(manager.entries.len().saturating_sub(1));
    manager.scroll = manager.selected.saturating_sub(6);
    if let Some(entry) = manager.entries.get(manager.selected) {
        runtime.status = format!(
            "Workspace: {} (Enter open | S save over | D delete)",
            entry.name
        );
    }
}
