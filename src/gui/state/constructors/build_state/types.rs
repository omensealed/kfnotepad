//! Intermediate values used while assembling complete GUI state.

use super::super::*;

pub(super) struct GuiLaunchDocuments {
    pub(super) documents: Vec<TextDocument>,
    pub(super) project_layout: Option<GuiLayout>,
    pub(super) project_active_ordinal: Option<usize>,
    pub(super) show_startup_help_panel: bool,
}

pub(super) struct GuiPaneBuild {
    pub(super) panes: pane_grid::State<GuiPane>,
    pub(super) minimized_panes: Vec<GuiPane>,
    pub(super) active_pane: pane_grid::Pane,
}

pub(super) struct GuiBrowserBuild {
    pub(super) browser: Option<GuiFileBrowser>,
    pub(super) browser_tree_rows: Vec<GuiFileTreeRowModel>,
    pub(super) browser_expanded_paths: HashSet<PathBuf>,
}
