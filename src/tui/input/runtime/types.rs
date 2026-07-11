//! Runtime state and active prompt variants.

use super::*;

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct EditorRuntime {
    pub(crate) status: String,
    pub(crate) quit_confirmation_pending: bool,
    pub(crate) close_tab_confirmation_pending: bool,
    pub(crate) search_active: bool,
    pub(crate) search_query: String,
    pub(crate) last_search_query: String,
    pub(crate) search_history: Vec<String>,
    pub(crate) search_history_index: Option<usize>,
    pub(crate) goto_line_active: bool,
    pub(crate) goto_line_query: String,
    pub(crate) menu: Option<MenuState>,
    pub(crate) page_rows: usize,
    pub(crate) settings: EditorSettings,
    pub(crate) config_path: Option<PathBuf>,
    pub(crate) workspace_projects_dir: Option<PathBuf>,
    pub(crate) workspace_prompt: Option<WorkspacePrompt>,
    pub(crate) workspace_query: String,
    pub(crate) workspace_pending_open: Option<GuiWorkspaceProject>,
    pub(crate) workspace_pending_delete: Option<(String, PathBuf)>,
    pub(crate) workspace_prompt_candidates: Vec<String>,
    pub(crate) workspace_prompt_candidate_index: Option<usize>,
    pub(crate) workspace_open_confirmation_pending: bool,
    pub(crate) workspace_manager: Option<WorkspaceManagerState>,
    pub(crate) sidebar: Option<FileSidebarState>,
    pub(crate) last_sidebar_dir: Option<PathBuf>,
    pub(crate) sidebar_prompt: Option<SidebarPrompt>,
    pub(crate) sidebar_query: String,
    pub(crate) overwrite_mode: bool,
    pub(crate) reader_scroll_milli_lines: u32,
    pub(crate) command_palette: Option<CommandPaletteState>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum WorkspacePrompt {
    SaveNamed,
    OpenNamed,
    DeleteNamed,
    ConfirmOpen,
    ConfirmDelete,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum SidebarPrompt {
    CreateFile,
    CreateDirectory,
    DeleteConfirm {
        entry: FileSidebarEntry,
        recursive: bool,
    },
}
