//! Editing, navigation, prompt, search, settings, and reader-mode commands.

use super::*;

include!("editor_commands/editing.rs");
mod modes_and_reader;
mod navigation;
mod prompts;

pub(crate) use modes_and_reader::*;
pub(crate) use navigation::*;
pub(crate) use prompts::*;
