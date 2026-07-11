//! Runtime state, tab actions, status text, and persisted TUI configuration.

use super::*;

include!("runtime/types.rs");
include!("runtime/defaults.rs");
include!("runtime/tab_actions.rs");
mod config;
mod status;

pub(crate) use config::*;
pub(crate) use status::*;
