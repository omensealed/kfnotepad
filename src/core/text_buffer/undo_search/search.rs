//! Forward and backward document search with configurable case sensitivity.

use super::*;

impl TextBuffer {
    pub fn find_next(&self, query: &str, start: Cursor) -> Option<Cursor> {
        self.find_next_with_mode(
            query,
            start,
            SearchMode {
                case_sensitive: true,
            },
        )
    }

    pub fn find_next_with_mode(
        &self,
        query: &str,
        start: Cursor,
        mode: SearchMode,
    ) -> Option<Cursor> {
        if query.is_empty() || self.validate_cursor(start).is_err() {
            return None;
        }

        for row in start.row..self.lines.len() {
            let column = if row == start.row { start.column } else { 0 };
            if let Some(cursor) = find_in_line_with_mode(&self.lines[row], query, column, row, mode)
            {
                return Some(cursor);
            }
        }

        for row in 0..start.row {
            if let Some(cursor) = find_in_line_with_mode(&self.lines[row], query, 0, row, mode) {
                return Some(cursor);
            }
        }

        None
    }

    pub fn find_previous(&self, query: &str, start: Cursor) -> Option<Cursor> {
        self.find_previous_with_mode(
            query,
            start,
            SearchMode {
                case_sensitive: true,
            },
        )
    }

    pub fn find_previous_with_mode(
        &self,
        query: &str,
        start: Cursor,
        mode: SearchMode,
    ) -> Option<Cursor> {
        if query.is_empty() || self.validate_cursor(start).is_err() {
            return None;
        }

        for row in (0..=start.row).rev() {
            let column = if row == start.row {
                start.column
            } else {
                self.lines[row].chars().count()
            };
            if let Some(cursor) =
                find_last_in_line_before_with_mode(&self.lines[row], query, column, row, mode)
            {
                return Some(cursor);
            }
        }

        for row in (start.row + 1..self.lines.len()).rev() {
            let column = self.lines[row].chars().count();
            if let Some(cursor) =
                find_last_in_line_before_with_mode(&self.lines[row], query, column, row, mode)
            {
                return Some(cursor);
            }
        }

        None
    }
}
