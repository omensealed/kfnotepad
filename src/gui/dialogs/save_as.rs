//! Native save-as dialog workflow.

use super::*;

impl KfnotepadGui {
    pub(in crate::gui) fn request_save_as_dialog(&mut self) -> Task<Message> {
        if let Some(reason) = Self::gui_file_dialog_unavailable_reason() {
            return self.request_file_dialog_fallback(GuiPathPrompt::SaveAs, reason);
        }

        self.path_prompt = None;
        self.path_prompt_value.clear();
        let active_path = self.workspace.active_tile().document.path.clone();
        let directory = active_path
            .parent()
            .map(Path::to_path_buf)
            .unwrap_or_else(|| self.current_browser_dir());
        let file_name = active_path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("untitled.txt")
            .to_string();
        self.status_message = "save as dialog".to_string();

        Task::perform(
            async move {
                rfd::AsyncFileDialog::new()
                    .set_directory(directory)
                    .set_file_name(file_name)
                    .save_file()
                    .await
                    .map(|handle| handle.path().to_path_buf())
            },
            Message::SaveAsDialogSelected,
        )
    }

    #[cfg(test)]
    pub(in crate::gui) fn handle_save_as_dialog_selected(
        &mut self,
        path: Option<PathBuf>,
    ) -> Task<Message> {
        match path {
            Some(path) => {
                let _saved = self.save_active_tile_as(path);
            }
            None => {
                self.status_message = "save as canceled".to_string();
            }
        }
        Task::none()
    }

    #[cfg(not(test))]
    pub(in crate::gui) fn handle_save_as_dialog_selected(
        &mut self,
        path: Option<PathBuf>,
    ) -> Task<Message> {
        match path {
            Some(path) => self.request_save_active_tile_as(path),
            None => {
                self.status_message = "save as canceled".to_string();
                Task::none()
            }
        }
    }
}
