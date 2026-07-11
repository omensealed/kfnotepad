impl TextBuffer {
    pub fn delete_range(&mut self, start: Cursor, end: Cursor) -> Result<(), BufferError> {
        self.delete_range_with_trailing_newline_policy(start, end, false)
    }

    #[cfg(feature = "gui")]
    pub(crate) fn delete_replacement_range(
        &mut self,
        start: Cursor,
        end: Cursor,
    ) -> Result<(), BufferError> {
        self.delete_range_with_trailing_newline_policy(start, end, true)
    }

    fn delete_range_with_trailing_newline_policy(
        &mut self,
        start: Cursor,
        end: Cursor,
        remove_trailing_newline: bool,
    ) -> Result<(), BufferError> {
        self.validate_cursor(start)?;
        self.validate_cursor(end)?;
        let start = self.range_start_boundary_cursor(start)?;
        let end = self.range_end_boundary_cursor(end)?;

        if start == end {
            return Ok(());
        }

        self.break_undo_group();
        self.record_undo();

        if start.row == end.row {
            let rows = self.lines.len();
            let line = self
                .lines
                .get_mut(start.row)
                .ok_or(BufferError::RowOutOfBounds {
                    row: start.row,
                    rows,
                })?;
            let start_byte = byte_index_for_char_column(line, start.column)?;
            let end_byte = byte_index_for_char_column(line, end.column)?;
            line.replace_range(start_byte..end_byte, "");
            if remove_trailing_newline {
                self.trailing_newline = false;
            }
            self.mark_changed();
            return Ok(());
        }

        let start_prefix = {
            let line = self
                .lines
                .get(start.row)
                .ok_or(BufferError::RowOutOfBounds {
                    row: start.row,
                    rows: self.lines.len(),
                })?;
            let start_byte = byte_index_for_char_column(line, start.column)?;
            line[..start_byte].to_string()
        };
        let end_suffix = {
            let line = self.lines.get(end.row).ok_or(BufferError::RowOutOfBounds {
                row: end.row,
                rows: self.lines.len(),
            })?;
            let end_byte = byte_index_for_char_column(line, end.column)?;
            line[end_byte..].to_string()
        };

        self.lines[start.row] = format!("{start_prefix}{end_suffix}");
        self.lines.drain((start.row + 1)..=end.row);
        if remove_trailing_newline {
            self.trailing_newline = false;
        }
        self.mark_changed();
        Ok(())
    }

    fn range_start_boundary_cursor(&self, cursor: Cursor) -> Result<Cursor, BufferError> {
        let line = self.lines.get(cursor.row).ok_or(BufferError::RowOutOfBounds {
            row: cursor.row,
            rows: self.lines.len(),
        })?;
        Ok(Cursor {
            row: cursor.row,
            column: grapheme_range_start_boundary_column(line, cursor.column),
        })
    }

    fn range_end_boundary_cursor(&self, cursor: Cursor) -> Result<Cursor, BufferError> {
        let line = self.lines.get(cursor.row).ok_or(BufferError::RowOutOfBounds {
            row: cursor.row,
            rows: self.lines.len(),
        })?;
        Ok(Cursor {
            row: cursor.row,
            column: grapheme_range_end_boundary_column(line, cursor.column),
        })
    }
}
