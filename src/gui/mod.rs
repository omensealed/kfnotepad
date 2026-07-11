//! GUI application module.
//!
//! `gui` is organized into a thin launcher (`app`) and supporting modules:
//! - `state` for shared application state construction and behavior methods,
//! - `update` for event transitions,
//! - `view` for rendering and styling,
//! - domain helper modules for dialogs, file browser, and preferences.

pub mod app;
