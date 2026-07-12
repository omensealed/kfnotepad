//! Preferences and startup-help transitions routed from GUI messages.

use super::*;

pub(super) fn handle_restore_last_workspace_changed(state: &mut KfnotepadGui, enabled: bool) {
    state.set_restore_last_workspace(enabled);
    state.persist_last_workspace_if_enabled();
}

pub(super) fn handle_path_prompt_changed(state: &mut KfnotepadGui, path: String) {
    state.path_prompt_value = path;
}

pub(super) fn handle_dismiss_startup_help(state: &mut KfnotepadGui) {
    state.show_startup_help_panel = false;
}
