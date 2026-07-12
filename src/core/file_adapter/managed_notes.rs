//! Managed-note path validation, listing, opening, creation, and deletion.

use super::*;

#[path = "managed_notes/delete.rs"]
mod delete;
#[path = "managed_notes/open_list.rs"]
mod open_list;
#[path = "managed_notes/paths.rs"]
mod paths;

pub use delete::delete_managed_note;
pub use open_list::{list_managed_notes, open_or_create_managed_note};
pub use paths::{managed_note_path, managed_notes_dir, note_slug};
