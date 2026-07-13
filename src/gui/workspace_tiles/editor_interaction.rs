//! Editor synchronization, clipboard/history, scrolling, input, and pointer interaction.

use super::*;

include!("editor_interaction/pane_sync.rs");
include!("editor_interaction/clipboard_undo.rs");
include!("editor_interaction/scrolling_reader.rs");
include!("editor_interaction/replacement_input.rs");
include!("editor_interaction/mouse_selection.rs");
include!("editor_interaction/drag_scrollbar.rs");
