//! Completion handling for saves to an existing document path.

use super::*;

impl KfnotepadGui {
    pub(in crate::gui::app::state) fn apply_save_active_tile_completion(
        &mut self,
        tile_id: GuiTileId,
        result: Result<GuiSaveResult, String>,
    ) {
        match result {
            Ok(saved) => {
                let (path, current_revision) = {
                    let Some(tile) = self.workspace.tile_mut(tile_id) else {
                        self.status_message =
                            "save skipped: active tile no longer exists".to_string();
                        return;
                    };

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
                self.ensure_visible_syntax_cache_for_tile(tile_id);
                self.status_message = if current_revision == saved.source_revision {
                    format!("saved {}", gui_file_name_label(&path))
                } else {
                    "save completed after edits; save again to persist latest text".to_string()
                };
            }
            Err(error) => {
                self.workspace.mark_tile_save_failed(tile_id, error.clone());
                self.status_message = format!("save failed: {error}");
            }
        }
    }
}
