//! Split, move, and resize layout-intent tracking.

use super::*;

impl GuiWorkspace {
    pub fn request_split(&mut self, tile_id: GuiTileId, direction: GuiSplitDirection) -> bool {
        if self.tile(tile_id).is_none() {
            return false;
        }
        self.pending_layout_intent = Some(GuiTileLayoutIntent::Split { tile_id, direction });
        true
    }

    pub fn request_move(&mut self, tile_id: GuiTileId, direction: GuiTileMoveDirection) -> bool {
        if self.tile(tile_id).is_none() {
            return false;
        }
        self.pending_layout_intent = Some(GuiTileLayoutIntent::Move { tile_id, direction });
        true
    }

    pub fn request_resize(
        &mut self,
        tile_id: GuiTileId,
        direction: GuiTileResizeDirection,
    ) -> bool {
        if self.tile(tile_id).is_none() {
            return false;
        }
        self.pending_layout_intent = Some(GuiTileLayoutIntent::Resize { tile_id, direction });
        true
    }

    pub fn clear_layout_intent(&mut self) {
        self.pending_layout_intent = None;
    }
}
