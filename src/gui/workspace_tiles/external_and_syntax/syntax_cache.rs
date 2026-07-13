//! Visible-pane syntax cache invalidation and incremental extension.

use super::*;

impl KfnotepadGui {
    pub(in crate::gui::app::state) fn refresh_visible_syntax_caches(&mut self) {
        let tile_ids = self
            .panes
            .iter()
            .map(|(_pane, pane_state)| pane_state.tile_id)
            .collect::<Vec<_>>();
        for tile_id in tile_ids {
            self.ensure_visible_syntax_cache_for_tile(tile_id);
        }
    }

    pub(in crate::gui::app::state) fn invalidate_syntax_cache(&mut self, tile_id: GuiTileId) {
        self.syntax_caches.remove(&tile_id);
    }

    pub(in crate::gui::app::state) fn invalidate_all_syntax_caches(&mut self) {
        self.syntax_caches.clear();
    }

    pub(in crate::gui::app::state) fn syntax_cache_target_end_for_tile(
        &self,
        tile_id: GuiTileId,
    ) -> Option<usize> {
        let pane = pane_for_tile_id(&self.panes, tile_id)?;
        let pane_state = self.panes.get(pane)?;
        let tile = self.workspace.tile(tile_id)?;
        let line_count = tile.document.buffer.line_count().max(1);
        Some(
            pane_state
                .editor
                .viewport
                .first_line
                .saturating_sub(1)
                .saturating_add(GUI_EDITOR_RENDER_LINE_BUDGET)
                .min(line_count),
        )
    }

    pub(in crate::gui::app::state) fn ensure_visible_syntax_cache_for_tile(
        &mut self,
        tile_id: GuiTileId,
    ) {
        let Some(target_end) = self.syntax_cache_target_end_for_tile(tile_id) else {
            self.syntax_caches.remove(&tile_id);
            return;
        };
        let Some(tile) = self.workspace.tile(tile_id) else {
            self.syntax_caches.remove(&tile_id);
            return;
        };
        let path = tile.document.path.clone();
        let line_count = tile.document.buffer.line_count().max(1);

        let reset_cache = self.syntax_caches.get(&tile_id).is_none_or(|cache| {
            cache.path != path
                || cache.line_count != line_count
                || cache.highlighted_until > line_count
        });
        if reset_cache {
            self.syntax_caches.insert(
                tile_id,
                GuiSyntaxCache {
                    path: path.clone(),
                    line_count,
                    highlighted_until: 0,
                    lines: Vec::with_capacity(target_end),
                    state: None,
                },
            );
        }

        let (start_line, requested_rows, state) = {
            let Some(cache) = self.syntax_caches.get_mut(&tile_id) else {
                return;
            };
            if target_end <= cache.highlighted_until {
                return;
            }
            let start_line = cache.highlighted_until;
            let requested_rows = target_end.saturating_sub(start_line);
            let state = cache.state.take();
            (start_line, requested_rows, state)
        };
        let Some(tile) = self.workspace.tile(tile_id) else {
            self.syntax_caches.remove(&tile_id);
            return;
        };
        let (highlighted_lines, next_state) = self
            .syntax_highlighter
            .highlight_lines_incremental_for_path(
                &tile.document.path,
                tile.document.buffer.lines(),
                start_line,
                requested_rows,
                state,
            );

        let Some(cache) = self.syntax_caches.get_mut(&tile_id) else {
            return;
        };
        let theme_id = self.settings.syntax_theme_id;
        cache.lines.extend(
            highlighted_lines
                .into_iter()
                .map(|line| line.map(|segments| gui_syntax_segments(segments, theme_id))),
        );
        cache.highlighted_until = cache.lines.len().min(line_count);
        cache.state = next_state;
    }
}
