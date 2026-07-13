//! Semantic roles used to normalize source syntax colors across themes.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::gui::app::state::theme) enum GuiSyntaxColorRole {
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
