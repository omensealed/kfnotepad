//! Terminal UI application orchestration.
//!
//! `tui::app` glues command dispatch, lifecycle entrypoints, and tests to the
//! terminal runtime/event loop that lives in `tui::app::event_loop`.

#[path = "app/commands.rs"]
mod commands;
mod event_loop;

use commands::{
    run_empty_command, run_file_command, run_list_managed_notes_command, run_managed_note_command,
};

include!("app/imports.rs");
include!("app/exports.rs");
include!("app/run.rs");
include!("app/helpers.rs");

#[cfg(test)]
#[path = "app/tests.rs"]
mod tests;
