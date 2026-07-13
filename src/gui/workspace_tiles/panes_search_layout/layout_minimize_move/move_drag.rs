impl KfnotepadGui {
    pub(in crate::gui::app::state) fn move_active_pane(&mut self, direction: pane_grid::Direction) {
        self.move_pane(self.active_pane, direction);
    }

    pub(in crate::gui::app::state) fn move_pane(&mut self, pane: pane_grid::Pane, direction: pane_grid::Direction) {
        if !self.focus_pane(pane) {
            self.status_message = "move failed: no such pane".to_string();
            return;
        }
        let Some(adjacent) = self.panes.adjacent(pane, direction) else {
            self.status_message = "move failed: no adjacent pane".to_string();
            return;
        };

        self.panes.swap(pane, adjacent);
        self.pending_close_tile = None;
        self.pending_app_quit = false;
        self.status_message = "moved active tile".to_string();
        self.persist_layout();
    }

    pub(in crate::gui::app::state) fn drag_pane(&mut self, event: pane_grid::DragEvent) {
        if let pane_grid::DragEvent::Dropped { pane, target } = event {
            self.panes.drop(pane, target);
            self.focus_pane(pane);
            self.pending_close_tile = None;
            self.pending_app_quit = false;
            self.status_message = "moved tile".to_string();
            self.persist_layout();
        }
    }
}
