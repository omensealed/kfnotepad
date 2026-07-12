//! Tile opening, validation, focus, and minimization behavior.

use super::*;

impl GuiWorkspace {
    pub fn open_tile(&mut self, document: TextDocument) -> GuiTileId {
        let tile_id = GuiTileId(self.next_tile_id);
        self.next_tile_id += 1;
        self.tiles.push(GuiDocumentTile {
            id: tile_id,
            document,
            state: EditorTabState::default(),
            minimized: false,
            last_save_error: None,
        });
        self.focus_tile(tile_id);
        tile_id
    }

    pub fn open_validated_tile(
        &mut self,
        document: Result<TextDocument, OpenError>,
    ) -> Result<GuiTileId, GuiTileOpenError> {
        match document {
            Ok(document) => Ok(self.open_tile(document)),
            Err(source) => Err(GuiTileOpenError::Invalid { source }),
        }
    }

    pub fn focus_tile(&mut self, tile_id: GuiTileId) -> bool {
        if self.tile(tile_id).is_none() {
            return false;
        }
        self.active = tile_id;
        self.focused = tile_id;
        true
    }

    pub fn set_tile_minimized(&mut self, tile_id: GuiTileId, minimized: bool) -> bool {
        let Some(tile) = self.tile_mut(tile_id) else {
            return false;
        };
        tile.minimized = minimized;
        if !minimized {
            self.focus_tile(tile_id);
        }
        true
    }
}
