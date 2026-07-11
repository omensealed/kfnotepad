fn byte_index_for_char_column(line: &str, column: usize) -> Result<usize, BufferError> {
    let columns = line.chars().count();
    if column > columns {
        return Err(BufferError::ColumnOutOfBounds { column, columns });
    }

    Ok(line
        .char_indices()
        .nth(column)
        .map_or(line.len(), |(index, _)| index))
}

fn find_in_line_with_mode(
    line: &str,
    query: &str,
    start_column: usize,
    row: usize,
    mode: SearchMode,
) -> Option<Cursor> {
    if !mode.case_sensitive {
        return find_in_line_case_insensitive(line, query, start_column, row);
    }
    let start_byte = byte_index_for_char_column(line, start_column).ok()?;
    let match_byte = line.get(start_byte..)?.find(query)? + start_byte;
    let match_start = line[..match_byte].chars().count();
    let match_end = match_start + query.chars().count();
    let match_range = expand_range_to_grapheme_boundaries(line, match_start..match_end);
    Some(Cursor {
        row,
        column: match_range.start,
    })
}

fn find_last_in_line_before_with_mode(
    line: &str,
    query: &str,
    end_column: usize,
    row: usize,
    mode: SearchMode,
) -> Option<Cursor> {
    if !mode.case_sensitive {
        return find_last_in_line_case_insensitive(line, query, end_column, row);
    }
    let end_byte = byte_index_for_char_column(line, end_column).ok()?;
    let match_byte = line.get(..end_byte)?.rfind(query)?;
    let match_start = line[..match_byte].chars().count();
    let match_end = match_start + query.chars().count();
    let match_range = expand_range_to_grapheme_boundaries(line, match_start..match_end);
    Some(Cursor {
        row,
        column: match_range.start,
    })
}

fn find_in_line_case_insensitive(
    line: &str,
    query: &str,
    start_column: usize,
    row: usize,
) -> Option<Cursor> {
    let range = find_case_insensitive_range(line, query, start_column)?;
    Some(Cursor {
        row,
        column: range.start,
    })
}

fn find_last_in_line_case_insensitive(
    line: &str,
    query: &str,
    end_column: usize,
    row: usize,
) -> Option<Cursor> {
    let range = find_last_case_insensitive_range(line, query, end_column)?;
    Some(Cursor {
        row,
        column: range.start,
    })
}

fn is_word_character(character: char) -> bool {
    character == '_' || character.is_alphanumeric()
}
