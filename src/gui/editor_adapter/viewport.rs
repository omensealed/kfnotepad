#[derive(Debug, Clone, PartialEq)]
pub(crate) struct GuiEditorLineNumberSnapshot {
    pub(crate) line_count: usize,
    pub(crate) gutter_start: usize,
    pub(crate) text: String,
    pub(crate) width: f32,
}

impl GuiEditorViewportState {
    pub(crate) fn new(visible_lines: usize) -> Self {
        Self {
            first_line: 1,
            visible_lines: visible_lines.max(1),
        }
    }

    pub(crate) fn with_visible_lines(mut self, visible_lines: usize, line_count: usize) -> Self {
        self.visible_lines = visible_lines.max(1);
        self.clamp_to_line_count(line_count);
        self
    }

    pub(crate) fn with_cursor_visible_for_render(
        mut self,
        cursor: DocumentCursor,
        line_count: usize,
    ) -> Self {
        let total = line_count.max(1);
        self.first_line = self.first_line.clamp(1, total);
        let cursor_line = (cursor.row + 1).clamp(1, total);
        let last_visible = self
            .first_line
            .saturating_add(self.visible_lines.saturating_sub(1));
        if cursor_line < self.first_line {
            self.first_line = cursor_line;
        } else if cursor_line > last_visible {
            self.first_line = cursor_line.saturating_sub(self.visible_lines.saturating_sub(1));
        }
        self.first_line = self.first_line.clamp(1, total);
        self
    }

    pub(crate) fn scroll_by(&mut self, delta: i32, line_count: usize) {
        let current = self.first_line as i64;
        let next = current + i64::from(delta);
        self.first_line = next.max(1) as usize;
        self.clamp_to_line_count(line_count);
    }

    pub(crate) fn keep_cursor_visible(&mut self, cursor: DocumentCursor, line_count: usize) {
        self.clamp_to_line_count(line_count);
        let total = line_count.max(1);
        let cursor_line = (cursor.row + 1).clamp(1, total);
        let last_visible = self
            .first_line
            .saturating_add(self.visible_lines.saturating_sub(1));

        if cursor_line < self.first_line {
            self.first_line = cursor_line;
        } else if cursor_line > last_visible {
            self.first_line = cursor_line.saturating_sub(self.visible_lines.saturating_sub(1));
        }

        self.clamp_to_line_count(line_count);
    }

    pub(crate) fn clamp_cursor_to_visible(
        &self,
        cursor: DocumentCursor,
        line_count: usize,
    ) -> DocumentCursor {
        let total = line_count.max(1);
        let cursor_line = (cursor.row + 1).clamp(1, total);
        let first = self.first_line.clamp(1, total);
        let last = self.last_visible_line(line_count);
        let clamped_line = cursor_line.clamp(first, last);

        DocumentCursor {
            row: clamped_line.saturating_sub(1),
            column: cursor.column,
        }
    }

    pub(crate) fn last_visible_line(&self, line_count: usize) -> usize {
        let total = line_count.max(1);
        self.first_line
            .saturating_add(self.visible_lines.saturating_sub(1))
            .min(total)
    }

    pub(crate) fn clamp_to_line_count(&mut self, line_count: usize) {
        let total = line_count.max(1);
        let max_first = total
            .saturating_sub(self.visible_lines.saturating_sub(1))
            .max(1);
        self.first_line = self.first_line.clamp(1, max_first);
    }
}
