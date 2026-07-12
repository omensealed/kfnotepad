//! Public syntax highlighter and cache data types.

use syntect::highlighting::{HighlightState, Style as SyntectStyle, Theme, ThemeSet};
use syntect::parsing::{ParseState, SyntaxSet};

pub struct SyntaxHighlighter {
    pub(super) syntax_set: SyntaxSet,
    pub(super) theme: Theme,
}

pub type SyntaxHighlightedLine = Option<Vec<(SyntectStyle, String)>>;
pub type SyntaxHighlightedLines = Vec<SyntaxHighlightedLine>;

#[derive(Clone)]
pub struct SyntaxHighlightCacheState {
    pub(super) highlight_state: HighlightState,
    pub(super) parse_state: ParseState,
}

impl Default for SyntaxHighlighter {
    fn default() -> Self {
        let syntax_set = SyntaxSet::load_defaults_newlines();
        let theme = ThemeSet::load_defaults()
            .themes
            .remove("base16-ocean.dark")
            .unwrap_or_default();
        Self { syntax_set, theme }
    }
}
