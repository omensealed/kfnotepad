pub(super) fn gui_editor_viewport_line_slice(
    line: &GuiEditorViewportLine,
    start: usize,
    end: usize,
) -> GuiEditorViewportLine {
    let line_columns = line.text.chars().count();
    let (start, end) = gui_editor_grapheme_slice_columns(&line.text, start, end);
    let row_text = line
        .text
        .chars()
        .skip(start)
        .take(end.saturating_sub(start))
        .collect::<String>();
    let cursor_column = line.cursor_column.and_then(|cursor| {
        if start == end {
            (cursor == start).then_some(0)
        } else if cursor >= start && cursor < end {
            Some(gui_editor_nearest_grapheme_column(
                &row_text,
                cursor.saturating_sub(start),
            ))
        } else if end == line_columns && cursor == end {
            Some(end - start)
        } else {
            None
        }
    });
    let selection = line.selection.and_then(|selection| {
        let selected_start = selection.start_column.max(start);
        let selected_end = selection.end_column.min(end);
        if selected_start >= selected_end {
            return None;
        }
        let (start_column, end_column) = gui_editor_grapheme_slice_columns(
            &row_text,
            selected_start.saturating_sub(start),
            selected_end.saturating_sub(start),
        );
        (start_column < end_column).then_some(GuiEditorSelectionSpan {
            start_column,
            end_column,
        })
    });
    let syntax_segments = gui_editor_slice_syntax_segments(line, start, end);

    GuiEditorViewportLine {
        number: line.number,
        text: row_text,
        cursor_column,
        selection,
        syntax_segments,
    }
}

fn gui_editor_grapheme_slice_columns(text: &str, start: usize, end: usize) -> (usize, usize) {
    use unicode_segmentation::UnicodeSegmentation;

    let line_columns = text.chars().count();
    let mut start = start.min(line_columns);
    let mut end = end.min(line_columns).max(start);
    for (byte_index, grapheme) in text.grapheme_indices(true) {
        let grapheme_start = text[..byte_index].chars().count();
        let grapheme_end = grapheme_start + grapheme.chars().count();
        if grapheme_start < start && start < grapheme_end {
            start = grapheme_start;
        }
        if grapheme_start < end && end < grapheme_end {
            end = grapheme_end;
        }
    }
    (start, end)
}

fn gui_editor_nearest_grapheme_column(text: &str, column: usize) -> usize {
    use unicode_segmentation::UnicodeSegmentation;

    let line_columns = text.chars().count();
    let column = column.min(line_columns);
    for (byte_index, grapheme) in text.grapheme_indices(true) {
        let start = text[..byte_index].chars().count();
        let end = start + grapheme.chars().count();
        if start <= column && column <= end {
            let before = column.saturating_sub(start);
            let after = end.saturating_sub(column);
            return if before < after { start } else { end };
        }
    }
    line_columns
}

pub(super) fn gui_editor_slice_syntax_segments(
    line: &GuiEditorViewportLine,
    start: usize,
    end: usize,
) -> Option<Vec<GuiEditorSyntaxSegment>> {
    line.syntax_segments.as_ref()?;
    if start == end {
        return Some(Vec::new());
    }
    let colors = gui_editor_line_syntax_colors(line);
    let line_chars = line.text.chars().collect::<Vec<_>>();
    let row_colors = colors.get(start..end)?;
    if row_colors.iter().any(Option::is_none) {
        return None;
    }

    let mut segments = Vec::new();
    let mut current_text = String::new();
    let mut current_color = row_colors.first().and_then(|color| *color)?;
    for (character, color) in line_chars[start..end]
        .iter()
        .copied()
        .zip(row_colors.iter())
    {
        let color = color.unwrap_or(current_color);
        if !current_text.is_empty() && color != current_color {
            segments.push(GuiEditorSyntaxSegment {
                text: std::mem::take(&mut current_text),
                color: current_color,
            });
            current_color = color;
        }
        current_text.push(character);
    }
    if !current_text.is_empty() {
        segments.push(GuiEditorSyntaxSegment {
            text: current_text,
            color: current_color,
        });
    }

    Some(segments)
}
