//! Synchronized editor viewport, cursor, selection, and status updates.

use super::*;

impl KfnotepadGui {
    pub(in crate::gui::app::state) fn update_replacement_editor_view_state(
        &mut self,
        pane: pane_grid::Pane,
        cursor: DocumentCursor,
        viewport: GuiEditorViewportState,
        replacement_selection: Option<GuiEditorReplacementSelection>,
        status: &str,
    ) {
        if let Some(pane_state) = self.panes.get_mut(pane) {
            pane_state.editor.move_to(cursor);
            pane_state.editor.viewport = viewport;
            pane_state.editor.replacement_selection = replacement_selection;
        }
        self.search_highlight = None;
        self.pending_close_tile = None;
        self.pending_app_quit = false;
        self.pending_project_open = None;
        self.status_message = status.to_string();
    }
}
