//! Read-only replacement editor composition and row rendering.

use super::*;

#[path = "read_only_view/body.rs"]
mod body;
#[path = "read_only_view/line_row.rs"]
mod line_row;
#[path = "read_only_view/main.rs"]
mod main;
#[path = "read_only_view/types.rs"]
mod types;

use body::gui_editor_read_only_body;
use line_row::gui_editor_read_only_line_row;
pub(in crate::gui::app::state) use main::gui_editor_read_only_view;
use self::types::{GuiReadOnlyBodyContext, GuiReadOnlyLineRowContext};
