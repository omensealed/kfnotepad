//! Renderer-neutral syntax highlighter and cache data types.

#[cfg(feature = "syntax")]
use syntect::highlighting::{HighlightState, Style as SyntectStyle, Theme, ThemeSet};
#[cfg(feature = "syntax")]
use syntect::parsing::{ParseState, SyntaxSet};

pub struct SyntaxHighlighter {
    #[cfg(feature = "syntax")]
    pub(super) syntax_set: SyntaxSet,
    #[cfg(feature = "syntax")]
    pub(super) theme: Theme,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SyntaxColor {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SyntaxStyle {
    pub foreground: SyntaxColor,
}

#[cfg(feature = "syntax")]
impl From<SyntectStyle> for SyntaxStyle {
    fn from(style: SyntectStyle) -> Self {
        Self {
            foreground: SyntaxColor {
                r: style.foreground.r,
                g: style.foreground.g,
                b: style.foreground.b,
                a: style.foreground.a,
            },
        }
    }
}

pub type SyntaxHighlightedLine = Option<Vec<(SyntaxStyle, String)>>;
pub type SyntaxHighlightedLines = Vec<SyntaxHighlightedLine>;

#[derive(Clone)]
pub struct SyntaxHighlightCacheState {
    #[cfg(feature = "syntax")]
    pub(super) highlight_state: HighlightState,
    #[cfg(feature = "syntax")]
    pub(super) parse_state: ParseState,
}

impl Default for SyntaxHighlighter {
    fn default() -> Self {
        #[cfg(feature = "syntax")]
        {
            let syntax_set = SyntaxSet::load_defaults_newlines();
            let theme = ThemeSet::load_defaults()
                .themes
                .remove("base16-ocean.dark")
                .unwrap_or_default();
            Self { syntax_set, theme }
        }
        #[cfg(not(feature = "syntax"))]
        Self {}
    }
}
