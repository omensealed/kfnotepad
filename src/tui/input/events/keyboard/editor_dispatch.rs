//! Active-editor keyboard dispatch.

use super::*;

mod active_modes;
mod command_shortcuts;
mod edit_keys;
mod entry;
mod movement_keys;

use active_modes::handle_active_editor_mode;
use command_shortcuts::handle_editor_command_shortcut;
use edit_keys::handle_editor_edit_key;
pub(crate) use entry::*;
use movement_keys::handle_editor_movement_key;
