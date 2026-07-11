#![forbid(unsafe_code)]

extern crate self as kfnotepad;

pub mod core;
#[cfg(feature = "gui")]
pub mod gui;
#[cfg(feature = "tui")]
pub mod tui;

pub use core::*;
