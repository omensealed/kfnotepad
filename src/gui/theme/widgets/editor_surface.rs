use super::*;

#[path = "editor_surface/read_only_view.rs"]
mod editor_read_only;
#[path = "editor_surface/scrollbar.rs"]
mod editor_scrollbar;
#[path = "editor_surface/spans.rs"]
mod editor_spans;

pub(in crate::gui::app::state) use editor_read_only::*;
pub(in crate::gui::app::state) use editor_scrollbar::*;
pub(in crate::gui::app::state) use editor_spans::*;
