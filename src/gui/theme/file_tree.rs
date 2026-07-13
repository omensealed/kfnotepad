// File-tree model construction, rendering, path validation, and deletion.
use super::*;

#[path = "file_tree/delete.rs"]
mod tree_delete;
#[path = "file_tree/paths.rs"]
mod tree_paths;
#[path = "file_tree/rows.rs"]
mod tree_rows;
#[path = "file_tree/sizing.rs"]
mod tree_sizing;
#[path = "file_tree/styles.rs"]
mod tree_styles;
#[path = "file_tree/view.rs"]
mod tree_view;

pub(in crate::gui::app::state) use tree_delete::*;
pub(in crate::gui::app::state) use tree_paths::*;
pub(in crate::gui::app::state) use tree_rows::*;
pub(in crate::gui::app::state) use tree_sizing::*;
pub(in crate::gui::app::state) use tree_styles::*;
pub(in crate::gui::app::state) use tree_view::*;
