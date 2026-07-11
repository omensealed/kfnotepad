fn handle_editor_scroll_mouse_event(
    workspace: &mut EditorWorkspace<'_>,
    runtime: &mut EditorRuntime,
    event: MouseEvent,
    context: MouseContext,
) -> Option<InputResult> {
    match event.kind {
        MouseEventKind::ScrollUp => {
            let active_tab = workspace.active_tab_mut();
            if scroll_editor_by_mouse(
                active_tab.document.as_mut(),
                &mut active_tab.state.cursor,
                runtime,
                context,
                event.row,
                CursorMove::Up,
            ) {
                Some(InputResult::Handled)
            } else {
                Some(InputResult::Ignored)
            }
        }
        MouseEventKind::ScrollDown => {
            let active_tab = workspace.active_tab_mut();
            if scroll_editor_by_mouse(
                active_tab.document.as_mut(),
                &mut active_tab.state.cursor,
                runtime,
                context,
                event.row,
                CursorMove::Down,
            ) {
                Some(InputResult::Handled)
            } else {
                Some(InputResult::Ignored)
            }
        }
        _ => None,
    }
}
