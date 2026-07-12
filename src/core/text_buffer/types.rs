//! Text-buffer state, snapshots, and public error types.

use super::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TextBuffer {
    pub(crate) lines: Vec<String>,
    pub(crate) trailing_newline: bool,
    pub(crate) dirty: bool,
    pub(super) edit_revision: u64,
    pub(crate) undo_history: VecDeque<BufferSnapshot>,
    pub(crate) redo_history: VecDeque<BufferSnapshot>,
    pub(crate) undo_bytes: usize,
    pub(crate) redo_bytes: usize,
    pub(crate) insert_undo_group: Option<InsertUndoGroup>,
    pub(crate) compound_edit: CompoundEditState,
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
    pub(super) row: usize,
    pub(super) next_column: usize,
    pub(super) last_edit: Instant,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct BufferSnapshot {
    pub(crate) lines: Vec<String>,
    pub(crate) trailing_newline: bool,
    pub(crate) byte_size: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum CompoundEditState {
    Inactive,
    Pending {
        depth: usize,
        snapshot: Box<BufferSnapshot>,
    },
    Recorded {
        depth: usize,
    },
}

#[derive(Debug, PartialEq, Eq)]
pub enum BufferError {
    RowOutOfBounds { row: usize, rows: usize },
    ColumnOutOfBounds { column: usize, columns: usize },
    TooLarge { bytes: usize, limit: usize },
    UseInsertNewline,
}
