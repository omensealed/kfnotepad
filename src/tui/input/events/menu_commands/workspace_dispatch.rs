pub(crate) fn run_workspace_menu_command(
    command: MenuCommand,
    workspace: &mut EditorWorkspace<'_>,
    runtime: &mut EditorRuntime,
) -> bool {
    match command {
        MenuCommand::NewFile => create_new_file_tab(workspace, runtime),
        MenuCommand::PreviousTab => select_previous_tab(workspace, runtime),
        MenuCommand::NextTab => select_next_tab(workspace, runtime),
        MenuCommand::CloseTab => close_active_tab(workspace, runtime),
        MenuCommand::SaveCurrentWorkspace => save_workspace_project_named(
            workspace,
            runtime,
            TUI_CURRENT_WORKSPACE_NAME,
            "current workspace",
        ),
        MenuCommand::SaveNamedWorkspace => start_workspace_save_prompt(runtime),
        MenuCommand::ListWorkspaces => open_workspace_manager(runtime),
        MenuCommand::OpenWorkspace => start_workspace_open_prompt(runtime),
        MenuCommand::DeleteWorkspace => start_workspace_delete_prompt(runtime),
        MenuCommand::OpenCurrentWorkspace => {
            open_workspace_project_named(workspace, runtime, TUI_CURRENT_WORKSPACE_NAME)
        }
        MenuCommand::ToggleRestoreLastWorkspace => toggle_restore_last_workspace(runtime),
        MenuCommand::OpenHelp => open_tui_help_document(workspace, runtime),
        MenuCommand::OpenCommandPalette => open_command_palette(runtime),
        _ => {
            let active_tab = workspace.active_tab_mut();
            return run_menu_command(
                command,
                active_tab.document.as_mut(),
                &mut active_tab.state.cursor,
                runtime,
            );
        }
    }
    false
}
