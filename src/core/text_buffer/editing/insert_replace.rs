//! Text insertion, character replacement, and newline splitting.

use super::*;

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
        self.break_undo_group();

        let next_cursor = self.insert_text_without_history(cursor, byte_index, text);
        self.record_insert_text_undo(cursor, next_cursor, text);

        self.mark_changed();
        let final_segment = text.rsplit('\n').next().unwrap_or(text);
        let end_byte = if next_cursor.row == cursor.row {
            byte_index.saturating_add(text.len())
        } else {
            final_segment.len()
        };
        let next_byte_is_ascii = self.lines[next_cursor.row]
            .as_bytes()
            .get(end_byte)
            .is_none_or(u8::is_ascii);
        let column = if final_segment.is_empty() || (final_segment.is_ascii() && next_byte_is_ascii)
        {
            next_cursor.column
        } else {
            self.grapheme_range_end_boundary_column(next_cursor.row, next_cursor.column)
                .unwrap_or(next_cursor.column)
        };
        Ok(Cursor {
            column,
            ..next_cursor
        })
    }

    pub(in crate::core::text_buffer) fn insert_text_without_history(
        &mut self,
        cursor: Cursor,
        byte_index: usize,
        text: &str,
    ) -> Cursor {
        let segments = text.split('\n').collect::<Vec<_>>();
        if segments.len() == 1 {
            self.lines[cursor.row].insert_str(byte_index, text);
            return Cursor {
                row: cursor.row,
                column: cursor.column.saturating_add(text.chars().count()),
            };
        }

        let remainder = self.lines[cursor.row].split_off(byte_index);
        self.lines[cursor.row].push_str(segments[0]);

        let last_index = segments.len() - 1;
        let mut inserted_lines = Vec::with_capacity(last_index);
        inserted_lines.extend(
            segments[1..last_index]
                .iter()
                .map(|line| (*line).to_string()),
        );
        inserted_lines.push(format!("{}{remainder}", segments[last_index]));
        self.lines
            .splice((cursor.row + 1)..(cursor.row + 1), inserted_lines);

        Cursor {
            row: cursor.row.saturating_add(last_index),
            column: segments[last_index].chars().count(),
        }
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

        let standalone_edit = matches!(self.compound_edit, CompoundEditState::Inactive);
        let line = self
            .lines
            .get_mut(row)
            .ok_or(BufferError::RowOutOfBounds { row, rows })?;
        line.insert(byte_index, value);
        if standalone_edit {
            self.record_typed_insert_undo(row, column, value);
        } else {
            self.record_insert_text_undo(
                Cursor { row, column },
                Cursor {
                    row,
                    column: column.saturating_add(1),
                },
                &value.to_string(),
            );
        }
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
            if line.is_ascii() {
                (column, column + 1)
            } else {
                grapheme_char_range_at_column(line, column)?.unwrap_or((column, column + 1))
            }
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

        let start_cursor = Cursor {
            row,
            column: start_column,
        };
        let before_end = Cursor {
            row,
            column: end_column,
        };
        let after = value.to_string();
        let after_end = Cursor {
            row,
            column: start_column.saturating_add(1),
        };
        let before = self.text_in_range_without_normalization(start_cursor, before_end)?;
        self.break_undo_group();
        self.replace_range_without_history(start_cursor, before_end, &after)?;
        self.record_replace_text_undo(start_cursor, before_end, after_end, before, after);
        self.mark_changed();
        Ok(())
    }

    pub fn insert_newline(&mut self, row: usize, column: usize) -> Result<(), BufferError> {
        self.insert_text(Cursor { row, column }, "\n").map(|_| ())
    }
}
