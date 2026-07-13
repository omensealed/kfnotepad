//! Header and transient top-of-window panel composition.

#[path = "top_panels/header.rs"]
mod header;
#[path = "top_panels/startup_help.rs"]
mod startup_help;
#[path = "top_panels/transient_panels.rs"]
mod transient_panels;

pub(super) use header::gui_view_header;
pub(super) use startup_help::startup_help_panel;
pub(super) use transient_panels::{gui_notes_panel, gui_path_prompt_panel};
