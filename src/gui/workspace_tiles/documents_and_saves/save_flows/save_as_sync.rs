//! Test-oriented synchronous Save As flow.

use super::*;

impl KfnotepadGui {
    #[cfg(test)]
    pub(in crate::gui::app::state) fn save_active_tile_as(&mut self, path: PathBuf) -> bool {
        self.sync_active_editor_to_document();
        let Some(tile_id) = self
            .panes
            .get(self.active_pane)
            .map(|pane_state| pane_state.tile_id)
        else {
            self.status_message = "save as failed: no active pane".to_string();
            return false;
        };

        if let Some(open_tile_id) = self.open_tile_id_for_path(&path) {
            if open_tile_id != tile_id {
                self.focus_or_restore_existing_tile(open_tile_id, &path);
                self.status_message = format!(
                    "save as refused: {} is already open in another tile",
                    path.display()
                );
                return false;
            }
        }

        let result = {
            let Some(tile) = self.workspace.tile_mut(tile_id) else {
                self.status_message = "save as failed: no active tile".to_string();
                return false;
            };
            let original_path = tile.document.path.clone();
            let original_snapshot = tile.document.buffer.file_snapshot().cloned();
            tile.document.path = path.clone();
            if !gui_paths_refer_to_same_file(&original_path, &path) {
                tile.document.buffer.set_file_snapshot(None);
            }
            match save_text_document(&mut tile.document) {
                Ok(()) => Ok(()),
                Err(error) => {
                    tile.document.path = original_path;
                    tile.document.buffer.set_file_snapshot(original_snapshot);
                    Err(error)
                }
            }
        };

        match result {
            Ok(()) => {
                self.workspace.clear_tile_save_error(tile_id);
                self.pending_close_tile = None;
                self.pending_app_quit = false;
                self.external_edit_locks.remove(&tile_id);
                self.refresh_file_snapshot_for_tile(tile_id);
                self.invalidate_syntax_cache(tile_id);
                self.ensure_visible_syntax_cache_for_tile(tile_id);
                self.status_message = format!("saved as {}", path.display());
                true
            }
            Err(error) => {
                let message = error.to_string();
                self.workspace
                    .mark_tile_save_failed(tile_id, message.clone());
                self.status_message = format!("save as failed: {message}");
                false
            }
        }
    }
}
