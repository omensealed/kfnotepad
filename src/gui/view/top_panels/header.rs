//! Responsive application menu and primary action header.

use super::super::*;

pub(in crate::gui::app::state::view) fn gui_view_header(
    state: &KfnotepadGui,
    viewport_size: Size,
) -> Element<'_, Message> {
    let menu_bar = gui_menu_bar(state.settings);

    match gui_header_layout_mode(viewport_size.width) {
        GuiHeaderLayoutMode::SingleRow => row![menu_bar, gui_header_action_row(state),]
            .spacing(GUI_HEADER_GROUP_SPACING)
            .align_y(Alignment::Center)
            .into(),
        GuiHeaderLayoutMode::SplitActions => {
            widget::column![menu_bar, gui_header_action_row(state),]
                .spacing(GUI_HEADER_SPLIT_SPACING)
                .into()
        }
    }
}
