fn run_list_managed_notes_command() -> ExitCode {
    let notes_dir = match current_managed_notes_dir() {
        Ok(notes_dir) => notes_dir,
        Err(error) => {
            eprintln!("kfnotepad: {error}");
            return ExitCode::from(2);
        }
    };

    match list_managed_notes(&notes_dir) {
        Ok(notes) => {
            for note in notes {
                println!("{}", note.file_name);
            }
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("kfnotepad: {error}");
            ExitCode::from(2)
        }
    }
}
