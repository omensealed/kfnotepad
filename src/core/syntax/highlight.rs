//! Single-line and viewport syntax highlighting operations.

use syntect::easy::HighlightLines;

use super::{SyntaxHighlightedLine, SyntaxHighlightedLines, SyntaxHighlighter};
use crate::core::TextDocument;

impl SyntaxHighlighter {
    pub fn highlight_line(
        &self,
        document: &TextDocument,
        target_line: &str,
    ) -> SyntaxHighlightedLine {
        let syntax = self.syntax_for_document(document);
        if syntax.name == "Plain Text" {
            return None;
        }

        let mut highlighter = HighlightLines::new(syntax, &self.theme);
        let highlighted = highlighter
            .highlight_line(target_line, &self.syntax_set)
            .ok()?;
        Some(
            highlighted
                .into_iter()
                .map(|(style, segment)| (style.into(), segment.to_string()))
                .collect(),
        )
    }

    pub fn highlight_visible_lines(
        &self,
        document: &TextDocument,
        viewport_start: usize,
        visible_rows: usize,
    ) -> SyntaxHighlightedLines {
        let syntax = self.syntax_for_document(document);
        let visible_rows = visible_rows.max(1);
        if syntax.name == "Plain Text" {
            return document
                .buffer
                .lines()
                .iter()
                .skip(viewport_start)
                .take(visible_rows)
                .map(|_| None)
                .collect();
        }

        let mut highlighter = HighlightLines::new(syntax, &self.theme);
        let mut highlighted_lines = Vec::new();
        let end = viewport_start.saturating_add(visible_rows);

        for (index, line) in document.buffer.lines().iter().enumerate().take(end) {
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
            if index >= viewport_start {
                highlighted_lines.push(highlighted);
            }
        }

        highlighted_lines
    }
}
