pub(crate) fn toggle_overwrite_mode(runtime: &mut EditorRuntime) {
    runtime.quit_confirmation_pending = false;
    runtime.overwrite_mode = !runtime.overwrite_mode;
    runtime.status = if runtime.overwrite_mode {
        String::from("Overwrite on")
    } else {
        String::from("Insert mode")
    };
}
