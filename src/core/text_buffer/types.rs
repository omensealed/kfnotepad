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
pub(crate) struct EditDelta {
    pub(crate) start: Cursor,
    pub(crate) before_end: Cursor,
    pub(crate) after_end: Cursor,
    pub(crate) before: String,
    pub(crate) after: String,
    pub(crate) trailing_newline_before: bool,
    pub(crate) trailing_newline_after: bool,
    pub(crate) byte_size: usize,
}

impl EditDelta {
    pub(crate) fn insertion(
        start: Cursor,
        after_end: Cursor,
        after: String,
        trailing_newline: bool,
    ) -> Self {
        Self::new(
            start,
            start,
            after_end,
            String::new(),
            after,
            trailing_newline,
            trailing_newline,
        )
    }

    pub(crate) fn deletion(
        start: Cursor,
        before_end: Cursor,
        before: String,
        trailing_newline_before: bool,
        trailing_newline_after: bool,
    ) -> Self {
        Self::new(
            start,
            before_end,
            start,
            before,
            String::new(),
            trailing_newline_before,
            trailing_newline_after,
        )
    }

    pub(crate) fn replacement(
        start: Cursor,
        before_end: Cursor,
        after_end: Cursor,
        before: String,
        after: String,
        trailing_newline: bool,
    ) -> Self {
        Self::new(
            start,
            before_end,
            after_end,
            before,
            after,
            trailing_newline,
            trailing_newline,
        )
    }

    fn new(
        start: Cursor,
        before_end: Cursor,
        after_end: Cursor,
        before: String,
        after: String,
        trailing_newline_before: bool,
        trailing_newline_after: bool,
    ) -> Self {
        let mut delta = Self {
            start,
            before_end,
            after_end,
            before,
            after,
            trailing_newline_before,
            trailing_newline_after,
            byte_size: 0,
        };
        delta.refresh_byte_size();
        delta
    }

    pub(crate) fn refresh_byte_size(&mut self) {
        self.byte_size = self
            .before
            .capacity()
            .saturating_add(self.after.capacity())
            .saturating_add(std::mem::size_of::<HistoryEntry>());
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum HistoryEntry {
    Snapshot(BufferSnapshot),
    Edit(EditDelta),
}

impl HistoryEntry {
    pub(crate) fn byte_size(&self) -> usize {
        match self {
            Self::Snapshot(snapshot) => snapshot.byte_size,
            Self::Edit(delta) => delta.byte_size,
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
