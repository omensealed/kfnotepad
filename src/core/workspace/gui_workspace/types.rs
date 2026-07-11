pub struct GuiWorkspace {
    pub tiles: Vec<GuiDocumentTile>,
    pub active: GuiTileId,
    pub focused: GuiTileId,
    pub pending_layout_intent: Option<GuiTileLayoutIntent>,
    next_tile_id: usize,
}
