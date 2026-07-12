//! Sidebar scrolling, selection routing, and file activation.

use super::*;

mod mouse_scroll;
mod selection;
mod single_document;
mod workspace_tabs;

pub(crate) use mouse_scroll::*;
pub(crate) use selection::*;
pub(crate) use single_document::*;
pub(crate) use workspace_tabs::*;
