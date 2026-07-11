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
    expected_text: String,
    result: Result<(), String>,
) {
    state.apply_save_active_tile_completion(tile_id, expected_text, result);
}

fn handle_save_active_tile_as_completed(
    state: &mut KfnotepadGui,
    tile_id: GuiTileId,
    original_path: PathBuf,
    requested_path: PathBuf,
    expected_text: String,
    clear_snapshot: bool,
    result: Result<(), String>,
) {
    state.apply_save_active_tile_as_completion(
        tile_id,
        original_path,
        requested_path,
        expected_text,
        clear_snapshot,
        result,
    );
}

fn handle_external_file_check_completed(
    state: &mut KfnotepadGui,
    results: Vec<GuiExternalFileCheckResult>,
) {
    state.complete_external_file_check(results);
}
