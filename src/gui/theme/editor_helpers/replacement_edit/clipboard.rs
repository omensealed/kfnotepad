#[allow(dead_code)]
pub(super) fn gui_editor_replacement_copy_selection(
    document: &TextDocument,
    selection: Option<GuiEditorReplacementSelection>,
) -> Option<String> {
    let selected = gui_editor_replacement_selected_text(document, selection?)?;
    (!selected.is_empty()).then_some(selected)
}

#[allow(dead_code)]
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

#[allow(dead_code)]
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

    document.buffer.break_undo_group();

    for character in text.chars() {
        let input = if character == '\n' {
            GuiEditorReplacementInput::InsertNewline
        } else {
            GuiEditorReplacementInput::InsertChar(character)
        };
        apply_gui_editor_replacement_input(document, cursor, viewport, selection, input);
    }
}
