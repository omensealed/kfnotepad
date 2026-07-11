impl KfnotepadGui {
    pub(super) fn save_active_tile(&mut self) {
        self.sync_active_editor_to_document();
        let Some(tile_id) = self
            .panes
            .get(self.active_pane)
            .map(|pane_state| pane_state.tile_id)
        else {
            self.status_message = "save failed: no active pane".to_string();
            return;
        };
        let result = {
            let Some(tile) = self.workspace.tile_mut(tile_id) else {
                self.status_message = "save failed: no active tile".to_string();
                return;
            };
            save_text_document(&mut tile.document)
        };

        match result {
            Ok(()) => {
                self.workspace.clear_tile_save_error(tile_id);
                if self.pending_close_tile == Some(tile_id) {
                    self.pending_close_tile = None;
                }
                self.pending_app_quit = false;
                self.external_edit_locks.remove(&tile_id);
                self.refresh_file_snapshot_for_tile(tile_id);
                self.ensure_visible_syntax_cache_for_tile(tile_id);
                self.status_message = format!(
                    "saved {}",
                    self.workspace.active_tile().document.path.display()
                );
            }
            Err(error) => {
                let message = error.to_string();
                self.workspace
                    .mark_tile_save_failed(tile_id, message.clone());
                self.status_message = format!("save failed: {message}");
            }
        }
    }
}
