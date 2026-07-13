//! Text-buffer state, exact undo entries, and public error types.

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
pub(crate) struct EditDelta {
    pub(crate) start: Cursor,
    pub(crate) before_end: Cursor,
    pub(crate) after_end: Cursor,
    pub(crate) before: String,
    pub(crate) after: String,
    pub(crate) trailing_newline_before: bool,
    pub(crate) trailing_newline_after: bool,
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
        Self {
            start,
            before_end,
            after_end,
            before,
            after,
            trailing_newline_before,
            trailing_newline_after,
        }
    }

    fn payload_bytes(&self) -> usize {
        self.before.capacity().saturating_add(self.after.capacity())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct EditGroup {
    pub(crate) deltas: Vec<EditDelta>,
}

impl EditGroup {
    pub(crate) fn push(&mut self, delta: EditDelta) {
        if let Some(previous) = self.deltas.last_mut() {
            let contiguous = previous.after_end == delta.start
                && previous.trailing_newline_after == delta.trailing_newline_before;
            if contiguous {
                previous.before.push_str(&delta.before);
                previous.after.push_str(&delta.after);
                previous.before_end = cursor_after_text(previous.start, &previous.before);
                previous.after_end = cursor_after_text(previous.start, &previous.after);
                previous.trailing_newline_after = delta.trailing_newline_after;
                return;
            }
        }
        self.deltas.push(delta);
    }

    pub(crate) fn byte_size(&self) -> usize {
        std::mem::size_of::<HistoryEntry>()
            .saturating_add(
                self.deltas
                    .capacity()
                    .saturating_mul(std::mem::size_of::<EditDelta>()),
            )
            .saturating_add(
                self.deltas
                    .iter()
                    .map(EditDelta::payload_bytes)
                    .sum::<usize>(),
            )
    }
}

fn cursor_after_text(start: Cursor, text: &str) -> Cursor {
    let mut segments = text.split('\n');
    let first = segments.next().unwrap_or_default();
    let mut row = start.row;
    let mut column = start.column.saturating_add(first.chars().count());
    for segment in segments {
        row = row.saturating_add(1);
        column = segment.chars().count();
    }
    Cursor { row, column }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum HistoryEntry {
    Edit(EditDelta),
    Group(EditGroup),
}

impl HistoryEntry {
    pub(crate) fn byte_size(&self) -> usize {
        match self {
            Self::Edit(delta) => std::mem::size_of::<Self>().saturating_add(delta.payload_bytes()),
            Self::Group(group) => group.byte_size(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum CompoundEditState {
    Inactive,
    Active {
        depth: usize,
        group: Option<EditGroup>,
    },
}

#[derive(Debug, PartialEq, Eq)]
pub enum BufferError {
    RowOutOfBounds { row: usize, rows: usize },
    ColumnOutOfBounds { column: usize, columns: usize },
    TooLarge { bytes: usize, limit: usize },
    UseInsertNewline,
}
