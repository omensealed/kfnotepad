impl KfnotepadGui {
    pub(super) fn copy_active_selection(&mut self) -> Task<Message> {
        let Some(selection) = self.active_editor_selection() else {
            self.status_message = "nothing selected".to_string();
            return Task::none();
        };
        self.status_message = "copied selection".to_string();
        clipboard::write(selection)
    }

    pub(super) fn cut_active_selection(&mut self) -> Task<Message> {
        let Some(selection) = self.active_editor_selection() else {
            self.status_message = "nothing selected".to_string();
            return Task::none();
        };
        self.perform_active_editor_command(GuiEditorCommand::Delete, "cut selection");
        clipboard::write(selection)
    }

    pub(super) fn paste_into_active_editor(&mut self, contents: Option<String>) {
        let Some(contents) = contents.filter(|contents| !contents.is_empty()) else {
            self.status_message = "clipboard is empty".to_string();
            return;
        };
        let selected_bytes = self
            .active_editor_selection()
            .map_or(0, |selection| selection.len());
        let tile = self.workspace.active_tile();
        let projected_bytes = tile
            .document
            .buffer
            .byte_len()
            .saturating_sub(selected_bytes)
            .saturating_add(contents.len());
        if let Err(BufferError::TooLarge { limit, .. }) =
            tile.document.buffer.ensure_byte_len(projected_bytes)
        {
            self.status_message = format!("paste exceeds {limit} byte limit");
            return;
        }
        self.perform_active_editor_command(GuiEditorCommand::Paste(contents), "pasted clipboard");
    }

    pub(super) fn select_all_active_editor(&mut self) {
        self.perform_active_editor_command(GuiEditorCommand::SelectAll, "selected all");
    }

    pub(super) fn undo_active_edit(&mut self) {
        self.apply_undo_redo_to_active_tile(true);
    }

    pub(super) fn redo_active_edit(&mut self) {
        self.apply_undo_redo_to_active_tile(false);
    }

    pub(super) fn apply_undo_redo_to_active_tile(&mut self, undo: bool) {
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
