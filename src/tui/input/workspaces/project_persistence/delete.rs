//! Workspace project delete preparation and confirmation.

use super::*;

pub(crate) fn prepare_delete_workspace_project(runtime: &mut EditorRuntime, name: &str) {
    let Some(projects_dir) = runtime.workspace_projects_dir.clone() else {
        runtime.status = String::from("Workspace delete failed: cannot resolve config directory");
        return;
    };
    let Some(path) = gui_workspace_project_path(&projects_dir, name) else {
        runtime.status = String::from("Workspace delete failed: invalid project name");
        return;
    };
    if !path.exists() {
        runtime.status = format!("Workspace not found: {name}");
        return;
    }

    runtime.workspace_pending_delete = Some((name.to_string(), path));
    runtime.workspace_prompt = Some(WorkspacePrompt::ConfirmDelete);
    runtime.workspace_query.clear();
    runtime.workspace_prompt_candidates.clear();
    runtime.workspace_prompt_candidate_index = None;
    runtime.status = String::from("Delete workspace project? type yes: ");
}

pub(crate) fn confirm_delete_workspace_project(runtime: &mut EditorRuntime) {
    if runtime.workspace_query.trim() != "yes" {
        runtime.status = String::from("Workspace delete cancelled; type yes to confirm");
        return;
    }
    let Some((name, path)) = runtime.workspace_pending_delete.clone() else {
        runtime.status = String::from("No workspace pending delete");
        runtime.workspace_prompt = None;
        runtime.workspace_query.clear();
        return;
    };
    let Some(projects_dir) = runtime.workspace_projects_dir.as_deref() else {
        runtime.status = String::from("Workspace delete failed: cannot resolve config directory");
        return;
    };

    match delete_gui_workspace_project(projects_dir, &path) {
        Ok(GuiWorkspaceProjectDeleteResult::Deleted) => {
            runtime.workspace_prompt = None;
            runtime.workspace_query.clear();
            runtime.workspace_pending_delete = None;
            runtime.workspace_prompt_candidates.clear();
            runtime.workspace_prompt_candidate_index = None;
            runtime.status = format!("Moved workspace to trash: {name}");
        }
        Ok(GuiWorkspaceProjectDeleteResult::Missing) => {
            runtime.workspace_prompt = None;
            runtime.workspace_query.clear();
            runtime.workspace_pending_delete = None;
            runtime.status = format!("Workspace already missing: {name}");
        }
        Err(error) => runtime.status = format!("Workspace delete failed: {error}"),
    }
}
