impl KfnotepadGui {
    pub(super) fn show_path_prompt(&mut self, prompt: GuiPathPrompt) {
        self.path_prompt = Some(prompt);
        self.path_prompt_value = match prompt {
            GuiPathPrompt::Open => String::new(),
            GuiPathPrompt::SaveAs => self
                .workspace
                .active_tile()
                .document
                .path
                .display()
                .to_string(),
            GuiPathPrompt::ManagedNote => String::new(),
            GuiPathPrompt::BrowserCreateFile => String::new(),
            GuiPathPrompt::BrowserCreateDirectory => String::new(),
        };
        self.status_message = match prompt {
            GuiPathPrompt::Open => "open path".to_string(),
            GuiPathPrompt::SaveAs => "save as path".to_string(),
            GuiPathPrompt::ManagedNote => "managed note title".to_string(),
            GuiPathPrompt::BrowserCreateFile => "create file name".to_string(),
            GuiPathPrompt::BrowserCreateDirectory => "create directory name".to_string(),
        };
    }

    pub(super) fn cancel_path_prompt(&mut self) {
        self.clear_path_prompt_context();
        self.status_message = "path prompt canceled".to_string();
    }
}
