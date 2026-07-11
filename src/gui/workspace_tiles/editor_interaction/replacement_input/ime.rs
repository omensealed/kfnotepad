impl KfnotepadGui {
    pub(super) fn apply_replacement_editor_ime_event(&mut self, event: input_method::Event) {
        let Some(tile_id) = self
            .panes
            .get(self.active_pane)
            .map(|pane_state| pane_state.tile_id)
        else {
            self.replacement_ime_preedit = None;
            return;
        };

        if self.is_external_edit_locked(tile_id) {
            self.replacement_ime_preedit = None;
            self.status_message = "external edit lock active; unlock to edit".to_string();
            return;
        }

        match event {
            input_method::Event::Opened => {
                self.replacement_ime_preedit = Some(GuiImePreedit {
                    tile_id,
                    content: String::new(),
                    selection: None,
                });
            }
            input_method::Event::Preedit(content, selection) => {
                self.replacement_ime_preedit = (!content.is_empty()).then_some(GuiImePreedit {
                    tile_id,
                    content,
                    selection,
                });
            }
            input_method::Event::Commit(text) => {
                self.replacement_ime_preedit = None;
                let inputs = gui_editor_replacement_inputs_from_text(&text);
                if !inputs.is_empty() {
                    self.search_highlight = None;
                    self.apply_replacement_editor_inputs_to_active_tile(inputs);
                }
            }
            input_method::Event::Closed => {
                self.replacement_ime_preedit = None;
            }
        }
    }
}
