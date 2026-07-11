pub(crate) fn toggle_reader_mode(runtime: &mut EditorRuntime) {
    runtime.quit_confirmation_pending = false;
    runtime.reader_scroll_milli_lines = 0;
    runtime.settings.gui_reader_mode_enabled = !runtime.settings.gui_reader_mode_enabled;
    runtime.status = if runtime.settings.gui_reader_mode_enabled {
        format!(
            "Reader mode on: {} lines/min",
            runtime.settings.gui_reader_lines_per_minute
        )
    } else {
        String::from("Reader mode off")
    };
    persist_runtime_settings(runtime);
}

pub(crate) fn adjust_reader_speed(runtime: &mut EditorRuntime, delta: i16) {
    runtime.quit_confirmation_pending = false;
    runtime.reader_scroll_milli_lines = 0;
    let current = i16::try_from(runtime.settings.gui_reader_lines_per_minute)
        .unwrap_or(DEFAULT_GUI_READER_LINES_PER_MINUTE as i16);
    let next = current.saturating_add(delta).clamp(
        MIN_GUI_READER_LINES_PER_MINUTE as i16,
        MAX_GUI_READER_LINES_PER_MINUTE as i16,
    ) as u16;
    runtime.settings.gui_reader_lines_per_minute = next;
    runtime.status = format!("Reader speed: {next} lines/min");
    persist_runtime_settings(runtime);
}

pub(crate) fn stop_reader_mode(runtime: &mut EditorRuntime, status: impl Into<String>) {
    if runtime.settings.gui_reader_mode_enabled {
        runtime.settings.gui_reader_mode_enabled = false;
        runtime.reader_scroll_milli_lines = 0;
        runtime.status = status.into();
        persist_runtime_settings(runtime);
    }
}

pub(crate) fn stop_reader_mode_for_edit(runtime: &mut EditorRuntime) {
    stop_reader_mode(runtime, "Reader mode stopped for edit");
}

pub(crate) fn apply_reader_tick(
    document: &TextDocument,
    tab_state: &mut EditorTabState,
    runtime: &mut EditorRuntime,
    visible_rows: usize,
) -> bool {
    if !runtime.settings.gui_reader_mode_enabled {
        return false;
    }

    let line_count = document.buffer.line_count().max(1);
    let max_start = line_count.saturating_sub(visible_rows.max(1));
    if tab_state.viewport_start >= max_start {
        stop_reader_mode(runtime, "Reader mode stopped at document end");
        return true;
    }

    let speed = u32::from(runtime.settings.gui_reader_lines_per_minute.max(1));
    let milli_lines_per_tick = speed
        .saturating_mul(TUI_READER_TICK_MS as u32)
        .saturating_mul(1000)
        / 60_000;
    runtime.reader_scroll_milli_lines = runtime
        .reader_scroll_milli_lines
        .saturating_add(milli_lines_per_tick.max(1));
    let lines = (runtime.reader_scroll_milli_lines / 1000) as usize;
    if lines == 0 {
        return false;
    }

    runtime.reader_scroll_milli_lines %= 1000;
    let next_start = tab_state
        .viewport_start
        .saturating_add(lines)
        .min(max_start);
    if next_start == tab_state.viewport_start {
        stop_reader_mode(runtime, "Reader mode stopped at document end");
        return true;
    }

    tab_state.viewport_start = next_start;
    runtime.status = format!(
        "Reader mode: {} lines/min",
        runtime.settings.gui_reader_lines_per_minute
    );
    true
}
