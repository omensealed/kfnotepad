impl KfnotepadGui {
    fn clear_path_prompt_context(&mut self) {
        self.path_prompt = None;
        self.path_prompt_value.clear();
        self.notes_panel = None;
        self.pending_managed_note_delete = None;
    }

    pub(in crate::gui) fn resolve_prompt_path(&self, raw_path: &str) -> PathBuf {
        let path = PathBuf::from(raw_path);
        if path.is_absolute() {
            path
        } else {
            self.current_browser_dir().join(path)
        }
    }
}
