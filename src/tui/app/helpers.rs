fn current_managed_notes_dir() -> Result<PathBuf, kfnotepad::ManagedNotesError> {
    kfnotepad::current_managed_notes_dir()
}

fn run_editor(document: &mut TextDocument) -> io::Result<()> {
    let workspace = EditorWorkspace::from_document(document);
    run_editor_workspace(workspace, None, None)
}

fn has_tui_terminal() -> bool {
    io::stdin().is_terminal() && io::stdout().is_terminal() && supports_tui_terminal()
}

fn maybe_print_tui_unavailable() {
    if supports_tui_terminal() {
        return;
    }
    match env::var("TERM").ok() {
        Some(term) if !term.is_empty() => {
            eprintln!("kfnotepad: interactive terminal mode is unavailable in TERM={term}")
        }
        _ => eprintln!("kfnotepad: interactive terminal mode is unavailable in this terminal"),
    }
}
