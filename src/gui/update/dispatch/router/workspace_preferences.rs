//! Workspace panel, preferences, and path-prompt messages.

use super::*;

pub(super) fn dispatch_workspace_and_preferences(
    state: &mut KfnotepadGui,
    message: Message,
) -> GuiDispatchResult {
    match message {
        Message::SelectLeftPanelMode(mode) => {
            state.select_left_panel_mode(mode);
            handled_none()
        }
        Message::WorkspaceProjectClicked(index) => {
            handle_workspace_project_clicked(state, index);
            handled_none()
        }
        Message::WorkspaceProjectNewWindowClicked(index) => {
            handle_workspace_project_new_window_clicked(state, index);
            handled_none()
        }
        Message::WorkspaceProjectDeleteClicked(index) => {
            handle_workspace_project_delete_clicked(state, index);
            handled_none()
        }
        Message::SaveCurrentWorkspaceProject => {
            state.save_current_workspace_project();
            handled_none()
        }
        Message::WorkspaceProjectNameChanged(name) => {
            handle_workspace_project_name_changed(state, name);
            handled_none()
        }
        Message::SaveNamedWorkspaceProject => {
            state.save_named_workspace_project();
            handled_none()
        }
        Message::RestoreLastWorkspaceChanged(enabled) => {
            handle_restore_last_workspace_changed(state, enabled);
            handled_none()
        }
        Message::ShowLineNumbersChanged(enabled) => {
            state.set_show_line_numbers(enabled);
            handled_none()
        }
        Message::WrapLinesChanged(enabled) => {
            state.set_wrap_lines(enabled);
            handled_none()
        }
        Message::SearchCaseSensitiveChanged(enabled) => {
            state.set_search_case_sensitive(enabled);
            handled_none()
        }
        Message::ReaderModeChanged(enabled) => {
            state.set_reader_mode_enabled(enabled);
            handled_none()
        }
        Message::ReaderSpeedChanged(lines_per_minute) => {
            state.set_reader_speed(lines_per_minute);
            handled_none()
        }
        Message::CycleGuiFontFamily => {
            state.cycle_gui_font_family();
            handled_none()
        }
        Message::GuiFontSizeChanged(size) => {
            state.set_gui_font_size(size);
            handled_none()
        }
        Message::GuiUiFontSizeChanged(size) => {
            state.set_gui_ui_font_size(size);
            handled_none()
        }
        Message::RefreshWorkspaceProjects => {
            state.refresh_workspace_projects();
            handled_none()
        }
        Message::PathPromptChanged(path) => {
            handle_path_prompt_changed(state, path);
            handled_none()
        }
        Message::SubmitPathPrompt => GuiDispatchResult::Handled(state.submit_path_prompt()),
        Message::CancelPathPrompt => {
            state.cancel_path_prompt();
            handled_none()
        }
        Message::DismissStartupHelp => {
            handle_dismiss_startup_help(state);
            handled_none()
        }
        other => GuiDispatchResult::Unhandled(other),
    }
}
