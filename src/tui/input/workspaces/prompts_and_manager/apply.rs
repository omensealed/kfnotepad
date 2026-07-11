pub(crate) fn apply_workspace_prompt(
    workspace: &mut EditorWorkspace<'_>,
    runtime: &mut EditorRuntime,
) {
    match runtime.workspace_prompt {
        Some(WorkspacePrompt::SaveNamed) => {
            let name = runtime.workspace_query.trim().to_string();
            if name.is_empty() {
                runtime.status = String::from("Workspace name is empty");
                return;
            }
            save_workspace_project_named(workspace, runtime, &name, &name);
            runtime.workspace_prompt = None;
            runtime.workspace_query.clear();
            runtime.workspace_prompt_candidates.clear();
            runtime.workspace_prompt_candidate_index = None;
        }
        Some(WorkspacePrompt::OpenNamed) => {
            let name = runtime.workspace_query.trim().to_string();
            if name.is_empty() {
                runtime.status = String::from("Workspace name is empty");
                return;
            }
            open_workspace_project_named(workspace, runtime, &name);
        }
        Some(WorkspacePrompt::DeleteNamed) => {
            let name = runtime.workspace_query.trim().to_string();
            if name.is_empty() {
                runtime.status = String::from("Workspace name is empty");
                return;
            }
            prepare_delete_workspace_project(runtime, &name);
        }
        Some(WorkspacePrompt::ConfirmOpen) => {
            if runtime.workspace_query.trim() != "yes" {
                runtime.status = String::from("Workspace open cancelled; type yes to confirm");
                return;
            }
            let Some(project) = runtime.workspace_pending_open.clone() else {
                runtime.status = String::from("No workspace pending");
                runtime.workspace_prompt = None;
                runtime.workspace_query.clear();
                return;
            };
            replace_workspace_from_project(workspace, runtime, &project);
        }
        Some(WorkspacePrompt::ConfirmDelete) => {
            confirm_delete_workspace_project(runtime);
        }
        None => {}
    }
}
