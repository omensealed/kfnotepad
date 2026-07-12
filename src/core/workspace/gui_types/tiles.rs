//! GUI document tile state, open errors, close results, and save status.

use super::*;

#[derive(Debug, PartialEq, Eq)]
pub enum GuiCloseTileResult {
    Missing,
    OnlyTile,
    Dirty { tile_id: GuiTileId },
    Closed { tile_id: GuiTileId, path: PathBuf },
}

#[derive(Debug)]
pub enum GuiTileOpenError {
    Invalid { source: OpenError },
}

pub struct GuiDocumentTile {
    pub id: GuiTileId,
    pub document: TextDocument,
    pub state: EditorTabState,
    pub minimized: bool,
    pub(in crate::core::workspace) last_save_error: Option<String>,
}

impl GuiDocumentTile {
    pub fn save_status(&self) -> GuiTileSaveStatus {
        if let Some(message) = &self.last_save_error {
            return GuiTileSaveStatus::SaveFailed {
                message: message.clone(),
            };
        }
        if self.document.buffer.is_dirty() {
            GuiTileSaveStatus::Modified
        } else {
            GuiTileSaveStatus::Saved
        }
    }
}
