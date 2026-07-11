pub fn resolve_editor_config_path(
    xdg_config_home: Option<&Path>,
    home: Option<&Path>,
) -> Option<PathBuf> {
    if let Some(xdg_config_home) = non_empty(xdg_config_home) {
        return Some(xdg_config_home.join("kfnotepad").join("config.toml"));
    }

    non_empty(home).map(|home| home.join(".config").join("kfnotepad").join("config.toml"))
}

pub fn resolve_gui_layout_path(
    xdg_config_home: Option<&Path>,
    home: Option<&Path>,
) -> Option<PathBuf> {
    if let Some(xdg_config_home) = non_empty(xdg_config_home) {
        return Some(xdg_config_home.join("kfnotepad").join("gui-layout.v1"));
    }

    non_empty(home).map(|home| home.join(".config").join("kfnotepad").join("gui-layout.v1"))
}

pub fn resolve_gui_workspace_projects_dir(
    xdg_config_home: Option<&Path>,
    home: Option<&Path>,
) -> Option<PathBuf> {
    if let Some(xdg_config_home) = non_empty(xdg_config_home) {
        return Some(xdg_config_home.join("kfnotepad").join("workspaces"));
    }

    non_empty(home).map(|home| home.join(".config").join("kfnotepad").join("workspaces"))
}

pub fn resolve_managed_notes_dir(
    xdg_data_home: Option<&Path>,
    home: Option<&Path>,
) -> Result<PathBuf, ManagedNotesError> {
    if let Some(xdg_data_home) = non_empty(xdg_data_home) {
        return Ok(xdg_data_home.join("kfnotepad").join("notes"));
    }

    non_empty(home)
        .map(|home| {
            home.join(".local")
                .join("share")
                .join("kfnotepad")
                .join("notes")
        })
        .ok_or(ManagedNotesError::MissingDataHome)
}
