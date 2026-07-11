fn gui_editor_read_only_line_row(
    context: GuiReadOnlyLineRowContext,
    visual_row: GuiEditorReadOnlyVisualRow,
    rendered_row: usize,
    ime_preedit: Option<&GuiImePreedit>,
) -> (Element<'static, Message>, Option<GuiImeInputMethodRequest>) {
    let viewport_row = visual_row.viewport_row;
    let source_column_start = visual_row.source_column_start;
    let ime_request = visual_row
        .line
        .cursor_column
        .map(|cursor_column| GuiImeInputMethodRequest {
            visual_row: rendered_row,
            cursor_column,
            gutter_width: context.gutter_width,
            character_width: context.character_width,
            row_height: context.row_height,
            preedit: ime_preedit.map(|preedit| input_method::Preedit {
                content: preedit.content.clone(),
                selection: preedit.selection.clone(),
                text_size: Some(Pixels(context.editor_size as f32)),
            }),
        });
    let line_for_render =
        gui_editor_viewport_line_with_ime_preedit(visual_row.line, ime_preedit);
    let visual_row_text = line_for_render.text.clone();
    let line_spans = gui_editor_read_only_line_spans(
        &line_for_render,
        context.palette,
        context.search_highlight_active,
    );
    let line_text = rich_text(line_spans)
        .font(context.editor_font)
        .size(context.editor_size)
        .line_height(GUI_EDITOR_LINE_HEIGHT)
        .wrapping(Wrapping::None)
        .width(Length::Fill)
        .color(context.palette.text);
    let line_body = mouse_area(
        container(line_text)
            .width(Length::Fill)
            .height(Length::Fixed(context.row_height))
            .clip(true)
            .style(move |_theme| container::Style {
                text_color: Some(context.palette.text),
                background: Some(context.palette.background.into()),
                ..container::Style::default()
            }),
    )
    .on_move(move |point| {
        Message::ReplacementEditorPointerMoved(
            context.pane,
            gui_editor_replacement_mouse_point_from_visual_row_point(
                point,
                viewport_row,
                source_column_start,
                &visual_row_text,
                context.settings,
            ),
        )
    })
    .on_press(Message::ReplacementEditorPointerPressed(context.pane))
    .on_release(Message::ReplacementEditorPointerReleased(context.pane));

    let mut line_row = iced::widget::Row::new()
        .spacing(0)
        .width(Length::Fill)
        .height(Length::Fixed(context.row_height));
    if let Some(line_number_width) = context.line_number_width {
        let line_number_label = if visual_row.show_line_number {
            line_for_render.number.to_string()
        } else {
            String::new()
        };
        let line_number_text = text(line_number_label)
            .font(context.editor_font)
            .size(context.editor_size)
            .line_height(GUI_EDITOR_LINE_HEIGHT)
            .width(Length::Fill)
            .align_x(iced::alignment::Horizontal::Right)
            .color(context.palette.primary);
        let gutter_separator = container(text(""))
            .width(Length::Fixed(GUI_LINE_NUMBER_SEPARATOR_WIDTH))
            .height(Length::Fixed(context.row_height))
            .style(move |_theme| container::Style {
                background: Some(
                    Color {
                        a: 0.55,
                        ..context.palette.primary
                    }
                    .into(),
                ),
                ..container::Style::default()
            });
        line_row = line_row
            .push(
                container(line_number_text)
                    .width(Length::Fixed(line_number_width))
                    .height(Length::Fixed(context.row_height))
                    .padding([0, 2])
                    .style(move |_theme| container::Style {
                        text_color: Some(context.palette.primary),
                        background: Some(context.palette.background.into()),
                        ..container::Style::default()
                    }),
            )
            .push(gutter_separator);
    }

    (line_row.push(line_body).into(), ime_request)
}
