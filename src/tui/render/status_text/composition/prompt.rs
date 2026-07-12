//! Prompt-aware status-line composition and cursor placement.

use super::*;

pub(crate) fn compose_prompt_status_line(
    prompt: &str,
    query: &str,
    right: &str,
    width: usize,
) -> StatusLineRender {
    let width = width.max(1);
    let right_count = text_display_width(right);
    let left_width = if right_count < width {
        width - right_count
    } else {
        width
    };
    let prefix = format!(" {prompt}");
    let prefix_width = text_display_width(&prefix);

    if left_width <= prefix_width {
        let text = fit_text_start(&prefix, left_width);
        let cursor_column = text_display_width(&text).saturating_sub(1);
        return StatusLineRender {
            text,
            cursor_column: Some(cursor_column as u16),
        };
    }

    let query_width = left_width.saturating_sub(prefix_width + 1).max(1);
    let visible_query = fit_text_end(query, query_width);
    let left = format!("{prefix}{visible_query} ");
    let cursor_column = text_display_width(&left).saturating_sub(1);
    let text = if right_count < width {
        let padding = width.saturating_sub(text_display_width(&left) + right_count);
        format!("{left}{}{right}", " ".repeat(padding))
    } else {
        fit_text_start(&left, width)
    };

    StatusLineRender {
        text,
        cursor_column: Some(cursor_column.min(width.saturating_sub(1)) as u16),
    }
}
