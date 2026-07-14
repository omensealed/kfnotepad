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
            #[cfg(test)]
            GuiEditorCommand::Delete => {
                if let Some(removes_complete_text) = self.materialize_replacement_selection() {
                    self.content
                        .perform(text_editor::Action::Edit(text_editor::Edit::Delete));
                    if removes_complete_text {
                        self.content
                            .perform(text_editor::Action::Edit(text_editor::Edit::Delete));
                    }
                    self.sync_viewport_to_cursor();
                }
            }
            GuiEditorCommand::MoveTo(cursor) => {
                self.content.move_to(editor_cursor_from_document(cursor));
                self.replacement_selection = None;
                self.sync_viewport_to_cursor();
            }
            #[cfg(test)]
            GuiEditorCommand::Paste(contents) => {
                if let Some(removes_complete_text) = self.materialize_replacement_selection() {
                    self.content
                        .perform(text_editor::Action::Edit(text_editor::Edit::Paste(
                            Arc::new(contents),
                        )));
                    if removes_complete_text {
                        self.content
                            .perform(text_editor::Action::Edit(text_editor::Edit::Delete));
                    }
                    self.sync_viewport_to_cursor();
                }
            }
            GuiEditorCommand::ScrollViewportLines(delta) => {
                self.scroll_viewport_by_lines(delta);
            }
            #[cfg(test)]
            GuiEditorCommand::SelectAll => {
                let text = self.text();
                let start = DocumentCursor { row: 0, column: 0 };
                let end = gui_editor_replacement_text_end_cursor(&text);
                self.content.move_to(editor_cursor_from_document(end));
                self.replacement_selection = GuiEditorReplacementSelection::new(start, end);
                self.sync_viewport_to_cursor();
            }
            #[cfg(test)]
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

    #[cfg(test)]
    fn materialize_replacement_selection(&mut self) -> Option<bool> {
        let Some(selection) = self.replacement_selection else {
            return Some(false);
        };
        let original_cursor = self.document_cursor();
        let (start, end) = selection.normalized();
        self.content.move_to(editor_cursor_from_document(end));
        let positioned_at_end = self.iced_selection_focus() == end;
        self.content
            .perform(text_editor::Action::Move(text_editor::Motion::DocumentEnd));
        let iced_document_end = self.iced_selection_focus();
        let ends_before_trailing_line_ending = iced_document_end
            == (DocumentCursor {
                row: end.row.saturating_add(1),
                column: 0,
            });
        let removes_complete_text = start == (DocumentCursor { row: 0, column: 0 })
            && positioned_at_end
            && (iced_document_end == end || ends_before_trailing_line_ending);
        self.content
            .move_to(editor_cursor_from_document(selection.anchor));

        let moves_right = (selection.anchor.row, selection.anchor.column)
            < (selection.focus.row, selection.focus.column);
        let edge_motion = if moves_right {
            text_editor::Motion::DocumentEnd
        } else {
            text_editor::Motion::DocumentStart
        };
        self.content
            .perform(text_editor::Action::Select(edge_motion));
        if self.iced_selection_focus() == selection.focus {
            self.replacement_selection = None;
            return Some(removes_complete_text);
        }
        self.content
            .move_to(editor_cursor_from_document(selection.anchor));

        while self.iced_selection_focus() != selection.focus {
            let before = self.iced_selection_focus();
            let moves_right =
                (before.row, before.column) < (selection.focus.row, selection.focus.column);
            let motion = if moves_right {
                text_editor::Motion::Right
            } else {
                text_editor::Motion::Left
            };
            self.content.perform(text_editor::Action::Select(motion));
            let after = self.iced_selection_focus();
            let passed_focus = if moves_right {
                (after.row, after.column) > (selection.focus.row, selection.focus.column)
            } else {
                (after.row, after.column) < (selection.focus.row, selection.focus.column)
            };
            if after == before || passed_focus {
                self.content
                    .move_to(editor_cursor_from_document(original_cursor));
                return None;
            }
        }

        self.replacement_selection = None;
        Some(removes_complete_text)
    }

    #[cfg(test)]
    fn iced_selection_focus(&self) -> DocumentCursor {
        let position = self.cursor().position;
        DocumentCursor {
            row: position.line,
            column: position.column,
        }
    }
}
