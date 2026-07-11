impl GuiWorkspace {
    pub fn mark_tile_save_failed(
        &mut self,
        tile_id: GuiTileId,
        message: impl Into<String>,
    ) -> bool {
        let Some(tile) = self.tile_mut(tile_id) else {
            return false;
        };
        tile.last_save_error = Some(message.into());
        true
    }

    pub fn clear_tile_save_error(&mut self, tile_id: GuiTileId) -> bool {
        let Some(tile) = self.tile_mut(tile_id) else {
            return false;
        };
        tile.last_save_error = None;
        true
    }
}
