//! Rendering state and helpers for individual wrapped line chunks.

use super::*;

pub(super) fn clear_remaining_editor_rows(
    writer: &mut impl Write,
    mut screen_row: u16,
    visible_rows: usize,
    frame: RenderFrame,
) -> io::Result<()> {
    let end_row = frame.body_top + visible_rows as u16;
    while screen_row < end_row {
        clear_editor_body_row(writer, screen_row, frame)?;
        screen_row += 1;
    }
    Ok(())
}

pub(super) struct WrappedEditorChunkView<'a> {
    pub(super) screen_row: u16,
    pub(super) document_row: usize,
    pub(super) chunk_index: usize,
    pub(super) line: &'a str,
    pub(super) chunk: &'a str,
    pub(super) chunk_start_column: usize,
    pub(super) highlighted_line: Option<Vec<(SyntaxStyle, String)>>,
    pub(super) settings: EditorSettings,
    pub(super) search_highlight: Option<SearchHighlightView<'a>>,
}

pub(super) fn write_wrapped_editor_chunk(
    writer: &mut impl Write,
    view: WrappedEditorChunkView<'_>,
    frame: RenderFrame,
) -> io::Result<()> {
    let mut remaining = frame.terminal_width;
    clear_editor_body_row(writer, view.screen_row, frame)?;
    if view.settings.show_line_numbers {
        queue_set_foreground_color(writer, &frame, frame.theme.gutter_fg)?;
        let gutter = if view.chunk_index == 0 {
            format!(
                "{:>width$} ",
                view.document_row + 1,
                width = frame.gutter_width
            )
        } else {
            format!("{:>width$} ", "", width = frame.gutter_width)
        };
        print_truncated(writer, &gutter, &mut remaining)?;
        queue!(writer, ResetColor)?;
    }
    write_editor_body_padding(writer, &mut remaining)?;
    let search_ranges = view
        .search_highlight
        .map(|highlight| search_match_ranges(view.line, highlight.query, highlight.mode))
        .unwrap_or_default();
    if let Some(segments) = view.highlighted_line {
        let mut chunk_remaining = remaining.min(text_display_width(view.chunk));
        write_highlighted_line_window(
            writer,
            segments,
            view.chunk_start_column,
            &mut chunk_remaining,
            view.settings.syntax_theme_id,
            &search_ranges,
            frame,
        )
    } else {
        let mut display_column = 0;
        let mut source_column = 0;
        print_line_window_with_search(
            writer,
            LineWindowSearchView {
                text: view.chunk,
                start_column: 0,
                display_column: &mut display_column,
                source_column: &mut source_column,
                remaining_columns: &mut remaining,
                search_ranges: &search_ranges,
                base_fg: None,
                frame,
            },
        )
    }
}
