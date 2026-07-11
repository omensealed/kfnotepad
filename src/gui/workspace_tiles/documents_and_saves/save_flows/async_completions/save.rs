impl KfnotepadGui {
    pub(super) fn apply_save_active_tile_completion(
        &mut self,
        tile_id: GuiTileId,
        expected_text: String,
        result: Result<(), String>,
    ) {
        match result {
            Ok(()) => {
                let path = {
                    let Some(tile) = self.workspace.tile_mut(tile_id) else {
                        self.status_message =
                            "save skipped: active tile no longer exists".to_string();
                        return;
                    };

                    let current_text = tile.document.buffer.to_text();
                    if current_text != expected_text {
                        self.workspace.clear_tile_save_error(tile_id);
                        self.status_message =
                            "save completed after edits; reopen save to persist latest text"
                                .to_string();
                        return;
                    }

                    if let Ok(document) = open_text_file(&tile.document.path) {
                        tile.document
                            .buffer
                            .set_file_snapshot(document.buffer.file_snapshot().cloned());
                    }

                    tile.document.buffer.mark_clean();
                    if self.pending_close_tile == Some(tile_id) {
                        self.pending_close_tile = None;
                    }
                    self.pending_app_quit = false;
                    tile.document.path.clone()
                };
                self.workspace.clear_tile_save_error(tile_id);
                self.external_edit_locks.remove(&tile_id);
                self.refresh_file_snapshot_for_tile(tile_id);
                self.ensure_visible_syntax_cache_for_tile(tile_id);
                self.status_message = format!("saved {}", gui_file_name_label(&path));
            }
            Err(error) => {
                self.workspace.mark_tile_save_failed(tile_id, error.clone());
                self.status_message = format!("save failed: {error}");
            }
        }
    }
}
