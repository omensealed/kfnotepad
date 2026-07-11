impl KfnotepadGui {
pub(super) fn set_browser_root(&mut self, directory: PathBuf) -> Task<Message> {
        match GuiFileBrowser::load(directory) {
            Ok(browser) => {
                let current_dir = browser.sidebar.current_dir.clone();
                self.browser = Some(browser);
                self.browser_tree = Some(gui_directory_tree(current_dir.clone()));
                self.browser_expanded_paths.clear();
                self.browser_expanded_paths.insert(current_dir.clone());
                self.browser_selected_path = Some(current_dir.clone());
                self.status_message = format!("browser: {}", current_dir.display());
                self.expand_browser_tree_root()
            }
            Err(error) => {
                self.status_message = format!("file browser error: {error}");
                Task::none()
            }
        }
    }

    pub(super) fn expand_browser_tree_root(&mut self) -> Task<Message> {
        let Some(tree) = self.browser_tree.as_mut() else {
            return Task::none();
        };
        let root = tree.root_path().to_path_buf();
        tree.update(DirectoryTreeEvent::Toggled(root))
        .map(Message::BrowserTreeEvent)
}
}
