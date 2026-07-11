impl KfnotepadGui {
pub(super) fn set_browser_root(&mut self, directory: PathBuf) -> Task<Message> {
        self.status_message = format!("loading browser: {}", directory.display());
        self.request_browser_load(directory, true)
    }

}
