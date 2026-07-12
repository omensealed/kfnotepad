//! Word and line-tail deletion commands.

use super::*;

impl TextBuffer {
    pub fn delete_previous_word(&mut self, cursor: Cursor) -> Result<Cursor, BufferError> {
        self.validate_cursor(cursor)?;
        let start = self.previous_word_cursor(cursor);
        self.delete_range(start, cursor)?;
        Ok(start)
    }

    pub fn delete_next_word(&mut self, cursor: Cursor) -> Result<Cursor, BufferError> {
        self.validate_cursor(cursor)?;
        let end = self.next_word_delete_end_cursor(cursor);
        self.delete_range(cursor, end)?;
        Ok(cursor)
    }

    pub fn delete_to_line_end(&mut self, cursor: Cursor) -> Result<Cursor, BufferError> {
        self.validate_cursor(cursor)?;
        let end = Cursor {
            row: cursor.row,
            column: self.line_char_count(cursor.row)?,
        };
        self.delete_range(cursor, end)?;
        Ok(cursor)
    }
}
