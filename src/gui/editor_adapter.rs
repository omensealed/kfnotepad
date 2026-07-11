//! GUI editor adapter and read-only/editor bridge helpers.
//!
//! This module owns `GuiEditorAdapter` and the selection/cursor/input helper
//! types that keep Iced editor and document synchronization coherent.

use super::*;

include!("editor_adapter/input_method.rs");
include!("editor_adapter/types.rs");
include!("editor_adapter/adapter.rs");
include!("editor_adapter/viewport.rs");
include!("editor_adapter/scrollbar_selection.rs");
