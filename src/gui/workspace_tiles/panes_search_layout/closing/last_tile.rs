//! Last-tile close behavior that resets the pane to a blank document.

use super::*;

impl KfnotepadGui {
    pub(in crate::gui::app::state) fn close_last_tile(
        &mut self,
        pane: pane_grid::Pane,
        tile_id: GuiTileId,
        confirm_dirty: bool,
    ) {
        let Some(tile) = self.workspace.tile(tile_id) else {
            self.pending_close_tile = None;
            self.status_message = "close failed: no such tile".to_string();
            return;
        };
        if tile.document.buffer.is_dirty() && !confirm_dirty {
            self.pending_close_tile = Some(tile_id);
            self.focus_pane(pane);
            self.status_message = "unsaved changes; close again to discard this tile".to_string();
            return;
        }

        let blank_dir = self
            .workspace
            .tile(tile_id)
            .and_then(|tile| tile.document.path.parent().map(Path::to_path_buf))
            .unwrap_or_else(|| self.current_browser_dir());
        let path = self.next_untitled_path_in_dir_excluding(blank_dir, Some(tile_id));
        let Some(tile) = self.workspace.tile_mut(tile_id) else {
            self.pending_close_tile = None;
            self.status_message = "close failed: no such tile".to_string();
            return;
        };
        tile.document = TextDocument {
            path: path.clone(),
            buffer: TextBuffer::from_text(""),
        };
        tile.state = EditorTabState::default();
        tile.minimized = false;
        if let Some(pane_state) = self.panes.get_mut(pane) {
            pane_state.editor = GuiEditorAdapter::new(1, DocumentCursor { row: 0, column: 0 });
        }
        self.active_pane = pane;
        self.workspace.focus_tile(tile_id);
        self.pending_close_tile = None;
        self.pending_app_quit = false;
        self.pending_project_open = None;
        self.file_snapshots.remove(&tile_id);
        self.external_edit_locks.remove(&tile_id);
        self.invalidate_syntax_cache(tile_id);
        self.ensure_visible_syntax_cache_for_tile(tile_id);
        self.status_message = format!("new blank tile {}", path.display());
        self.persist_layout();
    }
}
