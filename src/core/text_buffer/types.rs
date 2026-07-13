//! Text-buffer state, undo entries, snapshots, and public error types.

use super::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TextBuffer {
    pub(crate) lines: Vec<String>,
    pub(crate) trailing_newline: bool,
    pub(crate) dirty: bool,
    pub(super) edit_revision: u64,
    pub(crate) undo_history: VecDeque<HistoryEntry>,
    pub(crate) redo_history: VecDeque<HistoryEntry>,
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
pub(crate) enum HistoryEntry {
    Snapshot(BufferSnapshot),
    InsertText {
        start: Cursor,
        end: Cursor,
        text: String,
        byte_size: usize,
    },
    DeleteText {
        start: Cursor,
        end: Cursor,
        text: String,
        trailing_newline_before: bool,
        trailing_newline_after: bool,
        byte_size: usize,
    },
}

impl HistoryEntry {
    pub(crate) fn byte_size(&self) -> usize {
        match self {
            Self::Snapshot(snapshot) => snapshot.byte_size,
            Self::InsertText { byte_size, .. } | Self::DeleteText { byte_size, .. } => *byte_size,
        }
    }
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
