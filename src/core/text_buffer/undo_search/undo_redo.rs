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
            HistoryEntry::Edit(delta) => {
                if self
                    .replace_range_without_history(delta.start, delta.after_end, &delta.before)
                    .is_err()
                {
                    push_history_entry(
                        &mut self.undo_history,
                        &mut self.undo_bytes,
                        HistoryEntry::Edit(delta),
                        MAX_UNDO_HISTORY,
                        MAX_UNDO_BYTES,
                    );
                    return false;
                }
                self.trailing_newline = delta.trailing_newline_before;
                push_history_entry(
                    &mut self.redo_history,
                    &mut self.redo_bytes,
                    HistoryEntry::Edit(delta),
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
            HistoryEntry::Edit(delta) => {
                if self
                    .replace_range_without_history(delta.start, delta.before_end, &delta.after)
                    .is_err()
                {
                    push_history_entry(
                        &mut self.redo_history,
                        &mut self.redo_bytes,
                        HistoryEntry::Edit(delta),
                        MAX_UNDO_HISTORY,
                        MAX_UNDO_BYTES,
                    );
                    return false;
                }
                self.trailing_newline = delta.trailing_newline_after;
                push_history_entry(
                    &mut self.undo_history,
                    &mut self.undo_bytes,
                    HistoryEntry::Edit(delta),
                    MAX_UNDO_HISTORY,
                    MAX_UNDO_BYTES,
                );
            }
        }
        self.mark_changed();
        true
    }
}
