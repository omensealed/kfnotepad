impl KfnotepadGui {
    fn new_with_task(launch: GuiLaunch) -> (Self, Task<Message>) {
        let mut state = Self::new(launch);
        let task = state.expand_browser_tree_root();
        (state, task)
    }

    #[cfg(not(test))]
    fn new(launch: GuiLaunch) -> Self {
        let current_dir = env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        Self::new_with_current_dir(launch, current_dir)
    }

    #[cfg(test)]
    fn new(launch: GuiLaunch) -> Self {
        let current_dir = env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        Self::new_with_paths(launch, current_dir, None, None, None, None)
    }

    #[cfg(not(test))]
    fn new_with_current_dir(launch: GuiLaunch, current_dir: PathBuf) -> Self {
        Self::new_with_paths(
            launch,
            current_dir,
            current_editor_config_path(),
            current_gui_layout_path(),
            current_gui_workspace_project_launch_path(),
            current_gui_workspace_projects_dir(),
        )
    }

    #[cfg(test)]
    fn new_with_current_dir(launch: GuiLaunch, current_dir: PathBuf) -> Self {
        Self::new_with_paths(launch, current_dir, None, None, None, None)
    }
}
