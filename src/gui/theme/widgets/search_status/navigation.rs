pub(super) fn gui_navigation_controls<'a>(
    state: &'a KfnotepadGui,
    field_width: f32,
) -> Element<'a, Message> {
    let palette = gui_theme_palette(state.settings.theme_id);
    row![
        text_input("Line", &state.go_to_line_query)
            .on_input(Message::GoToLineQueryChanged)
            .on_submit(Message::GoToLineRequested)
            .size(gui_ui_text_size(state.settings))
            .style(move |_theme, status| gui_text_input_style(palette, status))
            .width(Length::Fixed(field_width)),
        gui_icon_tooltip_button(
            ICON_GO_TO_LINE,
            LABEL_GO_TO_LINE,
            Message::GoToLineRequested,
            LABEL_GO_TO_LINE,
            state.settings,
        ),
        gui_icon_tooltip_button(
            ICON_DOCUMENT_START,
            LABEL_DOCUMENT_START,
            Message::GoDocumentStart,
            LABEL_DOCUMENT_START,
            state.settings,
        ),
        gui_icon_tooltip_button(
            ICON_DOCUMENT_END,
            LABEL_DOCUMENT_END,
            Message::GoDocumentEnd,
            LABEL_DOCUMENT_END,
            state.settings,
        ),
    ]
    .spacing(8)
    .align_y(Alignment::Center)
    .into()
}
