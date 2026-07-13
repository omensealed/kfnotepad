//! GUI theme, menu, and editor-viewport helper functions.
//!
//! These symbols are shared by state and render/update modules.

use super::*;

#[path = "theme/palette.rs"]
mod palette;
include!("theme/editor_helpers.rs");
#[path = "theme/file_tree.rs"]
mod file_tree;
#[path = "theme/search_menu.rs"]
mod search_menu;
#[cfg(test)]
#[path = "theme/test_descriptors.rs"]
mod test_descriptors;
#[path = "theme/widgets.rs"]
mod widgets;

pub(super) use file_tree::*;
pub(super) use palette::*;
pub(super) use search_menu::*;
#[cfg(test)]
pub(super) use test_descriptors::*;
pub(super) use widgets::*;
