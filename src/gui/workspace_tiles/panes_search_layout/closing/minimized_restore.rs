impl KfnotepadGui {
    pub(in crate::gui::app::state) fn restore_first_minimized_into_pane(
        &mut self,
        pane: pane_grid::Pane,
    ) -> Option<GuiTileId> {
        if self.minimized_panes.is_empty() {
            return None;
        }

        let pane_state = self.minimized_panes.remove(0);
        let tile_id = pane_state.tile_id;
        if let Some(target) = self.panes.get_mut(pane) {
            *target = pane_state;
            self.workspace.set_tile_minimized(tile_id, false);
            self.active_pane = pane;
            self.workspace.focus_tile(tile_id);
            self.refresh_visible_syntax_caches();
            Some(tile_id)
        } else {
            self.minimized_panes.insert(0, pane_state);
            None
        }
    }
}
