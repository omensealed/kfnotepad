//! Clean external document replacement across visible and minimized panes.

use super::*;

impl KfnotepadGui {
    pub(in crate::gui::app::state) fn replace_tile_document_from_external_change(
        &mut self,
        tile_id: GuiTileId,
        mut document: TextDocument,
    ) {
        document.buffer.mark_clean();
        let line_count = gui_editor_line_count(&document.buffer);
        if let Some(tile) = self.workspace.tile_mut(tile_id) {
            tile.document = document;
            tile.state.cursor = DocumentCursor { row: 0, column: 0 };
            tile.state.viewport_start = 0;
            tile.state.horizontal_offset = 0;
        }
        for (_pane, pane_state) in self.panes.iter_mut() {
            if pane_state.tile_id == tile_id {
                pane_state.editor =
                    GuiEditorAdapter::new(line_count, DocumentCursor { row: 0, column: 0 });
            }
        }
        for pane_state in &mut self.minimized_panes {
            if pane_state.tile_id == tile_id {
                pane_state.editor =
                    GuiEditorAdapter::new(line_count, DocumentCursor { row: 0, column: 0 });
            }
        }
        self.invalidate_syntax_cache(tile_id);
        self.ensure_visible_syntax_cache_for_tile(tile_id);
    }
}
