//! Editing, navigation, prompt, search, settings, and reader-mode commands.

use super::*;

include!("editor_commands/editing.rs");
include!("editor_commands/modes_and_reader.rs");
mod navigation;
mod prompts;

pub(crate) use navigation::*;
pub(crate) use prompts::*;
