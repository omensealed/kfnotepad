pub(crate) struct LineWindowSearchView<'a> {
    pub(crate) text: &'a str,
    pub(crate) start_column: usize,
    pub(crate) display_column: &'a mut usize,
    pub(crate) source_column: &'a mut usize,
    pub(crate) remaining_columns: &'a mut usize,
    pub(crate) search_ranges: &'a [std::ops::Range<usize>],
    pub(crate) base_fg: Option<Color>,
    pub(crate) frame: RenderFrame,
}

pub(crate) fn print_line_window_with_search(
    writer: &mut impl Write,
    view: LineWindowSearchView<'_>,
) -> io::Result<()> {
    use unicode_segmentation::UnicodeSegmentation;

    let mut search_paint_active = false;
    for grapheme in view.text.graphemes(true) {
        if *view.remaining_columns == 0 {
            break;
        }

        let grapheme_width = grapheme_display_width(grapheme, *view.display_column);
        let grapheme_start = *view.display_column;
        let grapheme_end = grapheme_start + grapheme_width;
        *view.display_column = grapheme_end;
        let current_source_column = *view.source_column;
        let grapheme_source_end = current_source_column + grapheme.chars().count();
        *view.source_column = grapheme_source_end;

        if grapheme_width > 0 && grapheme_end <= view.start_column {
            continue;
        }
        if grapheme_width > *view.remaining_columns {
            break;
        }

        let in_match = view.search_ranges.iter().any(|range| {
            range.start < grapheme_source_end && current_source_column < range.end
        });
        if in_match {
            queue_set_foreground_color(writer, &view.frame, view.frame.theme.search_fg)?;
            queue_set_background_color(writer, &view.frame, view.frame.theme.search_bg)?;
            search_paint_active = true;
        } else if search_paint_active {
            queue!(writer, ResetColor)?;
            if let Some(base_fg) = view.base_fg {
                queue_set_foreground_color(writer, &view.frame, base_fg)?;
            }
            search_paint_active = false;
        }

        if grapheme == "\t" {
            queue!(writer, Print(" ".repeat(grapheme_width)))?;
        } else {
            queue!(writer, Print(grapheme))?;
        }
        *view.remaining_columns = view.remaining_columns.saturating_sub(grapheme_width);
    }
    if search_paint_active {
        queue!(writer, ResetColor)?;
        if let Some(base_fg) = view.base_fg {
            queue_set_foreground_color(writer, &view.frame, base_fg)?;
        }
    }
    Ok(())
}
