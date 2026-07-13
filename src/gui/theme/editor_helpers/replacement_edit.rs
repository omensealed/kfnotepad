use super::*;

#[path = "replacement_edit/clipboard.rs"]
mod clipboard;
#[path = "replacement_edit/cursors.rs"]
mod cursors;
#[path = "replacement_edit/input.rs"]
mod input;
#[path = "replacement_edit/selection_text.rs"]
mod selection_text;

pub(in crate::gui::app::state) use self::clipboard::*;
pub(in crate::gui::app::state) use self::cursors::*;
pub(in crate::gui::app::state) use self::input::*;
pub(in crate::gui::app::state) use self::selection_text::*;
