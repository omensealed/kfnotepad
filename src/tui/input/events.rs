//! Keyboard, mouse, menu, command-palette, save, and quit event dispatch.

use super::*;

include!("events/keyboard.rs");
include!("events/mouse.rs");
include!("events/menu_commands.rs");
mod command_palette;
mod save_quit;

pub(crate) use command_palette::*;
pub(crate) use save_quit::*;
