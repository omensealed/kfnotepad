impl KfnotepadGui {
    pub(super) fn focus_pane(&mut self, pane: pane_grid::Pane) -> bool {
        let Some(tile_id) = self.panes.get(pane).map(|pane_state| pane_state.tile_id) else {
            return false;
        };
        self.active_pane = pane;
        if self.panes.maximized().is_some() && self.panes.maximized() != Some(pane) {
            self.panes.restore();
            self.panes.maximize(pane);
        }
        self.workspace.focus_tile(tile_id)
    }

    pub(super) fn sync_pane_to_document(&mut self, pane: pane_grid::Pane) {
        let _ = self.sync_pane_to_document_text(pane);
    }

    pub(super) fn sync_pane_to_document_text(
        &mut self,
        pane: pane_grid::Pane,
    ) -> Option<(GuiTileId, String)> {
        let pane_state = self.panes.get(pane)?;
        let text = pane_state.editor.text();
        let tile_id = pane_state.tile_id;
        if let Some(tile) = self.workspace.tile_mut(tile_id) {
            tile.document.buffer.replace_text(&text);
            tile.state.cursor = pane_state.editor.document_cursor();
        }
        Some((tile_id, text))
    }

    pub(super) fn sync_pane_cursor_to_document(&mut self, pane: pane_grid::Pane) {
        let Some(pane_state) = self.panes.get(pane) else {
            return;
        };
        let tile_id = pane_state.tile_id;
        if let Some(tile) = self.workspace.tile_mut(tile_id) {
            tile.state.cursor = pane_state.editor.document_cursor();
        }
    }

    pub(super) fn sync_active_editor_to_document(&mut self) {
        self.sync_pane_to_document(self.active_pane);
    }

    pub(super) fn sync_active_editor_to_document_text(&mut self) -> Option<(GuiTileId, String)> {
        self.sync_pane_to_document_text(self.active_pane)
    }

    pub(super) fn perform_active_editor_command(
        &mut self,
        command: GuiEditorCommand,
        status: &str,
    ) {
        let invalidates_syntax = gui_editor_command_invalidates_syntax(&command);
        let may_extend_syntax = gui_editor_command_may_extend_syntax_cache(&command);
        let mutates_text = gui_editor_command_mutates_text(&command);
        if let Some(pane_state) = self.panes.get_mut(self.active_pane) {
            pane_state.editor.apply(command);
        }
        if mutates_text {
            self.sync_active_editor_to_document();
        } else {
            self.sync_pane_cursor_to_document(self.active_pane);
        }
        if let Some(tile_id) = self
            .panes
            .get(self.active_pane)
            .map(|pane_state| pane_state.tile_id)
        {
            self.workspace.clear_tile_save_error(tile_id);
            if invalidates_syntax {
                self.invalidate_syntax_cache(tile_id);
                self.ensure_visible_syntax_cache_for_tile(tile_id);
            } else if may_extend_syntax {
                self.ensure_visible_syntax_cache_for_tile(tile_id);
            }
        }
        self.pending_close_tile = None;
        self.pending_app_quit = false;
        self.pending_project_open = None;
        self.status_message = status.to_string();
    }

    pub(super) fn active_editor_selection(&self) -> Option<String> {
        self.panes
            .get(self.active_pane)
            .and_then(|pane_state| pane_state.editor.selection())
            .filter(|selection| !selection.is_empty())
    }
}
