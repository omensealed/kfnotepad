impl KfnotepadGui {
    pub(super) fn apply_replacement_editor_inputs_to_active_tile(
        &mut self,
        inputs: Vec<GuiEditorReplacementInput>,
    ) {
        if inputs.is_empty() {
            return;
        }
        let invalidates_syntax = gui_replacement_inputs_invalidate_syntax(&inputs);
        let mutates_text = gui_replacement_inputs_mutates_text(&inputs);
        self.replacement_ime_preedit = None;

        let Some(tile_id) = self
            .panes
            .get(self.active_pane)
            .map(|pane_state| pane_state.tile_id)
        else {
            return;
        };
        let initial_replacement_selection = self
            .panes
            .get(self.active_pane)
            .and_then(|pane_state| pane_state.editor.replacement_selection);
        let mut viewport = self
            .panes
            .get(self.active_pane)
            .map(|pane_state| pane_state.editor.viewport)
            .unwrap_or_else(|| GuiEditorViewportState::new(GUI_LINE_NUMBER_GUTTER_VISIBLE_LINES));
        let mut replacement_selection = self
            .panes
            .get(self.active_pane)
            .and_then(|pane_state| pane_state.editor.replacement_selection);

        let Some(tile) = self.workspace.tile_mut(tile_id) else {
            return;
        };
        for input in inputs.iter() {
            apply_gui_editor_replacement_input_with_mode(
                &mut tile.document,
                &mut tile.state.cursor,
                &mut viewport,
                &mut replacement_selection,
                self.replacement_overwrite_mode,
                *input,
            );
        }
        let cursor = tile.state.cursor;
        if let Some(pane_state) = self.panes.get_mut(self.active_pane) {
            if mutates_text {
                let can_sync_editor = !self.replacement_overwrite_mode
                    && initial_replacement_selection.is_none()
                    && replacement_selection.is_none()
                    && inputs
                        .iter()
                        .all(Self::gui_editor_replacement_input_has_editor_delta_binding);
                if can_sync_editor {
                    for input in inputs.iter() {
                        Self::gui_editor_replacement_input_apply_delta_to_editor(
                            &mut pane_state.editor,
                            input,
                        );
                    }
                } else {
                    let text = tile.document.buffer.to_text();
                    pane_state.editor =
                        GuiEditorAdapter::new(text_editor::Content::with_text(&text));
                }
                pane_state.editor.move_to(cursor);
            } else {
                pane_state.editor.move_to(cursor);
            }
            pane_state.editor.viewport = viewport;
            pane_state.editor.viewport_tracks_cursor = true;
            pane_state.editor.replacement_selection = replacement_selection;
        }
        self.workspace.clear_tile_save_error(tile_id);
        if invalidates_syntax {
            self.invalidate_syntax_cache(tile_id);
        }
        self.ensure_visible_syntax_cache_for_tile(tile_id);
        self.pending_close_tile = None;
        self.pending_app_quit = false;
        self.pending_project_open = None;
        self.status_message = "replacement edit".to_string();
    }
}
