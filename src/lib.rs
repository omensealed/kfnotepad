use std::collections::{HashMap, HashSet};
use std::fmt;
use std::fs::{self, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use syntect::easy::HighlightLines;
use syntect::highlighting::{HighlightState, Style as SyntectStyle, Theme, ThemeSet};
use syntect::parsing::{ParseState, SyntaxReference, SyntaxSet};

pub const VERSION: &str = env!("CARGO_PKG_VERSION");
pub const MAX_TEXT_FILE_BYTES: u64 = 8 * 1024 * 1024;
pub const MAX_UNDO_HISTORY: usize = 256;

#[derive(Debug, PartialEq, Eq)]
pub enum Command {
    Help,
    Version,
    LaunchEmpty,
    InspectFile(String),
    ListManagedNotes,
    OpenManagedNote(String),
}

#[derive(Debug, PartialEq, Eq)]
pub struct FileSummary {
    pub bytes: u64,
    pub lines: usize,
    pub trailing_newline: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TextBuffer {
    lines: Vec<String>,
    trailing_newline: bool,
    dirty: bool,
    undo_history: Vec<BufferSnapshot>,
    redo_history: Vec<BufferSnapshot>,
    file_snapshot: Option<FileSnapshot>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct BufferSnapshot {
    lines: Vec<String>,
    trailing_newline: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileSnapshot {
    pub bytes: u64,
    pub modified: Option<SystemTime>,
    pub fingerprint: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Cursor {
    pub row: usize,
    pub column: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CursorMove {
    Left,
    Right,
    WordLeft,
    WordRight,
    Up,
    Down,
}

#[derive(Debug, PartialEq, Eq)]
pub enum BufferError {
    RowOutOfBounds { row: usize, rows: usize },
    ColumnOutOfBounds { column: usize, columns: usize },
    UseInsertNewline,
}

#[derive(Debug, PartialEq, Eq)]
pub struct TextDocument {
    pub path: PathBuf,
    pub buffer: TextBuffer,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct EditorTabState {
    pub cursor: Cursor,
    pub viewport_start: usize,
    pub horizontal_offset: usize,
}

impl Default for EditorTabState {
    fn default() -> Self {
        Self {
            cursor: Cursor { row: 0, column: 0 },
            viewport_start: 0,
            horizontal_offset: 0,
        }
    }
}

pub enum EditorTabDocument<'a> {
    Borrowed(&'a mut TextDocument),
    Owned(TextDocument),
}

impl AsRef<TextDocument> for EditorTabDocument<'_> {
    fn as_ref(&self) -> &TextDocument {
        match self {
            Self::Borrowed(document) => document,
            Self::Owned(document) => document,
        }
    }
}

impl AsMut<TextDocument> for EditorTabDocument<'_> {
    fn as_mut(&mut self) -> &mut TextDocument {
        match self {
            Self::Borrowed(document) => document,
            Self::Owned(document) => document,
        }
    }
}

pub struct EditorTab<'a> {
    pub document: EditorTabDocument<'a>,
    pub state: EditorTabState,
}

pub struct EditorWorkspace<'a> {
    pub tabs: Vec<EditorTab<'a>>,
    pub active: usize,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TabStripItem {
    pub label: String,
    pub active: bool,
    pub dirty: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CloseActiveTabResult {
    OnlyTab,
    Dirty,
    Closed { path: PathBuf },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum UndoRedoResult {
    Applied,
    NothingToApply,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum EditResult {
    Modified,
    Unchanged,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SearchRepeatResult {
    NoPreviousSearch,
    Found { query: String },
    NoMatch { query: String },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SearchMode {
    pub case_sensitive: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum GoToLineResult {
    Empty,
    Invalid,
    OutOfRange { line_number: usize },
    Moved { line_number: usize },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct GuiTileId(pub usize);

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GuiTileSaveStatus {
    Saved,
    Modified,
    SaveFailed { message: String },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GuiSplitDirection {
    Horizontal,
    Vertical,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GuiTileMoveDirection {
    Left,
    Right,
    Up,
    Down,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GuiTileResizeDirection {
    Wider,
    Narrower,
    Taller,
    Shorter,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GuiTileLayoutIntent {
    Split {
        tile_id: GuiTileId,
        direction: GuiSplitDirection,
    },
    Move {
        tile_id: GuiTileId,
        direction: GuiTileMoveDirection,
    },
    Resize {
        tile_id: GuiTileId,
        direction: GuiTileResizeDirection,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GuiLayoutAxis {
    Horizontal,
    Vertical,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum GuiLayoutNode {
    Leaf {
        ordinal: usize,
    },
    Split {
        axis: GuiLayoutAxis,
        ratio_per_mille: u16,
        first: Box<GuiLayoutNode>,
        second: Box<GuiLayoutNode>,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GuiLayout {
    pub browser_visible: bool,
    pub browser_width_px: Option<u16>,
    pub root: GuiLayoutNode,
    pub minimized_ordinals: Vec<usize>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GuiWorkspaceProject {
    pub name: String,
    pub files: Vec<PathBuf>,
    pub active_ordinal: usize,
    pub layout: Option<GuiLayout>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GuiWorkspaceProjectEntry {
    pub path: PathBuf,
    pub project: GuiWorkspaceProject,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GuiWorkspaceProjectDeleteResult {
    Deleted,
    Missing,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GuiLeftPanelMode {
    Files,
    Workspaces,
    Preferences,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GuiLeftPanelState {
    pub visible: bool,
    pub mode: GuiLeftPanelMode,
}

impl Default for GuiLeftPanelState {
    fn default() -> Self {
        Self {
            visible: true,
            mode: GuiLeftPanelMode::Files,
        }
    }
}

impl GuiLeftPanelState {
    pub fn toggle_visibility(&mut self) {
        self.visible = !self.visible;
    }

    pub fn show_files(&mut self) {
        self.visible = true;
        self.mode = GuiLeftPanelMode::Files;
    }

    pub fn show_workspaces(&mut self) {
        self.visible = true;
        self.mode = GuiLeftPanelMode::Workspaces;
    }

    pub fn show_preferences(&mut self) {
        self.visible = true;
        self.mode = GuiLeftPanelMode::Preferences;
    }

    pub fn toggle_mode(&mut self) {
        self.visible = true;
        self.mode = match self.mode {
            GuiLeftPanelMode::Files => GuiLeftPanelMode::Workspaces,
            GuiLeftPanelMode::Workspaces => GuiLeftPanelMode::Preferences,
            GuiLeftPanelMode::Preferences => GuiLeftPanelMode::Files,
        };
    }

    pub fn title(&self) -> &'static str {
        match self.mode {
            GuiLeftPanelMode::Files => "Files",
            GuiLeftPanelMode::Workspaces => "Workspaces",
            GuiLeftPanelMode::Preferences => "Preferences",
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum GuiCloseTileResult {
    Missing,
    OnlyTile,
    Dirty { tile_id: GuiTileId },
    Closed { tile_id: GuiTileId, path: PathBuf },
}

#[derive(Debug)]
pub enum GuiTileOpenError {
    Invalid { source: OpenError },
}

pub struct GuiDocumentTile {
    pub id: GuiTileId,
    pub document: TextDocument,
    pub state: EditorTabState,
    pub minimized: bool,
    last_save_error: Option<String>,
}

impl GuiDocumentTile {
    pub fn save_status(&self) -> GuiTileSaveStatus {
        if let Some(message) = &self.last_save_error {
            return GuiTileSaveStatus::SaveFailed {
                message: message.clone(),
            };
        }
        if self.document.buffer.is_dirty() {
            GuiTileSaveStatus::Modified
        } else {
            GuiTileSaveStatus::Saved
        }
    }
}

pub struct GuiWorkspace {
    pub tiles: Vec<GuiDocumentTile>,
    pub active: GuiTileId,
    pub focused: GuiTileId,
    pub pending_layout_intent: Option<GuiTileLayoutIntent>,
    next_tile_id: usize,
}

impl GuiWorkspace {
    pub fn from_document(document: TextDocument) -> Self {
        let first_id = GuiTileId(0);
        Self {
            tiles: vec![GuiDocumentTile {
                id: first_id,
                document,
                state: EditorTabState::default(),
                minimized: false,
                last_save_error: None,
            }],
            active: first_id,
            focused: first_id,
            pending_layout_intent: None,
            next_tile_id: 1,
        }
    }

    pub fn active_tile(&self) -> &GuiDocumentTile {
        self.tile(self.active)
            .expect("active GUI tile id must refer to an existing tile")
    }

    pub fn active_tile_mut(&mut self) -> &mut GuiDocumentTile {
        self.tile_mut(self.active)
            .expect("active GUI tile id must refer to an existing tile")
    }

    pub fn focused_tile(&self) -> &GuiDocumentTile {
        self.tile(self.focused)
            .expect("focused GUI tile id must refer to an existing tile")
    }

    pub fn tile(&self, tile_id: GuiTileId) -> Option<&GuiDocumentTile> {
        self.tiles.iter().find(|tile| tile.id == tile_id)
    }

    pub fn tile_mut(&mut self, tile_id: GuiTileId) -> Option<&mut GuiDocumentTile> {
        self.tiles.iter_mut().find(|tile| tile.id == tile_id)
    }

    pub fn open_tile(&mut self, document: TextDocument) -> GuiTileId {
        let tile_id = GuiTileId(self.next_tile_id);
        self.next_tile_id += 1;
        self.tiles.push(GuiDocumentTile {
            id: tile_id,
            document,
            state: EditorTabState::default(),
            minimized: false,
            last_save_error: None,
        });
        self.focus_tile(tile_id);
        tile_id
    }

    pub fn open_validated_tile(
        &mut self,
        document: Result<TextDocument, OpenError>,
    ) -> Result<GuiTileId, GuiTileOpenError> {
        match document {
            Ok(document) => Ok(self.open_tile(document)),
            Err(source) => Err(GuiTileOpenError::Invalid { source }),
        }
    }

    pub fn focus_tile(&mut self, tile_id: GuiTileId) -> bool {
        if self.tile(tile_id).is_none() {
            return false;
        }
        self.active = tile_id;
        self.focused = tile_id;
        true
    }

    pub fn set_tile_minimized(&mut self, tile_id: GuiTileId, minimized: bool) -> bool {
        let Some(tile) = self.tile_mut(tile_id) else {
            return false;
        };
        tile.minimized = minimized;
        if !minimized {
            self.focus_tile(tile_id);
        }
        true
    }

    pub fn close_tile(&mut self, tile_id: GuiTileId, confirm_dirty: bool) -> GuiCloseTileResult {
        let Some(index) = self.tiles.iter().position(|tile| tile.id == tile_id) else {
            return GuiCloseTileResult::Missing;
        };
        if self.tiles.len() <= 1 {
            return GuiCloseTileResult::OnlyTile;
        }
        if self.tiles[index].document.buffer.is_dirty() && !confirm_dirty {
            return GuiCloseTileResult::Dirty { tile_id };
        }

        let removed = self.tiles.remove(index);
        if self.active == tile_id || self.focused == tile_id {
            let fallback_index = index.min(self.tiles.len().saturating_sub(1));
            let fallback_id = self.tiles[fallback_index].id;
            self.active = fallback_id;
            self.focused = fallback_id;
        }
        GuiCloseTileResult::Closed {
            tile_id,
            path: removed.document.path,
        }
    }

    pub fn request_split(&mut self, tile_id: GuiTileId, direction: GuiSplitDirection) -> bool {
        if self.tile(tile_id).is_none() {
            return false;
        }
        self.pending_layout_intent = Some(GuiTileLayoutIntent::Split { tile_id, direction });
        true
    }

    pub fn request_move(&mut self, tile_id: GuiTileId, direction: GuiTileMoveDirection) -> bool {
        if self.tile(tile_id).is_none() {
            return false;
        }
        self.pending_layout_intent = Some(GuiTileLayoutIntent::Move { tile_id, direction });
        true
    }

    pub fn request_resize(
        &mut self,
        tile_id: GuiTileId,
        direction: GuiTileResizeDirection,
    ) -> bool {
        if self.tile(tile_id).is_none() {
            return false;
        }
        self.pending_layout_intent = Some(GuiTileLayoutIntent::Resize { tile_id, direction });
        true
    }

    pub fn clear_layout_intent(&mut self) {
        self.pending_layout_intent = None;
    }

    pub fn mark_tile_save_failed(
        &mut self,
        tile_id: GuiTileId,
        message: impl Into<String>,
    ) -> bool {
        let Some(tile) = self.tile_mut(tile_id) else {
            return false;
        };
        tile.last_save_error = Some(message.into());
        true
    }

    pub fn clear_tile_save_error(&mut self, tile_id: GuiTileId) -> bool {
        let Some(tile) = self.tile_mut(tile_id) else {
            return false;
        };
        tile.last_save_error = None;
        true
    }
}

pub struct GuiFileBrowser {
    pub sidebar: FileSidebarState,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum GuiFileBrowserActivation {
    Navigated { current_dir: PathBuf },
    OpenTile { path: PathBuf },
}

#[derive(Debug)]
pub enum GuiFileBrowserError {
    EmptySelection,
    Navigate { source: FileSidebarError },
}

impl fmt::Display for GuiFileBrowserError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptySelection => write!(formatter, "no file-browser entry selected"),
            Self::Navigate { source } => write!(formatter, "{source}"),
        }
    }
}

impl GuiFileBrowser {
    pub fn load(current_dir: PathBuf) -> Result<Self, FileSidebarError> {
        Ok(Self {
            sidebar: FileSidebarState::load(current_dir)?,
        })
    }

    pub fn refresh(&mut self) -> Result<(), FileSidebarError> {
        let selected_path = self
            .sidebar
            .selected_entry()
            .map(|entry| entry.path.clone());
        let previous_selected = self.sidebar.selected;
        let previous_scroll = self.sidebar.scroll;
        let mut refreshed = FileSidebarState::load(self.sidebar.current_dir.clone())?;
        refreshed.selected = selected_path
            .and_then(|path| {
                refreshed
                    .entries
                    .iter()
                    .position(|entry| entry.path == path)
            })
            .unwrap_or_else(|| previous_selected.min(refreshed.entries.len().saturating_sub(1)));
        refreshed.scroll = previous_scroll.min(refreshed.selected);
        refreshed.keep_selection_visible(1);
        self.sidebar = refreshed;
        Ok(())
    }

    pub fn selected_entry(&self) -> Option<&FileSidebarEntry> {
        self.sidebar.selected_entry()
    }

    pub fn select_previous_wrapping(&mut self, visible_rows: usize) {
        self.sidebar.select_previous_wrapping(visible_rows);
    }

    pub fn select_next_wrapping(&mut self, visible_rows: usize) {
        self.sidebar.select_next_wrapping(visible_rows);
    }

    pub fn activate_selected(&mut self) -> Result<GuiFileBrowserActivation, GuiFileBrowserError> {
        let Some(entry) = self.sidebar.selected_entry().cloned() else {
            return Err(GuiFileBrowserError::EmptySelection);
        };
        self.activate_entry(entry)
    }

    pub fn activate_mouse_row(
        &mut self,
        row: u16,
    ) -> Result<Option<GuiFileBrowserActivation>, GuiFileBrowserError> {
        let Some(entry) = self.sidebar.selected_entry_for_mouse_row(row) else {
            return Ok(None);
        };
        self.activate_entry(entry).map(Some)
    }

    fn activate_entry(
        &mut self,
        entry: FileSidebarEntry,
    ) -> Result<GuiFileBrowserActivation, GuiFileBrowserError> {
        match entry.kind {
            FileSidebarEntryKind::Parent | FileSidebarEntryKind::Directory => {
                self.sidebar = FileSidebarState::load(entry.path)
                    .map_err(|source| GuiFileBrowserError::Navigate { source })?;
                Ok(GuiFileBrowserActivation::Navigated {
                    current_dir: self.sidebar.current_dir.clone(),
                })
            }
            FileSidebarEntryKind::File => {
                Ok(GuiFileBrowserActivation::OpenTile { path: entry.path })
            }
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct EditorSettings {
    pub show_line_numbers: bool,
    pub theme_id: EditorThemeId,
    pub syntax_theme_id: EditorThemeId,
    pub wrap_lines: bool,
    pub search_case_sensitive: bool,
    pub gui_restore_last_workspace: bool,
    pub gui_reader_mode_enabled: bool,
    pub gui_reader_lines_per_minute: u16,
    pub gui_font_family: GuiFontFamily,
    pub gui_font_size: u16,
    pub gui_ui_font_size: u16,
}

impl Default for EditorSettings {
    fn default() -> Self {
        Self {
            show_line_numbers: true,
            theme_id: EditorThemeId::Nocturne,
            syntax_theme_id: EditorThemeId::Nocturne,
            wrap_lines: false,
            search_case_sensitive: false,
            gui_restore_last_workspace: false,
            gui_reader_mode_enabled: false,
            gui_reader_lines_per_minute: DEFAULT_GUI_READER_LINES_PER_MINUTE,
            gui_font_family: GuiFontFamily::Monospace,
            gui_font_size: DEFAULT_GUI_FONT_SIZE,
            gui_ui_font_size: DEFAULT_GUI_UI_FONT_SIZE,
        }
    }
}

pub const MIN_GUI_FONT_SIZE: u16 = 10;
pub const DEFAULT_GUI_FONT_SIZE: u16 = 16;
pub const DEFAULT_GUI_UI_FONT_SIZE: u16 = 14;
pub const MAX_GUI_FONT_SIZE: u16 = 32;
pub const MIN_GUI_READER_LINES_PER_MINUTE: u16 = 20;
pub const DEFAULT_GUI_READER_LINES_PER_MINUTE: u16 = 60;
pub const MAX_GUI_READER_LINES_PER_MINUTE: u16 = 240;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum GuiFontFamily {
    #[default]
    Monospace,
    SansSerif,
    Serif,
    JetBrainsMono,
    FiraCode,
}

impl GuiFontFamily {
    pub const ALL: [Self; 5] = [
        Self::Monospace,
        Self::SansSerif,
        Self::Serif,
        Self::JetBrainsMono,
        Self::FiraCode,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Self::Monospace => "monospace",
            Self::SansSerif => "sans-serif",
            Self::Serif => "serif",
            Self::JetBrainsMono => "jetbrains-mono",
            Self::FiraCode => "fira-code",
        }
    }

    pub fn display_label(self) -> &'static str {
        match self {
            Self::Monospace => "Monospace",
            Self::SansSerif => "Sans serif",
            Self::Serif => "Serif",
            Self::JetBrainsMono => "JetBrains Mono",
            Self::FiraCode => "Fira Code",
        }
    }

    pub fn from_label(label: &str) -> Option<Self> {
        match label {
            "monospace" => Some(Self::Monospace),
            "sans-serif" => Some(Self::SansSerif),
            "serif" => Some(Self::Serif),
            "jetbrains-mono" => Some(Self::JetBrainsMono),
            "fira-code" => Some(Self::FiraCode),
            _ => None,
        }
    }

    pub fn next(self) -> Self {
        match self {
            Self::Monospace => Self::SansSerif,
            Self::SansSerif => Self::Serif,
            Self::Serif => Self::JetBrainsMono,
            Self::JetBrainsMono => Self::FiraCode,
            Self::FiraCode => Self::Monospace,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum EditorThemeId {
    #[default]
    Nocturne,
    Aurora,
    Paper,
    Terminal,
    Abyss,
    Terror,
}

impl EditorThemeId {
    pub fn next(self) -> Self {
        match self {
            Self::Nocturne => Self::Aurora,
            Self::Aurora => Self::Paper,
            Self::Paper => Self::Terminal,
            Self::Terminal => Self::Abyss,
            Self::Abyss => Self::Terror,
            Self::Terror => Self::Nocturne,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Nocturne => "nocturne",
            Self::Aurora => "aurora",
            Self::Paper => "pastel",
            Self::Terminal => "terminal",
            Self::Abyss => "abyss",
            Self::Terror => "terror",
        }
    }

    pub fn from_label(label: &str) -> Option<Self> {
        match label {
            "nocturne" => Some(Self::Nocturne),
            "aurora" => Some(Self::Aurora),
            "paper" | "pastel" => Some(Self::Paper),
            "terminal" => Some(Self::Terminal),
            "abyss" => Some(Self::Abyss),
            "terror" => Some(Self::Terror),
            _ => None,
        }
    }
}

#[derive(Debug)]
pub enum EditorConfigError {
    Read {
        path: PathBuf,
        source: io::Error,
    },
    Invalid {
        path: PathBuf,
        message: String,
    },
    CreateDir {
        path: PathBuf,
        source: io::Error,
    },
    CreateTemp {
        path: PathBuf,
        source: io::Error,
    },
    WriteTemp {
        path: PathBuf,
        source: io::Error,
    },
    Remove {
        path: PathBuf,
        source: io::Error,
    },
    FlushTemp {
        path: PathBuf,
        source: io::Error,
    },
    Rename {
        from: PathBuf,
        to: PathBuf,
        source: io::Error,
    },
}

impl fmt::Display for EditorConfigError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Read { path, source } => {
                write!(formatter, "cannot read {}: {source}", path.display())
            }
            Self::Invalid { path, message } => {
                write!(formatter, "invalid config {}: {message}", path.display())
            }
            Self::CreateDir { path, source } => {
                write!(
                    formatter,
                    "cannot create config directory {}: {source}",
                    path.display()
                )
            }
            Self::CreateTemp { path, source } => {
                write!(
                    formatter,
                    "cannot create temporary config {}: {source}",
                    path.display()
                )
            }
            Self::WriteTemp { path, source } => {
                write!(
                    formatter,
                    "cannot write temporary config {}: {source}",
                    path.display()
                )
            }
            Self::Remove { path, source } => {
                write!(
                    formatter,
                    "cannot remove config {}: {source}",
                    path.display()
                )
            }
            Self::FlushTemp { path, source } => {
                write!(
                    formatter,
                    "cannot flush temporary config {}: {source}",
                    path.display()
                )
            }
            Self::Rename { from, to, source } => {
                write!(
                    formatter,
                    "cannot replace config {} with {}: {source}",
                    to.display(),
                    from.display()
                )
            }
        }
    }
}

pub fn editor_config_path(xdg_config_home: Option<&Path>, home: Option<&Path>) -> Option<PathBuf> {
    if let Some(xdg_config_home) = xdg_config_home.filter(|path| !path.as_os_str().is_empty()) {
        return Some(xdg_config_home.join("kfnotepad").join("config.toml"));
    }

    home.filter(|path| !path.as_os_str().is_empty())
        .map(|home| home.join(".config").join("kfnotepad").join("config.toml"))
}

pub fn gui_layout_path(xdg_config_home: Option<&Path>, home: Option<&Path>) -> Option<PathBuf> {
    if let Some(xdg_config_home) = xdg_config_home.filter(|path| !path.as_os_str().is_empty()) {
        return Some(xdg_config_home.join("kfnotepad").join("gui-layout.v1"));
    }

    home.filter(|path| !path.as_os_str().is_empty())
        .map(|home| home.join(".config").join("kfnotepad").join("gui-layout.v1"))
}

pub fn gui_workspace_projects_dir(
    xdg_config_home: Option<&Path>,
    home: Option<&Path>,
) -> Option<PathBuf> {
    if let Some(xdg_config_home) = xdg_config_home.filter(|path| !path.as_os_str().is_empty()) {
        return Some(xdg_config_home.join("kfnotepad").join("workspaces"));
    }

    home.filter(|path| !path.as_os_str().is_empty())
        .map(|home| home.join(".config").join("kfnotepad").join("workspaces"))
}

pub fn load_editor_settings(path: &Path) -> Result<EditorSettings, EditorConfigError> {
    let text = match fs::read_to_string(path) {
        Ok(text) => text,
        Err(error) if error.kind() == io::ErrorKind::NotFound => {
            return Ok(EditorSettings::default());
        }
        Err(source) => {
            return Err(EditorConfigError::Read {
                path: path.to_path_buf(),
                source,
            });
        }
    };

    Ok(parse_editor_settings_config(&text))
}

pub fn parse_editor_settings_config(text: &str) -> EditorSettings {
    let mut settings = EditorSettings::default();

    for line in text.lines() {
        let Some((raw_key, raw_value)) = line.split_once('=') else {
            continue;
        };
        let key = raw_key.trim();
        let value = raw_value
            .split_once('#')
            .map_or(raw_value, |(value, _)| value)
            .trim();

        match key {
            "theme" => {
                if let Some(theme_id) =
                    parse_config_string(value).and_then(EditorThemeId::from_label)
                {
                    settings.theme_id = theme_id;
                }
            }
            "syntax_theme" => {
                if let Some(theme_id) =
                    parse_config_string(value).and_then(EditorThemeId::from_label)
                {
                    settings.syntax_theme_id = theme_id;
                }
            }
            "line_numbers" => {
                if let Some(show_line_numbers) = parse_config_bool(value) {
                    settings.show_line_numbers = show_line_numbers;
                }
            }
            "wrap" => {
                if let Some(wrap_lines) = parse_config_bool(value) {
                    settings.wrap_lines = wrap_lines;
                }
            }
            "search_case_sensitive" => {
                if let Some(search_case_sensitive) = parse_config_bool(value) {
                    settings.search_case_sensitive = search_case_sensitive;
                }
            }
            "gui_restore_last_workspace" => {
                if let Some(gui_restore_last_workspace) = parse_config_bool(value) {
                    settings.gui_restore_last_workspace = gui_restore_last_workspace;
                }
            }
            "gui_reader_mode_enabled" => {
                if let Some(gui_reader_mode_enabled) = parse_config_bool(value) {
                    settings.gui_reader_mode_enabled = gui_reader_mode_enabled;
                }
            }
            "gui_reader_lines_per_minute" => {
                if let Ok(lines_per_minute) = value.parse::<u16>() {
                    if (MIN_GUI_READER_LINES_PER_MINUTE..=MAX_GUI_READER_LINES_PER_MINUTE)
                        .contains(&lines_per_minute)
                    {
                        settings.gui_reader_lines_per_minute = lines_per_minute;
                    }
                }
            }
            "gui_font_family" => {
                if let Some(gui_font_family) =
                    parse_config_string(value).and_then(GuiFontFamily::from_label)
                {
                    settings.gui_font_family = gui_font_family;
                }
            }
            "gui_font_size" => {
                if let Ok(gui_font_size) = value.parse::<u16>() {
                    if (MIN_GUI_FONT_SIZE..=MAX_GUI_FONT_SIZE).contains(&gui_font_size) {
                        settings.gui_font_size = gui_font_size;
                    }
                }
            }
            "gui_ui_font_size" => {
                if let Ok(gui_ui_font_size) = value.parse::<u16>() {
                    if (MIN_GUI_FONT_SIZE..=MAX_GUI_FONT_SIZE).contains(&gui_ui_font_size) {
                        settings.gui_ui_font_size = gui_ui_font_size;
                    }
                }
            }
            _ => {}
        }
    }

    settings
}

pub fn parse_gui_layout(text: &str, pane_count: usize) -> Option<GuiLayout> {
    if pane_count == 0 {
        return None;
    }

    let mut version = None;
    let mut browser_visible = true;
    let mut browser_width_px = None;
    let mut root_id = None;
    let mut minimized_ordinals = Vec::new();
    let mut node_specs = HashMap::new();

    for line in text.lines() {
        let line = line.split_once('#').map_or(line, |(value, _)| value).trim();
        if line.is_empty() {
            continue;
        }
        let (raw_key, raw_value) = line.split_once('=')?;
        let key = raw_key.trim();
        let value = raw_value.trim();

        if key == "version" {
            version = value.parse::<usize>().ok();
        } else if key == "browser_visible" {
            browser_visible = parse_config_bool(value)?;
        } else if key == "browser_width_px" {
            let parsed_width = value.parse::<u16>().ok()?;
            if parsed_width == 0 {
                return None;
            }
            browser_width_px = Some(parsed_width);
        } else if key == "root" {
            root_id = value.parse::<usize>().ok();
        } else if key == "minimized" {
            minimized_ordinals = parse_layout_ordinals(value)?;
        } else if let Some(raw_id) = key.strip_prefix("node.") {
            let id = raw_id.parse::<usize>().ok()?;
            node_specs.insert(id, value.to_string());
        } else {
            continue;
        }
    }

    if version != Some(1) {
        return None;
    }

    let mut seen_nodes = HashSet::new();
    let mut seen_ordinals = HashSet::new();
    let root = parse_gui_layout_node(
        root_id?,
        pane_count,
        &node_specs,
        &mut seen_nodes,
        &mut seen_ordinals,
    )?;
    if seen_ordinals.len() != pane_count {
        return None;
    }

    let mut minimized_seen = HashSet::new();
    for ordinal in &minimized_ordinals {
        if *ordinal >= pane_count || !minimized_seen.insert(*ordinal) {
            return None;
        }
    }

    Some(GuiLayout {
        browser_visible,
        browser_width_px,
        root,
        minimized_ordinals,
    })
}

pub fn serialize_gui_layout(layout: &GuiLayout) -> String {
    let mut lines = vec![
        "version = 1".to_string(),
        format!("browser_visible = {}", layout.browser_visible),
        "root = 0".to_string(),
    ];
    if let Some(browser_width_px) = layout.browser_width_px {
        lines.insert(2, format!("browser_width_px = {browser_width_px}"));
    }
    let mut next_id = 1;
    write_gui_layout_node(&layout.root, 0, &mut next_id, &mut lines);
    let minimized = layout
        .minimized_ordinals
        .iter()
        .map(|ordinal| ordinal.to_string())
        .collect::<Vec<_>>()
        .join(",");
    lines.push(format!("minimized = {minimized}"));
    let mut text = lines.join("\n");
    text.push('\n');
    text
}

pub fn save_gui_layout(path: &Path, layout: &GuiLayout) -> Result<(), EditorConfigError> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|source| EditorConfigError::CreateDir {
            path: parent.to_path_buf(),
            source,
        })?;
        set_private_config_dir_permissions(parent).map_err(|source| {
            EditorConfigError::CreateDir {
                path: parent.to_path_buf(),
                source,
            }
        })?;
    }

    let text = serialize_gui_layout(layout);
    let temp_path = temporary_config_path(path);
    let result = write_config_temp_then_rename(path, &temp_path, text.as_bytes());
    if result.is_err() {
        let _ = fs::remove_file(&temp_path);
    }
    result
}

pub fn parse_gui_workspace_project(text: &str) -> Option<GuiWorkspaceProject> {
    let mut version = None;
    let mut name = None;
    let mut active_ordinal = 0usize;
    let mut files = HashMap::new();
    let mut layout_lines = Vec::new();

    for line in text.lines() {
        let line = line.split_once('#').map_or(line, |(value, _)| value).trim();
        if line.is_empty() {
            continue;
        }
        let (raw_key, raw_value) = line.split_once('=')?;
        let key = raw_key.trim();
        let value = raw_value.trim();

        if key == "version" {
            version = value.parse::<usize>().ok();
        } else if key == "name_hex" {
            name = String::from_utf8(hex_to_bytes(value)?).ok();
        } else if key == "active" {
            active_ordinal = value.parse::<usize>().ok()?;
        } else if let Some(raw_ordinal) = key.strip_prefix("file.") {
            let ordinal = raw_ordinal.parse::<usize>().ok()?;
            if files.insert(ordinal, path_from_hex(value)?).is_some() {
                return None;
            }
        } else if let Some(layout_key) = key.strip_prefix("layout.") {
            layout_lines.push(format!("{layout_key} = {value}"));
        } else {
            continue;
        }
    }

    if version != Some(1) {
        return None;
    }

    let mut ordered_files = Vec::new();
    for ordinal in 0..files.len() {
        ordered_files.push(files.remove(&ordinal)?);
    }
    if !files.is_empty() || ordered_files.is_empty() || active_ordinal >= ordered_files.len() {
        return None;
    }

    let layout = if layout_lines.is_empty() {
        None
    } else {
        Some(parse_gui_layout(
            &layout_lines.join("\n"),
            ordered_files.len(),
        )?)
    };

    Some(GuiWorkspaceProject {
        name: name?,
        files: ordered_files,
        active_ordinal,
        layout,
    })
}

pub fn serialize_gui_workspace_project(project: &GuiWorkspaceProject) -> Option<String> {
    if project.name.is_empty()
        || project.files.is_empty()
        || project.active_ordinal >= project.files.len()
    {
        return None;
    }
    if let Some(layout) = &project.layout {
        parse_gui_layout(&serialize_gui_layout(layout), project.files.len())?;
    }

    let mut lines = vec![
        "version = 1".to_string(),
        format!("name_hex = {}", bytes_to_hex(project.name.as_bytes())),
        format!("active = {}", project.active_ordinal),
    ];
    for (ordinal, path) in project.files.iter().enumerate() {
        lines.push(format!("file.{ordinal} = {}", path_to_hex(path)));
    }
    if let Some(layout) = &project.layout {
        for line in serialize_gui_layout(layout).lines() {
            lines.push(format!("layout.{line}"));
        }
    }
    let mut text = lines.join("\n");
    text.push('\n');
    Some(text)
}

pub fn save_gui_workspace_project(
    path: &Path,
    project: &GuiWorkspaceProject,
) -> Result<(), EditorConfigError> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|source| EditorConfigError::CreateDir {
            path: parent.to_path_buf(),
            source,
        })?;
        set_private_config_dir_permissions(parent).map_err(|source| {
            EditorConfigError::CreateDir {
                path: parent.to_path_buf(),
                source,
            }
        })?;
    }

    let text =
        serialize_gui_workspace_project(project).ok_or_else(|| EditorConfigError::Invalid {
            path: path.to_path_buf(),
            message: "invalid GUI workspace project snapshot".to_string(),
        })?;
    let temp_path = temporary_config_path(path);
    let result = write_config_temp_then_rename(path, &temp_path, text.as_bytes());
    if result.is_err() {
        let _ = fs::remove_file(&temp_path);
    }
    result
}

pub fn gui_workspace_project_path(projects_dir: &Path, name: &str) -> Option<PathBuf> {
    Some(projects_dir.join(format!("{}.v1", gui_workspace_project_slug(name)?)))
}

pub fn list_gui_workspace_projects(
    projects_dir: &Path,
) -> Result<Vec<GuiWorkspaceProjectEntry>, EditorConfigError> {
    let entries = match fs::read_dir(projects_dir) {
        Ok(entries) => entries,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(source) => {
            return Err(EditorConfigError::Read {
                path: projects_dir.to_path_buf(),
                source,
            });
        }
    };

    let mut projects = Vec::new();
    for entry in entries {
        let Ok(entry) = entry else {
            continue;
        };
        let Ok(file_type) = entry.file_type() else {
            continue;
        };
        if !file_type.is_file()
            || entry.path().extension().and_then(|ext| ext.to_str()) != Some("v1")
        {
            continue;
        }
        let Ok(text) = fs::read_to_string(entry.path()) else {
            continue;
        };
        let Some(project) = parse_gui_workspace_project(&text) else {
            continue;
        };
        projects.push(GuiWorkspaceProjectEntry {
            path: entry.path(),
            project,
        });
    }

    projects.sort_by(|left, right| {
        left.project
            .name
            .cmp(&right.project.name)
            .then_with(|| left.path.cmp(&right.path))
    });
    Ok(projects)
}

pub fn delete_gui_workspace_project(
    projects_dir: &Path,
    project_path: &Path,
) -> Result<GuiWorkspaceProjectDeleteResult, EditorConfigError> {
    if project_path.extension().and_then(|ext| ext.to_str()) != Some("v1") {
        return Err(EditorConfigError::Invalid {
            path: project_path.to_path_buf(),
            message: "workspace project path must end in .v1".to_string(),
        });
    }
    let Some(project_parent) = project_path.parent() else {
        return Err(EditorConfigError::Invalid {
            path: project_path.to_path_buf(),
            message: "workspace project path has no parent directory".to_string(),
        });
    };

    let canonical_projects_dir =
        projects_dir
            .canonicalize()
            .map_err(|source| EditorConfigError::Read {
                path: projects_dir.to_path_buf(),
                source,
            })?;
    let canonical_project_parent =
        project_parent
            .canonicalize()
            .map_err(|source| EditorConfigError::Read {
                path: project_parent.to_path_buf(),
                source,
            })?;
    if canonical_project_parent != canonical_projects_dir {
        return Err(EditorConfigError::Invalid {
            path: project_path.to_path_buf(),
            message: "workspace project path is outside the project directory".to_string(),
        });
    }

    match project_path.canonicalize() {
        Ok(canonical_project_path) => {
            if canonical_project_path.parent() != Some(canonical_projects_dir.as_path()) {
                return Err(EditorConfigError::Invalid {
                    path: project_path.to_path_buf(),
                    message: "workspace project target is outside the project directory"
                        .to_string(),
                });
            }
        }
        Err(error) if error.kind() == io::ErrorKind::NotFound => {
            return Ok(GuiWorkspaceProjectDeleteResult::Missing);
        }
        Err(source) => {
            return Err(EditorConfigError::Read {
                path: project_path.to_path_buf(),
                source,
            });
        }
    }

    match fs::remove_file(project_path) {
        Ok(()) => Ok(GuiWorkspaceProjectDeleteResult::Deleted),
        Err(error) if error.kind() == io::ErrorKind::NotFound => {
            Ok(GuiWorkspaceProjectDeleteResult::Missing)
        }
        Err(source) => Err(EditorConfigError::Remove {
            path: project_path.to_path_buf(),
            source,
        }),
    }
}

fn parse_gui_layout_node(
    node_id: usize,
    pane_count: usize,
    node_specs: &HashMap<usize, String>,
    seen_nodes: &mut HashSet<usize>,
    seen_ordinals: &mut HashSet<usize>,
) -> Option<GuiLayoutNode> {
    if !seen_nodes.insert(node_id) {
        return None;
    }

    let spec = node_specs.get(&node_id)?;
    let parts = spec.split_whitespace().collect::<Vec<_>>();
    match parts.as_slice() {
        ["leaf", raw_ordinal] => {
            let ordinal = raw_ordinal.parse::<usize>().ok()?;
            if ordinal >= pane_count || !seen_ordinals.insert(ordinal) {
                return None;
            }
            Some(GuiLayoutNode::Leaf { ordinal })
        }
        ["split", raw_axis, raw_ratio, raw_first, raw_second] => {
            let axis = match *raw_axis {
                "horizontal" => GuiLayoutAxis::Horizontal,
                "vertical" => GuiLayoutAxis::Vertical,
                _ => return None,
            };
            let ratio_per_mille = raw_ratio.parse::<u16>().ok()?;
            if !(1..=999).contains(&ratio_per_mille) {
                return None;
            }
            let first_id = raw_first.parse::<usize>().ok()?;
            let second_id = raw_second.parse::<usize>().ok()?;
            Some(GuiLayoutNode::Split {
                axis,
                ratio_per_mille,
                first: Box::new(parse_gui_layout_node(
                    first_id,
                    pane_count,
                    node_specs,
                    seen_nodes,
                    seen_ordinals,
                )?),
                second: Box::new(parse_gui_layout_node(
                    second_id,
                    pane_count,
                    node_specs,
                    seen_nodes,
                    seen_ordinals,
                )?),
            })
        }
        _ => None,
    }
}

fn parse_layout_ordinals(value: &str) -> Option<Vec<usize>> {
    let value = value.trim();
    if value.is_empty() {
        return Some(Vec::new());
    }

    value
        .split(',')
        .map(|ordinal| ordinal.trim().parse::<usize>().ok())
        .collect()
}

fn bytes_to_hex(bytes: &[u8]) -> String {
    let mut hex = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        hex.push(hex_digit(byte >> 4));
        hex.push(hex_digit(byte & 0x0f));
    }
    hex
}

fn hex_to_bytes(hex: &str) -> Option<Vec<u8>> {
    let hex = hex.trim();
    if !hex.len().is_multiple_of(2) {
        return None;
    }

    let mut bytes = Vec::with_capacity(hex.len() / 2);
    for chunk in hex.as_bytes().chunks_exact(2) {
        let high = hex_value(chunk[0])?;
        let low = hex_value(chunk[1])?;
        bytes.push((high << 4) | low);
    }
    Some(bytes)
}

fn hex_digit(value: u8) -> char {
    match value {
        0..=9 => char::from(b'0' + value),
        10..=15 => char::from(b'a' + value - 10),
        _ => unreachable!("hex digit nibble must be below 16"),
    }
}

fn hex_value(value: u8) -> Option<u8> {
    match value {
        b'0'..=b'9' => Some(value - b'0'),
        b'a'..=b'f' => Some(value - b'a' + 10),
        b'A'..=b'F' => Some(value - b'A' + 10),
        _ => None,
    }
}

#[cfg(unix)]
fn path_to_hex(path: &Path) -> String {
    use std::os::unix::ffi::OsStrExt;

    bytes_to_hex(path.as_os_str().as_bytes())
}

#[cfg(unix)]
fn path_from_hex(hex: &str) -> Option<PathBuf> {
    use std::ffi::OsString;
    use std::os::unix::ffi::OsStringExt;

    Some(PathBuf::from(OsString::from_vec(hex_to_bytes(hex)?)))
}

#[cfg(not(unix))]
fn path_to_hex(path: &Path) -> String {
    bytes_to_hex(path.to_string_lossy().as_bytes())
}

#[cfg(not(unix))]
fn path_from_hex(hex: &str) -> Option<PathBuf> {
    String::from_utf8(hex_to_bytes(hex)?)
        .ok()
        .map(PathBuf::from)
}

fn gui_workspace_project_slug(name: &str) -> Option<String> {
    let mut slug = String::new();
    let mut previous_dash = false;
    for character in name.chars() {
        if character.is_ascii_alphanumeric() {
            slug.push(character.to_ascii_lowercase());
            previous_dash = false;
        } else if character.is_whitespace() || matches!(character, '-' | '_') {
            if !slug.is_empty() && !previous_dash {
                slug.push('-');
                previous_dash = true;
            }
        } else {
            return None;
        }
    }
    if previous_dash {
        slug.pop();
    }
    (!slug.is_empty()).then_some(slug)
}

fn write_gui_layout_node(
    node: &GuiLayoutNode,
    node_id: usize,
    next_id: &mut usize,
    lines: &mut Vec<String>,
) {
    match node {
        GuiLayoutNode::Leaf { ordinal } => {
            lines.push(format!("node.{node_id} = leaf {ordinal}"));
        }
        GuiLayoutNode::Split {
            axis,
            ratio_per_mille,
            first,
            second,
        } => {
            let first_id = *next_id;
            *next_id += 1;
            let second_id = *next_id;
            *next_id += 1;
            let axis = match axis {
                GuiLayoutAxis::Horizontal => "horizontal",
                GuiLayoutAxis::Vertical => "vertical",
            };
            lines.push(format!(
                "node.{node_id} = split {axis} {ratio_per_mille} {first_id} {second_id}"
            ));
            write_gui_layout_node(first, first_id, next_id, lines);
            write_gui_layout_node(second, second_id, next_id, lines);
        }
    }
}

fn parse_config_string(value: &str) -> Option<&str> {
    value
        .strip_prefix('"')
        .and_then(|value| value.strip_suffix('"'))
        .or_else(|| (!value.contains(char::is_whitespace)).then_some(value))
}

fn parse_config_bool(value: &str) -> Option<bool> {
    match value {
        "true" => Some(true),
        "false" => Some(false),
        _ => None,
    }
}

pub fn save_editor_settings(
    path: &Path,
    settings: EditorSettings,
) -> Result<(), EditorConfigError> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|source| EditorConfigError::CreateDir {
            path: parent.to_path_buf(),
            source,
        })?;
        set_private_config_dir_permissions(parent).map_err(|source| {
            EditorConfigError::CreateDir {
                path: parent.to_path_buf(),
                source,
            }
        })?;
    }

    let text = format!(
        "theme = \"{}\"\nsyntax_theme = \"{}\"\nline_numbers = {}\nwrap = {}\nsearch_case_sensitive = {}\ngui_restore_last_workspace = {}\ngui_reader_mode_enabled = {}\ngui_reader_lines_per_minute = {}\ngui_font_family = \"{}\"\ngui_font_size = {}\ngui_ui_font_size = {}\n",
        settings.theme_id.label(),
        settings.syntax_theme_id.label(),
        settings.show_line_numbers,
        settings.wrap_lines,
        settings.search_case_sensitive,
        settings.gui_restore_last_workspace,
        settings.gui_reader_mode_enabled,
        settings.gui_reader_lines_per_minute,
        settings.gui_font_family.label(),
        settings.gui_font_size,
        settings.gui_ui_font_size
    );
    let temp_path = temporary_config_path(path);
    let result = write_config_temp_then_rename(path, &temp_path, text.as_bytes());
    if result.is_err() {
        let _ = fs::remove_file(&temp_path);
    }
    result
}

fn write_config_temp_then_rename(
    target_path: &Path,
    temp_path: &Path,
    bytes: &[u8],
) -> Result<(), EditorConfigError> {
    let mut options = OpenOptions::new();
    options.write(true).create_new(true);

    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }

    let mut file = options
        .open(temp_path)
        .map_err(|source| EditorConfigError::CreateTemp {
            path: temp_path.to_path_buf(),
            source,
        })?;
    file.write_all(bytes)
        .map_err(|source| EditorConfigError::WriteTemp {
            path: temp_path.to_path_buf(),
            source,
        })?;
    file.sync_all()
        .map_err(|source| EditorConfigError::FlushTemp {
            path: temp_path.to_path_buf(),
            source,
        })?;
    drop(file);

    fs::rename(temp_path, target_path).map_err(|source| EditorConfigError::Rename {
        from: temp_path.to_path_buf(),
        to: target_path.to_path_buf(),
        source,
    })
}

#[cfg(unix)]
fn set_private_config_dir_permissions(path: &Path) -> io::Result<()> {
    use std::os::unix::fs::PermissionsExt;

    fs::set_permissions(path, fs::Permissions::from_mode(0o700))
}

#[cfg(not(unix))]
fn set_private_config_dir_permissions(_path: &Path) -> io::Result<()> {
    Ok(())
}

fn temporary_config_path(path: &Path) -> PathBuf {
    let file_name = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("config.toml");
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |duration| duration.as_nanos());
    path.with_file_name(format!(
        ".{file_name}.kfnotepad-{}-{nonce}.tmp",
        std::process::id()
    ))
}

pub struct SyntaxHighlighter {
    syntax_set: SyntaxSet,
    theme: Theme,
}

pub type SyntaxHighlightedLine = Option<Vec<(SyntectStyle, String)>>;
pub type SyntaxHighlightedLines = Vec<SyntaxHighlightedLine>;

pub struct SyntaxHighlightCacheState {
    highlight_state: HighlightState,
    parse_state: ParseState,
}

impl Default for SyntaxHighlighter {
    fn default() -> Self {
        let syntax_set = SyntaxSet::load_defaults_newlines();
        let theme = ThemeSet::load_defaults()
            .themes
            .remove("base16-ocean.dark")
            .unwrap_or_default();
        Self { syntax_set, theme }
    }
}

impl SyntaxHighlighter {
    fn syntax_for_path(&self, path: &Path) -> &SyntaxReference {
        self.syntax_set
            .find_syntax_for_file(path)
            .ok()
            .flatten()
            .unwrap_or_else(|| self.syntax_set.find_syntax_plain_text())
    }

    fn syntax_for_document(&self, document: &TextDocument) -> &SyntaxReference {
        self.syntax_for_path(&document.path)
    }

    pub fn syntax_name_for_document(&self, document: &TextDocument) -> &str {
        self.syntax_for_document(document).name.as_str()
    }

    pub fn syntax_token_for_document(&self, document: &TextDocument) -> String {
        let syntax = self.syntax_for_document(document);
        syntax
            .file_extensions
            .first()
            .cloned()
            .unwrap_or_else(|| syntax.name.clone())
    }

    pub fn highlight_line(
        &self,
        document: &TextDocument,
        target_line: &str,
    ) -> SyntaxHighlightedLine {
        let syntax = self.syntax_for_document(document);
        if syntax.name == "Plain Text" {
            return None;
        }

        let mut highlighter = HighlightLines::new(syntax, &self.theme);
        let highlighted = highlighter
            .highlight_line(target_line, &self.syntax_set)
            .ok()?;
        Some(
            highlighted
                .into_iter()
                .map(|(style, segment)| (style, segment.to_string()))
                .collect(),
        )
    }

    pub fn highlight_visible_lines(
        &self,
        document: &TextDocument,
        viewport_start: usize,
        visible_rows: usize,
    ) -> SyntaxHighlightedLines {
        let syntax = self.syntax_for_document(document);
        let visible_rows = visible_rows.max(1);
        if syntax.name == "Plain Text" {
            return document
                .buffer
                .lines()
                .iter()
                .skip(viewport_start)
                .take(visible_rows)
                .map(|_| None)
                .collect();
        }

        let mut highlighter = HighlightLines::new(syntax, &self.theme);
        let mut highlighted_lines = Vec::new();
        let end = viewport_start.saturating_add(visible_rows);

        for (index, line) in document.buffer.lines().iter().enumerate().take(end) {
            let highlighted =
                highlighter
                    .highlight_line(line, &self.syntax_set)
                    .ok()
                    .map(|segments| {
                        segments
                            .into_iter()
                            .map(|(style, segment)| (style, segment.to_string()))
                            .collect()
                    });
            if index >= viewport_start {
                highlighted_lines.push(highlighted);
            }
        }

        highlighted_lines
    }

    pub fn highlight_lines_incremental(
        &self,
        document: &TextDocument,
        start_line: usize,
        visible_rows: usize,
        state: Option<SyntaxHighlightCacheState>,
    ) -> (SyntaxHighlightedLines, Option<SyntaxHighlightCacheState>) {
        self.highlight_lines_incremental_for_path(
            &document.path,
            document.buffer.lines(),
            start_line,
            visible_rows,
            state,
        )
    }

    pub fn highlight_lines_incremental_for_path(
        &self,
        path: &Path,
        lines: &[String],
        start_line: usize,
        visible_rows: usize,
        state: Option<SyntaxHighlightCacheState>,
    ) -> (SyntaxHighlightedLines, Option<SyntaxHighlightCacheState>) {
        let syntax = self.syntax_for_path(path);
        let visible_rows = visible_rows.max(1);
        if syntax.name == "Plain Text" {
            return (
                lines
                    .iter()
                    .skip(start_line)
                    .take(visible_rows)
                    .map(|_| None)
                    .collect(),
                None,
            );
        }

        let has_cached_state = state.is_some();
        let mut highlighter = match state {
            Some(state) => {
                HighlightLines::from_state(&self.theme, state.highlight_state, state.parse_state)
            }
            None => HighlightLines::new(syntax, &self.theme),
        };

        let mut highlighted_lines = Vec::new();
        if !has_cached_state {
            for line in lines.iter().take(start_line) {
                let _ = highlighter.highlight_line(line, &self.syntax_set);
            }
        }
        for line in lines.iter().skip(start_line).take(visible_rows) {
            let highlighted =
                highlighter
                    .highlight_line(line, &self.syntax_set)
                    .ok()
                    .map(|segments| {
                        segments
                            .into_iter()
                            .map(|(style, segment)| (style, segment.to_string()))
                            .collect()
                    });
            highlighted_lines.push(highlighted);
        }

        let (highlight_state, parse_state) = highlighter.state();
        (
            highlighted_lines,
            Some(SyntaxHighlightCacheState {
                highlight_state,
                parse_state,
            }),
        )
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FileSidebarState {
    pub current_dir: PathBuf,
    pub entries: Vec<FileSidebarEntry>,
    pub selected: usize,
    pub scroll: usize,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FileSidebarEntry {
    pub label: String,
    pub path: PathBuf,
    pub kind: FileSidebarEntryKind,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FileSidebarEntryKind {
    Parent,
    Directory,
    File,
}

#[derive(Debug)]
pub enum FileSidebarError {
    ReadDir { path: PathBuf, source: io::Error },
}

impl fmt::Display for FileSidebarError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ReadDir { path, source } => {
                write!(formatter, "cannot list {}: {source}", path.display())
            }
        }
    }
}

impl FileSidebarState {
    pub fn load(current_dir: PathBuf) -> Result<Self, FileSidebarError> {
        let current_dir = current_dir.canonicalize().unwrap_or(current_dir);
        Ok(Self {
            entries: list_file_sidebar_entries(&current_dir)?,
            current_dir,
            selected: 0,
            scroll: 0,
        })
    }

    pub fn selected_entry(&self) -> Option<&FileSidebarEntry> {
        self.entries.get(self.selected)
    }

    pub fn selected_entry_for_mouse_row(&mut self, row: u16) -> Option<FileSidebarEntry> {
        if row == 0 {
            return None;
        }
        let index = self.scroll + row as usize - 1;
        let entry = self.entries.get(index)?.clone();
        self.selected = index;
        Some(entry)
    }

    pub fn select_previous_wrapping(&mut self, visible_rows: usize) {
        if self.entries.is_empty() {
            return;
        }
        self.selected = if self.selected == 0 {
            self.entries.len() - 1
        } else {
            self.selected - 1
        };
        self.keep_selection_visible(visible_rows);
    }

    pub fn select_next_wrapping(&mut self, visible_rows: usize) {
        if self.entries.is_empty() {
            return;
        }
        self.selected = (self.selected + 1) % self.entries.len();
        self.keep_selection_visible(visible_rows);
    }

    pub fn scroll_selection_up(&mut self, visible_rows: usize) -> bool {
        if self.entries.is_empty() || self.selected == 0 {
            return false;
        }
        self.selected -= 1;
        self.keep_selection_visible(visible_rows);
        true
    }

    pub fn scroll_selection_down(&mut self, visible_rows: usize) -> bool {
        if self.entries.is_empty() || self.selected + 1 >= self.entries.len() {
            return false;
        }
        self.selected += 1;
        self.keep_selection_visible(visible_rows);
        true
    }

    pub fn keep_selection_visible(&mut self, visible_rows: usize) {
        if self.selected < self.scroll {
            self.scroll = self.selected;
        }
        let visible = visible_rows.max(1);
        if self.selected >= self.scroll + visible {
            self.scroll = self.selected.saturating_sub(visible - 1);
        }
    }
}

pub fn list_file_sidebar_entries(
    current_dir: &Path,
) -> Result<Vec<FileSidebarEntry>, FileSidebarError> {
    let mut directories = Vec::new();
    let mut files = Vec::new();

    if let Some(parent) = current_dir.parent() {
        directories.push(FileSidebarEntry {
            label: String::from("../"),
            path: parent.to_path_buf(),
            kind: FileSidebarEntryKind::Parent,
        });
    }

    let entries = fs::read_dir(current_dir).map_err(|source| FileSidebarError::ReadDir {
        path: current_dir.to_path_buf(),
        source,
    })?;

    for entry in entries {
        let Ok(entry) = entry else {
            continue;
        };
        let Ok(file_type) = entry.file_type() else {
            continue;
        };
        if file_type.is_symlink() {
            continue;
        }
        let Some(name) = entry.file_name().to_str().map(ToOwned::to_owned) else {
            continue;
        };
        if file_type.is_dir() {
            directories.push(FileSidebarEntry {
                label: format!("{name}/"),
                path: entry.path(),
                kind: FileSidebarEntryKind::Directory,
            });
        } else if file_type.is_file() {
            files.push(FileSidebarEntry {
                label: name,
                path: entry.path(),
                kind: FileSidebarEntryKind::File,
            });
        }
    }

    let sort_start = usize::from(
        directories
            .first()
            .is_some_and(|entry| entry.kind == FileSidebarEntryKind::Parent),
    );
    directories[sort_start..].sort_by_key(|entry| entry.label.to_lowercase());
    files.sort_by_key(|entry| entry.label.to_lowercase());
    directories.extend(files);
    Ok(directories)
}

impl<'a> EditorWorkspace<'a> {
    pub fn from_document(document: &'a mut TextDocument) -> Self {
        Self {
            tabs: vec![EditorTab {
                document: EditorTabDocument::Borrowed(document),
                state: EditorTabState::default(),
            }],
            active: 0,
        }
    }

    pub fn active_tab(&self) -> &EditorTab<'a> {
        &self.tabs[self.active]
    }

    pub fn active_tab_mut(&mut self) -> &mut EditorTab<'a> {
        &mut self.tabs[self.active]
    }

    pub fn push_owned_tab(&mut self, document: TextDocument) {
        self.tabs.push(EditorTab {
            document: EditorTabDocument::Owned(document),
            state: EditorTabState::default(),
        });
        self.active = self.tabs.len() - 1;
    }

    pub fn select_previous_tab(&mut self) -> bool {
        if self.tabs.len() <= 1 {
            return false;
        }
        self.active = if self.active == 0 {
            self.tabs.len() - 1
        } else {
            self.active - 1
        };
        true
    }

    pub fn select_next_tab(&mut self) -> bool {
        if self.tabs.len() <= 1 {
            return false;
        }
        self.active = (self.active + 1) % self.tabs.len();
        true
    }

    pub fn close_active_tab(&mut self, confirm_dirty: bool) -> CloseActiveTabResult {
        if self.tabs.len() <= 1 {
            return CloseActiveTabResult::OnlyTab;
        }

        if self.active_tab().document.as_ref().buffer.is_dirty() && !confirm_dirty {
            return CloseActiveTabResult::Dirty;
        }

        let closed_path = self.active_tab().document.as_ref().path.clone();
        self.tabs.remove(self.active);
        if self.active >= self.tabs.len() {
            self.active = self.tabs.len().saturating_sub(1);
        }
        CloseActiveTabResult::Closed { path: closed_path }
    }

    pub fn tab_strip_items(&self) -> Vec<TabStripItem> {
        self.tabs
            .iter()
            .enumerate()
            .map(|(index, tab)| TabStripItem {
                label: document_display_name(&tab.document.as_ref().path).to_string(),
                active: index == self.active,
                dirty: tab.document.as_ref().buffer.is_dirty(),
            })
            .collect()
    }
}

fn document_display_name(path: &Path) -> &str {
    path.file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("untitled")
}

pub fn clamp_cursor_to_document(document: &TextDocument, cursor: &mut Cursor) {
    cursor.row = cursor
        .row
        .min(document.buffer.line_count().saturating_sub(1));
    cursor.column = cursor
        .column
        .min(document.buffer.line_char_count(cursor.row).unwrap_or(0));
}

pub fn move_document_cursor(document: &TextDocument, cursor: &mut Cursor, direction: CursorMove) {
    if let Ok(moved) = document.buffer.move_cursor(*cursor, direction) {
        *cursor = moved;
    }
}

pub fn undo_document_edit(document: &mut TextDocument, cursor: &mut Cursor) -> UndoRedoResult {
    if document.buffer.undo_last_edit() {
        clamp_cursor_to_document(document, cursor);
        UndoRedoResult::Applied
    } else {
        UndoRedoResult::NothingToApply
    }
}

pub fn redo_document_edit(document: &mut TextDocument, cursor: &mut Cursor) -> UndoRedoResult {
    if document.buffer.redo_last_undo() {
        clamp_cursor_to_document(document, cursor);
        UndoRedoResult::Applied
    } else {
        UndoRedoResult::NothingToApply
    }
}

pub fn delete_previous_word(document: &mut TextDocument, cursor: &mut Cursor) -> EditResult {
    if let Ok(moved) = document.buffer.delete_previous_word(*cursor) {
        *cursor = moved;
        EditResult::Modified
    } else {
        EditResult::Unchanged
    }
}

pub fn delete_next_word(document: &mut TextDocument, cursor: &mut Cursor) -> EditResult {
    if let Ok(moved) = document.buffer.delete_next_word(*cursor) {
        *cursor = moved;
        EditResult::Modified
    } else {
        EditResult::Unchanged
    }
}

pub fn delete_to_line_end(document: &mut TextDocument, cursor: &mut Cursor) -> EditResult {
    if let Ok(moved) = document.buffer.delete_to_line_end(*cursor) {
        *cursor = moved;
        EditResult::Modified
    } else {
        EditResult::Unchanged
    }
}

pub fn page_up(document: &TextDocument, cursor: &mut Cursor, page_rows: usize) {
    cursor.row = cursor.row.saturating_sub(page_rows.max(1));
    clamp_cursor_to_document(document, cursor);
}

pub fn page_down(document: &TextDocument, cursor: &mut Cursor, page_rows: usize) {
    cursor.row = cursor
        .row
        .saturating_add(page_rows.max(1))
        .min(document.buffer.line_count().saturating_sub(1));
    clamp_cursor_to_document(document, cursor);
}

pub fn go_to_document_start(cursor: &mut Cursor) {
    *cursor = Cursor { row: 0, column: 0 };
}

pub fn go_to_document_end(document: &TextDocument, cursor: &mut Cursor) {
    let row = document.buffer.line_count().saturating_sub(1);
    let column = document.buffer.line_char_count(row).unwrap_or(0);
    *cursor = Cursor { row, column };
}

pub fn go_to_line(document: &TextDocument, cursor: &mut Cursor, query: &str) -> GoToLineResult {
    if query.is_empty() {
        return GoToLineResult::Empty;
    }

    let Ok(line_number) = query.parse::<usize>() else {
        return GoToLineResult::Invalid;
    };

    if !(1..=document.buffer.line_count()).contains(&line_number) {
        return GoToLineResult::OutOfRange { line_number };
    }

    cursor.row = line_number - 1;
    clamp_cursor_to_document(document, cursor);
    GoToLineResult::Moved { line_number }
}

pub fn repeat_search_next(
    document: &TextDocument,
    cursor: &mut Cursor,
    query: &str,
) -> SearchRepeatResult {
    repeat_search_next_with_mode(
        document,
        cursor,
        query,
        SearchMode {
            case_sensitive: true,
        },
    )
}

pub fn repeat_search_next_with_mode(
    document: &TextDocument,
    cursor: &mut Cursor,
    query: &str,
    mode: SearchMode,
) -> SearchRepeatResult {
    if query.is_empty() {
        return SearchRepeatResult::NoPreviousSearch;
    }

    let start = next_search_start(document, *cursor);
    if let Some(found) = document
        .buffer
        .find_next_with_mode(query, start, mode)
        .or_else(|| {
            document
                .buffer
                .find_next_with_mode(query, Cursor { row: 0, column: 0 }, mode)
        })
    {
        *cursor = found;
        SearchRepeatResult::Found {
            query: query.to_string(),
        }
    } else {
        SearchRepeatResult::NoMatch {
            query: query.to_string(),
        }
    }
}

pub fn repeat_search_previous(
    document: &TextDocument,
    cursor: &mut Cursor,
    query: &str,
) -> SearchRepeatResult {
    repeat_search_previous_with_mode(
        document,
        cursor,
        query,
        SearchMode {
            case_sensitive: true,
        },
    )
}

pub fn repeat_search_previous_with_mode(
    document: &TextDocument,
    cursor: &mut Cursor,
    query: &str,
    mode: SearchMode,
) -> SearchRepeatResult {
    if query.is_empty() {
        return SearchRepeatResult::NoPreviousSearch;
    }

    if let Some(found) = document
        .buffer
        .find_previous_with_mode(query, *cursor, mode)
    {
        *cursor = found;
        SearchRepeatResult::Found {
            query: query.to_string(),
        }
    } else {
        SearchRepeatResult::NoMatch {
            query: query.to_string(),
        }
    }
}

fn next_search_start(document: &TextDocument, cursor: Cursor) -> Cursor {
    let columns = document.buffer.line_char_count(cursor.row).unwrap_or(0);
    if cursor.column < columns {
        return Cursor {
            row: cursor.row,
            column: cursor.column + 1,
        };
    }
    if cursor.row + 1 < document.buffer.line_count() {
        return Cursor {
            row: cursor.row + 1,
            column: 0,
        };
    }
    Cursor { row: 0, column: 0 }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ManagedNoteEntry {
    pub file_name: String,
    pub path: PathBuf,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ManagedNoteDeleteResult {
    Deleted,
    Missing,
}

#[derive(Debug)]
pub enum OpenError {
    Access {
        path: PathBuf,
        source: io::Error,
    },
    Directory {
        path: PathBuf,
    },
    Symlink {
        path: PathBuf,
    },
    NotRegular {
        path: PathBuf,
    },
    TooLarge {
        path: PathBuf,
        bytes: u64,
        limit: u64,
    },
    ReadUtf8 {
        path: PathBuf,
        source: io::Error,
    },
}

#[derive(Debug)]
pub enum SaveError {
    Metadata {
        path: PathBuf,
        source: io::Error,
    },
    Directory {
        path: PathBuf,
    },
    Symlink {
        path: PathBuf,
    },
    NotRegular {
        path: PathBuf,
    },
    ExternalModification {
        path: PathBuf,
    },
    TooLarge {
        path: PathBuf,
        bytes: u64,
        limit: u64,
    },
    CreateTemp {
        path: PathBuf,
        source: io::Error,
    },
    WriteTemp {
        path: PathBuf,
        source: io::Error,
    },
    FlushTemp {
        path: PathBuf,
        source: io::Error,
    },
    Rename {
        from: PathBuf,
        to: PathBuf,
        source: io::Error,
    },
}

#[derive(Debug)]
pub enum ManagedNotesError {
    MissingDataHome,
    InvalidNoteName,
    InvalidNotePath { path: PathBuf, message: String },
    CreateNotesDir { path: PathBuf, source: io::Error },
    CreateNote { path: PathBuf, source: SaveError },
    OpenNote { path: PathBuf, source: OpenError },
    ListNotesDir { path: PathBuf, source: io::Error },
    InspectNote { path: PathBuf, source: io::Error },
    RemoveNote { path: PathBuf, source: io::Error },
}

impl fmt::Display for ManagedNotesError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ManagedNotesError::MissingDataHome => {
                write!(formatter, "cannot resolve managed notes directory")
            }
            ManagedNotesError::InvalidNoteName => write!(formatter, "invalid managed note name"),
            ManagedNotesError::InvalidNotePath { path, message } => {
                write!(
                    formatter,
                    "invalid managed note path {}: {message}",
                    path.display()
                )
            }
            ManagedNotesError::CreateNotesDir { path, source } => {
                write!(
                    formatter,
                    "cannot create managed notes directory {}: {source}",
                    path.display()
                )
            }
            ManagedNotesError::CreateNote { path, source } => {
                write!(
                    formatter,
                    "cannot create managed note {}: {source}",
                    path.display()
                )
            }
            ManagedNotesError::OpenNote { path, source } => {
                write!(
                    formatter,
                    "cannot open managed note {}: {source}",
                    path.display()
                )
            }
            ManagedNotesError::ListNotesDir { path, source } => {
                write!(
                    formatter,
                    "cannot list managed notes directory {}: {source}",
                    path.display()
                )
            }
            ManagedNotesError::InspectNote { path, source } => {
                write!(
                    formatter,
                    "cannot inspect managed note entry {}: {source}",
                    path.display()
                )
            }
            ManagedNotesError::RemoveNote { path, source } => {
                write!(
                    formatter,
                    "cannot delete managed note {}: {source}",
                    path.display()
                )
            }
        }
    }
}

impl fmt::Display for SaveError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SaveError::Metadata { path, source } => {
                write!(
                    formatter,
                    "cannot inspect {} before save: {source}",
                    path.display()
                )
            }
            SaveError::Directory { path } => {
                write!(formatter, "cannot save over directory {}", path.display())
            }
            SaveError::Symlink { path } => {
                write!(
                    formatter,
                    "refusing to save through symlink {}",
                    path.display()
                )
            }
            SaveError::NotRegular { path } => {
                write!(
                    formatter,
                    "refusing to save over non-regular file {}",
                    path.display()
                )
            }
            SaveError::ExternalModification { path } => {
                write!(
                    formatter,
                    "file changed on disk since open or last save: {}; reload or save as a new file",
                    path.display()
                )
            }
            SaveError::TooLarge { path, bytes, limit } => {
                write!(
                    formatter,
                    "cannot save {}: {bytes} bytes exceeds {limit} bytes",
                    path.display()
                )
            }
            SaveError::CreateTemp { path, source } => {
                write!(
                    formatter,
                    "cannot create temporary file {}: {source}",
                    path.display()
                )
            }
            SaveError::WriteTemp { path, source } => {
                write!(
                    formatter,
                    "cannot write temporary file {}: {source}",
                    path.display()
                )
            }
            SaveError::FlushTemp { path, source } => {
                write!(
                    formatter,
                    "cannot flush temporary file {}: {source}",
                    path.display()
                )
            }
            SaveError::Rename { from, to, source } => {
                write!(
                    formatter,
                    "cannot replace {} with {}: {source}",
                    to.display(),
                    from.display()
                )
            }
        }
    }
}

pub fn managed_notes_dir(
    xdg_data_home: Option<&Path>,
    home: Option<&Path>,
) -> Result<PathBuf, ManagedNotesError> {
    if let Some(xdg_data_home) = xdg_data_home.filter(|path| !path.as_os_str().is_empty()) {
        return Ok(xdg_data_home.join("kfnotepad").join("notes"));
    }

    if let Some(home) = home.filter(|path| !path.as_os_str().is_empty()) {
        return Ok(home
            .join(".local")
            .join("share")
            .join("kfnotepad")
            .join("notes"));
    }

    Err(ManagedNotesError::MissingDataHome)
}

pub fn note_slug(title: &str) -> Result<String, ManagedNotesError> {
    let title = title.trim();
    if title.is_empty()
        || title == "."
        || title == ".."
        || title.starts_with('.')
        || title.contains(['/', '\\'])
        || title.chars().any(char::is_control)
    {
        return Err(ManagedNotesError::InvalidNoteName);
    }

    let mut slug = String::new();
    let mut pending_separator = false;
    for character in title.chars() {
        if character.is_alphanumeric() {
            if pending_separator && !slug.is_empty() {
                slug.push('-');
            }
            for lowercase in character.to_lowercase() {
                slug.push(lowercase);
            }
            pending_separator = false;
        } else {
            pending_separator = true;
        }
    }

    if slug.is_empty() {
        return Err(ManagedNotesError::InvalidNoteName);
    }

    slug.push_str(".md");
    Ok(slug)
}

pub fn managed_note_path(notes_dir: &Path, title: &str) -> Result<PathBuf, ManagedNotesError> {
    Ok(notes_dir.join(note_slug(title)?))
}

pub fn open_or_create_managed_note(
    notes_dir: &Path,
    title: &str,
) -> Result<TextDocument, ManagedNotesError> {
    let path = managed_note_path(notes_dir, title)?;

    fs::create_dir_all(notes_dir).map_err(|source| ManagedNotesError::CreateNotesDir {
        path: notes_dir.to_path_buf(),
        source,
    })?;

    if !path.exists() {
        let empty = TextBuffer::from_text("");
        save_text_buffer(&path, &empty).map_err(|source| ManagedNotesError::CreateNote {
            path: path.clone(),
            source,
        })?;
    }

    open_text_file(&path).map_err(|source| ManagedNotesError::OpenNote {
        path: path.clone(),
        source,
    })
}

pub fn list_managed_notes(notes_dir: &Path) -> Result<Vec<ManagedNoteEntry>, ManagedNotesError> {
    let directory = match fs::read_dir(notes_dir) {
        Ok(directory) => directory,
        Err(source) if source.kind() == io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(source) => {
            return Err(ManagedNotesError::ListNotesDir {
                path: notes_dir.to_path_buf(),
                source,
            });
        }
    };

    let mut notes = Vec::new();
    for entry in directory {
        let entry = entry.map_err(|source| ManagedNotesError::ListNotesDir {
            path: notes_dir.to_path_buf(),
            source,
        })?;
        let path = entry.path();
        let file_type = entry
            .file_type()
            .map_err(|source| ManagedNotesError::InspectNote {
                path: path.clone(),
                source,
            })?;

        if !file_type.is_file() || !is_managed_note_file_name(&path) {
            continue;
        }

        let file_name = path
            .file_name()
            .and_then(|name| name.to_str())
            .expect("validated managed note file name")
            .to_string();
        notes.push(ManagedNoteEntry { file_name, path });
    }

    notes.sort_by(|left, right| left.file_name.cmp(&right.file_name));
    Ok(notes)
}

pub fn delete_managed_note(
    notes_dir: &Path,
    note_path: &Path,
) -> Result<ManagedNoteDeleteResult, ManagedNotesError> {
    if !is_managed_note_file_name(note_path) {
        return Err(ManagedNotesError::InvalidNotePath {
            path: note_path.to_path_buf(),
            message: "managed note path must be a visible .md file".to_string(),
        });
    }

    let Some(note_parent) = note_path.parent() else {
        return Err(ManagedNotesError::InvalidNotePath {
            path: note_path.to_path_buf(),
            message: "managed note path has no parent directory".to_string(),
        });
    };

    let canonical_notes_dir = match notes_dir.canonicalize() {
        Ok(path) => path,
        Err(error) if error.kind() == io::ErrorKind::NotFound => {
            return Ok(ManagedNoteDeleteResult::Missing);
        }
        Err(source) => {
            return Err(ManagedNotesError::ListNotesDir {
                path: notes_dir.to_path_buf(),
                source,
            });
        }
    };

    let canonical_note_parent = match note_parent.canonicalize() {
        Ok(path) => path,
        Err(error) if error.kind() == io::ErrorKind::NotFound => {
            return Ok(ManagedNoteDeleteResult::Missing);
        }
        Err(source) => {
            return Err(ManagedNotesError::InspectNote {
                path: note_parent.to_path_buf(),
                source,
            });
        }
    };

    if canonical_note_parent != canonical_notes_dir {
        return Err(ManagedNotesError::InvalidNotePath {
            path: note_path.to_path_buf(),
            message: "managed note path is outside the notes directory".to_string(),
        });
    }

    let metadata = match fs::symlink_metadata(note_path) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == io::ErrorKind::NotFound => {
            return Ok(ManagedNoteDeleteResult::Missing);
        }
        Err(source) => {
            return Err(ManagedNotesError::InspectNote {
                path: note_path.to_path_buf(),
                source,
            });
        }
    };
    let file_type = metadata.file_type();
    if file_type.is_symlink() {
        return Err(ManagedNotesError::InvalidNotePath {
            path: note_path.to_path_buf(),
            message: "refusing to delete a symlinked managed note".to_string(),
        });
    }
    if !file_type.is_file() {
        return Err(ManagedNotesError::InvalidNotePath {
            path: note_path.to_path_buf(),
            message: "managed note path is not a normal file".to_string(),
        });
    }

    match fs::remove_file(note_path) {
        Ok(()) => Ok(ManagedNoteDeleteResult::Deleted),
        Err(error) if error.kind() == io::ErrorKind::NotFound => {
            Ok(ManagedNoteDeleteResult::Missing)
        }
        Err(source) => Err(ManagedNotesError::RemoveNote {
            path: note_path.to_path_buf(),
            source,
        }),
    }
}

fn is_managed_note_file_name(path: &Path) -> bool {
    let Some(file_name) = path.file_name().and_then(|name| name.to_str()) else {
        return false;
    };

    if file_name.starts_with('.') {
        return false;
    }

    let Some(stem) = path.file_stem().and_then(|stem| stem.to_str()) else {
        return false;
    };

    !stem.is_empty() && path.extension().and_then(|extension| extension.to_str()) == Some("md")
}

impl fmt::Display for OpenError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            OpenError::Access { path, source } => {
                write!(formatter, "cannot access {}: {source}", path.display())
            }
            OpenError::Directory { path } => {
                write!(
                    formatter,
                    "{} is a directory, not a text file",
                    path.display()
                )
            }
            OpenError::Symlink { path } => {
                write!(formatter, "refusing to open symlink {}", path.display())
            }
            OpenError::NotRegular { path } => {
                write!(
                    formatter,
                    "refusing to open non-regular file {}",
                    path.display()
                )
            }
            OpenError::TooLarge { path, bytes, limit } => {
                write!(
                    formatter,
                    "{} is too large to open safely: {bytes} bytes exceeds {limit} bytes",
                    path.display()
                )
            }
            OpenError::ReadUtf8 { path, source } => {
                write!(
                    formatter,
                    "cannot read {} as UTF-8 text: {source}",
                    path.display()
                )
            }
        }
    }
}

impl TextBuffer {
    pub fn from_text(text: &str) -> Self {
        let mut lines: Vec<String> = text.lines().map(ToString::to_string).collect();
        if lines.is_empty() {
            lines.push(String::new());
        }

        Self {
            lines,
            trailing_newline: text.ends_with('\n'),
            dirty: false,
            undo_history: Vec::new(),
            redo_history: Vec::new(),
            file_snapshot: None,
        }
    }

    pub fn lines(&self) -> &[String] {
        &self.lines
    }

    pub fn line(&self, row: usize) -> Option<&str> {
        self.lines.get(row).map(String::as_str)
    }

    pub fn line_count(&self) -> usize {
        self.lines.len()
    }

    pub fn has_trailing_newline(&self) -> bool {
        self.trailing_newline
    }

    pub fn is_dirty(&self) -> bool {
        self.dirty
    }

    pub fn mark_clean(&mut self) {
        self.dirty = false;
        self.undo_history.clear();
        self.redo_history.clear();
    }

    pub fn file_snapshot(&self) -> Option<&FileSnapshot> {
        self.file_snapshot.as_ref()
    }

    pub fn set_file_snapshot(&mut self, snapshot: Option<FileSnapshot>) {
        self.file_snapshot = snapshot;
    }

    pub fn to_text(&self) -> String {
        let mut text = self.lines.join("\n");
        if self.trailing_newline {
            text.push('\n');
        }
        text
    }

    pub fn replace_text(&mut self, text: &str) {
        if self.to_text() == text {
            return;
        }

        let mut replacement = Self::from_text(text);
        replacement.dirty = true;
        replacement.file_snapshot = self.file_snapshot.clone();
        *self = replacement;
    }

    pub fn line_char_count(&self, row: usize) -> Result<usize, BufferError> {
        let rows = self.lines.len();
        let line = self
            .lines
            .get(row)
            .ok_or(BufferError::RowOutOfBounds { row, rows })?;
        Ok(line.chars().count())
    }

    pub fn move_cursor(
        &self,
        cursor: Cursor,
        direction: CursorMove,
    ) -> Result<Cursor, BufferError> {
        self.validate_cursor(cursor)?;

        let moved = match direction {
            CursorMove::Left if cursor.column > 0 => Cursor {
                row: cursor.row,
                column: cursor.column - 1,
            },
            CursorMove::Left if cursor.row > 0 => {
                let row = cursor.row - 1;
                Cursor {
                    row,
                    column: self.line_char_count(row)?,
                }
            }
            CursorMove::Left => cursor,
            CursorMove::Right if cursor.column < self.line_char_count(cursor.row)? => Cursor {
                row: cursor.row,
                column: cursor.column + 1,
            },
            CursorMove::Right if cursor.row + 1 < self.lines.len() => Cursor {
                row: cursor.row + 1,
                column: 0,
            },
            CursorMove::Right => cursor,
            CursorMove::WordLeft => self.previous_word_cursor(cursor),
            CursorMove::WordRight => self.next_word_cursor(cursor),
            CursorMove::Up if cursor.row > 0 => {
                self.cursor_on_row(cursor.row - 1, cursor.column)?
            }
            CursorMove::Up => cursor,
            CursorMove::Down if cursor.row + 1 < self.lines.len() => {
                self.cursor_on_row(cursor.row + 1, cursor.column)?
            }
            CursorMove::Down => cursor,
        };

        Ok(moved)
    }

    fn previous_word_cursor(&self, cursor: Cursor) -> Cursor {
        let mut row = cursor.row;
        let mut column = cursor.column;

        loop {
            let chars: Vec<char> = self.lines[row].chars().collect();
            let mut index = column.min(chars.len());
            while index > 0 && !is_word_character(chars[index - 1]) {
                index -= 1;
            }
            if index > 0 {
                while index > 0 && is_word_character(chars[index - 1]) {
                    index -= 1;
                }
                return Cursor { row, column: index };
            }
            if row == 0 {
                return Cursor { row: 0, column: 0 };
            }
            row -= 1;
            column = self.lines[row].chars().count();
        }
    }

    fn next_word_cursor(&self, cursor: Cursor) -> Cursor {
        let mut row = cursor.row;
        let mut column = cursor.column;

        loop {
            let chars: Vec<char> = self.lines[row].chars().collect();
            let mut index = column.min(chars.len());
            if index < chars.len() && is_word_character(chars[index]) {
                while index < chars.len() && is_word_character(chars[index]) {
                    index += 1;
                }
            }
            while index < chars.len() && !is_word_character(chars[index]) {
                index += 1;
            }
            if index < chars.len() {
                return Cursor { row, column: index };
            }
            if row + 1 >= self.lines.len() {
                return Cursor {
                    row,
                    column: chars.len(),
                };
            }
            row += 1;
            column = 0;
        }
    }

    fn next_word_delete_end_cursor(&self, cursor: Cursor) -> Cursor {
        let mut row = cursor.row;
        let mut column = cursor.column;

        loop {
            let chars: Vec<char> = self.lines[row].chars().collect();
            let mut index = column.min(chars.len());

            if index < chars.len() && is_word_character(chars[index]) {
                while index < chars.len() && is_word_character(chars[index]) {
                    index += 1;
                }
                return Cursor { row, column: index };
            }

            while index < chars.len() && !is_word_character(chars[index]) {
                index += 1;
            }

            if index < chars.len() {
                while index < chars.len() && is_word_character(chars[index]) {
                    index += 1;
                }
                return Cursor { row, column: index };
            }

            if row + 1 >= self.lines.len() {
                return Cursor {
                    row,
                    column: chars.len(),
                };
            }

            row += 1;
            column = 0;
        }
    }

    pub fn insert_char(
        &mut self,
        row: usize,
        column: usize,
        value: char,
    ) -> Result<(), BufferError> {
        if value == '\n' {
            return Err(BufferError::UseInsertNewline);
        }

        let rows = self.lines.len();
        let byte_index = {
            let line = self
                .lines
                .get(row)
                .ok_or(BufferError::RowOutOfBounds { row, rows })?;
            byte_index_for_char_column(line, column)?
        };

        self.record_undo();
        let line = self
            .lines
            .get_mut(row)
            .ok_or(BufferError::RowOutOfBounds { row, rows })?;
        line.insert(byte_index, value);
        self.dirty = true;
        Ok(())
    }

    pub fn replace_char(
        &mut self,
        row: usize,
        column: usize,
        value: char,
    ) -> Result<(), BufferError> {
        if value == '\n' {
            return Err(BufferError::UseInsertNewline);
        }

        let rows = self.lines.len();
        let line_columns = self.line_char_count(row)?;
        if column >= line_columns {
            return self.insert_char(row, column, value);
        }

        let start = {
            let line = self
                .lines
                .get(row)
                .ok_or(BufferError::RowOutOfBounds { row, rows })?;
            byte_index_for_char_column(line, column)?
        };
        let end = {
            let line = self
                .lines
                .get(row)
                .ok_or(BufferError::RowOutOfBounds { row, rows })?;
            byte_index_for_char_column(line, column + 1)?
        };

        self.record_undo();
        let line = self
            .lines
            .get_mut(row)
            .ok_or(BufferError::RowOutOfBounds { row, rows })?;
        line.replace_range(start..end, &value.to_string());
        self.dirty = true;
        Ok(())
    }

    pub fn insert_newline(&mut self, row: usize, column: usize) -> Result<(), BufferError> {
        let rows = self.lines.len();
        let byte_index = {
            let line = self
                .lines
                .get(row)
                .ok_or(BufferError::RowOutOfBounds { row, rows })?;
            byte_index_for_char_column(line, column)?
        };

        self.record_undo();
        let line = self
            .lines
            .get_mut(row)
            .ok_or(BufferError::RowOutOfBounds { row, rows })?;
        let remainder = line.split_off(byte_index);
        self.lines.insert(row + 1, remainder);
        self.dirty = true;
        Ok(())
    }

    pub fn delete_char(&mut self, row: usize, column: usize) -> Result<(), BufferError> {
        let rows = self.lines.len();
        let line_columns = self.line_char_count(row)?;

        if column < line_columns {
            let start = {
                let line = self
                    .lines
                    .get(row)
                    .ok_or(BufferError::RowOutOfBounds { row, rows })?;
                byte_index_for_char_column(line, column)?
            };
            let end = {
                let line = self
                    .lines
                    .get(row)
                    .ok_or(BufferError::RowOutOfBounds { row, rows })?;
                byte_index_for_char_column(line, column + 1)?
            };

            self.record_undo();
            let line = self
                .lines
                .get_mut(row)
                .ok_or(BufferError::RowOutOfBounds { row, rows })?;
            line.replace_range(start..end, "");
            self.dirty = true;
            return Ok(());
        }

        if column == line_columns && row + 1 < rows {
            self.record_undo();
            let next_line = self.lines.remove(row + 1);
            self.lines[row].push_str(&next_line);
            self.dirty = true;
            return Ok(());
        }

        if column == line_columns {
            return Ok(());
        }

        Err(BufferError::ColumnOutOfBounds {
            column,
            columns: line_columns,
        })
    }

    pub fn delete_before_cursor(&mut self, cursor: Cursor) -> Result<Cursor, BufferError> {
        self.validate_cursor(cursor)?;

        if cursor.column > 0 {
            self.delete_char(cursor.row, cursor.column - 1)?;
            return Ok(Cursor {
                row: cursor.row,
                column: cursor.column - 1,
            });
        }

        if cursor.row > 0 {
            let previous_row = cursor.row - 1;
            let previous_columns = self.line_char_count(previous_row)?;
            self.record_undo();
            let current_line = self.lines.remove(cursor.row);
            self.lines[previous_row].push_str(&current_line);
            self.dirty = true;
            return Ok(Cursor {
                row: previous_row,
                column: previous_columns,
            });
        }

        Ok(cursor)
    }

    pub fn delete_previous_word(&mut self, cursor: Cursor) -> Result<Cursor, BufferError> {
        self.validate_cursor(cursor)?;
        let start = self.previous_word_cursor(cursor);
        self.delete_range(start, cursor)?;
        Ok(start)
    }

    pub fn delete_next_word(&mut self, cursor: Cursor) -> Result<Cursor, BufferError> {
        self.validate_cursor(cursor)?;
        let end = self.next_word_delete_end_cursor(cursor);
        self.delete_range(cursor, end)?;
        Ok(cursor)
    }

    pub fn delete_to_line_end(&mut self, cursor: Cursor) -> Result<Cursor, BufferError> {
        self.validate_cursor(cursor)?;
        let end = Cursor {
            row: cursor.row,
            column: self.line_char_count(cursor.row)?,
        };
        self.delete_range(cursor, end)?;
        Ok(cursor)
    }

    fn delete_range(&mut self, start: Cursor, end: Cursor) -> Result<(), BufferError> {
        self.validate_cursor(start)?;
        self.validate_cursor(end)?;

        if start == end {
            return Ok(());
        }

        self.record_undo();

        if start.row == end.row {
            let rows = self.lines.len();
            let line = self
                .lines
                .get_mut(start.row)
                .ok_or(BufferError::RowOutOfBounds {
                    row: start.row,
                    rows,
                })?;
            let start_byte = byte_index_for_char_column(line, start.column)?;
            let end_byte = byte_index_for_char_column(line, end.column)?;
            line.replace_range(start_byte..end_byte, "");
            self.dirty = true;
            return Ok(());
        }

        let start_prefix = {
            let line = self
                .lines
                .get(start.row)
                .ok_or(BufferError::RowOutOfBounds {
                    row: start.row,
                    rows: self.lines.len(),
                })?;
            let start_byte = byte_index_for_char_column(line, start.column)?;
            line[..start_byte].to_string()
        };
        let end_suffix = {
            let line = self.lines.get(end.row).ok_or(BufferError::RowOutOfBounds {
                row: end.row,
                rows: self.lines.len(),
            })?;
            let end_byte = byte_index_for_char_column(line, end.column)?;
            line[end_byte..].to_string()
        };

        self.lines[start.row] = format!("{start_prefix}{end_suffix}");
        self.lines.drain((start.row + 1)..=end.row);
        self.dirty = true;
        Ok(())
    }

    pub fn undo_last_edit(&mut self) -> bool {
        let Some(snapshot) = self.undo_history.pop() else {
            return false;
        };

        self.redo_history.push(BufferSnapshot {
            lines: self.lines.clone(),
            trailing_newline: self.trailing_newline,
        });
        self.lines = snapshot.lines;
        self.trailing_newline = snapshot.trailing_newline;
        self.dirty = true;
        true
    }

    pub fn redo_last_undo(&mut self) -> bool {
        let Some(snapshot) = self.redo_history.pop() else {
            return false;
        };

        self.undo_history.push(BufferSnapshot {
            lines: self.lines.clone(),
            trailing_newline: self.trailing_newline,
        });
        self.lines = snapshot.lines;
        self.trailing_newline = snapshot.trailing_newline;
        self.dirty = true;
        true
    }

    pub fn find_next(&self, query: &str, start: Cursor) -> Option<Cursor> {
        self.find_next_with_mode(
            query,
            start,
            SearchMode {
                case_sensitive: true,
            },
        )
    }

    pub fn find_next_with_mode(
        &self,
        query: &str,
        start: Cursor,
        mode: SearchMode,
    ) -> Option<Cursor> {
        if query.is_empty() || self.validate_cursor(start).is_err() {
            return None;
        }

        for row in start.row..self.lines.len() {
            let column = if row == start.row { start.column } else { 0 };
            if let Some(cursor) = find_in_line_with_mode(&self.lines[row], query, column, row, mode)
            {
                return Some(cursor);
            }
        }

        for row in 0..start.row {
            if let Some(cursor) = find_in_line_with_mode(&self.lines[row], query, 0, row, mode) {
                return Some(cursor);
            }
        }

        None
    }

    pub fn find_previous(&self, query: &str, start: Cursor) -> Option<Cursor> {
        self.find_previous_with_mode(
            query,
            start,
            SearchMode {
                case_sensitive: true,
            },
        )
    }

    pub fn find_previous_with_mode(
        &self,
        query: &str,
        start: Cursor,
        mode: SearchMode,
    ) -> Option<Cursor> {
        if query.is_empty() || self.validate_cursor(start).is_err() {
            return None;
        }

        for row in (0..=start.row).rev() {
            let column = if row == start.row {
                start.column
            } else {
                self.lines[row].chars().count()
            };
            if let Some(cursor) =
                find_last_in_line_before_with_mode(&self.lines[row], query, column, row, mode)
            {
                return Some(cursor);
            }
        }

        for row in (start.row + 1..self.lines.len()).rev() {
            let column = self.lines[row].chars().count();
            if let Some(cursor) =
                find_last_in_line_before_with_mode(&self.lines[row], query, column, row, mode)
            {
                return Some(cursor);
            }
        }

        None
    }

    fn record_undo(&mut self) {
        self.undo_history.push(BufferSnapshot {
            lines: self.lines.clone(),
            trailing_newline: self.trailing_newline,
        });
        if self.undo_history.len() > MAX_UNDO_HISTORY {
            let excess = self.undo_history.len() - MAX_UNDO_HISTORY;
            self.undo_history.drain(0..excess);
        }
        self.redo_history.clear();
    }

    fn validate_cursor(&self, cursor: Cursor) -> Result<(), BufferError> {
        let columns = self.line_char_count(cursor.row)?;
        if cursor.column > columns {
            return Err(BufferError::ColumnOutOfBounds {
                column: cursor.column,
                columns,
            });
        }
        Ok(())
    }

    fn cursor_on_row(&self, row: usize, requested_column: usize) -> Result<Cursor, BufferError> {
        Ok(Cursor {
            row,
            column: requested_column.min(self.line_char_count(row)?),
        })
    }
}

pub fn parse_args(args: &[String]) -> Result<Command, String> {
    match args {
        [] => Ok(Command::LaunchEmpty),
        [flag] if flag == "--help" || flag == "-h" => Ok(Command::Help),
        [flag] if flag == "--version" || flag == "-V" => Ok(Command::Version),
        [flag] if flag == "--notes" => Ok(Command::ListManagedNotes),
        [flag, title] if flag == "--note" && title.trim().is_empty() => {
            Err("managed note title must not be empty".to_string())
        }
        [flag, title] if flag == "--note" => Ok(Command::OpenManagedNote(title.clone())),
        [path] if path.starts_with('-') => Err(format!("unknown option: {path}")),
        [path] if path.trim().is_empty() => Err("file path must not be empty".to_string()),
        [path] => Ok(Command::InspectFile(path.clone())),
        _ => Err(
            "expected zero arguments, --help, --version, --notes, --note TITLE, or one file path"
                .to_string(),
        ),
    }
}

pub fn help_text() -> &'static str {
    r#"kfnotepad 0.1.0

Usage:
  kfnotepad [FILE]
  kfnotepad --note TITLE
  kfnotepad --notes
  kfnotepad --help
  kfnotepad --version

Behavior:
  With FILE in an interactive terminal, opens the editor.
  With FILE in a non-interactive context, prints a read-only summary.
  With --note TITLE, creates or opens a managed Markdown note under the local data directory.
  With --notes, lists managed note filenames in deterministic order.
  With no FILE, verifies the executable can launch unless workspace restore is enabled.

Editor keys:
  Arrow keys move the cursor.
  Mouse clicks move the cursor, select visible tabs, and operate menu items.
  Ctrl-B toggles the file sidebar.
  Ctrl-N creates a new untitled file tab without writing it until save.
  In the file sidebar, Enter opens or focuses the selected file as a tab.
  In the file sidebar, Ctrl-N creates a file, Ctrl-D creates a directory, and Delete prompts for removal.
  Ctrl-PageUp and Ctrl-PageDown switch tabs; Ctrl-F4 closes the active tab.
  F10 -> Workspace saves, lists, opens, deletes, and restores TUI workspace projects.
  Ctrl-Left and Ctrl-Right move by word.
  PageUp and PageDown move by one visible page.
  F2 opens the command palette for typed access to menu commands.
  F10 opens the keyboard menu.
  Home, End, Ctrl-A, and Ctrl-E move within the current line.
  Ctrl-Home and Ctrl-End move to the start or end of the document.
  Printable characters insert text.
  Tab inserts one indentation level.
  Shift-Tab removes one indentation level before the cursor.
  Enter splits the current line.
  Backspace deletes before the cursor.
  Delete removes at the cursor.
  Ctrl-Backspace and Ctrl-Delete delete by word.
  Ctrl-K deletes to the end of the current line.
  Ctrl-Z undoes recent edits since the last save; undo history is bounded.
  Ctrl-Y redoes the last undone edit.
  Ctrl-F searches text; accepted matches are highlighted.
  Search defaults to ignore case; Ctrl-Shift-F toggles exact-case search.
  Up and Down recall the last ten search queries while the search prompt is active.
  F3 repeats the last search forward.
  Shift-F3 repeats the last search backward.
  Ctrl-G goes to a line number.
  Ctrl-L toggles line numbers.
  Ctrl-T cycles built-in themes.
  Ctrl-Shift-T cycles syntax highlighting themes.
  Ctrl-R toggles reader mode auto-scroll.
  Ctrl-W toggles word wrap.
  Ctrl-S saves.
  Ctrl-Q quits; dirty buffers require Ctrl-Q twice.
"#
}

pub fn tui_help_document_text() -> &'static str {
    r#"# kfnotepad help

kfnotepad is a local UTF-8 text-file editor for modern terminals. It edits normal files on disk; there is no database, account, sync service, or autosave.

## Editor basics

- Type normally in the active document.
- Ctrl-S saves the active document.
- Ctrl-N creates a new untitled file tab without writing it until save.
- Ctrl-Q quits; modified documents require Ctrl-Q twice.
- F2 opens the command palette. Type part of a command, shortcut, or menu group; Up/Down chooses a result; Enter runs it; Esc closes it.
- F10 opens the menu bar. Left/Right or Tab/Shift-Tab changes groups, Up/Down chooses an item, Home/End jumps within a menu, and Enter runs the selected item.
- Mouse clicks move the cursor and operate menu items.
- Clipboard copy and paste use the terminal's native selection and paste behavior.

## Movement and editing

- Arrow keys move the cursor.
- Home and End move within the current line.
- Ctrl-A and Ctrl-E also move to the start or end of the current line.
- Ctrl-Home and Ctrl-End move to the document start or end.
- Ctrl-Left and Ctrl-Right move by word.
- PageUp and PageDown move by one visible page.
- Enter splits the current line.
- Backspace deletes before the cursor.
- Delete removes at the cursor.
- Ctrl-Backspace and Ctrl-Delete delete by word.
- Ctrl-K deletes to the end of the current line.
- Tab inserts one four-space indentation level.
- Shift-Tab removes up to one indentation level before the cursor.
- Insert toggles overwrite mode. In overwrite mode, printable characters replace the character under the cursor when possible and insert normally at line end.
- Ctrl-Z undoes edits since the last save.
- Ctrl-Y redoes the last undone edit.

## Search and navigation

- Ctrl-F starts a search prompt.
- Search is case-insensitive by default.
- Ctrl-Shift-F toggles exact-case search.
- Accepted matches are highlighted.
- Up and Down recall the last ten accepted search queries while the search prompt is active.
- F3 repeats the last search forward.
- Shift-F3 repeats the last search backward.
- Ctrl-G opens the go-to-line prompt.
- Go menu entries can jump by page, document edge, or word.

## Tabs

- File > New file or Ctrl-N creates a new untitled file tab. The file is not written until Save.
- Ctrl-PageUp and Ctrl-PageDown switch documents.
- Ctrl-F4 closes the active tab.
- Closing a modified tab requires confirmation.
- Ctrl-B opens the file sidebar; Enter opens or focuses the selected sidebar file as a tab.
- Ctrl-Enter also opens the selected sidebar file as a tab when the terminal delivers it.
- The tab strip appears when more than one tab is open, and visible tab labels can be clicked with the mouse.

## File sidebar

- Ctrl-B toggles the file sidebar. Reopening it returns to the last visited sidebar directory in this session.
- Up and Down move the sidebar selection.
- Enter opens or focuses a selected file as a tab, or navigates into a directory.
- Ctrl-Enter also opens or focuses the selected file as a tab when the terminal delivers it.
- Ctrl-N creates a child file in the selected directory or current sidebar directory.
- Ctrl-D creates a child directory in the selected directory or current sidebar directory.
- Delete starts a typed yes confirmation for removal.
- Directory deletion warns because nested files and directories are removed too.
- Symlinks are not opened or deleted by the default file actions.
- Escape cancels sidebar prompts or closes the sidebar.

## Workspaces

- F10 -> Workspace saves, lists, opens, deletes, and restores workspace projects.
- Workspace projects store normal file paths and active-tab selection.
- Save named, Open project, and Delete project prompts support Up and Down to cycle saved project names.
- Delete project requires typing `yes` before the project snapshot is removed.
- Opening a project into a dirty session requires typed confirmation.
- Restore last workspace uses the existing persisted preference and opens the saved TUI current workspace on argument-free interactive launch.
- TUI workspace projects live under the `workspaces/tui` config subdirectory, separate from GUI tile workspaces, while using the same path-only project format.
- TUI workspace restore ignores GUI tile geometry and opens the project files as terminal tabs.

## Reader mode

- Ctrl-R toggles reader mode.
- View -> Reader mode also toggles it.
- View -> Reader slower and View -> Reader faster adjust the persisted lines-per-minute speed.
- Reader mode scrolls the active document downward without moving the edit cursor.
- Reader mode stops at the end of the document, when you edit, when you switch tabs, when you open a file, or when you open a workspace project.

## Themes and syntax

- Ctrl-T cycles the terminal chrome theme.
- Ctrl-Shift-T cycles the syntax highlighting theme independently.
- Ctrl-L toggles line numbers.
- Ctrl-W toggles word wrap.
- Syntax highlighting is extension-based with a plain-text fallback.

## Managed notes

- `kfnotepad --note TITLE` creates or opens a managed Markdown note under the local XDG data directory.
- `kfnotepad --notes` lists managed note filenames.
- Managed notes are normal Markdown files. They are not stored in a database.

## Safety and limits

- kfnotepad opens local UTF-8 text files only.
- Directories, symlinks, missing files, non-regular files, non-UTF-8 files, and files above the configured size limit are rejected by the safe open path.
- Save uses the same local atomic file adapter as the GUI. Existing save targets must be regular files.
- If a file changed on disk since open or last save, kfnotepad refuses to overwrite it silently.
- Saved text is normalized to LF line endings.
- The editor does not use network services, user accounts, or background sync.

## Troubleshooting

- If a desktop, terminal, multiplexer, or shell intercepts a shortcut before kfnotepad receives it, use F2 and run the command by name.
- If Ctrl-Left, Ctrl-Right, Ctrl-Backspace, or Ctrl-Delete do not work, confirm the terminal supports modified key reporting.
- If colors or icons look wrong, check the selected theme and terminal font.
- If workspace restore opens unexpected files, disable Restore last workspace from F10 -> Workspace, then save a fresh current workspace after opening the intended tabs.
"#
}

pub fn summarize_text(text: &str) -> FileSummary {
    FileSummary {
        bytes: text.len() as u64,
        lines: text.lines().count(),
        trailing_newline: text.ends_with('\n'),
    }
}

pub fn summarize_path(path: &Path) -> Result<FileSummary, String> {
    let text = read_text_file(path).map_err(|error| error.to_string())?;
    Ok(summarize_text(&text))
}

pub fn open_text_file(path: &Path) -> Result<TextDocument, OpenError> {
    let (text, snapshot) = read_text_file_with_snapshot(path)?;
    let mut buffer = TextBuffer::from_text(&text);
    buffer.set_file_snapshot(Some(snapshot));
    Ok(TextDocument {
        path: path.to_path_buf(),
        buffer,
    })
}

pub fn save_text_document(document: &mut TextDocument) -> Result<(), SaveError> {
    save_text_buffer_for_document(&document.path, &mut document.buffer)?;
    document.buffer.mark_clean();
    Ok(())
}

pub fn save_text_buffer(path: &Path, buffer: &TextBuffer) -> Result<(), SaveError> {
    save_text_buffer_inner(path, buffer, None).map(|_| ())
}

fn save_text_buffer_for_document(path: &Path, buffer: &mut TextBuffer) -> Result<(), SaveError> {
    let expected_snapshot = buffer.file_snapshot().cloned();
    let snapshot = save_text_buffer_inner(path, buffer, expected_snapshot.as_ref())?;
    buffer.set_file_snapshot(Some(snapshot));
    Ok(())
}

fn save_text_buffer_inner(
    path: &Path,
    buffer: &TextBuffer,
    expected_snapshot: Option<&FileSnapshot>,
) -> Result<FileSnapshot, SaveError> {
    let text = buffer.to_text();
    if text.len() as u64 > MAX_TEXT_FILE_BYTES {
        return Err(SaveError::TooLarge {
            path: path.to_path_buf(),
            bytes: text.len() as u64,
            limit: MAX_TEXT_FILE_BYTES,
        });
    }

    let existing_permissions = validate_save_target(path)?;
    if let Some(expected_snapshot) = expected_snapshot {
        match file_snapshot(path) {
            Ok(current_snapshot) if current_snapshot != *expected_snapshot => {
                return Err(SaveError::ExternalModification {
                    path: path.to_path_buf(),
                });
            }
            Ok(_) => {}
            Err(error) if error.kind() == io::ErrorKind::NotFound => {
                return Err(SaveError::ExternalModification {
                    path: path.to_path_buf(),
                });
            }
            Err(source) => {
                return Err(SaveError::Metadata {
                    path: path.to_path_buf(),
                    source,
                });
            }
        }
    }

    let temp_path = temporary_sibling_path(path);
    let save_result =
        write_temp_then_rename(path, &temp_path, text.as_bytes(), existing_permissions);

    if save_result.is_err() {
        let _ = fs::remove_file(&temp_path);
    }

    save_result?;
    file_snapshot(path).map_err(|source| SaveError::Metadata {
        path: path.to_path_buf(),
        source,
    })
}

fn read_text_file(path: &Path) -> Result<String, OpenError> {
    read_text_file_with_snapshot(path).map(|(text, _snapshot)| text)
}

fn read_text_file_with_snapshot(path: &Path) -> Result<(String, FileSnapshot), OpenError> {
    let metadata = fs::symlink_metadata(path).map_err(|source| OpenError::Access {
        path: path.to_path_buf(),
        source,
    })?;

    if metadata.file_type().is_symlink() {
        return Err(OpenError::Symlink {
            path: path.to_path_buf(),
        });
    }

    if metadata.is_dir() {
        return Err(OpenError::Directory {
            path: path.to_path_buf(),
        });
    }

    if !metadata.file_type().is_file() {
        return Err(OpenError::NotRegular {
            path: path.to_path_buf(),
        });
    }

    if metadata.len() > MAX_TEXT_FILE_BYTES {
        return Err(OpenError::TooLarge {
            path: path.to_path_buf(),
            bytes: metadata.len(),
            limit: MAX_TEXT_FILE_BYTES,
        });
    }

    fs::read_to_string(path)
        .map_err(|source| {
            if source.kind() == io::ErrorKind::InvalidData {
                OpenError::ReadUtf8 {
                    path: path.to_path_buf(),
                    source,
                }
            } else {
                OpenError::Access {
                    path: path.to_path_buf(),
                    source,
                }
            }
        })
        .map(|text| {
            let snapshot = FileSnapshot {
                bytes: metadata.len(),
                modified: metadata.modified().ok(),
                fingerprint: fingerprint_bytes(text.as_bytes()),
            };
            (text, snapshot)
        })
}

fn validate_save_target(path: &Path) -> Result<Option<fs::Permissions>, SaveError> {
    match fs::symlink_metadata(path) {
        Ok(metadata) if metadata.file_type().is_symlink() => Err(SaveError::Symlink {
            path: path.to_path_buf(),
        }),
        Ok(metadata) if metadata.is_dir() => Err(SaveError::Directory {
            path: path.to_path_buf(),
        }),
        Ok(metadata) if !metadata.file_type().is_file() => Err(SaveError::NotRegular {
            path: path.to_path_buf(),
        }),
        Ok(metadata) => Ok(Some(metadata.permissions())),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(None),
        Err(source) => Err(SaveError::Metadata {
            path: path.to_path_buf(),
            source,
        }),
    }
}

fn file_snapshot(path: &Path) -> io::Result<FileSnapshot> {
    let metadata = fs::symlink_metadata(path)?;
    let bytes = fs::read(path)?;
    Ok(FileSnapshot {
        bytes: metadata.len(),
        modified: metadata.modified().ok(),
        fingerprint: fingerprint_bytes(&bytes),
    })
}

fn fingerprint_bytes(bytes: &[u8]) -> u64 {
    let mut hash = 0xcbf29ce484222325u64;
    for byte in bytes {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    hash
}

fn write_temp_then_rename(
    target_path: &Path,
    temp_path: &Path,
    bytes: &[u8],
    permissions: Option<fs::Permissions>,
) -> Result<(), SaveError> {
    let mut options = OpenOptions::new();
    options.write(true).create_new(true);

    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }

    let mut file = options
        .open(temp_path)
        .map_err(|source| SaveError::CreateTemp {
            path: temp_path.to_path_buf(),
            source,
        })?;

    if let Some(permissions) = permissions {
        file.set_permissions(permissions)
            .map_err(|source| SaveError::CreateTemp {
                path: temp_path.to_path_buf(),
                source,
            })?;
    }

    file.write_all(bytes)
        .map_err(|source| SaveError::WriteTemp {
            path: temp_path.to_path_buf(),
            source,
        })?;
    file.sync_all().map_err(|source| SaveError::FlushTemp {
        path: temp_path.to_path_buf(),
        source,
    })?;
    drop(file);

    fs::rename(temp_path, target_path).map_err(|source| SaveError::Rename {
        from: temp_path.to_path_buf(),
        to: target_path.to_path_buf(),
        source,
    })
}

fn temporary_sibling_path(path: &Path) -> PathBuf {
    let file_name = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("untitled");
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |duration| duration.as_nanos());
    let temp_name = format!(".{file_name}.kfnotepad-{}-{nonce}.tmp", std::process::id());
    path.with_file_name(temp_name)
}

fn byte_index_for_char_column(line: &str, column: usize) -> Result<usize, BufferError> {
    let columns = line.chars().count();
    if column > columns {
        return Err(BufferError::ColumnOutOfBounds { column, columns });
    }

    Ok(line
        .char_indices()
        .nth(column)
        .map_or(line.len(), |(index, _)| index))
}

fn find_in_line_with_mode(
    line: &str,
    query: &str,
    start_column: usize,
    row: usize,
    mode: SearchMode,
) -> Option<Cursor> {
    if !mode.case_sensitive {
        return find_in_line_case_insensitive(line, query, start_column, row);
    }
    let start_byte = byte_index_for_char_column(line, start_column).ok()?;
    let match_byte = line.get(start_byte..)?.find(query)? + start_byte;
    Some(Cursor {
        row,
        column: line[..match_byte].chars().count(),
    })
}

fn find_last_in_line_before_with_mode(
    line: &str,
    query: &str,
    end_column: usize,
    row: usize,
    mode: SearchMode,
) -> Option<Cursor> {
    if !mode.case_sensitive {
        return find_last_in_line_case_insensitive(line, query, end_column, row);
    }
    let end_byte = byte_index_for_char_column(line, end_column).ok()?;
    let match_byte = line.get(..end_byte)?.rfind(query)?;
    Some(Cursor {
        row,
        column: line[..match_byte].chars().count(),
    })
}

fn find_in_line_case_insensitive(
    line: &str,
    query: &str,
    start_column: usize,
    row: usize,
) -> Option<Cursor> {
    let start_byte = byte_index_for_char_column(line, start_column).ok()?;
    let lower_tail = line.get(start_byte..)?.to_lowercase();
    let lower_query = query.to_lowercase();
    let match_byte = lower_tail.find(&lower_query)?;
    Some(Cursor {
        row,
        column: line[..start_byte].chars().count() + lower_tail[..match_byte].chars().count(),
    })
}

fn find_last_in_line_case_insensitive(
    line: &str,
    query: &str,
    end_column: usize,
    row: usize,
) -> Option<Cursor> {
    let end_byte = byte_index_for_char_column(line, end_column).ok()?;
    let lower_prefix = line.get(..end_byte)?.to_lowercase();
    let lower_query = query.to_lowercase();
    let match_byte = lower_prefix.rfind(&lower_query)?;
    Some(Cursor {
        row,
        column: lower_prefix[..match_byte].chars().count(),
    })
}

fn is_word_character(character: char) -> bool {
    character == '_' || character.is_alphanumeric()
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TempArea {
        root: PathBuf,
    }

    impl TempArea {
        fn new(label: &str) -> Self {
            let root = std::env::temp_dir().join(format!(
                "kfnotepad-lib-{label}-{}-{}",
                std::process::id(),
                SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .expect("system time")
                    .as_nanos()
            ));
            fs::create_dir_all(&root).expect("create temp area");
            Self { root }
        }

        fn path(&self, name: &str) -> PathBuf {
            self.root.join(name)
        }
    }

    impl Drop for TempArea {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.root);
        }
    }

    fn assert_no_temp_files(directory: &Path) {
        let entries = fs::read_dir(directory).expect("read temp dir");
        for entry in entries {
            let entry = entry.expect("read temp entry");
            let file_name = entry.file_name();
            let file_name = file_name.to_string_lossy();
            assert!(
                !file_name.contains(".kfnotepad-"),
                "unexpected temporary config file left behind: {file_name}"
            );
        }
    }

    #[test]
    fn parses_help_flag() {
        assert_eq!(parse_args(&["--help".to_string()]), Ok(Command::Help));
    }

    #[test]
    fn parses_managed_notes_flags() {
        assert_eq!(
            parse_args(&["--notes".to_string()]),
            Ok(Command::ListManagedNotes)
        );
        assert_eq!(
            parse_args(&["--note".to_string(), "Daily Note".to_string()]),
            Ok(Command::OpenManagedNote("Daily Note".to_string()))
        );
        assert_eq!(
            parse_args(&["--note".to_string(), "   ".to_string()]),
            Err("managed note title must not be empty".to_string())
        );
    }

    #[test]
    fn rejects_unknown_option() {
        assert_eq!(
            parse_args(&["--bogus".to_string()]),
            Err("unknown option: --bogus".to_string())
        );
    }

    #[test]
    fn summarizes_text_without_mutation() {
        assert_eq!(
            summarize_text("one\ntwo\n"),
            FileSummary {
                bytes: 8,
                lines: 2,
                trailing_newline: true
            }
        );
    }

    #[test]
    fn buffer_starts_clean_and_preserves_lines() {
        let buffer = TextBuffer::from_text("alpha\nbeta\n");

        assert_eq!(buffer.lines(), &["alpha".to_string(), "beta".to_string()]);
        assert_eq!(buffer.line_count(), 2);
        assert!(buffer.has_trailing_newline());
        assert!(!buffer.is_dirty());
    }

    #[test]
    fn buffer_inserts_unicode_by_character_column() {
        let mut buffer = TextBuffer::from_text("aé\n");

        buffer.insert_char(0, 2, '!').expect("insert char");

        assert_eq!(buffer.lines(), &["aé!".to_string()]);
        assert!(buffer.is_dirty());
    }

    #[test]
    fn buffer_splits_line_without_losing_text() {
        let mut buffer = TextBuffer::from_text("abcdef");

        buffer.insert_newline(0, 3).expect("insert newline");

        assert_eq!(buffer.lines(), &["abc".to_string(), "def".to_string()]);
        assert!(!buffer.has_trailing_newline());
        assert!(buffer.is_dirty());
    }

    #[test]
    fn buffer_replace_text_marks_dirty_only_when_changed() {
        let mut buffer = TextBuffer::from_text("alpha\n");

        buffer.replace_text("alpha\n");
        assert_eq!(buffer.to_text(), "alpha\n");
        assert!(!buffer.is_dirty());

        buffer.replace_text("beta");
        assert_eq!(buffer.lines(), &["beta".to_string()]);
        assert_eq!(buffer.to_text(), "beta");
        assert!(buffer.is_dirty());

        buffer.mark_clean();
        buffer.replace_text("beta\n");
        assert_eq!(buffer.to_text(), "beta\n");
        assert!(buffer.has_trailing_newline());
        assert!(buffer.is_dirty());
    }

    #[test]
    fn buffer_rejects_out_of_range_position() {
        let mut buffer = TextBuffer::from_text("abc");

        assert_eq!(
            buffer.insert_char(0, 4, '!'),
            Err(BufferError::ColumnOutOfBounds {
                column: 4,
                columns: 3
            })
        );
    }

    #[test]
    fn cursor_moves_horizontally_by_character_column() {
        let buffer = TextBuffer::from_text("aé\nz");

        assert_eq!(
            buffer.move_cursor(Cursor { row: 0, column: 1 }, CursorMove::Right),
            Ok(Cursor { row: 0, column: 2 })
        );
        assert_eq!(
            buffer.move_cursor(Cursor { row: 0, column: 2 }, CursorMove::Right),
            Ok(Cursor { row: 1, column: 0 })
        );
        assert_eq!(
            buffer.move_cursor(Cursor { row: 1, column: 0 }, CursorMove::Left),
            Ok(Cursor { row: 0, column: 2 })
        );
    }

    #[test]
    fn cursor_moves_right_by_word_boundaries() {
        let buffer = TextBuffer::from_text("alpha, beta_gamma 42\n");

        assert_eq!(
            buffer.move_cursor(Cursor { row: 0, column: 0 }, CursorMove::WordRight),
            Ok(Cursor { row: 0, column: 7 })
        );
        assert_eq!(
            buffer.move_cursor(Cursor { row: 0, column: 7 }, CursorMove::WordRight),
            Ok(Cursor { row: 0, column: 18 })
        );
    }

    #[test]
    fn cursor_moves_left_by_word_boundaries() {
        let buffer = TextBuffer::from_text("alpha, beta gamma\n");

        assert_eq!(
            buffer.move_cursor(Cursor { row: 0, column: 17 }, CursorMove::WordLeft),
            Ok(Cursor { row: 0, column: 12 })
        );
        assert_eq!(
            buffer.move_cursor(Cursor { row: 0, column: 12 }, CursorMove::WordLeft),
            Ok(Cursor { row: 0, column: 7 })
        );
    }

    #[test]
    fn cursor_word_movement_crosses_lines_and_handles_unicode() {
        let buffer = TextBuffer::from_text("héllo\n  世界 next\n");

        assert_eq!(
            buffer.move_cursor(Cursor { row: 0, column: 0 }, CursorMove::WordRight),
            Ok(Cursor { row: 1, column: 2 })
        );
        assert_eq!(
            buffer.move_cursor(Cursor { row: 1, column: 5 }, CursorMove::WordLeft),
            Ok(Cursor { row: 1, column: 2 })
        );
        assert_eq!(
            buffer.move_cursor(Cursor { row: 1, column: 2 }, CursorMove::WordLeft),
            Ok(Cursor { row: 0, column: 0 })
        );
    }

    #[test]
    fn cursor_word_movement_clamps_at_document_edges() {
        let buffer = TextBuffer::from_text("alpha\n");

        assert_eq!(
            buffer.move_cursor(Cursor { row: 0, column: 0 }, CursorMove::WordLeft),
            Ok(Cursor { row: 0, column: 0 })
        );
        assert_eq!(
            buffer.move_cursor(Cursor { row: 0, column: 5 }, CursorMove::WordRight),
            Ok(Cursor { row: 0, column: 5 })
        );
    }

    #[test]
    fn buffer_deletes_previous_word_by_boundaries() {
        let mut buffer = TextBuffer::from_text("alpha, beta gamma\n");

        let moved = buffer
            .delete_previous_word(Cursor { row: 0, column: 17 })
            .expect("delete previous word");

        assert_eq!(moved, Cursor { row: 0, column: 12 });
        assert_eq!(buffer.line(0), Some("alpha, beta "));
        assert!(buffer.is_dirty());
        assert!(buffer.undo_last_edit());
        assert_eq!(buffer.line(0), Some("alpha, beta gamma"));
    }

    #[test]
    fn buffer_deletes_next_word_by_boundaries() {
        let mut buffer = TextBuffer::from_text("alpha, beta gamma\n");

        let moved = buffer
            .delete_next_word(Cursor { row: 0, column: 5 })
            .expect("delete next word");

        assert_eq!(moved, Cursor { row: 0, column: 5 });
        assert_eq!(buffer.line(0), Some("alpha gamma"));
        assert!(buffer.is_dirty());
    }

    #[test]
    fn buffer_deletes_to_line_end_by_character_column() {
        let mut buffer = TextBuffer::from_text("héllo world\nnext\n");

        let moved = buffer
            .delete_to_line_end(Cursor { row: 0, column: 2 })
            .expect("delete to line end");

        assert_eq!(moved, Cursor { row: 0, column: 2 });
        assert_eq!(buffer.lines(), &["hé".to_string(), "next".to_string()]);
        assert!(buffer.is_dirty());
        assert!(buffer.undo_last_edit());
        assert_eq!(
            buffer.lines(),
            &["héllo world".to_string(), "next".to_string()]
        );
    }

    #[test]
    fn buffer_word_deletion_crosses_lines_and_handles_unicode() {
        let mut buffer = TextBuffer::from_text("héllo\n  世界 next\n");

        let moved = buffer
            .delete_next_word(Cursor { row: 0, column: 5 })
            .expect("delete next word across line");

        assert_eq!(moved, Cursor { row: 0, column: 5 });
        assert_eq!(buffer.lines(), &["héllo next".to_string()]);
        assert!(buffer.has_trailing_newline());
        assert!(buffer.undo_last_edit());
        assert_eq!(
            buffer.lines(),
            &["héllo".to_string(), "  世界 next".to_string()]
        );

        let moved = buffer
            .delete_previous_word(Cursor { row: 1, column: 5 })
            .expect("delete previous word across line");

        assert_eq!(moved, Cursor { row: 1, column: 2 });
        assert_eq!(buffer.lines(), &["héllo".to_string(), "  next".to_string()]);
    }

    #[test]
    fn cursor_stays_inside_buffer_edges() {
        let buffer = TextBuffer::from_text("abc");

        assert_eq!(
            buffer.move_cursor(Cursor { row: 0, column: 0 }, CursorMove::Left),
            Ok(Cursor { row: 0, column: 0 })
        );
        assert_eq!(
            buffer.move_cursor(Cursor { row: 0, column: 3 }, CursorMove::Right),
            Ok(Cursor { row: 0, column: 3 })
        );
        assert_eq!(
            buffer.move_cursor(Cursor { row: 0, column: 0 }, CursorMove::Up),
            Ok(Cursor { row: 0, column: 0 })
        );
        assert_eq!(
            buffer.move_cursor(Cursor { row: 0, column: 0 }, CursorMove::Down),
            Ok(Cursor { row: 0, column: 0 })
        );
    }

    #[test]
    fn cursor_vertical_movement_clamps_to_target_line_length() {
        let buffer = TextBuffer::from_text("abcd\né\nxyz");

        assert_eq!(
            buffer.move_cursor(Cursor { row: 0, column: 4 }, CursorMove::Down),
            Ok(Cursor { row: 1, column: 1 })
        );
        assert_eq!(
            buffer.move_cursor(Cursor { row: 1, column: 1 }, CursorMove::Down),
            Ok(Cursor { row: 2, column: 1 })
        );
        assert_eq!(
            buffer.move_cursor(Cursor { row: 2, column: 3 }, CursorMove::Up),
            Ok(Cursor { row: 1, column: 1 })
        );
    }

    #[test]
    fn cursor_rejects_invalid_start_position() {
        let buffer = TextBuffer::from_text("abc");

        assert_eq!(
            buffer.move_cursor(Cursor { row: 0, column: 4 }, CursorMove::Right),
            Err(BufferError::ColumnOutOfBounds {
                column: 4,
                columns: 3
            })
        );
    }

    #[test]
    fn delete_removes_character_at_cursor() {
        let mut buffer = TextBuffer::from_text("aé!");

        buffer.delete_char(0, 1).expect("delete char");

        assert_eq!(buffer.lines(), &["a!".to_string()]);
        assert!(buffer.is_dirty());
    }

    #[test]
    fn delete_at_line_end_joins_next_line() {
        let mut buffer = TextBuffer::from_text("abc\ndef");

        buffer.delete_char(0, 3).expect("delete newline");

        assert_eq!(buffer.lines(), &["abcdef".to_string()]);
        assert!(buffer.is_dirty());
    }

    #[test]
    fn replace_char_replaces_one_character_and_undoes_as_one_edit() {
        let mut buffer = TextBuffer::from_text("aé!");

        buffer.replace_char(0, 1, 'x').expect("replace char");

        assert_eq!(buffer.lines(), &["ax!".to_string()]);
        assert!(buffer.is_dirty());
        assert!(buffer.undo_last_edit());
        assert_eq!(buffer.lines(), &["aé!".to_string()]);
        assert!(!buffer.undo_last_edit());
    }

    #[test]
    fn replace_char_at_line_end_inserts_without_deleting_newline() {
        let mut buffer = TextBuffer::from_text("abc\ndef");

        buffer.replace_char(0, 3, '!').expect("insert at line end");

        assert_eq!(buffer.lines(), &["abc!".to_string(), "def".to_string()]);
    }

    #[test]
    fn backspace_removes_character_before_cursor() {
        let mut buffer = TextBuffer::from_text("abc");

        let cursor = buffer
            .delete_before_cursor(Cursor { row: 0, column: 2 })
            .expect("backspace char");

        assert_eq!(cursor, Cursor { row: 0, column: 1 });
        assert_eq!(buffer.lines(), &["ac".to_string()]);
        assert!(buffer.is_dirty());
    }

    #[test]
    fn backspace_at_line_start_joins_previous_line() {
        let mut buffer = TextBuffer::from_text("abc\ndef");

        let cursor = buffer
            .delete_before_cursor(Cursor { row: 1, column: 0 })
            .expect("backspace newline");

        assert_eq!(cursor, Cursor { row: 0, column: 3 });
        assert_eq!(buffer.lines(), &["abcdef".to_string()]);
        assert!(buffer.is_dirty());
    }

    #[test]
    fn undo_restores_previous_text_after_insert() {
        let mut buffer = TextBuffer::from_text("abc");

        buffer.insert_char(0, 1, '!').expect("insert char");
        assert_eq!(buffer.lines(), &["a!bc".to_string()]);

        assert!(buffer.undo_last_edit());
        assert_eq!(buffer.lines(), &["abc".to_string()]);
        assert!(buffer.is_dirty());
    }

    #[test]
    fn redo_restores_undone_text() {
        let mut buffer = TextBuffer::from_text("abc");

        buffer.insert_char(0, 3, '!').expect("insert char");
        assert_eq!(buffer.lines(), &["abc!".to_string()]);

        assert!(buffer.undo_last_edit());
        assert_eq!(buffer.lines(), &["abc".to_string()]);

        assert!(buffer.redo_last_undo());
        assert_eq!(buffer.lines(), &["abc!".to_string()]);
        assert!(buffer.is_dirty());
    }

    #[test]
    fn new_edit_clears_redo_history() {
        let mut buffer = TextBuffer::from_text("abc");

        buffer.insert_char(0, 3, '!').expect("insert char");
        assert!(buffer.undo_last_edit());
        buffer
            .insert_char(0, 3, '?')
            .expect("insert replacement char");

        assert!(!buffer.redo_last_undo());
        assert_eq!(buffer.lines(), &["abc?".to_string()]);
    }

    #[test]
    fn undo_restores_previous_text_after_line_join() {
        let mut buffer = TextBuffer::from_text("abc\ndef");

        buffer.delete_char(0, 3).expect("delete newline");
        assert_eq!(buffer.lines(), &["abcdef".to_string()]);

        assert!(buffer.undo_last_edit());
        assert_eq!(buffer.lines(), &["abc".to_string(), "def".to_string()]);
    }

    #[test]
    fn mark_clean_clears_undo_history() {
        let mut buffer = TextBuffer::from_text("abc");

        buffer.insert_char(0, 3, '!').expect("insert char");
        buffer.mark_clean();

        assert!(!buffer.undo_last_edit());
        assert!(!buffer.redo_last_undo());
        assert_eq!(buffer.lines(), &["abc!".to_string()]);
        assert!(!buffer.is_dirty());
    }

    #[test]
    fn find_next_finds_query_from_cursor() {
        let buffer = TextBuffer::from_text("alpha\nbeta alphabet\n");

        assert_eq!(
            buffer.find_next("alpha", Cursor { row: 0, column: 1 }),
            Some(Cursor { row: 1, column: 5 })
        );
    }

    #[test]
    fn find_next_wraps_to_top() {
        let buffer = TextBuffer::from_text("first match\nsecond\n");

        assert_eq!(
            buffer.find_next("first", Cursor { row: 1, column: 0 }),
            Some(Cursor { row: 0, column: 0 })
        );
    }

    #[test]
    fn find_next_handles_unicode_columns() {
        let buffer = TextBuffer::from_text("aé match\n");

        assert_eq!(
            buffer.find_next("match", Cursor { row: 0, column: 0 }),
            Some(Cursor { row: 0, column: 3 })
        );
    }

    #[test]
    fn find_next_can_ignore_case_without_changing_default_search() {
        let buffer = TextBuffer::from_text("Alpha\nbeta alpha\n");

        assert_eq!(
            buffer.find_next("alpha", Cursor { row: 0, column: 0 }),
            Some(Cursor { row: 1, column: 5 })
        );
        assert_eq!(
            buffer.find_next_with_mode(
                "alpha",
                Cursor { row: 0, column: 0 },
                SearchMode {
                    case_sensitive: false,
                },
            ),
            Some(Cursor { row: 0, column: 0 })
        );
    }

    #[test]
    fn find_previous_finds_query_before_cursor() {
        let buffer = TextBuffer::from_text("alpha\nbeta alpha\n");

        assert_eq!(
            buffer.find_previous("alpha", Cursor { row: 1, column: 10 }),
            Some(Cursor { row: 1, column: 5 })
        );
    }

    #[test]
    fn find_previous_wraps_to_bottom() {
        let buffer = TextBuffer::from_text("first\nsecond match\n");

        assert_eq!(
            buffer.find_previous("match", Cursor { row: 0, column: 0 }),
            Some(Cursor { row: 1, column: 7 })
        );
    }

    #[test]
    fn find_previous_handles_unicode_columns() {
        let buffer = TextBuffer::from_text("aé match\n");

        assert_eq!(
            buffer.find_previous("match", Cursor { row: 0, column: 8 }),
            Some(Cursor { row: 0, column: 3 })
        );
    }

    #[test]
    fn repeat_search_with_mode_wraps_and_honors_case() {
        let document = TextDocument {
            path: PathBuf::from("note.txt"),
            buffer: TextBuffer::from_text("Alpha\nbeta\n"),
        };
        let mut cursor = Cursor { row: 1, column: 4 };

        assert_eq!(
            repeat_search_next_with_mode(
                &document,
                &mut cursor,
                "alpha",
                SearchMode {
                    case_sensitive: false,
                },
            ),
            SearchRepeatResult::Found {
                query: "alpha".to_string()
            }
        );
        assert_eq!(cursor, Cursor { row: 0, column: 0 });
    }

    #[test]
    fn editor_tab_state_starts_at_document_origin() {
        assert_eq!(
            EditorTabState::default(),
            EditorTabState {
                cursor: Cursor { row: 0, column: 0 },
                viewport_start: 0,
                horizontal_offset: 0,
            }
        );
    }

    #[test]
    fn editor_workspace_keeps_borrowed_document_editable() {
        let mut document = TextDocument {
            path: PathBuf::from("note.txt"),
            buffer: TextBuffer::from_text("alpha\n"),
        };

        {
            let mut workspace = EditorWorkspace::from_document(&mut document);
            let active_tab = workspace.active_tab_mut();
            active_tab.state.cursor = Cursor { row: 0, column: 5 };
            active_tab
                .document
                .as_mut()
                .buffer
                .insert_char(0, 5, '!')
                .expect("insert through active tab");

            assert_eq!(active_tab.state.cursor, Cursor { row: 0, column: 5 });
            assert!(active_tab.document.as_ref().buffer.is_dirty());
        }

        assert_eq!(document.buffer.to_text(), "alpha!\n");
        assert!(document.buffer.is_dirty());
    }

    #[test]
    fn editor_workspace_switches_and_closes_tabs_without_terminal_types() {
        let mut first = TextDocument {
            path: PathBuf::from("first.txt"),
            buffer: TextBuffer::from_text("one\n"),
        };
        let second = TextDocument {
            path: PathBuf::from("second.txt"),
            buffer: TextBuffer::from_text("two\n"),
        };
        let mut workspace = EditorWorkspace::from_document(&mut first);

        assert!(!workspace.select_next_tab());
        workspace.push_owned_tab(second);
        assert_eq!(workspace.active, 1);
        assert_eq!(
            workspace.tab_strip_items(),
            vec![
                TabStripItem {
                    label: String::from("first.txt"),
                    active: false,
                    dirty: false,
                },
                TabStripItem {
                    label: String::from("second.txt"),
                    active: true,
                    dirty: false,
                },
            ]
        );

        assert!(workspace.select_next_tab());
        assert_eq!(workspace.active, 0);
        assert!(workspace.select_previous_tab());
        assert_eq!(workspace.active, 1);

        assert_eq!(
            workspace.close_active_tab(false),
            CloseActiveTabResult::Closed {
                path: PathBuf::from("second.txt"),
            }
        );
        assert_eq!(workspace.tabs.len(), 1);
        assert_eq!(workspace.active, 0);
    }

    #[test]
    fn editor_workspace_requires_confirmation_before_dirty_tab_close() {
        let mut first = TextDocument {
            path: PathBuf::from("first.txt"),
            buffer: TextBuffer::from_text("one\n"),
        };
        let mut second = TextDocument {
            path: PathBuf::from("second.txt"),
            buffer: TextBuffer::from_text("two\n"),
        };
        second
            .buffer
            .insert_char(0, 0, '!')
            .expect("dirty second tab");
        let mut workspace = EditorWorkspace::from_document(&mut first);
        workspace.push_owned_tab(second);

        assert_eq!(
            workspace.close_active_tab(false),
            CloseActiveTabResult::Dirty
        );
        assert_eq!(workspace.tabs.len(), 2);
        assert_eq!(
            workspace.close_active_tab(true),
            CloseActiveTabResult::Closed {
                path: PathBuf::from("second.txt"),
            }
        );
        assert_eq!(workspace.tabs.len(), 1);
    }

    #[test]
    fn shared_document_commands_edit_and_clamp_cursor_without_terminal_types() {
        let mut document = TextDocument {
            path: PathBuf::from("note.txt"),
            buffer: TextBuffer::from_text("alpha beta\nsecond\n"),
        };
        let mut cursor = Cursor { row: 0, column: 10 };

        assert_eq!(
            delete_previous_word(&mut document, &mut cursor),
            EditResult::Modified
        );
        assert_eq!(document.buffer.to_text(), "alpha \nsecond\n");
        assert_eq!(cursor, Cursor { row: 0, column: 6 });

        assert_eq!(
            undo_document_edit(&mut document, &mut cursor),
            UndoRedoResult::Applied
        );
        assert_eq!(document.buffer.to_text(), "alpha beta\nsecond\n");
        assert_eq!(cursor, Cursor { row: 0, column: 6 });

        assert_eq!(
            redo_document_edit(&mut document, &mut cursor),
            UndoRedoResult::Applied
        );
        assert_eq!(document.buffer.to_text(), "alpha \nsecond\n");
        assert_eq!(cursor, Cursor { row: 0, column: 6 });

        assert_eq!(
            delete_to_line_end(&mut document, &mut cursor),
            EditResult::Modified
        );
        assert_eq!(document.buffer.to_text(), "alpha \nsecond\n");
        cursor.column = 0;
        assert_eq!(
            delete_next_word(&mut document, &mut cursor),
            EditResult::Modified
        );
        assert_eq!(document.buffer.to_text(), " \nsecond\n");
    }

    #[test]
    fn shared_navigation_commands_move_without_terminal_types() {
        let document = TextDocument {
            path: PathBuf::from("note.txt"),
            buffer: TextBuffer::from_text("alpha beta\nsecond line\nthird\n"),
        };
        let mut cursor = Cursor { row: 0, column: 0 };

        move_document_cursor(&document, &mut cursor, CursorMove::WordRight);
        assert_eq!(cursor, Cursor { row: 0, column: 6 });

        page_down(&document, &mut cursor, 2);
        assert_eq!(cursor, Cursor { row: 2, column: 5 });

        go_to_document_end(&document, &mut cursor);
        assert_eq!(cursor, Cursor { row: 2, column: 5 });

        page_up(&document, &mut cursor, 10);
        assert_eq!(cursor, Cursor { row: 0, column: 5 });

        go_to_document_start(&mut cursor);
        assert_eq!(cursor, Cursor { row: 0, column: 0 });

        cursor = Cursor { row: 0, column: 99 };
        assert_eq!(
            go_to_line(&document, &mut cursor, "2"),
            GoToLineResult::Moved { line_number: 2 }
        );
        assert_eq!(cursor, Cursor { row: 1, column: 11 });
        assert_eq!(
            go_to_line(&document, &mut cursor, ""),
            GoToLineResult::Empty
        );
        assert_eq!(
            go_to_line(&document, &mut cursor, "abc"),
            GoToLineResult::Invalid
        );
        assert_eq!(
            go_to_line(&document, &mut cursor, "99"),
            GoToLineResult::OutOfRange { line_number: 99 }
        );
        assert_eq!(cursor, Cursor { row: 1, column: 11 });
    }

    #[test]
    fn shared_repeat_search_updates_cursor_and_reports_result() {
        let document = TextDocument {
            path: PathBuf::from("note.txt"),
            buffer: TextBuffer::from_text("alpha\nbeta alpha\n"),
        };
        let mut cursor = Cursor { row: 0, column: 0 };

        assert_eq!(
            repeat_search_next(&document, &mut cursor, ""),
            SearchRepeatResult::NoPreviousSearch
        );
        assert_eq!(
            repeat_search_next(&document, &mut cursor, "alpha"),
            SearchRepeatResult::Found {
                query: String::from("alpha"),
            }
        );
        assert_eq!(cursor, Cursor { row: 1, column: 5 });
        assert_eq!(
            repeat_search_previous(&document, &mut cursor, "alpha"),
            SearchRepeatResult::Found {
                query: String::from("alpha"),
            }
        );
        assert_eq!(cursor, Cursor { row: 0, column: 0 });
        assert_eq!(
            repeat_search_next(&document, &mut cursor, "missing"),
            SearchRepeatResult::NoMatch {
                query: String::from("missing"),
            }
        );
    }

    #[test]
    fn shared_editor_config_paths_parse_and_persist_without_terminal_types() {
        let temp = TempArea::new("editor-config");
        let xdg = temp.path("xdg");
        let home = temp.path("home");

        assert_eq!(
            editor_config_path(Some(xdg.as_path()), Some(home.as_path())),
            Some(xdg.join("kfnotepad").join("config.toml"))
        );
        assert_eq!(
            editor_config_path(None, Some(home.as_path())),
            Some(home.join(".config").join("kfnotepad").join("config.toml"))
        );
        assert!(editor_config_path(None, None).is_none());

        let settings = parse_editor_settings_config(
            r#"
theme = "terror"
syntax_theme = "abyss"
line_numbers = false
wrap = true
search_case_sensitive = true
gui_restore_last_workspace = true
gui_reader_mode_enabled = true
gui_reader_lines_per_minute = 180
gui_font_family = "fira-code"
gui_font_size = 20
gui_ui_font_size = 13
unknown = "ignored"
"#,
        );
        assert_eq!(
            settings,
            EditorSettings {
                show_line_numbers: false,
                theme_id: EditorThemeId::Terror,
                syntax_theme_id: EditorThemeId::Abyss,
                wrap_lines: true,
                search_case_sensitive: true,
                gui_restore_last_workspace: true,
                gui_reader_mode_enabled: true,
                gui_reader_lines_per_minute: 180,
                gui_font_family: GuiFontFamily::FiraCode,
                gui_font_size: 20,
                gui_ui_font_size: 13,
            }
        );

        let fallback = parse_editor_settings_config(
            r#"
theme = "not-a-theme"
line_numbers = maybe
wrap = "true"
gui_restore_last_workspace = yep
gui_font_family = "papyrus"
gui_font_size = 500
gui_ui_font_size = 500
"#,
        );
        assert_eq!(fallback, EditorSettings::default());

        let path = temp.path("config").join("kfnotepad").join("config.toml");
        save_editor_settings(
            &path,
            EditorSettings {
                show_line_numbers: false,
                theme_id: EditorThemeId::Abyss,
                wrap_lines: true,
                gui_restore_last_workspace: true,
                gui_font_family: GuiFontFamily::JetBrainsMono,
                gui_font_size: 18,
                gui_ui_font_size: 15,
                ..EditorSettings::default()
            },
        )
        .expect("save editor config");
        assert_eq!(
            fs::read_to_string(&path).expect("read config"),
            "theme = \"abyss\"\nsyntax_theme = \"nocturne\"\nline_numbers = false\nwrap = true\nsearch_case_sensitive = false\ngui_restore_last_workspace = true\ngui_reader_mode_enabled = false\ngui_reader_lines_per_minute = 60\ngui_font_family = \"jetbrains-mono\"\ngui_font_size = 18\ngui_ui_font_size = 15\n"
        );
        assert_no_temp_files(path.parent().expect("config parent"));
        assert_eq!(
            load_editor_settings(&path).expect("load config"),
            EditorSettings {
                show_line_numbers: false,
                theme_id: EditorThemeId::Abyss,
                wrap_lines: true,
                gui_restore_last_workspace: true,
                gui_font_family: GuiFontFamily::JetBrainsMono,
                gui_font_size: 18,
                gui_ui_font_size: 15,
                ..EditorSettings::default()
            }
        );

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;

            let file_mode = fs::metadata(&path)
                .expect("config metadata")
                .permissions()
                .mode()
                & 0o777;
            let dir_mode = fs::metadata(path.parent().expect("config parent"))
                .expect("config dir metadata")
                .permissions()
                .mode()
                & 0o777;
            assert_eq!(file_mode, 0o600);
            assert_eq!(dir_mode, 0o700);
        }
    }

    #[test]
    fn gui_layout_path_parse_and_serialize_round_trip_without_sensitive_paths() {
        let temp = TempArea::new("gui-layout");
        let xdg = temp.path("xdg");
        let home = temp.path("home");

        assert_eq!(
            gui_layout_path(Some(xdg.as_path()), Some(home.as_path())),
            Some(xdg.join("kfnotepad").join("gui-layout.v1"))
        );
        assert_eq!(
            gui_layout_path(None, Some(home.as_path())),
            Some(home.join(".config").join("kfnotepad").join("gui-layout.v1"))
        );
        assert!(gui_layout_path(None, None).is_none());

        let layout = GuiLayout {
            browser_visible: false,
            browser_width_px: Some(260),
            root: GuiLayoutNode::Split {
                axis: GuiLayoutAxis::Vertical,
                ratio_per_mille: 625,
                first: Box::new(GuiLayoutNode::Leaf { ordinal: 0 }),
                second: Box::new(GuiLayoutNode::Split {
                    axis: GuiLayoutAxis::Horizontal,
                    ratio_per_mille: 400,
                    first: Box::new(GuiLayoutNode::Leaf { ordinal: 1 }),
                    second: Box::new(GuiLayoutNode::Leaf { ordinal: 2 }),
                }),
            },
            minimized_ordinals: vec![1],
        };

        let text = serialize_gui_layout(&layout);

        assert_eq!(parse_gui_layout(&text, 3), Some(layout));
        assert!(text.contains("browser_width_px = 260"));
        assert!(!text.contains("note.txt"));
        assert!(!text.contains("/home"));
        assert!(!text.contains("search"));
        assert!(!text.contains("cursor"));
    }

    #[test]
    fn gui_layout_parser_falls_back_for_malformed_or_incompatible_input() {
        let valid = r#"
version = 1
browser_visible = true
root = 0
node.0 = split vertical 500 1 2
node.1 = leaf 0
node.2 = leaf 1
minimized =
"#;
        let old_without_width = r#"
version = 1
browser_visible = true
root = 0
node.0 = leaf 0
minimized =
"#;

        assert!(parse_gui_layout(valid, 2).is_some());
        assert_eq!(
            parse_gui_layout(old_without_width, 1)
                .expect("old layout without width should parse")
                .browser_width_px,
            None
        );
        assert!(parse_gui_layout("version = 2\nroot = 0\nnode.0 = leaf 0\n", 1).is_none());
        assert!(parse_gui_layout("version = 1\nroot = 0\nnode.0 = leaf x\n", 1).is_none());
        assert!(parse_gui_layout(
            "version = 1\nbrowser_width_px = nope\nroot = 0\nnode.0 = leaf 0\n",
            1
        )
        .is_none());
        assert!(parse_gui_layout(
            "version = 1\nbrowser_width_px = 0\nroot = 0\nnode.0 = leaf 0\n",
            1
        )
        .is_none());
        assert!(parse_gui_layout("version = 1\nroot = 0\nnode.0 = split vertical 0 1 2\nnode.1 = leaf 0\nnode.2 = leaf 1\n", 2).is_none());
        assert!(parse_gui_layout("version = 1\nroot = 0\nnode.0 = leaf 0\n", 2).is_none());
        assert!(parse_gui_layout("version = 1\nroot = 0\nnode.0 = split vertical 500 1 2\nnode.1 = leaf 0\nnode.2 = leaf 0\n", 2).is_none());
        assert!(parse_gui_layout(
            "version = 1\nroot = 0\nnode.0 = leaf 0\nminimized = 0,0\n",
            1
        )
        .is_none());
    }

    #[test]
    fn save_gui_layout_writes_atomic_private_layout_file() {
        let temp = TempArea::new("gui-layout-save");
        let path = temp.path("xdg").join("kfnotepad").join("gui-layout.v1");
        let layout = GuiLayout {
            browser_visible: true,
            browser_width_px: Some(240),
            root: GuiLayoutNode::Leaf { ordinal: 0 },
            minimized_ordinals: Vec::new(),
        };

        save_gui_layout(&path, &layout).expect("save gui layout");

        let text = fs::read_to_string(&path).expect("read gui layout");
        assert_eq!(parse_gui_layout(&text, 1), Some(layout));
        assert_no_temp_files(path.parent().expect("layout parent"));

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;

            let file_mode = fs::metadata(&path)
                .expect("layout metadata")
                .permissions()
                .mode()
                & 0o777;
            let dir_mode = fs::metadata(path.parent().expect("layout parent"))
                .expect("layout dir metadata")
                .permissions()
                .mode()
                & 0o777;
            assert_eq!(file_mode, 0o600);
            assert_eq!(dir_mode, 0o700);
        }
    }

    #[test]
    fn gui_workspace_project_path_and_round_trip_store_files_and_layout() {
        let temp = TempArea::new("gui-workspace-project");
        let xdg = temp.path("xdg");
        let home = temp.path("home");

        assert_eq!(
            gui_workspace_projects_dir(Some(xdg.as_path()), Some(home.as_path())),
            Some(xdg.join("kfnotepad").join("workspaces"))
        );
        assert_eq!(
            gui_workspace_projects_dir(None, Some(home.as_path())),
            Some(home.join(".config").join("kfnotepad").join("workspaces"))
        );
        assert!(gui_workspace_projects_dir(None, None).is_none());

        let project = GuiWorkspaceProject {
            name: "docs workspace".to_string(),
            files: vec![temp.path("README.md"), temp.path("docs/17-GUI-CONTRACT.md")],
            active_ordinal: 1,
            layout: Some(GuiLayout {
                browser_visible: false,
                browser_width_px: Some(220),
                root: GuiLayoutNode::Split {
                    axis: GuiLayoutAxis::Vertical,
                    ratio_per_mille: 550,
                    first: Box::new(GuiLayoutNode::Leaf { ordinal: 0 }),
                    second: Box::new(GuiLayoutNode::Leaf { ordinal: 1 }),
                },
                minimized_ordinals: vec![0],
            }),
        };

        let text = serialize_gui_workspace_project(&project).expect("serialize project");

        assert_eq!(parse_gui_workspace_project(&text), Some(project));
        assert!(text.contains("version = 1"));
        assert!(text.contains("file.0 = "));
        assert!(text.contains("layout.version = 1"));
        assert!(!text.contains(temp.root.to_string_lossy().as_ref()));
    }

    #[test]
    fn gui_left_panel_model_switches_between_files_workspaces_and_preferences() {
        let mut panel = GuiLeftPanelState::default();

        assert!(panel.visible);
        assert_eq!(panel.mode, GuiLeftPanelMode::Files);
        assert_eq!(panel.title(), "Files");

        panel.toggle_visibility();
        assert!(!panel.visible);
        assert_eq!(panel.mode, GuiLeftPanelMode::Files);

        panel.show_workspaces();
        assert!(panel.visible);
        assert_eq!(panel.mode, GuiLeftPanelMode::Workspaces);
        assert_eq!(panel.title(), "Workspaces");

        panel.show_preferences();
        assert!(panel.visible);
        assert_eq!(panel.mode, GuiLeftPanelMode::Preferences);
        assert_eq!(panel.title(), "Preferences");

        panel.toggle_visibility();
        assert!(!panel.visible);
        assert_eq!(panel.mode, GuiLeftPanelMode::Preferences);

        panel.show_files();
        assert!(panel.visible);
        assert_eq!(panel.mode, GuiLeftPanelMode::Files);

        panel.toggle_mode();
        assert!(panel.visible);
        assert_eq!(panel.mode, GuiLeftPanelMode::Workspaces);
        panel.toggle_mode();
        assert!(panel.visible);
        assert_eq!(panel.mode, GuiLeftPanelMode::Preferences);
        panel.toggle_mode();
        assert!(panel.visible);
        assert_eq!(panel.mode, GuiLeftPanelMode::Files);
    }

    #[test]
    fn gui_workspace_project_parser_rejects_invalid_snapshots() {
        let temp = TempArea::new("gui-workspace-project-invalid");
        let path_hex = path_to_hex(&temp.path("README.md"));
        let second_hex = path_to_hex(&temp.path("LICENSE"));

        assert!(parse_gui_workspace_project("version = 2\n").is_none());
        assert!(parse_gui_workspace_project("version = 1\nname_hex = zz\n").is_none());
        assert!(
            parse_gui_workspace_project("version = 1\nname_hex = 646f6373\nactive = 0\n").is_none()
        );
        assert!(parse_gui_workspace_project(&format!(
            "version = 1\nname_hex = 646f6373\nactive = 2\nfile.0 = {path_hex}\n"
        ))
        .is_none());
        assert!(parse_gui_workspace_project(&format!(
            "version = 1\nname_hex = 646f6373\nactive = 0\nfile.1 = {path_hex}\n"
        ))
        .is_none());
        assert!(parse_gui_workspace_project(&format!(
            "version = 1\nname_hex = 646f6373\nactive = 0\nfile.0 = {path_hex}\nfile.1 = {second_hex}\nlayout.version = 1\nlayout.root = 0\nlayout.node.0 = leaf 0\n"
        ))
        .is_none());
    }

    #[test]
    fn save_gui_workspace_project_writes_atomic_private_project_file() {
        let temp = TempArea::new("gui-workspace-project-save");
        let path = temp
            .path("xdg")
            .join("kfnotepad")
            .join("workspaces")
            .join("docs.v1");
        let project = GuiWorkspaceProject {
            name: "docs".to_string(),
            files: vec![temp.path("README.md")],
            active_ordinal: 0,
            layout: None,
        };

        save_gui_workspace_project(&path, &project).expect("save project");

        let text = fs::read_to_string(&path).expect("read project");
        assert_eq!(parse_gui_workspace_project(&text), Some(project));
        assert_no_temp_files(path.parent().expect("project parent"));

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;

            let file_mode = fs::metadata(&path)
                .expect("project metadata")
                .permissions()
                .mode()
                & 0o777;
            let dir_mode = fs::metadata(path.parent().expect("project parent"))
                .expect("project dir metadata")
                .permissions()
                .mode()
                & 0o777;
            assert_eq!(file_mode, 0o600);
            assert_eq!(dir_mode, 0o700);
        }
    }

    #[test]
    fn list_gui_workspace_projects_filters_and_sorts_valid_projects() {
        let temp = TempArea::new("gui-workspace-project-list");
        let projects_dir = temp.path("workspaces");
        fs::create_dir_all(&projects_dir).expect("create workspaces dir");
        let alpha = GuiWorkspaceProject {
            name: "alpha".to_string(),
            files: vec![temp.path("alpha.md")],
            active_ordinal: 0,
            layout: None,
        };
        let zeta = GuiWorkspaceProject {
            name: "zeta".to_string(),
            files: vec![temp.path("zeta.md")],
            active_ordinal: 0,
            layout: None,
        };

        save_gui_workspace_project(&projects_dir.join("zeta.v1"), &zeta).expect("save zeta");
        save_gui_workspace_project(&projects_dir.join("alpha.v1"), &alpha).expect("save alpha");
        fs::write(projects_dir.join("broken.v1"), "not a project").expect("write broken");
        fs::write(projects_dir.join("ignore.txt"), "ignored").expect("write ignored");
        fs::create_dir(projects_dir.join("folder.v1")).expect("create ignored dir");

        let projects = list_gui_workspace_projects(&projects_dir).expect("list projects");

        assert_eq!(
            projects
                .iter()
                .map(|entry| entry.project.name.as_str())
                .collect::<Vec<_>>(),
            vec!["alpha", "zeta"]
        );
        assert_eq!(projects[0].project, alpha);
        assert_eq!(projects[1].project, zeta);
        assert_eq!(
            gui_workspace_project_path(&projects_dir, "Daily Notes"),
            Some(projects_dir.join("daily-notes.v1"))
        );
        assert_eq!(gui_workspace_project_path(&projects_dir, "../bad"), None);
    }

    #[test]
    fn list_gui_workspace_projects_returns_empty_for_missing_directory() {
        let temp = TempArea::new("gui-workspace-project-list-missing");

        assert_eq!(
            list_gui_workspace_projects(&temp.path("missing")).expect("list missing"),
            Vec::new()
        );
    }

    #[test]
    fn shared_theme_ids_cycle_and_parse_without_terminal_types() {
        let mut theme_id = EditorThemeId::Nocturne;

        for expected in [
            EditorThemeId::Aurora,
            EditorThemeId::Paper,
            EditorThemeId::Terminal,
            EditorThemeId::Abyss,
            EditorThemeId::Terror,
            EditorThemeId::Nocturne,
        ] {
            theme_id = theme_id.next();
            assert_eq!(theme_id, expected);
            assert_eq!(EditorThemeId::from_label(theme_id.label()), Some(theme_id));
        }

        assert_eq!(
            EditorThemeId::from_label("paper"),
            Some(EditorThemeId::Paper)
        );
        assert_eq!(
            EditorThemeId::from_label("pastel"),
            Some(EditorThemeId::Paper)
        );
        assert_eq!(EditorThemeId::from_label("missing"), None);
    }

    #[test]
    fn shared_syntax_highlighter_detects_and_keeps_state_without_terminal_types() {
        let highlighter = SyntaxHighlighter::default();
        let rust_document = TextDocument {
            path: PathBuf::from("main.rs"),
            buffer: TextBuffer::from_text("/* start\ninside\n*/\nfn main() {}\n"),
        };
        let text_document = TextDocument {
            path: PathBuf::from("note.txt"),
            buffer: TextBuffer::from_text("plain note\n"),
        };

        assert_eq!(highlighter.syntax_name_for_document(&rust_document), "Rust");
        assert_eq!(highlighter.syntax_token_for_document(&rust_document), "rs");
        assert_eq!(
            highlighter.syntax_name_for_document(&text_document),
            "Plain Text"
        );
        assert_eq!(highlighter.syntax_token_for_document(&text_document), "txt");
        assert!(highlighter
            .highlight_line(&rust_document, "fn main() {}")
            .is_some());
        assert!(highlighter
            .highlight_line(&text_document, "plain note")
            .is_none());

        let stateful = highlighter.highlight_visible_lines(&rust_document, 1, 1);
        let reset = highlighter
            .highlight_line(&rust_document, "inside")
            .expect("standalone Rust line highlights");
        let stateful_line = stateful
            .first()
            .and_then(Option::as_ref)
            .expect("stateful Rust line highlights");

        assert_ne!(stateful_line[0].0.foreground, reset[0].0.foreground);
        assert_eq!(stateful_line[0].1, "inside");
    }

    #[test]
    fn shared_file_sidebar_lists_parent_dirs_and_files_in_order() {
        let temp = TempArea::new("sidebar-list");
        fs::create_dir(temp.path("z-dir")).expect("create z dir");
        fs::create_dir(temp.path("a-dir")).expect("create a dir");
        fs::write(temp.path("z.txt"), "z\n").expect("write z file");
        fs::write(temp.path("a.txt"), "a\n").expect("write a file");

        let sidebar = FileSidebarState::load(temp.root.clone()).expect("load sidebar");
        let labels: Vec<_> = sidebar
            .entries
            .iter()
            .map(|entry| entry.label.as_str())
            .collect();

        assert_eq!(labels, ["../", "a-dir/", "z-dir/", "a.txt", "z.txt"]);
        assert_eq!(sidebar.selected_entry().expect("selected").label, "../");
    }

    #[test]
    fn shared_file_sidebar_loads_subdirectories_and_parent_entries() {
        let temp = TempArea::new("sidebar-nav");
        fs::create_dir(temp.path("sub")).expect("create sub dir");
        fs::write(temp.path("sub").join("inside.txt"), "inside\n").expect("write sub file");

        let sidebar = FileSidebarState::load(temp.root.clone()).expect("load root sidebar");
        let sub = sidebar
            .entries
            .iter()
            .find(|entry| entry.label == "sub/")
            .expect("subdirectory entry")
            .clone();
        assert_eq!(sub.kind, FileSidebarEntryKind::Directory);

        let sub_sidebar = FileSidebarState::load(sub.path).expect("load sub sidebar");
        assert_eq!(
            sub_sidebar.current_dir,
            temp.path("sub")
                .canonicalize()
                .expect("canonicalize subdirectory")
        );
        assert_eq!(
            sub_sidebar.entries.first().expect("parent entry").kind,
            FileSidebarEntryKind::Parent
        );
    }

    #[test]
    fn shared_file_sidebar_selection_wraps_and_scrolls_without_terminal_types() {
        let mut sidebar = FileSidebarState {
            current_dir: PathBuf::from("."),
            entries: (0..5)
                .map(|index| FileSidebarEntry {
                    label: format!("file-{index}.txt"),
                    path: PathBuf::from(format!("file-{index}.txt")),
                    kind: FileSidebarEntryKind::File,
                })
                .collect(),
            selected: 0,
            scroll: 0,
        };

        sidebar.select_previous_wrapping(3);
        assert_eq!(sidebar.selected, 4);
        assert_eq!(sidebar.scroll, 2);

        sidebar.select_next_wrapping(3);
        assert_eq!(sidebar.selected, 0);
        assert_eq!(sidebar.scroll, 0);

        assert!(sidebar.scroll_selection_down(3));
        assert_eq!(sidebar.selected, 1);
        assert_eq!(sidebar.scroll, 0);
        assert!(sidebar.scroll_selection_down(3));
        assert!(sidebar.scroll_selection_down(3));
        assert_eq!(sidebar.selected, 3);
        assert_eq!(sidebar.scroll, 1);
        assert!(sidebar.scroll_selection_up(3));
        assert_eq!(sidebar.selected, 2);
        assert_eq!(sidebar.scroll, 1);

        sidebar.selected = 0;
        sidebar.scroll = 0;
        assert!(!sidebar.scroll_selection_up(3));
        assert_eq!(sidebar.selected, 0);
        assert_eq!(sidebar.scroll, 0);
    }

    #[test]
    fn shared_file_sidebar_mouse_row_selects_visible_entry() {
        let mut sidebar = FileSidebarState {
            current_dir: PathBuf::from("."),
            entries: (0..4)
                .map(|index| FileSidebarEntry {
                    label: format!("file-{index}.txt"),
                    path: PathBuf::from(format!("file-{index}.txt")),
                    kind: FileSidebarEntryKind::File,
                })
                .collect(),
            selected: 0,
            scroll: 1,
        };

        assert_eq!(sidebar.selected_entry_for_mouse_row(0), None);
        assert_eq!(
            sidebar
                .selected_entry_for_mouse_row(2)
                .expect("visible entry")
                .label,
            "file-2.txt"
        );
        assert_eq!(sidebar.selected, 2);
        assert_eq!(sidebar.selected_entry_for_mouse_row(5), None);
        assert_eq!(sidebar.selected, 2);
    }

    #[test]
    fn gui_workspace_opens_two_documents_as_focused_tiles() {
        let first = TextDocument {
            path: PathBuf::from("first.txt"),
            buffer: TextBuffer::from_text("one\n"),
        };
        let second = TextDocument {
            path: PathBuf::from("second.txt"),
            buffer: TextBuffer::from_text("two\n"),
        };
        let mut workspace = GuiWorkspace::from_document(first);

        assert_eq!(workspace.tiles.len(), 1);
        assert_eq!(workspace.active, GuiTileId(0));
        assert_eq!(workspace.focused, GuiTileId(0));
        assert_eq!(
            workspace.active_tile().document.path,
            PathBuf::from("first.txt")
        );

        let second_id = workspace.open_tile(second);

        assert_eq!(second_id, GuiTileId(1));
        assert_eq!(workspace.tiles.len(), 2);
        assert_eq!(workspace.active, second_id);
        assert_eq!(workspace.focused, second_id);
        assert_eq!(
            workspace.focused_tile().document.path,
            PathBuf::from("second.txt")
        );

        assert!(workspace.focus_tile(GuiTileId(0)));
        assert_eq!(
            workspace.active_tile().document.path,
            PathBuf::from("first.txt")
        );
        assert!(!workspace.focus_tile(GuiTileId(99)));
        assert_eq!(workspace.active, GuiTileId(0));
    }

    #[test]
    fn gui_workspace_blocks_invalid_open_without_mutation() {
        let first = TextDocument {
            path: PathBuf::from("first.txt"),
            buffer: TextBuffer::from_text("one\n"),
        };
        let mut workspace = GuiWorkspace::from_document(first);

        let result = workspace.open_validated_tile(Err(OpenError::Directory {
            path: PathBuf::from("dir"),
        }));

        assert!(matches!(
            result,
            Err(GuiTileOpenError::Invalid {
                source: OpenError::Directory { .. }
            })
        ));
        assert_eq!(workspace.tiles.len(), 1);
        assert_eq!(workspace.active, GuiTileId(0));
        assert_eq!(workspace.focused, GuiTileId(0));
    }

    #[test]
    fn gui_workspace_dirty_close_requires_confirmation() {
        let first = TextDocument {
            path: PathBuf::from("first.txt"),
            buffer: TextBuffer::from_text("one\n"),
        };
        let mut second = TextDocument {
            path: PathBuf::from("second.txt"),
            buffer: TextBuffer::from_text("two\n"),
        };
        second
            .buffer
            .insert_char(0, 0, '!')
            .expect("dirty second tile");
        let mut workspace = GuiWorkspace::from_document(first);
        let second_id = workspace.open_tile(second);

        assert_eq!(
            workspace.close_tile(second_id, false),
            GuiCloseTileResult::Dirty { tile_id: second_id }
        );
        assert_eq!(workspace.tiles.len(), 2);
        assert_eq!(
            workspace.close_tile(second_id, true),
            GuiCloseTileResult::Closed {
                tile_id: second_id,
                path: PathBuf::from("second.txt"),
            }
        );
        assert_eq!(workspace.tiles.len(), 1);
        assert_eq!(workspace.active, GuiTileId(0));
        assert_eq!(
            workspace.close_tile(GuiTileId(0), true),
            GuiCloseTileResult::OnlyTile
        );
    }

    #[test]
    fn gui_workspace_tracks_minimize_and_layout_intents() {
        let first = TextDocument {
            path: PathBuf::from("first.txt"),
            buffer: TextBuffer::from_text("one\n"),
        };
        let second = TextDocument {
            path: PathBuf::from("second.txt"),
            buffer: TextBuffer::from_text("two\n"),
        };
        let mut workspace = GuiWorkspace::from_document(first);
        let second_id = workspace.open_tile(second);

        assert!(workspace.set_tile_minimized(second_id, true));
        assert!(workspace.tile(second_id).expect("tile").minimized);
        assert!(workspace.set_tile_minimized(second_id, false));
        assert_eq!(workspace.focused, second_id);
        assert!(!workspace.set_tile_minimized(GuiTileId(99), true));

        assert!(workspace.request_split(second_id, GuiSplitDirection::Vertical));
        assert_eq!(
            workspace.pending_layout_intent,
            Some(GuiTileLayoutIntent::Split {
                tile_id: second_id,
                direction: GuiSplitDirection::Vertical,
            })
        );
        assert!(workspace.request_move(second_id, GuiTileMoveDirection::Left));
        assert_eq!(
            workspace.pending_layout_intent,
            Some(GuiTileLayoutIntent::Move {
                tile_id: second_id,
                direction: GuiTileMoveDirection::Left,
            })
        );
        assert!(workspace.request_resize(second_id, GuiTileResizeDirection::Wider));
        assert_eq!(
            workspace.pending_layout_intent,
            Some(GuiTileLayoutIntent::Resize {
                tile_id: second_id,
                direction: GuiTileResizeDirection::Wider,
            })
        );
        assert!(!workspace.request_split(GuiTileId(99), GuiSplitDirection::Horizontal));
        workspace.clear_layout_intent();
        assert_eq!(workspace.pending_layout_intent, None);
    }

    #[test]
    fn gui_workspace_reports_save_status_from_buffer_and_failures() {
        let first = TextDocument {
            path: PathBuf::from("first.txt"),
            buffer: TextBuffer::from_text("one\n"),
        };
        let mut workspace = GuiWorkspace::from_document(first);
        let tile_id = workspace.active;

        assert_eq!(
            workspace.active_tile().save_status(),
            GuiTileSaveStatus::Saved
        );
        workspace
            .active_tile_mut()
            .document
            .buffer
            .insert_char(0, 0, '!')
            .expect("dirty tile");
        assert_eq!(
            workspace.active_tile().save_status(),
            GuiTileSaveStatus::Modified
        );

        assert!(workspace.mark_tile_save_failed(tile_id, "permission denied"));
        assert_eq!(
            workspace.active_tile().save_status(),
            GuiTileSaveStatus::SaveFailed {
                message: String::from("permission denied"),
            }
        );
        workspace.active_tile_mut().document.buffer.mark_clean();
        assert!(workspace.clear_tile_save_error(tile_id));
        assert_eq!(
            workspace.active_tile().save_status(),
            GuiTileSaveStatus::Saved
        );
        assert!(!workspace.mark_tile_save_failed(GuiTileId(99), "missing"));
    }

    #[test]
    fn gui_file_browser_lists_and_navigates_without_iced_types() {
        let temp = TempArea::new("gui-browser-nav");
        fs::create_dir(temp.path("z-dir")).expect("create z dir");
        fs::create_dir(temp.path("a-dir")).expect("create a dir");
        fs::write(temp.path("z.txt"), "z\n").expect("write z file");
        fs::write(temp.path("a.txt"), "a\n").expect("write a file");
        fs::write(temp.path("a-dir").join("inside.txt"), "inside\n").expect("write nested file");
        let mut browser = GuiFileBrowser::load(temp.root.clone()).expect("load browser");

        let labels: Vec<_> = browser
            .sidebar
            .entries
            .iter()
            .map(|entry| entry.label.as_str())
            .collect();
        assert_eq!(labels, ["../", "a-dir/", "z-dir/", "a.txt", "z.txt"]);

        browser.sidebar.selected = browser
            .sidebar
            .entries
            .iter()
            .position(|entry| entry.label == "a-dir/")
            .expect("a-dir entry");
        assert_eq!(
            browser.activate_selected().expect("activate directory"),
            GuiFileBrowserActivation::Navigated {
                current_dir: temp
                    .path("a-dir")
                    .canonicalize()
                    .expect("canonicalize a-dir"),
            }
        );
        assert_eq!(
            browser.selected_entry().expect("parent entry").kind,
            FileSidebarEntryKind::Parent
        );
    }

    #[test]
    fn gui_file_browser_file_activation_opens_new_tile_through_existing_adapter() {
        let temp = TempArea::new("gui-browser-open");
        let first = TextDocument {
            path: PathBuf::from("first.txt"),
            buffer: TextBuffer::from_text("one\n"),
        };
        let next_path = temp.path("next.txt");
        fs::write(&next_path, "next\n").expect("write next file");
        let mut browser = GuiFileBrowser::load(temp.root.clone()).expect("load browser");
        let mut workspace = GuiWorkspace::from_document(first);

        browser.sidebar.selected = browser
            .sidebar
            .entries
            .iter()
            .position(|entry| entry.label == "next.txt")
            .expect("next file entry");

        let activation = browser.activate_selected().expect("activate file");
        assert_eq!(
            activation,
            GuiFileBrowserActivation::OpenTile {
                path: next_path.clone(),
            }
        );

        let GuiFileBrowserActivation::OpenTile { path } = activation else {
            panic!("expected open tile activation");
        };
        let tile_id = workspace
            .open_validated_tile(open_text_file(&path))
            .expect("open validated tile");
        assert_eq!(tile_id, GuiTileId(1));
        assert_eq!(workspace.tiles.len(), 2);
        assert_eq!(workspace.active_tile().document.path, next_path);
        assert_eq!(
            workspace.active_tile().document.buffer.lines(),
            &["next".to_string()]
        );
    }

    #[test]
    fn gui_file_browser_refresh_picks_up_external_files_and_preserves_selection() {
        let temp = TempArea::new("gui-browser-refresh");
        fs::write(temp.path("keep.txt"), "keep\n").expect("write keep");
        let mut browser = GuiFileBrowser::load(temp.root.clone()).expect("load browser");
        browser.sidebar.selected = browser
            .sidebar
            .entries
            .iter()
            .position(|entry| entry.label == "keep.txt")
            .expect("keep entry");

        fs::write(temp.path("added.txt"), "added\n").expect("write added");

        browser.refresh().expect("refresh browser");

        let labels = browser
            .sidebar
            .entries
            .iter()
            .map(|entry| entry.label.as_str())
            .collect::<Vec<_>>();
        assert!(labels.contains(&"added.txt"));
        assert_eq!(
            browser.selected_entry().expect("selected").label,
            "keep.txt"
        );
    }

    #[test]
    fn gui_file_browser_refresh_clamps_selection_when_selected_entry_disappears() {
        let temp = TempArea::new("gui-browser-refresh-clamp");
        let removed = temp.path("removed.txt");
        fs::write(&removed, "removed\n").expect("write removed");
        let mut browser = GuiFileBrowser::load(temp.root.clone()).expect("load browser");
        browser.sidebar.selected = browser
            .sidebar
            .entries
            .iter()
            .position(|entry| entry.label == "removed.txt")
            .expect("removed entry");

        fs::remove_file(removed).expect("remove selected file");

        browser.refresh().expect("refresh browser");

        assert!(browser.sidebar.selected < browser.sidebar.entries.len());
        assert!(!browser
            .sidebar
            .entries
            .iter()
            .any(|entry| entry.label == "removed.txt"));
    }

    #[test]
    fn gui_file_browser_rejects_invalid_roots_and_empty_selections() {
        let temp = TempArea::new("gui-browser-invalid");
        let missing = temp.path("missing");
        assert!(matches!(
            GuiFileBrowser::load(missing),
            Err(FileSidebarError::ReadDir { .. })
        ));

        let mut browser = GuiFileBrowser {
            sidebar: FileSidebarState {
                current_dir: temp.root.clone(),
                entries: Vec::new(),
                selected: 0,
                scroll: 0,
            },
        };
        assert!(matches!(
            browser.activate_selected(),
            Err(GuiFileBrowserError::EmptySelection)
        ));
    }

    #[test]
    fn gui_file_browser_mouse_row_activation_selects_visible_file() {
        let temp = TempArea::new("gui-browser-mouse");
        fs::write(temp.path("first.txt"), "first\n").expect("write first");
        fs::write(temp.path("second.txt"), "second\n").expect("write second");
        let mut browser = GuiFileBrowser::load(temp.root.clone()).expect("load browser");
        browser.sidebar.scroll = 1;

        assert_eq!(browser.activate_mouse_row(0).expect("row zero"), None);
        assert_eq!(
            browser.activate_mouse_row(2).expect("activate row"),
            Some(GuiFileBrowserActivation::OpenTile {
                path: temp.path("second.txt"),
            })
        );
        assert_eq!(
            browser.selected_entry().expect("selected").label,
            "second.txt"
        );
    }

    #[test]
    fn undo_history_is_bounded_and_redo_still_restores_latest_edit() {
        let mut buffer = TextBuffer::from_text("");

        for _ in 0..(MAX_UNDO_HISTORY + 10) {
            let column = buffer.line_char_count(0).expect("line count");
            buffer.insert_char(0, column, 'x').expect("insert");
        }

        assert_eq!(buffer.undo_history.len(), MAX_UNDO_HISTORY);
        assert!(buffer.undo_last_edit());
        let after_undo = buffer.to_text();
        assert_eq!(after_undo.len(), MAX_UNDO_HISTORY + 9);
        assert!(buffer.redo_last_undo());
        assert_eq!(buffer.to_text().len(), MAX_UNDO_HISTORY + 10);
    }
}
