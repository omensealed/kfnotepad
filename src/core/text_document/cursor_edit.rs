pub fn clamp_cursor_to_document(document: &TextDocument, cursor: &mut Cursor) {
    cursor.row = cursor
        .row
        .min(document.buffer.line_count().saturating_sub(1));
    cursor.column = cursor
        .column
        .min(document.buffer.line_char_count(cursor.row).unwrap_or(0));
}

impl TextDocument {
    pub fn with_compound_edit<R>(&mut self, edit: impl FnOnce(&mut Self) -> R) -> R {
        self.buffer.begin_compound_edit();
        let result = edit(self);
        self.buffer.end_compound_edit();
        result
    }
}

pub fn move_document_cursor(document: &TextDocument, cursor: &mut Cursor, direction: CursorMove) {
    if let Ok(moved) = document.buffer.move_cursor(*cursor, direction) {
        *cursor = moved;
    }
}

pub fn undo_document_edit(document: &mut TextDocument, cursor: &mut Cursor) -> UndoRedoResult {
    if document.buffer.undo_last_edit() {
        clamp_cursor_to_document(document, cursor);
        UndoRedoResult::Applied
    } else {
        UndoRedoResult::NothingToApply
    }
}

pub fn redo_document_edit(document: &mut TextDocument, cursor: &mut Cursor) -> UndoRedoResult {
    if document.buffer.redo_last_undo() {
        clamp_cursor_to_document(document, cursor);
        UndoRedoResult::Applied
    } else {
        UndoRedoResult::NothingToApply
    }
}

pub fn delete_previous_word(document: &mut TextDocument, cursor: &mut Cursor) -> EditResult {
    if let Ok(moved) = document.buffer.delete_previous_word(*cursor) {
        *cursor = moved;
        EditResult::Modified
    } else {
        EditResult::Unchanged
    }
}

pub fn delete_next_word(document: &mut TextDocument, cursor: &mut Cursor) -> EditResult {
    if let Ok(moved) = document.buffer.delete_next_word(*cursor) {
        *cursor = moved;
        EditResult::Modified
    } else {
        EditResult::Unchanged
    }
}

pub fn delete_to_line_end(document: &mut TextDocument, cursor: &mut Cursor) -> EditResult {
    if let Ok(moved) = document.buffer.delete_to_line_end(*cursor) {
        *cursor = moved;
        EditResult::Modified
    } else {
        EditResult::Unchanged
    }
}

pub fn page_up(document: &TextDocument, cursor: &mut Cursor, page_rows: usize) {
    cursor.row = cursor.row.saturating_sub(page_rows.max(1));
    clamp_cursor_to_document(document, cursor);
}

pub fn page_down(document: &TextDocument, cursor: &mut Cursor, page_rows: usize) {
    cursor.row = cursor
        .row
        .saturating_add(page_rows.max(1))
        .min(document.buffer.line_count().saturating_sub(1));
    clamp_cursor_to_document(document, cursor);
}

pub fn go_to_document_start(cursor: &mut Cursor) {
    *cursor = Cursor { row: 0, column: 0 };
}

pub fn go_to_document_end(document: &TextDocument, cursor: &mut Cursor) {
    let row = document.buffer.line_count().saturating_sub(1);
    let column = document.buffer.line_char_count(row).unwrap_or(0);
    *cursor = Cursor { row, column };
}

pub fn go_to_line(document: &TextDocument, cursor: &mut Cursor, query: &str) -> GoToLineResult {
    if query.is_empty() {
        return GoToLineResult::Empty;
    }

    let Ok(line_number) = query.parse::<usize>() else {
        return GoToLineResult::Invalid;
    };

    if !(1..=document.buffer.line_count()).contains(&line_number) {
        return GoToLineResult::OutOfRange { line_number };
    }

    cursor.row = line_number - 1;
    clamp_cursor_to_document(document, cursor);
    GoToLineResult::Moved { line_number }
}
