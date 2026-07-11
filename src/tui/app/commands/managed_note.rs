fn run_managed_note_command(title: &str) -> ExitCode {
    let notes_dir = match current_managed_notes_dir() {
        Ok(notes_dir) => notes_dir,
        Err(error) => {
            eprintln!("kfnotepad: {error}");
            return ExitCode::from(2);
        }
    };

    match open_or_create_managed_note(&notes_dir, title) {
        Ok(mut document) if has_tui_terminal() => match run_editor(&mut document) {
            Ok(()) => ExitCode::SUCCESS,
            Err(error) => {
                eprintln!("kfnotepad: terminal error: {error}");
                ExitCode::from(1)
            }
        },
        Ok(document) => {
            let summary = summarize_text(&document.buffer.to_text());
            let file_name = document
                .path
                .file_name()
                .and_then(|name| name.to_str())
                .unwrap_or("managed-note");
            if io::stdin().is_terminal() && io::stdout().is_terminal() {
                maybe_print_tui_unavailable();
            }
            println!(
                "Opened managed note {file_name}: {} bytes, {} lines, trailing_newline={}",
                summary.bytes, summary.lines, summary.trailing_newline
            );
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("kfnotepad: {error}");
            ExitCode::from(2)
        }
    }
}
