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
