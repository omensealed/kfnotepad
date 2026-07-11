pub(super) fn gui_editor_read_only_line_spans(
    line: &GuiEditorViewportLine,
    palette: iced::theme::Palette,
    search_highlight_active: bool,
) -> Vec<iced::widget::text::Span<'static, Message, Font>> {
    gui_editor_read_only_line_segments(line)
        .into_iter()
        .map(|segment| {
            let selected = segment.selected;
            let segment_color = if selected {
                palette.background
            } else {
                segment.syntax_color.unwrap_or(palette.text)
            };
            let mut text_span = span(segment.text).color(segment_color);
            if selected {
                text_span = text_span
                    .background(gui_replacement_editor_overlay_color(
                        palette,
                        search_highlight_active,
                    ))
                    .padding(1);
            }
            text_span
        })
        .collect()
}

pub(super) fn gui_replacement_editor_overlay_color(
    palette: iced::theme::Palette,
    search_highlight_active: bool,
) -> Color {
    Color {
        a: if search_highlight_active { 0.95 } else { 0.78 },
        ..palette.primary
    }
}
