impl TextBuffer {
    pub fn undo_last_edit(&mut self) -> bool {
        let Some(snapshot) = self.undo_history.pop() else {
            return false;
        };
        self.break_undo_group();

        self.redo_history.push(BufferSnapshot {
            lines: self.lines.clone(),
            trailing_newline: self.trailing_newline,
            byte_size: buffer_bytes(&self.lines, self.trailing_newline),
        });
        trim_undo_history(&mut self.redo_history, MAX_UNDO_HISTORY, MAX_UNDO_BYTES);

        self.lines = snapshot.lines;
        self.trailing_newline = snapshot.trailing_newline;
        self.mark_changed();
        true
    }

    pub fn redo_last_undo(&mut self) -> bool {
        let Some(snapshot) = self.redo_history.pop() else {
            return false;
        };
        self.break_undo_group();

        self.undo_history.push(BufferSnapshot {
            lines: self.lines.clone(),
            trailing_newline: self.trailing_newline,
            byte_size: buffer_bytes(&self.lines, self.trailing_newline),
        });
        trim_undo_history(&mut self.undo_history, MAX_UNDO_HISTORY, MAX_UNDO_BYTES);

        self.lines = snapshot.lines;
        self.trailing_newline = snapshot.trailing_newline;
        self.mark_changed();
        true
    }
}
