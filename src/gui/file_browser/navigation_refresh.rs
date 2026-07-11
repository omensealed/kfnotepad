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
        let Some(directory) = self
            .browser
            .as_ref()
            .map(|browser| browser.sidebar.current_dir.clone())
        else {
            return self.set_browser_root(self.current_browser_dir());
        };
        self.status_message = format!("refreshed {}", directory.display());
        self.request_browser_load(directory, false)
    }
}
