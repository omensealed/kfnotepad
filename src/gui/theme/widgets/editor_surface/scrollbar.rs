//! Replacement-editor scrollbar view and track segments.

use super::*;

pub(in crate::gui::app::state) fn gui_editor_scrollbar_view(
    pane: pane_grid::Pane,
    model: GuiEditorScrollbarModel,
    palette: iced::theme::Palette,
    settings: EditorSettings,
) -> Element<'static, Message> {
    let top_height = model.thumb_top.max(0.0);
    let thumb_height = model.thumb_height.max(1.0);
    let bottom_height = (model.track_height - model.thumb_top - model.thumb_height).max(0.0);

    let track_above = gui_scrollbar_track_segment(top_height, palette, model.visible);
    let thumb = container(text(""))
        .width(Length::Fixed(GUI_EDITOR_SCROLLBAR_WIDTH))
        .height(Length::Fixed(thumb_height))
        .style(move |_theme| gui_scrollbar_thumb_style(palette, model.visible));
    let track_below = gui_scrollbar_track_segment(bottom_height, palette, model.visible);

    mouse_area(
        iced::widget::column![track_above, thumb, track_below]
            .spacing(0)
            .width(Length::Fixed(GUI_EDITOR_SCROLLBAR_WIDTH))
            .height(Length::Fill),
    )
    .on_move(move |point| Message::ReplacementEditorScrollbarMoved(pane, point.y, model))
    .on_press(Message::ReplacementEditorScrollbarPressed(pane))
    .on_release(Message::ReplacementEditorScrollbarReleased(pane))
    .on_scroll(move |delta| {
        Message::ReplacementEditorWheelScrolled(
            pane,
            gui_editor_replacement_scroll_delta_lines(delta, settings),
        )
    })
    .into()
}

pub(in crate::gui::app::state) fn gui_scrollbar_track_segment(
    height: f32,
    palette: iced::theme::Palette,
    enabled: bool,
) -> Element<'static, Message> {
    if !enabled || height < 1.0 {
        return container(text(""))
            .width(Length::Fixed(GUI_EDITOR_SCROLLBAR_WIDTH))
            .height(Length::Fixed(height.max(0.0)))
            .style(move |_theme| gui_scrollbar_track_style(palette, false))
            .into();
    }

    container(text(""))
        .width(Length::Fixed(GUI_EDITOR_SCROLLBAR_WIDTH))
        .height(Length::Fixed(height))
        .style(move |_theme| gui_scrollbar_track_style(palette, enabled))
        .into()
}
