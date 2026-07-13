impl KfnotepadGui {
    pub(in crate::gui::app::state) fn create_new_tile(&mut self) {
        let path = self.next_untitled_path();
        let document = TextDocument {
            path: path.clone(),
            buffer: TextBuffer::from_text(""),
        };
        let editor = text_editor::Content::with_text("");
        let tile_id = self.workspace.open_tile(document);
        let split_axis = split_axis_for_pane(&self.panes, self.active_pane);
        let was_maximized = self.panes.maximized().is_some();

        if let Some((pane, _split)) =
            self.panes
                .split(split_axis, self.active_pane, GuiPane::new(tile_id, editor))
        {
            self.active_pane = pane;
            self.workspace.focus_tile(tile_id);
            if was_maximized {
                self.panes.restore();
                self.panes.maximize(pane);
            }
            self.pending_close_tile = None;
            self.pending_app_quit = false;
            self.pending_project_open = None;
            self.file_snapshots.remove(&tile_id);
            self.external_edit_locks.remove(&tile_id);
            self.invalidate_syntax_cache(tile_id);
            self.ensure_visible_syntax_cache_for_tile(tile_id);
            self.status_message = format!("new tile {}", path.display());
            self.persist_layout();
            self.persist_last_workspace_if_enabled();
        } else {
            self.status_message = format!(
                "cannot create new tile {}: pane split failed",
                path.display()
            );
        }
    }

    pub(in crate::gui::app::state) fn next_untitled_path(&self) -> PathBuf {
        let current_dir = self
            .browser
            .as_ref()
            .map(|browser| browser.sidebar.current_dir.clone())
            .unwrap_or_else(|| self.current_dir.clone());
        self.next_untitled_path_in_dir_excluding(current_dir, None)
    }

    pub(in crate::gui::app::state) fn next_untitled_path_in_dir_excluding(
        &self,
        directory: PathBuf,
        excluded_tile_id: Option<GuiTileId>,
    ) -> PathBuf {
        for index in 1.. {
            let file_name = if index == 1 {
                "untitled.txt".to_string()
            } else {
                format!("untitled-{index}.txt")
            };
            let candidate = directory.join(file_name);
            let already_open =
                self.workspace.tiles.iter().any(|tile| {
                    Some(tile.id) != excluded_tile_id && tile.document.path == candidate
                });
            if !already_open && !candidate.exists() {
                return candidate;
            }
        }

        unreachable!("untitled candidate search is unbounded")
    }
}
