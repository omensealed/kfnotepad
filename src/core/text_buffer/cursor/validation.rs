//! Cursor validation and grapheme-boundary normalization.

use super::*;

impl TextBuffer {
    pub(in crate::core::text_buffer) fn validate_cursor(
        &self,
        cursor: Cursor,
    ) -> Result<(), BufferError> {
        let columns = self.line_char_count(cursor.row)?;
        if cursor.column > columns {
            return Err(BufferError::ColumnOutOfBounds {
                column: cursor.column,
                columns,
            });
        }
        Ok(())
    }

    pub(super) fn cursor_on_row(
        &self,
        row: usize,
        requested_column: usize,
    ) -> Result<Cursor, BufferError> {
        let line = self.lines.get(row).ok_or(BufferError::RowOutOfBounds {
            row,
            rows: self.lines.len(),
        })?;
        let column = requested_column.min(self.line_char_count(row)?);
        Ok(Cursor {
            row,
            column: nearest_grapheme_boundary_column(line, column)?,
        })
    }
}
