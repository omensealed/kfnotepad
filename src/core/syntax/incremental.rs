//! Incremental syntax highlighting with reusable parser and theme state.

use std::path::Path;

use syntect::easy::HighlightLines;

use super::{SyntaxHighlightCacheState, SyntaxHighlightedLines, SyntaxHighlighter};
use crate::core::TextDocument;

impl SyntaxHighlighter {
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
        path: &Path,
        lines: &[String],
        start_line: usize,
        visible_rows: usize,
        state: Option<SyntaxHighlightCacheState>,
    ) -> (SyntaxHighlightedLines, Option<SyntaxHighlightCacheState>) {
        let syntax = self.syntax_for_path(path);
        let visible_rows = visible_rows.max(1);
        if syntax.name == "Plain Text" {
            return (
                lines
                    .iter()
                    .skip(start_line)
                    .take(visible_rows)
                    .map(|_| None)
                    .collect(),
                None,
            );
        }

        let has_cached_state = state.is_some();
        let mut highlighter = match state {
            Some(state) => {
                HighlightLines::from_state(&self.theme, state.highlight_state, state.parse_state)
            }
            None => HighlightLines::new(syntax, &self.theme),
        };

        let mut highlighted_lines = Vec::new();
        if !has_cached_state {
            for line in lines.iter().take(start_line) {
                let _ = highlighter.highlight_line(line, &self.syntax_set);
            }
        }
        for line in lines.iter().skip(start_line).take(visible_rows) {
            let highlighted =
                highlighter
                    .highlight_line(line, &self.syntax_set)
                    .ok()
                    .map(|segments| {
                        segments
                            .into_iter()
                            .map(|(style, segment)| (style.into(), segment.to_string()))
                            .collect()
                    });
            highlighted_lines.push(highlighted);
        }

        let (highlight_state, parse_state) = highlighter.state();
        (
            highlighted_lines,
            Some(SyntaxHighlightCacheState {
                highlight_state,
                parse_state,
            }),
        )
    }
}
