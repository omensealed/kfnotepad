//! Passive vertical viewport clamping for reader-driven scrolling.

use super::*;

pub(crate) fn clamp_passive_viewport(
    document: &TextDocument,
    viewport_start: usize,
    visible_rows: usize,
    settings: EditorSettings,
) -> usize {
    let visible_rows = visible_rows.max(1);
    let max_start = max_viewport_start(document, visible_rows, settings);
    viewport_start.min(max_start)
}

pub(super) fn max_viewport_start(
    document: &TextDocument,
    visible_rows: usize,
    settings: EditorSettings,
) -> usize {
    if settings.wrap_lines {
        document.buffer.line_count().saturating_sub(1)
    } else {
        document
            .buffer
            .line_count()
            .saturating_sub(visible_rows.max(1))
    }
}
