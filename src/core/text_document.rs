//! Document-level types and cursor/navigation operations.

use super::{GoToLineResult, SearchMode, SearchRepeatResult, TextBuffer};

include!("text_document/types.rs");
include!("text_document/cursor_edit.rs");
include!("text_document/search.rs");
