//! Open and save-as path-prompt submissions.

use super::*;

impl KfnotepadGui {
    pub(super) fn submit_open_path_prompt(&mut self, raw_path: &str) -> Task<Message> {
        let path = self.resolve_prompt_path(raw_path);
        #[cfg(test)]
        {
            let success = self.open_path_in_new_pane(path);
            if success {
                self.clear_path_prompt_context();
            }
            Task::none()
        }
        #[cfg(not(test))]
        {
            self.status_message = format!("opening {}", path.display());
            self.open_path_in_new_pane_async(path)
        }
    }

    pub(super) fn submit_save_as_path_prompt(&mut self, raw_path: &str) -> Task<Message> {
        let path = self.resolve_prompt_path(raw_path);
        #[cfg(test)]
        {
            let success = self.save_active_tile_as(path);
            if success {
                self.clear_path_prompt_context();
            }
            Task::none()
        }
        #[cfg(not(test))]
        {
            self.request_save_active_tile_as(path)
        }
    }
}
