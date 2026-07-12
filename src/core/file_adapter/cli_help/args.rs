//! Command-line argument parsing into the shared command model.

use super::*;

pub fn parse_args(args: &[String]) -> Result<Command, String> {
    match args {
        [] => Ok(Command::LaunchEmpty),
        [flag] if flag == "--help" || flag == "-h" => Ok(Command::Help),
        [flag] if flag == "--version" || flag == "-V" => Ok(Command::Version),
        [flag] if flag == "--notes" => Ok(Command::ListManagedNotes),
        [flag, title] if flag == "--note" && title.trim().is_empty() => {
            Err("managed note title must not be empty".to_string())
        }
        [flag, title] if flag == "--note" => Ok(Command::OpenManagedNote(title.clone())),
        [path] if path.starts_with('-') => Err(format!("unknown option: {path}")),
        [path] if path.trim().is_empty() => Err("file path must not be empty".to_string()),
        [path] => Ok(Command::InspectFile(path.clone())),
        _ => Err(
            "expected zero arguments, --help, --version, --notes, --note TITLE, or one file path"
                .to_string(),
        ),
    }
}
