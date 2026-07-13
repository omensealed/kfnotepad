//! Semantic roles used to normalize source syntax colors across themes.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum GuiSyntaxColorRole {
    Text,
    Comment,
    Rose,
    Orange,
    Yellow,
    Green,
    Cyan,
    Blue,
    Purple,
}
