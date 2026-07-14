//! Clipboard commands and shared document undo/redo synchronization.

use super::*;

impl KfnotepadGui {
    pub(in crate::gui::app::state) fn copy_active_selection(&mut self) -> Task<Message> {
        let Some(selection) = self.active_editor_selection() else {
            self.status_message = "nothing selected".to_string();
            return Task::none();
        };
        self.status_message = "copied selection".to_string();
        clipboard::write(selection)
    }

    pub(in crate::gui::app::state) fn cut_active_selection(&mut self) -> Task<Message> {
        let Some(selection) = self.active_editor_selection() else {
            self.status_message = "nothing selected".to_string();
            return Task::none();
        };
        let Some(tile_id) = self
            .panes
            .get(self.active_pane)
            .map(|pane_state| pane_state.tile_id)
        else {
            return Task::none();
        };
        if self.is_external_edit_locked(tile_id) {
            self.status_message = "external edit lock active; unlock to edit".to_string();
            return Task::none();
        }
        if !self.delete_active_replacement_selection() {
            self.status_message = "cut could not be applied".to_string();
            return Task::none();
        }
        self.status_message = "cut selection".to_string();
        clipboard::write(selection)
    }

    fn delete_active_replacement_selection(&mut self) -> bool {
        let Some(tile_id) = self
            .panes
            .get(self.active_pane)
            .map(|pane_state| pane_state.tile_id)
        else {
            return false;
        };
        self.sync_pane_cursor_to_document(self.active_pane);
        let mut viewport = self
            .panes
            .get(self.active_pane)
            .map(|pane_state| pane_state.editor.viewport)
            .unwrap_or_else(|| GuiEditorViewportState::new(GUI_LINE_NUMBER_GUTTER_VISIBLE_LINES));
        let mut selection = self
            .panes
            .get(self.active_pane)
            .and_then(|pane_state| pane_state.editor.replacement_selection);

        let Some(tile) = self.workspace.tile_mut(tile_id) else {
            return false;
        };
        if !delete_gui_editor_replacement_selection(
            &mut tile.document,
            &mut tile.state.cursor,
            &mut selection,
        ) {
            return false;
        }
        let cursor = tile.state.cursor;
        viewport.keep_cursor_visible(cursor, tile.document.buffer.line_count());
        let text = tile.document.buffer.to_text();

        if let Some(pane_state) = self.panes.get_mut(self.active_pane) {
            pane_state.editor = GuiEditorAdapter::new(text_editor::Content::with_text(&text));
            pane_state.editor.move_to(cursor);
            pane_state.editor.viewport = viewport;
            pane_state.editor.viewport_tracks_cursor = true;
            pane_state.editor.replacement_selection = selection;
        }
        self.workspace.clear_tile_save_error(tile_id);
        self.invalidate_syntax_cache(tile_id);
        self.ensure_visible_syntax_cache_for_tile(tile_id);
        self.search_highlight = None;
        self.pending_close_tile = None;
        self.pending_app_quit = false;
        self.pending_project_open = None;
        true
    }

    pub(in crate::gui::app::state) fn paste_into_active_editor(
        &mut self,
        contents: Option<String>,
    ) {
        let Some(contents) = contents.filter(|contents| !contents.is_empty()) else {
            self.status_message = "clipboard is empty".to_string();
            return;
        };
        let Some(tile_id) = self
            .panes
            .get(self.active_pane)
            .map(|pane_state| pane_state.tile_id)
        else {
            return;
        };
        if self.is_external_edit_locked(tile_id) {
            self.status_message = "external edit lock active; unlock to edit".to_string();
            return;
        }
        if self.apply_replacement_editor_bulk_text_to_active_tile(&contents) {
            self.status_message = "pasted clipboard".to_string();
        } else {
            self.status_message = "paste could not be applied".to_string();
        }
    }

    pub(in crate::gui::app::state) fn select_all_active_editor(&mut self) {
        let Some(tile_id) = self
            .panes
            .get(self.active_pane)
            .map(|pane_state| pane_state.tile_id)
        else {
            return;
        };
        let Some(end) = self
            .workspace
            .tile(tile_id)
            .map(|tile| gui_editor_replacement_document_end_cursor(&tile.document.buffer))
        else {
            return;
        };
        if let Some(pane_state) = self.panes.get_mut(self.active_pane) {
            pane_state.editor.set_replacement_selection(
                DocumentCursor { row: 0, column: 0 },
                end,
                end,
            );
        }
        if let Some(tile) = self.workspace.tile_mut(tile_id) {
            tile.state.cursor = end;
        }
        self.status_message = "selected all".to_string();
    }

    pub(in crate::gui::app::state) fn undo_active_edit(&mut self) {
        self.apply_undo_redo_to_active_tile(true);
    }

    pub(in crate::gui::app::state) fn redo_active_edit(&mut self) {
        self.apply_undo_redo_to_active_tile(false);
    }

    pub(in crate::gui::app::state) fn apply_undo_redo_to_active_tile(&mut self, undo: bool) {
        let Some(tile_id) = self
            .panes
            .get(self.active_pane)
            .map(|pane_state| pane_state.tile_id)
        else {
            return;
        };
        if self.is_external_edit_locked(tile_id) {
            self.status_message = "external edit lock active; unlock to edit".to_string();
            return;
        }

        let mut applied = false;
        let mut text = String::new();
        let mut cursor = DocumentCursor { row: 0, column: 0 };
        if let Some(tile) = self.workspace.tile_mut(tile_id) {
            let result = if undo {
                undo_document_edit(&mut tile.document, &mut tile.state.cursor)
            } else {
                redo_document_edit(&mut tile.document, &mut tile.state.cursor)
            };
            applied = result == UndoRedoResult::Applied;
            text = tile.document.buffer.to_text();
            cursor = tile.state.cursor;
        }

        if !applied {
            self.status_message = if undo {
                "nothing to undo".to_string()
            } else {
                "nothing to redo".to_string()
            };
            return;
        }

        if let Some(pane_state) = self.panes.get_mut(self.active_pane) {
            let mut viewport = pane_state.editor.viewport;
            viewport.keep_cursor_visible(cursor, text.lines().count().max(1));
            pane_state.editor = GuiEditorAdapter::new(text_editor::Content::with_text(&text));
            pane_state.editor.move_to(cursor);
            pane_state.editor.viewport = viewport;
            pane_state.editor.replacement_selection = None;
        }
        self.workspace.clear_tile_save_error(tile_id);
        self.invalidate_syntax_cache(tile_id);
        self.ensure_visible_syntax_cache_for_tile(tile_id);
        self.search_highlight = None;
        self.pending_close_tile = None;
        self.pending_app_quit = false;
        self.pending_project_open = None;
        self.status_message = if undo {
            "undo".to_string()
        } else {
            "redo".to_string()
        };
    }
}
