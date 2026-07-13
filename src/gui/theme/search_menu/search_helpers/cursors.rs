use super::*;

pub(in crate::gui::app::state) fn document_cursor_from_editor(
    cursor: text_editor::Cursor,
) -> DocumentCursor {
    let position = match cursor.selection {
        Some(selection)
            if (selection.line, selection.column)
                < (cursor.position.line, cursor.position.column) =>
        {
            selection
        }
        _ => cursor.position,
    };

    DocumentCursor {
        row: position.line,
        column: position.column,
    }
}

pub(in crate::gui::app::state) fn editor_cursor_from_document(
    cursor: DocumentCursor,
) -> text_editor::Cursor {
    text_editor::Cursor {
        position: text_editor::Position {
            line: cursor.row,
            column: cursor.column,
        },
        selection: None,
    }
}
