fn run_empty_command() -> ExitCode {
    if has_tui_terminal() {
        if let Some((project_path, settings)) = current_tui_restore_project_request() {
            if let Ok(restored) = load_tui_workspace_project(&project_path).and_then(|project| {
                workspace_from_project_documents(&project, env::current_dir().unwrap_or_default())
            }) {
                let status = restored.status_message();
                return match run_editor_workspace(restored.workspace, Some(settings), status) {
                    Ok(()) => ExitCode::SUCCESS,
                    Err(error) => {
                        eprintln!("kfnotepad: terminal error: {error}");
                        ExitCode::from(1)
                    }
                };
            }
        }
    }

    if io::stdin().is_terminal() && io::stdout().is_terminal() {
        maybe_print_tui_unavailable();
        return ExitCode::SUCCESS;
    }

    println!("kfnotepad executable baseline is ready.");
    println!("Run `kfnotepad --help` for supported commands.");
    ExitCode::SUCCESS
}
