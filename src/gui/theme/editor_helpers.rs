// Editor viewport, syntax, and replacement-input helpers for the Iced GUI.

#[path = "editor_helpers/viewport.rs"]
mod editor_viewport;
include!("editor_helpers/syntax_colors.rs");
include!("editor_helpers/render_model.rs");
include!("editor_helpers/replacement_edit.rs");
include!("editor_helpers/mouse_layout.rs");
include!("editor_helpers/text_ranges.rs");
include!("editor_helpers/keyboard_inputs.rs");

pub(in crate::gui::app::state) use editor_viewport::*;
