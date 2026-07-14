//! Pane focus and synchronization between editor and shared document state.

use super::*;

impl KfnotepadGui {
    pub(in crate::gui::app::state) fn focus_pane(&mut self, pane: pane_grid::Pane) -> bool {
        let Some(tile_id) = self.panes.get(pane).map(|pane_state| pane_state.tile_id) else {
            return false;
        };
        self.active_pane = pane;
        if self.panes.maximized().is_some() && self.panes.maximized() != Some(pane) {
            self.panes.restore();
            self.panes.maximize(pane);
        }
        self.workspace.focus_tile(tile_id)
    }

    pub(in crate::gui::app::state) fn sync_pane_to_document(&mut self, pane: pane_grid::Pane) {
        self.sync_pane_cursor_to_document(pane);
    }

    pub(in crate::gui::app::state) fn sync_pane_to_document_text(
        &mut self,
        pane: pane_grid::Pane,
    ) -> Option<(GuiTileId, String)> {
        let pane_state = self.panes.get(pane)?;
        let tile_id = pane_state.tile_id;
        let cursor = pane_state.editor.document_cursor();
        if let Some(tile) = self.workspace.tile_mut(tile_id) {
            tile.state.cursor = cursor;
            return Some((tile_id, tile.document.buffer.to_text()));
        }
        None
    }

    pub(in crate::gui::app::state) fn sync_pane_cursor_to_document(
        &mut self,
        pane: pane_grid::Pane,
    ) {
        let Some(pane_state) = self.panes.get(pane) else {
            return;
        };
        let tile_id = pane_state.tile_id;
        if let Some(tile) = self.workspace.tile_mut(tile_id) {
            tile.state.cursor = pane_state.editor.document_cursor();
        }
    }

    pub(in crate::gui::app::state) fn sync_active_editor_to_document(&mut self) {
        self.sync_pane_to_document(self.active_pane);
    }

    pub(in crate::gui::app::state) fn active_editor_selection(&self) -> Option<String> {
        let pane_state = self.panes.get(self.active_pane)?;
        let selection = pane_state.editor.replacement_selection?;
        let tile = self.workspace.tile(pane_state.tile_id)?;
        gui_editor_replacement_selected_text(&tile.document, selection)
            .filter(|selection| !selection.is_empty())
    }
}
