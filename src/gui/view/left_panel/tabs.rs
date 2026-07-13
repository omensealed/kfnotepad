//! Icon tabs for selecting the active left-panel mode.

use super::super::*;

pub(in crate::gui::app::state::view) fn gui_left_panel_tabs(
    state: &KfnotepadGui,
) -> Element<'_, Message> {
    row![
        gui_icon_tooltip_button(
            ICON_FILES,
            LABEL_FILES,
            Message::SelectLeftPanelMode(GuiLeftPanelMode::Files),
            "Show Files panel",
            state.settings,
        ),
        gui_icon_tooltip_button(
            ICON_WORKSPACES,
            LABEL_WORKSPACES,
            Message::SelectLeftPanelMode(GuiLeftPanelMode::Workspaces),
            "Show Workspaces panel",
            state.settings,
        ),
        gui_icon_tooltip_button(
            ICON_PREFERENCES,
            LABEL_PREFERENCES,
            Message::SelectLeftPanelMode(GuiLeftPanelMode::Preferences),
            "Show Preferences panel",
            state.settings,
        ),
    ]
    .spacing(GUI_PANEL_CONTROL_SPACING)
    .into()
}
