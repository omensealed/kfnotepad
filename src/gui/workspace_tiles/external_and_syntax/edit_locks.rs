//! External-change edit lock queries and explicit unlock behavior.

use super::*;

impl KfnotepadGui {
    pub(in crate::gui::app::state) fn is_external_edit_locked(&self, tile_id: GuiTileId) -> bool {
        self.external_edit_locks.contains(&tile_id)
    }

    pub(in crate::gui::app::state) fn unlock_external_edit(&mut self, tile_id: GuiTileId) {
        if self.external_edit_locks.remove(&tile_id) {
            self.status_message = "external edit lock cleared".to_string();
        }
    }
}
