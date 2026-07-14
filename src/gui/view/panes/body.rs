//! Minimized and editable tile body rendering.

use super::super::*;

pub(in crate::gui::app::state::view) fn gui_pane_body<'a>(
    state: &'a KfnotepadGui,
    pane: pane_grid::Pane,
    tile_id: GuiTileId,
    editor: &'a GuiEditorAdapter,
    tile: &'a GuiDocumentTile,
) -> Element<'a, Message> {
    let body = if tile.minimized {
        widget::column![text("Minimized").size(gui_ui_text_size(state.settings))]
            .padding(GUI_TILE_BODY_PADDING)
            .height(Length::Fill)
    } else {
        let editor_surface = gui_editor_surface_model(
            state.settings,
            &tile.document,
            editor,
            &state.syntax_highlighter,
            state.syntax_caches.get(&tile_id),
        );
        let search_highlight_active = state
            .search_highlight
            .as_ref()
            .is_some_and(|highlight| highlight.tile_id == tile_id);
        let ime_preedit = state
            .replacement_ime_preedit
            .as_ref()
            .filter(|preedit| preedit.tile_id == tile_id)
            .cloned();
        let editor_body: Element<'_, Message> = gui_editor_read_only_view(
            pane,
            &editor_surface,
            state.settings,
            search_highlight_active,
            ime_preedit,
        );

        widget::column![editor_body]
            .padding(GUI_TILE_BODY_PADDING)
            .height(Length::Fill)
    };

    body.into()
}
