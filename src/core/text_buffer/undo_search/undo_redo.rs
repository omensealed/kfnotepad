impl TextBuffer {
    pub fn undo_last_edit(&mut self) -> bool {
        let Some(snapshot) = pop_history_snapshot(&mut self.undo_history, &mut self.undo_bytes)
        else {
            return false;
        };
        self.break_undo_group();

        let redo_snapshot = BufferSnapshot {
            lines: self.lines.clone(),
            trailing_newline: self.trailing_newline,
            byte_size: buffer_bytes(&self.lines, self.trailing_newline),
        };
        push_history_snapshot(
            &mut self.redo_history,
            &mut self.redo_bytes,
            redo_snapshot,
            MAX_UNDO_HISTORY,
            MAX_UNDO_BYTES,
        );

        self.lines = snapshot.lines;
        self.trailing_newline = snapshot.trailing_newline;
        self.mark_changed();
        true
    }

    pub fn redo_last_undo(&mut self) -> bool {
        let Some(snapshot) = pop_history_snapshot(&mut self.redo_history, &mut self.redo_bytes)
        else {
            return false;
        };
        self.break_undo_group();

        let undo_snapshot = BufferSnapshot {
            lines: self.lines.clone(),
            trailing_newline: self.trailing_newline,
            byte_size: buffer_bytes(&self.lines, self.trailing_newline),
        };
        push_history_snapshot(
            &mut self.undo_history,
            &mut self.undo_bytes,
            undo_snapshot,
            MAX_UNDO_HISTORY,
            MAX_UNDO_BYTES,
        );

        self.lines = snapshot.lines;
        self.trailing_newline = snapshot.trailing_newline;
        self.mark_changed();
        true
    }
}
