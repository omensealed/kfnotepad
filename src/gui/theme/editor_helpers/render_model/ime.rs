pub(super) fn gui_editor_viewport_line_with_ime_preedit(
    mut line: GuiEditorViewportLine,
    preedit: Option<&GuiImePreedit>,
) -> GuiEditorViewportLine {
    let Some(preedit) = preedit else {
        return line;
    };
    if preedit.content.is_empty() {
        return line;
    }
    let Some(cursor_column) = line.cursor_column else {
        return line;
    };

    let line_columns = line.text.chars().count();
    let cursor_column = cursor_column.min(line_columns);
    let before = line.text.chars().take(cursor_column).collect::<String>();
    let after = line.text.chars().skip(cursor_column).collect::<String>();
    let preedit_columns = preedit.content.chars().count();
    let (selected_start, selected_end) =
        gui_ime_preedit_selection_columns(preedit).unwrap_or((0, preedit_columns));

    line.text = format!("{before}{}{after}", preedit.content);
    line.cursor_column = None;
    line.selection = Some(GuiEditorSelectionSpan {
        start_column: cursor_column.saturating_add(selected_start.min(preedit_columns)),
        end_column: cursor_column
            .saturating_add(selected_end.min(preedit_columns))
            .max(cursor_column.saturating_add(1)),
    });
    line.syntax_segments = None;
    line
}

pub(super) fn gui_ime_preedit_selection_columns(preedit: &GuiImePreedit) -> Option<(usize, usize)> {
    use unicode_segmentation::UnicodeSegmentation;

    let selection = preedit.selection.as_ref()?;
    let mut start = gui_byte_index_to_char_column(&preedit.content, selection.start);
    let mut end = gui_byte_index_to_char_column(&preedit.content, selection.end).max(start);
    for (byte_index, grapheme) in preedit.content.grapheme_indices(true) {
        let grapheme_start = preedit.content[..byte_index].chars().count();
        let grapheme_end = grapheme_start + grapheme.chars().count();
        if grapheme_start < start && start < grapheme_end {
            start = grapheme_start;
        }
        if grapheme_start < end && end < grapheme_end {
            end = grapheme_end;
        }
    }
    Some((start, end))
}

pub(super) fn gui_byte_index_to_char_column(text: &str, byte_index: usize) -> usize {
    let clamped = byte_index.min(text.len());
    text.char_indices()
        .take_while(|(index, _)| *index < clamped)
        .count()
}
