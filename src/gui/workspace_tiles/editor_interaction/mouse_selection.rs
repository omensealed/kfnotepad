//! Replacement-editor pointer routing, cursor lookup, and mouse selection.

use super::*;

#[path = "mouse_selection/apply.rs"]
mod apply;
#[path = "mouse_selection/cursor_lookup.rs"]
mod cursor_lookup;
#[path = "mouse_selection/pointer_events.rs"]
mod pointer_events;
#[path = "mouse_selection/view_state.rs"]
mod view_state;
