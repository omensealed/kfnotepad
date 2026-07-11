#[cfg(test)]
mod tests {
    use super::*;

    use std::path::Path;

    #[test]
    fn editor_config_path_prefers_config_home_and_falls_back() {
        let xdg = Path::new("/tmp/test-xdg");
        let home = Path::new("/tmp/home");

        assert_eq!(
            resolve_editor_config_path(Some(xdg), Some(home)),
            Some(xdg.join("kfnotepad").join("config.toml"))
        );
        assert_eq!(
            resolve_editor_config_path(None, Some(home)),
            Some(home.join(".config").join("kfnotepad").join("config.toml"))
        );
        assert!(resolve_editor_config_path(None, None).is_none());
        assert_eq!(
            resolve_editor_config_path(Some(Path::new("")), Some(home)),
            Some(home.join(".config").join("kfnotepad").join("config.toml"))
        );
    }

    #[test]
    fn gui_layout_path_prefers_config_home_and_falls_back() {
        let xdg = Path::new("/tmp/test-xdg");
        let home = Path::new("/tmp/home");

        assert_eq!(
            resolve_gui_layout_path(Some(xdg), Some(home)),
            Some(xdg.join("kfnotepad").join("gui-layout.v1"))
        );
        assert_eq!(
            resolve_gui_layout_path(None, Some(home)),
            Some(home.join(".config").join("kfnotepad").join("gui-layout.v1"))
        );
        assert!(resolve_gui_layout_path(None, None).is_none());
    }

    #[test]
    fn workspace_projects_path_prefers_config_home_and_falls_back() {
        let xdg = Path::new("/tmp/test-xdg");
        let home = Path::new("/tmp/home");

        assert_eq!(
            resolve_gui_workspace_projects_dir(Some(xdg), Some(home)),
            Some(xdg.join("kfnotepad").join("workspaces"))
        );
        assert_eq!(
            resolve_gui_workspace_projects_dir(None, Some(home)),
            Some(home.join(".config").join("kfnotepad").join("workspaces"))
        );
        assert!(resolve_gui_workspace_projects_dir(None, None).is_none());
    }

    #[test]
    fn managed_notes_path_prefers_data_home_and_requires_data_or_home() {
        let xdg = Path::new("/tmp/test-data");
        let home = Path::new("/tmp/home");

        assert_eq!(
            resolve_managed_notes_dir(Some(xdg), Some(home)).expect("xdg data"),
            xdg.join("kfnotepad").join("notes")
        );
        assert_eq!(
            resolve_managed_notes_dir(None, Some(home)).expect("home data"),
            home.join(".local")
                .join("share")
                .join("kfnotepad")
                .join("notes")
        );
        assert!(resolve_managed_notes_dir(None, None).is_err());
        assert_eq!(
            resolve_managed_notes_dir(Some(Path::new("")), Some(home)).expect("home data"),
            home.join(".local")
                .join("share")
                .join("kfnotepad")
                .join("notes")
        );
    }
}
