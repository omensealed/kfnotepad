impl TextBuffer {
    pub fn move_cursor(
        &self,
        cursor: Cursor,
        direction: CursorMove,
    ) -> Result<Cursor, BufferError> {
        self.validate_cursor(cursor)?;

        let moved = match direction {
            CursorMove::Left if cursor.column > 0 => {
                let line = &self.lines[cursor.row];
                Cursor {
                    row: cursor.row,
                    column: previous_grapheme_column(line, cursor.column)?,
                }
            }
            CursorMove::Left if cursor.row > 0 => {
                let row = cursor.row - 1;
                Cursor {
                    row,
                    column: self.line_char_count(row)?,
                }
            }
            CursorMove::Left => cursor,
            CursorMove::Right if cursor.column < self.line_char_count(cursor.row)? => {
                let line = &self.lines[cursor.row];
                Cursor {
                    row: cursor.row,
                    column: next_grapheme_column(line, cursor.column)?,
                }
            }
            CursorMove::Right if cursor.row + 1 < self.lines.len() => Cursor {
                row: cursor.row + 1,
                column: 0,
            },
            CursorMove::Right => cursor,
            CursorMove::WordLeft => self.previous_word_cursor(cursor),
            CursorMove::WordRight => self.next_word_cursor(cursor),
            CursorMove::Up if cursor.row > 0 => {
                self.cursor_on_row(cursor.row - 1, cursor.column)?
            }
            CursorMove::Up => cursor,
            CursorMove::Down if cursor.row + 1 < self.lines.len() => {
                self.cursor_on_row(cursor.row + 1, cursor.column)?
            }
            CursorMove::Down => cursor,
        };

        Ok(moved)
    }
}
