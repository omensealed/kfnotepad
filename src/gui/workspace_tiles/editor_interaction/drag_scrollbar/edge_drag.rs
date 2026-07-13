//! Edge-drag viewport scrolling and selection extension.

use super::*;

impl KfnotepadGui {
    pub(in crate::gui::app::state) fn replacement_editor_drag_tick(&mut self) {
        let Some(edge) = self.replacement_drag_edge else {
            return;
        };
        if self
            .replacement_drag
            .is_none_or(|drag| drag.pane != edge.pane)
        {
            self.replacement_drag_edge = None;
            return;
        }
        let Some(tile_id) = self
            .panes
            .get(edge.pane)
            .map(|pane_state| pane_state.tile_id)
        else {
            self.clear_replacement_drag();
            return;
        };
        let Some(pane_state) = self.panes.get(edge.pane) else {
            self.clear_replacement_drag();
            return;
        };
        let mut viewport = pane_state.editor.viewport;
        let line_count = self
            .workspace
            .tile(tile_id)
            .map(|tile| tile.document.buffer.line_count())
            .unwrap_or(1);
        let before = viewport.first_line;
        viewport.scroll_by(edge.direction, line_count);
        if viewport.first_line == before {
            return;
        }
        if let Some(pane_state) = self.panes.get_mut(edge.pane) {
            pane_state.editor.viewport = viewport;
            pane_state.editor.viewport_tracks_cursor = false;
        }

        let focus_row = if edge.direction < 0 {
            viewport.first_line.saturating_sub(1)
        } else {
            viewport.last_visible_line(line_count).saturating_sub(1)
        };
        let focus_point = GuiEditorReplacementMousePoint {
            viewport_row: focus_row
                .saturating_add(1)
                .saturating_sub(viewport.first_line),
            column: edge.column,
        };
        self.apply_replacement_editor_mouse_drag_to_pane(edge.pane, focus_point);
    }
}
