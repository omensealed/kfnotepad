pub fn expand_range_to_grapheme_boundaries(
    text: &str,
    range: std::ops::Range<usize>,
) -> std::ops::Range<usize> {
    use unicode_segmentation::UnicodeSegmentation;

    let columns = text.chars().count();
    let mut start = range.start.min(columns);
    let mut end = range.end.min(columns);
    if start > end {
        std::mem::swap(&mut start, &mut end);
    }

    for (byte_index, grapheme) in text.grapheme_indices(true) {
        let grapheme_start = text[..byte_index].chars().count();
        let grapheme_end = grapheme_start + grapheme.chars().count();
        if grapheme_start < start && start < grapheme_end {
            start = grapheme_start;
        }
        if grapheme_start < end && end < grapheme_end {
            end = grapheme_end;
        }
    }

    start..end
}
