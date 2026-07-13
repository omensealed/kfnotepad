//! Validated Go To Line execution and editor synchronization.

use super::*;

impl KfnotepadGui {
    pub(in crate::gui::app::state) fn go_active_line(&mut self) {
        self.sync_active_editor_to_document();
        let Some(tile_id) = self
            .panes
            .get(self.active_pane)
            .map(|pane_state| pane_state.tile_id)
        else {
            self.status_message = "go to line failed: no active pane".to_string();
            return;
        };
        let Some(tile) = self.workspace.tile_mut(tile_id) else {
            self.status_message = "go to line failed: no active tile".to_string();
            return;
        };

        let result = go_to_line(
            &tile.document,
            &mut tile.state.cursor,
            self.go_to_line_query.trim(),
        );
        self.status_message = go_to_line_status(result);
        self.move_active_editor_to_document_cursor();
    }
}
