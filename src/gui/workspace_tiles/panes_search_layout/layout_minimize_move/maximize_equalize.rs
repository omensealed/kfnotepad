//! Pane maximize/restore and equalized layout reconstruction.

use super::*;

impl KfnotepadGui {
    pub(in crate::gui::app::state) fn toggle_active_maximize(&mut self) {
        self.toggle_pane_maximized(self.active_pane);
    }

    pub(in crate::gui::app::state) fn toggle_pane_maximized(&mut self, pane: pane_grid::Pane) {
        let was_maximized = self.panes.maximized() == Some(pane);
        if !self.focus_pane(pane) {
            self.status_message = "maximize failed: no such pane".to_string();
            return;
        }

        if was_maximized {
            self.panes.restore();
            self.status_message = "restored tile layout".to_string();
        } else {
            self.panes.maximize(pane);
            self.status_message = "maximized tile".to_string();
        }
        self.pending_close_tile = None;
        self.pending_app_quit = false;
    }

    pub(in crate::gui::app::state) fn equalize_tile_layout(&mut self) {
        self.sync_all_panes_to_documents();
        let visible_tile_ids = self
            .workspace
            .tiles
            .iter()
            .filter(|tile| !tile.minimized)
            .map(|tile| tile.id)
            .collect::<Vec<_>>();
        if visible_tile_ids.len() <= 1 {
            self.status_message = "tile layout already equalized".to_string();
            return;
        }

        let Some(layout_root) = equalized_tile_layout_node(visible_tile_ids.len()) else {
            self.status_message = "equalize failed: no visible tiles".to_string();
            return;
        };
        let mut pane_states = Vec::with_capacity(visible_tile_ids.len());
        for tile_id in &visible_tile_ids {
            let Some(pane) = pane_for_tile_id(&self.panes, *tile_id) else {
                self.status_message = "equalize failed: missing pane".to_string();
                return;
            };
            let Some(pane_state) = self.panes.get(pane) else {
                self.status_message = "equalize failed: missing pane".to_string();
                return;
            };
            pane_states.push(GuiPane {
                tile_id: *tile_id,
                editor: pane_state.editor.clone_for_relayout(),
            });
        }

        let layout = GuiLayout {
            browser_visible: self.browser_visible,
            browser_width_px: Some(persisted_browser_width(self.browser_width)),
            root: layout_root,
            minimized_ordinals: self
                .workspace
                .tiles
                .iter()
                .enumerate()
                .filter_map(|(ordinal, tile)| tile.minimized.then_some(ordinal))
                .collect(),
        };
        let active_tile_id = self.workspace.active;
        let was_maximized = self.panes.maximized().is_some();
        let (mut panes, fallback_pane) = panes_from_gui_layout(layout, pane_states);
        let active_pane = pane_for_tile_id(&panes, active_tile_id).unwrap_or(fallback_pane);
        if was_maximized {
            panes.maximize(active_pane);
        }
        self.panes = panes;
        self.active_pane = active_pane;
        self.workspace.focus_tile(active_tile_id);
        self.pending_close_tile = None;
        self.pending_app_quit = false;
        self.pending_project_open = None;
        self.status_message = format!("equalized {} tiles", visible_tile_ids.len());
        self.refresh_visible_syntax_caches();
        self.persist_layout();
    }
}
