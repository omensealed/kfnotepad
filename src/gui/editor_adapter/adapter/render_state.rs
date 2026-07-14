//! Render snapshots, line-number state, and viewport synchronization.

use super::*;

impl GuiEditorAdapter {
    pub(crate) fn render_state(
        &self,
        visible_line_numbers: usize,
        editor_font_size: u16,
    ) -> GuiEditorRenderState {
        GuiEditorRenderState {
            line_numbers: self.line_number_snapshot(visible_line_numbers, editor_font_size),
        }
    }

    pub(crate) fn line_number_snapshot(
        &self,
        visible_lines: usize,
        editor_font_size: u16,
    ) -> GuiEditorLineNumberSnapshot {
        let line_count = self.line_count();
        let viewport = self.viewport.with_visible_lines(visible_lines, line_count);
        let viewport = if self.viewport_tracks_cursor {
            viewport.with_cursor_visible_for_render(self.document_cursor(), line_count)
        } else {
            viewport
        };
        GuiEditorLineNumberSnapshot {
            line_count,
            gutter_start: viewport.first_line,
            text: gui_line_number_gutter_text(viewport.first_line, line_count, visible_lines),
            width: gui_line_number_gutter_width(line_count, editor_font_size),
        }
    }

    pub(crate) fn sync_viewport_to_cursor(&mut self) {
        self.viewport
            .keep_cursor_visible(self.document_cursor(), self.line_count());
        self.viewport_tracks_cursor = true;
    }

    pub(crate) fn render_viewport_slice_from_lines(
        &self,
        document_lines: &[String],
        visible_lines: usize,
    ) -> GuiEditorViewportSlice {
        let line_count = self.line_count();
        let total = line_count.max(1);
        let mut viewport = self.viewport;
        viewport.visible_lines = visible_lines.max(1);
        viewport.first_line = viewport.first_line.clamp(1, total);
        if self.viewport_tracks_cursor {
            viewport = viewport.with_cursor_visible_for_render(self.document_cursor(), line_count);
        }
        gui_editor_viewport_slice_from_lines(
            document_lines,
            line_count,
            viewport,
            self.document_cursor(),
            self.replacement_selection,
        )
    }
}
