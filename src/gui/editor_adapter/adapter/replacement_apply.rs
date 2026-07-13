//! Replacement editor command application.

use super::*;

impl GuiEditorAdapter {
    pub(crate) fn apply_replacement_command(&mut self, command: GuiEditorCommand) {
        match command {
            GuiEditorCommand::IcedAction(action) => match action {
                text_editor::Action::Scroll { lines } => {
                    self.scroll_viewport_by_lines(lines);
                }
                text_editor::Action::Move(motion) => {
                    self.apply_text_editor_motion_to_replacement(motion);
                }
                _ => {
                    self.content.perform(action);
                    self.sync_viewport_to_cursor();
                }
            },
            GuiEditorCommand::Delete => {
                if self.replacement_selection.is_none() {
                    self.content
                        .perform(text_editor::Action::Edit(text_editor::Edit::Delete));
                    self.sync_viewport_to_cursor();
                } else {
                    let mut document = TextDocument {
                        path: PathBuf::from("replacement-delete.txt"),
                        buffer: TextBuffer::from_text(&self.text()),
                    };
                    let mut cursor = self.document_cursor();
                    let mut selection = self.replacement_selection;
                    let mut viewport = self.viewport;
                    apply_gui_editor_replacement_input(
                        &mut document,
                        &mut cursor,
                        &mut viewport,
                        &mut selection,
                        GuiEditorReplacementInput::DeleteForward,
                    );
                    self.content = text_editor::Content::with_text(&document.buffer.to_text());
                    self.content.move_to(editor_cursor_from_document(cursor));
                    self.viewport = viewport;
                    self.viewport_tracks_cursor = true;
                    self.replacement_selection = selection;
                }
            }
            GuiEditorCommand::MoveTo(cursor) => {
                self.content.move_to(editor_cursor_from_document(cursor));
                self.replacement_selection = None;
                self.sync_viewport_to_cursor();
            }
            GuiEditorCommand::Paste(contents) => {
                if self.replacement_selection.is_none() {
                    self.content
                        .perform(text_editor::Action::Edit(text_editor::Edit::Paste(
                            Arc::new(contents),
                        )));
                    self.sync_viewport_to_cursor();
                } else {
                    let mut document = TextDocument {
                        path: PathBuf::from("replacement-paste.txt"),
                        buffer: TextBuffer::from_text(&self.text()),
                    };
                    let mut cursor = self.document_cursor();
                    let mut selection = self.replacement_selection;
                    let mut viewport = self.viewport;
                    gui_editor_replacement_paste_text(
                        &mut document,
                        &mut cursor,
                        &mut viewport,
                        &mut selection,
                        &contents,
                    );
                    self.content = text_editor::Content::with_text(&document.buffer.to_text());
                    self.content.move_to(editor_cursor_from_document(cursor));
                    self.viewport = viewport;
                    self.viewport_tracks_cursor = true;
                    self.replacement_selection = selection;
                }
            }
            GuiEditorCommand::ScrollViewportLines(delta) => {
                self.scroll_viewport_by_lines(delta);
            }
            GuiEditorCommand::SelectAll => {
                let text = self.text();
                let start = DocumentCursor { row: 0, column: 0 };
                let end = gui_editor_replacement_text_end_cursor(&text);
                self.content.move_to(editor_cursor_from_document(end));
                self.replacement_selection = GuiEditorReplacementSelection::new(start, end);
                self.sync_viewport_to_cursor();
            }
            GuiEditorCommand::SelectRightChars(count) => {
                let start = self.document_cursor();
                let text = self.text();
                let buffer = TextBuffer::from_text(&text);
                let lines: Vec<&str> = text.split('\n').collect();
                let max_columns = lines
                    .get(start.row)
                    .map(|line| line.chars().count())
                    .unwrap_or(start.column);
                let requested_focus = start.column.saturating_add(count).min(max_columns);
                let (start_column, focus_column) = buffer
                    .grapheme_range_boundary_columns(start.row, start.column, requested_focus)
                    .unwrap_or((start.column, requested_focus));
                let start = DocumentCursor {
                    row: start.row,
                    column: start_column,
                };
                let focus = DocumentCursor {
                    row: start.row,
                    column: focus_column,
                };
                self.content.move_to(editor_cursor_from_document(start));
                self.replacement_selection = GuiEditorReplacementSelection::new(start, focus);
                self.sync_viewport_to_cursor();
            }
        }
    }
}
