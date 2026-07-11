struct GuiLaunchDocuments {
    documents: Vec<TextDocument>,
    project_layout: Option<GuiLayout>,
    project_active_ordinal: Option<usize>,
    show_startup_help_panel: bool,
}

struct GuiPaneBuild {
    panes: pane_grid::State<GuiPane>,
    minimized_panes: Vec<GuiPane>,
    active_pane: pane_grid::Pane,
}

struct GuiBrowserBuild {
    browser: Option<GuiFileBrowser>,
    browser_tree: Option<DirectoryTree>,
    browser_expanded_paths: HashSet<PathBuf>,
}
