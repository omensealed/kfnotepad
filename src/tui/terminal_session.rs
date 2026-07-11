//! Terminal setup/teardown helpers for the TUI runtime.

// Terminal-session helpers are binary-driven; the library target keeps them
// compiled for tests and feature checks.
#![allow(dead_code)]

include!("terminal_session/capabilities.rs");
include!("terminal_session/backend.rs");
include!("terminal_session/session.rs");

#[cfg(test)]
mod tests {
    include!("terminal_session/tests.rs");
}
