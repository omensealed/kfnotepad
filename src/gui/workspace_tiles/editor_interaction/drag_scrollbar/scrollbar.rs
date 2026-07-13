impl KfnotepadGui {
    pub(in crate::gui::app::state) fn replacement_editor_scrollbar_moved(
        &mut self,
        pane: pane_grid::Pane,
        y: f32,
        model: GuiEditorScrollbarModel,
    ) {
        self.replacement_scrollbar_pointer = Some((pane, y, model));
        let Some(drag) = self.replacement_scrollbar_drag else {
            return;
        };
        if drag.pane != pane {
            return;
        }
        self.set_replacement_editor_scrollbar_first_line(
            pane,
            gui_editor_scrollbar_first_line_from_thumb_y(model, y, drag.thumb_offset),
        );
    }

    pub(in crate::gui::app::state) fn replacement_editor_scrollbar_pressed(&mut self, pane: pane_grid::Pane) {
        let Some((pointer_pane, y, model)) = self.replacement_scrollbar_pointer else {
            return;
        };
        if pointer_pane != pane {
            return;
        }
        match gui_editor_scrollbar_press_target(model, y) {
            GuiEditorScrollbarPressTarget::None => {}
            GuiEditorScrollbarPressTarget::Page(delta) => {
                self.scroll_replacement_editor_pane_viewport(pane, delta);
            }
            GuiEditorScrollbarPressTarget::Thumb { offset } => {
                self.clear_replacement_drag();
                self.replacement_scrollbar_drag = Some(GuiEditorScrollbarDrag {
                    pane,
                    thumb_offset: offset,
                });
                self.set_replacement_editor_scrollbar_first_line(
                    pane,
                    gui_editor_scrollbar_first_line_from_thumb_y(model, y, offset),
                );
            }
        }
    }

    pub(in crate::gui::app::state) fn replacement_editor_scrollbar_released(&mut self, pane: pane_grid::Pane) {
        if self
            .replacement_scrollbar_drag
            .is_some_and(|drag| drag.pane == pane)
        {
            self.replacement_scrollbar_drag = None;
        }
    }

    pub(in crate::gui::app::state) fn set_replacement_editor_scrollbar_first_line(
        &mut self,
        pane: pane_grid::Pane,
        first_line: usize,
    ) {
        let Some(pane_state) = self.panes.get_mut(pane) else {
            return;
        };
        let line_count = pane_state.editor.line_count();
        pane_state.editor.viewport.first_line = first_line;
        pane_state.editor.viewport.clamp_to_line_count(line_count);
        pane_state.editor.viewport_tracks_cursor = false;
        self.status_message = "viewport scrolled".to_string();
    }
}
