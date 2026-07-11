impl KfnotepadGui {
pub(super) fn create_browser_file_named(&mut self, raw_name: &str) -> bool {
        let directory = self.selected_browser_create_directory();
        let path = match resolve_browser_child_path(&directory, raw_name) {
            Ok(path) => path,
            Err(message) => {
                self.status_message = format!("create file failed: {message}");
                return false;
            }
        };
        let buffer = TextBuffer::from_text("");

        if let Err(error) = save_text_buffer(&path, &buffer) {
            self.status_message = format!("create file failed: {error}");
            return false;
        }

        let _refresh_task = self.refresh_file_browser();
        self.rebuild_cached_file_tree_rows_now();
        self.select_browser_path(&path);
        let opened = self.open_path_in_new_pane(path.clone());
        if opened {
            self.status_message = format!("created {}", path.display());
        }
        opened
    }

    pub(super) fn create_browser_directory_named(&mut self, raw_name: &str) -> bool {
        let directory = self.selected_browser_create_directory();
        let path = match resolve_browser_child_path(&directory, raw_name) {
            Ok(path) => path,
            Err(message) => {
                self.status_message = format!("create directory failed: {message}");
                return false;
            }
        };

        match fs::create_dir(&path) {
            Ok(()) => {
                self.browser_expanded_paths.insert(path.clone());
                let _refresh_task = self.refresh_file_browser();
                self.rebuild_cached_file_tree_rows_now();
                self.select_browser_path(&path);
                self.status_message = format!("created directory {}", path.display());
                true
            }
            Err(error) => {
                self.status_message = format!("create directory failed: {error}");
                false
            }
        }
    }

    pub(super) fn selected_browser_create_directory(&self) -> PathBuf {
        self.selected_browser_action_entry()
            .and_then(|entry| match entry.kind {
                FileSidebarEntryKind::Directory => Some(entry.path),
                FileSidebarEntryKind::Parent | FileSidebarEntryKind::File => None,
            })
        .unwrap_or_else(|| self.current_browser_dir())
}
}
