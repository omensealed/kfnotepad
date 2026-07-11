pub(crate) fn compose_status_line(left: &str, right: &str, width: usize) -> String {
    let width = width.max(1);
    let right_count = text_display_width(right);
    if right_count >= width {
        return fit_text_start(right, width);
    }

    let left_width = width - right_count;
    let left = fit_text_start(left, left_width);
    let padding = width.saturating_sub(text_display_width(&left) + right_count);
    format!("{left}{}{right}", " ".repeat(padding))
}

pub(crate) struct StatusLineRender {
    pub(crate) text: String,
    pub(crate) cursor_column: Option<u16>,
}
