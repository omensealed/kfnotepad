impl KfnotepadGui {
    pub(super) fn request_browser_load(
        &mut self,
        directory: PathBuf,
        reset_root: bool,
    ) -> Task<Message> {
        self.browser_tree_generation = self.browser_tree_generation.wrapping_add(1);
        let generation = self.browser_tree_generation;
        self.browser_tree_loading = true;
        let expanded_paths = if reset_root {
            HashSet::new()
        } else {
            self.browser_expanded_paths.clone()
        };
        let selected_path = if reset_root {
            None
        } else {
            self.browser_selected_path.clone()
        };

        #[cfg(test)]
        {
            let result = load_gui_browser_and_rows(
                directory,
                expanded_paths,
                selected_path,
                reset_root,
            );
            self.apply_browser_load(generation, result);
            Task::none()
        }
        #[cfg(not(test))]
        Task::perform(
            async move {
                tokio::task::spawn_blocking(move || {
                    load_gui_browser_and_rows(
                        directory,
                        expanded_paths,
                        selected_path,
                        reset_root,
                    )
                })
                .await
                .map_err(|error| format!("file browser worker failed: {error}"))?
            },
            move |result| Message::BrowserLoaded { generation, result },
        )
    }

    pub(super) fn apply_browser_load(
        &mut self,
        generation: u64,
        result: Result<GuiBrowserLoadResult, String>,
    ) {
        if generation != self.browser_tree_generation {
            return;
        }
        self.browser_tree_loading = false;
        match result {
            Ok(loaded) => {
                let root = loaded.browser.sidebar.current_dir.clone();
                self.browser = Some(loaded.browser);
                self.browser_tree_rows = loaded.rows;
                self.browser_selected_path = loaded.selected_path.or_else(|| Some(root.clone()));
                self.browser_expanded_paths = loaded.expanded_paths;
            }
            Err(error) => self.status_message = format!("file browser error: {error}"),
        }
    }
}

fn load_gui_browser_and_rows(
    directory: PathBuf,
    mut expanded_paths: HashSet<PathBuf>,
    selected_path: Option<PathBuf>,
    reset_root: bool,
) -> Result<GuiBrowserLoadResult, String> {
    let mut browser = GuiFileBrowser::load(directory).map_err(|error| error.to_string())?;
    let root = browser.sidebar.current_dir.clone();
    if reset_root {
        expanded_paths.clear();
    }
    expanded_paths.insert(root.clone());
    let selected_path = selected_path
        .filter(|path| path.exists())
        .or_else(|| Some(root.clone()));
    if let Some(selected) = selected_path.as_ref() {
        if let Some(index) = browser
            .sidebar
            .entries
            .iter()
            .position(|entry| entry.path == *selected)
        {
            browser.sidebar.selected = index;
            browser.sidebar.keep_selection_visible(1);
        }
    }
    let rows = gui_file_tree_rows(&root, &expanded_paths, selected_path.as_deref());
    Ok(GuiBrowserLoadResult {
        browser,
        rows,
        selected_path,
        expanded_paths,
    })
}
