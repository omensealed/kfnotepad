//! Terminal UI application module.
//!
//! `tui::app` contains the input, update, render, and state-handling
//! logic for the crossterm-based terminal editor flow.
//!
//! Responsibility split:
//! - `app` contains module entry and CLI launch logic.
//! - `input` owns keyboard/mouse command dispatch and event interpretation.
//! - `render` owns terminal layout/render primitives and cursor math.
//! - `sidebar` owns workspace overlay and manager/palette sidebar renderers.
//! - `terminal_session` owns raw terminal setup/teardown and extension flags.
//! - `app::event_loop` wires input, render, and state persistence together.

pub mod app;
mod input;
mod menu;
mod render;
mod sidebar;
pub mod terminal_session;
mod theme;
