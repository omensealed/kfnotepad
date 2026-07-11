//! Terminal UI application orchestration.
//!
//! `tui::app` glues command dispatch, lifecycle entrypoints, and tests to the
//! terminal runtime/event loop that lives in `tui::app::event_loop`.

// The library target compiles TUI internals that are exercised by the binary
// entrypoint and test targets, so rustc's per-target dead-code lint sees false
// positives here.
#![allow(dead_code)]

mod event_loop;

include!("app/imports.rs");
include!("app/exports.rs");
include!("app/run.rs");
include!("app/commands.rs");
include!("app/helpers.rs");
