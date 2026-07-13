//! Undo grouping, compound-edit boundaries, and history entry recording.

use super::*;

impl TextBuffer {
    pub(in crate::core::text_buffer) fn record_typed_insert_undo(
        &mut self,
        row: usize,
        column: usize,
        value: char,
    ) {
        let now = Instant::now();
        let can_merge = self.insert_undo_group.is_some_and(|group| {
            group.row == row
                && group.next_column == column
                && now.duration_since(group.last_edit) <= TYPING_UNDO_COALESCE_WINDOW
        });

        let start = Cursor { row, column };
        let end = Cursor {
            row,
            column: column.saturating_add(1),
        };
        let mut merged = false;
        if can_merge {
            if let Some(entry) = pop_history_entry(&mut self.undo_history, &mut self.undo_bytes) {
                match entry {
                    HistoryEntry::Edit(mut delta)
                        if delta.before.is_empty()
                            && delta.before_end == delta.start
                            && delta.after_end == start
                            && delta.trailing_newline_before == delta.trailing_newline_after =>
                    {
                        delta.after.push(value);
                        delta.after_end = end;
                        delta.refresh_byte_size();
                        push_history_entry(
                            &mut self.undo_history,
                            &mut self.undo_bytes,
                            HistoryEntry::Edit(delta),
                            MAX_UNDO_HISTORY,
                            MAX_UNDO_BYTES,
                        );
                        merged = true;
                    }
                    entry => push_history_entry(
                        &mut self.undo_history,
                        &mut self.undo_bytes,
                        entry,
                        MAX_UNDO_HISTORY,
                        MAX_UNDO_BYTES,
                    ),
                }
            }
        }

        if !merged {
            push_history_entry(
                &mut self.undo_history,
                &mut self.undo_bytes,
                HistoryEntry::Edit(EditDelta::insertion(
                    start,
                    end,
                    value.to_string(),
                    self.trailing_newline,
                )),
                MAX_UNDO_HISTORY,
                MAX_UNDO_BYTES,
            );
        }

        self.insert_undo_group = Some(InsertUndoGroup {
            row,
            next_column: column.saturating_add(1),
            last_edit: now,
        });
        self.redo_history.clear();
        self.redo_bytes = 0;
    }

    pub(crate) fn break_undo_group(&mut self) {
        self.insert_undo_group = None;
    }

    pub(crate) fn begin_compound_edit(&mut self) {
        self.compound_edit =
            match std::mem::replace(&mut self.compound_edit, CompoundEditState::Inactive) {
                CompoundEditState::Inactive => {
                    self.break_undo_group();
                    CompoundEditState::Pending {
                        depth: 1,
                        snapshot: Box::new(BufferSnapshot::from_buffer(self)),
                    }
                }
                CompoundEditState::Pending { depth, snapshot } => CompoundEditState::Pending {
                    depth: depth.saturating_add(1),
                    snapshot,
                },
                CompoundEditState::Recorded { depth } => CompoundEditState::Recorded {
                    depth: depth.saturating_add(1),
                },
            };
    }

    pub(crate) fn end_compound_edit(&mut self) {
        self.compound_edit =
            match std::mem::replace(&mut self.compound_edit, CompoundEditState::Inactive) {
                CompoundEditState::Pending { depth, snapshot } if depth > 1 => {
                    CompoundEditState::Pending {
                        depth: depth - 1,
                        snapshot,
                    }
                }
                CompoundEditState::Recorded { depth } if depth > 1 => {
                    CompoundEditState::Recorded { depth: depth - 1 }
                }
                _ => {
                    self.break_undo_group();
                    CompoundEditState::Inactive
                }
            };
        if matches!(self.compound_edit, CompoundEditState::Inactive) {
            self.break_undo_group();
        }
    }

    pub(in crate::core::text_buffer) fn mark_changed(&mut self) {
        self.dirty = true;
        self.edit_revision = self.edit_revision.wrapping_add(1);
    }

    pub(in crate::core::text_buffer) fn record_undo(&mut self) {
        self.insert_undo_group = None;
        let snapshot = match std::mem::replace(&mut self.compound_edit, CompoundEditState::Inactive)
        {
            CompoundEditState::Pending { depth, snapshot } => {
                self.compound_edit = CompoundEditState::Recorded { depth };
                *snapshot
            }
            recorded @ CompoundEditState::Recorded { .. } => {
                self.compound_edit = recorded;
                return;
            }
            CompoundEditState::Inactive => BufferSnapshot::from_buffer(self),
        };
        push_history_snapshot(
            &mut self.undo_history,
            &mut self.undo_bytes,
            snapshot,
            MAX_UNDO_HISTORY,
            MAX_UNDO_BYTES,
        );
        self.redo_history.clear();
        self.redo_bytes = 0;
    }

    pub(in crate::core::text_buffer) fn record_insert_text_undo(
        &mut self,
        start: Cursor,
        end: Cursor,
        text: &str,
    ) {
        self.record_edit_delta(EditDelta::insertion(
            start,
            end,
            text.to_owned(),
            self.trailing_newline,
        ));
    }

    pub(in crate::core::text_buffer) fn record_delete_text_undo(
        &mut self,
        start: Cursor,
        end: Cursor,
        text: String,
        trailing_newline_before: bool,
        trailing_newline_after: bool,
    ) {
        self.record_edit_delta(EditDelta::deletion(
            start,
            end,
            text,
            trailing_newline_before,
            trailing_newline_after,
        ));
    }

    pub(in crate::core::text_buffer) fn record_replace_text_undo(
        &mut self,
        start: Cursor,
        before_end: Cursor,
        after_end: Cursor,
        before: String,
        after: String,
    ) {
        self.record_edit_delta(EditDelta::replacement(
            start,
            before_end,
            after_end,
            before,
            after,
            self.trailing_newline,
        ));
    }

    fn record_edit_delta(&mut self, delta: EditDelta) {
        push_history_entry(
            &mut self.undo_history,
            &mut self.undo_bytes,
            HistoryEntry::Edit(delta),
            MAX_UNDO_HISTORY,
            MAX_UNDO_BYTES,
        );
        self.redo_history.clear();
        self.redo_bytes = 0;
    }
}
