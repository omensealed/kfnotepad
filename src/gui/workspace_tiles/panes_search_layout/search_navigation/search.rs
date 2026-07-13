//! Forward/backward active-document search and match selection.

use super::*;

impl KfnotepadGui {
    pub(in crate::gui::app::state) fn search_active(&mut self, backwards: bool) {
        self.sync_active_editor_to_document();
        let query = self.search_query.trim().to_string();
        self.remember_search_query(&query);
        self.search_history_open = false;
        let Some(tile_id) = self
            .panes
            .get(self.active_pane)
            .map(|pane_state| pane_state.tile_id)
        else {
            self.status_message = "search failed: no active pane".to_string();
            return;
        };
        let Some(tile) = self.workspace.tile_mut(tile_id) else {
            self.status_message = "search failed: no active tile".to_string();
            return;
        };

        let result = gui_repeat_search(
            &tile.document,
            &mut tile.state.cursor,
            &query,
            backwards,
            self.settings.search_case_sensitive,
        );
        let matched_chars = match &result {
            SearchRepeatResult::Found { query } => gui_search_match_columns(
                &tile.document,
                tile.state.cursor,
                query,
                self.settings.search_case_sensitive,
            )
            .unwrap_or_else(|| query.chars().count()),
            SearchRepeatResult::NoPreviousSearch | SearchRepeatResult::NoMatch { .. } => 0,
        };
        self.search_highlight = match &result {
            SearchRepeatResult::Found { query } => Some(GuiSearchHighlight {
                tile_id,
                query: query.clone(),
            }),
            SearchRepeatResult::NoPreviousSearch | SearchRepeatResult::NoMatch { .. } => None,
        };

        self.status_message = search_result_status(result, backwards);
        self.move_active_editor_to_document_cursor();
        if matched_chars > 0 {
            if let Some(pane_state) = self.panes.get_mut(self.active_pane) {
                pane_state.editor.select_right_chars(matched_chars);
            }
        }
    }
}
