//! Byte-budgeted undo and redo entry application.

use super::*;

impl TextBuffer {
    pub fn undo_last_edit(&mut self) -> bool {
        let Some(entry) = pop_history_entry(&mut self.undo_history, &mut self.undo_bytes) else {
            return false;
        };
        self.break_undo_group();

        match entry {
            HistoryEntry::Snapshot(snapshot) => {
                let redo_snapshot = BufferSnapshot::from_buffer(self);
                push_history_snapshot(
                    &mut self.redo_history,
                    &mut self.redo_bytes,
                    redo_snapshot,
                    MAX_UNDO_HISTORY,
                    MAX_UNDO_BYTES,
                );
                self.lines = snapshot.lines;
                self.trailing_newline = snapshot.trailing_newline;
            }
            HistoryEntry::InsertText {
                start,
                end,
                text,
                byte_size,
            } => {
                let entry = HistoryEntry::InsertText {
                    start,
                    end,
                    text,
                    byte_size,
                };
                if self.delete_range_without_history(start, end).is_err() {
                    push_history_entry(
                        &mut self.undo_history,
                        &mut self.undo_bytes,
                        entry,
                        MAX_UNDO_HISTORY,
                        MAX_UNDO_BYTES,
                    );
                    return false;
                }
                push_history_entry(
                    &mut self.redo_history,
                    &mut self.redo_bytes,
                    entry,
                    MAX_UNDO_HISTORY,
                    MAX_UNDO_BYTES,
                );
            }
            HistoryEntry::DeleteText {
                start,
                end,
                text,
                trailing_newline_before,
                trailing_newline_after,
                byte_size,
            } => {
                let Ok(byte_index) = self
                    .lines
                    .get(start.row)
                    .ok_or(BufferError::RowOutOfBounds {
                        row: start.row,
                        rows: self.lines.len(),
                    })
                    .and_then(|line| byte_index_for_char_column(line, start.column))
                else {
                    push_history_entry(
                        &mut self.undo_history,
                        &mut self.undo_bytes,
                        HistoryEntry::DeleteText {
                            start,
                            end,
                            text,
                            trailing_newline_before,
                            trailing_newline_after,
                            byte_size,
                        },
                        MAX_UNDO_HISTORY,
                        MAX_UNDO_BYTES,
                    );
                    return false;
                };
                self.insert_text_without_history(start, byte_index, &text);
                self.trailing_newline = trailing_newline_before;
                push_history_entry(
                    &mut self.redo_history,
                    &mut self.redo_bytes,
                    HistoryEntry::DeleteText {
                        start,
                        end,
                        text,
                        trailing_newline_before,
                        trailing_newline_after,
                        byte_size,
                    },
                    MAX_UNDO_HISTORY,
                    MAX_UNDO_BYTES,
                );
            }
        }
        self.mark_changed();
        true
    }

    pub fn redo_last_undo(&mut self) -> bool {
        let Some(entry) = pop_history_entry(&mut self.redo_history, &mut self.redo_bytes) else {
            return false;
        };
        self.break_undo_group();

        match entry {
            HistoryEntry::Snapshot(snapshot) => {
                let undo_snapshot = BufferSnapshot::from_buffer(self);
                push_history_snapshot(
                    &mut self.undo_history,
                    &mut self.undo_bytes,
                    undo_snapshot,
                    MAX_UNDO_HISTORY,
                    MAX_UNDO_BYTES,
                );
                self.lines = snapshot.lines;
                self.trailing_newline = snapshot.trailing_newline;
            }
            HistoryEntry::InsertText {
                start,
                end,
                text,
                byte_size,
            } => {
                let Ok(byte_index) = self
                    .lines
                    .get(start.row)
                    .ok_or(BufferError::RowOutOfBounds {
                        row: start.row,
                        rows: self.lines.len(),
                    })
                    .and_then(|line| byte_index_for_char_column(line, start.column))
                else {
                    push_history_entry(
                        &mut self.redo_history,
                        &mut self.redo_bytes,
                        HistoryEntry::InsertText {
                            start,
                            end,
                            text,
                            byte_size,
                        },
                        MAX_UNDO_HISTORY,
                        MAX_UNDO_BYTES,
                    );
                    return false;
                };
                self.insert_text_without_history(start, byte_index, &text);
                push_history_entry(
                    &mut self.undo_history,
                    &mut self.undo_bytes,
                    HistoryEntry::InsertText {
                        start,
                        end,
                        text,
                        byte_size,
                    },
                    MAX_UNDO_HISTORY,
                    MAX_UNDO_BYTES,
                );
            }
            HistoryEntry::DeleteText {
                start,
                end,
                text,
                trailing_newline_before,
                trailing_newline_after,
                byte_size,
            } => {
                if self.delete_range_without_history(start, end).is_err() {
                    push_history_entry(
                        &mut self.redo_history,
                        &mut self.redo_bytes,
                        HistoryEntry::DeleteText {
                            start,
                            end,
                            text,
                            trailing_newline_before,
                            trailing_newline_after,
                            byte_size,
                        },
                        MAX_UNDO_HISTORY,
                        MAX_UNDO_BYTES,
                    );
                    return false;
                }
                self.trailing_newline = trailing_newline_after;
                push_history_entry(
                    &mut self.undo_history,
                    &mut self.undo_bytes,
                    HistoryEntry::DeleteText {
                        start,
                        end,
                        text,
                        trailing_newline_before,
                        trailing_newline_after,
                        byte_size,
                    },
                    MAX_UNDO_HISTORY,
                    MAX_UNDO_BYTES,
                );
            }
        }
        self.mark_changed();
        true
    }
}
