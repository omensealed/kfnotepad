impl KfnotepadGui {
    pub(super) fn scroll_active_editor_viewport(&mut self, delta: i32) {
        self.perform_active_editor_command(
            GuiEditorCommand::ScrollViewportLines(delta),
            if delta < 0 {
                "viewport up"
            } else {
                "viewport down"
            },
        );
    }

    pub(super) fn scroll_active_editor_viewport_preserving_cursor(
        &mut self,
        delta: i32,
        status: &str,
    ) {
        if delta == 0 {
            return;
        }
        let Some(tile_id) = self
            .panes
            .get(self.active_pane)
            .map(|pane_state| pane_state.tile_id)
        else {
            return;
        };
        if let Some(pane_state) = self.panes.get_mut(self.active_pane) {
            pane_state
                .editor
                .scroll_viewport_by_lines_preserving_cursor(delta);
        }
        self.ensure_visible_syntax_cache_for_tile(tile_id);
        self.status_message = status.to_string();
    }

    pub(super) fn reader_scroll_tick(&mut self) {
        if !self.settings.gui_reader_mode_enabled {
            return;
        }
        let Some(tile_id) = self
            .panes
            .get(self.active_pane)
            .map(|pane_state| pane_state.tile_id)
        else {
            self.set_reader_mode_enabled(false);
            return;
        };
        let Some(tile) = self.workspace.tile(tile_id) else {
            self.set_reader_mode_enabled(false);
            return;
        };
        let Some(pane_state) = self.panes.get(self.active_pane) else {
            self.set_reader_mode_enabled(false);
            return;
        };

        let line_count = tile.document.buffer.line_count().max(1);
        if pane_state.editor.viewport.first_line >= line_count {
            self.set_reader_mode_enabled(false);
            self.status_message = "reader mode stopped at document end".to_string();
            return;
        }

        let lines_per_tick = f32::from(self.settings.gui_reader_lines_per_minute)
            * GUI_READER_TICK_MS as f32
            / 60_000.0;
        self.reader_scroll_accumulator += lines_per_tick;
        let lines = self.reader_scroll_accumulator.floor() as i32;
        if lines <= 0 {
            return;
        }
        self.reader_scroll_accumulator -= lines as f32;
        self.scroll_active_editor_viewport_preserving_cursor(lines, "reader mode");
        self.status_message = format!(
            "reader mode: {} lines/min",
            self.settings.gui_reader_lines_per_minute
        );
    }

    pub(super) fn scroll_replacement_editor_pane_viewport(
        &mut self,
        pane: pane_grid::Pane,
        delta: i32,
    ) {
        if delta == 0 || !self.focus_pane(pane) {
            return;
        }
        self.scroll_active_editor_viewport_preserving_cursor(
            delta,
            if delta < 0 {
                "viewport up"
            } else {
                "viewport down"
            },
        );
    }
}
