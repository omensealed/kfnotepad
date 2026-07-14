use super::*;

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
