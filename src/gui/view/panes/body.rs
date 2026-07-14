//! Minimized and editable tile body rendering.

use super::super::*;

pub(in crate::gui::app::state::view) fn gui_pane_body<'a>(
    state: &'a KfnotepadGui,
    pane: pane_grid::Pane,
    tile_id: GuiTileId,
    editor: &'a GuiEditorAdapter,
    tile: &'a GuiDocumentTile,
    tile_palette: iced::theme::Palette,
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
        let editor_body: Element<'_, Message> = if GUI_USE_READ_ONLY_EDITOR_RENDERER {
            gui_editor_read_only_view(
                pane,
                &editor_surface,
                state.settings,
                search_highlight_active,
                ime_preedit,
            )
        } else {
            let editor = text_editor(editor_surface.content)
                .placeholder("Type here...")
                .font(editor_surface.editor_font)
                .size(editor_surface.editor_size)
                .line_height(GUI_EDITOR_LINE_HEIGHT)
                .wrapping(editor_surface.wrapping);
            let editor = editor
                .style(move |_theme, status| {
                    gui_native_editor_style(tile_palette, status, search_highlight_active)
                })
                .on_action(move |action| Message::Edit(pane, action))
                .height(Length::Fill);
            let _line_numbers = editor_surface.line_numbers;
            editor.into()
        };

        widget::column![editor_body]
            .padding(GUI_TILE_BODY_PADDING)
            .height(Length::Fill)
    };

    body.into()
}
