#[cfg(test)]
pub(super) fn gui_editor_viewport_slice(
    text: &str,
    line_count: usize,
    viewport: GuiEditorViewportState,
    cursor: DocumentCursor,
    selection: Option<GuiEditorReplacementSelection>,
) -> GuiEditorViewportSlice {
    let document_lines = gui_document_lines(text, line_count);
    gui_editor_viewport_slice_from_lines(&document_lines, line_count, viewport, cursor, selection)
}

pub(super) fn gui_editor_viewport_slice_from_lines(
    document_lines: &[String],
    line_count: usize,
    viewport: GuiEditorViewportState,
    cursor: DocumentCursor,
    selection: Option<GuiEditorReplacementSelection>,
) -> GuiEditorViewportSlice {
    let total = line_count.max(1);
    let first_line = viewport.first_line.clamp(1, total);
    let last_line = viewport.last_visible_line(total);

    let lines = (first_line..=last_line)
        .map(|number| {
            let row = number.saturating_sub(1);
            GuiEditorViewportLine {
                number,
                text: document_lines.get(row).cloned().unwrap_or_default(),
                cursor_column: (cursor.row == row).then_some(cursor.column),
                syntax_segments: None,
                selection: gui_editor_viewport_selection_span(
                    document_lines
                        .get(row)
                        .map(String::as_str)
                        .unwrap_or_default(),
                    row,
                    selection,
                ),
            }
        })
        .collect();

    GuiEditorViewportSlice {
        line_count: total,
        first_line,
        lines,
    }
}

#[cfg(test)]
pub(super) fn gui_document_lines(text: &str, line_count: usize) -> Vec<String> {
    let total = line_count.max(1);
    let mut lines = text
        .split('\n')
        .map(ToString::to_string)
        .collect::<Vec<_>>();
    lines.resize(total, String::new());
    lines.truncate(total);
    lines
}

pub(super) fn gui_editor_viewport_slice_with_cached_syntax(
    mut slice: GuiEditorViewportSlice,
    syntax_cache: Option<&GuiSyntaxCache>,
) -> GuiEditorViewportSlice {
    let Some(cache) = syntax_cache else {
        return slice;
    };

    for line in &mut slice.lines {
        let row = line.number.saturating_sub(1);
        line.syntax_segments = cache.lines.get(row).cloned().flatten();
    }

    slice
}
