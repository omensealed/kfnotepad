//! CLI command handlers for TUI launch and noninteractive fallbacks.

#[path = "commands/empty.rs"]
mod empty;
#[path = "commands/file.rs"]
mod file;
#[path = "commands/managed_list.rs"]
mod managed_list;
#[path = "commands/managed_note.rs"]
mod managed_note;

pub(super) use empty::run_empty_command;
pub(super) use file::run_file_command;
pub(super) use managed_list::run_list_managed_notes_command;
pub(super) use managed_note::run_managed_note_command;
