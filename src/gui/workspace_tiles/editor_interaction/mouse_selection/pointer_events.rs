//! Pointer movement, press, release, and drag lifecycle routing.

use super::*;

impl KfnotepadGui {
    pub(in crate::gui::app::state) fn replacement_editor_pointer_moved(
        &mut self,
        pane: pane_grid::Pane,
        point: GuiEditorReplacementMousePoint,
    ) {
        self.replacement_pointer_point = Some((pane, point));
        if self.replacement_drag.is_some_and(|drag| drag.pane == pane) {
            self.apply_replacement_editor_mouse_drag_to_pane(pane, point);
        }
    }

    pub(in crate::gui::app::state) fn replacement_editor_body_pointer_moved(
        &mut self,
        pane: pane_grid::Pane,
        point: GuiEditorReplacementMousePoint,
        edge: GuiEditorDragEdge,
    ) {
        self.replacement_pointer_point = Some((pane, point));
        if self.replacement_drag.is_some_and(|drag| drag.pane == pane) {
            self.replacement_drag_edge = (edge.direction != 0).then_some(edge);
            self.apply_replacement_editor_mouse_drag_to_pane(pane, point);
        }
    }

    pub(in crate::gui::app::state) fn replacement_editor_pointer_pressed(
        &mut self,
        pane: pane_grid::Pane,
    ) {
        let Some((point_pane, point)) = self.replacement_pointer_point else {
            return;
        };
        if point_pane != pane {
            return;
        }

        let Some(anchor) = self.replacement_editor_cursor_for_point(pane, point) else {
            return;
        };
        self.replacement_drag = Some(GuiEditorDragState { pane, anchor });
        self.replacement_drag_edge = None;
        self.apply_replacement_editor_mouse_click_to_pane(pane, point);
    }

    pub(in crate::gui::app::state) fn replacement_editor_pointer_released(
        &mut self,
        pane: pane_grid::Pane,
    ) {
        if self.replacement_drag.is_some_and(|drag| drag.pane == pane) {
            self.clear_replacement_drag();
        }
    }

    pub(in crate::gui::app::state) fn replacement_editor_global_pointer_released(&mut self) {
        self.clear_replacement_drag();
        self.replacement_scrollbar_drag = None;
    }

    pub(in crate::gui::app::state) fn clear_replacement_drag(&mut self) {
        self.replacement_drag = None;
        self.replacement_drag_edge = None;
    }
}
