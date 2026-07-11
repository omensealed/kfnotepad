#[cfg(test)]
pub(super) fn gui_editor_replacement_copy_selection(
    document: &TextDocument,
    selection: Option<GuiEditorReplacementSelection>,
) -> Option<String> {
    let selected = gui_editor_replacement_selected_text(document, selection?)?;
    (!selected.is_empty()).then_some(selected)
}

#[cfg(test)]
pub(super) fn gui_editor_replacement_cut_selection(
    document: &mut TextDocument,
    cursor: &mut DocumentCursor,
    viewport: &mut GuiEditorViewportState,
    selection: &mut Option<GuiEditorReplacementSelection>,
) -> Option<String> {
    let selected = gui_editor_replacement_copy_selection(document, *selection)?;
    delete_gui_editor_replacement_selection(document, cursor, selection);
    viewport.keep_cursor_visible(*cursor, document.buffer.line_count());
    Some(selected)
}

pub(super) fn gui_editor_replacement_paste_text(
    document: &mut TextDocument,
    cursor: &mut DocumentCursor,
    viewport: &mut GuiEditorViewportState,
    selection: &mut Option<GuiEditorReplacementSelection>,
    text: &str,
) {
    if text.is_empty() {
        return;
    }

    let selected_bytes = selection
        .and_then(|selection| gui_editor_replacement_selected_text(document, selection))
        .map_or(0, |selected| selected.len());
    let projected_bytes = document
        .buffer
        .byte_len()
        .saturating_sub(selected_bytes)
        .saturating_add(text.len());
    if document.buffer.ensure_byte_len(projected_bytes).is_err() {
        return;
    }

    document.with_compound_edit(|document| {
        delete_gui_editor_replacement_selection(document, cursor, selection);
        if let Ok(next_cursor) = document.buffer.insert_text(*cursor, text) {
            *cursor = next_cursor;
        }
        viewport.keep_cursor_visible(*cursor, document.buffer.line_count());
    });
}
