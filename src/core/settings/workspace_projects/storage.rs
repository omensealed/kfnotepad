//! Workspace project persistence, deterministic listing, and safe deletion.

use super::*;

#[path = "storage/delete.rs"]
mod delete;
#[path = "storage/list.rs"]
mod list;
#[path = "storage/save.rs"]
mod save;

pub use delete::delete_gui_workspace_project;
pub use list::list_gui_workspace_projects;
pub use save::{gui_workspace_project_path, save_gui_workspace_project};
