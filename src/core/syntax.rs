//! Syntax selection and stateful highlighting backed by Syntect.

#[path = "syntax/highlight.rs"]
mod highlight;
#[path = "syntax/incremental.rs"]
mod incremental;
#[path = "syntax/selection.rs"]
mod selection;
#[path = "syntax/types.rs"]
mod types;

pub use types::{
    SyntaxHighlightCacheState, SyntaxHighlightedLine, SyntaxHighlightedLines, SyntaxHighlighter,
};
