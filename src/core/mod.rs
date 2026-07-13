//! Core domain model for `kfnotepad`.
//!
//! This module owns non-UI state and behavior:
//! - text documents and undo state,
//! - workspace and tab descriptors,
//! - search/navigation helpers,
//! - syntax-highlighter state management,
//! - file/settings/workspace I/O primitives.

#[cfg(test)]
mod tests;

mod errors;
mod file_adapter;
mod paths;
mod search;
mod settings;
mod syntax;
mod text_buffer;
mod text_document;
mod workspace;

pub use errors::*;
pub use file_adapter::*;
pub use paths::*;
pub(crate) use search::next_search_start;
pub use search::*;
pub use settings::*;
pub use syntax::{
    SyntaxColor, SyntaxHighlightCacheState, SyntaxHighlightedLine, SyntaxHighlightedLines,
    SyntaxHighlighter, SyntaxStyle,
};
pub use text_buffer::*;
pub use text_document::*;
pub use workspace::*;
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
pub const MAX_TEXT_FILE_BYTES: u64 = 8 * 1024 * 1024;
pub const MAX_UNDO_HISTORY: usize = 256;
pub const MAX_UNDO_BYTES: usize = 64 * 1024 * 1024;
const TYPING_UNDO_COALESCE_WINDOW: std::time::Duration = std::time::Duration::from_millis(750);
