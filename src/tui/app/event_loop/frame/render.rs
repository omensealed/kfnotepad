fn render_frame<W: Write>(
    stdout: &mut W,
    workspace: &mut EditorWorkspace<'_>,
    runtime: &mut EditorRuntime,
    highlighter: &SyntaxHighlighter,
    syntax_cache: &mut render::TuiSyntaxHighlightCache,
    layout: &mut LoopLayout,
) -> io::Result<()> {
    let tab_items = workspace.tab_strip_items();
    let active_tab = workspace.active_tab_mut();
    let frame_layout = prepare_frame_layout(active_tab, runtime, &tab_items, layout);
    render::render_editor_with_cache(
        stdout,
        active_tab.document.as_ref(),
        render::EditorView {
            cursor: active_tab.state.cursor,
            viewport_start: active_tab.state.viewport_start,
            horizontal_offset: active_tab.state.horizontal_offset,
            visible_rows: layout.visible_rows,
            status: &runtime.status,
            settings: runtime.settings,
            menu: runtime.menu,
            sidebar_width: runtime.sidebar.as_ref().map_or(0, |_| super::SIDEBAR_WIDTH),
            tab_strip: &tab_items,
            search_highlight: runtime.search_highlight(),
        },
        highlighter,
        syntax_cache,
    )?;
    write_runtime_overlays(stdout, runtime, &tab_items, frame_layout)
}
