//! TUI runtime event loop extracted from the monolithic terminal app module.

#[path = "event_loop/dispatch.rs"]
mod dispatch;
#[path = "event_loop/event_read.rs"]
mod event_read;
#[path = "event_loop/frame.rs"]
mod frame;
#[path = "event_loop/run.rs"]
mod run;
#[path = "event_loop/runtime_setup.rs"]
mod runtime_setup;
#[path = "event_loop/types.rs"]
mod types;

pub(crate) use run::run_editor_workspace;
use types::LoopLayout;
