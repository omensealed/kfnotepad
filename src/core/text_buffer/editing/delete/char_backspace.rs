impl TextBuffer {
    pub fn delete_char(&mut self, row: usize, column: usize) -> Result<(), BufferError> {
        let rows = self.lines.len();
        let line_columns = self.line_char_count(row)?;

        if column < line_columns {
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
            line.replace_range(start..end, "");
            self.mark_changed();
            return Ok(());
        }

        if column == line_columns && row + 1 < rows {
            self.break_undo_group();
            self.record_undo();
            let next_line = self.lines.remove(row + 1);
            self.lines[row].push_str(&next_line);
            self.mark_changed();
            return Ok(());
        }

        if column == line_columns {
            return Ok(());
        }

        Err(BufferError::ColumnOutOfBounds {
            column,
            columns: line_columns,
        })
    }

    pub fn delete_before_cursor(&mut self, cursor: Cursor) -> Result<Cursor, BufferError> {
        self.validate_cursor(cursor)?;

        if cursor.column > 0 {
            let previous_column = {
                let line = self
                    .lines
                    .get(cursor.row)
                    .ok_or(BufferError::RowOutOfBounds {
                        row: cursor.row,
                        rows: self.lines.len(),
                    })?;
                previous_grapheme_column(line, cursor.column)?
            };
            self.delete_char(cursor.row, previous_column)?;
            return Ok(Cursor {
                row: cursor.row,
                column: previous_column,
            });
        }

        if cursor.row > 0 {
            let previous_row = cursor.row - 1;
            let previous_columns = self.line_char_count(previous_row)?;
            self.break_undo_group();
            self.record_undo();
            let current_line = self.lines.remove(cursor.row);
            self.lines[previous_row].push_str(&current_line);
            self.mark_changed();
            return Ok(Cursor {
                row: previous_row,
                column: previous_columns,
            });
        }

        Ok(cursor)
    }
}
