//! GUI prompts, menus, layout modes, browser models, and save results.

use super::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum GuiPathPrompt {
    Open,
    SaveAs,
    ManagedNote,
    BrowserCreateFile,
    BrowserCreateDirectory,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum GuiHeaderLayoutMode {
    SingleRow,
    SplitActions,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum GuiSearchLayoutMode {
    SingleRow,
    SplitRows,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum GuiMenuCommand {
    NewTile,
    Open,
    OpenPath,
    Save,
    SaveAs,
    SaveAsPath,
    ClosePane,
    Quit,
    OpenManagedNote,
    ListManagedNotes,
    Copy,
    Cut,
    Paste,
    Undo,
    Redo,
    SelectAll,
    FindNext,
    FindPrevious,
    ToggleBrowser,
    CycleTheme,
    CycleSyntaxTheme,
    ToggleReaderMode,
    GoDocumentStart,
    GoDocumentEnd,
    GoToLine,
    ToggleMinimize,
    ToggleMaximize,
    EqualizeTiles,
    MoveLeft,
    MoveRight,
    MoveUp,
    MoveDown,
    OpenHelp,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum GuiMenuGroup {
    File,
    Edit,
    View,
    Go,
    Notes,
    Tile,
    Help,
}

#[cfg(test)]
#[derive(Clone, Copy)]
pub(crate) struct GuiActionDescriptor {
    pub(crate) label: &'static str,
    pub(crate) shortcut: Option<&'static str>,
    pub(crate) menu_group: Option<GuiMenuGroup>,
}

#[cfg(test)]
#[derive(Clone, Copy)]
pub(crate) struct GuiFocusStep {
    pub(crate) area: &'static str,
    pub(crate) label: &'static str,
    pub(crate) keyboard: Option<&'static str>,
}

pub(crate) struct GuiMenuItem {
    pub(crate) label: &'static str,
    pub(crate) command: GuiMenuCommand,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(in crate::gui::app::state) struct GuiFileTreeRowModel {
    pub(in crate::gui::app::state) path: PathBuf,
    pub(in crate::gui::app::state) label: String,
    pub(in crate::gui::app::state) kind: FileSidebarEntryKind,
    pub(in crate::gui::app::state) depth: usize,
    pub(in crate::gui::app::state) expanded: bool,
    pub(in crate::gui::app::state) selected: bool,
    pub(in crate::gui::app::state) error: bool,
}

#[cfg(test)]
impl GuiFileTreeRowModel {
    pub(in crate::gui::app::state) fn path(&self) -> &std::path::Path {
        &self.path
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(in crate::gui::app::state) struct GuiBrowserLoadResult {
    pub(in crate::gui::app::state) browser: kfnotepad::GuiFileBrowser,
    pub(in crate::gui::app::state) rows: Vec<GuiFileTreeRowModel>,
    pub(in crate::gui::app::state) selected_path: Option<PathBuf>,
    pub(in crate::gui::app::state) expanded_paths: HashSet<PathBuf>,
}

#[derive(Clone, Debug)]
pub(in crate::gui::app::state) struct GuiSaveResult {
    pub(in crate::gui::app::state) source_revision: u64,
    pub(in crate::gui::app::state) snapshot: kfnotepad::FileSnapshot,
}
