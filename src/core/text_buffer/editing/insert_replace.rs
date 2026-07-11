impl TextBuffer {
    pub fn insert_char(
        &mut self,
        row: usize,
        column: usize,
        value: char,
    ) -> Result<(), BufferError> {
        if value == '\n' {
            return Err(BufferError::UseInsertNewline);
        }

        let rows = self.lines.len();
        let byte_index = {
            let line = self
                .lines
                .get(row)
                .ok_or(BufferError::RowOutOfBounds { row, rows })?;
            byte_index_for_char_column(line, column)?
        };

        self.record_undo_for_typed_insert(row, column);
        let line = self
            .lines
            .get_mut(row)
            .ok_or(BufferError::RowOutOfBounds { row, rows })?;
        line.insert(byte_index, value);
        self.mark_changed();
        Ok(())
    }

    pub fn replace_char(
        &mut self,
        row: usize,
        column: usize,
        value: char,
    ) -> Result<(), BufferError> {
        if value == '\n' {
            return Err(BufferError::UseInsertNewline);
        }

        let rows = self.lines.len();
        let line_columns = self.line_char_count(row)?;
        if column >= line_columns {
            return self.insert_char(row, column, value);
        }

        let (start_column, end_column) = {
            let line = self
                .lines
                .get(row)
                .ok_or(BufferError::RowOutOfBounds { row, rows })?;
            grapheme_char_range_at_column(line, column)?.unwrap_or((column, column + 1))
        };
        let start = {
            let line = self
                .lines
                .get(row)
                .ok_or(BufferError::RowOutOfBounds { row, rows })?;
            byte_index_for_char_column(line, start_column)?
        };
        let end = {
            let line = self
                .lines
                .get(row)
                .ok_or(BufferError::RowOutOfBounds { row, rows })?;
            byte_index_for_char_column(line, end_column)?
        };

        self.break_undo_group();
        self.record_undo();
        let line = self
            .lines
            .get_mut(row)
            .ok_or(BufferError::RowOutOfBounds { row, rows })?;
        line.replace_range(start..end, &value.to_string());
        self.mark_changed();
        Ok(())
    }

    pub fn insert_newline(&mut self, row: usize, column: usize) -> Result<(), BufferError> {
        let rows = self.lines.len();
        let byte_index = {
            let line = self
                .lines
                .get(row)
                .ok_or(BufferError::RowOutOfBounds { row, rows })?;
            byte_index_for_char_column(line, column)?
        };

        self.break_undo_group();
        self.record_undo();
        let line = self
            .lines
            .get_mut(row)
            .ok_or(BufferError::RowOutOfBounds { row, rows })?;
        let remainder = line.split_off(byte_index);
        self.lines.insert(row + 1, remainder);
        self.mark_changed();
        Ok(())
    }
}
