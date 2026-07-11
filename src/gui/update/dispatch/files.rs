fn handle_open_dialog_selected(
    state: &mut KfnotepadGui,
    path: Option<PathBuf>,
) -> Task<Message> {
    let task = state.handle_open_dialog_selected(path);
    state.persist_last_workspace_if_enabled();
    task
}

fn handle_open_dialog_completed(
    state: &mut KfnotepadGui,
    path: PathBuf,
    result: Result<TextDocument, String>,
) {
    state.handle_open_dialog_completed(path, result);
    state.persist_last_workspace_if_enabled();
}

fn handle_save_active_tile_completed(
    state: &mut KfnotepadGui,
    tile_id: GuiTileId,
    result: Result<GuiSaveResult, String>,
) {
    state.apply_save_active_tile_completion(tile_id, result);
}

fn handle_save_active_tile_as_completed(
    state: &mut KfnotepadGui,
    tile_id: GuiTileId,
    requested_path: PathBuf,
    result: Result<GuiSaveResult, String>,
) {
    state.apply_save_active_tile_as_completion(tile_id, requested_path, result);
}

fn handle_external_file_check_completed(
    state: &mut KfnotepadGui,
    results: Vec<GuiExternalFileCheckResult>,
) {
    state.complete_external_file_check(results);
}
