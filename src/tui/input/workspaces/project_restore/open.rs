//! Named project opening and dirty-workspace checks.

use super::*;

pub(crate) fn open_workspace_project_named(
    workspace: &mut EditorWorkspace<'_>,
    runtime: &mut EditorRuntime,
    name: &str,
) {
    let Some(projects_dir) = runtime.workspace_projects_dir.clone() else {
        runtime.status = String::from("Workspace open failed: cannot resolve config directory");
        return;
    };
    let Some(path) = gui_workspace_project_path(&projects_dir, name) else {
        runtime.status = String::from("Workspace open failed: invalid project name");
        return;
    };

    let project = match load_tui_workspace_project(&path) {
        Ok(project) => project,
        Err(error) => {
            runtime.status = format!("Workspace open failed: {error}");
            return;
        }
    };

    if workspace_has_dirty_tabs(workspace) && !runtime.workspace_open_confirmation_pending {
        runtime.workspace_pending_open = Some(project);
        runtime.workspace_prompt = Some(WorkspacePrompt::ConfirmOpen);
        runtime.workspace_query.clear();
        runtime.workspace_open_confirmation_pending = true;
        runtime.status = String::from("Replace dirty workspace? type yes: ");
        return;
    }

    replace_workspace_from_project(workspace, runtime, &project);
}

pub(crate) fn workspace_has_dirty_tabs(workspace: &EditorWorkspace<'_>) -> bool {
    workspace
        .tabs
        .iter()
        .any(|tab| tab.document.as_ref().buffer.is_dirty())
}
