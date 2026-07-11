impl KfnotepadGui {
    pub(super) fn request_open_dialog(&mut self) -> Task<Message> {
        if let Some(reason) = Self::gui_file_dialog_unavailable_reason() {
            return self.request_file_dialog_fallback(GuiPathPrompt::Open, reason);
        }

        self.path_prompt = None;
        self.path_prompt_value.clear();
        let directory = self.current_browser_dir();
        self.status_message = "open dialog".to_string();

        Task::perform(
            async move {
                rfd::AsyncFileDialog::new()
                    .set_directory(directory)
                    .pick_file()
                    .await
                    .map(|handle| handle.path().to_path_buf())
            },
            Message::OpenDialogSelectedAsync,
        )
    }

    pub(super) fn open_path_in_new_pane_async(&mut self, path: PathBuf) -> Task<Message> {
        self.status_message = format!("opening {}", path.display());
        let requested_path = path.clone();

        Task::perform(
            async move {
                open_text_file(&path)
                    .map(Box::new)
                    .map_err(|error| error.to_string())
            },
            move |result| Message::OpenDialogCompleted {
                path: requested_path.clone(),
                result,
            },
        )
    }

    pub(super) fn handle_open_dialog_selected_async(
        &mut self,
        path: Option<PathBuf>,
    ) -> Task<Message> {
        match path {
            Some(path) => self.open_path_in_new_pane_async(path),
            None => {
                self.status_message = "open canceled".to_string();
                Task::none()
            }
        }
    }

    pub(super) fn handle_open_dialog_completed(
        &mut self,
        path: PathBuf,
        result: Result<TextDocument, String>,
    ) {
        match result {
            Ok(document) => {
                let _opened =
                    self.open_document_in_new_pane(document, format!("opened {}", path.display()));
                if self.path_prompt == Some(GuiPathPrompt::Open) {
                    self.path_prompt = None;
                    self.path_prompt_value.clear();
                    self.notes_panel = None;
                    self.pending_managed_note_delete = None;
                }
            }
            Err(error) => {
                self.status_message = format!("cannot open {}: {error}", path.display());
            }
        }
    }

    #[cfg(test)]
    pub(super) fn handle_open_dialog_selected(&mut self, path: Option<PathBuf>) -> Task<Message> {
        match path {
            Some(path) => {
                let _opened = self.open_path_in_new_pane(path);
                Task::none()
            }
            None => {
                self.status_message = "open canceled".to_string();
                Task::none()
            }
        }
    }
}
