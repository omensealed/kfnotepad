//! Minimized pane ordinal list parsing.

pub(super) fn parse_layout_ordinals(value: &str) -> Option<Vec<usize>> {
    let value = value.trim();
    if value.is_empty() {
        return Some(Vec::new());
    }

    value
        .split(',')
        .map(|ordinal| ordinal.trim().parse::<usize>().ok())
        .collect()
}
