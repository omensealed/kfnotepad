// Editor viewport, syntax, and replacement-input helpers for the Iced GUI.

#[path = "editor_helpers/viewport.rs"]
mod editor_viewport;
#[path = "editor_helpers/syntax_colors.rs"]
mod syntax_colors;
include!("editor_helpers/render_model.rs");
include!("editor_helpers/replacement_edit.rs");
#[path = "editor_helpers/mouse_layout.rs"]
mod mouse_layout;
#[path = "editor_helpers/text_ranges.rs"]
mod text_ranges;
#[path = "editor_helpers/keyboard_inputs.rs"]
mod keyboard_inputs;

pub(in crate::gui::app::state) use editor_viewport::*;
pub(in crate::gui::app::state) use keyboard_inputs::*;
pub(in crate::gui::app::state) use mouse_layout::*;
pub(in crate::gui::app::state) use syntax_colors::*;
pub(in crate::gui::app::state) use text_ranges::*;
