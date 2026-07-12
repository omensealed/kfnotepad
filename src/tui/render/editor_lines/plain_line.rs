//! Unwrapped editor-line rendering.

use super::*;

pub(crate) fn write_editor_line(
    writer: &mut impl Write,
    view: EditorLineView<'_>,
    frame: RenderFrame,
) -> io::Result<()> {
    let mut remaining = frame.terminal_width;
    clear_editor_body_row(writer, view.screen_row, frame)?;
    if view.settings.show_line_numbers {
        queue_set_foreground_color(writer, &frame, frame.theme.gutter_fg)?;
        print_truncated(
            writer,
            &format!(
                "{:>width$} ",
                view.document_row + 1,
                width = frame.gutter_width
            ),
            &mut remaining,
        )?;
        queue!(writer, ResetColor)?;
    }
    write_editor_body_padding(writer, &mut remaining)?;

    let search_ranges = view
        .search_highlight
        .map(|highlight| search_match_ranges(view.line, highlight.query, highlight.mode))
        .unwrap_or_default();
    if let Some(segments) = view.highlighted_line {
        write_highlighted_line_window(
            writer,
            segments,
            view.horizontal_offset,
            &mut remaining,
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
                text: view.line,
                start_column: view.horizontal_offset,
                display_column: &mut display_column,
                source_column: &mut source_column,
                remaining_columns: &mut remaining,
                search_ranges: &search_ranges,
                base_fg: None,
                frame,
            },
        )?;
        Ok(())
    }
}
