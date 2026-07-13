//! Window title and application theme derivation.

use super::*;

pub(in crate::gui::app::state) fn title(state: &KfnotepadGui) -> String {
    format!(
        "kfnotepad-gui - {}",
        state.workspace.active_tile().document.path.display()
    )
}

pub(in crate::gui::app::state) fn theme(state: &KfnotepadGui) -> Theme {
    gui_theme(state.settings.theme_id)
}
