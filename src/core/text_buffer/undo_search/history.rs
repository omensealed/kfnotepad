impl TextBuffer {
    fn record_undo_for_typed_insert(&mut self, row: usize, column: usize) {
        let now = Instant::now();
        let can_merge = self.insert_undo_group.is_some_and(|group| {
            group.row == row
                && group.next_column == column
                && now.duration_since(group.last_edit) <= TYPING_UNDO_COALESCE_WINDOW
        });

        if can_merge {
            self.insert_undo_group = Some(InsertUndoGroup {
                row,
                next_column: column.saturating_add(1),
                last_edit: now,
            });
            return;
        }

        self.record_undo();
        self.insert_undo_group = Some(InsertUndoGroup {
            row,
            next_column: column.saturating_add(1),
            last_edit: now,
        });
    }

    pub(crate) fn break_undo_group(&mut self) {
        self.insert_undo_group = None;
    }

    fn mark_changed(&mut self) {
        self.dirty = true;
        self.edit_revision = self.edit_revision.wrapping_add(1);
    }

    fn record_undo(&mut self) {
        self.insert_undo_group = None;
        self.undo_history.push(BufferSnapshot::from_buffer(self));
        trim_undo_history(&mut self.undo_history, MAX_UNDO_HISTORY, MAX_UNDO_BYTES);
        self.redo_history.clear();
    }
}
