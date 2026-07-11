use std::path::Path;

use syntect::easy::HighlightLines;
use syntect::highlighting::{HighlightState, Style as SyntectStyle, Theme, ThemeSet};
use syntect::parsing::{ParseState, SyntaxReference, SyntaxSet};

use crate::core::TextDocument;

include!("syntax/types.rs");
include!("syntax/selection.rs");
include!("syntax/highlight.rs");
include!("syntax/incremental.rs");
