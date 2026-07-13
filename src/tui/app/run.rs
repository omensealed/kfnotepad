//! Public TUI command-line entrypoint and top-level command dispatch.

use std::env;
use std::process::ExitCode;

use kfnotepad::{parse_args, Command, VERSION};

use super::commands::{
    run_empty_command, run_file_command, run_list_managed_notes_command, run_managed_note_command,
};

pub fn run() -> ExitCode {
    let args: Vec<String> = env::args().skip(1).collect();

    match parse_args(&args) {
        Ok(Command::Help) => {
            print!("{}", kfnotepad::help_text());
            ExitCode::SUCCESS
        }
        Ok(Command::Version) => {
            println!("kfnotepad {VERSION}");
            ExitCode::SUCCESS
        }
        Ok(Command::LaunchEmpty) => run_empty_command(),
        Ok(Command::InspectFile(path)) => run_file_command(&path),
        Ok(Command::ListManagedNotes) => run_list_managed_notes_command(),
        Ok(Command::OpenManagedNote(title)) => run_managed_note_command(&title),
        Err(error) => {
            eprintln!("kfnotepad: {error}");
            eprintln!("Try `kfnotepad --help`.");
            ExitCode::from(2)
        }
    }
}
