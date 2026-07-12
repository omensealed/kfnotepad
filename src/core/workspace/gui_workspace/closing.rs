//! Dirty-aware tile closing and focus fallback behavior.

use super::*;

impl GuiWorkspace {
    pub fn close_tile(&mut self, tile_id: GuiTileId, confirm_dirty: bool) -> GuiCloseTileResult {
        let Some(index) = self.tiles.iter().position(|tile| tile.id == tile_id) else {
            return GuiCloseTileResult::Missing;
        };
        if self.tiles.len() <= 1 {
            return GuiCloseTileResult::OnlyTile;
        }
        if self.tiles[index].document.buffer.is_dirty() && !confirm_dirty {
            return GuiCloseTileResult::Dirty { tile_id };
        }

        let removed = self.tiles.remove(index);
        if self.active == tile_id || self.focused == tile_id {
            let fallback_index = index.min(self.tiles.len().saturating_sub(1));
            let fallback_id = self.tiles[fallback_index].id;
            self.active = fallback_id;
            self.focused = fallback_id;
        }
        GuiCloseTileResult::Closed {
            tile_id,
            path: removed.document.path,
        }
    }
}
