//! Keyboard, mouse, menu, command-palette, save, and quit event dispatch.

use super::*;

include!("events/keyboard.rs");
include!("events/mouse.rs");
include!("events/menu_commands.rs");
include!("events/command_palette.rs");
include!("events/save_quit.rs");
