pub(crate) fn clamp_viewport(
    document: &TextDocument,
    cursor: Cursor,
    viewport_start: usize,
    visible_rows: usize,
    settings: EditorSettings,
    gutter_width: usize,
    terminal_width: usize,
) -> usize {
    let visible_rows = visible_rows.max(1);
    let max_start = max_viewport_start(document, visible_rows, settings);
    let mut start = viewport_start.min(max_start);

    if cursor.row < start {
        start = cursor.row;
    } else if cursor.row >= start + visible_rows {
        start = cursor.row + 1 - visible_rows;
    }

    start = start.min(max_start);
    if settings.wrap_lines {
        while start < max_start
            && !cursor_is_visible_from_viewport(
                document,
                cursor,
                start,
                visible_rows,
                settings,
                gutter_width,
                terminal_width,
            )
        {
            start += 1;
        }
    }

    start
}

fn cursor_is_visible_from_viewport(
    document: &TextDocument,
    cursor: Cursor,
    viewport_start: usize,
    visible_rows: usize,
    settings: EditorSettings,
    gutter_width: usize,
    terminal_width: usize,
) -> bool {
    let frame = RenderFrame {
        theme: EditorTheme::for_id(settings.theme_id),
        gutter_width,
        terminal_width,
        origin_column: 0,
        body_top: 0,
        no_color: false,
    };
    let view = EditorView {
        cursor,
        viewport_start,
        horizontal_offset: 0,
        visible_rows,
        status: "",
        settings,
        menu: None,
        sidebar_width: 0,
        tab_strip: &[],
        search_highlight: None,
    };
    cursor_row_is_visible(document, view, frame)
}
