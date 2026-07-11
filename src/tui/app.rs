//! Terminal UI application orchestration.
//!
//! `tui::app` glues command dispatch, lifecycle entrypoints, and tests to the
//! terminal runtime/event loop that lives in `tui::app::event_loop`.

mod event_loop;

include!("app/imports.rs");
include!("app/exports.rs");
include!("app/run.rs");
include!("app/commands.rs");
include!("app/helpers.rs");

#[cfg(test)]
#[path = "app/tests.rs"]
mod tests;
