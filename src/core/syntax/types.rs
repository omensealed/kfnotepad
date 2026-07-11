pub struct SyntaxHighlighter {
    syntax_set: SyntaxSet,
    theme: Theme,
}

pub type SyntaxHighlightedLine = Option<Vec<(SyntectStyle, String)>>;
pub type SyntaxHighlightedLines = Vec<SyntaxHighlightedLine>;

#[derive(Clone)]
pub struct SyntaxHighlightCacheState {
    highlight_state: HighlightState,
    parse_state: ParseState,
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
