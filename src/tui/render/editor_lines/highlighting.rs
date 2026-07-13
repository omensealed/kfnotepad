//! Visible-line syntax highlighting and grapheme-safe segment rendering.

use super::*;

pub(super) fn highlighter_lines_for_wrapped_view(
    document: &TextDocument,
    view: EditorView<'_>,
    highlighter: &SyntaxHighlighter,
    syntax_cache: Option<&mut TuiSyntaxHighlightCache>,
    body_width: usize,
) -> SyntaxHighlightedLines {
    let visible_source_lines = wrapped_visible_source_line_count(document, view, body_width);
    highlight_lines_for_render(
        document,
        view.viewport_start,
        visible_source_lines,
        highlighter,
        syntax_cache,
    )
}

pub(crate) fn highlight_lines_for_render(
    document: &TextDocument,
    start_line: usize,
    visible_rows: usize,
    highlighter: &SyntaxHighlighter,
    syntax_cache: Option<&mut TuiSyntaxHighlightCache>,
) -> SyntaxHighlightedLines {
    if let Some(cache) = syntax_cache {
        cache.highlight(document, start_line, visible_rows, highlighter)
    } else {
        highlighter.highlight_visible_lines(document, start_line, visible_rows)
    }
}

fn wrapped_visible_source_line_count(
    document: &TextDocument,
    view: EditorView<'_>,
    body_width: usize,
) -> usize {
    let mut visual_rows = 0usize;
    let mut source_rows = 0usize;
    for line in document.buffer.lines().iter().skip(view.viewport_start) {
        if visual_rows >= view.visible_rows {
            break;
        }
        visual_rows += wrapped_line_chunk_count(line, body_width);
        source_rows += 1;
    }
    source_rows.max(1)
}
pub(super) fn write_highlighted_line_window(
    writer: &mut impl Write,
    segments: Vec<(SyntaxStyle, String)>,
    start_column: usize,
    remaining: &mut usize,
    syntax_theme_id: EditorThemeId,
    search_ranges: &[std::ops::Range<usize>],
    frame: RenderFrame,
) -> io::Result<()> {
    let segments = grapheme_safe_highlight_segments(segments);
    let mut skipped_columns = 0usize;
    let mut source_column = 0usize;
    for (style, segment) in segments {
        if *remaining == 0 {
            break;
        }
        let segment_columns = line_segment_display_width(&segment, skipped_columns);
        if skipped_columns + segment_columns <= start_column {
            skipped_columns += segment_columns;
            source_column += segment.chars().count();
            continue;
        }
        queue_set_foreground_color(
            writer,
            &frame,
            syntax_color_to_terminal(style.foreground, syntax_theme_id),
        )?;
        print_line_window_with_search(
            writer,
            LineWindowSearchView {
                text: &segment,
                start_column,
                display_column: &mut skipped_columns,
                source_column: &mut source_column,
                remaining_columns: remaining,
                search_ranges,
                base_fg: Some(syntax_color_to_terminal(style.foreground, syntax_theme_id)),
                frame,
            },
        )?;
    }
    queue!(writer, ResetColor)
}

pub(crate) fn grapheme_safe_highlight_segments(
    segments: Vec<(SyntaxStyle, String)>,
) -> Vec<(SyntaxStyle, String)> {
    use unicode_segmentation::UnicodeSegmentation;

    let mut text = String::new();
    let mut styles = Vec::new();
    for (style, segment) in segments {
        for character in segment.chars() {
            text.push(character);
            styles.push(style);
        }
    }

    let mut safe = Vec::<(SyntaxStyle, String)>::new();
    for (byte_index, grapheme) in text.grapheme_indices(true) {
        let start_column = text[..byte_index].chars().count();
        let Some(style) = styles.get(start_column).copied() else {
            continue;
        };
        if let Some((previous_style, previous_text)) = safe.last_mut() {
            if *previous_style == style {
                previous_text.push_str(grapheme);
                continue;
            }
        }
        safe.push((style, grapheme.to_string()));
    }
    safe
}
