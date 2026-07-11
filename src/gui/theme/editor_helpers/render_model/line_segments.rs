pub(super) fn gui_editor_read_only_line_segments(
    line: &GuiEditorViewportLine,
) -> Vec<GuiEditorReadOnlyLineSegment> {
    use unicode_segmentation::UnicodeSegmentation;

    let line_columns = line.text.chars().count();
    let syntax_colors = gui_editor_line_syntax_colors(line);
    let overlay = line
        .selection
        .map(|selection| {
            (
                selection.start_column.min(line_columns),
                selection.end_column.min(line_columns),
            )
        })
        .or_else(|| {
            line.cursor_column.map(|cursor_column| {
                let cursor = cursor_column.min(line_columns);
                (cursor, cursor.saturating_add(1).min(line_columns))
            })
        });

    if line_columns == 0 {
        return vec![GuiEditorReadOnlyLineSegment {
            text: if overlay.is_some() {
                " ".to_string()
            } else {
                String::new()
            },
            selected: overlay.is_some(),
            syntax_color: None,
        }];
    }

    let mut segments = Vec::new();
    let mut current_text = String::new();
    let mut current_selected = false;
    let mut current_color = None;

    for (byte_index, grapheme) in line.text.grapheme_indices(true) {
        let start_column = line.text[..byte_index].chars().count();
        let end_column = start_column + grapheme.chars().count();
        let selected = overlay.is_some_and(|(start, end)| {
            let end = end.max(start + 1);
            start < end_column && start_column < end
        });
        let syntax_color = if selected {
            None
        } else {
            syntax_colors.get(start_column).copied().flatten()
        };

        if current_text.is_empty() {
            current_selected = selected;
            current_color = syntax_color;
        } else if current_selected != selected || current_color != syntax_color {
            gui_editor_push_read_only_segment(
                &mut segments,
                &mut current_text,
                current_selected,
                current_color,
            );
            current_selected = selected;
            current_color = syntax_color;
        }
        current_text.push_str(grapheme);
    }

    gui_editor_push_read_only_segment(
        &mut segments,
        &mut current_text,
        current_selected,
        current_color,
    );

    if overlay.is_some_and(|(start, end)| start == end && start >= line_columns) {
        segments.push(GuiEditorReadOnlyLineSegment {
            text: " ".to_string(),
            selected: true,
            syntax_color: None,
        });
    }

    segments
}
