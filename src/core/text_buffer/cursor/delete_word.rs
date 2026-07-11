impl TextBuffer {
    fn next_word_delete_end_cursor(&self, cursor: Cursor) -> Cursor {
        let mut row = cursor.row;
        let mut column = cursor.column;

        loop {
            let (units, mut index) = grapheme_word_index_for_column(&self.lines[row], column)
                .unwrap_or_else(|_| (Vec::new(), 0));

            if index < units.len() && units[index].is_word {
                while index < units.len() && units[index].is_word {
                    index += 1;
                }
                return Cursor {
                    row,
                    column: units[index - 1].end,
                };
            }

            while index < units.len() && !units[index].is_word {
                index += 1;
            }

            if index < units.len() {
                while index < units.len() && units[index].is_word {
                    index += 1;
                }
                return Cursor {
                    row,
                    column: units[index - 1].end,
                };
            }

            if row + 1 >= self.lines.len() {
                return Cursor {
                    row,
                    column: self.lines[row].chars().count(),
                };
            }

            row += 1;
            column = 0;
        }
    }
}
