impl GuiWorkspace {
    pub fn from_document(document: TextDocument) -> Self {
        let first_id = GuiTileId(0);
        Self {
            tiles: vec![GuiDocumentTile {
                id: first_id,
                document,
                state: EditorTabState::default(),
                minimized: false,
                last_save_error: None,
            }],
            active: first_id,
            focused: first_id,
            pending_layout_intent: None,
            next_tile_id: 1,
        }
    }

    pub fn active_tile(&self) -> &GuiDocumentTile {
        self.tile(self.active)
            .expect("active GUI tile id must refer to an existing tile")
    }

    pub fn active_tile_mut(&mut self) -> &mut GuiDocumentTile {
        self.tile_mut(self.active)
            .expect("active GUI tile id must refer to an existing tile")
    }

    pub fn focused_tile(&self) -> &GuiDocumentTile {
        self.tile(self.focused)
            .expect("focused GUI tile id must refer to an existing tile")
    }

    pub fn tile(&self, tile_id: GuiTileId) -> Option<&GuiDocumentTile> {
        self.tiles.iter().find(|tile| tile.id == tile_id)
    }

    pub fn tile_mut(&mut self, tile_id: GuiTileId) -> Option<&mut GuiDocumentTile> {
        self.tiles.iter_mut().find(|tile| tile.id == tile_id)
    }
}
