//! GUI app entrypoint.
//!
//! `gui::app` stays the thin launcher for the Iced GUI, while
//! the bulk of state/update/view/editor logic lives in `gui::state`.

#[path = "state.rs"]
pub(crate) mod state;

pub fn run() -> iced::Result {
    state::run()
}
