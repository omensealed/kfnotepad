//! Undo/redo history, edit grouping, and document search behavior.

use super::*;

#[path = "undo_search/history.rs"]
mod history;
#[path = "undo_search/search.rs"]
mod search;
#[path = "undo_search/undo_redo.rs"]
mod undo_redo;
