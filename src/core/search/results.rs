#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SearchRepeatResult {
    NoPreviousSearch,
    Found { query: String },
    NoMatch { query: String },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum GoToLineResult {
    Empty,
    Invalid,
    OutOfRange { line_number: usize },
    Moved { line_number: usize },
}
