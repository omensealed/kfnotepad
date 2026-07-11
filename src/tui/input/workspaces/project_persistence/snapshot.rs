pub(crate) fn current_tui_workspace_project(
    workspace: &EditorWorkspace<'_>,
    project_name: &str,
) -> Option<GuiWorkspaceProject> {
    let files = workspace
        .tabs
        .iter()
        .map(|tab| tab.document.as_ref().path.clone())
        .collect::<Vec<_>>();
    if files.is_empty() {
        return None;
    }
    Some(GuiWorkspaceProject {
        name: project_name.to_string(),
        files,
        active_ordinal: workspace.active.min(workspace.tabs.len().saturating_sub(1)),
        layout: None,
    })
}
