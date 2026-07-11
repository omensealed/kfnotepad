impl KfnotepadGui {
pub(super) fn set_browser_root(&mut self, directory: PathBuf) -> Task<Message> {
        match GuiFileBrowser::load(directory) {
            Ok(browser) => {
                let current_dir = browser.sidebar.current_dir.clone();
                self.browser = Some(browser);
                self.browser_expanded_paths.clear();
                self.browser_expanded_paths.insert(current_dir.clone());
                self.browser_selected_path = Some(current_dir.clone());
                self.status_message = format!("browser: {}", current_dir.display());
                self.request_cached_file_tree_rows()
            }
            Err(error) => {
                self.status_message = format!("file browser error: {error}");
                Task::none()
            }
        }
    }

}
