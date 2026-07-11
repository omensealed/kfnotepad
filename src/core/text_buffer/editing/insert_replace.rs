impl TextBuffer {
    pub fn insert_text(&mut self, cursor: Cursor, text: &str) -> Result<Cursor, BufferError> {
        self.validate_cursor(cursor)?;
        if text.is_empty() {
            return Ok(cursor);
        }
        let next_bytes = self.byte_len().saturating_add(text.len());
        self.ensure_byte_len(next_bytes)?;

        let byte_index = {
            let line = &self.lines[cursor.row];
            byte_index_for_char_column(line, cursor.column)?
        };
        let segments = text.split('\n').collect::<Vec<_>>();

        self.break_undo_group();
        self.record_undo();

        let next_cursor = if segments.len() == 1 {
            self.lines[cursor.row].insert_str(byte_index, text);
            Cursor {
                row: cursor.row,
                column: cursor.column.saturating_add(text.chars().count()),
            }
        } else {
            let remainder = self.lines[cursor.row].split_off(byte_index);
            self.lines[cursor.row].push_str(segments[0]);

            let last_index = segments.len() - 1;
            let mut inserted_lines = Vec::with_capacity(last_index);
            inserted_lines.extend(segments[1..last_index].iter().map(|line| (*line).to_string()));
            inserted_lines.push(format!("{}{remainder}", segments[last_index]));
            self.lines
                .splice((cursor.row + 1)..(cursor.row + 1), inserted_lines);

            Cursor {
                row: cursor.row.saturating_add(last_index),
                column: segments[last_index].chars().count(),
            }
        };

        self.mark_changed();
        let column = self
            .grapheme_range_end_boundary_column(next_cursor.row, next_cursor.column)
            .unwrap_or(next_cursor.column);
        Ok(Cursor {
            column,
            ..next_cursor
        })
    }

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

        let next_bytes = self.byte_len().saturating_add(value.len_utf8());
        self.ensure_byte_len(next_bytes)?;

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

        let removed_bytes = end.saturating_sub(start);
        let next_bytes = self
            .byte_len()
            .saturating_sub(removed_bytes)
            .saturating_add(value.len_utf8());
        self.ensure_byte_len(next_bytes)?;

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

        let next_bytes = self.byte_len().saturating_add(1);
        self.ensure_byte_len(next_bytes)?;

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
