struct PreparedFrameLayout {
    visible_rows: usize,
    terminal_width: usize,
    editor_width: usize,
    no_color: bool,
}

fn prepare_frame_layout(
    active_tab: &mut EditorTab<'_>,
    runtime: &mut EditorRuntime,
    tab_items: &[TabStripItem],
    layout: &mut LoopLayout,
) -> PreparedFrameLayout {
    layout.terminal_width = super::terminal_width();
    let sidebar_width = runtime.sidebar.as_ref().map_or(0, |_| super::SIDEBAR_WIDTH);
    let editor_width = layout.terminal_width.saturating_sub(sidebar_width).max(1);
    let no_color = super::no_color_enabled();
    let tab_extra_rows = super::tab_strip_height_for_width(tab_items, editor_width)
        .saturating_sub(1)
        .into();
    layout.visible_rows = super::visible_editor_rows(tab_extra_rows);
    runtime.page_rows = layout.visible_rows;
    layout.gutter_width = super::line_number_width(active_tab.document.as_ref());
    active_tab.state.viewport_start = if runtime.settings.gui_reader_mode_enabled {
        super::clamp_passive_viewport(
            active_tab.document.as_ref(),
            active_tab.state.viewport_start,
            layout.visible_rows,
            runtime.settings,
        )
    } else {
        super::clamp_viewport(
            active_tab.document.as_ref(),
            active_tab.state.cursor,
            active_tab.state.viewport_start,
            layout.visible_rows,
            runtime.settings,
            layout.gutter_width,
            editor_width,
        )
    };
    active_tab.state.horizontal_offset = if runtime.settings.wrap_lines {
        0
    } else {
        super::clamp_horizontal_viewport(
            active_tab.document.as_ref(),
            active_tab.state.cursor,
            runtime.settings,
            layout.gutter_width,
            layout.terminal_width,
            active_tab.state.horizontal_offset,
        )
    };
    PreparedFrameLayout {
        visible_rows: layout.visible_rows,
        terminal_width: layout.terminal_width,
        editor_width,
        no_color,
    }
}
