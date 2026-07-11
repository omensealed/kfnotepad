fn handle_workspace_left_click(
    workspace: &mut EditorWorkspace<'_>,
    runtime: &mut EditorRuntime,
    event: MouseEvent,
    context: MouseContext,
    frame: RenderFrame,
) -> InputResult {
    if let Some(menu) = runtime.menu {
        if let Some(command) = menu_command_at_mouse(event.column, event.row, menu, frame) {
            runtime.menu = None;
            return if run_workspace_menu_command(command, workspace, runtime) {
                InputResult::Quit
            } else {
                InputResult::Handled
            };
        }
    }

    if event.row == 0 {
        if let Some(group) = menu_group_at_mouse(event.column, frame) {
            runtime.menu = Some(MenuState { group, selected: 0 });
            runtime.status = format!("Menu: {}", group.label());
            return InputResult::Handled;
        }
        return InputResult::Ignored;
    }

    let tab_items = workspace.tab_strip_items();
    if let Some(index) = tab_index_at_mouse(event.column, event.row, &tab_items, frame) {
        workspace.active = index;
        runtime.quit_confirmation_pending = false;
        runtime.close_tab_confirmation_pending = false;
        runtime.menu = None;
        stop_reader_mode(runtime, "Reader mode stopped for tab switch");
        runtime.status = active_tab_status(workspace);
        autosave_tui_current_workspace(workspace, runtime);
        return InputResult::Handled;
    }

    let active_tab = workspace.active_tab_mut();
    if let Some(clicked) = cursor_at_mouse(
        active_tab.document.as_ref(),
        event.column,
        event.row,
        runtime,
        context,
    ) {
        runtime.quit_confirmation_pending = false;
        runtime.menu = None;
        active_tab.state.cursor = clicked;
        return InputResult::Handled;
    }

    InputResult::Ignored
}
