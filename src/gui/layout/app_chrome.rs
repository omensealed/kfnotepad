pub(super) fn title(state: &KfnotepadGui) -> String {
    format!(
        "kfnotepad-gui - {}",
        state.workspace.active_tile().document.path.display()
    )
}

pub(super) fn theme(state: &KfnotepadGui) -> Theme {
    gui_theme(state.settings.theme_id)
}
