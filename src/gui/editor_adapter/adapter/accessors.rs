//! Text, cursor, selection, and document metadata accessors.

use super::*;

impl GuiEditorAdapter {
    pub(crate) fn clone_for_relayout(&self) -> Self {
        Self {
            cursor: self.cursor,
            line_count: self.line_count,
            viewport: self.viewport,
            viewport_tracks_cursor: self.viewport_tracks_cursor,
            replacement_selection: self.replacement_selection,
            visual_layout_cache: self.visual_layout_cache.clone(),
        }
    }

    pub(crate) fn document_cursor(&self) -> DocumentCursor {
        self.cursor
    }

    pub(crate) fn line_count(&self) -> usize {
        self.line_count
    }

    pub(crate) fn sync_document_metadata(&mut self, line_count: usize, cursor: DocumentCursor) {
        self.line_count = line_count.max(1);
        self.cursor = cursor;
        self.sync_viewport_to_cursor();
    }

    #[cfg(test)]
    pub(crate) fn visual_layout_cache_stats(&self) -> (usize, usize) {
        let cache = self
            .visual_layout_cache
            .lock()
            .expect("visual layout cache lock");
        (cache.hits, cache.misses)
    }
}
