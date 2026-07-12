pub(crate) fn text_display_width(text: &str) -> usize {
    let mut display_column = 0;
    for character in text.chars() {
        display_column += character_display_width(character, display_column);
    }
    display_column
}

pub(crate) fn line_segment_display_width(text: &str, start_column: usize) -> usize {
    let mut display_column = start_column;
    for character in text.chars() {
        display_column += character_display_width(character, display_column);
    }
    display_column - start_column
}

pub(crate) fn line_display_width_until(line: &str, character_column: usize) -> usize {
    let mut display_column = 0;
    for (start_column, end_column, grapheme) in grapheme_column_units(line) {
        if start_column >= character_column {
            break;
        }
        display_column += grapheme_display_width_for_display_column(grapheme, display_column);
        if character_column < end_column {
            break;
        }
    }
    display_column
}

pub(crate) fn char_column_for_display_column(line: &str, target_display_column: usize) -> usize {
    let mut display_column = 0;
    for (start_column, end_column, grapheme) in grapheme_column_units(line) {
        let width = grapheme_display_width_for_display_column(grapheme, display_column);
        if width > 0 && display_column + width > target_display_column {
            return if target_display_column == display_column {
                start_column
            } else {
                end_column
            };
        }
        display_column += width;
    }
    line.chars().count()
}

fn grapheme_column_units(line: &str) -> impl Iterator<Item = (usize, usize, &str)> {
    use unicode_segmentation::UnicodeSegmentation;

    line.grapheme_indices(true).map(|(byte_index, grapheme)| {
        let start_column = line[..byte_index].chars().count();
        let end_column = start_column + grapheme.chars().count();
        (start_column, end_column, grapheme)
    })
}

fn grapheme_display_width_for_display_column(grapheme: &str, display_column: usize) -> usize {
    let mut current = display_column;
    for character in grapheme.chars() {
        current += character_display_width(character, current);
    }
    current - display_column
}

pub(crate) fn character_display_width(character: char, display_column: usize) -> usize {
    if character == '\t' {
        let remainder = display_column % TAB_WIDTH;
        if remainder == 0 {
            TAB_WIDTH
        } else {
            TAB_WIDTH - remainder
        }
    } else {
        UnicodeWidthChar::width(character).unwrap_or(0)
    }
}

pub(super) fn cursor_cell_character(character: char) -> char {
    if character == '\t' || character_display_width(character, 0) == 0 {
        ' '
    } else {
        character
    }
}
