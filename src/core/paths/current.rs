pub fn current_editor_config_path() -> Option<PathBuf> {
    resolve_editor_config_path(
        platform_config_dir().as_deref(),
        platform_home_dir().as_deref(),
    )
}

pub fn current_gui_layout_path() -> Option<PathBuf> {
    resolve_gui_layout_path(
        platform_config_dir().as_deref(),
        platform_home_dir().as_deref(),
    )
}

pub fn current_gui_workspace_projects_dir() -> Option<PathBuf> {
    resolve_gui_workspace_projects_dir(
        platform_config_dir().as_deref(),
        platform_home_dir().as_deref(),
    )
}

pub fn current_managed_notes_dir() -> Result<PathBuf, ManagedNotesError> {
    resolve_managed_notes_dir(
        platform_data_dir().as_deref(),
        platform_home_dir().as_deref(),
    )
}
