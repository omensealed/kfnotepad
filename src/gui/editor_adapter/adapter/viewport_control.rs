impl GuiEditorAdapter {
    pub(crate) fn move_to(&mut self, cursor: DocumentCursor) {
        self.apply(GuiEditorCommand::MoveTo(cursor));
    }

    pub(crate) fn select_right_chars(&mut self, count: usize) {
        self.apply(GuiEditorCommand::SelectRightChars(count));
    }

    pub(crate) fn scroll_viewport_by_lines(&mut self, delta: i32) {
        let line_count = self.line_count();
        self.viewport.scroll_by(delta, line_count);
        let cursor = self.document_cursor();
        let visible_cursor = self.viewport.clamp_cursor_to_visible(cursor, line_count);
        if visible_cursor != cursor {
            self.content
                .move_to(editor_cursor_from_document(visible_cursor));
        }
        self.viewport_tracks_cursor = true;
    }

    pub(crate) fn scroll_viewport_by_lines_preserving_cursor(&mut self, delta: i32) {
        let line_count = self.line_count();
        self.viewport.scroll_by(delta, line_count);
        self.viewport_tracks_cursor = false;
    }
}
