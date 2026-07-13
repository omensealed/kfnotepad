use super::*;

pub(in crate::gui::app::state) fn gui_search_controls<'a>(
    state: &'a KfnotepadGui,
    viewport_width: f32,
) -> Element<'a, Message> {
    match gui_search_layout_mode(viewport_width) {
        GuiSearchLayoutMode::SingleRow => row![
            gui_find_controls(state, 220.0),
            gui_navigation_controls(state, 90.0),
        ]
        .spacing(8)
        .align_y(Alignment::Center)
        .into(),
        GuiSearchLayoutMode::SplitRows => iced::widget::column![
            gui_find_controls(state, 190.0),
            gui_navigation_controls(state, 82.0),
        ]
        .spacing(6)
        .into(),
    }
}
