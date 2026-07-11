fn read_event_or_apply_reader_tick(
    workspace: &mut EditorWorkspace<'_>,
    runtime: &mut EditorRuntime,
    visible_rows: usize,
) -> io::Result<(Option<Event>, bool)> {
    if runtime.settings.gui_reader_mode_enabled {
        if poll(Duration::from_millis(TUI_READER_TICK_MS))? {
            Ok((Some(read()?), false))
        } else {
            let active_tab = workspace.active_tab_mut();
            let redraw = apply_reader_tick(
                active_tab.document.as_ref(),
                &mut active_tab.state,
                runtime,
                visible_rows,
            );
            Ok((None, redraw))
        }
    } else {
        Ok((Some(read()?), false))
    }
}
