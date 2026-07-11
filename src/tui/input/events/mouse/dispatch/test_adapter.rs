#[cfg(test)]
pub(crate) fn handle_mouse_event(
    document: &mut TextDocument,
    cursor: &mut Cursor,
    runtime: &mut EditorRuntime,
    event: MouseEvent,
    context: MouseContext,
) -> InputResult {
    let mut workspace = EditorWorkspace::from_document(document);
    workspace.active_tab_mut().state.cursor = *cursor;
    let result = handle_workspace_mouse_event(&mut workspace, runtime, event, context);
    *cursor = workspace.active_tab().state.cursor;
    result
}
