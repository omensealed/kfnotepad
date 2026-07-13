impl KfnotepadGui {
    pub(in crate::gui::app::state) fn replacement_editor_cursor_for_point(
        &self,
        pane: pane_grid::Pane,
        point: GuiEditorReplacementMousePoint,
    ) -> Option<DocumentCursor> {
        let tile_id = self.panes.get(pane)?.tile_id;
        let viewport = self.panes.get(pane)?.editor.viewport;
        let tile = self.workspace.tile(tile_id)?;
        Some(gui_editor_replacement_cursor_from_mouse_point(
            &tile.document.buffer,
            viewport,
            point,
        ))
    }
}
