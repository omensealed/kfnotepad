//! Workspace project formats, storage, path encoding, and name validation.

use super::*;

#[path = "workspace_projects/format.rs"]
mod format;
#[path = "workspace_projects/hex_paths.rs"]
mod hex_paths;
#[path = "workspace_projects/slug.rs"]
mod slug;
#[path = "workspace_projects/storage.rs"]
mod storage;

pub use format::{parse_gui_workspace_project, serialize_gui_workspace_project};
use hex_paths::{bytes_to_hex, hex_to_bytes};
pub(crate) use hex_paths::{path_from_hex, path_to_hex};
use slug::gui_workspace_project_slug;
pub use storage::*;
