//! Terminal UI application orchestration.
//!
//! `tui::app` glues command dispatch, lifecycle entrypoints, and tests to the
//! terminal runtime/event loop that lives in `tui::app::event_loop`.

#[path = "app/commands.rs"]
mod commands;
mod event_loop;
#[path = "app/helpers.rs"]
mod helpers;
#[path = "app/run.rs"]
mod run;

use helpers::{
    current_managed_notes_dir, has_tui_terminal, maybe_print_tui_unavailable, run_editor,
};

pub use run::run;

#[cfg(test)]
pub(crate) use crate::tui::theme::EditorTheme;
pub(crate) use event_loop::run_editor_workspace;

pub(crate) const SIDEBAR_WIDTH: usize = 22;

#[cfg(test)]
#[path = "app/tests.rs"]
mod tests;
