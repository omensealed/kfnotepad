//! Keyboard, mouse, menu, command-palette, save, and quit event dispatch.

use super::*;

include!("events/keyboard.rs");
mod command_palette;
mod menu_commands;
mod mouse;
mod save_quit;

pub(crate) use command_palette::*;
pub(crate) use menu_commands::*;
pub(crate) use mouse::*;
pub(crate) use save_quit::*;
