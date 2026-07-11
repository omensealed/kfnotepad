pub(crate) fn run_menu_command(
    command: MenuCommand,
    document: &mut TextDocument,
    cursor: &mut Cursor,
    runtime: &mut EditorRuntime,
) -> bool {
    match command {
        MenuCommand::NewFile => {
            runtime.status = String::from("New file unavailable in this context");
        }
        MenuCommand::Save => save_document(document, runtime),
        MenuCommand::Quit => return request_quit(document, runtime),
        MenuCommand::ToggleSidebar => toggle_file_sidebar(runtime),
        MenuCommand::Find => start_search(runtime),
        MenuCommand::ToggleSearchCase => toggle_search_case(runtime),
        MenuCommand::Undo => undo_document(document, cursor, runtime),
        MenuCommand::Redo => redo_document(document, cursor, runtime),
        MenuCommand::DeletePreviousWord => delete_previous_word(document, cursor, runtime),
        MenuCommand::DeleteNextWord => delete_next_word(document, cursor, runtime),
        MenuCommand::DeleteToLineEnd => delete_to_line_end(document, cursor, runtime),
        MenuCommand::FindNext => repeat_search(document, cursor, runtime),
        MenuCommand::FindPrevious => repeat_search_previous(document, cursor, runtime),
        MenuCommand::GoToLine => start_goto_line(runtime),
        MenuCommand::ToggleLineNumbers => toggle_line_numbers(runtime),
        MenuCommand::CycleTheme => cycle_theme(runtime),
        MenuCommand::CycleSyntaxTheme => cycle_syntax_theme(runtime),
        MenuCommand::ToggleReaderMode => toggle_reader_mode(runtime),
        MenuCommand::DecreaseReaderSpeed => adjust_reader_speed(runtime, -10),
        MenuCommand::IncreaseReaderSpeed => adjust_reader_speed(runtime, 10),
        MenuCommand::ToggleWrap => toggle_wrap(runtime),
        MenuCommand::PageUp => page_up(document, cursor, runtime),
        MenuCommand::PageDown => page_down(document, cursor, runtime),
        MenuCommand::DocumentStart => go_to_document_start(cursor, runtime),
        MenuCommand::DocumentEnd => go_to_document_end(document, cursor, runtime),
        MenuCommand::PreviousWord => go_to_previous_word(document, cursor, runtime),
        MenuCommand::NextWord => go_to_next_word(document, cursor, runtime),
        MenuCommand::PreviousTab | MenuCommand::NextTab | MenuCommand::CloseTab => {
            runtime.status = String::from("Tab command unavailable in this context");
        }
        MenuCommand::SaveCurrentWorkspace
        | MenuCommand::SaveNamedWorkspace
        | MenuCommand::ListWorkspaces
        | MenuCommand::OpenWorkspace
        | MenuCommand::DeleteWorkspace
        | MenuCommand::OpenCurrentWorkspace
        | MenuCommand::ToggleRestoreLastWorkspace
        | MenuCommand::OpenCommandPalette
        | MenuCommand::OpenHelp => {
            runtime.status = String::from("Workspace command unavailable in this context");
        }
        MenuCommand::HelpOnly => {
            runtime.status = String::from("Help: choose a menu item or use the shown shortcut");
        }
    }
    false
}
