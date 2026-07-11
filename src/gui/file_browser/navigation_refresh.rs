impl KfnotepadGui {
pub(super) fn navigate_browser_parent(&mut self) -> Task<Message> {
        let current_dir = self.current_browser_dir();
        let Some(parent) = current_dir.parent() else {
            self.status_message = "already at filesystem root".to_string();
            return Task::none();
        };
        self.set_browser_root(parent.to_path_buf())
    }

    pub(super) fn refresh_file_browser(&mut self) -> Task<Message> {
        let Some(browser) = self.browser.as_mut() else {
            return self.set_browser_root(self.current_browser_dir());
        };

        match browser.refresh() {
            Ok(()) => {
                let current_dir = browser.sidebar.current_dir.clone();
                self.browser_tree = Some(gui_directory_tree(current_dir.clone()));
                self.browser_expanded_paths.insert(current_dir.clone());
                if self
                    .browser_selected_path
                    .as_ref()
                    .is_some_and(|path| !path.exists())
                {
                    self.browser_selected_path = None;
                }
                self.status_message = format!("refreshed {}", current_dir.display());
                self.expand_browser_tree_root()
            }
            Err(error) => {
                self.status_message = format!("file browser error: {error}");
                Task::none()
    }
}
}
    }
