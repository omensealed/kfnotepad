//! GUI workspace tile collection, focus state, and pending layout intent.

use super::*;

pub struct GuiWorkspace {
    pub tiles: Vec<GuiDocumentTile>,
    pub active: GuiTileId,
    pub focused: GuiTileId,
    pub pending_layout_intent: Option<GuiTileLayoutIntent>,
    pub(super) next_tile_id: usize,
}
