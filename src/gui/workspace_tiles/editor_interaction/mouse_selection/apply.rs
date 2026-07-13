//! Mouse click and drag application to document cursor and selection state.

use super::*;

impl KfnotepadGui {
    pub(in crate::gui::app::state) fn apply_replacement_editor_mouse_click_to_pane(
        &mut self,
        pane: pane_grid::Pane,
        point: GuiEditorReplacementMousePoint,
    ) {
        if !self.focus_pane(pane) {
            return;
        }
        let Some(tile_id) = self.panes.get(pane).map(|pane_state| pane_state.tile_id) else {
            return;
        };
        let mut viewport = self
            .panes
            .get(pane)
            .map(|pane_state| pane_state.editor.viewport)
            .unwrap_or_else(|| GuiEditorViewportState::new(GUI_LINE_NUMBER_GUTTER_VISIBLE_LINES));
        let mut replacement_selection = self
            .panes
            .get(pane)
            .and_then(|pane_state| pane_state.editor.replacement_selection);

        let Some(tile) = self.workspace.tile_mut(tile_id) else {
            return;
        };
        gui_editor_replacement_mouse_click(
            &tile.document,
            &mut tile.state.cursor,
            &mut viewport,
            &mut replacement_selection,
            point,
        );
        let cursor = tile.state.cursor;
        self.replacement_ime_preedit = None;
        self.update_replacement_editor_view_state(
            pane,
            cursor,
            viewport,
            replacement_selection,
            "cursor moved",
        );
    }

    pub(in crate::gui::app::state) fn apply_replacement_editor_mouse_drag_to_pane(
        &mut self,
        pane: pane_grid::Pane,
        focus: GuiEditorReplacementMousePoint,
    ) {
        if !self.focus_pane(pane) {
            return;
        }
        let Some(drag) = self.replacement_drag else {
            return;
        };
        if drag.pane != pane {
            return;
        }
        let Some(tile_id) = self.panes.get(pane).map(|pane_state| pane_state.tile_id) else {
            return;
        };
        let mut viewport = self
            .panes
            .get(pane)
            .map(|pane_state| pane_state.editor.viewport)
            .unwrap_or_else(|| GuiEditorViewportState::new(GUI_LINE_NUMBER_GUTTER_VISIBLE_LINES));
        let mut replacement_selection = self
            .panes
            .get(pane)
            .and_then(|pane_state| pane_state.editor.replacement_selection);

        let Some(tile) = self.workspace.tile_mut(tile_id) else {
            return;
        };
        gui_editor_replacement_mouse_drag(
            &tile.document,
            &mut tile.state.cursor,
            &mut viewport,
            &mut replacement_selection,
            drag.anchor,
            focus,
        );
        let cursor = tile.state.cursor;
        let status = if replacement_selection.is_some() {
            "selected text"
        } else {
            "cursor moved"
        };
        self.replacement_ime_preedit = None;
        self.update_replacement_editor_view_state(
            pane,
            cursor,
            viewport,
            replacement_selection,
            status,
        );
    }
}
