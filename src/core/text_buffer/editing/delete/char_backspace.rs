//! Forward character deletion and cursor-relative backspace behavior.

use super::*;

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
            return self.delete_range(
                Cursor {
                    row,
                    column: start_column,
                },
                Cursor {
                    row,
                    column: end_column,
                },
            );
        }

        if column == line_columns && row + 1 < rows {
            return self.delete_range(
                Cursor { row, column },
                Cursor {
                    row: row + 1,
                    column: 0,
                },
            );
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
            self.delete_range(
                Cursor {
                    row: previous_row,
                    column: previous_columns,
                },
                Cursor {
                    row: cursor.row,
                    column: 0,
                },
            )?;
            return Ok(Cursor {
                row: previous_row,
                column: previous_columns,
            });
        }

        Ok(cursor)
    }
}
