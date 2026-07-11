pub(crate) fn replace_workspace_from_project(
    workspace: &mut EditorWorkspace<'_>,
    runtime: &mut EditorRuntime,
    project: &GuiWorkspaceProject,
) {
    match workspace_from_project_documents(project, env::current_dir().unwrap_or_default()) {
        Ok(restored) => {
            let status = restored
                .status_message()
                .unwrap_or_else(|| format!("Opened workspace: {}", project.name));
            *workspace = restored.workspace;
            runtime.workspace_prompt = None;
            runtime.workspace_query.clear();
            runtime.workspace_pending_open = None;
            runtime.workspace_pending_delete = None;
            runtime.workspace_prompt_candidates.clear();
            runtime.workspace_prompt_candidate_index = None;
            runtime.workspace_open_confirmation_pending = false;
            runtime.workspace_manager = None;
            close_file_sidebar(runtime);
            runtime.search_active = false;
            runtime.goto_line_active = false;
            runtime.quit_confirmation_pending = false;
            runtime.close_tab_confirmation_pending = false;
            stop_reader_mode(runtime, "Reader mode stopped for workspace open");
            runtime.status = status;
            autosave_tui_current_workspace(workspace, runtime);
        }
        Err(error) => {
            runtime.status = format!("Workspace open failed: {error}");
        }
    }
}
