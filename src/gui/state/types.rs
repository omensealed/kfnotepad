struct KfnotepadGui {
    workspace: GuiWorkspace,
    panes: pane_grid::State<GuiPane>,
    active_pane: pane_grid::Pane,
    minimized_panes: Vec<GuiPane>,
    browser: Option<GuiFileBrowser>,
    browser_tree: Option<DirectoryTree>,
    browser_expanded_paths: HashSet<PathBuf>,
    browser_selected_path: Option<PathBuf>,
    browser_visible: bool,
    browser_width: f32,
    left_panel: GuiLeftPanelState,
    current_dir: PathBuf,
    notes_dir: Option<PathBuf>,
    workspace_projects_dir: Option<PathBuf>,
    workspace_projects: Vec<GuiWorkspaceProjectEntry>,
    workspace_project_name: String,
    pending_project_delete: Option<usize>,
    pending_browser_delete: Option<PathBuf>,
    #[cfg(test)]
    spawned_workspace_project_paths: Vec<PathBuf>,
    path_prompt: Option<GuiPathPrompt>,
    path_prompt_value: String,
    notes_panel: Option<Vec<ManagedNoteEntry>>,
    pending_managed_note_delete: Option<PathBuf>,
    file_snapshots: HashMap<GuiTileId, GuiFileSnapshot>,
    external_edit_locks: HashSet<GuiTileId>,
    syntax_caches: HashMap<GuiTileId, GuiSyntaxCache>,
    replacement_pointer_point: Option<(pane_grid::Pane, GuiEditorReplacementMousePoint)>,
    replacement_drag: Option<GuiEditorDragState>,
    replacement_drag_edge: Option<GuiEditorDragEdge>,
    replacement_scrollbar_drag: Option<GuiEditorScrollbarDrag>,
    replacement_scrollbar_pointer: Option<(pane_grid::Pane, f32, GuiEditorScrollbarModel)>,
    replacement_ime_preedit: Option<GuiImePreedit>,
    replacement_overwrite_mode: bool,
    pending_project_open: Option<usize>,
    pending_close_tile: Option<GuiTileId>,
    pending_app_quit: bool,
    search_query: String,
    search_history: Vec<String>,
    search_history_open: bool,
    search_highlight: Option<GuiSearchHighlight>,
    reader_scroll_accumulator: f32,
    go_to_line_query: String,
    syntax_highlighter: SyntaxHighlighter,
    settings: EditorSettings,
    config_path: Option<PathBuf>,
    layout_path: Option<PathBuf>,
    status_message: String,
    show_startup_help_panel: bool,
}

struct GuiPane {
    tile_id: GuiTileId,
    editor: GuiEditorAdapter,
}

impl GuiPane {
    fn new(tile_id: GuiTileId, editor: text_editor::Content) -> Self {
        Self {
            tile_id,
            editor: GuiEditorAdapter::new(editor),
        }
    }
}

struct GuiMinimizedTrayItem {
    tile_id: GuiTileId,
    title: String,
    tooltip: String,
}

type GuiFileSnapshot = FileSnapshot;

#[derive(Debug, Clone)]
struct GuiExternalFileCheckCandidate {
    tile_id: GuiTileId,
    path: PathBuf,
    dirty: bool,
    previous_snapshot: Option<GuiFileSnapshot>,
}

#[derive(Debug, Clone)]
enum GuiExternalFileCheckResult {
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
        document: TextDocument,
    },
    LoadFailed {
        tile_id: GuiTileId,
        path: PathBuf,
        message: String,
    },
}
