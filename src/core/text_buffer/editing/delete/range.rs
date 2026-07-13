//! Grapheme-normalized same-line and multiline range deletion.

use super::*;

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
        let last_row = self.lines.len().saturating_sub(1);
        let removes_entire_document = start == (Cursor { row: 0, column: 0 })
            && end.row == last_row
            && self
                .lines
                .get(last_row)
                .is_some_and(|line| end.column == line.chars().count());
        self.delete_range_with_trailing_newline_policy(start, end, removes_entire_document)
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
        let deleted_text = self.text_in_range_without_normalization(start, end)?;
        let trailing_newline_before = self.trailing_newline;
        let trailing_newline_after = trailing_newline_before && !remove_trailing_newline;

        self.delete_range_without_history(start, end)?;
        self.trailing_newline = trailing_newline_after;
        self.record_delete_text_undo(
            start,
            end,
            deleted_text,
            trailing_newline_before,
            trailing_newline_after,
        );
        self.mark_changed();
        Ok(())
    }

    pub(in crate::core::text_buffer) fn text_in_range_without_normalization(
        &self,
        start: Cursor,
        end: Cursor,
    ) -> Result<String, BufferError> {
        self.validate_cursor(start)?;
        self.validate_cursor(end)?;

        let start_byte = byte_index_for_char_column(&self.lines[start.row], start.column)?;
        let end_byte = byte_index_for_char_column(&self.lines[end.row], end.column)?;
        if start.row == end.row {
            return Ok(self.lines[start.row][start_byte..end_byte].to_owned());
        }

        let mut text = self.lines[start.row][start_byte..].to_owned();
        for row in (start.row + 1)..end.row {
            text.push('\n');
            text.push_str(&self.lines[row]);
        }
        text.push('\n');
        text.push_str(&self.lines[end.row][..end_byte]);
        Ok(text)
    }

    pub(in crate::core::text_buffer) fn delete_range_without_history(
        &mut self,
        start: Cursor,
        end: Cursor,
    ) -> Result<(), BufferError> {
        self.validate_cursor(start)?;
        self.validate_cursor(end)?;

        if start.row == end.row {
            let line = &mut self.lines[start.row];
            let start_byte = byte_index_for_char_column(line, start.column)?;
            let end_byte = byte_index_for_char_column(line, end.column)?;
            line.replace_range(start_byte..end_byte, "");
            return Ok(());
        }

        let start_byte = byte_index_for_char_column(&self.lines[start.row], start.column)?;
        let end_byte = byte_index_for_char_column(&self.lines[end.row], end.column)?;
        let end_suffix = self.lines[end.row][end_byte..].to_owned();
        self.lines[start.row].truncate(start_byte);
        self.lines[start.row].push_str(&end_suffix);
        self.lines.drain((start.row + 1)..=end.row);
        Ok(())
    }

    pub(in crate::core::text_buffer) fn replace_range_without_history(
        &mut self,
        start: Cursor,
        end: Cursor,
        text: &str,
    ) -> Result<Cursor, BufferError> {
        self.validate_cursor(start)?;
        self.validate_cursor(end)?;
        let byte_index = byte_index_for_char_column(&self.lines[start.row], start.column)?;
        if start.row == end.row && !text.contains('\n') {
            let end_byte = byte_index_for_char_column(&self.lines[end.row], end.column)?;
            if end_byte.saturating_sub(byte_index) == text.len() {
                self.lines[start.row].replace_range(byte_index..end_byte, text);
                return Ok(Cursor {
                    row: start.row,
                    column: start.column.saturating_add(text.chars().count()),
                });
            }
        }
        self.delete_range_without_history(start, end)?;
        Ok(self.insert_text_without_history(start, byte_index, text))
    }

    fn range_start_boundary_cursor(&self, cursor: Cursor) -> Result<Cursor, BufferError> {
        let line = self
            .lines
            .get(cursor.row)
            .ok_or(BufferError::RowOutOfBounds {
                row: cursor.row,
                rows: self.lines.len(),
            })?;
        if line.is_ascii() {
            return Ok(cursor);
        }
        Ok(Cursor {
            row: cursor.row,
            column: grapheme_range_start_boundary_column(line, cursor.column),
        })
    }

    fn range_end_boundary_cursor(&self, cursor: Cursor) -> Result<Cursor, BufferError> {
        let line = self
            .lines
            .get(cursor.row)
            .ok_or(BufferError::RowOutOfBounds {
                row: cursor.row,
                rows: self.lines.len(),
            })?;
        if line.is_ascii() {
            return Ok(cursor);
        }
        Ok(Cursor {
            row: cursor.row,
            column: grapheme_range_end_boundary_column(line, cursor.column),
        })
    }
}
