//! Batched replacement-editor input application and synchronization.

use super::*;

impl KfnotepadGui {
    pub(in crate::gui::app::state) fn apply_replacement_editor_bulk_text_to_active_tile(
        &mut self,
        text: &str,
    ) -> bool {
        if text.is_empty() {
            return false;
        }
        self.replacement_ime_preedit = None;

        let Some(tile_id) = self
            .panes
            .get(self.active_pane)
            .map(|pane_state| pane_state.tile_id)
        else {
            return false;
        };
        let mut viewport = self
            .panes
            .get(self.active_pane)
            .map(|pane_state| pane_state.editor.viewport)
            .unwrap_or_else(|| GuiEditorViewportState::new(GUI_LINE_NUMBER_GUTTER_VISIBLE_LINES));
        let mut replacement_selection = self
            .panes
            .get(self.active_pane)
            .and_then(|pane_state| pane_state.editor.replacement_selection);

        let Some(tile) = self.workspace.tile_mut(tile_id) else {
            return false;
        };
        let first_changed_line = replacement_selection
            .map(|selection| selection.normalized().0.row)
            .unwrap_or(tile.state.cursor.row);
        let changed = gui_editor_replacement_paste_text_with_mode(
            &mut tile.document,
            &mut tile.state.cursor,
            &mut viewport,
            &mut replacement_selection,
            self.replacement_overwrite_mode,
            text,
        );
        if !changed {
            self.status_message = "edit could not be applied".to_string();
            return false;
        }
        let cursor = tile.state.cursor;
        let line_count = gui_editor_line_count(&tile.document.buffer);

        if let Some(pane_state) = self.panes.get_mut(self.active_pane) {
            pane_state.editor.sync_document_metadata(line_count, cursor);
            pane_state.editor.viewport = viewport;
            pane_state.editor.viewport_tracks_cursor = true;
            pane_state.editor.replacement_selection = replacement_selection;
        }
        self.workspace.clear_tile_save_error(tile_id);
        self.invalidate_syntax_cache_from(tile_id, first_changed_line, line_count);
        self.ensure_visible_syntax_cache_for_tile(tile_id);
        self.pending_close_tile = None;
        self.pending_app_quit = false;
        self.pending_project_open = None;
        self.status_message = "replacement edit".to_string();
        true
    }

    pub(in crate::gui::app::state) fn apply_replacement_editor_inputs_to_active_tile(
        &mut self,
        inputs: Vec<GuiEditorReplacementInput>,
    ) {
        if inputs.is_empty() {
            return;
        }
        if self.replacement_overwrite_mode && inputs.len() > 1 {
            let text = inputs
                .iter()
                .map(|input| match input {
                    GuiEditorReplacementInput::InsertChar(value) => Some(*value),
                    _ => None,
                })
                .collect::<Option<String>>();
            if let Some(text) = text {
                self.apply_replacement_editor_bulk_text_to_active_tile(&text);
                return;
            }
        }
        self.replacement_ime_preedit = None;

        let Some(tile_id) = self
            .panes
            .get(self.active_pane)
            .map(|pane_state| pane_state.tile_id)
        else {
            return;
        };
        let mut viewport = self
            .panes
            .get(self.active_pane)
            .map(|pane_state| pane_state.editor.viewport)
            .unwrap_or_else(|| GuiEditorViewportState::new(GUI_LINE_NUMBER_GUTTER_VISIBLE_LINES));
        let mut replacement_selection = self
            .panes
            .get(self.active_pane)
            .and_then(|pane_state| pane_state.editor.replacement_selection);

        let Some(tile) = self.workspace.tile_mut(tile_id) else {
            return;
        };
        let mut first_changed_line = None;
        for input in inputs.iter() {
            if let Some(candidate) = gui_replacement_input_syntax_start_line(
                *input,
                tile.state.cursor,
                replacement_selection,
            ) {
                first_changed_line =
                    Some(first_changed_line.map_or(candidate, |line: usize| line.min(candidate)));
            }
            apply_gui_editor_replacement_input_with_mode(
                &mut tile.document,
                &mut tile.state.cursor,
                &mut viewport,
                &mut replacement_selection,
                self.replacement_overwrite_mode,
                *input,
            );
        }
        let cursor = tile.state.cursor;
        let line_count = gui_editor_line_count(&tile.document.buffer);
        if let Some(pane_state) = self.panes.get_mut(self.active_pane) {
            pane_state.editor.sync_document_metadata(line_count, cursor);
            pane_state.editor.viewport = viewport;
            pane_state.editor.viewport_tracks_cursor = true;
            pane_state.editor.replacement_selection = replacement_selection;
        }
        self.workspace.clear_tile_save_error(tile_id);
        if let Some(first_changed_line) = first_changed_line {
            self.invalidate_syntax_cache_from(tile_id, first_changed_line, line_count);
        }
        self.ensure_visible_syntax_cache_for_tile(tile_id);
        self.pending_close_tile = None;
        self.pending_app_quit = false;
        self.pending_project_open = None;
        self.status_message = "replacement edit".to_string();
    }
}
