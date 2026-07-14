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
        self.invalidate_syntax_cache(tile_id);
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
        let invalidates_syntax = gui_replacement_inputs_invalidate_syntax(&inputs);
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
        for input in inputs.iter() {
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
        if invalidates_syntax {
            self.invalidate_syntax_cache(tile_id);
        }
        self.ensure_visible_syntax_cache_for_tile(tile_id);
        self.pending_close_tile = None;
        self.pending_app_quit = false;
        self.pending_project_open = None;
        self.status_message = "replacement edit".to_string();
    }
}
