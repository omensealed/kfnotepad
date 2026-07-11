//! GUI app entrypoint.
//!
//! `gui::app` stays the thin launcher for the Iced GUI, while
//! the bulk of state/update/view/editor logic lives in `gui::state`.

// GUI internals are binary-driven but compiled through the library target for
// feature checks and tests, which leaves false dead-code positives per target.
#![allow(dead_code)]

#[path = "state.rs"]
pub(crate) mod state;

pub fn run() -> iced::Result {
    state::run()
}
