pub(super) fn gui_editor_viewport_selection_span(
    line: &str,
    row: usize,
    selection: Option<GuiEditorReplacementSelection>,
) -> Option<GuiEditorSelectionSpan> {
    let (start, end) = selection?.normalized();
    if start == end || row < start.row || row > end.row {
        return None;
    }

    let line_columns = line.chars().count();
    let start_column = if row == start.row {
        start.column.min(line_columns)
    } else {
        0
    };
    let end_column = if row == end.row {
        end.column.min(line_columns)
    } else {
        line_columns
    };

    if row == start.row && row == end.row && start_column == end_column {
        return None;
    }

    Some(GuiEditorSelectionSpan {
        start_column,
        end_column,
    })
}

#[cfg(test)]
pub(super) fn gui_editor_read_only_render_model(
    slice: &GuiEditorViewportSlice,
) -> GuiEditorReadOnlyRenderModel {
    let cursor = slice
        .lines
        .iter()
        .enumerate()
        .find_map(|(index, line)| line.cursor_column.map(|column| (index, column)));

    GuiEditorReadOnlyRenderModel {
        line_count: slice.line_count,
        first_line: slice.first_line,
        gutter_text: slice
            .lines
            .iter()
            .map(|line| line.number.to_string())
            .collect::<Vec<_>>()
            .join("\n"),
        body_text: slice
            .lines
            .iter()
            .map(|line| line.text.as_str())
            .collect::<Vec<_>>()
            .join("\n"),
        cursor_row_in_view: cursor.map(|(row, _column)| row),
        cursor_column: cursor.map(|(_row, column)| column),
    }
}
