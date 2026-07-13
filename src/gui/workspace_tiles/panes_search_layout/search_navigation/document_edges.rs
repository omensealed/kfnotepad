impl KfnotepadGui {
    pub(in crate::gui::app::state) fn go_active_document_start(&mut self) {
        self.sync_active_editor_to_document();
        let Some(tile_id) = self
            .panes
            .get(self.active_pane)
            .map(|pane_state| pane_state.tile_id)
        else {
            self.status_message = "navigation failed: no active pane".to_string();
            return;
        };
        let Some(tile) = self.workspace.tile_mut(tile_id) else {
            self.status_message = "navigation failed: no active tile".to_string();
            return;
        };
        go_to_document_start(&mut tile.state.cursor);
        self.move_active_editor_to_document_cursor();
        self.status_message = "moved to document start".to_string();
    }

    pub(in crate::gui::app::state) fn go_active_document_end(&mut self) {
        self.sync_active_editor_to_document();
        let Some(tile_id) = self
            .panes
            .get(self.active_pane)
            .map(|pane_state| pane_state.tile_id)
        else {
            self.status_message = "navigation failed: no active pane".to_string();
            return;
        };
        let Some(tile) = self.workspace.tile_mut(tile_id) else {
            self.status_message = "navigation failed: no active tile".to_string();
            return;
        };
        go_to_document_end(&tile.document, &mut tile.state.cursor);
        self.move_active_editor_to_document_cursor();
        self.status_message = "moved to document end".to_string();
    }
}
