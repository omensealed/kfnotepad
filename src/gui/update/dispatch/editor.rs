//! Native and replacement-editor state transitions.

use super::*;

pub(super) fn handle_replacement_editor_inputs(
    state: &mut KfnotepadGui,
    inputs: Vec<GuiEditorReplacementInput>,
) -> Task<Message> {
    let Some(tile_id) = state
        .panes
        .get(state.active_pane)
        .map(|pane_state| pane_state.tile_id)
    else {
        return Task::none();
    };
    if state.is_external_edit_locked(tile_id) {
        state.status_message = "external edit lock active; unlock to edit".to_string();
        return Task::none();
    }
    state.search_highlight = None;
    state.apply_replacement_editor_inputs_to_active_tile(inputs);

    Task::none()
}

pub(super) fn handle_replacement_editor_ime(state: &mut KfnotepadGui, event: input_method::Event) {
    state.apply_replacement_editor_ime_event(event);
}

pub(super) fn handle_toggle_replacement_overwrite_mode(state: &mut KfnotepadGui) {
    state.replacement_overwrite_mode = !state.replacement_overwrite_mode;
    state.status_message = if state.replacement_overwrite_mode {
        "overwrite mode".to_string()
    } else {
        "insert mode".to_string()
    };
}
