//! Editor synchronization, clipboard/history, scrolling, input, and pointer interaction.

use super::*;

#[path = "editor_interaction/clipboard_undo.rs"]
mod clipboard_undo;
#[path = "editor_interaction/drag_scrollbar.rs"]
mod drag_scrollbar;
#[path = "editor_interaction/mouse_selection.rs"]
mod mouse_selection;
#[path = "editor_interaction/pane_sync.rs"]
mod pane_sync;
#[path = "editor_interaction/replacement_input.rs"]
mod replacement_input;
#[path = "editor_interaction/scrolling_reader.rs"]
mod scrolling_reader;
