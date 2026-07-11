impl GuiEditorAdapter {
    pub(crate) fn text(&self) -> String {
        self.content.text()
    }

    pub(crate) fn clone_for_relayout(&self) -> Self {
        let mut adapter = Self {
            content: self.content.clone(),
            viewport: self.viewport,
            viewport_tracks_cursor: self.viewport_tracks_cursor,
            replacement_selection: self.replacement_selection,
        };
        adapter.content.move_to(self.cursor());
        adapter
    }

    pub(crate) fn cursor(&self) -> text_editor::Cursor {
        self.content.cursor()
    }

    pub(crate) fn document_cursor(&self) -> DocumentCursor {
        document_cursor_from_editor(self.cursor())
    }

    pub(crate) fn selection(&self) -> Option<String> {
        if let Some(selection) = self.replacement_selection {
            return gui_editor_replacement_copy_selection_from_text(&self.text(), Some(selection));
        }
        self.content.selection()
    }

    pub(crate) fn line_count(&self) -> usize {
        self.content.line_count()
    }
}
