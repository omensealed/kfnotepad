//! Search case-sensitivity options.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SearchMode {
    pub case_sensitive: bool,
}
