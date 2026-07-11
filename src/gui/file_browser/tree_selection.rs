impl KfnotepadGui {
    pub(super) fn toggle_local_browser_tree_path(&mut self, path: PathBuf) {
        if self.browser_expanded_paths.contains(&path) {
            self.browser_expanded_paths.remove(&path);
        } else {
            self.browser_expanded_paths.insert(path);
        }
        self.refresh_cached_file_tree_rows();
    }

    pub(super) fn select_local_browser_tree_path(
        &mut self,
        path: PathBuf,
        is_dir: bool,
    ) -> Task<Message> {
        if !self.browser_visible || self.left_panel.mode != GuiLeftPanelMode::Files {
            return Task::none();
        }

        self.select_browser_path(&path);
        self.status_message = if is_dir {
            format!("selected folder {}", path.display())
        } else {
            format!("selected file {}", path.display())
        };
        Task::none()
    }

    pub(super) fn activate_local_browser_tree_path(
        &mut self,
        path: PathBuf,
        is_dir: bool,
    ) -> Task<Message> {
        if !self.browser_visible || self.left_panel.mode != GuiLeftPanelMode::Files {
            return Task::none();
        }

        self.select_browser_path(&path);
        if is_dir {
            self.set_browser_root(path)
        } else {
            #[cfg(test)]
            {
                let _opened = self.open_path_in_new_pane(path);
                Task::none()
            }
            #[cfg(not(test))]
            {
                self.open_path_in_new_pane_async(path)
            }
        }
    }

    pub(super) fn select_browser_path(&mut self, path: &Path) {
        self.browser_selected_path = Some(path.to_path_buf());
        self.update_cached_file_tree_selection(path);
        let Some(browser) = self.browser.as_mut() else {
            return;
        };
        if let Some(index) = browser
            .sidebar
            .entries
            .iter()
            .position(|entry| entry.path == path)
        {
            browser.sidebar.selected = index;
            browser.sidebar.keep_selection_visible(1);
        }
        self.pending_browser_delete = None;
    }

    pub(super) fn refresh_cached_file_tree_rows(&mut self) {
        let Some(root) = self
            .browser
            .as_ref()
            .map(|browser| browser.sidebar.current_dir.clone())
        else {
            self.browser_tree_rows.clear();
            return;
        };
        self.browser_tree_rows = gui_file_tree_rows(
            &root,
            &self.browser_expanded_paths,
            self.browser_selected_path.as_deref(),
        );
    }

    fn update_cached_file_tree_selection(&mut self, selected_path: &Path) {
        for row in &mut self.browser_tree_rows {
            row.selected = row.path == selected_path;
        }
    }
}
