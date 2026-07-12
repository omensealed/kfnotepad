//! Replacement-editor input, pointer, IME, and scrollbar messages.

use super::*;

pub(super) fn dispatch_replacement_editor(
    state: &mut KfnotepadGui,
    message: Message,
) -> GuiDispatchResult {
    match message {
        Message::ReplacementEditorWheelScrolled(pane, delta) => {
            state.scroll_replacement_editor_pane_viewport(pane, delta);
            handled_none()
        }
        Message::ReplacementEditorInputs(inputs) => {
            GuiDispatchResult::Handled(handle_replacement_editor_inputs(state, inputs))
        }
        Message::ReplacementEditorIme(event) => {
            handle_replacement_editor_ime(state, event);
            handled_none()
        }
        Message::ToggleReplacementOverwriteMode => {
            handle_toggle_replacement_overwrite_mode(state);
            handled_none()
        }
        Message::ReplacementEditorPointerMoved(pane, point) => {
            state.replacement_editor_pointer_moved(pane, point);
            handled_none()
        }
        Message::ReplacementEditorBodyPointerMoved(pane, point, edge) => {
            state.replacement_editor_body_pointer_moved(pane, point, edge);
            handled_none()
        }
        Message::ReplacementEditorPointerPressed(pane) => {
            state.replacement_editor_pointer_pressed(pane);
            handled_none()
        }
        Message::ReplacementEditorPointerReleased(pane) => {
            state.replacement_editor_pointer_released(pane);
            handled_none()
        }
        Message::ReplacementEditorDragTick => {
            state.replacement_editor_drag_tick();
            handled_none()
        }
        Message::ReplacementEditorGlobalPointerReleased => {
            state.replacement_editor_global_pointer_released();
            handled_none()
        }
        Message::ReplacementEditorScrollbarMoved(pane, y, model) => {
            state.replacement_editor_scrollbar_moved(pane, y, model);
            handled_none()
        }
        Message::ReplacementEditorScrollbarPressed(pane) => {
            state.replacement_editor_scrollbar_pressed(pane);
            handled_none()
        }
        Message::ReplacementEditorScrollbarReleased(pane) => {
            state.replacement_editor_scrollbar_released(pane);
            handled_none()
        }
        other => GuiDispatchResult::Unhandled(other),
    }
}
