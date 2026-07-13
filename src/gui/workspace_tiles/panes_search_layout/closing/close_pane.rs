//! Pane close orchestration and dirty-buffer confirmation.

use super::*;

impl KfnotepadGui {
    pub(in crate::gui::app::state) fn close_pane(&mut self, pane: pane_grid::Pane) {
        self.sync_pane_to_document(pane);
        let Some(tile_id) = self.panes.get(pane).map(|pane_state| pane_state.tile_id) else {
            self.status_message = "close failed: no such pane".to_string();
            return;
        };

        let confirm_dirty = self.pending_close_tile == Some(tile_id);
        if self.workspace.tiles.len() <= 1 {
            self.close_last_tile(pane, tile_id, confirm_dirty);
            return;
        }

        match self.workspace.close_tile(tile_id, confirm_dirty) {
            GuiCloseTileResult::Missing => {
                self.pending_close_tile = None;
                self.status_message = "close failed: no such tile".to_string();
            }
            GuiCloseTileResult::OnlyTile => {
                self.pending_close_tile = None;
                self.close_last_tile(pane, tile_id, confirm_dirty);
            }
            GuiCloseTileResult::Dirty { tile_id } => {
                self.pending_close_tile = Some(tile_id);
                self.focus_pane(pane);
                self.status_message =
                    "unsaved changes; close again to discard this tile".to_string();
            }
            GuiCloseTileResult::Closed { tile_id, path } => {
                self.file_snapshots.remove(&tile_id);
                self.external_edit_locks.remove(&tile_id);
                self.invalidate_syntax_cache(tile_id);
                self.pending_close_tile = None;
                self.pending_app_quit = false;
                if self.panes.len() <= 1 && !self.minimized_panes.is_empty() {
                    if self.restore_first_minimized_into_pane(pane).is_some() {
                        self.status_message = format!("closed {}", path.display());
                        self.persist_layout();
                    } else {
                        self.status_message =
                            format!("closed {} but pane replacement failed", path.display());
                    }
                } else if let Some((_closed, fallback_pane)) = self.panes.close(pane) {
                    self.active_pane = fallback_pane;
                    if let Some(fallback_tile) = self
                        .panes
                        .get(fallback_pane)
                        .map(|pane_state| pane_state.tile_id)
                    {
                        self.workspace.focus_tile(fallback_tile);
                    }
                    self.status_message = format!("closed {}", path.display());
                    self.persist_layout();
                } else {
                    self.status_message =
                        format!("closed {} but pane removal failed", path.display());
                }
            }
        }
    }
}
