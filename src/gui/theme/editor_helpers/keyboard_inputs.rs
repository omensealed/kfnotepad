use super::*;

#[path = "keyboard_inputs/clipboard.rs"]
mod clipboard;
#[path = "keyboard_inputs/ime.rs"]
mod ime;
#[path = "keyboard_inputs/replacement_keyboard.rs"]
mod replacement_keyboard;
#[path = "keyboard_inputs/text.rs"]
mod text;

pub(in crate::gui::app::state) use self::clipboard::*;
#[cfg(test)]
pub(in crate::gui::app::state) use self::ime::*;
pub(in crate::gui::app::state) use self::replacement_keyboard::*;
pub(in crate::gui::app::state) use self::text::*;
