//! Named and automatic workspace project saves.

use super::*;

pub(crate) fn save_workspace_project_named(
    workspace: &EditorWorkspace<'_>,
    runtime: &mut EditorRuntime,
    project_name: &str,
    status_name: &str,
) {
    let Some(projects_dir) = runtime.workspace_projects_dir.clone() else {
        runtime.status = String::from("Workspace save failed: cannot resolve config directory");
        return;
    };
    let Some(path) = gui_workspace_project_path(&projects_dir, project_name) else {
        runtime.status = String::from("Workspace save failed: invalid project name");
        return;
    };
    let Some(project) = current_tui_workspace_project(workspace, project_name) else {
        runtime.status = String::from("Workspace save failed: no files to save");
        return;
    };

    match save_gui_workspace_project(&path, &project) {
        Ok(()) => runtime.status = format!("Workspace saved: {status_name}"),
        Err(error) => runtime.status = format!("Workspace save failed: {error}"),
    }
}

pub(crate) fn autosave_tui_current_workspace(
    workspace: &EditorWorkspace<'_>,
    runtime: &mut EditorRuntime,
) {
    if !runtime.settings.gui_restore_last_workspace {
        return;
    }
    let Some(projects_dir) = runtime.workspace_projects_dir.clone() else {
        return;
    };
    let Some(path) = gui_workspace_project_path(&projects_dir, TUI_CURRENT_WORKSPACE_NAME) else {
        return;
    };
    let Some(project) = current_tui_workspace_project(workspace, TUI_CURRENT_WORKSPACE_NAME) else {
        return;
    };

    if let Err(error) = save_gui_workspace_project(&path, &project) {
        runtime.status = format!("{}; workspace autosave failed: {error}", runtime.status);
    }
}
