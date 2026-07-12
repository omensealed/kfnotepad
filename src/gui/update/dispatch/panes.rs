//! Pane focus, sizing, layout, and lifecycle transitions.

use super::*;

pub(super) fn handle_pane_clicked(state: &mut KfnotepadGui, pane: pane_grid::Pane) {
    state.focus_pane(pane);
}

pub(super) fn handle_pane_resized(state: &mut KfnotepadGui, event: pane_grid::ResizeEvent) {
    state.panes.resize(event.split, event.ratio);
    state.persist_layout();
    state.persist_last_workspace_if_enabled();
}

pub(super) fn handle_new_tile_requested(state: &mut KfnotepadGui) {
    state.create_new_tile();
    state.persist_last_workspace_if_enabled();
}

pub(super) fn handle_close_pane(state: &mut KfnotepadGui, pane: pane_grid::Pane) {
    state.close_pane(pane);
    state.persist_last_workspace_if_enabled();
}

pub(super) fn handle_close_active_pane(state: &mut KfnotepadGui) {
    state.close_active_pane();
    state.persist_last_workspace_if_enabled();
}

pub(super) fn handle_toggle_minimize_pane(state: &mut KfnotepadGui, pane: pane_grid::Pane) {
    state.toggle_pane_minimized(pane);
    state.persist_last_workspace_if_enabled();
}

pub(super) fn handle_restore_minimized_tile(state: &mut KfnotepadGui, tile_id: GuiTileId) {
    state.restore_minimized_tile(tile_id);
    state.persist_last_workspace_if_enabled();
}

pub(super) fn handle_toggle_active_minimize(state: &mut KfnotepadGui) {
    state.toggle_active_minimize();
    state.persist_last_workspace_if_enabled();
}

pub(super) fn handle_toggle_active_maximize(state: &mut KfnotepadGui) {
    state.toggle_active_maximize();
    state.persist_last_workspace_if_enabled();
}

pub(super) fn handle_toggle_maximize_pane(state: &mut KfnotepadGui, pane: pane_grid::Pane) {
    state.toggle_pane_maximized(pane);
    state.persist_last_workspace_if_enabled();
}

pub(super) fn handle_move_active_pane(state: &mut KfnotepadGui, direction: pane_grid::Direction) {
    state.move_active_pane(direction);
    state.persist_last_workspace_if_enabled();
}

pub(super) fn handle_move_pane(
    state: &mut KfnotepadGui,
    pane: pane_grid::Pane,
    direction: pane_grid::Direction,
) {
    state.move_pane(pane, direction);
    state.persist_last_workspace_if_enabled();
}
