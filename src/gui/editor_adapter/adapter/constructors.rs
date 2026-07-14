//! Editor adapter construction and test fixtures.

use super::*;

impl GuiEditorAdapter {
    pub(crate) fn new(line_count: usize, cursor: DocumentCursor) -> Self {
        let mut adapter = Self {
            cursor,
            line_count: line_count.max(1),
            viewport: GuiEditorViewportState::new(GUI_LINE_NUMBER_GUTTER_VISIBLE_LINES),
            viewport_tracks_cursor: true,
            replacement_selection: None,
        };
        adapter.sync_viewport_to_cursor();
        adapter
    }

    #[cfg(test)]
    pub(crate) fn from_text(text: &str) -> Self {
        let buffer = TextBuffer::from_text(text);
        Self::new(
            gui_editor_line_count(&buffer),
            DocumentCursor { row: 0, column: 0 },
        )
    }
}
