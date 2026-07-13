//! Optional syntax selection and stateful highlighting.

#[cfg(feature = "syntax")]
#[path = "syntax/highlight.rs"]
mod highlight;
#[cfg(feature = "syntax")]
#[path = "syntax/incremental.rs"]
mod incremental;
#[cfg(not(feature = "syntax"))]
#[path = "syntax/plain.rs"]
mod plain;
#[cfg(feature = "syntax")]
#[path = "syntax/selection.rs"]
mod selection;
#[path = "syntax/types.rs"]
mod types;

pub use types::{
    SyntaxColor, SyntaxHighlightCacheState, SyntaxHighlightedLine, SyntaxHighlightedLines,
    SyntaxHighlighter, SyntaxStyle,
};
