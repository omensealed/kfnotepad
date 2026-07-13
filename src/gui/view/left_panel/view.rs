//! Active left-panel selection, scrolling, dimensions, and padding.

use super::super::*;
use super::{gui_files_panel, gui_left_panel_tabs, gui_preferences_panel, gui_workspaces_panel};

pub(in crate::gui::app::state::view) fn gui_left_panel_view(
    state: &KfnotepadGui,
) -> Option<Element<'_, Message>> {
    if !state.left_panel.visible {
        return None;
    }

    let palette = gui_theme_palette(state.settings.theme_id);
    let panel_tabs = gui_left_panel_tabs(state);
    let panel_content: Element<'_, Message> = match state.left_panel.mode {
        GuiLeftPanelMode::Files => gui_files_panel(state, panel_tabs),
        GuiLeftPanelMode::Workspaces => gui_workspaces_panel(state, panel_tabs),
        GuiLeftPanelMode::Preferences => gui_preferences_panel(state, panel_tabs, palette),
    };

    Some(
        container(scrollable(panel_content))
            .width(Length::Fixed(gui_left_panel_width(
                state.left_panel.visible,
                state.browser_width,
            )))
            .height(Length::Fill)
            .padding(iced::Padding {
                top: GUI_PANEL_PADDING_VERTICAL,
                right: GUI_PANEL_PADDING_RIGHT,
                bottom: GUI_PANEL_PADDING_VERTICAL,
                left: GUI_PANEL_PADDING_LEFT,
            })
            .into(),
    )
}
