impl KfnotepadGui {
pub(super) fn create_browser_file_named(&mut self, raw_name: &str) -> Option<Task<Message>> {
        let directory = self.selected_browser_create_directory();
        let path = match resolve_browser_child_path(&directory, raw_name) {
            Ok(path) => path,
            Err(message) => {
                self.status_message = format!("create file failed: {message}");
                return None;
            }
        };
        let buffer = TextBuffer::from_text("");

        if let Err(error) = save_text_buffer(&path, &buffer) {
            self.status_message = format!("create file failed: {error}");
            return None;
        }

        self.select_browser_path(&path);
        let opened = self.open_path_in_new_pane(path.clone());
        if opened {
            let refresh_task = self.refresh_file_browser();
            self.status_message = format!("created {}", path.display());
            Some(refresh_task)
        } else {
            None
        }
    }

    pub(super) fn create_browser_directory_named(
        &mut self,
        raw_name: &str,
    ) -> Option<Task<Message>> {
        let directory = self.selected_browser_create_directory();
        let path = match resolve_browser_child_path(&directory, raw_name) {
            Ok(path) => path,
            Err(message) => {
                self.status_message = format!("create directory failed: {message}");
                return None;
            }
        };

        match fs::create_dir(&path) {
            Ok(()) => {
                self.browser_expanded_paths.insert(path.clone());
                self.select_browser_path(&path);
                let refresh_task = self.refresh_file_browser();
                self.status_message = format!("created directory {}", path.display());
                Some(refresh_task)
            }
            Err(error) => {
                self.status_message = format!("create directory failed: {error}");
                None
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
