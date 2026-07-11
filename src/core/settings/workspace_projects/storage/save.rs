pub fn save_gui_workspace_project(
    path: &Path,
    project: &GuiWorkspaceProject,
) -> Result<(), EditorConfigError> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|source| EditorConfigError::CreateDir {
            path: parent.to_path_buf(),
            source,
        })?;
        set_private_config_dir_permissions(parent).map_err(|source| {
            EditorConfigError::CreateDir {
                path: parent.to_path_buf(),
                source,
            }
        })?;
    }

    let text =
        serialize_gui_workspace_project(project).ok_or_else(|| EditorConfigError::Invalid {
            path: path.to_path_buf(),
            message: "invalid GUI workspace project snapshot".to_string(),
        })?;
    let temp_path = temporary_config_path(path);
    let result = write_config_temp_then_rename(path, &temp_path, text.as_bytes());
    if result.is_err() {
        let _ = fs::remove_file(&temp_path);
    }
    result
}

pub fn gui_workspace_project_path(projects_dir: &Path, name: &str) -> Option<PathBuf> {
    Some(projects_dir.join(format!("{}.v1", gui_workspace_project_slug(name)?)))
}
