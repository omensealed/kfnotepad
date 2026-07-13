//! Byte-budgeted undo and redo entry application.

use super::*;

impl TextBuffer {
    pub fn undo_last_edit(&mut self) -> bool {
        let Some(entry) = pop_history_entry(&mut self.undo_history, &mut self.undo_bytes) else {
            return false;
        };
        self.break_undo_group();

        match entry {
            HistoryEntry::Edit(delta) => {
                if !self.apply_delta_undo(&delta) {
                    push_history_entry(
                        &mut self.undo_history,
                        &mut self.undo_bytes,
                        HistoryEntry::Edit(delta),
                        MAX_UNDO_HISTORY,
                        MAX_UNDO_BYTES,
                    );
                    return false;
                }
                push_history_entry(
                    &mut self.redo_history,
                    &mut self.redo_bytes,
                    HistoryEntry::Edit(delta),
                    MAX_UNDO_HISTORY,
                    MAX_UNDO_BYTES,
                );
            }
            HistoryEntry::Group(group) => {
                if !self.apply_group_undo(&group) {
                    push_history_entry(
                        &mut self.undo_history,
                        &mut self.undo_bytes,
                        HistoryEntry::Group(group),
                        MAX_UNDO_HISTORY,
                        MAX_UNDO_BYTES,
                    );
                    return false;
                }
                push_history_entry(
                    &mut self.redo_history,
                    &mut self.redo_bytes,
                    HistoryEntry::Group(group),
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
            HistoryEntry::Edit(delta) => {
                if !self.apply_delta_redo(&delta) {
                    push_history_entry(
                        &mut self.redo_history,
                        &mut self.redo_bytes,
                        HistoryEntry::Edit(delta),
                        MAX_UNDO_HISTORY,
                        MAX_UNDO_BYTES,
                    );
                    return false;
                }
                push_history_entry(
                    &mut self.undo_history,
                    &mut self.undo_bytes,
                    HistoryEntry::Edit(delta),
                    MAX_UNDO_HISTORY,
                    MAX_UNDO_BYTES,
                );
            }
            HistoryEntry::Group(group) => {
                if !self.apply_group_redo(&group) {
                    push_history_entry(
                        &mut self.redo_history,
                        &mut self.redo_bytes,
                        HistoryEntry::Group(group),
                        MAX_UNDO_HISTORY,
                        MAX_UNDO_BYTES,
                    );
                    return false;
                }
                push_history_entry(
                    &mut self.undo_history,
                    &mut self.undo_bytes,
                    HistoryEntry::Group(group),
                    MAX_UNDO_HISTORY,
                    MAX_UNDO_BYTES,
                );
            }
        }
        self.mark_changed();
        true
    }

    fn apply_delta_undo(&mut self, delta: &EditDelta) -> bool {
        if self
            .replace_range_without_history(delta.start, delta.after_end, &delta.before)
            .is_err()
        {
            return false;
        }
        self.trailing_newline = delta.trailing_newline_before;
        true
    }

    fn apply_delta_redo(&mut self, delta: &EditDelta) -> bool {
        if self
            .replace_range_without_history(delta.start, delta.before_end, &delta.after)
            .is_err()
        {
            return false;
        }
        self.trailing_newline = delta.trailing_newline_after;
        true
    }

    fn apply_group_undo(&mut self, group: &EditGroup) -> bool {
        for (applied, delta) in group.deltas.iter().rev().enumerate() {
            if !self.apply_delta_undo(delta) {
                let rollback_start = group.deltas.len().saturating_sub(applied);
                for rollback in &group.deltas[rollback_start..] {
                    let _ = self.apply_delta_redo(rollback);
                }
                return false;
            }
        }
        true
    }

    fn apply_group_redo(&mut self, group: &EditGroup) -> bool {
        for (applied, delta) in group.deltas.iter().enumerate() {
            if !self.apply_delta_redo(delta) {
                for rollback in group.deltas[..applied].iter().rev() {
                    let _ = self.apply_delta_undo(rollback);
                }
                return false;
            }
        }
        true
    }
}
