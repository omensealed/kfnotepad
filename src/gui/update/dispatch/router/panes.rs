fn dispatch_panes(state: &mut KfnotepadGui, message: Message) -> GuiDispatchResult {
    match message {
        Message::PaneClicked(pane) => {
            handle_pane_clicked(state, pane);
            handled_none()
        }
        Message::PaneResized(event) => {
            handle_pane_resized(state, event);
            handled_none()
        }
        Message::PaneDragged(event) => {
            state.drag_pane(event);
            handled_none()
        }
        Message::NewTileRequested => {
            handle_new_tile_requested(state);
            handled_none()
        }
        Message::ClosePane(pane) => {
            handle_close_pane(state, pane);
            handled_none()
        }
        Message::CloseActivePane => {
            handle_close_active_pane(state);
            handled_none()
        }
        Message::ToggleMinimizePane(pane) => {
            handle_toggle_minimize_pane(state, pane);
            handled_none()
        }
        Message::RestoreMinimizedTile(tile_id) => {
            handle_restore_minimized_tile(state, tile_id);
            handled_none()
        }
        Message::ToggleActiveMinimize => {
            handle_toggle_active_minimize(state);
            handled_none()
        }
        Message::ToggleActiveMaximize => {
            handle_toggle_active_maximize(state);
            handled_none()
        }
        Message::ToggleMaximizePane(pane) => {
            handle_toggle_maximize_pane(state, pane);
            handled_none()
        }
        Message::MoveActivePane(direction) => {
            handle_move_active_pane(state, direction);
            handled_none()
        }
        Message::MovePane(pane, direction) => {
            handle_move_pane(state, pane, direction);
            handled_none()
        }
        other => GuiDispatchResult::Unhandled(other),
    }
}
