//! Document-level types and cursor, editing, and search operations.

#[path = "text_document/cursor_edit.rs"]
mod cursor_edit;
#[path = "text_document/search.rs"]
mod search;
#[path = "text_document/types.rs"]
mod types;

pub use cursor_edit::*;
pub use search::*;
pub use types::*;
