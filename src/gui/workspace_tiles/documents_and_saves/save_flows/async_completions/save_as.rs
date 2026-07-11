impl KfnotepadGui {
    pub(super) fn apply_save_active_tile_as_completion(
        &mut self,
        tile_id: GuiTileId,
        original_path: PathBuf,
        requested_path: PathBuf,
        expected_text: String,
        clear_snapshot: bool,
        result: Result<(), String>,
    ) {
        match result {
            Ok(()) => {
                let path = {
                    let Some(tile) = self.workspace.tile_mut(tile_id) else {
                        self.status_message =
                            "save as skipped: active tile no longer exists".to_string();
                        return;
                    };

                    let current_text = tile.document.buffer.to_text();
                    if current_text != expected_text {
                        self.workspace.clear_tile_save_error(tile_id);
                        self.status_message =
                            "save as completed after edits; reopen save as to persist latest text"
                                .to_string();
                        return;
                    }

                    tile.document.path = requested_path.clone();
                    if clear_snapshot {
                        tile.document.buffer.set_file_snapshot(None);
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
                self.invalidate_syntax_cache(tile_id);
                self.ensure_visible_syntax_cache_for_tile(tile_id);
                self.path_prompt = None;
                self.path_prompt_value.clear();
                self.notes_panel = None;
                self.pending_managed_note_delete = None;
                self.status_message = format!("saved as {}", gui_file_name_label(&path));
            }
            Err(error) => {
                {
                    let Some(tile) = self.workspace.tile_mut(tile_id) else {
                        self.status_message =
                            "save as skipped: active tile no longer exists".to_string();
                        return;
                    };
                    tile.document.path = original_path;
                    if clear_snapshot {
                        if let Ok(document) = open_text_file(&tile.document.path) {
                            tile.document
                                .buffer
                                .set_file_snapshot(document.buffer.file_snapshot().cloned());
                        } else {
                            tile.document.buffer.set_file_snapshot(None);
                        }
                    }
                }
                self.workspace.mark_tile_save_failed(tile_id, error.clone());
                self.status_message = format!("save as failed: {error}");
            }
        }
    }
}
