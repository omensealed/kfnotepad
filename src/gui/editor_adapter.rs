//! GUI editor adapter and read-only/editor bridge helpers.
//!
//! This module owns `GuiEditorAdapter` and the selection/cursor/input helper
//! types that keep Iced editor and document synchronization coherent.

use super::*;

#[path = "editor_adapter/adapter.rs"]
mod adapter;
#[path = "editor_adapter/input_method.rs"]
mod ime_bridge;
#[path = "editor_adapter/scrollbar_selection.rs"]
mod scrollbar_selection;
#[path = "editor_adapter/types.rs"]
mod types;
#[path = "editor_adapter/viewport.rs"]
mod viewport;

pub(crate) use ime_bridge::*;
pub(crate) use scrollbar_selection::*;
pub(crate) use types::*;
pub(crate) use viewport::*;
