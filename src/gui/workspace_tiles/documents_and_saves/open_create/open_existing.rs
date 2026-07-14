//! Existing document opening, duplicate detection, and pane focus restoration.

use super::*;

impl KfnotepadGui {
    pub(in crate::gui::app::state) fn open_document_in_new_pane(
        &mut self,
        document: TextDocument,
        opened_status: String,
    ) -> bool {
        if self.active_tile_is_replaceable_blank() {
            return self.replace_initial_blank_tile(document, opened_status);
        }
        if let Some(tile_id) = self.open_tile_id_for_path(&document.path) {
            self.focus_or_restore_existing_tile(tile_id, &document.path);
            return true;
        }

        let line_count = gui_editor_line_count(&document.buffer);
        let tile_id = self.workspace.open_tile(document);
        let split_axis = split_axis_for_pane(&self.panes, self.active_pane);
        let was_maximized = self.panes.maximized().is_some();
        if let Some((pane, _split)) = self.panes.split(
            split_axis,
            self.active_pane,
            GuiPane::new(tile_id, line_count, DocumentCursor { row: 0, column: 0 }),
        ) {
            self.active_pane = pane;
            self.workspace.focus_tile(tile_id);
            if was_maximized {
                self.panes.restore();
                self.panes.maximize(pane);
            }
            self.pending_close_tile = None;
            self.pending_app_quit = false;
            self.pending_project_open = None;
            self.status_message = opened_status;
            self.external_edit_locks.remove(&tile_id);
            self.refresh_file_snapshot_for_tile(tile_id);
            self.invalidate_syntax_cache(tile_id);
            self.ensure_visible_syntax_cache_for_tile(tile_id);
            self.persist_layout();
            self.persist_last_workspace_if_enabled();
            true
        } else {
            self.status_message = "cannot open document: pane split failed".to_string();
            false
        }
    }

    pub(in crate::gui::app::state) fn open_help_document(&mut self) {
        self.show_startup_help_panel = false;
        let path = self.current_browser_dir().join(GUI_HELP_DOCUMENT_PATH);
        let document = TextDocument {
            path: path.clone(),
            buffer: TextBuffer::from_text(GUI_HELP_DOCUMENT_TEXT),
        };
        self.open_document_in_new_pane(document, format!("opened help {}", path.display()));
    }

    pub(in crate::gui::app::state) fn open_tile_id_for_path(
        &self,
        path: &Path,
    ) -> Option<GuiTileId> {
        self.workspace
            .tiles
            .iter()
            .find(|tile| gui_paths_refer_to_same_file(&tile.document.path, path))
            .map(|tile| tile.id)
    }

    pub(in crate::gui::app::state) fn focus_or_restore_existing_tile(
        &mut self,
        tile_id: GuiTileId,
        path: &Path,
    ) {
        if let Some(pane) = pane_for_tile_id(&self.panes, tile_id) {
            self.focus_pane(pane);
            self.status_message = format!("already open: {}", path.display());
            return;
        }

        if self
            .minimized_panes
            .iter()
            .any(|pane| pane.tile_id == tile_id)
        {
            self.restore_minimized_tile(tile_id);
            self.status_message = format!("restored open file: {}", path.display());
            return;
        }

        self.workspace.focus_tile(tile_id);
        self.status_message = format!("already open: {}", path.display());
    }

    pub(in crate::gui::app::state) fn open_path_in_new_pane(&mut self, path: PathBuf) -> bool {
        match open_text_file(&path) {
            Ok(document) => {
                let opened_path = document.path.clone();
                self.open_document_in_new_pane(
                    document,
                    format!("opened {}", opened_path.display()),
                )
            }
            Err(error) => {
                self.status_message = format!("cannot open {}: {error}", path.display());
                false
            }
        }
    }
}
