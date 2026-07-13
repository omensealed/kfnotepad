//! Shared GUI application, pane, and external-file state types.

use super::*;

pub(super) struct KfnotepadGui {
    pub(super) workspace: GuiWorkspace,
    pub(super) panes: pane_grid::State<GuiPane>,
    pub(super) active_pane: pane_grid::Pane,
    pub(super) minimized_panes: Vec<GuiPane>,
    pub(super) browser: Option<GuiFileBrowser>,
    pub(super) browser_tree_rows: Vec<GuiFileTreeRowModel>,
    pub(super) browser_tree_generation: u64,
    pub(super) browser_tree_loading: bool,
    pub(super) browser_expanded_paths: HashSet<PathBuf>,
    pub(super) browser_selected_path: Option<PathBuf>,
    pub(super) browser_visible: bool,
    pub(super) browser_width: f32,
    pub(super) left_panel: GuiLeftPanelState,
    pub(super) current_dir: PathBuf,
    pub(super) notes_dir: Option<PathBuf>,
    pub(super) workspace_projects_dir: Option<PathBuf>,
    pub(super) workspace_projects: Vec<GuiWorkspaceProjectEntry>,
    pub(super) workspace_project_name: String,
    pub(super) pending_project_delete: Option<usize>,
    pub(super) pending_browser_delete: Option<PathBuf>,
    #[cfg(test)]
    pub(super) spawned_workspace_project_paths: Vec<PathBuf>,
    pub(super) path_prompt: Option<GuiPathPrompt>,
    pub(super) path_prompt_value: String,
    pub(super) notes_panel: Option<Vec<ManagedNoteEntry>>,
    pub(super) pending_managed_note_delete: Option<PathBuf>,
    pub(super) file_snapshots: HashMap<GuiTileId, GuiFileSnapshot>,
    pub(super) external_file_check_in_flight: bool,
    pub(super) external_file_check_tick: u32,
    pub(super) external_file_watcher: Option<GuiExternalFileWatcher>,
    pub(super) external_file_watcher_error: Option<String>,
    pub(super) external_edit_locks: HashSet<GuiTileId>,
    pub(super) syntax_caches: HashMap<GuiTileId, GuiSyntaxCache>,
    pub(super) replacement_pointer_point: Option<(pane_grid::Pane, GuiEditorReplacementMousePoint)>,
    pub(super) replacement_drag: Option<GuiEditorDragState>,
    pub(super) replacement_drag_edge: Option<GuiEditorDragEdge>,
    pub(super) replacement_scrollbar_drag: Option<GuiEditorScrollbarDrag>,
    pub(super) replacement_scrollbar_pointer:
        Option<(pane_grid::Pane, f32, GuiEditorScrollbarModel)>,
    pub(super) replacement_ime_preedit: Option<GuiImePreedit>,
    pub(super) replacement_overwrite_mode: bool,
    pub(super) pending_project_open: Option<usize>,
    pub(super) pending_close_tile: Option<GuiTileId>,
    pub(super) save_in_flight: HashSet<GuiTileId>,
    pub(super) save_requested_after_in_flight: HashSet<GuiTileId>,
    pub(super) pending_app_quit: bool,
    pub(super) search_query: String,
    pub(super) search_history: Vec<String>,
    pub(super) search_history_open: bool,
    pub(super) search_highlight: Option<GuiSearchHighlight>,
    pub(super) reader_scroll_accumulator: f32,
    pub(super) go_to_line_query: String,
    pub(super) syntax_highlighter: SyntaxHighlighter,
    pub(super) settings: EditorSettings,
    pub(super) config_path: Option<PathBuf>,
    pub(super) layout_path: Option<PathBuf>,
    pub(super) status_message: String,
    pub(super) show_startup_help_panel: bool,
}

pub(super) struct GuiPane {
    pub(super) tile_id: GuiTileId,
    pub(super) editor: GuiEditorAdapter,
}

impl GuiPane {
    pub(super) fn new(tile_id: GuiTileId, editor: text_editor::Content) -> Self {
        Self {
            tile_id,
            editor: GuiEditorAdapter::new(editor),
        }
    }
}

pub(super) struct GuiMinimizedTrayItem {
    pub(super) tile_id: GuiTileId,
    pub(super) title: String,
    pub(super) tooltip: String,
}

pub(super) type GuiFileSnapshot = FileSnapshot;

#[derive(Debug, Clone)]
pub(super) struct GuiExternalFileCheckCandidate {
    pub(super) tile_id: GuiTileId,
    pub(super) path: PathBuf,
    pub(super) dirty: bool,
    pub(super) previous_snapshot: Option<GuiFileSnapshot>,
    pub(super) force_deep_check: bool,
}

#[derive(Debug, Clone)]
pub(super) enum GuiExternalFileCheckResult {
    SnapshotInitialized {
        tile_id: GuiTileId,
        snapshot: GuiFileSnapshot,
    },
    DirtyChanged {
        tile_id: GuiTileId,
        path: PathBuf,
        snapshot: GuiFileSnapshot,
    },
    Reloaded {
        tile_id: GuiTileId,
        path: PathBuf,
        snapshot: GuiFileSnapshot,
        document: Box<TextDocument>,
    },
    LoadFailed {
        tile_id: GuiTileId,
        path: PathBuf,
        message: String,
    },
}
