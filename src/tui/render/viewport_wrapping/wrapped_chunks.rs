use unicode_segmentation::UnicodeSegmentation;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct WrappedLineChunk<'a> {
    pub(crate) start_column: usize,
    pub(crate) text: &'a str,
}

pub(crate) fn wrapped_line_chunks(line: &str, width: usize) -> Vec<WrappedLineChunk<'_>> {
    let width = width.max(1);
    if line.is_empty() {
        return vec![WrappedLineChunk {
            start_column: 0,
            text: "",
        }];
    }

    let mut chunks = Vec::new();
    let mut remaining_start = 0usize;
    let mut first_chunk = true;

    while remaining_start < line.len() {
        let mut remaining = &line[remaining_start..];
        if !first_chunk {
            let trimmed = remaining.trim_start_matches(char::is_whitespace);
            remaining = trimmed;
            remaining_start = line.len().saturating_sub(remaining.len());
            if remaining.is_empty() {
                break;
            }
        }
        let start_column = text_display_width(&line[..remaining_start]);
        let (chunk, rest) = take_wrapped_line_chunk(remaining, width);
        remaining_start = line.len().saturating_sub(rest.len());
        chunks.push(WrappedLineChunk {
            start_column,
            text: chunk,
        });
        first_chunk = false;
    }

    chunks
}

pub(crate) fn wrapped_line_chunk_count(line: &str, width: usize) -> usize {
    let width = width.max(1);
    if line.is_empty() {
        return 1;
    }

    let mut count = 0usize;
    let mut remaining_start = 0usize;
    let mut first_chunk = true;

    while remaining_start < line.len() {
        let mut remaining = &line[remaining_start..];
        if !first_chunk {
            let trimmed = remaining.trim_start_matches(char::is_whitespace);
            remaining = trimmed;
            if remaining.is_empty() {
                break;
            }
        }
        let (_, rest) = take_wrapped_line_chunk(remaining, width);
        remaining_start = line.len().saturating_sub(rest.len());
        count += 1;
        first_chunk = false;
    }

    count.max(1)
}

fn take_wrapped_line_chunk(line: &str, width: usize) -> (&str, &str) {
    let mut display_column = 0;
    let mut last_break_byte = None;

    for (byte_index, grapheme) in line.grapheme_indices(true) {
        let grapheme_width = grapheme_display_width(grapheme, display_column);
        if grapheme_width > 0 && display_column + grapheme_width > width {
            if grapheme.chars().all(char::is_whitespace) {
                let chunk = line[..byte_index].trim_end_matches(char::is_whitespace);
                return (chunk, &line[byte_index + grapheme.len()..]);
            }
            if let Some(break_byte) = last_break_byte {
                let chunk = line[..break_byte].trim_end_matches(char::is_whitespace);
                return (chunk, &line[break_byte..]);
            }
            if byte_index == 0 {
                let next_byte = byte_index + grapheme.len();
                return (&line[..next_byte], &line[next_byte..]);
            }
            return (&line[..byte_index], &line[byte_index..]);
        }

        display_column += grapheme_width;
        if grapheme.chars().all(char::is_whitespace) {
            last_break_byte = Some(byte_index + grapheme.len());
        }
    }

    (line, "")
}

pub(crate) fn grapheme_display_width(grapheme: &str, start_column: usize) -> usize {
    let mut display_column = start_column;
    for character in grapheme.chars() {
        display_column += character_display_width(character, display_column);
    }
    display_column - start_column
}
