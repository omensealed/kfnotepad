impl KfnotepadGui {
    fn submit_managed_note_prompt(&mut self, raw_path: &str) {
        let success = self.open_managed_note_by_title(raw_path);
        if success {
            self.clear_path_prompt_context();
        }
    }

    fn submit_browser_create_file_prompt(&mut self, raw_path: &str) {
        let success = self.create_browser_file_named(raw_path);
        if success {
            self.clear_path_prompt_context();
        }
    }

    fn submit_browser_create_directory_prompt(&mut self, raw_path: &str) {
        let success = self.create_browser_directory_named(raw_path);
        if success {
            self.clear_path_prompt_context();
        }
    }
}
