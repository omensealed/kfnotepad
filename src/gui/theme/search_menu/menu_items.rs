use super::*;

#[path = "menu_items/dispatch.rs"]
mod menu_dispatch;
#[path = "menu_items/file_edit.rs"]
mod menu_file_edit;
#[path = "menu_items/groups.rs"]
mod menu_groups;
#[path = "menu_items/notes_tile_help.rs"]
mod menu_notes_tile_help;
#[path = "menu_items/view_go.rs"]
mod menu_view_go;

pub(in crate::gui::app::state) use menu_dispatch::*;
pub(in crate::gui::app::state) use menu_file_edit::*;
pub(in crate::gui::app::state) use menu_groups::*;
pub(in crate::gui::app::state) use menu_notes_tile_help::*;
pub(in crate::gui::app::state) use menu_view_go::*;
