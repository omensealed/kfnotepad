impl KfnotepadGui {
    pub(in crate::gui::app::state) fn apply_save_active_tile_as_completion(
        &mut self,
        tile_id: GuiTileId,
        requested_path: PathBuf,
        result: Result<GuiSaveResult, String>,
    ) {
        match result {
            Ok(saved) => {
                let (path, current_revision) = {
                    let Some(tile) = self.workspace.tile_mut(tile_id) else {
                        self.status_message =
                            "save as skipped: active tile no longer exists".to_string();
                        return;
                    };

                    tile.document.path = requested_path.clone();
                    tile.document
                        .buffer
                        .set_file_snapshot(Some(saved.snapshot.clone()));
                    let current_revision = tile.document.buffer.edit_revision();
                    if current_revision == saved.source_revision {
                        tile.document.buffer.mark_clean();
                        if self.pending_close_tile == Some(tile_id) {
                            self.pending_close_tile = None;
                        }
                        self.pending_app_quit = false;
                    }
                    (tile.document.path.clone(), current_revision)
                };
                self.workspace.clear_tile_save_error(tile_id);
                self.file_snapshots.insert(tile_id, saved.snapshot);
                self.external_edit_locks.remove(&tile_id);
                self.invalidate_syntax_cache(tile_id);
                self.ensure_visible_syntax_cache_for_tile(tile_id);
                self.path_prompt = None;
                self.path_prompt_value.clear();
                self.notes_panel = None;
                self.pending_managed_note_delete = None;
                self.status_message = if current_revision == saved.source_revision {
                    format!("saved as {}", gui_file_name_label(&path))
                } else {
                    "save as completed after edits; save again to persist latest text".to_string()
                };
            }
            Err(error) => {
                if self.workspace.tile(tile_id).is_none() {
                    self.status_message =
                        "save as skipped: active tile no longer exists".to_string();
                    return;
                }
                self.workspace.mark_tile_save_failed(tile_id, error.clone());
                self.status_message = format!("save as failed: {error}");
            }
        }
    }
}
