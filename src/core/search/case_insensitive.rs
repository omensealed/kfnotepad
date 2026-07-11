use std::ops::Range;

pub fn find_case_insensitive_range(
    text: &str,
    query: &str,
    start_column: usize,
) -> Option<Range<usize>> {
    let start_byte = byte_index_for_char_column(text, start_column)?;
    let (folded_tail, column_map) = folded_with_column_map(text.get(start_byte..)?);
    let folded_query = folded_text(query);
    let match_byte = folded_tail.find(&folded_query)?;
    folded_match_to_original_range(&folded_tail, &column_map, match_byte, folded_query.len())
        .map(|range| {
            let base_column = text[..start_byte].chars().count();
            expand_range_to_grapheme_boundaries(
                text,
                base_column + range.start..base_column + range.end,
            )
        })
}

pub fn find_last_case_insensitive_range(
    text: &str,
    query: &str,
    end_column: usize,
) -> Option<Range<usize>> {
    let end_byte = byte_index_for_char_column(text, end_column)?;
    let (folded_prefix, column_map) = folded_with_column_map(text.get(..end_byte)?);
    let folded_query = folded_text(query);
    let match_byte = folded_prefix.rfind(&folded_query)?;
    folded_match_to_original_range(&folded_prefix, &column_map, match_byte, folded_query.len())
        .map(|range| expand_range_to_grapheme_boundaries(text, range))
}

pub fn case_insensitive_match_ranges(text: &str, query: &str) -> Vec<Range<usize>> {
    let folded_query = folded_text(query);
    if folded_query.is_empty() {
        return Vec::new();
    }

    let (folded_text, column_map) = folded_with_column_map(text);
    let mut ranges = Vec::new();
    let mut search_byte = 0usize;
    while search_byte <= folded_text.len() {
        let Some(relative_match) = folded_text[search_byte..].find(&folded_query) else {
            break;
        };
        let match_byte = search_byte + relative_match;
        if let Some(range) = folded_match_to_original_range(
            &folded_text,
            &column_map,
            match_byte,
            folded_query.len(),
        ) {
            let expanded = expand_range_to_grapheme_boundaries(text, range);
            if ranges.last() != Some(&expanded) {
                ranges.push(expanded);
            }
        }
        search_byte = match_byte + folded_query.len().max(1);
    }
    ranges
}

fn folded_with_column_map(text: &str) -> (String, Vec<usize>) {
    let mut folded = String::new();
    let mut column_map = Vec::new();
    for (column, character) in text.chars().enumerate() {
        push_folded_character(character, &mut folded, || column_map.push(column));
    }
    (folded, column_map)
}

fn folded_text(text: &str) -> String {
    let mut folded = String::new();
    for character in text.chars() {
        push_folded_character(character, &mut folded, || {});
    }
    folded
}

fn push_folded_character(
    character: char,
    folded: &mut String,
    mut on_pushed_character: impl FnMut(),
) {
    match character {
        'ß' | 'ẞ' => {
            folded.push('s');
            on_pushed_character();
            folded.push('s');
            on_pushed_character();
        }
        _ => {
            for folded_character in character.to_lowercase() {
                folded.push(folded_character);
                on_pushed_character();
            }
        }
    }
}

fn folded_match_to_original_range(
    folded_text: &str,
    column_map: &[usize],
    match_byte: usize,
    match_len: usize,
) -> Option<Range<usize>> {
    if match_len == 0 || match_byte > folded_text.len() {
        return None;
    }
    let match_end = match_byte.checked_add(match_len)?;
    if match_end > folded_text.len()
        || !folded_text.is_char_boundary(match_byte)
        || !folded_text.is_char_boundary(match_end)
    {
        return None;
    }

    let start_index = folded_text[..match_byte].chars().count();
    let end_index = folded_text[..match_end].chars().count();
    let start_column = *column_map.get(start_index)?;
    let end_column = column_map
        .get(end_index.saturating_sub(1))?
        .saturating_add(1);
    Some(start_column..end_column)
}

fn byte_index_for_char_column(text: &str, column: usize) -> Option<usize> {
    if column > text.chars().count() {
        return None;
    }
    Some(
        text.char_indices()
            .nth(column)
            .map_or(text.len(), |(index, _)| index),
    )
}
