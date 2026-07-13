impl KfnotepadGui {
    pub(in crate::gui::app::state) fn close_active_pane(&mut self) {
        self.close_pane(self.active_pane);
    }

    pub(in crate::gui::app::state) fn visible_tile_count(&self) -> usize {
        self.workspace
            .tiles
            .iter()
            .filter(|tile| !tile.minimized)
            .count()
    }

    pub(in crate::gui::app::state) fn sync_all_panes_to_documents(&mut self) {
        let panes: Vec<_> = self.panes.iter().map(|(pane, _pane_state)| *pane).collect();
        for pane in panes {
            self.sync_pane_to_document(pane);
        }
    }

    pub(in crate::gui::app::state) fn has_dirty_tile(&mut self) -> bool {
        self.sync_all_panes_to_documents();
        self.workspace
            .tiles
            .iter()
            .any(|tile| tile.document.buffer.is_dirty())
    }

    pub(in crate::gui::app::state) fn request_app_close(&mut self, window_id: window::Id) -> Task<Message> {
        if self.has_dirty_tile() {
            if self.pending_app_quit {
                return window::close(window_id);
            }
            self.pending_app_quit = true;
            self.pending_close_tile = None;
            self.status_message =
                "unsaved changes; close window again to discard all dirty tiles".to_string();
            return Task::none();
        }

        window::close(window_id)
    }

    pub(in crate::gui::app::state) fn move_active_editor_to_document_cursor(&mut self) {
        let Some(tile_id) = self
            .panes
            .get(self.active_pane)
            .map(|pane_state| pane_state.tile_id)
        else {
            return;
        };
        let Some(cursor) = self.workspace.tile(tile_id).map(|tile| tile.state.cursor) else {
            return;
        };
        if let Some(pane_state) = self.panes.get_mut(self.active_pane) {
            pane_state.editor.move_to(cursor);
        }
    }
}
