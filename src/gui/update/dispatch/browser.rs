fn handle_browser_width_changed(state: &mut KfnotepadGui, width: f32) {
    state.browser_width = clamp_browser_width(width);
    state.status_message = format!("file browser width: {:.0}px", state.browser_width);
    state.persist_layout();
    state.persist_last_workspace_if_enabled();
}
