//! Search helpers and navigation result types for document operations.

use super::{Cursor, TextDocument};

#[path = "search/case_insensitive.rs"]
mod case_insensitive;
#[path = "search/grapheme_ranges.rs"]
mod grapheme_ranges;
#[path = "search/mode.rs"]
mod mode;
#[path = "search/navigation.rs"]
mod navigation;
#[path = "search/results.rs"]
mod results;

pub use case_insensitive::*;
pub use grapheme_ranges::expand_range_to_grapheme_boundaries;
pub use mode::SearchMode;
pub(crate) use navigation::next_search_start;
pub use results::*;
