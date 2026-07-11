impl KfnotepadGui {
    pub(super) fn replace_initial_blank_tile(
        &mut self,
        document: TextDocument,
        opened_status: String,
    ) -> bool {
        if !self.active_tile_is_replaceable_blank() {
            return false;
        }

        let Some(pane_state) = self.panes.get_mut(self.active_pane) else {
            return false;
        };
        let tile_id = pane_state.tile_id;
        let editor = text_editor::Content::with_text(&document.buffer.to_text());
        let Some(tile) = self.workspace.tile_mut(tile_id) else {
            return false;
        };

        tile.document = document;
        tile.state = EditorTabState::default();
        tile.minimized = false;
        self.workspace.focus_tile(tile_id);
        pane_state.editor = GuiEditorAdapter::new(editor);
        self.pending_close_tile = None;
        self.pending_app_quit = false;
        self.pending_project_open = None;
        self.status_message = opened_status;
        self.external_edit_locks.remove(&tile_id);
        self.refresh_file_snapshot_for_tile(tile_id);
        self.invalidate_syntax_cache(tile_id);
        self.ensure_visible_syntax_cache_for_tile(tile_id);
        self.persist_last_workspace_if_enabled();
        true
    }

    pub(super) fn active_tile_is_replaceable_blank(&self) -> bool {
        if self.workspace.tiles.len() != 1 || self.panes.len() != 1 {
            return false;
        }
        let Some(pane_state) = self.panes.get(self.active_pane) else {
            return false;
        };
        let Some(tile) = self.workspace.tile(pane_state.tile_id) else {
            return false;
        };
        let is_untitled = tile
            .document
            .path
            .file_name()
            .and_then(|name| name.to_str())
            .is_some_and(|name| name == "untitled.txt");

        is_untitled && !tile.document.buffer.is_dirty() && tile.document.buffer.to_text().is_empty()
    }
}
