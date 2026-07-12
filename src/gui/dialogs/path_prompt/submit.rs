//! Path-prompt validation and submission routing.

use super::*;

impl KfnotepadGui {
    pub(in crate::gui) fn submit_path_prompt(&mut self) -> Task<Message> {
        let Some(prompt) = self.path_prompt else {
            return Task::none();
        };
        let raw_path = self.path_prompt_value.trim().to_string();
        if raw_path.is_empty() {
            self.status_message = match prompt {
                GuiPathPrompt::Open => "open failed: path required".to_string(),
                GuiPathPrompt::SaveAs => "save as failed: path required".to_string(),
                GuiPathPrompt::ManagedNote => "managed note failed: title required".to_string(),
                GuiPathPrompt::BrowserCreateFile => "create file failed: name required".to_string(),
                GuiPathPrompt::BrowserCreateDirectory => {
                    "create directory failed: name required".to_string()
                }
            };
            return Task::none();
        }

        match prompt {
            GuiPathPrompt::Open => self.submit_open_path_prompt(&raw_path),
            GuiPathPrompt::SaveAs => self.submit_save_as_path_prompt(&raw_path),
            GuiPathPrompt::ManagedNote => {
                self.submit_managed_note_prompt(&raw_path);
                Task::none()
            }
            GuiPathPrompt::BrowserCreateFile => self.submit_browser_create_file_prompt(&raw_path),
            GuiPathPrompt::BrowserCreateDirectory => {
                self.submit_browser_create_directory_prompt(&raw_path)
            }
        }
    }
}
