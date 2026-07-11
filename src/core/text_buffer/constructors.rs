impl TextBuffer {
    pub fn from_text(text: &str) -> Self {
        #[cfg(test)]
        FROM_TEXT_CALL_COUNT.with(|count| count.set(count.get() + 1));

        let mut lines: Vec<String> = text.lines().map(ToString::to_string).collect();
        if lines.is_empty() {
            lines.push(String::new());
        }

        Self {
            lines,
            trailing_newline: text.ends_with('\n'),
            dirty: false,
            edit_revision: 0,
            undo_history: Vec::new(),
            redo_history: Vec::new(),
            insert_undo_group: None,
            file_snapshot: None,
        }
    }

    pub fn lines(&self) -> &[String] {
        &self.lines
    }

    pub fn line(&self, row: usize) -> Option<&str> {
        self.lines.get(row).map(String::as_str)
    }

    pub fn line_count(&self) -> usize {
        self.lines.len()
    }

    pub fn has_trailing_newline(&self) -> bool {
        self.trailing_newline
    }

    pub fn is_dirty(&self) -> bool {
        self.dirty
    }

    pub fn edit_revision(&self) -> u64 {
        self.edit_revision
    }

    pub fn mark_clean(&mut self) {
        self.dirty = false;
        self.undo_history.clear();
        self.redo_history.clear();
        self.insert_undo_group = None;
    }

    pub fn file_snapshot(&self) -> Option<&FileSnapshot> {
        self.file_snapshot.as_ref()
    }

    pub fn set_file_snapshot(&mut self, snapshot: Option<FileSnapshot>) {
        self.file_snapshot = snapshot;
    }

    pub fn to_text(&self) -> String {
        #[cfg(test)]
        TO_TEXT_CALL_COUNT.with(|count| count.set(count.get() + 1));

        let mut text = self.lines.join("\n");
        if self.trailing_newline {
            text.push('\n');
        }
        text
    }

    pub fn replace_text(&mut self, text: &str) {
        if self.to_text() == text {
            return;
        }

        let next_revision = self.edit_revision.wrapping_add(1);
        let mut replacement = Self::from_text(text);
        replacement.dirty = true;
        replacement.edit_revision = next_revision;
        replacement.file_snapshot = self.file_snapshot.clone();
        *self = replacement;
    }

    pub fn line_char_count(&self, row: usize) -> Result<usize, BufferError> {
        let rows = self.lines.len();
        let line = self
            .lines
            .get(row)
            .ok_or(BufferError::RowOutOfBounds { row, rows })?;
        Ok(line.chars().count())
    }

    pub fn grapheme_boundary_column(
        &self,
        row: usize,
        column: usize,
    ) -> Result<usize, BufferError> {
        let rows = self.lines.len();
        let line = self
            .lines
            .get(row)
            .ok_or(BufferError::RowOutOfBounds { row, rows })?;
        nearest_grapheme_boundary_column(line, column)
    }

    pub fn grapheme_range_boundary_columns(
        &self,
        row: usize,
        start_column: usize,
        end_column: usize,
    ) -> Result<(usize, usize), BufferError> {
        let rows = self.lines.len();
        let line = self
            .lines
            .get(row)
            .ok_or(BufferError::RowOutOfBounds { row, rows })?;
        grapheme_range_boundary_columns_for_line(line, start_column, end_column)
    }

    pub fn grapheme_range_start_boundary_column(
        &self,
        row: usize,
        column: usize,
    ) -> Result<usize, BufferError> {
        let rows = self.lines.len();
        let line = self
            .lines
            .get(row)
            .ok_or(BufferError::RowOutOfBounds { row, rows })?;
        let columns = line.chars().count();
        if column > columns {
            return Err(BufferError::ColumnOutOfBounds { column, columns });
        }
        Ok(grapheme_range_start_boundary_column(line, column))
    }

    pub fn grapheme_range_end_boundary_column(
        &self,
        row: usize,
        column: usize,
    ) -> Result<usize, BufferError> {
        let rows = self.lines.len();
        let line = self
            .lines
            .get(row)
            .ok_or(BufferError::RowOutOfBounds { row, rows })?;
        let columns = line.chars().count();
        if column > columns {
            return Err(BufferError::ColumnOutOfBounds { column, columns });
        }
        Ok(grapheme_range_end_boundary_column(line, column))
    }
}
