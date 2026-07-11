pub(crate) fn toggle_restore_last_workspace(runtime: &mut EditorRuntime) {
    runtime.settings.gui_restore_last_workspace = !runtime.settings.gui_restore_last_workspace;
    runtime.status = if runtime.settings.gui_restore_last_workspace {
        String::from("Restore last workspace: on")
    } else {
        String::from("Restore last workspace: off")
    };
    persist_runtime_settings(runtime);
}
