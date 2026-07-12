//! Native and replacement-editor state transitions.

use super::*;

pub(super) fn handle_editor_edit(
    state: &mut KfnotepadGui,
    pane: pane_grid::Pane,
    action: text_editor::Action,
) -> Task<Message> {
    if !state.focus_pane(pane) {
        return Task::none();
    }
    let is_scroll = matches!(action, text_editor::Action::Scroll { .. });
    let is_edit = action.is_edit();
    let Some(tile_id) = state.panes.get(pane).map(|pane_state| pane_state.tile_id) else {
        return Task::none();
    };
    if state.is_external_edit_locked(tile_id) && !is_scroll {
        state.status_message = "external edit lock active; unlock to edit".to_string();
        return Task::none();
    }
    state.search_highlight = None;
    if GUI_USE_READ_ONLY_EDITOR_RENDERER {
        if let text_editor::Action::Edit(ref edit) = action {
            if let Some(inputs) = replacement_inputs_from_edit(edit) {
                state.sync_pane_cursor_to_document(pane);
                state.apply_replacement_editor_inputs_to_active_tile(inputs);
                return Task::none();
            }
        }

        if is_edit {
            state.status_message = "edit ignored by read-only renderer".to_string();
            return Task::none();
        }
    }

    if let Some(pane_state) = state.panes.get_mut(pane) {
        pane_state
            .editor
            .apply(GuiEditorCommand::IcedAction(action));
    }
    if is_edit {
        state.sync_pane_to_document(pane);
        state.workspace.clear_tile_save_error(tile_id);
        state.external_edit_locks.remove(&tile_id);
        state.invalidate_syntax_cache(tile_id);
        state.ensure_visible_syntax_cache_for_tile(tile_id);
        state.pending_close_tile = None;
        state.pending_app_quit = false;
        state.pending_project_open = None;
        state.status_message = "modified".to_string();
    } else {
        state.sync_pane_cursor_to_document(pane);
        if is_scroll {
            state.ensure_visible_syntax_cache_for_tile(tile_id);
            state.status_message = "scrolled".to_string();
        }
    }

    Task::none()
}

pub(super) fn handle_replacement_editor_inputs(
    state: &mut KfnotepadGui,
    inputs: Vec<GuiEditorReplacementInput>,
) -> Task<Message> {
    if GUI_USE_READ_ONLY_EDITOR_RENDERER {
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
    } else {
        state.status_message = "replacement editor inactive".to_string();
    }

    Task::none()
}

pub(super) fn handle_replacement_editor_ime(state: &mut KfnotepadGui, event: input_method::Event) {
    if GUI_USE_READ_ONLY_EDITOR_RENDERER {
        state.apply_replacement_editor_ime_event(event);
    } else {
        state.status_message = "replacement editor inactive".to_string();
    }
}

pub(super) fn handle_toggle_replacement_overwrite_mode(state: &mut KfnotepadGui) {
    state.replacement_overwrite_mode = !state.replacement_overwrite_mode;
    state.status_message = if state.replacement_overwrite_mode {
        "overwrite mode".to_string()
    } else {
        "insert mode".to_string()
    };
}

fn replacement_inputs_from_edit(
    edit: &text_editor::Edit,
) -> Option<Vec<GuiEditorReplacementInput>> {
    match edit {
        text_editor::Edit::Insert(value) => {
            Some(vec![GuiEditorReplacementInput::InsertChar(*value)])
        }
        text_editor::Edit::Enter => Some(vec![GuiEditorReplacementInput::InsertNewline]),
        text_editor::Edit::Backspace => Some(vec![GuiEditorReplacementInput::DeleteBackward]),
        text_editor::Edit::Delete => Some(vec![GuiEditorReplacementInput::DeleteForward]),
        text_editor::Edit::Paste(text) => {
            Some(gui_editor_replacement_inputs_from_text(text.as_str()))
        }
        _ => None,
    }
}
