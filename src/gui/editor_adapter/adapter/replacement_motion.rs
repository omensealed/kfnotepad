//! Replacement editor cursor and page motion.

use super::*;

impl GuiEditorAdapter {
    pub(crate) fn apply_text_editor_motion_to_replacement(&mut self, motion: text_editor::Motion) {
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
                self.replacement_selection = None;
                self.sync_viewport_to_cursor();
            }
            text_editor::Motion::PageUp => {
                self.scroll_viewport_by_lines(-(GUI_LINE_NUMBER_GUTTER_VISIBLE_LINES as i32));
            }
            text_editor::Motion::PageDown => {
                self.scroll_viewport_by_lines(GUI_LINE_NUMBER_GUTTER_VISIBLE_LINES as i32);
            }
        }
    }
}
