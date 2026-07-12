//! Browser, file-dialog, save, managed-note, and external-file messages.

use super::*;

pub(super) fn dispatch_browser_and_files(
    state: &mut KfnotepadGui,
    message: Message,
) -> GuiDispatchResult {
    match message {
        Message::BrowserLocalTreeToggle(path) => {
            GuiDispatchResult::Handled(state.toggle_local_browser_tree_path(path))
        }
        Message::BrowserLocalTreeSelected(path, is_dir) => {
            GuiDispatchResult::Handled(state.select_local_browser_tree_path(path, is_dir))
        }
        Message::BrowserLocalTreeActivated(path, is_dir) => {
            GuiDispatchResult::Handled(state.activate_local_browser_tree_path(path, is_dir))
        }
        Message::BrowserTreeRowsLoaded { generation, result } => {
            state.apply_cached_file_tree_rows(generation, result);
            handled_none()
        }
        Message::BrowserLoaded { generation, result } => {
            state.apply_browser_load(generation, result);
            handled_none()
        }
        Message::BrowserParentRequested => {
            GuiDispatchResult::Handled(state.navigate_browser_parent())
        }
        Message::BrowserRefreshRequested => {
            GuiDispatchResult::Handled(state.refresh_file_browser())
        }
        Message::BrowserCreateFileRequested => {
            state.show_path_prompt(GuiPathPrompt::BrowserCreateFile);
            handled_none()
        }
        Message::BrowserCreateDirectoryRequested => {
            state.show_path_prompt(GuiPathPrompt::BrowserCreateDirectory);
            handled_none()
        }
        Message::BrowserDeleteSelectedRequested => {
            GuiDispatchResult::Handled(state.delete_selected_browser_entry())
        }
        Message::BrowserWidthChanged(width) => {
            handle_browser_width_changed(state, width);
            handled_none()
        }
        Message::ToggleBrowser => {
            state.toggle_left_panel();
            handled_none()
        }
        Message::OpenPromptRequested => GuiDispatchResult::Handled(state.request_open_dialog()),
        #[cfg(test)]
        Message::OpenDialogSelected(path) => {
            GuiDispatchResult::Handled(handle_open_dialog_selected(state, path))
        }
        Message::OpenDialogSelectedAsync(path) => {
            GuiDispatchResult::Handled(state.handle_open_dialog_selected_async(path))
        }
        Message::OpenDialogCompleted { path, result } => {
            handle_open_dialog_completed(state, path, result.map(|document| *document));
            handled_none()
        }
        Message::SaveActiveTileCompleted { tile_id, result } => {
            GuiDispatchResult::Handled(handle_save_active_tile_completed(state, tile_id, result))
        }
        Message::SaveActiveTileAsCompleted {
            tile_id,
            requested_path,
            result,
        } => GuiDispatchResult::Handled(handle_save_active_tile_as_completed(
            state,
            tile_id,
            requested_path,
            result,
        )),
        Message::SaveAsPromptRequested => {
            GuiDispatchResult::Handled(state.request_save_as_dialog())
        }
        Message::SaveAsDialogSelected(path) => {
            GuiDispatchResult::Handled(state.handle_save_as_dialog_selected(path))
        }
        Message::ManagedNoteClicked(index) => {
            state.open_managed_note_from_panel(index);
            handled_none()
        }
        Message::ManagedNoteDeleteClicked(index) => {
            state.delete_managed_note_from_panel(index);
            handled_none()
        }
        Message::ExternalFileCheckTick => {
            GuiDispatchResult::Handled(state.request_external_file_check())
        }
        Message::ExternalFileCheckCompleted(results) => {
            handle_external_file_check_completed(state, results);
            handled_none()
        }
        Message::UnlockExternalEdit(tile_id) => {
            state.unlock_external_edit(tile_id);
            handled_none()
        }
        Message::SaveRequested => {
            GuiDispatchResult::Handled(state.request_save_active_tile_async())
        }
        other => GuiDispatchResult::Unhandled(other),
    }
}
