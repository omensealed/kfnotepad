//! Incremental syntax-highlight cache for the visible TUI viewport.

use super::*;

#[derive(Default)]
pub(crate) struct TuiSyntaxHighlightCache {
    path: PathBuf,
    revision: u64,
    start_line: usize,
    visible_rows: usize,
    pub(crate) lines: SyntaxHighlightedLines,
    state_before_start: Option<SyntaxHighlightCacheState>,
    state_after_end: Option<SyntaxHighlightCacheState>,
    next_start_line: usize,
    valid: bool,
}

impl TuiSyntaxHighlightCache {
    fn matches_document(&self, document: &TextDocument) -> bool {
        self.valid && self.path == document.path && self.revision == document.buffer.edit_revision()
    }

    fn reset_for_document(&mut self, document: &TextDocument) {
        self.path = document.path.clone();
        self.revision = document.buffer.edit_revision();
        self.start_line = 0;
        self.visible_rows = 0;
        self.lines.clear();
        self.state_before_start = None;
        self.state_after_end = None;
        self.next_start_line = 0;
        self.valid = true;
    }

    pub(crate) fn highlight(
        &mut self,
        document: &TextDocument,
        start_line: usize,
        visible_rows: usize,
        highlighter: &SyntaxHighlighter,
    ) -> SyntaxHighlightedLines {
        let visible_rows = visible_rows.max(1);
        if self.matches_document(document)
            && self.start_line == start_line
            && self.visible_rows == visible_rows
        {
            return self.lines.clone();
        }

        if !self.matches_document(document) {
            self.reset_for_document(document);
        }

        let state_before_start = if start_line == self.next_start_line {
            self.state_after_end.clone()
        } else if start_line >= self.start_line {
            let advance_by = start_line.saturating_sub(self.start_line);
            if advance_by == 0 {
                self.state_before_start.clone()
            } else {
                let (_, state) = highlighter.highlight_lines_incremental(
                    document,
                    self.start_line,
                    advance_by,
                    self.state_before_start.clone(),
                );
                state
            }
        } else {
            None
        };
        let (lines, state_after_end) = highlighter.highlight_lines_incremental(
            document,
            start_line,
            visible_rows,
            state_before_start.clone(),
        );

        self.start_line = start_line;
        self.visible_rows = visible_rows;
        self.next_start_line = start_line.saturating_add(visible_rows);
        self.state_before_start = state_before_start;
        self.state_after_end = state_after_end;
        self.lines = lines.clone();
        lines
    }
}
