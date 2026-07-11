impl GuiEditorAdapter {
    pub(crate) fn apply_text_editor_motion_to_replacement(&mut self, motion: text_editor::Motion) {
        let mut selection = self.replacement_selection;
        let mut viewport = self.viewport;

        match motion {
            text_editor::Motion::Left
            | text_editor::Motion::Right
            | text_editor::Motion::Up
            | text_editor::Motion::Down
            | text_editor::Motion::WordLeft
            | text_editor::Motion::WordRight
            | text_editor::Motion::Home
            | text_editor::Motion::End
            | text_editor::Motion::DocumentStart
            | text_editor::Motion::DocumentEnd => {
                self.content.perform(text_editor::Action::Move(motion));
                selection = None;
                self.replacement_selection = selection;
                self.sync_viewport_to_cursor();
                return;
            }
            _ => {
                let mut document = TextDocument {
                    path: PathBuf::from("replacement-motion.txt"),
                    buffer: TextBuffer::from_text(&self.text()),
                };
                let mut cursor = self.document_cursor();
                match motion {
                    text_editor::Motion::PageUp => apply_gui_editor_replacement_input(
                        &mut document,
                        &mut cursor,
                        &mut viewport,
                        &mut selection,
                        GuiEditorReplacementInput::ScrollViewportLines(
                            -(GUI_LINE_NUMBER_GUTTER_VISIBLE_LINES as i32),
                        ),
                    ),
                    text_editor::Motion::PageDown => apply_gui_editor_replacement_input(
                        &mut document,
                        &mut cursor,
                        &mut viewport,
                        &mut selection,
                        GuiEditorReplacementInput::ScrollViewportLines(
                            GUI_LINE_NUMBER_GUTTER_VISIBLE_LINES as i32,
                        ),
                    ),
                    _ => unreachable!(
                        "All supported replacement motions are handled in the primary match branch"
                    ),
                }
                self.content.move_to(editor_cursor_from_document(cursor));
            }
        }

        self.viewport = viewport;
        self.viewport_tracks_cursor = true;
        self.replacement_selection = selection;
        self.sync_viewport_to_cursor();
    }
}
