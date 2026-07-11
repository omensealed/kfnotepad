pub fn editor_config_path(xdg_config_home: Option<&Path>, home: Option<&Path>) -> Option<PathBuf> {
    resolve_editor_config_path(xdg_config_home, home)
}

pub fn gui_layout_path(xdg_config_home: Option<&Path>, home: Option<&Path>) -> Option<PathBuf> {
    resolve_gui_layout_path(xdg_config_home, home)
}

pub fn gui_workspace_projects_dir(
    xdg_config_home: Option<&Path>,
    home: Option<&Path>,
) -> Option<PathBuf> {
    resolve_gui_workspace_projects_dir(xdg_config_home, home)
}
