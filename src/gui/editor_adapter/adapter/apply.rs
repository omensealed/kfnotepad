impl GuiEditorAdapter {
    pub(crate) fn apply(&mut self, command: GuiEditorCommand) {
        if GUI_USE_READ_ONLY_EDITOR_RENDERER {
            self.apply_replacement_command(command);
            return;
        }

        let sync_viewport_to_cursor = match command {
            GuiEditorCommand::IcedAction(action) => {
                let scroll_lines = match action {
                    text_editor::Action::Scroll { lines } => Some(lines),
                    _ => None,
                };
                self.content.perform(action);
                if let Some(lines) = scroll_lines {
                    self.scroll_viewport_by_lines(lines);
                    false
                } else {
                    true
                }
            }
            GuiEditorCommand::Delete => {
                self.content
                    .perform(text_editor::Action::Edit(text_editor::Edit::Delete));
                true
            }
            GuiEditorCommand::MoveTo(cursor) => {
                self.content
                    .perform(text_editor::Action::Move(text_editor::Motion::Right));
                self.content.move_to(editor_cursor_from_document(cursor));
                true
            }
            GuiEditorCommand::Paste(contents) => {
                self.content
                    .perform(text_editor::Action::Edit(text_editor::Edit::Paste(
                        Arc::new(contents),
                    )));
                true
            }
            GuiEditorCommand::ScrollViewportLines(delta) => {
                self.scroll_viewport_by_lines(delta);
                false
            }
            GuiEditorCommand::SelectAll => {
                self.content.perform(text_editor::Action::SelectAll);
                true
            }
            GuiEditorCommand::SelectRightChars(count) => {
                for _ in 0..count {
                    self.content
                        .perform(text_editor::Action::Select(text_editor::Motion::Right));
                }
                true
            }
        };
        if sync_viewport_to_cursor {
            self.sync_viewport_to_cursor();
        }
    }
}
