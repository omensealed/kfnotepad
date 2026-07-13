use super::*;

#[cfg(test)]
pub(in crate::gui::app::state) fn gui_editor_replacement_copy_selection(
    document: &TextDocument,
    selection: Option<GuiEditorReplacementSelection>,
) -> Option<String> {
    let selected = gui_editor_replacement_selected_text(document, selection?)?;
    (!selected.is_empty()).then_some(selected)
}

#[cfg(test)]
pub(in crate::gui::app::state) fn gui_editor_replacement_cut_selection(
    document: &mut TextDocument,
    cursor: &mut DocumentCursor,
    viewport: &mut GuiEditorViewportState,
    selection: &mut Option<GuiEditorReplacementSelection>,
) -> Option<String> {
    let selected = gui_editor_replacement_copy_selection(document, *selection)?;
    delete_gui_editor_replacement_selection(document, cursor, selection);
    viewport.keep_cursor_visible(*cursor, document.buffer.line_count());
    Some(selected)
}

pub(in crate::gui::app::state) fn gui_editor_replacement_paste_text(
    document: &mut TextDocument,
    cursor: &mut DocumentCursor,
    viewport: &mut GuiEditorViewportState,
    selection: &mut Option<GuiEditorReplacementSelection>,
    text: &str,
) {
    let _ = gui_editor_replacement_paste_text_with_mode(
        document, cursor, viewport, selection, false, text,
    );
}

pub(in crate::gui::app::state) fn gui_editor_replacement_paste_text_with_mode(
    document: &mut TextDocument,
    cursor: &mut DocumentCursor,
    viewport: &mut GuiEditorViewportState,
    selection: &mut Option<GuiEditorReplacementSelection>,
    overwrite_mode: bool,
    text: &str,
) -> bool {
    if text.is_empty() {
        return false;
    }

    // Overwrite without a selection can stay at the current byte length. Let
    // TextBuffer perform its exact projected-size check for that case.
    if !overwrite_mode || selection.is_some() {
        let selected_bytes = selection
            .and_then(|selection| gui_editor_replacement_selected_text(document, selection))
            .map_or(0, |selected| selected.len());
        let projected_bytes = document
            .buffer
            .byte_len()
            .saturating_sub(selected_bytes)
            .saturating_add(text.len());
        if document.buffer.ensure_byte_len(projected_bytes).is_err() {
            return false;
        }
    }

    let revision_before = document.buffer.edit_revision();
    document.with_compound_edit(|document| {
        let deleted_selection =
            delete_gui_editor_replacement_selection(document, cursor, selection);
        if overwrite_mode && deleted_selection {
            let (first, remainder) = split_first_char(text);
            let inserted = if first == '\n' {
                document
                    .buffer
                    .insert_newline(cursor.row, cursor.column)
                    .map(|()| DocumentCursor {
                        row: cursor.row.saturating_add(1),
                        column: 0,
                    })
            } else {
                document
                    .buffer
                    .insert_char(cursor.row, cursor.column, first)
                    .map(|()| {
                        let inserted_end = cursor.column.saturating_add(1);
                        DocumentCursor {
                            row: cursor.row,
                            column: document
                                .buffer
                                .grapheme_range_end_boundary_column(cursor.row, inserted_end)
                                .unwrap_or(inserted_end),
                        }
                    })
            };
            if let Ok(next_cursor) = inserted {
                *cursor = next_cursor;
                if !remainder.is_empty() {
                    if let Ok(next_cursor) = document.buffer.overwrite_text(*cursor, remainder) {
                        *cursor = next_cursor;
                    }
                }
            }
        } else if overwrite_mode {
            if let Ok(next_cursor) = document.buffer.overwrite_text(*cursor, text) {
                *cursor = next_cursor;
            }
        } else if let Ok(next_cursor) = document.buffer.insert_text(*cursor, text) {
            *cursor = next_cursor;
        }
        viewport.keep_cursor_visible(*cursor, document.buffer.line_count());
    });
    document.buffer.edit_revision() != revision_before
}

fn split_first_char(text: &str) -> (char, &str) {
    let first = text.chars().next().expect("non-empty paste text");
    (first, &text[first.len_utf8()..])
}
