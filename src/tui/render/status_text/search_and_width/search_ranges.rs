pub(crate) fn search_match_ranges(
    text: &str,
    query: &str,
    mode: SearchMode,
) -> Vec<std::ops::Range<usize>> {
    if query.is_empty() {
        return Vec::new();
    }
    if !mode.case_sensitive {
        return case_insensitive_match_ranges(text, query);
    }

    let query_columns = query.chars().count().max(1);
    let mut ranges = Vec::new();
    let mut search_byte = 0usize;
    while search_byte <= text.len() {
        let Some(relative_match) = text[search_byte..].find(query) else {
            break;
        };
        let match_byte = search_byte + relative_match;
        let start_column = text[..match_byte].chars().count();
        ranges.push(expand_range_to_grapheme_boundaries(
            text,
            start_column..start_column + query_columns,
        ));
        search_byte = match_byte + query.len().max(1);
    }
    ranges
}
