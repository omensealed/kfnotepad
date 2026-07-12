//! Terminal setup/teardown helpers for the TUI runtime.

#[path = "terminal_session/backend.rs"]
mod backend;
#[path = "terminal_session/capabilities.rs"]
mod capabilities;
#[path = "terminal_session/session.rs"]
mod session;

pub(crate) use backend::{CrosstermBackend, TerminalBackend};
pub(crate) use capabilities::{editor_keyboard_enhancement_flags, supports_tui_terminal};
pub(crate) use session::TerminalSession;

#[cfg(test)]
#[path = "terminal_session/tests.rs"]
mod tests;
