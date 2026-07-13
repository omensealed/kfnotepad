//! Plain-text fallback used when the `syntax` feature is disabled.

use std::path::Path;

use super::{
    SyntaxHighlightCacheState, SyntaxHighlightedLine, SyntaxHighlightedLines, SyntaxHighlighter,
};
use crate::core::TextDocument;

impl SyntaxHighlighter {
    pub fn highlight_line(
        &self,
        _document: &TextDocument,
        _target_line: &str,
    ) -> SyntaxHighlightedLine {
        None
    }

    pub fn highlight_visible_lines(
        &self,
        document: &TextDocument,
        viewport_start: usize,
        visible_rows: usize,
    ) -> SyntaxHighlightedLines {
        document
            .buffer
            .lines()
            .iter()
            .skip(viewport_start)
            .take(visible_rows.max(1))
            .map(|_| None)
            .collect()
    }

    pub fn highlight_lines_incremental(
        &self,
        document: &TextDocument,
        start_line: usize,
        visible_rows: usize,
        state: Option<SyntaxHighlightCacheState>,
    ) -> (SyntaxHighlightedLines, Option<SyntaxHighlightCacheState>) {
        self.highlight_lines_incremental_for_path(
            &document.path,
            document.buffer.lines(),
            start_line,
            visible_rows,
            state,
        )
    }

    pub fn highlight_lines_incremental_for_path(
        &self,
        _path: &Path,
        lines: &[String],
        start_line: usize,
        visible_rows: usize,
        _state: Option<SyntaxHighlightCacheState>,
    ) -> (SyntaxHighlightedLines, Option<SyntaxHighlightCacheState>) {
        (
            lines
                .iter()
                .skip(start_line)
                .take(visible_rows.max(1))
                .map(|_| None)
                .collect(),
            None,
        )
    }

    pub fn syntax_name_for_document(&self, _document: &TextDocument) -> &str {
        "Plain Text"
    }

    pub fn syntax_token_for_document(&self, _document: &TextDocument) -> String {
        "txt".to_string()
    }
}
