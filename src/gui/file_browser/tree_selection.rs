impl KfnotepadGui {
    pub(super) fn toggle_local_browser_tree_path(&mut self, path: PathBuf) -> Task<Message> {
        if self.browser_expanded_paths.contains(&path) {
            self.browser_expanded_paths.remove(&path);
        } else {
            self.browser_expanded_paths.insert(path);
        }
        self.request_cached_file_tree_rows()
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

    pub(super) fn request_cached_file_tree_rows(&mut self) -> Task<Message> {
        let Some(root) = self
            .browser
            .as_ref()
            .map(|browser| browser.sidebar.current_dir.clone())
        else {
            self.browser_tree_rows.clear();
            self.browser_tree_loading = false;
            return Task::none();
        };
        let expanded_paths = self.browser_expanded_paths.clone();
        let selected_path = self.browser_selected_path.clone();
        self.browser_tree_generation = self.browser_tree_generation.wrapping_add(1);
        let generation = self.browser_tree_generation;
        self.browser_tree_loading = true;

        #[cfg(test)]
        {
            let rows = gui_file_tree_rows(&root, &expanded_paths, selected_path.as_deref());
            self.apply_cached_file_tree_rows(generation, Ok(rows));
            Task::none()
        }
        #[cfg(not(test))]
        Task::perform(
            async move {
                tokio::task::spawn_blocking(move || {
                    gui_file_tree_rows(&root, &expanded_paths, selected_path.as_deref())
                })
                .await
                .map_err(|error| format!("file tree worker failed: {error}"))
            },
            move |result| Message::BrowserTreeRowsLoaded { generation, result },
        )
    }

    pub(super) fn apply_cached_file_tree_rows(
        &mut self,
        generation: u64,
        result: Result<Vec<GuiFileTreeRowModel>, String>,
    ) {
        if generation != self.browser_tree_generation {
            return;
        }
        self.browser_tree_loading = false;
        match result {
            Ok(rows) => self.browser_tree_rows = rows,
            Err(error) => self.status_message = error,
        }
    }

    fn update_cached_file_tree_selection(&mut self, selected_path: &Path) {
        for row in &mut self.browser_tree_rows {
            row.selected = row.path == selected_path;
        }
    }
}
