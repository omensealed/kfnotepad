pub(crate) fn open_workspace_manager(runtime: &mut EditorRuntime) {
    let Some(projects_dir) = runtime.workspace_projects_dir.as_deref() else {
        runtime.status = String::from("Workspaces unavailable: cannot resolve config directory");
        return;
    };
    match list_gui_workspace_projects(projects_dir) {
        Ok(projects) => {
            runtime.workspace_prompt = None;
            runtime.workspace_query.clear();
            runtime.workspace_pending_open = None;
            runtime.workspace_pending_delete = None;
            runtime.workspace_prompt_candidates.clear();
            runtime.workspace_prompt_candidate_index = None;
            runtime.workspace_open_confirmation_pending = false;
            runtime.workspace_manager = Some(WorkspaceManagerState {
                entries: projects
                    .into_iter()
                    .map(|entry| WorkspaceManagerEntry {
                        name: entry.project.name,
                        files: entry.project.files.len(),
                    })
                    .collect(),
                selected: 0,
                scroll: 0,
            });
            runtime.status = if runtime
                .workspace_manager
                .as_ref()
                .is_some_and(|manager| manager.entries.is_empty())
            {
                String::from("No workspace projects saved; press N to save a new project")
            } else {
                String::from("Workspace manager: Enter open | S save over | D delete | N new | Esc")
            };
        }
        Err(error) => runtime.status = format!("Workspace list failed: {error}"),
    }
}
