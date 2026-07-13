//! Live pane-grid composition, title state, and interaction routing.

use super::super::*;
use super::{gui_pane_body, gui_pane_controls};

pub(in crate::gui::app::state::view) fn gui_pane_grid_view(
    state: &KfnotepadGui,
) -> Element<'_, Message> {
    pane_grid(&state.panes, |pane, pane_state, is_maximized| {
        let Some(tile) = state.workspace.tile(pane_state.tile_id) else {
            return pane_grid::Content::new(text("Missing tile"));
        };
        let mut save_status = match tile.save_status() {
            GuiTileSaveStatus::Saved => "saved".to_string(),
            GuiTileSaveStatus::Modified => "modified".to_string(),
            GuiTileSaveStatus::SaveFailed { message } => format!("save failed: {message}"),
        };
        if state.is_external_edit_locked(pane_state.tile_id) {
            save_status = if save_status == "saved" {
                "locked".to_string()
            } else {
                format!("{save_status} | locked")
            };
        }
        let title =
            gui_tile_title_label(&tile.document.path, pane == state.active_pane, &save_status);
        let title_tooltip = tile.document.path.display().to_string();
        let tile_palette = gui_theme_palette(state.settings.theme_id);
        let active_tile = pane == state.active_pane;
        let mut title_bar = pane_grid::TitleBar::new(gui_tooltip(
            text(title).size(gui_ui_text_size(state.settings)),
            title_tooltip,
            iced::widget::tooltip::Position::Bottom,
            state.settings,
        ))
        .padding(GUI_TILE_TITLE_PADDING)
        .style(move |_theme| gui_tile_title_style(tile_palette, active_tile));
        if gui_tile_title_controls_attached(pane == state.active_pane) {
            title_bar = title_bar.controls(pane_grid::Controls::new(gui_pane_controls(
                state,
                pane,
                pane_state.tile_id,
                tile.minimized,
                is_maximized,
            )));
        }

        pane_grid::Content::new(gui_pane_body(
            state,
            pane,
            pane_state.tile_id,
            &pane_state.editor,
            tile,
            tile_palette,
        ))
        .title_bar(title_bar)
        .style(move |_theme| gui_tile_body_style(tile_palette, active_tile))
    })
    .height(Length::Fill)
    .spacing(GUI_PANE_GRID_SPACING)
    .style(move |_theme| gui_pane_grid_style(gui_theme_palette(state.settings.theme_id)))
    .on_click(Message::PaneClicked)
    .on_resize(8, Message::PaneResized)
    .on_drag(Message::PaneDragged)
    .into()
}
