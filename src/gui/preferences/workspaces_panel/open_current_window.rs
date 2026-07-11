impl KfnotepadGui {
    pub(super) fn open_workspace_project_in_current_window(&mut self, index: usize) {
        let Some(entry) = self.workspace_projects.get(index).cloned() else {
            return;
        };
        if self.has_dirty_tile() && self.pending_project_open != Some(index) {
            self.pending_project_open = Some(index);
            self.pending_close_tile = None;
            self.pending_app_quit = false;
            self.status_message =
                "unsaved changes; open project again to replace current workspace".to_string();
            return;
        }

        let restored = restore_gui_workspace_project_documents(
            entry.project.clone(),
            self.current_dir.clone(),
        );
        let skipped_status_message = restored.skipped_status_message();
        let skipped_files_empty = restored.skipped_files.is_empty();
        let active_loaded_ordinal = restored.active_loaded_ordinal;
        let mut documents = restored.documents.into_iter();
        let Some(first_document) = documents.next() else {
            self.pending_project_open = None;
            self.status_message = "workspace project open failed: empty project".to_string();
            return;
        };

        let mut workspace = GuiWorkspace::from_document(first_document);
        let mut pane_states = vec![GuiPane::new(
            workspace.active,
            text_editor::Content::with_text(&workspace.active_tile().document.buffer.to_text()),
        )];
        for document in documents {
            let editor = text_editor::Content::with_text(&document.buffer.to_text());
            let tile_id = workspace.open_tile(document);
            pane_states.push(GuiPane::new(tile_id, editor));
        }

        let project_layout = if skipped_files_empty {
            entry.project.layout.clone()
        } else {
            None
        };
        let (panes, mut active_pane) = if let Some(layout) = project_layout {
            let (panes, pane) = panes_from_gui_layout(layout.clone(), pane_states);
            for ordinal in &layout.minimized_ordinals {
                if let Some(tile_id) = workspace.tiles.get(*ordinal).map(|tile| tile.id) {
                    workspace.set_tile_minimized(tile_id, true);
                }
            }
            self.browser_visible = layout.browser_visible;
            self.left_panel.visible = layout.browser_visible;
            if let Some(width) = layout.browser_width_px {
                self.browser_width = clamp_browser_width(f32::from(width));
            }
            (panes, pane)
        } else {
            default_panes(pane_states)
        };
        let (panes, minimized_panes, active_pane_after_minimize) =
            close_minimized_panes_into_tray(panes, &workspace, active_pane);
        active_pane = active_pane_after_minimize;

        if let Some(active_loaded_ordinal) = active_loaded_ordinal {
            if let Some(active_tile_id) = workspace
                .tiles
                .get(active_loaded_ordinal)
                .map(|tile| tile.id)
            {
                workspace.focus_tile(active_tile_id);
                if let Some(pane) = pane_for_tile_id(&panes, active_tile_id) {
                    active_pane = pane;
                }
            }
        } else if let Some(active_tile_id) = workspace
            .tiles
            .get(entry.project.active_ordinal)
            .map(|tile| tile.id)
        {
            workspace.focus_tile(active_tile_id);
            if let Some(pane) = pane_for_tile_id(&panes, active_tile_id) {
                active_pane = pane;
            }
        }

        self.workspace = workspace;
        self.panes = panes;
        self.active_pane = active_pane;
        self.minimized_panes = minimized_panes;
        self.pending_project_open = None;
        self.pending_close_tile = None;
        self.pending_app_quit = false;
        self.external_edit_locks.clear();
        self.refresh_all_file_snapshots();
        self.invalidate_all_syntax_caches();
        self.refresh_visible_syntax_caches();
        self.status_message = format!("opened workspace project {}", entry.project.name);
        if let Some(message) = skipped_status_message {
            self.status_message = format!("{}; {message}", self.status_message);
        }
        self.persist_layout();
    }
}
