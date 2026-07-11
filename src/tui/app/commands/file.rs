fn run_file_command(path: &str) -> ExitCode {
    if has_tui_terminal() {
        match open_text_file(Path::new(path)) {
            Ok(mut document) => match run_editor(&mut document) {
                Ok(()) => ExitCode::SUCCESS,
                Err(error) => {
                    eprintln!("kfnotepad: terminal error: {error}");
                    ExitCode::from(1)
                }
            },
            Err(error) => {
                eprintln!("kfnotepad: {error}");
                ExitCode::from(2)
            }
        }
    } else {
        if io::stdin().is_terminal() && io::stdout().is_terminal() {
            maybe_print_tui_unavailable();
        }
        match summarize_path(Path::new(path)) {
            Ok(summary) => {
                println!(
                    "Opened {path}: {} bytes, {} lines, trailing_newline={}",
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
}
