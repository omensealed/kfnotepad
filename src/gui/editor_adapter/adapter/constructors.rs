impl GuiEditorAdapter {
    pub(crate) fn new(content: text_editor::Content) -> Self {
        let mut adapter = Self {
            content,
            viewport: GuiEditorViewportState::new(GUI_LINE_NUMBER_GUTTER_VISIBLE_LINES),
            viewport_tracks_cursor: true,
            replacement_selection: None,
        };
        adapter.sync_viewport_to_cursor();
        adapter
    }

    #[cfg(test)]
    pub(crate) fn from_text(text: &str) -> Self {
        Self::new(text_editor::Content::with_text(text))
    }
}
