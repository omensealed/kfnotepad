//! Managed-note and file-browser creation prompt submissions.

use super::*;

impl KfnotepadGui {
    pub(super) fn submit_managed_note_prompt(&mut self, raw_path: &str) {
        let success = self.open_managed_note_by_title(raw_path);
        if success {
            self.clear_path_prompt_context();
        }
    }

    pub(super) fn submit_browser_create_file_prompt(&mut self, raw_path: &str) -> Task<Message> {
        let task = self.create_browser_file_named(raw_path);
        if task.is_some() {
            self.clear_path_prompt_context();
        }
        task.unwrap_or_else(Task::none)
    }

    pub(super) fn submit_browser_create_directory_prompt(
        &mut self,
        raw_path: &str,
    ) -> Task<Message> {
        let task = self.create_browser_directory_named(raw_path);
        if task.is_some() {
            self.clear_path_prompt_context();
        }
        task.unwrap_or_else(Task::none)
    }
}
