#[allow(dead_code)]
pub(super) fn gui_editor_replacement_mouse_click(
    document: &TextDocument,
    cursor: &mut DocumentCursor,
    viewport: &mut GuiEditorViewportState,
    selection: &mut Option<GuiEditorReplacementSelection>,
    point: GuiEditorReplacementMousePoint,
) {
    *cursor = gui_editor_replacement_cursor_from_mouse_point(&document.buffer, *viewport, point);
    *selection = None;
}

#[allow(dead_code)]
pub(super) fn gui_editor_replacement_mouse_drag(
    document: &TextDocument,
    cursor: &mut DocumentCursor,
    viewport: &mut GuiEditorViewportState,
    selection: &mut Option<GuiEditorReplacementSelection>,
    anchor: DocumentCursor,
    focus: GuiEditorReplacementMousePoint,
) {
    let focus = gui_editor_replacement_cursor_from_mouse_point(&document.buffer, *viewport, focus);
    *cursor = focus;
    *selection = GuiEditorReplacementSelection::new(anchor, focus);
}
