//! Document, cursor, command, and edit result data types.

use crate::core::TextBuffer;

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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UndoRedoResult {
    Applied,
    NothingToApply,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum EditResult {
    Modified,
    Unchanged,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TextDocument {
    pub path: std::path::PathBuf,
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
