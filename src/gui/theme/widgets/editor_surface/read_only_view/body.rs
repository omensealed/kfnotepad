use super::*;

pub(super) fn gui_editor_read_only_body(
    editor_rows: iced::widget::Column<'static, Message>,
    context: GuiReadOnlyBodyContext,
) -> Element<'static, Message> {
    mouse_area(
        container(editor_rows)
            .width(Length::Fill)
            .height(Length::Fill)
            .clip(true)
            .style(move |_theme| container::Style {
                text_color: Some(context.palette.text),
                background: Some(context.palette.background.into()),
                ..container::Style::default()
            }),
    )
    .on_move(move |point| {
        let pointer = gui_editor_replacement_mouse_point_from_body_point(
            point,
            &context.source_lines,
            context.first_line,
            context.wrapping,
            GuiEditorBodyHitTest {
                columns: context.body_columns,
                visible_rows: context.visible_row_budget,
                text_origin_x: context.gutter_width,
            },
            context.settings,
        );
        let edge = gui_editor_drag_edge_from_body_point(
            context.pane,
            point,
            context.surface_height,
            pointer.column,
            context.settings,
        );
        Message::ReplacementEditorBodyPointerMoved(context.pane, pointer, edge)
    })
    .on_press(Message::ReplacementEditorPointerPressed(context.pane))
    .on_release(Message::ReplacementEditorPointerReleased(context.pane))
    .on_scroll(move |delta| {
        Message::ReplacementEditorWheelScrolled(
            context.pane,
            gui_editor_replacement_scroll_delta_lines(delta, context.settings),
        )
    })
    .into()
}
