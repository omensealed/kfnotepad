use super::*;

pub(in crate::gui::app::state) fn gui_editor_replacement_selected_text_from_text(
    text: &str,
    selection: GuiEditorReplacementSelection,
) -> Option<String> {
    let lines: Vec<&str> = text.split('\n').collect();
    let (start, end) = selection.normalized();
    if lines.is_empty() {
        return None;
    }

    let buffer = TextBuffer::from_text(text);
    let text_end = gui_editor_replacement_document_end_cursor(&buffer);
    if start.row == 0 && start.column == 0 && end == text_end {
        return Some(text.to_string());
    }

    let end_cursor = DocumentCursor {
        row: lines.len().saturating_sub(1),
        column: lines.last().unwrap_or(&"").chars().count(),
    };
    if start.row > end_cursor.row || end.row > end_cursor.row {
        return None;
    }
    if start.row == 0 {
        let start_columns = lines
            .first()
            .map(|line| line.chars().count())
            .unwrap_or_default();
        if start.column > start_columns {
            return None;
        }
    } else {
        let start_columns = lines
            .get(start.row)
            .map(|line| line.chars().count())
            .unwrap_or_default();
        if start.column > start_columns {
            return None;
        }
    }
    if end.row == 0 {
        let end_columns = lines
            .first()
            .map(|line| line.chars().count())
            .unwrap_or_default();
        if end.column > end_columns {
            return None;
        }
    } else {
        let end_columns = lines
            .get(end.row)
            .map(|line| line.chars().count())
            .unwrap_or_default();
        if end.column > end_columns {
            return None;
        }
    }
    if start == (DocumentCursor { row: 0, column: 0 }) && end == end_cursor {
        return Some(text.to_string());
    }
    let (start, end) = gui_editor_replacement_grapheme_range(&buffer, start, end).ok()?;

    if start.row == end.row {
        return lines
            .get(start.row)
            .map(|line| char_slice(line, start.column, end.column));
    }

    let mut selected = Vec::new();
    let first = lines.get(start.row)?;
    selected.push(char_suffix(first, start.column));
    for row in (start.row + 1)..end.row {
        selected.push(lines.get(row)?.to_string());
    }
    let last = lines.get(end.row)?;
    selected.push(char_prefix(last, end.column));
    Some(selected.join("\n"))
}

pub(in crate::gui::app::state) fn gui_editor_replacement_copy_selection_from_text(
    text: &str,
    selection: Option<GuiEditorReplacementSelection>,
) -> Option<String> {
    let selected = gui_editor_replacement_selected_text_from_text(text, selection?)?;
    (!selected.is_empty()).then_some(selected)
}

pub(in crate::gui::app::state) fn gui_editor_replacement_selected_text(
    document: &TextDocument,
    selection: GuiEditorReplacementSelection,
) -> Option<String> {
    let (start, end) = selection.normalized();
    if !gui_editor_replacement_cursor_is_valid(&document.buffer, start)
        || !gui_editor_replacement_cursor_is_valid(&document.buffer, end)
    {
        return None;
    }

    if gui_editor_replacement_selection_covers_full_text(document, start, end) {
        return Some(document.buffer.to_text());
    }
    let (start, end) = gui_editor_replacement_grapheme_range(&document.buffer, start, end).ok()?;

    let lines = document.buffer.lines();
    if start.row == end.row {
        return lines
            .get(start.row)
            .map(|line| char_slice(line, start.column, end.column));
    }

    let mut selected = Vec::new();
    let first = lines.get(start.row)?;
    selected.push(char_suffix(first, start.column));
    for row in (start.row + 1)..end.row {
        selected.push(lines.get(row)?.to_string());
    }
    let last = lines.get(end.row)?;
    selected.push(char_prefix(last, end.column));
    Some(selected.join("\n"))
}
