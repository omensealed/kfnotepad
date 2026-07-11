#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TextBuffer {
    pub(crate) lines: Vec<String>,
    pub(crate) trailing_newline: bool,
    pub(crate) dirty: bool,
    edit_revision: u64,
    pub(crate) undo_history: Vec<BufferSnapshot>,
    pub(crate) redo_history: Vec<BufferSnapshot>,
    pub(crate) insert_undo_group: Option<InsertUndoGroup>,
    pub(crate) file_snapshot: Option<FileSnapshot>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileSnapshot {
    pub bytes: u64,
    pub modified: Option<SystemTime>,
    pub fingerprint: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct InsertUndoGroup {
    row: usize,
    next_column: usize,
    last_edit: Instant,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct BufferSnapshot {
    pub(crate) lines: Vec<String>,
    pub(crate) trailing_newline: bool,
    pub(crate) byte_size: usize,
}

#[derive(Debug, PartialEq, Eq)]
pub enum BufferError {
    RowOutOfBounds { row: usize, rows: usize },
    ColumnOutOfBounds { column: usize, columns: usize },
    UseInsertNewline,
}
