use super::*;
use unicode_segmentation::UnicodeSegmentation;

pub(in crate::gui::app::state) fn gui_editor_read_only_visual_rows(
    lines: &[GuiEditorViewportLine],
    first_line: usize,
    wrapping: Wrapping,
    body_columns: usize,
) -> Vec<GuiEditorReadOnlyVisualRow> {
    let mut rows = Vec::new();
    let wrap_columns = body_columns.max(1);

    for line in lines {
        let viewport_row = line.number.saturating_sub(first_line);
        let ranges = if wrapping == Wrapping::None {
            vec![(0, line.text.chars().count())]
        } else {
            gui_editor_word_wrap_ranges(&line.text, wrap_columns)
        };

        for (index, (start, end)) in ranges.into_iter().enumerate() {
            rows.push(GuiEditorReadOnlyVisualRow {
                line: gui_editor_viewport_line_slice(line, start, end),
                viewport_row,
                source_column_start: start,
                show_line_number: index == 0,
            });
        }
    }

    rows
}

pub(in crate::gui::app::state) fn gui_editor_word_wrap_ranges(
    text: &str,
    max_columns: usize,
) -> Vec<(usize, usize)> {
    let max_columns = max_columns.max(1);
    let graphemes = gui_editor_grapheme_wrap_units(text);
    let len = graphemes.len();
    if len == 0 {
        return vec![(0, 0)];
    }

    let mut ranges = Vec::new();
    let mut start = 0;
    while start < len {
        let hard_end = gui_editor_display_width_hard_end(&graphemes, start, max_columns);
        if hard_end >= len {
            ranges.push((graphemes[start].start_column, text.chars().count()));
            break;
        }

        let break_at = (start + 1..hard_end)
            .rev()
            .find(|index| graphemes[*index].is_whitespace)
            .filter(|index| index.saturating_sub(start) >= max_columns / 3)
            .map(|index| index + 1)
            .unwrap_or(hard_end);
        let end = break_at.max(start + 1);
        ranges.push((graphemes[start].start_column, graphemes[end - 1].end_column));
        start = break_at.max(start + 1);
    }

    ranges
}

#[derive(Clone, Copy)]
pub(in crate::gui::app::state) struct GuiEditorGraphemeWrapUnit {
    start_column: usize,
    end_column: usize,
    display_width: usize,
    is_whitespace: bool,
}

pub(in crate::gui::app::state) fn gui_editor_display_width_hard_end(
    graphemes: &[GuiEditorGraphemeWrapUnit],
    start: usize,
    max_columns: usize,
) -> usize {
    let mut end = start;
    let mut width = 0usize;
    while end < graphemes.len() {
        let next_width = graphemes[end].display_width;
        if end > start && width.saturating_add(next_width) > max_columns {
            break;
        }
        width = width.saturating_add(next_width);
        end += 1;
    }

    end.max(start.saturating_add(1)).min(graphemes.len())
}

pub(in crate::gui::app::state) fn gui_editor_char_display_width(character: char) -> usize {
    if character == '\t' {
        GUI_TAB_WIDTH
    } else {
        UnicodeWidthChar::width(character).unwrap_or(0)
    }
}

pub(in crate::gui::app::state) fn gui_editor_char_column_from_pixel_x(
    text: &str,
    x: f32,
    character_width: f32,
) -> usize {
    let x = x.max(0.0);
    let character_width = character_width.max(1.0);
    let mut display_width = 0usize;

    for (column, character) in text.chars().enumerate() {
        let char_width = gui_editor_char_display_width(character).max(1);
        let start = display_width as f32 * character_width;
        let end = display_width.saturating_add(char_width) as f32 * character_width;
        let midpoint = start + (end - start) / 2.0;
        if x < midpoint {
            return column;
        }
        if x < end {
            return column + 1;
        }
        display_width = display_width.saturating_add(char_width);
    }

    text.chars().count()
}

fn gui_editor_grapheme_wrap_units(text: &str) -> Vec<GuiEditorGraphemeWrapUnit> {
    text.grapheme_indices(true)
        .map(|(byte_index, grapheme)| {
            let start_column = text[..byte_index].chars().count();
            let end_column = start_column + grapheme.chars().count();
            GuiEditorGraphemeWrapUnit {
                start_column,
                end_column,
                display_width: gui_editor_grapheme_display_width(grapheme),
                is_whitespace: grapheme.chars().all(char::is_whitespace),
            }
        })
        .collect()
}

fn gui_editor_grapheme_display_width(grapheme: &str) -> usize {
    grapheme.chars().map(gui_editor_char_display_width).sum()
}
