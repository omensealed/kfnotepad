//! GUI workspace construction, tile lifecycle, layout intents, and save errors.

use super::*;

#[path = "gui_workspace/closing.rs"]
mod closing;
#[path = "gui_workspace/constructors.rs"]
mod constructors;
#[path = "gui_workspace/layout_intents.rs"]
mod layout_intents;
#[path = "gui_workspace/open_focus.rs"]
mod open_focus;
#[path = "gui_workspace/save_errors.rs"]
mod save_errors;
#[path = "gui_workspace/types.rs"]
mod types;

pub use types::GuiWorkspace;
