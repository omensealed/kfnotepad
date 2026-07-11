impl KfnotepadGui {
    pub(super) fn toggle_active_minimize(&mut self) {
        self.toggle_pane_minimized(self.active_pane);
    }

    pub(super) fn toggle_pane_minimized(&mut self, pane: pane_grid::Pane) {
        self.sync_pane_to_document(pane);
        let Some(tile_id) = self.panes.get(pane).map(|pane_state| pane_state.tile_id) else {
            self.status_message = "minimize failed: no such pane".to_string();
            return;
        };
        let Some(was_minimized) = self.workspace.tile(tile_id).map(|tile| tile.minimized) else {
            self.status_message = "minimize failed: no such tile".to_string();
            return;
        };
        if !was_minimized && self.visible_tile_count() <= 1 {
            self.focus_pane(pane);
            self.status_message = "cannot minimize the only visible tile".to_string();
            return;
        }

        let minimized = !was_minimized;
        if minimized {
            let Some((pane_state, fallback_pane)) = self.panes.close(pane) else {
                self.status_message = "minimize failed: pane close failed".to_string();
                return;
            };
            if !self.workspace.set_tile_minimized(tile_id, true) {
                self.status_message = "minimize failed: no such tile".to_string();
                return;
            }
            self.minimized_panes.push(pane_state);
            self.active_pane = fallback_pane;
            if let Some(fallback_tile_id) = self.panes.get(fallback_pane).map(|pane| pane.tile_id) {
                self.workspace.focus_tile(fallback_tile_id);
            }
            self.status_message = "minimized tile".to_string();
            self.refresh_visible_syntax_caches();
        } else {
            self.restore_minimized_tile(tile_id);
            return;
        }
        self.pending_close_tile = None;
        self.pending_app_quit = false;
        self.persist_layout();
    }

    pub(super) fn restore_minimized_tile(&mut self, tile_id: GuiTileId) {
        let Some(index) = self
            .minimized_panes
            .iter()
            .position(|pane_state| pane_state.tile_id == tile_id)
        else {
            self.status_message = "restore failed: no such minimized tile".to_string();
            return;
        };
        let pane_state = self.minimized_panes.remove(index);
        if !self.workspace.set_tile_minimized(tile_id, false) {
            self.status_message = "minimize failed: no such tile".to_string();
            return;
        }

        let split_axis = split_axis_for_pane(&self.panes, self.active_pane);
        let was_maximized = self.panes.maximized().is_some();
        if let Some((pane, _split)) = self.panes.split(split_axis, self.active_pane, pane_state) {
            self.active_pane = pane;
            self.workspace.focus_tile(tile_id);
            if was_maximized {
                self.panes.restore();
                self.panes.maximize(pane);
            }
            self.status_message = "restored tile".to_string();
            self.pending_close_tile = None;
            self.pending_app_quit = false;
            self.refresh_visible_syntax_caches();
            self.persist_layout();
        } else {
            if let Some(tile) = self.workspace.tile_mut(tile_id) {
                tile.minimized = true;
            }
            self.status_message = "restore failed: pane split failed".to_string();
        }
    }

    pub(super) fn minimized_tray_items(&self) -> Vec<GuiMinimizedTrayItem> {
        self.minimized_panes
            .iter()
            .filter_map(|pane_state| {
                let tile = self.workspace.tile(pane_state.tile_id)?;
                let save_status = match tile.save_status() {
                    GuiTileSaveStatus::Saved => "saved".to_string(),
                    GuiTileSaveStatus::Modified => "modified".to_string(),
                    GuiTileSaveStatus::SaveFailed { message } => {
                        format!("save failed: {message}")
                    }
                };
                Some(GuiMinimizedTrayItem {
                    tile_id: pane_state.tile_id,
                    title: gui_tile_title_label(&tile.document.path, false, &save_status),
                    tooltip: tile.document.path.display().to_string(),
                })
            })
            .collect()
    }
}
