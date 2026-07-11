pub(crate) fn scroll_editor_by_mouse(
    document: &TextDocument,
    cursor: &mut Cursor,
    runtime: &mut EditorRuntime,
    context: MouseContext,
    row: u16,
    direction: CursorMove,
) -> bool {
    if runtime.menu.is_some() || runtime.search_active || runtime.goto_line_active {
        return false;
    }
    if row == 0 || row as usize > context.visible_rows {
        return false;
    }

    let original = *cursor;
    for _ in 0..MOUSE_WHEEL_ROWS {
        move_cursor(document, cursor, direction);
    }
    if *cursor == original {
        return false;
    }
    runtime.quit_confirmation_pending = false;
    runtime.status = match direction {
        CursorMove::Up => String::from("Scroll up"),
        CursorMove::Down => String::from("Scroll down"),
        _ => runtime.status.clone(),
    };
    true
}
