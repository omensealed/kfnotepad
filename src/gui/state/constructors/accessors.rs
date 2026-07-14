//! Frequently used active-editor and browser-directory accessors.

use super::*;

impl KfnotepadGui {
    #[cfg(test)]
    pub(in crate::gui::app::state) fn active_editor(&self) -> &GuiEditorAdapter {
        &self
            .panes
            .get(self.active_pane)
            .expect("active GUI pane must exist")
            .editor
    }

    #[cfg(test)]
    pub(in crate::gui::app::state) fn active_document_text(&self) -> String {
        self.workspace.active_tile().document.buffer.to_text()
    }

    #[cfg(test)]
    pub(in crate::gui::app::state) fn active_editor_selection_text(&self) -> Option<String> {
        self.active_editor_selection()
    }

    #[cfg(test)]
    pub(in crate::gui::app::state) fn replace_active_document_text(&mut self, text: &str) {
        let tile_id = self.workspace.active;
        let Some(tile) = self.workspace.tile_mut(tile_id) else {
            return;
        };
        tile.document.buffer.replace_text(text);
        tile.state.cursor = DocumentCursor { row: 0, column: 0 };
        let line_count = gui_editor_line_count(&tile.document.buffer);
        if let Some(pane_state) = self.panes.get_mut(self.active_pane) {
            pane_state
                .editor
                .sync_document_metadata(line_count, DocumentCursor { row: 0, column: 0 });
        }
    }

    pub(in crate::gui::app::state) fn current_browser_dir(&self) -> PathBuf {
        self.browser
            .as_ref()
            .map(|browser| browser.sidebar.current_dir.clone())
            .unwrap_or_else(|| self.current_dir.clone())
    }
}
